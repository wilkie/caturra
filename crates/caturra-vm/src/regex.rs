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
}

/// A syntax error, carrying what Java's `PatternSyntaxException` reports.
#[derive(Debug, Clone)]
pub struct SyntaxError {
    pub description: String,
    pub index: usize,
    pub pattern: String,
}

impl SyntaxError {
    /// Java's `PatternSyntaxException.getMessage()` layout: description, the
    /// pattern, and a caret under the offending index.
    pub fn message(&self) -> String {
        format!(
            "{} near index {}\n{}\n{}^",
            self.description,
            self.index,
            self.pattern,
            " ".repeat(self.index)
        )
    }
}

#[derive(Debug, Clone)]
enum Node {
    /// Matches at the current position without consuming.
    Empty,
    Literal(u16),
    /// `.` — any character except a line terminator.
    AnyChar,
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
    BackRef(usize),
    /// `^`
    Start,
    /// `$`
    End,
    /// `\b` (true) and `\B` (false).
    WordBoundary(bool),
    /// `\A`
    InputStart,
    /// `\z`
    InputEnd,
    /// `\Z` — end of input, but before a final line terminator.
    InputEndBeforeFinalTerminator,
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
fn is_java_space(unit: u16) -> bool {
    matches!(unit, 0x20 | 0x09 | 0x0A | 0x0B | 0x0C | 0x0D)
}

fn is_java_digit(unit: u16) -> bool {
    (0x30..=0x39).contains(&unit)
}

/// Java's `\w` is `[a-zA-Z_0-9]` — ASCII only.
fn is_java_word(unit: u16) -> bool {
    is_java_digit(unit)
        || (0x41..=0x5A).contains(&unit)
        || (0x61..=0x7A).contains(&unit)
        || unit == 0x5F
}

/// The line terminators `.` refuses to match.
fn is_line_terminator(unit: u16) -> bool {
    matches!(unit, 0x0A | 0x0D | 0x85 | 0x2028 | 0x2029)
}

impl Predefined {
    fn matches(self, unit: u16) -> bool {
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

impl CharClass {
    fn matches(&self, unit: u16) -> bool {
        let mut hit = self.items.iter().any(|item| match item {
            ClassItem::Single(single) => *single == unit,
            ClassItem::Range(low, high) => *low <= unit && unit <= *high,
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
    pattern: String,
}

type ParseResult<T> = Result<T, SyntaxError>;

impl Parser<'_> {
    fn error(&self, description: &str, index: usize) -> SyntaxError {
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
    fn parse_concat(&mut self) -> ParseResult<Node> {
        let mut nodes = Vec::new();
        while let Some(unit) = self.peek() {
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
        while let Some(unit) = self.peek() {
            if !is_java_digit(unit) {
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
        if self.eat(u16::from(b'}')) {
            return Ok(Some((min, Some(min))));
        }
        if !self.eat(u16::from(b',')) {
            self.at = open;
            return Ok(None);
        }
        if self.eat(u16::from(b'}')) {
            return Ok(Some((min, None)));
        }
        let mut max_digits = String::new();
        while let Some(unit) = self.peek() {
            if !is_java_digit(unit) {
                break;
            }
            max_digits.push(char::from(unit as u8));
            self.at += 1;
        }
        if max_digits.is_empty() || !self.eat(u16::from(b'}')) {
            return Err(self.error("Unclosed counted closure", open));
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
            u if u == u16::from(b'.') => Ok(Node::AnyChar),
            u if u == u16::from(b'^') => Ok(Node::Start),
            u if u == u16::from(b'$') => Ok(Node::End),
            u if u == u16::from(b'(') => self.parse_group(start),
            u if u == u16::from(b'[') => Ok(Node::Class(self.parse_class(start)?)),
            u if u == u16::from(b'\\') => self.parse_escape(start),
            // The JDK's `error` reports the cursor it stopped at, one before
            // the character it just read — for `a)` that is index 0, not the
            // `)`'s own index.
            u if u == u16::from(b')') => {
                Err(self.error("Unmatched closing ')'", start.saturating_sub(1)))
            }
            u if u == u16::from(b'*') || u == u16::from(b'+') || u == u16::from(b'?') => {
                let meta = char::from_u32(u32::from(u)).unwrap_or('?');
                Err(self.error(&format!("Dangling meta character '{meta}'"), start))
            }
            other => Ok(Node::Literal(other)),
        }
    }

    fn parse_group(&mut self, open: usize) -> ParseResult<Node> {
        let mut index = None;
        if self.eat(u16::from(b'?')) {
            // `(?:...)` is the only group flag caturra models; the lookaround
            // and named-group forms are refused rather than silently ignored.
            if !self.eat(u16::from(b':')) {
                return Err(self.error("Unsupported group construct", open));
            }
        } else {
            self.groups += 1;
            index = Some(self.groups);
        }
        let node = self.parse_alt()?;
        if !self.eat(u16::from(b')')) {
            // At END of input the JDK's cursor is one past the last character.
            return Err(self.error("Unclosed group", self.units.len()));
        }
        Ok(Node::Group {
            index,
            node: Box::new(node),
        })
    }

    fn parse_class(&mut self, open: usize) -> ParseResult<CharClass> {
        let mut class = CharClass {
            negated: self.eat(u16::from(b'^')),
            items: Vec::new(),
            intersections: Vec::new(),
        };
        // A `]` in first position is a literal, not the terminator.
        if self.eat(u16::from(b']')) {
            class.items.push(ClassItem::Single(u16::from(b']')));
        }
        loop {
            let Some(unit) = self.peek() else {
                return Err(self.error("Unclosed character class", open));
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
            return Err(self.error("Unclosed character class", start));
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
                .is_some_and(|next| *next != u16::from(b']') && *next != u16::from(b'['))
        {
            self.at += 1;
            let high_start = self.at;
            let Some(high_unit) = self.next() else {
                return Err(self.error("Unclosed character class", high_start));
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

    fn parse_escape(&mut self, start: usize) -> ParseResult<Node> {
        let Some(unit) = self.peek() else {
            return Err(self.error("Trailing backslash", start));
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
                    nodes.push(Node::Literal(next));
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
                    if !is_java_digit(next) {
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
                Ok(Node::BackRef(number))
            }
            _ => match self.parse_class_escape(start)? {
                ClassEscape::Predefined(predefined) => Ok(Node::Class(CharClass {
                    negated: false,
                    items: vec![ClassItem::Predefined(predefined)],
                    intersections: Vec::new(),
                })),
                ClassEscape::Literal(literal) => Ok(Node::Literal(literal)),
            },
        }
    }

    /// The escapes meaningful both inside and outside a character class.
    fn parse_class_escape(&mut self, start: usize) -> ParseResult<ClassEscape> {
        let Some(unit) = self.next() else {
            return Err(self.error("Trailing backslash", start));
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
            other if is_java_word(other) && other != u16::from(b'_') => {
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
}

struct Matcher<'a> {
    input: &'a [u16],
    /// A ceiling on backtracking steps. A pathological pattern must fail
    /// rather than hang the browser tab the engine runs in.
    steps: std::cell::Cell<u64>,
}

/// The backtracking budget. It exists so a pathological pattern fails instead
/// of hanging the browser tab this engine runs in — NOT to bound ordinary
/// work. At two million, `"a".repeat(1000) + "c"` against `a*b|c` (a plain
/// alternation with quadratic backtracking, and a pattern a student really
/// writes) ran out and reported "no match", which is a silent wrong answer.
const STEP_LIMIT: u64 = 20_000_000;

impl<'a> Matcher<'a> {
    fn at_word(&self, at: usize) -> bool {
        self.input.get(at).copied().is_some_and(is_java_word)
    }

    fn is_boundary(&self, at: usize) -> bool {
        let before = at > 0 && self.at_word(at - 1);
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
                if self.input.get(pos) == Some(expected) {
                    return self.resume(pos + 1, caps, cont);
                }
                None
            }
            Node::AnyChar => match self.input.get(pos) {
                Some(unit) if !is_line_terminator(*unit) => self.resume(pos + 1, caps, cont),
                _ => None,
            },
            Node::Class(class) => match self.input.get(pos) {
                Some(unit) if class.matches(*unit) => self.resume(pos + 1, caps, cont),
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
            Node::BackRef(index) => {
                let Some(Some((from, to))) = caps.get(*index).copied() else {
                    // An unmatched group's backreference fails, per Java.
                    return None;
                };
                let text = &self.input[from..to];
                if self.input.len() < pos + text.len() {
                    return None;
                }
                if &self.input[pos..pos + text.len()] == text {
                    return self.resume(pos + text.len(), caps, cont);
                }
                None
            }
            Node::Start | Node::InputStart => {
                if pos == 0 {
                    return self.resume(pos, caps, cont);
                }
                None
            }
            Node::End => {
                // Java's `$` (without MULTILINE) also matches before a final
                // line terminator.
                if pos == self.input.len() || self.at_final_terminator(pos) {
                    return self.resume(pos, caps, cont);
                }
                None
            }
            Node::InputEnd => {
                if pos == self.input.len() {
                    return self.resume(pos, caps, cont);
                }
                None
            }
            Node::InputEndBeforeFinalTerminator => {
                if pos == self.input.len() || self.at_final_terminator(pos) {
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
        }
    }

    /// Whether `pos` sits just before the input's final line terminator
    /// (`\n`, `\r\n`, or a lone `\r`).
    fn at_final_terminator(&self, pos: usize) -> bool {
        let len = self.input.len();
        if pos == len {
            return false;
        }
        if pos + 1 == len && is_line_terminator(self.input[pos]) {
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

        // A SIMPLE body — one that consumes exactly one unit and has no
        // internal choice — repeats in a LOOP rather than one stack frame per
        // repetition. `a*b` over a thousand characters is an ordinary pattern,
        // and recursing per iteration overflowed the stack long before the
        // step budget noticed. The behaviour is identical: match as far as the
        // body goes, then try the continuation from the longest run down to
        // the minimum, which is what the recursion did.
        if kind == RepeatKind::Greedy
            && done == 0
            && matches!(node, Node::Literal(_) | Node::Class(_) | Node::AnyChar)
        {
            let mut end = pos;
            let mut taken = 0u32;
            while max.is_none_or(|max| taken < max) {
                if !self.step() {
                    return None;
                }
                let mut probe = caps.clone();
                let Some(next) = self.run(node, end, &mut probe, &Cont::Done) else {
                    break;
                };
                if next == end {
                    break; // a zero-width body would loop forever
                }
                end = next;
                taken += 1;
            }
            while taken >= min {
                let saved = caps.clone();
                if let Some(matched) = self.resume(end, caps, cont) {
                    return Some(matched);
                }
                *caps = saved;
                if taken == 0 {
                    break;
                }
                taken -= 1;
                end -= 1;
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
            pattern: String::from_utf16_lossy(pattern),
        };
        let node = parser.parse_alt()?;
        if parser.at < parser.units.len() {
            // Only an unbalanced `)` can stop the parse early.
            // The JDK's cursor sits one BEFORE the `)` it stopped at.
            return Err(parser.error("Unmatched closing ')'", parser.at.saturating_sub(1)));
        }
        Ok(Regex {
            node,
            group_count: parser.groups,
        })
    }

    /// The number of capturing groups, excluding group 0.
    pub fn group_count(&self) -> usize {
        self.group_count
    }

    /// The leftmost match at or after `from`.
    pub fn find_at(&self, input: &[u16], from: usize) -> Option<Match> {
        let matcher = Matcher {
            input,
            steps: std::cell::Cell::new(STEP_LIMIT),
        };
        for start in from..=input.len() {
            let mut caps: Captures = vec![None; self.group_count + 1];
            if let Some(end) = matcher.run(&self.node, start, &mut caps, &Cont::Done) {
                caps[0] = Some((start, end));
                return Some(Match {
                    start,
                    end,
                    groups: caps,
                });
            }
        }
        None
    }

    /// Whether the pattern matches the *entire* input — `String.matches`.
    ///
    /// The pattern is anchored with an explicit end marker rather than by
    /// testing the returned end position: a greedy body may first match a
    /// prefix (`"aab".matches("a*b?")` stops after `aa`), and only an anchor
    /// inside the pattern makes the engine give those characters back.
    pub fn matches_whole(&self, input: &[u16]) -> bool {
        let anchored = Node::Concat(vec![self.node.clone(), Node::InputEnd]);
        let matcher = Matcher {
            input,
            steps: std::cell::Cell::new(STEP_LIMIT),
        };
        let mut caps: Captures = vec![None; self.group_count + 1];
        matcher
            .run(&anchored, 0, &mut caps, &Cont::Done)
            .is_some_and(|end| end == input.len())
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
