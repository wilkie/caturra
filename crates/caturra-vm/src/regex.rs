//! A backtracking regular-expression engine over UTF-16 code units, following
//! `java.util.regex` semantics.
//!
//! caturra needs this because `String.split` takes a *regex*, not a literal —
//! before this module it split on the literal argument, so `"1.2.3".split("\\.")`
//! returned one element instead of three. That was a silent wrong answer in one
//! of the most-used String methods, masked whenever the delimiter happened to be
//! regex-inert (`split(",")`), where literal and regex semantics coincide.
//!
//! **Why hand-written rather than the `regex` crate.** Two reasons. The engine
//! must be faithful to `java.util.regex`, which is a backtracking engine with
//! backreferences and reluctant/possessive quantifiers — constructs the `regex`
//! crate deliberately does not have. And caturra ships to a browser, where the
//! crate's automata tables are a large payload for a feature students use on
//! short strings. Matching over `&[u16]` also avoids re-encoding: caturra
//! already stores Java strings as UTF-16 units, exactly what Java's own matcher
//! walks.
//!
//! Scope: the constructs that appear in real Java source. Unsupported syntax is
//! reported as a `PatternSyntaxException`, never silently mis-parsed — the
//! failure mode this module exists to remove.

/// A parsed pattern, ready to match.
#[derive(Debug, Clone)]
pub struct Regex {
    node: Node,
    /// Capturing groups, excluding group 0 (the whole match).
    group_count: usize,
    /// `(?<name>X)` names, and the group each stands for.
    names: Vec<(String, usize)>,
}

/// A syntax error, carrying what Java's `PatternSyntaxException` reports.
#[derive(Debug, Clone)]
pub struct SyntaxError {
    pub description: String,
    /// Where the parser stopped, as a JDK reports it: the cursor MINUS one, so
    /// a pattern that is nothing but `)` reports -1 — an index before the
    /// pattern begins, which is why this is signed.
    pub index: isize,
    pub pattern: String,
}

impl SyntaxError {
    /// Java's `PatternSyntaxException.getMessage()` layout: description, the
    /// pattern, and a caret under the offending index.
    pub fn message(&self) -> String {
        // A NEGATIVE index is left out of the message entirely, as a JDK
        // leaves it out: `Pattern.compile(")")` reports the description and
        // the pattern, and nothing about where.
        let Ok(index) = usize::try_from(self.index) else {
            return format!("{}\n{}", self.description, self.pattern);
        };
        let head = format!(
            "{} near index {}\n{}",
            self.description, index, self.pattern
        );
        // The caret line only appears when the index points INTO the pattern:
        // a JDK writes no caret for an error at the very end ("a(b" is
        // unclosed at index 3, which is past its last character).
        if index >= self.pattern.chars().count() {
            return head;
        }
        format!("{head}\n{}^", " ".repeat(index))
    }
}

#[derive(Debug, Clone)]
enum Node {
    /// Matches at the current position without consuming.
    Empty,
    Literal(u16),
    /// `.` — any character except a line terminator.
    AnyChar,
    /// `.` under `(?s)` (DOTALL) — any character at all.
    AnyCharDotAll,
    Class(CharClass),
    Concat(Vec<Node>),
    Alt(Vec<Node>),
    Repeat {
        node: Box<Node>,
        min: u32,
        max: Option<u32>,
        kind: RepeatKind,
    },
    Group {
        /// `None` for `(?:...)`, which does not capture.
        index: Option<usize>,
        node: Box<Node>,
    },
    BackRef {
        index: usize,
        /// Under `(?i)` a backreference compares case-insensitively too.
        fold: bool,
    },
    /// `^`
    Start,
    /// `$`
    End,
    /// `^` under `(?m)` (MULTILINE) — the start of any line.
    LineStart,
    /// `$` under `(?m)` — the end of any line.
    LineEnd,
    /// `\b` (true) and `\B` (false).
    WordBoundary(bool),
    /// `\A`
    InputStart,
    /// `\z`
    InputEnd,
    /// `\Z` — end of input, but before a final line terminator.
    InputEndBeforeFinalTerminator,
    /// `(?=X)`, `(?!X)`, `(?<=X)`, `(?<!X)` — match without consuming.
    Look {
        direction: Look,
        negated: bool,
        node: Box<Node>,
    },
}

/// Which way a lookaround looks, and for a lookbehind how far back its body
/// can reach — `None` for "as far as the input goes", which Java allows
/// (`(?<=a*)b` compiles and matches) though it is often assumed not to.
#[derive(Debug, Clone, Copy)]
enum Look {
    Ahead,
    Behind(Width),
}

/// The widest match a lookbehind body can make: a count, or `Unbounded` for
/// one that can reach as far as the input goes. `max_width` answers `None`
/// instead when the width is not knowable at all — Java's "obvious maximum
/// length" complaint, which applies to a backreference and only to one: its
/// width is whatever some other group captured at run time.
#[derive(Debug, Clone, Copy)]
enum Width {
    Fixed(usize),
    Unbounded,
}

impl Width {
    fn combine(self, other: Width, join: impl Fn(usize, usize) -> usize) -> Width {
        match (self, other) {
            (Width::Fixed(a), Width::Fixed(b)) => Width::Fixed(join(a, b)),
            _ => Width::Unbounded,
        }
    }
}

fn max_width(node: &Node) -> Option<Width> {
    match node {
        // A lookaround inside a lookbehind consumes nothing itself.
        Node::Empty
        | Node::Start
        | Node::End
        | Node::WordBoundary(_)
        | Node::InputStart
        | Node::InputEnd
        | Node::InputEndBeforeFinalTerminator
        | Node::LineStart
        | Node::LineEnd
        | Node::Look { .. } => Some(Width::Fixed(0)),
        Node::Literal(_) | Node::AnyChar | Node::AnyCharDotAll | Node::Class(_) => {
            Some(Width::Fixed(1))
        }
        Node::Concat(nodes) => nodes.iter().try_fold(Width::Fixed(0), |total, node| {
            Some(total.combine(max_width(node)?, |a, b| a + b))
        }),
        Node::Alt(branches) => branches.iter().try_fold(Width::Fixed(0), |widest, node| {
            Some(widest.combine(max_width(node)?, usize::max))
        }),
        Node::Group { node, .. } => max_width(node),
        Node::Repeat { node, max, .. } => match (max_width(node)?, *max) {
            (Width::Fixed(width), Some(max)) => Some(Width::Fixed(width * max as usize)),
            _ => Some(Width::Unbounded),
        },
        Node::BackRef { .. } => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RepeatKind {
    /// `X*` — take as much as possible, give back on failure.
    Greedy,
    /// `X*?` — take as little as possible.
    Reluctant,
    /// `X*+` — take as much as possible and never give back.
    Possessive,
}

#[derive(Debug, Clone)]
struct CharClass {
    /// Set by `(?i)`: this class also matches the other case of an ASCII
    /// letter.
    fold: bool,
    negated: bool,
    items: Vec<ClassItem>,
    /// `[a-z&&[^bc]]` — every intersected class must also match.
    intersections: Vec<CharClass>,
}

#[derive(Debug, Clone)]
enum ClassItem {
    Single(u16),
    Range(u16, u16),
    /// A predefined class such as `\d`, usable inside `[...]` too.
    Predefined(Predefined),
    Nested(CharClass),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Predefined {
    Digit,
    NotDigit,
    Space,
    NotSpace,
    Word,
    NotWord,
}

/// Java's `\s` is exactly these six, NOT Unicode whitespace.
/// A letter Java accepts in an inline flag group.
fn is_flag_letter(unit: u16) -> bool {
    matches!(
        u8::try_from(unit),
        Ok(b'i' | b's' | b'm' | b'u' | b'U' | b'd' | b'x')
    )
}

fn is_java_space(unit: u32) -> bool {
    matches!(unit, 0x20 | 0x09 | 0x0A | 0x0B | 0x0C | 0x0D)
}

fn is_java_digit(unit: u32) -> bool {
    (0x30..=0x39).contains(&unit)
}

/// Java's `\w` is `[a-zA-Z_0-9]` — ASCII only.
fn is_java_word(unit: u32) -> bool {
    is_java_digit(unit)
        || (0x41..=0x5A).contains(&unit)
        || (0x61..=0x7A).contains(&unit)
        || unit == 0x5F
}

/// The line terminators `.` refuses to match.
fn is_line_terminator(unit: u32) -> bool {
    matches!(unit, 0x0A | 0x0D | 0x85 | 0x2028 | 0x2029)
}

impl Predefined {
    fn matches(self, unit: u32) -> bool {
        match self {
            Predefined::Digit => is_java_digit(unit),
            Predefined::NotDigit => !is_java_digit(unit),
            Predefined::Space => is_java_space(unit),
            Predefined::NotSpace => !is_java_space(unit),
            Predefined::Word => is_java_word(unit),
            Predefined::NotWord => !is_java_word(unit),
        }
    }
}

/// The other case of an ASCII letter, or the unit unchanged. Java's
/// `CASE_INSENSITIVE` without `UNICODE_CASE` folds ASCII and nothing else.
fn ascii_fold(unit: u32) -> u32 {
    match u8::try_from(unit) {
        Ok(byte) if byte.is_ascii_alphabetic() => u32::from(byte ^ 0x20),
        _ => unit,
    }
}

impl CharClass {
    fn matches(&self, unit: u32) -> bool {
        if self.fold {
            let other = ascii_fold(unit);
            if other != unit && self.matches_exactly(other) != self.negated {
                // Fold BEFORE negating: `(?i)[^a]` must reject `A`, and
                // negating each case separately would accept it.
                return !self.negated;
            }
        }
        self.matches_exactly(unit)
    }

    /// `matches` without the case folding, negation included.
    fn matches_exactly(&self, unit: u32) -> bool {
        let mut hit = self.items.iter().any(|item| match item {
            ClassItem::Single(single) => u32::from(*single) == unit,
            ClassItem::Range(low, high) => u32::from(*low) <= unit && unit <= u32::from(*high),
            ClassItem::Predefined(predefined) => predefined.matches(unit),
            ClassItem::Nested(nested) => nested.matches(unit),
        });
        // The INTERSECTION binds tighter than the negation: `[^a-c&&[^b]]` is
        // "not (in a-c and not b)", which `b` satisfies. Negating first made it
        // "(not a-c) and not b", which nothing in `a-c` can satisfy — so every
        // negated intersection answered false.
        hit = hit && self.intersections.iter().all(|other| other.matches(unit));
        if self.negated {
            hit = !hit;
        }
        hit
    }
}

// ---------------------------------------------------------------------------
// Parsing
// ---------------------------------------------------------------------------

struct Parser<'a> {
    units: &'a [u16],
    at: usize,
    groups: usize,
    /// `(?<name>X)` — each name and the group number it stands for, so
    /// `\k<name>` and a `${name}` replacement can find it.
    names: Vec<(String, usize)>,
    pattern: String,
    /// The inline flags in force here. `(?i)` applies from where it appears
    /// to the end of the ENCLOSING group, so each group saves and restores
    /// this on the way in and out.
    flags: Flags,
}

/// The inline flags caturra models: `(?i)`, `(?s)`, `(?m)`. Java's `u`, `d`,
/// `x` and `U` parse and are accepted where they do not change what matches
/// for the input this engine sees; anything else is refused rather than
/// silently ignored.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools)] // Java has exactly these four switches
struct Flags {
    /// `i` — `CASE_INSENSITIVE`. Without `u` this is ASCII-only folding,
    /// which is exactly Java's rule for `(?i)` on its own.
    fold: bool,
    /// `s` — `DOTALL`: `.` matches a line terminator too.
    dotall: bool,
    /// `m` — `MULTILINE`: `^` and `$` match at every line boundary.
    multiline: bool,
    /// `x` — `COMMENTS`: unescaped whitespace and `#` to end of line are not
    /// part of the pattern. This changes LEXING rather than matching, so it is
    /// applied when reading each token instead of baked into a node.
    comments: bool,
}

type ParseResult<T> = Result<T, SyntaxError>;

impl Parser<'_> {
    fn error(&self, description: &str, index: usize) -> SyntaxError {
        self.error_at(description, isize::try_from(index).unwrap_or(0))
    }

    /// The same, for the one site whose index can be NEGATIVE — a `)` in first
    /// position, which a JDK reports at -1.
    fn error_at(&self, description: &str, index: isize) -> SyntaxError {
        SyntaxError {
            description: description.to_owned(),
            index,
            pattern: self.pattern.clone(),
        }
    }

    fn peek(&self) -> Option<u16> {
        self.units.get(self.at).copied()
    }

    fn next(&mut self) -> Option<u16> {
        let unit = self.peek();
        if unit.is_some() {
            self.at += 1;
        }
        unit
    }

    fn eat(&mut self, unit: u16) -> bool {
        if self.peek() == Some(unit) {
            self.at += 1;
            return true;
        }
        false
    }

    /// alternation := concat ( '|' concat )*
    fn parse_alt(&mut self) -> ParseResult<Node> {
        let mut branches = vec![self.parse_concat()?];
        while self.eat(u16::from(b'|')) {
            branches.push(self.parse_concat()?);
        }
        if branches.len() == 1 {
            return Ok(branches.pop().expect("one branch"));
        }
        Ok(Node::Alt(branches))
    }

    /// concat := quantified*
    /// Under `(?x)` whitespace and `# comment` are not part of the pattern.
    /// Java ignores them inside a character class and inside `{n,m}` bounds
    /// too, so this is called wherever a token is about to be read — an
    /// ESCAPED space (`\ `) never reaches here, and stays a literal.
    fn skip_ignorable(&mut self) {
        if !self.flags.comments {
            return;
        }
        while let Some(unit) = self.units.get(self.at).copied() {
            if matches!(
                u8::try_from(unit),
                Ok(b' ' | b'\t' | b'\n' | b'\r' | 0x0B | 0x0C)
            ) {
                self.at += 1;
                continue;
            }
            if unit == u16::from(b'#') {
                while let Some(unit) = self.units.get(self.at).copied() {
                    if is_line_terminator(u32::from(unit)) {
                        break;
                    }
                    self.at += 1;
                }
                continue;
            }
            break;
        }
    }

    fn parse_concat(&mut self) -> ParseResult<Node> {
        let mut nodes = Vec::new();
        loop {
            self.skip_ignorable();
            let Some(unit) = self.peek() else { break };
            if unit == u16::from(b'|') || unit == u16::from(b')') {
                break;
            }
            nodes.push(self.parse_quantified()?);
        }
        match nodes.len() {
            0 => Ok(Node::Empty),
            1 => Ok(nodes.pop().expect("one node")),
            _ => Ok(Node::Concat(nodes)),
        }
    }

    /// quantified := atom quantifier?
    fn parse_quantified(&mut self) -> ParseResult<Node> {
        let start = self.at;
        let atom = self.parse_atom()?;
        self.skip_ignorable();
        let (min, max) = match self.peek() {
            Some(unit) if unit == u16::from(b'*') => {
                self.at += 1;
                (0, None)
            }
            Some(unit) if unit == u16::from(b'+') => {
                self.at += 1;
                (1, None)
            }
            Some(unit) if unit == u16::from(b'?') => {
                self.at += 1;
                (0, Some(1))
            }
            Some(unit) if unit == u16::from(b'{') => match self.parse_bounds()? {
                Some(bounds) => bounds,
                // A `{` after a quantifiable atom must open a repetition: the
                // JDK's `closure` says "Illegal repetition" for anything else.
                // (A `{` in atom position IS a literal brace, which
                // `parse_atom` handles.) Treating this one as a literal
                // accepted `a{x`, which no JDK compiles.
                None => return Err(self.error("Illegal repetition", start)),
            },
            _ => return Ok(atom),
        };
        // A quantifier must follow something quantifiable.
        if matches!(atom, Node::Start | Node::End | Node::WordBoundary(_)) {
            // The JDK names the offending character: "Dangling meta
            // character '+' near index 0".
            let meta = self
                .units
                .get(self.at.saturating_sub(1))
                .and_then(|u| char::from_u32(u32::from(*u)))
                .unwrap_or('?');
            return Err(self.error(&format!("Dangling meta character '{meta}'"), start));
        }
        let kind = if self.eat(u16::from(b'?')) {
            RepeatKind::Reluctant
        } else if self.eat(u16::from(b'+')) {
            RepeatKind::Possessive
        } else {
            RepeatKind::Greedy
        };
        if max.is_some_and(|max| max < min) {
            return Err(self.error("Illegal repetition range", self.at.saturating_sub(1)));
        }
        // `\Qab\E+` — a quoted run expands to a SEQUENCE of literals, and the
        // quantifier binds only its LAST character (so the pattern is `a`
        // followed by one-or-more `b`). Repeating the whole run made `abb`
        // fail and `abab` match, each the opposite of a JDK.
        let atom = match atom {
            Node::Concat(mut nodes) if !nodes.is_empty() => {
                let last = nodes.pop().expect("non-empty");
                nodes.push(Node::Repeat {
                    node: Box::new(last),
                    min,
                    max,
                    kind,
                });
                return Ok(Node::Concat(nodes));
            }
            other => other,
        };
        Ok(Node::Repeat {
            node: Box::new(atom),
            min,
            max,
            kind,
        })
    }

    /// `{n}`, `{n,}`, `{n,m}`. `None` when the brace does not open a bound,
    /// which Java treats as a literal `{`.
    fn parse_bounds(&mut self) -> ParseResult<Option<(u32, Option<u32>)>> {
        let open = self.at;
        self.at += 1; // consume '{'
        let mut min_digits = String::new();
        self.skip_ignorable();
        while let Some(unit) = self.peek() {
            if !is_java_digit(u32::from(unit)) {
                break;
            }
            min_digits.push(char::from(unit as u8));
            self.at += 1;
        }
        if min_digits.is_empty() {
            self.at = open;
            return Ok(None);
        }
        let min = min_digits
            .parse::<u32>()
            .map_err(|_| self.error("Illegal repetition range", open))?;
        self.skip_ignorable();
        if self.eat(u16::from(b'}')) {
            return Ok(Some((min, Some(min))));
        }
        if !self.eat(u16::from(b',')) {
            // Digits, then neither `}` nor `,`: the closure was OPENED and
            // never closed, which a JDK reports at the character that should
            // have closed it (or at the end of the pattern).
            return Err(self.error("Unclosed counted closure", self.at));
        }
        self.skip_ignorable();
        if self.eat(u16::from(b'}')) {
            return Ok(Some((min, None)));
        }
        let mut max_digits = String::new();
        while let Some(unit) = self.peek() {
            if !is_java_digit(u32::from(unit)) {
                break;
            }
            max_digits.push(char::from(unit as u8));
            self.at += 1;
        }
        self.skip_ignorable();
        if max_digits.is_empty() || !self.eat(u16::from(b'}')) {
            return Err(self.error("Unclosed counted closure", self.at));
        }
        let max = max_digits
            .parse::<u32>()
            .map_err(|_| self.error("Illegal repetition range", open))?;
        Ok(Some((min, Some(max))))
    }

    fn parse_atom(&mut self) -> ParseResult<Node> {
        let start = self.at;
        let Some(unit) = self.next() else {
            return Ok(Node::Empty);
        };
        match unit {
            u if u == u16::from(b'.') => Ok(if self.flags.dotall {
                Node::AnyCharDotAll
            } else {
                Node::AnyChar
            }),
            u if u == u16::from(b'^') => Ok(if self.flags.multiline {
                Node::LineStart
            } else {
                Node::Start
            }),
            u if u == u16::from(b'$') => Ok(if self.flags.multiline {
                Node::LineEnd
            } else {
                Node::End
            }),
            u if u == u16::from(b'(') => self.parse_group(start),
            u if u == u16::from(b'[') => Ok(Node::Class(self.parse_class(start)?)),
            u if u == u16::from(b'\\') => self.parse_escape(start),
            // The JDK's `error` reports the cursor it stopped at, one before
            // the character it just read — for `a)` that is index 0, not the
            // `)`'s own index.
            u if u == u16::from(b')') => Err(self.error_at(
                "Unmatched closing ')'",
                isize::try_from(start).unwrap_or(0) - 1,
            )),
            u if u == u16::from(b'*') || u == u16::from(b'+') || u == u16::from(b'?') => {
                let meta = char::from_u32(u32::from(u)).unwrap_or('?');
                Err(self.error(&format!("Dangling meta character '{meta}'"), start))
            }
            other => Ok(self.literal(other)),
        }
    }

    /// One literal character, as a fold-aware node. Under `(?i)` it becomes a
    /// one-item class that also accepts the other case — so folding lives in
    /// ONE place and reaches `\Q...\E` and the escapes for free.
    fn literal(&self, unit: u16) -> Node {
        if self.flags.fold && ascii_fold(u32::from(unit)) != u32::from(unit) {
            return Node::Class(CharClass {
                fold: true,
                negated: false,
                items: vec![ClassItem::Single(unit)],
                intersections: Vec::new(),
            });
        }
        Node::Literal(unit)
    }

    fn parse_group(&mut self, open: usize) -> ParseResult<Node> {
        let mut index = None;
        if self.eat(u16::from(b'?')) {
            // Lookaround, then the plain non-capturing group. Anything else
            // (a named group, a flag setting) is refused rather than silently
            // ignored — a mis-parsed pattern is a wrong answer.
            let look = if self.eat(u16::from(b'=')) {
                Some((false, false))
            } else if self.eat(u16::from(b'!')) {
                Some((false, true))
            } else if self.peek() == Some(u16::from(b'<')) {
                let after = self.units.get(self.at + 1).copied();
                match after {
                    Some(unit) if unit == u16::from(b'=') || unit == u16::from(b'!') => {
                        self.at += 2;
                        Some((true, unit == u16::from(b'!')))
                    }
                    // Neither `=` nor `!` after `<`: a NAMED group.
                    _ => {
                        self.at += 1;
                        let name = self.parse_group_name(open)?;
                        self.groups += 1;
                        let group = self.groups;
                        if self.names.iter().any(|(taken, _)| *taken == name) {
                            return Err(self.error(
                                &format!("Named capturing group <{name}> is already defined"),
                                self.at.saturating_sub(1),
                            ));
                        }
                        self.names.push((name, group));
                        let saved = self.flags;
                        let node = self.parse_alt()?;
                        self.flags = saved;
                        if !self.eat(u16::from(b')')) {
                            return Err(self.error("Unclosed group", self.units.len()));
                        }
                        return Ok(Node::Group {
                            index: Some(group),
                            node: Box::new(node),
                        });
                    }
                }
            } else {
                None
            };
            if look.is_none()
                && self
                    .peek()
                    .is_some_and(|unit| is_flag_letter(unit) || unit == u16::from(b'-'))
            {
                return self.parse_flags(open);
            }
            if let Some((behind, negated)) = look {
                let node = self.parse_alt()?;
                if !self.eat(u16::from(b')')) {
                    return Err(self.error("Unclosed group", self.units.len()));
                }
                let direction = if behind {
                    // Java refuses only a lookbehind whose width is not
                    // knowable at all — a backreference. An UNBOUNDED body
                    // (`(?<=a*)b`) compiles and matches, so the bound here is
                    // an optimisation of the backwards search, not a rule.
                    let Some(width) = max_width(&node) else {
                        // The JDK points at the body's LAST character, one
                        // before the closing paren `self.at` has just passed.
                        return Err(self.error(
                            "Look-behind group does not have an obvious maximum length",
                            self.at.saturating_sub(2),
                        ));
                    };
                    Look::Behind(width)
                } else {
                    Look::Ahead
                };
                return Ok(Node::Look {
                    direction,
                    negated,
                    node: Box::new(node),
                });
            }
            if !self.eat(u16::from(b':')) {
                // `(?>` is an ATOMIC group — real Java syntax this engine does
                // not implement, and saying so is honest. Anything else after
                // `(?` is a modifier a JDK does not know either, reported at
                // the offending character (or at the end, if there is none).
                let unsupported = self.peek() == Some(u16::from(b'>'));
                let description = if unsupported {
                    "Unsupported group construct"
                } else {
                    "Unknown inline modifier"
                };
                let at = if unsupported { open } else { self.at };
                return Err(self.error(description, at));
            }
        } else {
            self.groups += 1;
            index = Some(self.groups);
        }
        // `(?i)` reaches the end of the group it sits in and no further.
        let saved = self.flags;
        let node = self.parse_alt()?;
        self.flags = saved;
        if !self.eat(u16::from(b')')) {
            // At END of input the JDK's cursor is one past the last character.
            return Err(self.error("Unclosed group", self.units.len()));
        }
        Ok(Node::Group {
            index,
            node: Box::new(node),
        })
    }

    /// A group name: a letter, then letters and digits, then `>`. Java's
    /// rules exactly — anything else is a syntax error rather than a group
    /// that silently never matches.
    fn parse_group_name(&mut self, open: usize) -> ParseResult<String> {
        let mut name = String::new();
        loop {
            let Some(unit) = self.peek() else {
                // Ran out inside the name: with nothing read yet the JDK
                // complains about the first character, and with a name in
                // hand about the `>` that never came.
                let _ = open;
                let description = if name.is_empty() {
                    "capturing group name does not start with a Latin letter"
                } else {
                    "named capturing group is missing trailing '>'"
                };
                return Err(self.error(description, self.units.len()));
            };
            if unit == u16::from(b'>') {
                if name.is_empty() {
                    // An EMPTY name is the same complaint as a bad first
                    // character, reported at the `>`.
                    return Err(self.error(
                        "capturing group name does not start with a Latin letter",
                        self.at,
                    ));
                }
                self.at += 1;
                break;
            }
            let byte = u8::try_from(unit).unwrap_or(0);
            let first = name.is_empty();
            if first && !byte.is_ascii_alphabetic() {
                // Java's wording, and it covers the EMPTY name too — `(?<>a)`
                // reports this at the `>`, not a separate "0 length" error.
                return Err(self.error(
                    "capturing group name does not start with a Latin letter",
                    self.at,
                ));
            }
            if !(byte.is_ascii_alphabetic() || byte.is_ascii_digit()) {
                return Err(self.error("named capturing group is missing trailing '>'", self.at));
            }
            name.push(char::from(byte));
            self.at += 1;
        }
        Ok(name)
    }

    /// `(?i)`, `(?im-sx)` — set flags for the rest of the enclosing group —
    /// and `(?i:X)`, which scopes them to `X`. The cursor sits on the first
    /// flag letter (or the `-`).
    fn parse_flags(&mut self, open: usize) -> ParseResult<Node> {
        let saved = self.flags;
        let mut flags = self.flags;
        let mut clearing = false;
        loop {
            let Some(unit) = self.peek() else {
                // `(?i` — the flags ran into the end of the pattern, which a
                // JDK reports as an unknown modifier rather than an unclosed
                // group.
                return Err(self.error("Unknown inline modifier", self.units.len()));
            };
            if unit == u16::from(b'-') {
                self.at += 1;
                clearing = true;
                continue;
            }
            if !is_flag_letter(unit) {
                break;
            }
            self.at += 1;
            let on = !clearing;
            match u8::try_from(unit).unwrap_or(0) {
                b'i' => flags.fold = on,
                b's' => flags.dotall = on,
                b'm' => flags.multiline = on,
                // `u`/`U` (Unicode case and character classes), `d` (UNIX
                // lines) and `x` (comments) parse and are accepted: for the
                // patterns this engine sees they select behaviour it already
                // has. `x` genuinely changes parsing, so it is refused.
                b'x' => flags.comments = on,
                b'u' | b'U' | b'd' => {}
                _ => return Err(self.error("Unsupported group construct", open)),
            }
        }
        if self.eat(u16::from(b')')) {
            // Set for the REST of the enclosing group: the caller's parse
            // continues with these flags, and its own group restores them.
            self.flags = flags;
            return Ok(Node::Empty);
        }
        if !self.eat(u16::from(b':')) {
            let _ = open;
            return Err(self.error("Unknown inline modifier", self.at));
        }
        self.flags = flags;
        let node = self.parse_alt()?;
        self.flags = saved;
        if !self.eat(u16::from(b')')) {
            return Err(self.error("Unclosed group", self.units.len()));
        }
        Ok(Node::Group {
            index: None,
            node: Box::new(node),
        })
    }

    fn parse_class(&mut self, open: usize) -> ParseResult<CharClass> {
        let mut class = CharClass {
            fold: self.flags.fold,
            negated: self.eat(u16::from(b'^')),
            items: Vec::new(),
            intersections: Vec::new(),
        };
        // A `]` in first position is a literal, not the terminator.
        if self.eat(u16::from(b']')) {
            class.items.push(ClassItem::Single(u16::from(b']')));
        }
        loop {
            self.skip_ignorable();
            let Some(unit) = self.peek() else {
                // Ran out of input: a JDK reports the cursor minus one, which
                // is the pattern's LAST character — not the `[` that opened
                // the class.
                let _ = open;
                return Err(self.error("Unclosed character class", self.at.saturating_sub(1)));
            };
            if unit == u16::from(b']') {
                self.at += 1;
                return Ok(class);
            }
            // `&&` intersects with the class that follows.
            if unit == u16::from(b'&') && self.units.get(self.at + 1) == Some(&u16::from(b'&')) {
                self.at += 2;
                let nested_open = self.at;
                let nested = if self.eat(u16::from(b'[')) {
                    self.parse_class(nested_open)?
                } else {
                    // `[a-z&&b]` — the right side is a bare item sequence.
                    let mut items = Vec::new();
                    while let Some(next) = self.peek() {
                        if next == u16::from(b']') {
                            break;
                        }
                        items.push(self.parse_class_item()?);
                    }
                    CharClass {
                        fold: self.flags.fold,
                        negated: false,
                        items,
                        intersections: Vec::new(),
                    }
                };
                class.intersections.push(nested);
                continue;
            }
            class.items.push(self.parse_class_item()?);
        }
    }

    fn parse_class_item(&mut self) -> ParseResult<ClassItem> {
        let start = self.at;
        let Some(unit) = self.next() else {
            let _ = start;
            return Err(self.error("Unclosed character class", self.at.saturating_sub(1)));
        };
        // A nested class: `[a-d[m-p]]`.
        if unit == u16::from(b'[') {
            return Ok(ClassItem::Nested(self.parse_class(start)?));
        }
        let low = if unit == u16::from(b'\\') {
            match self.parse_class_escape(start)? {
                ClassEscape::Predefined(predefined) => {
                    return Ok(ClassItem::Predefined(predefined));
                }
                ClassEscape::Literal(literal) => literal,
            }
        } else {
            unit
        };
        // A range, unless the `-` is last (`[a-]`) or starts one (`[-a]`).
        // A `-` opens a RANGE unless what follows ends the class. `[a-]` and
        // `[a-[b]]` are not ranges: the JDK reads the latter as `a` UNION the
        // nested class, while `[a-&&b]` really is the illegal range it looks
        // like (the `&` is the range's high end, and `&` < `a`).
        if self.peek() == Some(u16::from(b'-'))
            && self
                .units
                .get(self.at + 1)
                // A `-` with NOTHING after it opens a range all the same: the
                // pattern ran out before its high end, which a JDK reports as
                // an illegal range rather than an unclosed class.
                .is_none_or(|next| *next != u16::from(b']') && *next != u16::from(b'['))
        {
            self.at += 1;
            let high_start = self.at;
            let Some(high_unit) = self.next() else {
                return Err(self.error("Illegal character range", high_start));
            };
            let high = if high_unit == u16::from(b'\\') {
                match self.parse_class_escape(high_start)? {
                    ClassEscape::Literal(literal) => literal,
                    ClassEscape::Predefined(_) => {
                        return Err(self.error("Illegal character range", high_start));
                    }
                }
            } else {
                high_unit
            };
            if high < low {
                return Err(self.error("Illegal character range", high_start));
            }
            return Ok(ClassItem::Range(low, high));
        }
        Ok(ClassItem::Single(low))
    }

    #[allow(clippy::too_many_lines)] // one arm per escape kind
    fn parse_escape(&mut self, start: usize) -> ParseResult<Node> {
        let Some(unit) = self.peek() else {
            // A backslash at the very end: the JDK reads PAST the pattern and
            // reports its own confusion, at the index one past the last
            // character. Odd wording for an ordinary typo, but it is the
            // wording a student sees.
            let _ = start;
            return Err(self.error("Unexpected internal error", self.units.len()));
        };
        match unit {
            u if u == u16::from(b'b') => {
                self.at += 1;
                Ok(Node::WordBoundary(true))
            }
            u if u == u16::from(b'B') => {
                self.at += 1;
                Ok(Node::WordBoundary(false))
            }
            u if u == u16::from(b'A') => {
                self.at += 1;
                Ok(Node::InputStart)
            }
            u if u == u16::from(b'z') => {
                self.at += 1;
                Ok(Node::InputEnd)
            }
            u if u == u16::from(b'Z') => {
                self.at += 1;
                Ok(Node::InputEndBeforeFinalTerminator)
            }
            // `\R` — ANY line terminator (JDK 8+): a CRLF pair, or one of the
            // single terminators. The pair must be tried first, or `\R` splits
            // a CRLF into two.
            u if u == u16::from(b'R') => {
                self.at += 1;
                Ok(Node::Alt(vec![
                    Node::Concat(vec![Node::Literal(0x0D), Node::Literal(0x0A)]),
                    Node::Class(CharClass {
                        fold: false,
                        negated: false,
                        items: vec![
                            ClassItem::Single(0x0A),
                            ClassItem::Single(0x0B),
                            ClassItem::Single(0x0C),
                            ClassItem::Single(0x0D),
                            ClassItem::Single(0x85),
                            ClassItem::Single(0x2028),
                            ClassItem::Single(0x2029),
                        ],
                        intersections: Vec::new(),
                    }),
                ]))
            }
            u if u == u16::from(b'Q') => {
                // `\Q ... \E` — everything between is literal.
                self.at += 1;
                let mut nodes = Vec::new();
                while let Some(next) = self.peek() {
                    if next == u16::from(b'\\')
                        && self.units.get(self.at + 1) == Some(&u16::from(b'E'))
                    {
                        self.at += 2;
                        return Ok(Node::Concat(nodes));
                    }
                    nodes.push(self.literal(next));
                    self.at += 1;
                }
                Ok(Node::Concat(nodes))
            }
            // A backreference: `\1` .. `\9`, greedily extended while the
            // resulting group number still exists (Java's rule).
            // A backreference. Java compiles `\1` with no group at all — it
            // simply never matches — so refusing it at compile time rejected a
            // pattern a JDK accepts.
            u if (0x31..=0x39).contains(&u) => {
                let mut number = usize::from(u - 0x30);
                self.at += 1;
                while let Some(next) = self.peek() {
                    if !is_java_digit(u32::from(next)) {
                        break;
                    }
                    let extended = number * 10 + usize::from(next - 0x30);
                    if extended > self.groups {
                        break;
                    }
                    number = extended;
                    self.at += 1;
                }
                // The JDK compiles a reference to a group that does not exist
                // (it simply never matches), so only a group number of ZERO is
                // an error — `\0` is not a backreference at all.
                if number == 0 {
                    return Err(self.error("No group to reference", start));
                }
                Ok(Node::BackRef {
                    index: number,
                    fold: self.flags.fold,
                })
            }
            // `\k<name>` — a backreference by name.
            u if u == u16::from(b'k') => {
                // `parse_escape` PEEKS, so each arm consumes its own letter.
                self.at += 1;
                if !self.eat(u16::from(b'<')) {
                    return Err(self.error(
                        "\\k is not followed by '<' for named capturing group",
                        self.at,
                    ));
                }
                let name = self.parse_group_name(start)?;
                let Some(index) = self
                    .names
                    .iter()
                    .find(|(taken, _)| *taken == name)
                    .map(|(_, index)| *index)
                else {
                    // The JDK points at the closing `>`.
                    return Err(self.error(
                        &format!("named capturing group <{name}> does not exist"),
                        self.at.saturating_sub(1),
                    ));
                };
                Ok(Node::BackRef {
                    index,
                    fold: self.flags.fold,
                })
            }
            _ => match self.parse_class_escape(start)? {
                ClassEscape::Predefined(predefined) => Ok(Node::Class(CharClass {
                    fold: false,
                    negated: false,
                    items: vec![ClassItem::Predefined(predefined)],
                    intersections: Vec::new(),
                })),
                ClassEscape::Literal(literal) => Ok(self.literal(literal)),
            },
        }
    }

    /// The escapes meaningful both inside and outside a character class.
    fn parse_class_escape(&mut self, start: usize) -> ParseResult<ClassEscape> {
        let Some(unit) = self.next() else {
            // Inside a class the same truncation is the CLASS's complaint:
            // `[\` never closed.
            let _ = start;
            return Err(self.error("Unclosed character class", self.units.len()));
        };
        let literal = match unit {
            u if u == u16::from(b'd') => return Ok(ClassEscape::Predefined(Predefined::Digit)),
            u if u == u16::from(b'D') => return Ok(ClassEscape::Predefined(Predefined::NotDigit)),
            u if u == u16::from(b's') => return Ok(ClassEscape::Predefined(Predefined::Space)),
            u if u == u16::from(b'S') => return Ok(ClassEscape::Predefined(Predefined::NotSpace)),
            u if u == u16::from(b'w') => return Ok(ClassEscape::Predefined(Predefined::Word)),
            u if u == u16::from(b'W') => return Ok(ClassEscape::Predefined(Predefined::NotWord)),
            u if u == u16::from(b'n') => 0x0A,
            u if u == u16::from(b'r') => 0x0D,
            u if u == u16::from(b't') => 0x09,
            u if u == u16::from(b'f') => 0x0C,
            u if u == u16::from(b'a') => 0x07,
            u if u == u16::from(b'e') => 0x1B,
            u if u == u16::from(b'0') => {
                // `\0n`, `\0nn`, `\0mnn` — octal.
                let mut value: u32 = 0;
                let mut digits = 0;
                while digits < 3 {
                    let Some(next) = self.peek() else { break };
                    if !(0x30..=0x37).contains(&next) {
                        break;
                    }
                    value = value * 8 + u32::from(next - 0x30);
                    self.at += 1;
                    digits += 1;
                }
                if digits == 0 {
                    return Err(self.error("Illegal octal escape sequence", start));
                }
                u16::try_from(value).unwrap_or(u16::MAX)
            }
            // `\x{...}` — a code point in braces (JDK 7+), beside the
            // two-digit `\xhh`.
            u if u == u16::from(b'x') && self.peek() == Some(u16::from(b'{')) => {
                // `next()` already consumed the `x`, so `self.at` is the brace.
                self.at += 1;
                let mut value: u32 = 0;
                let mut digits = 0usize;
                while let Some(next) = self.peek() {
                    if next == u16::from(b'}') {
                        break;
                    }
                    let Some(digit) = char::from_u32(u32::from(next)).and_then(|c| c.to_digit(16))
                    else {
                        return Err(self.error("Illegal hexadecimal escape sequence", start));
                    };
                    value = value * 16 + digit;
                    digits += 1;
                    self.at += 1;
                }
                if digits == 0 || !self.eat(u16::from(b'}')) {
                    return Err(self.error("Unclosed hexadecimal escape sequence", start));
                }
                // The engine walks UTF-16 units, so a supplementary code point
                // becomes its high surrogate here; the low half follows as a
                // literal only for a pattern that spells the pair out.
                u16::try_from(value).unwrap_or(u16::MAX)
            }
            u if u == u16::from(b'x') => self.parse_hex(2, start)?,
            u if u == u16::from(b'u') => self.parse_hex(4, start)?,
            // A letter or digit after a backslash with no meaning is an error
            // in Java, not a literal — being permissive here would accept
            // patterns a real JDK refuses.
            other if is_java_word(u32::from(other)) && other != u16::from(b'_') => {
                return Err(self.error(
                    "Illegal/unsupported escape sequence",
                    self.at.saturating_sub(1),
                ));
            }
            other => other,
        };
        Ok(ClassEscape::Literal(literal))
    }

    fn parse_hex(&mut self, count: usize, start: usize) -> ParseResult<u16> {
        let mut value: u32 = 0;
        for _ in 0..count {
            let Some(unit) = self.peek() else {
                return Err(self.error("Illegal hexadecimal escape sequence", start));
            };
            let digit = char::from_u32(u32::from(unit))
                .and_then(|character| character.to_digit(16))
                .ok_or_else(|| self.error("Illegal hexadecimal escape sequence", start))?;
            value = value * 16 + digit;
            self.at += 1;
        }
        Ok(u16::try_from(value).unwrap_or(u16::MAX))
    }
}

enum ClassEscape {
    Literal(u16),
    Predefined(Predefined),
}

// ---------------------------------------------------------------------------
// Matching
// ---------------------------------------------------------------------------

/// Where a group matched, as `(start, end)`. Index 0 is the whole match.
type Captures = Vec<Option<(usize, usize)>>;

/// The continuation: what remains to match after the current node succeeds.
///
/// Modelled as a linked list on the Rust stack rather than a closure, so the
/// borrow checker stays out of the way and a repeat can re-enter itself.
enum Cont<'a> {
    /// Match `seq[at..]`, then continue with `parent`.
    Seq {
        seq: &'a [Node],
        at: usize,
        parent: &'a Cont<'a>,
    },
    /// One iteration of a repeat just succeeded; try for another.
    Repeat {
        node: &'a Node,
        min: u32,
        max: Option<u32>,
        kind: RepeatKind,
        done: u32,
        /// Where this iteration began, to detect a zero-width body.
        from: usize,
        parent: &'a Cont<'a>,
    },
    /// Close a capturing group that opened at `from`.
    CloseGroup {
        index: usize,
        from: usize,
        parent: &'a Cont<'a>,
    },
    /// The whole pattern matched.
    Done,
    /// Succeed only at exactly this position — a lookbehind's body has to end
    /// where the lookbehind sits, not merely somewhere after its start.
    EndAt(usize),
}

struct Matcher<'a> {
    input: &'a [u16],
    /// The REGION the match may read — `Matcher.region(start, end)`, and the
    /// whole input by default. Reads stop at `end`, so a pattern cannot match
    /// text outside the region even when the input has more of it.
    start: usize,
    end: usize,
    /// Anchoring bounds (the JDK's default): `^`, `$`, `\A` and `\z` treat the
    /// REGION's edges as the input's. Turned off, they match only at the true
    /// edges, which is what `useAnchoringBounds(false)` asks for.
    anchoring: bool,
    /// Transparent bounds: lookaround and `\b` may READ outside the region,
    /// though the match itself still may not. Off by default, which is why a
    /// region behaves like a substring.
    transparent: bool,
    /// How far a READ may go right now — the region's end, or the whole input
    /// while a transparent lookaround is running. `floor` is the same from the
    /// left, for lookbehind.
    limit: std::cell::Cell<usize>,
    floor: std::cell::Cell<usize>,
    /// Whether the last attempt ran out of INPUT (rather than failing on what
    /// it read), and whether its success depended on an end anchor — Java's
    /// `hitEnd()` and `requireEnd()`, which a program uses to tell "no match"
    /// from "not yet".
    hit_end: std::cell::Cell<bool>,
    require_end: std::cell::Cell<bool>,
    /// A ceiling on backtracking steps. A pathological pattern must fail
    /// rather than hang the browser tab the engine runs in.
    steps: std::cell::Cell<u64>,
}

impl<'a> Matcher<'a> {
    fn over(
        input: &'a [u16],
        start: usize,
        end: usize,
        anchoring: bool,
        transparent: bool,
    ) -> Matcher<'a> {
        Matcher {
            input,
            start,
            end,
            anchoring,
            transparent,
            limit: std::cell::Cell::new(end),
            floor: std::cell::Cell::new(start),
            hit_end: std::cell::Cell::new(false),
            require_end: std::cell::Cell::new(false),
            steps: std::cell::Cell::new(STEP_LIMIT),
        }
    }

    /// Where a read may stop, and where an ANCHOR thinks the input ends.
    fn read_limit(&self) -> usize {
        self.limit.get()
    }

    fn anchor_end(&self) -> usize {
        if self.anchoring {
            self.end
        } else {
            self.input.len()
        }
    }

    fn anchor_start(&self) -> usize {
        if self.anchoring { self.start } else { 0 }
    }

    /// A read at `at` that found nothing because the region ended there: the
    /// attempt HIT THE END, which is the whole of what `hitEnd` reports.
    fn note_end(&self, at: usize) {
        if at >= self.read_limit() {
            self.hit_end.set(true);
        }
    }
}

/// The backtracking budget. It exists so a pathological pattern fails instead
/// of hanging the browser tab this engine runs in — NOT to bound ordinary
/// work. At two million, `"a".repeat(1000) + "c"` against `a*b|c` (a plain
/// alternation with quadratic backtracking, and a pattern a student really
/// writes) ran out and reported "no match", which is a silent wrong answer.
const STEP_LIMIT: u64 = 20_000_000;

impl<'a> Matcher<'a> {
    /// The code point at `at`, and the UTF-16 units it occupies.
    ///
    /// Java's regex engine works on CODE POINTS, not on `char`s: `.` matches
    /// an astral character whole. Stepping a unit at a time let `.` match half
    /// a surrogate pair, so `"a\u{1F600}".replaceAll("a.", "#")` returned a
    /// string containing a LONE surrogate — a corrupt result, not merely a
    /// wrong count.
    fn code_point_at(&self, at: usize) -> Option<(u32, usize)> {
        if at >= self.read_limit() {
            self.hit_end.set(true);
            return None;
        }
        let high = *self.input.get(at)?;
        if (0xD800..0xDC00).contains(&high)
            && let Some(&low) = self.input.get(at + 1)
            && (0xDC00..0xE000).contains(&low)
        {
            let pair = 0x1_0000 + ((u32::from(high) - 0xD800) << 10) + (u32::from(low) - 0xDC00);
            return Some((pair, 2));
        }
        Some((u32::from(high), 1))
    }

    /// The code point at `at` with no bookkeeping — what a BOUNDARY reads.
    /// `\b` looks at a character without consuming it, so a read here must not
    /// count as running out of input the way a consuming one does.
    fn point_at(&self, at: usize) -> Option<u32> {
        let high = *self.input.get(at)?;
        if (0xD800..0xDC00).contains(&high)
            && let Some(&low) = self.input.get(at + 1)
            && (0xDC00..0xE000).contains(&low)
        {
            return Some(0x1_0000 + ((u32::from(high) - 0xD800) << 10) + (u32::from(low) - 0xDC00));
        }
        Some(u32::from(high))
    }

    /// What a boundary can SEE. A boundary reads, so it stops where a read
    /// stops — outside the region the text is not there at all, which is what
    /// makes a region behave like a substring. Under transparent bounds it is
    /// there: `\bcat\b` over the region `[3, 6)` of "thecat here" does not
    /// match, because the "the" before it is visible and there is no boundary.
    fn word_window(&self) -> (usize, usize) {
        if self.transparent {
            (0, self.input.len())
        } else {
            (self.floor.get(), self.read_limit())
        }
    }

    fn at_word(&self, at: usize) -> bool {
        // A boundary at the end of the input READ the end, and that counts
        // twice: `"dog\b"` against "dog" reports BOTH `hitEnd()` and
        // `requireEnd()` on a JDK — the engine had to look past the g to
        // decide the boundary was there, and one more word character would
        // take it away.
        if at >= self.read_limit() {
            self.hit_end.set(true);
            self.require_end.set(true);
        }
        let (floor, limit) = self.word_window();
        if at >= limit || at < floor {
            return false;
        }
        self.point_at(at).is_some_and(is_java_word)
    }

    fn is_boundary(&self, at: usize) -> bool {
        let (floor, _) = self.word_window();
        let before = at > floor && self.at_word(at - 1);
        let after = self.at_word(at);
        before != after
    }

    fn step(&self) -> bool {
        let left = self.steps.get();
        if left == 0 {
            return false;
        }
        self.steps.set(left - 1);
        true
    }

    /// Match `node` at `pos`, then the continuation. Returns the end of the
    /// overall match.
    #[allow(clippy::too_many_lines)] // one arm per node kind
    fn run(
        &self,
        node: &'a Node,
        pos: usize,
        caps: &mut Captures,
        cont: &Cont<'a>,
    ) -> Option<usize> {
        if !self.step() {
            return None;
        }
        match node {
            Node::Empty => self.resume(pos, caps, cont),
            Node::Literal(expected) => {
                self.note_end(pos);
                if pos < self.read_limit() && self.input.get(pos) == Some(expected) {
                    return self.resume(pos + 1, caps, cont);
                }
                None
            }
            Node::AnyChar => match self.code_point_at(pos) {
                Some((point, width)) if !is_line_terminator(point) => {
                    self.resume(pos + width, caps, cont)
                }
                _ => None,
            },
            Node::AnyCharDotAll => match self.code_point_at(pos) {
                Some((_, width)) => self.resume(pos + width, caps, cont),
                None => None,
            },
            Node::Class(class) => match self.code_point_at(pos) {
                Some((point, width)) if class.matches(point) => {
                    self.resume(pos + width, caps, cont)
                }
                _ => None,
            },
            Node::Concat(nodes) => {
                if nodes.is_empty() {
                    return self.resume(pos, caps, cont);
                }
                let next = Cont::Seq {
                    seq: nodes,
                    at: 1,
                    parent: cont,
                };
                self.run(&nodes[0], pos, caps, &next)
            }
            Node::Alt(branches) => {
                for branch in branches {
                    let saved = caps.clone();
                    if let Some(end) = self.run(branch, pos, caps, cont) {
                        return Some(end);
                    }
                    *caps = saved;
                }
                None
            }
            Node::Group { index, node } => match index {
                Some(index) => {
                    let next = Cont::CloseGroup {
                        index: *index,
                        from: pos,
                        parent: cont,
                    };
                    self.run(node, pos, caps, &next)
                }
                None => self.run(node, pos, caps, cont),
            },
            Node::Repeat {
                node,
                min,
                max,
                kind: RepeatKind::Possessive,
            } => self.repeat_possessive(node, *min, *max, pos, caps, cont),
            Node::Repeat {
                node,
                min,
                max,
                kind,
            } => self.repeat(node, *min, *max, *kind, 0, pos, caps, cont),
            Node::BackRef { index, fold } => {
                let Some(Some((from, to))) = caps.get(*index).copied() else {
                    // An unmatched group's backreference fails, per Java.
                    return None;
                };
                let text = &self.input[from..to];
                if self.read_limit() < pos + text.len() {
                    self.hit_end.set(true);
                    return None;
                }
                let here = &self.input[pos..pos + text.len()];
                let same = if *fold {
                    here.iter()
                        .zip(text)
                        .all(|(a, b)| a == b || ascii_fold(u32::from(*a)) == u32::from(*b))
                } else {
                    here == text
                };
                if same {
                    return self.resume(pos + text.len(), caps, cont);
                }
                None
            }
            Node::Start | Node::InputStart => {
                if pos == self.anchor_start() {
                    return self.resume(pos, caps, cont);
                }
                None
            }
            Node::End => {
                // Java's `$` (without MULTILINE) also matches before a final
                // line terminator. Reaching one is what `requireEnd` reports:
                // more input could have changed the answer.
                self.hit_end.set(true);
                if pos == self.anchor_end() || self.at_final_terminator(pos) {
                    self.require_end.set(true);
                    return self.resume(pos, caps, cont);
                }
                None
            }
            Node::InputEnd => {
                self.hit_end.set(true);
                if pos == self.anchor_end() {
                    self.require_end.set(true);
                    return self.resume(pos, caps, cont);
                }
                None
            }
            Node::InputEndBeforeFinalTerminator => {
                self.hit_end.set(true);
                if pos == self.anchor_end() || self.at_final_terminator(pos) {
                    self.require_end.set(true);
                    return self.resume(pos, caps, cont);
                }
                None
            }
            // `(?m)`: a line starts at the input's start and just after any
            // terminator; it ends at the input's end and just before one. A
            // CRLF pair is ONE terminator, so `$` sits before the CR.
            Node::LineStart => {
                // Java's own comment: "Perl does not match ^ at end of input
                // even after newline". So the END of the input is never a line
                // start — which for an EMPTY input is position 0 too, and
                // `"".matches("(?m)^.*$")` is false because of it.
                if pos == self.anchor_end() {
                    return None;
                }
                let after_terminator = pos > self.anchor_start()
                    && is_line_terminator(u32::from(self.input[pos - 1]))
                    // NOT between a CR and its LF: the pair is ONE terminator.
                    && !(self.input[pos - 1] == 0x0D && self.input.get(pos) == Some(&0x0A));
                if pos == self.anchor_start() || after_terminator {
                    return self.resume(pos, caps, cont);
                }
                None
            }
            Node::LineEnd => {
                let before_terminator = self
                    .input
                    .get(pos)
                    .is_some_and(|unit| is_line_terminator(u32::from(*unit)))
                    // A CRLF pair is ONE terminator: the line ends before the
                    // CR, not again between the CR and the LF.
                    && !(self.input[pos] == 0x0A && pos > 0 && self.input[pos - 1] == 0x0D);
                if pos == self.anchor_end() || before_terminator {
                    if pos == self.anchor_end() {
                        self.hit_end.set(true);
                        self.require_end.set(true);
                    }
                    return self.resume(pos, caps, cont);
                }
                None
            }
            Node::WordBoundary(wanted) => {
                if self.is_boundary(pos) == *wanted {
                    return self.resume(pos, caps, cont);
                }
                None
            }
            Node::Look {
                direction,
                negated,
                node,
            } => {
                // TRANSPARENT bounds: a lookaround may read outside the
                // region, though the match itself may not. The window is
                // widened for the duration and put back after, which is the
                // whole of what `useTransparentBounds(true)` asks for.
                let (kept_limit, kept_floor) = (self.limit.get(), self.floor.get());
                if self.transparent {
                    self.limit.set(self.input.len());
                    self.floor.set(0);
                }
                let restore = |matcher: &Self| {
                    matcher.limit.set(kept_limit);
                    matcher.floor.set(kept_floor);
                };
                let mut probe = caps.clone();
                let hit = match direction {
                    // Backwards: the body must END at `pos`, and it can begin
                    // no earlier than its widest match allows — or at the
                    // start of the input when that width is unbounded.
                    Look::Behind(width) => {
                        let earliest = match width {
                            Width::Fixed(width) => pos.saturating_sub(*width),
                            Width::Unbounded => 0,
                        };
                        (earliest..=pos).rev().any(|start| {
                            let mut attempt = caps.clone();
                            let landed = self.run(node, start, &mut attempt, &Cont::EndAt(pos));
                            if landed.is_some() {
                                probe = attempt;
                            }
                            landed.is_some()
                        })
                    }
                    Look::Ahead => self.run(node, pos, &mut probe, &Cont::Done).is_some(),
                };
                restore(self);
                if hit == *negated {
                    return None;
                }
                // A POSITIVE lookaround keeps what its body captured (Java
                // does); a negative one matched nothing, so it captures
                // nothing.
                if !*negated {
                    *caps = probe;
                }
                self.resume(pos, caps, cont)
            }
        }
    }

    /// Whether `pos` sits just before the input's final line terminator
    /// (`\n`, `\r\n`, or a lone `\r`).
    fn at_final_terminator(&self, pos: usize) -> bool {
        let len = self.anchor_end();
        if pos == len {
            return false;
        }
        if pos + 1 == len && is_line_terminator(u32::from(self.input[pos])) {
            // NOT between a final CR and LF: the pair is ONE terminator, so
            // `$` fires before the CR and nowhere else. Treating the LF as its
            // own terminator gave `"a\r\n".replaceAll("$", "X")` an extra X.
            return !(pos >= 1 && self.input[pos] == 0x0A && self.input[pos - 1] == 0x0D);
        }
        pos + 2 == len && self.input[pos] == 0x0D && self.input[pos + 1] == 0x0A
    }

    #[allow(clippy::too_many_arguments)]
    fn repeat(
        &self,
        node: &'a Node,
        min: u32,
        max: Option<u32>,
        kind: RepeatKind,
        done: u32,
        pos: usize,
        caps: &mut Captures,
        cont: &Cont<'a>,
    ) -> Option<usize> {
        if !self.step() {
            return None;
        }
        let may_take_more = max.is_none_or(|max| done < max);
        let take_more = |caps: &mut Captures| -> Option<usize> {
            if !may_take_more {
                return None;
            }
            let next = Cont::Repeat {
                node,
                min,
                max,
                kind,
                done: done + 1,
                from: pos,
                parent: cont,
            };
            self.run(node, pos, caps, &next)
        };

        // A SIMPLE body — one with no internal choice — repeats in a LOOP
        // rather than one stack frame per repetition. `a*b` over a thousand
        // characters is an ordinary pattern, and recursing per iteration
        // overflowed the stack long before the step budget noticed. The
        // behaviour is identical: match as far as the body goes, then try the
        // continuation from the longest run down to the minimum, which is what
        // the recursion did.
        //
        // Each repetition's END is recorded rather than assumed. A body that
        // consumes a SUPPLEMENTARY code point takes two code units, so backing
        // off by one landed BETWEEN the surrogates — and from there the
        // positions the loop visited were not the ones the repetitions had
        // reached, so `"a\ud83d\ude00b".matches(".*a.*")` gave up before
        // trying position 0 and answered false.
        if kind == RepeatKind::Greedy
            && done == 0
            && matches!(node, Node::Literal(_) | Node::Class(_) | Node::AnyChar)
        {
            let mut ends = vec![pos];
            let mut taken = 0u32;
            while max.is_none_or(|max| taken < max) {
                if !self.step() {
                    return None;
                }
                let end = *ends.last().expect("seeded with the start");
                let mut probe = caps.clone();
                let Some(next) = self.run(node, end, &mut probe, &Cont::Done) else {
                    break;
                };
                if next == end {
                    break; // a zero-width body would loop forever
                }
                ends.push(next);
                taken += 1;
            }
            while taken >= min {
                let saved = caps.clone();
                let end = ends[taken as usize];
                if let Some(matched) = self.resume(end, caps, cont) {
                    return Some(matched);
                }
                *caps = saved;
                if taken == 0 {
                    break;
                }
                taken -= 1;
            }
            return None;
        }
        // Below the minimum there is no choice to make.
        if done < min {
            return take_more(caps);
        }
        match kind {
            RepeatKind::Greedy => {
                let saved = caps.clone();
                if let Some(end) = take_more(caps) {
                    return Some(end);
                }
                *caps = saved;
                self.resume(pos, caps, cont)
            }
            RepeatKind::Reluctant => {
                let saved = caps.clone();
                if let Some(end) = self.resume(pos, caps, cont) {
                    return Some(end);
                }
                *caps = saved;
                take_more(caps)
            }
            // Possessive repeats never enter this path: `run` sends them to
            // `repeat_possessive`, and so no `Cont::Repeat` is ever built for
            // one. Resuming is the conservative fallback.
            RepeatKind::Possessive => self.resume(pos, caps, cont),
        }
    }

    /// `X*+` — match the body as many times as it will go, atomically, and
    /// never reconsider. If the continuation then fails, the whole repeat
    /// fails rather than giving a repetition back, which is exactly what makes
    /// `"aaa".matches("a*+a")` false where `"aaa".matches("a*a")` is true.
    fn repeat_possessive(
        &self,
        node: &'a Node,
        min: u32,
        max: Option<u32>,
        pos: usize,
        caps: &mut Captures,
        cont: &Cont<'a>,
    ) -> Option<usize> {
        let mut at = pos;
        let mut done = 0;
        while max.is_none_or(|max| done < max) {
            if !self.step() {
                return None;
            }
            // Each iteration matches on its own, with no continuation to
            // backtrack into — that independence is the "atomic" part.
            let mut trial = caps.clone();
            let Some(end) = self.run(node, at, &mut trial, &Cont::Done) else {
                break;
            };
            *caps = trial;
            done += 1;
            // A body that consumed nothing would repeat forever.
            if end == at {
                break;
            }
            at = end;
        }
        if done < min {
            return None;
        }
        self.resume(at, caps, cont)
    }

    /// Continue after a node matched, ending at `pos`.
    fn resume(&self, pos: usize, caps: &mut Captures, cont: &Cont<'a>) -> Option<usize> {
        if !self.step() {
            return None;
        }
        match cont {
            Cont::Done => Some(pos),
            Cont::EndAt(at) => (pos == *at).then_some(pos),
            Cont::Seq { seq, at, parent } => {
                if *at >= seq.len() {
                    return self.resume(pos, caps, parent);
                }
                let next = Cont::Seq {
                    seq,
                    at: at + 1,
                    parent,
                };
                self.run(&seq[*at], pos, caps, &next)
            }
            Cont::CloseGroup {
                index,
                from,
                parent,
            } => {
                let previous = caps[*index];
                caps[*index] = Some((*from, pos));
                if let Some(end) = self.resume(pos, caps, parent) {
                    return Some(end);
                }
                caps[*index] = previous;
                None
            }
            Cont::Repeat {
                node,
                min,
                max,
                kind,
                done,
                from,
                parent,
            } => {
                // A body that consumed nothing would loop forever; once the
                // minimum is met, stop. (Java's engine does the same.)
                if pos == *from && *done >= *min {
                    return self.resume(pos, caps, parent);
                }
                self.repeat(node, *min, *max, *kind, *done, pos, caps, parent)
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Public surface
// ---------------------------------------------------------------------------

/// The window a match may read and anchor against — `Matcher.region` and the
/// two bound modes beside it. The default is the whole input, anchored, opaque.
#[derive(Debug, Clone, Copy)]
pub struct Bounds {
    pub start: usize,
    pub end: usize,
    pub anchoring: bool,
    pub transparent: bool,
}

impl Bounds {
    #[must_use]
    pub fn whole(input: &[u16]) -> Bounds {
        Bounds {
            start: 0,
            end: input.len(),
            anchoring: true,
            transparent: false,
        }
    }
}

/// What one attempt found, and what it learned on the way.
#[derive(Debug, Clone)]
pub struct Attempt {
    pub matched: Option<Match>,
    pub hit_end: bool,
    pub require_end: bool,
}

/// One match: the whole-match span plus each group's span.
#[derive(Debug, Clone)]
pub struct Match {
    pub start: usize,
    pub end: usize,
    pub groups: Vec<Option<(usize, usize)>>,
}

impl Regex {
    /// Parse `pattern`, or report where it is malformed.
    pub fn new(pattern: &[u16]) -> Result<Regex, SyntaxError> {
        let mut parser = Parser {
            units: pattern,
            at: 0,
            groups: 0,
            names: Vec::new(),
            pattern: String::from_utf16_lossy(pattern),
            flags: Flags::default(),
        };
        let node = parser.parse_alt()?;
        if parser.at < parser.units.len() {
            // Only an unbalanced `)` can stop the parse early.
            // The JDK's cursor sits one BEFORE the `)` it stopped at.
            return Err(parser.error_at(
                "Unmatched closing ')'",
                isize::try_from(parser.at).unwrap_or(0) - 1,
            ));
        }
        Ok(Regex {
            node,
            group_count: parser.groups,
            names: parser.names,
        })
    }

    /// The number of capturing groups, excluding group 0.
    pub fn group_count(&self) -> usize {
        self.group_count
    }

    /// The group a `(?<name>X)` stands for, for `\k<name>` and a `${name}`
    /// replacement.
    pub fn group_named(&self, name: &str) -> Option<usize> {
        self.names
            .iter()
            .find(|(taken, _)| taken == name)
            .map(|(_, index)| *index)
    }

    /// The leftmost match at or after `from`.
    pub fn find_at(&self, input: &[u16], from: usize) -> Option<Match> {
        self.find_in(input, from, Bounds::whole(input)).matched
    }

    /// The leftmost match at or after `from`, within `bounds` — and what the
    /// attempt learned on the way: whether it ran out of input, and whether
    /// its answer depended on the end. `Matcher.find`, `hitEnd` and
    /// `requireEnd` are the same walk asked three questions.
    pub fn find_in(&self, input: &[u16], from: usize, bounds: Bounds) -> Attempt {
        let matcher = Matcher::over(
            input,
            bounds.start,
            bounds.end,
            bounds.anchoring,
            bounds.transparent,
        );
        for start in from..=bounds.end {
            let mut caps: Captures = vec![None; self.group_count + 1];
            if let Some(end) = matcher.run(&self.node, start, &mut caps, &Cont::Done) {
                caps[0] = Some((start, end));
                return Attempt {
                    matched: Some(Match {
                        start,
                        end,
                        groups: caps,
                    }),
                    hit_end: matcher.hit_end.get(),
                    require_end: matcher.require_end.get(),
                };
            }
        }
        Attempt {
            matched: None,
            hit_end: matcher.hit_end.get(),
            require_end: matcher.require_end.get(),
        }
    }

    /// The match ANCHORED at `bounds.start` — `Matcher.lookingAt`, which is
    /// `find` that may not move.
    pub fn looking_at(&self, input: &[u16], bounds: Bounds) -> Attempt {
        let matcher = Matcher::over(
            input,
            bounds.start,
            bounds.end,
            bounds.anchoring,
            bounds.transparent,
        );
        let mut caps: Captures = vec![None; self.group_count + 1];
        let landed = matcher.run(&self.node, bounds.start, &mut caps, &Cont::Done);
        let found = landed.map(|end| {
            caps[0] = Some((bounds.start, end));
            Match {
                start: bounds.start,
                end,
                groups: caps,
            }
        });
        Attempt {
            matched: found,
            hit_end: matcher.hit_end.get(),
            require_end: matcher.require_end.get(),
        }
    }

    /// The match that fills the whole region — `Matcher.matches`, with the
    /// groups it captured, which `matches_whole` (a bare yes/no) cannot give.
    pub fn matches_in(&self, input: &[u16], bounds: Bounds) -> Attempt {
        let anchored = Node::Concat(vec![self.node.clone(), Node::InputEnd]);
        let matcher = Matcher::over(
            input,
            bounds.start,
            bounds.end,
            // `matches` compares against the REGION's end whatever the
            // anchoring mode says, since it is the region it must fill.
            true,
            bounds.transparent,
        );
        let mut caps: Captures = vec![None; self.group_count + 1];
        let landed = matcher
            .run(&anchored, bounds.start, &mut caps, &Cont::Done)
            .filter(|end| *end == bounds.end);
        let found = landed.map(|end| {
            caps[0] = Some((bounds.start, end));
            Match {
                start: bounds.start,
                end,
                groups: caps,
            }
        });
        Attempt {
            matched: found,
            hit_end: matcher.hit_end.get(),
            require_end: matcher.require_end.get(),
        }
    }

    /// Whether the pattern matches the *entire* input — `String.matches`.
    ///
    /// The pattern is anchored with an explicit end marker rather than by
    /// testing the returned end position: a greedy body may first match a
    /// prefix (`"aab".matches("a*b?")` stops after `aa`), and only an anchor
    /// inside the pattern makes the engine give those characters back.
    pub fn matches_whole(&self, input: &[u16]) -> bool {
        self.matches_in(input, Bounds::whole(input))
            .matched
            .is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn units(text: &str) -> Vec<u16> {
        text.encode_utf16().collect()
    }

    fn compile(pattern: &str) -> Regex {
        Regex::new(&units(pattern)).expect("pattern parses")
    }

    /// Every span the pattern matches, as substrings.
    fn find_all(pattern: &str, text: &str) -> Vec<String> {
        let regex = compile(pattern);
        let input = units(text);
        let mut out = Vec::new();
        let mut at = 0;
        while at <= input.len() {
            let Some(found) = regex.find_at(&input, at) else {
                break;
            };
            out.push(String::from_utf16_lossy(&input[found.start..found.end]));
            at = if found.end == found.start {
                found.end + 1
            } else {
                found.end
            };
        }
        out
    }

    fn matches(pattern: &str, text: &str) -> bool {
        compile(pattern).matches_whole(&units(text))
    }

    #[test]
    fn literals_and_metacharacters() {
        assert!(matches("abc", "abc"));
        assert!(!matches("abc", "abd"));
        // `.` is any character but a line terminator.
        assert!(matches("a.c", "abc"));
        assert!(!matches("a.c", "a\nc"));
        // An escaped dot is literal.
        assert!(matches(r"a\.c", "a.c"));
        assert!(!matches(r"a\.c", "abc"));
    }

    #[test]
    fn predefined_classes_use_javas_definitions() {
        assert!(matches(r"\d+", "0123456789"));
        assert!(!matches(r"\d+", "12a"));
        assert!(matches(r"\w+", "a_Z9"));
        // Java's `\s` is exactly the six ASCII whitespace characters, so a
        // non-breaking space is NOT whitespace.
        assert!(matches(r"\s+", " \t\n\u{0b}\u{0c}\r"));
        assert!(!matches(r"\s", "\u{a0}"));
        assert!(matches(r"\S", "\u{a0}"));
    }

    #[test]
    fn character_classes() {
        assert!(matches("[abc]+", "cabba"));
        assert!(!matches("[abc]+", "cabd"));
        assert!(matches("[^abc]+", "xyz"));
        assert!(matches("[a-z0-9]+", "a9z"));
        // A `]` in first position is a literal.
        assert!(matches("[]]", "]"));
        // A trailing `-` is a literal, not a range.
        assert!(matches("[a-]+", "a-a"));
        // Nested union and intersection.
        assert!(matches("[a-d[m-p]]+", "abmp"));
        assert!(matches("[a-z&&[^bc]]+", "adz"));
        assert!(!matches("[a-z&&[^bc]]+", "ab"));
    }

    #[test]
    fn quantifiers_greedy_reluctant_and_possessive() {
        assert_eq!(find_all("a+", "aa b aaa"), vec!["aa", "aaa"]);
        // Greedy takes everything it can, reluctant the least.
        assert_eq!(find_all("<.+>", "<a><b>"), vec!["<a><b>"]);
        assert_eq!(find_all("<.+?>", "<a><b>"), vec!["<a>", "<b>"]);
        // Possessive never gives back, so this cannot match.
        assert!(!matches("a*+a", "aaa"));
        assert!(matches("a*a", "aaa"));
        // Counted repetition.
        assert!(matches("a{3}", "aaa"));
        assert!(!matches("a{3}", "aa"));
        assert!(matches("a{2,}", "aaaa"));
        assert!(matches("a{2,3}", "aaa"));
        assert!(!matches("a{2,3}", "aaaa"));
        // A `{` in ATOM position is a literal brace, as in Java. One after a
        // quantifiable atom must open a repetition: `a{x` is the JDK's
        // "Illegal repetition" (see `reject_illegal_repetition` in the
        // differential suite).
        assert!(matches("{x", "{x"));
        assert!(Regex::new(&units("a{x")).is_err());
    }

    #[test]
    fn alternation_groups_and_backreferences() {
        assert!(matches("cat|dog", "dog"));
        assert!(matches("(ab)+", "ababab"));
        // A backreference must match what the group captured.
        assert!(matches(r"(a+)b\1", "aabaa"));
        assert!(!matches(r"(a+)b\1", "aabaaa"));
        // A non-capturing group does not shift the numbering.
        assert!(matches(r"(?:x)(y)\1", "xyy"));
    }

    #[test]
    fn anchors_and_boundaries() {
        assert_eq!(find_all("^a", "aa"), vec!["a"]);
        assert_eq!(find_all(r"\bcat\b", "a cat here"), vec!["cat"]);
        assert_eq!(find_all(r"\bcat\b", "concatenate"), Vec::<String>::new());
        // Java's `$` also matches before a final line terminator.
        assert!(compile("a$").find_at(&units("a\n"), 0).is_some());
    }

    #[test]
    fn zero_width_matches_are_found_at_every_position() {
        // Four positions in a 3-character string.
        assert_eq!(find_all("x*", "abc").len(), 4);
    }

    #[test]
    fn capture_groups_report_their_spans() {
        let regex = compile(r"(\d+)-(\d+)");
        let input = units("id 12-345 end");
        let found = regex.find_at(&input, 0).expect("matches");
        assert_eq!(found.groups[1], Some((3, 5)));
        assert_eq!(found.groups[2], Some((6, 9)));
    }

    #[test]
    fn malformed_patterns_are_reported_not_mis_parsed() {
        assert!(Regex::new(&units("[a-")).is_err());
        assert!(Regex::new(&units("(ab")).is_err());
        assert!(Regex::new(&units("a)")).is_err());
        assert!(Regex::new(&units("*a")).is_err());
        assert!(Regex::new(&units(r"\")).is_err());
        // An unknown letter escape is an error in Java, not a literal.
        assert!(Regex::new(&units(r"\q")).is_err());
    }

    #[test]
    fn a_pathological_pattern_fails_rather_than_hanging() {
        // Classic catastrophic backtracking; must terminate via the step limit.
        assert!(!matches("(a+)+b", "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"));
    }
}
