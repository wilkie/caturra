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

use crate::unicode;

/// A parsed pattern, ready to match.
#[derive(Debug, Clone)]
pub struct Regex {
    node: Node,
    /// Capturing groups, excluding group 0 (the whole match).
    group_count: usize,
    /// `(?<name>X)` names, and the group each stands for.
    names: Vec<(String, usize)>,
    /// Whether the SEARCH steps by code point rather than by code unit —
    /// `java.util.regex`'s `StartS` rather than `Start`, chosen when the
    /// pattern mentions a supplementary code point or a surrogate
    /// (`Pattern.isSupplementary` counts a lone surrogate as one).
    ///
    /// It is observable. `[\uD800-\uDFFF]` mentions surrogates, so the scan
    /// never begins inside a pair and an astral character is left whole; `\X`
    /// mentions none, so the scan tries every unit and `\X{2}` really does
    /// match a flag emoji starting one unit in.
    steps_by_code_point: bool,
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
    /// `.` — any character except a line terminator, under `(?d)` or not.
    AnyChar(bool),
    /// `.` under `(?s)` (DOTALL) — any character at all.
    AnyCharDotAll,
    Class(CharClass),
    Concat(Vec<Node>),
    Alt(Vec<Node>),
    Repeat {
        node: Box<Node>,
        /// Whether the quantifier was a COUNTED closure — `*`, `+`, `{m,n}`.
        /// A `?` is not: `java.util.regex` compiles `X?` to a branch and
        /// `X{0,1}` to a loop, and a loop over a capturing group puts the
        /// group's boundaries back on its way out. So `((?!x))?` reports the
        /// group as "" and `((?!x)){0,1}` reports it as null.
        counted: bool,
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
    /// `$`. The flag is `(?d)` — `UNIX_LINES`, where only `\n` ends a line.
    End(bool),
    /// `^` under `(?m)` (MULTILINE) — the start of any line.
    LineStart(bool),
    /// `$` under `(?m)` — the end of any line.
    LineEnd(bool),
    /// `\b` (true) and `\B` (false). The second flag is `(?U)`, where a word
    /// character is the Unicode set rather than `[a-zA-Z_0-9]`.
    WordBoundary(bool, bool),
    /// `\A`
    InputStart,
    /// `\z`
    InputEnd,
    /// `\G` — where the PREVIOUS match ended, which for the first attempt of
    /// a search is where the search began.
    PreviousEnd,
    /// `\X` — one extended grapheme cluster: what a reader would call one
    /// character, however many code points it takes.
    Grapheme,
    /// `\b{g}` — a grapheme cluster boundary, consuming nothing.
    GraphemeBound,
    /// `\Z` — end of input, but before a final line terminator.
    InputEndBeforeFinalTerminator(bool),
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
        | Node::End(_)
        | Node::WordBoundary(..)
        | Node::InputStart
        | Node::InputEnd
        | Node::PreviousEnd
        | Node::GraphemeBound
        | Node::InputEndBeforeFinalTerminator(_)
        | Node::LineStart(_)
        | Node::LineEnd(_)
        | Node::Look { .. } => Some(Width::Fixed(0)),
        Node::Literal(_) | Node::AnyChar(_) | Node::AnyCharDotAll | Node::Class(_) => {
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
        // A cluster is one code point or a dozen — never a knowable width,
        // which is also why a JDK will not put one inside a lookbehind.
        Node::Grapheme | Node::BackRef { .. } => None,
    }
}

/// Whether a repeated body always matches the SAME number of code units —
/// what `java.util.regex` calls a deterministic node, and the reason it
/// compiles `(X){m,n}` two different ways. A fixed-width body becomes a
/// `GroupCurly`, which re-asserts the group's boundaries once the rest of the
/// pattern has matched; a variable one becomes a `Loop`, where the boundaries
/// are simply whatever the last iteration wrote. The two answer differently
/// only when the same group is written again deeper in the match, which is
/// what `fixed_width` is asked about.
fn fixed_width(node: &Node) -> Option<usize> {
    match node {
        Node::Empty
        | Node::Start
        | Node::End(_)
        | Node::WordBoundary(..)
        | Node::InputStart
        | Node::InputEnd
        | Node::PreviousEnd
        | Node::GraphemeBound
        | Node::InputEndBeforeFinalTerminator(_)
        | Node::LineStart(_)
        | Node::LineEnd(_)
        | Node::Look { .. } => Some(0),
        Node::Literal(_) | Node::AnyChar(_) | Node::AnyCharDotAll | Node::Class(_) => Some(1),
        Node::Concat(nodes) => nodes
            .iter()
            .try_fold(0, |total, node| Some(total + fixed_width(node)?)),
        Node::Alt(branches) => {
            let mut widths = branches.iter().map(fixed_width);
            let first = widths.next()??;
            widths.all(|width| width == Some(first)).then_some(first)
        }
        Node::Group { node, .. } => fixed_width(node),
        // `X{2}` is fixed where `X{1,2}` is not, whatever `X` is.
        Node::Repeat { node, min, max, .. } => {
            (Some(*min) == *max).then(|| fixed_width(node).map(|width| width * *min as usize))?
        }
        Node::Grapheme | Node::BackRef { .. } => None,
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
    /// A CODE POINT, not a unit: `[\x{1F600}]` is one item, and the class is
    /// asked about code points because the engine decodes a surrogate pair
    /// before it consults one.
    Single(u32),
    Range(u32, u32),
    /// A predefined class such as `\d`, usable inside `[...]` too.
    Predefined(Predefined),
    /// `\p{...}`, and `\P{...}` with `negated` set.
    Named {
        property: Property,
        negated: bool,
    },
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
    /// `\h` — Perl's HORIZONTAL whitespace, which is not Java's `\s`.
    Horizontal,
    NotHorizontal,
    /// `\v` — Perl's VERTICAL whitespace: the line terminators.
    Vertical,
    NotVertical,
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

/// The line terminators `.` refuses to match — and under `(?d)`, `UNIX_LINES`,
/// the ONE that `.`, `^` and `$` recognise there.
fn is_line_terminator(unit: u32, unix_lines: bool) -> bool {
    if unix_lines {
        return unit == 0x0A;
    }
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
            Predefined::Horizontal => is_horizontal_space(unit),
            Predefined::NotHorizontal => !is_horizontal_space(unit),
            Predefined::Vertical => is_vertical_space(unit),
            Predefined::NotVertical => !is_vertical_space(unit),
        }
    }
}

/// `\p{...}` — a NAMED character property. Every arm is one case of the
/// JDK's own `CharPredicates.forProperty`, which is where the surprises live:
/// the POSIX names are US-ASCII only, and the same name means something wider
/// under `(?U)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Property {
    /// A set of `Character.getType` categories, one bit each.
    Categories(u32),
    /// A code point range, which several POSIX classes simply are.
    Range(u32, u32),
    /// One of `Character`'s own predicates, or a Unicode binary property.
    Of(Pred),
    /// `\p{IsLatin}` and `\p{script=Latin}`.
    Script(u8),
    /// `\p{InGreek}` and `\p{block=Greek}`.
    Block(u16),
    /// `\p{all}`.
    All,
}

/// The properties that are a predicate rather than a set of categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Pred {
    // The US-ASCII POSIX classes that are not one range.
    AsciiAlnum,
    AsciiAlpha,
    AsciiBlank,
    AsciiCntrl,
    AsciiGraph,
    AsciiPunct,
    AsciiSpace,
    AsciiXDigit,
    // The Unicode binary properties — and what a POSIX name means under `(?U)`.
    Alphabetic,
    Assigned,
    HexDigit,
    Ideographic,
    JoinControl,
    Letter,
    LetterOrDigit,
    Lowercase,
    Uppercase,
    Titlecase,
    NonCharacter,
    WhiteSpace,
    Word,
    Digit,
    Blank,
    Graph,
    Print,
    Alnum,
    // `Character`'s own, where they are not one of the above.
    JavaWhitespace,
    SpaceChar,
    IsoControl,
    Mirrored,
    JavaIdentifierStart,
    JavaIdentifierPart,
    UnicodeIdentifierStart,
    UnicodeIdentifierPart,
    IdentifierIgnorable,
}

/// Every `Character.getType` value, as a bit.
const fn category_bit(category: u8) -> u32 {
    1 << category
}

const CATEGORY_LETTER: u32 =
    category_bit(1) | category_bit(2) | category_bit(3) | category_bit(4) | category_bit(5);
const CATEGORY_MARK: u32 = category_bit(6) | category_bit(7) | category_bit(8);
const CATEGORY_NUMBER: u32 = category_bit(9) | category_bit(10) | category_bit(11);
const CATEGORY_SEPARATOR: u32 = category_bit(12) | category_bit(13) | category_bit(14);
const CATEGORY_OTHER: u32 =
    category_bit(15) | category_bit(16) | category_bit(18) | category_bit(19) | category_bit(0);
const CATEGORY_PUNCTUATION: u32 = category_bit(20)
    | category_bit(21)
    | category_bit(22)
    | category_bit(23)
    | category_bit(24)
    | category_bit(29)
    | category_bit(30);
const CATEGORY_SYMBOL: u32 =
    category_bit(25) | category_bit(26) | category_bit(27) | category_bit(28);

impl Property {
    /// A property is asked about a CODE POINT.
    fn matches(self, point: u32) -> bool {
        match self {
            Property::All => true,
            Property::Categories(mask) => mask & category_bit(unicode::category_of(point)) != 0,
            Property::Range(low, high) => low <= point && point <= high,
            // Scripts and blocks are recorded over the BMP, which is where
            // every script a program is likely to name lives; above it a code
            // point belongs to no block and to the unknown script.
            Property::Script(script) => {
                u16::try_from(point).is_ok_and(|unit| unicode::script_of(unit) == script)
            }
            Property::Block(block) => {
                u16::try_from(point).is_ok_and(|unit| unicode::block_of(unit) == Some(block))
            }
            Property::Of(pred) => pred.matches(point),
        }
    }
}

impl Pred {
    #[allow(clippy::too_many_lines)] // one arm per named property
    fn matches(self, point: u32) -> bool {
        let ascii = u8::try_from(point).unwrap_or(0xFF);
        let category = unicode::category_of(point);
        let in_categories = |mask: u32| mask & category_bit(category) != 0;
        match self {
            // US-ASCII, from the JDK's own `ASCII.ctype` table.
            Pred::AsciiAlnum => ascii.is_ascii_alphanumeric(),
            Pred::AsciiAlpha => ascii.is_ascii_alphabetic(),
            Pred::AsciiBlank => matches!(point, 0x09 | 0x20),
            Pred::AsciiCntrl => matches!(point, 0x00..=0x1F | 0x7F),
            Pred::AsciiGraph => (0x21..=0x7E).contains(&point),
            Pred::AsciiPunct => (0x21..=0x7E).contains(&point) && !ascii.is_ascii_alphanumeric(),
            Pred::AsciiSpace => matches!(point, 0x09..=0x0D | 0x20),
            Pred::AsciiXDigit => ascii.is_ascii_hexdigit(),
            // Unicode.
            Pred::Alphabetic => unicode::is_alphabetic(point),
            Pred::Assigned => unicode::is_defined(point),
            Pred::HexDigit => {
                unicode::is_digit(point)
                    || matches!(
                        point,
                        0x30..=0x39
                            | 0x41..=0x46
                            | 0x61..=0x66
                            | 0xFF10..=0xFF19
                            | 0xFF21..=0xFF26
                            | 0xFF41..=0xFF46
                    )
            }
            Pred::Ideographic => unicode::is_ideographic(point),
            Pred::JoinControl => matches!(point, 0x200C | 0x200D),
            Pred::Letter => unicode::is_letter(point),
            Pred::LetterOrDigit => unicode::is_letter(point) || unicode::is_digit(point),
            Pred::Lowercase => unicode::is_lower(point),
            Pred::Uppercase => unicode::is_upper(point),
            Pred::Titlecase => unicode::is_title_case(point),
            Pred::NonCharacter => point & 0xFFFE == 0xFFFE || (0xFDD0..=0xFDEF).contains(&point),
            // `\p{IsWhite_Space}` is NOT `Character.isWhitespace`: it keeps
            // the no-break spaces the latter drops.
            Pred::WhiteSpace => {
                in_categories(CATEGORY_SEPARATOR) || (0x9..=0xD).contains(&point) || point == 0x85
            }
            Pred::Word => {
                unicode::is_alphabetic(point)
                    || in_categories(CATEGORY_MARK | category_bit(9) | category_bit(23))
                    || matches!(point, 0x200C | 0x200D)
            }
            Pred::Digit => unicode::is_digit(point),
            Pred::Blank => category == 12 || point == 0x09,
            Pred::Graph => {
                !in_categories(CATEGORY_SEPARATOR | category_bit(15) | category_bit(19) | 1)
            }
            Pred::Print => {
                (Pred::Graph.matches(point) || Pred::Blank.matches(point)) && category != 15
            }
            Pred::Alnum => unicode::is_alphabetic(point) || unicode::is_digit(point),
            Pred::JavaWhitespace => unicode::is_whitespace(point),
            Pred::SpaceChar => unicode::is_space_char(point),
            Pred::IsoControl => unicode::is_iso_control(point),
            Pred::Mirrored => unicode::is_mirrored(point),
            Pred::JavaIdentifierStart => unicode::is_java_identifier_start(point),
            Pred::JavaIdentifierPart => unicode::is_java_identifier_part(point),
            Pred::UnicodeIdentifierStart => unicode::is_unicode_identifier_start(point),
            Pred::UnicodeIdentifierPart => unicode::is_unicode_identifier_part(point),
            Pred::IdentifierIgnorable => unicode::is_identifier_ignorable(point),
        }
    }
}

/// What a JDK complains about when a property name is not one it knows.
fn property_error(name: &str) -> String {
    if let Some((key, value)) = name.split_once('=') {
        return format!(
            "Unknown Unicode property {{name=<{}>, value=<{}>}}",
            key.to_ascii_lowercase(),
            value
        );
    }
    // For `\p{IsFoo}` the "Is" is stripped before the message is built, and
    // for `\p{InFoo}` it is not — so both read "{In/Is...}".
    let shown = name.strip_prefix("Is").unwrap_or(name);
    format!("Unknown character property name {{In/Is{shown}}}")
}

/// The JDK's `Pattern.family` dispatch, name for name.
fn resolve_property(name: &str, unicode_classes: bool) -> Option<Property> {
    if let Some((key, value)) = name.split_once('=') {
        return match key.to_ascii_lowercase().as_str() {
            "sc" | "script" => unicode::script_by_name(value).map(Property::Script),
            "blk" | "block" => unicode::block_by_name(value).map(Property::Block),
            "gc" | "general_category" => named_property(value),
            _ => None,
        };
    }
    if let Some(rest) = name.strip_prefix("In") {
        return unicode::block_by_name(rest).map(Property::Block);
    }
    if let Some(rest) = name.strip_prefix("Is") {
        return unicode_property(rest)
            .or_else(|| named_property(rest))
            .or_else(|| unicode::script_by_name(rest).map(Property::Script));
    }
    // Under `(?U)` a bare POSIX name means the UNICODE class of that name,
    // and only falls back to the US-ASCII one if it is not a POSIX name.
    if unicode_classes && let Some(property) = posix_property(name) {
        return Some(property);
    }
    named_property(name)
}

/// `\p{IsAlphabetic}` and the other binary properties, matched without
/// regard to case; the POSIX names answer here too.
fn unicode_property(name: &str) -> Option<Property> {
    let upper = name.to_ascii_uppercase();
    let pred = match upper.as_str() {
        "ALPHABETIC" => Pred::Alphabetic,
        "ASSIGNED" => Pred::Assigned,
        "CONTROL" => return Some(Property::Categories(category_bit(15))),
        "HEXDIGIT" | "HEX_DIGIT" => Pred::HexDigit,
        "IDEOGRAPHIC" => Pred::Ideographic,
        "JOINCONTROL" | "JOIN_CONTROL" => Pred::JoinControl,
        "LETTER" => Pred::Letter,
        "LOWERCASE" => Pred::Lowercase,
        "NONCHARACTERCODEPOINT" | "NONCHARACTER_CODE_POINT" => Pred::NonCharacter,
        "TITLECASE" => Pred::Titlecase,
        "PUNCTUATION" => return Some(Property::Categories(CATEGORY_PUNCTUATION)),
        "UPPERCASE" => Pred::Uppercase,
        "WHITESPACE" | "WHITE_SPACE" => Pred::WhiteSpace,
        "WORD" => Pred::Word,
        _ => return posix_property(&upper),
    };
    Some(Property::Of(pred))
}

/// The POSIX names as `(?U)` and `\p{Is...}` read them — Unicode-wide, not
/// the US-ASCII sets the bare names mean.
fn posix_property(name: &str) -> Option<Property> {
    let pred = match name.to_ascii_uppercase().as_str() {
        "ALPHA" => Pred::Alphabetic,
        "LOWER" => Pred::Lowercase,
        "UPPER" => Pred::Uppercase,
        "SPACE" => Pred::WhiteSpace,
        "PUNCT" => return Some(Property::Categories(CATEGORY_PUNCTUATION)),
        "XDIGIT" => Pred::HexDigit,
        "ALNUM" => Pred::Alnum,
        "CNTRL" => return Some(Property::Categories(category_bit(15))),
        "DIGIT" => Pred::Digit,
        "BLANK" => Pred::Blank,
        "GRAPH" => Pred::Graph,
        "PRINT" => Pred::Print,
        _ => return None,
    };
    Some(Property::Of(pred))
}

/// `CharPredicates.forProperty` — the categories, the US-ASCII POSIX classes,
/// and `Character`'s own predicates. These names are CASE SENSITIVE.
fn named_property(name: &str) -> Option<Property> {
    let categories = match name {
        "Cn" => category_bit(0),
        "Lu" => category_bit(1),
        "Ll" => category_bit(2),
        "Lt" => category_bit(3),
        "Lm" => category_bit(4),
        "Lo" => category_bit(5),
        "Mn" => category_bit(6),
        "Me" => category_bit(7),
        "Mc" => category_bit(8),
        "Nd" => category_bit(9),
        "Nl" => category_bit(10),
        "No" => category_bit(11),
        "Zs" => category_bit(12),
        "Zl" => category_bit(13),
        "Zp" => category_bit(14),
        "Cc" => category_bit(15),
        "Cf" => category_bit(16),
        "Co" => category_bit(18),
        "Cs" => category_bit(19),
        "Pd" => category_bit(20),
        "Ps" => category_bit(21),
        "Pe" => category_bit(22),
        "Pc" => category_bit(23),
        "Po" => category_bit(24),
        "Sm" => category_bit(25),
        "Sc" => category_bit(26),
        "Sk" => category_bit(27),
        "So" => category_bit(28),
        "Pi" => category_bit(29),
        "Pf" => category_bit(30),
        "L" => CATEGORY_LETTER,
        "M" => CATEGORY_MARK,
        "N" => CATEGORY_NUMBER,
        "Z" => CATEGORY_SEPARATOR,
        "C" => CATEGORY_OTHER,
        "P" => CATEGORY_PUNCTUATION,
        "S" => CATEGORY_SYMBOL,
        "LC" => category_bit(1) | category_bit(2) | category_bit(3),
        "LD" => CATEGORY_LETTER | category_bit(9),
        "L1" => return Some(Property::Range(0x00, 0xFF)),
        "all" => return Some(Property::All),
        // The POSIX classes, US-ASCII only, which is what these names mean
        // WITHOUT `(?U)`.
        "ASCII" => return Some(Property::Range(0x00, 0x7F)),
        "Digit" => return Some(Property::Range(0x30, 0x39)),
        "Lower" => return Some(Property::Range(0x61, 0x7A)),
        "Upper" => return Some(Property::Range(0x41, 0x5A)),
        "Print" => return Some(Property::Range(0x20, 0x7E)),
        "Alnum" => return Some(Property::Of(Pred::AsciiAlnum)),
        "Alpha" => return Some(Property::Of(Pred::AsciiAlpha)),
        "Blank" => return Some(Property::Of(Pred::AsciiBlank)),
        "Cntrl" => return Some(Property::Of(Pred::AsciiCntrl)),
        "Graph" => return Some(Property::Of(Pred::AsciiGraph)),
        "Punct" => return Some(Property::Of(Pred::AsciiPunct)),
        "Space" => return Some(Property::Of(Pred::AsciiSpace)),
        "XDigit" => return Some(Property::Of(Pred::AsciiXDigit)),
        // `Character`'s own.
        "javaLowerCase" => return Some(Property::Of(Pred::Lowercase)),
        "javaUpperCase" => return Some(Property::Of(Pred::Uppercase)),
        "javaAlphabetic" => return Some(Property::Of(Pred::Alphabetic)),
        "javaIdeographic" => return Some(Property::Of(Pred::Ideographic)),
        "javaTitleCase" => return Some(Property::Of(Pred::Titlecase)),
        "javaDigit" => return Some(Property::Of(Pred::Digit)),
        "javaDefined" => return Some(Property::Of(Pred::Assigned)),
        "javaLetter" => return Some(Property::Of(Pred::Letter)),
        "javaLetterOrDigit" => return Some(Property::Of(Pred::LetterOrDigit)),
        "javaJavaIdentifierStart" => return Some(Property::Of(Pred::JavaIdentifierStart)),
        "javaJavaIdentifierPart" => return Some(Property::Of(Pred::JavaIdentifierPart)),
        "javaUnicodeIdentifierStart" => return Some(Property::Of(Pred::UnicodeIdentifierStart)),
        "javaUnicodeIdentifierPart" => return Some(Property::Of(Pred::UnicodeIdentifierPart)),
        "javaIdentifierIgnorable" => return Some(Property::Of(Pred::IdentifierIgnorable)),
        "javaSpaceChar" => return Some(Property::Of(Pred::SpaceChar)),
        "javaWhitespace" => return Some(Property::Of(Pred::JavaWhitespace)),
        "javaISOControl" => return Some(Property::Of(Pred::IsoControl)),
        "javaMirrored" => return Some(Property::Of(Pred::Mirrored)),
        _ => return None,
    };
    Some(Property::Categories(categories))
}

/// `\h` — a tab, a space, and the Unicode spaces beside them.
fn is_horizontal_space(unit: u32) -> bool {
    matches!(
        unit,
        0x09 | 0x20 | 0xA0 | 0x1680 | 0x180E | 0x2000..=0x200A | 0x202F | 0x205F | 0x3000
    )
}

/// `\v` — the line terminators.
fn is_vertical_space(unit: u32) -> bool {
    matches!(unit, 0x0A..=0x0D | 0x85 | 0x2028 | 0x2029)
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
            ClassItem::Single(single) => *single == unit,
            ClassItem::Range(low, high) => *low <= unit && unit <= *high,
            ClassItem::Predefined(predefined) => predefined.matches(unit),
            ClassItem::Named { property, negated } => property.matches(unit) != *negated,
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
    /// `U` — `UNICODE_CHARACTER_CLASS`: `\w`, `\d`, `\s`, `\b` and the POSIX
    /// names mean their UNICODE sets rather than the ASCII ones.
    unicode_classes: bool,
    /// `d` — `UNIX_LINES`: only `\n` ends a line, for `.`, `^` and `$`.
    unix_lines: bool,
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
                    if is_line_terminator(u32::from(unit), false) {
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
        let mut counted = true;
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
                counted = false;
                (0, Some(1))
            }
            Some(unit) if unit == u16::from(b'{') => {
                // The JDK points one BEFORE the brace, which is the atom's
                // last character and not its first — the two are the same
                // only while the atom is one character long, so `\b{x}` used
                // to answer 0 where a JDK answers 1.
                let before_brace = self.at.saturating_sub(1);
                match self.parse_bounds()? {
                    Some(bounds) => bounds,
                    // A `{` after a quantifiable atom must open a repetition:
                    // the JDK's `closure` says "Illegal repetition" for
                    // anything else. (A `{` in atom position IS a literal
                    // brace, which `parse_atom` handles.) Treating this one as
                    // a literal accepted `a{x`, which no JDK compiles.
                    None => return Err(self.error("Illegal repetition", before_brace)),
                }
            }
            _ => return Ok(atom),
        };
        // A quantifier must follow something quantifiable.
        if matches!(atom, Node::Start | Node::End(_) | Node::WordBoundary(..)) {
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
                    counted,
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
            counted,
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
                Node::AnyChar(self.flags.unix_lines)
            }),
            u if u == u16::from(b'^') => Ok(if self.flags.multiline {
                Node::LineStart(self.flags.unix_lines)
            } else {
                Node::Start
            }),
            u if u == u16::from(b'$') => Ok(if self.flags.multiline {
                Node::LineEnd(self.flags.unix_lines)
            } else {
                Node::End(self.flags.unix_lines)
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
                items: vec![ClassItem::Single(u32::from(unit))],
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
            if self.eat(u16::from(b'>')) {
                return self.parse_atomic();
            }
            if !self.eat(u16::from(b':')) {
                // Anything else after `(?` is a modifier a JDK does not know
                // either, reported at the offending character (or at the end,
                // if there is none).
                return Err(self.error("Unknown inline modifier", self.at));
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

    /// `(?>X)` — an ATOMIC group. It matches `X` once and then refuses to
    /// reconsider, which is exactly one possessive repetition: the engine
    /// already has that, so the construct needs no node of its own.
    fn parse_atomic(&mut self) -> ParseResult<Node> {
        let saved = self.flags;
        let node = self.parse_alt()?;
        self.flags = saved;
        if !self.eat(u16::from(b')')) {
            return Err(self.error("Unclosed group", self.units.len()));
        }
        Ok(Node::Repeat {
            node: Box::new(Node::Group {
                index: None,
                node: Box::new(node),
            }),
            counted: true,
            min: 1,
            max: Some(1),
            kind: RepeatKind::Possessive,
        })
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
                b'U' => flags.unicode_classes = on,
                b'd' => flags.unix_lines = on,
                // `u` (UNICODE_CASE) widens `(?i)` beyond ASCII; the folding
                // this engine does is ASCII either way.
                b'u' => {}
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
            class.items.push(ClassItem::Single(u32::from(b']')));
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
                ClassEscape::Named { property, negated } => {
                    return Ok(ClassItem::Named { property, negated });
                }
                ClassEscape::Literal(literal) => literal,
            }
        } else {
            u32::from(unit)
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
                    ClassEscape::Predefined(_) | ClassEscape::Named { .. } => {
                        return Err(self.error("Illegal character range", high_start));
                    }
                }
            } else {
                u32::from(high_unit)
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
                // `\b{g}` — a GRAPHEME boundary, which is a different
                // assertion that happens to be spelled with a `\b`. Anything
                // else after the brace is not an error here: the JDK winds
                // back and lets `{...}` fail as the quantifier it looks like.
                if self.peek() == Some(u16::from(b'{')) {
                    let open = self.at;
                    if self.units.get(open + 1) == Some(&u16::from(b'g')) {
                        self.at = open + 2;
                        if self.eat(u16::from(b'}')) {
                            return Ok(Node::GraphemeBound);
                        }
                        return Err(
                            self.error("Illegal/unsupported escape sequence", self.units.len())
                        );
                    }
                }
                Ok(Node::WordBoundary(true, self.flags.unicode_classes))
            }
            u if u == u16::from(b'B') => {
                self.at += 1;
                Ok(Node::WordBoundary(false, self.flags.unicode_classes))
            }
            u if u == u16::from(b'A') => {
                self.at += 1;
                Ok(Node::InputStart)
            }
            u if u == u16::from(b'z') => {
                self.at += 1;
                Ok(Node::InputEnd)
            }
            u if u == u16::from(b'G') => {
                self.at += 1;
                Ok(Node::PreviousEnd)
            }
            u if u == u16::from(b'X') => {
                self.at += 1;
                Ok(Node::Grapheme)
            }
            u if u == u16::from(b'Z') => {
                self.at += 1;
                Ok(Node::InputEndBeforeFinalTerminator(self.flags.unix_lines))
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
                ClassEscape::Literal(literal) => Ok(self.code_point(literal)),
                ClassEscape::Named { property, negated } => Ok(Node::Class(CharClass {
                    fold: false,
                    negated: false,
                    items: vec![ClassItem::Named { property, negated }],
                    intersections: Vec::new(),
                })),
            },
        }
    }

    /// One code point as a node: a unit, or the SURROGATE PAIR a
    /// supplementary one is written as, since the engine walks UTF-16.
    fn code_point(&self, point: u32) -> Node {
        if let Ok(unit) = u16::try_from(point) {
            return self.literal(unit);
        }
        let above = point - 0x1_0000;
        Node::Concat(vec![
            Node::Literal(u16::try_from(0xD800 + (above >> 10)).unwrap_or(u16::MAX)),
            Node::Literal(u16::try_from(0xDC00 + (above & 0x3FF)).unwrap_or(u16::MAX)),
        ])
    }

    /// The escapes meaningful both inside and outside a character class.
    fn parse_class_escape(&mut self, start: usize) -> ParseResult<ClassEscape> {
        let Some(unit) = self.next() else {
            // Inside a class the same truncation is the CLASS's complaint:
            // `[\` never closed.
            let _ = start;
            return Err(self.error("Unclosed character class", self.units.len()));
        };
        let literal: u32 = match unit {
            // Under `(?U)` these three mean their UNICODE sets: `\w` becomes
            // the Word property, `\d` every decimal digit, `\s` every
            // whitespace — which is a different answer, not a wider spelling.
            u if matches!(u8::try_from(u), Ok(b'd' | b'D' | b's' | b'S' | b'w' | b'W'))
                && self.flags.unicode_classes =>
            {
                let letter = u8::try_from(u).unwrap_or(b'w');
                let property = match letter.to_ascii_lowercase() {
                    b'd' => Property::Of(Pred::Digit),
                    b's' => Property::Of(Pred::WhiteSpace),
                    _ => Property::Of(Pred::Word),
                };
                return Ok(ClassEscape::Named {
                    property,
                    negated: letter.is_ascii_uppercase(),
                });
            }
            u if u == u16::from(b'd') => return Ok(ClassEscape::Predefined(Predefined::Digit)),
            u if u == u16::from(b'D') => return Ok(ClassEscape::Predefined(Predefined::NotDigit)),
            u if u == u16::from(b's') => return Ok(ClassEscape::Predefined(Predefined::Space)),
            u if u == u16::from(b'S') => return Ok(ClassEscape::Predefined(Predefined::NotSpace)),
            u if u == u16::from(b'w') => return Ok(ClassEscape::Predefined(Predefined::Word)),
            u if u == u16::from(b'W') => return Ok(ClassEscape::Predefined(Predefined::NotWord)),
            u if u == u16::from(b'h') => {
                return Ok(ClassEscape::Predefined(Predefined::Horizontal));
            }
            u if u == u16::from(b'H') => {
                return Ok(ClassEscape::Predefined(Predefined::NotHorizontal));
            }
            u if u == u16::from(b'p') || u == u16::from(b'P') => {
                let negated = u == u16::from(b'P');
                let property = self.parse_property(start)?;
                return Ok(ClassEscape::Named { property, negated });
            }
            u if u == u16::from(b'v') => return Ok(ClassEscape::Predefined(Predefined::Vertical)),
            u if u == u16::from(b'V') => {
                return Ok(ClassEscape::Predefined(Predefined::NotVertical));
            }
            // `\cX` — the control character X stands for, which is X with bit
            // 6 flipped. A JDK reads the NEXT character whatever it is.
            u if u == u16::from(b'c') => {
                let Some(letter) = self.next() else {
                    return Err(self.error("Illegal control escape sequence", start));
                };
                u32::from(letter ^ 64)
            }
            u if u == u16::from(b'n') => 0x0A,
            u if u == u16::from(b'r') => 0x0D,
            u if u == u16::from(b't') => 0x09,
            u if u == u16::from(b'f') => 0x0C,
            u if u == u16::from(b'a') => 0x07,
            u if u == u16::from(b'e') => 0x1B,
            u if u == u16::from(b'0') => self.parse_octal(start)?,
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
                value
            }
            u if u == u16::from(b'x') => u32::from(self.parse_hex(2, start)?),
            u if u == u16::from(b'u') => u32::from(self.parse_hex(4, start)?),
            // A letter or digit after a backslash with no meaning is an error
            // in Java, not a literal — being permissive here would accept
            // patterns a real JDK refuses.
            other if is_java_word(u32::from(other)) && other != u16::from(b'_') => {
                return Err(self.error(
                    "Illegal/unsupported escape sequence",
                    self.at.saturating_sub(1),
                ));
            }
            other => u32::from(other),
        };
        Ok(ClassEscape::Literal(literal))
    }

    /// `\0n`, `\0nn`, `\0mnn` — an octal escape. The cursor sits past the `0`.
    fn parse_octal(&mut self, start: usize) -> ParseResult<u32> {
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
        Ok(value)
    }

    /// `\p{Name}`, `\p{key=value}`, or the one-letter `\pL`. The cursor sits
    /// just past the `p`.
    fn parse_property(&mut self, start: usize) -> ParseResult<Property> {
        let name = if self.peek() == Some(u16::from(b'{')) {
            self.at += 1;
            let from = self.at;
            while let Some(next) = self.peek() {
                if next == u16::from(b'}') {
                    break;
                }
                self.at += 1;
            }
            if self.peek().is_none() {
                return Err(self.error("Unclosed character family", self.units.len()));
            }
            if self.at == from {
                return Err(self.error("Empty character family", self.at));
            }
            let name = String::from_utf16_lossy(&self.units[from..self.at]);
            self.at += 1; // the `}`
            name
        } else {
            // `\pL` — one character IS the name.
            let Some(letter) = self.next() else {
                return Err(self.error("Unclosed character family", self.units.len()));
            };
            String::from_utf16_lossy(&[letter])
        };
        let _ = start;
        // The JDK reports the LAST character of the family — the `}`, or the
        // single letter — not the one after it.
        let at = self.at.saturating_sub(1);
        resolve_property(&name, self.flags.unicode_classes)
            .ok_or_else(|| self.error(&property_error(&name), at))
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
    /// A CODE POINT — `\x{1F600}` is one, above the BMP.
    Literal(u32),
    Predefined(Predefined),
    Named {
        property: Property,
        negated: bool,
    },
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
        counted: bool,
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
    /// Where `\G` matches.
    since: usize,
}

impl<'a> Matcher<'a> {
    fn over(
        input: &'a [u16],
        start: usize,
        end: usize,
        anchoring: bool,
        transparent: bool,
        since: usize,
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
            since,
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

    /// The code point ENDING at `at`, which is a pair when `at` sits just
    /// after a low surrogate.
    fn point_before(&self, at: usize) -> Option<u32> {
        let low = *self.input.get(at.checked_sub(1)?)?;
        if (0xDC00..0xE000).contains(&low)
            && at >= 2
            && let Some(&high) = self.input.get(at - 2)
            && (0xD800..0xDC00).contains(&high)
        {
            return Some(0x1_0000 + ((u32::from(high) - 0xD800) << 10) + (u32::from(low) - 0xDC00));
        }
        Some(u32::from(low))
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

    fn at_word(&self, at: usize, unicode: bool) -> bool {
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
        self.point_at(at).is_some_and(|point| {
            if unicode {
                Property::Of(Pred::Word).matches(point)
            } else {
                is_java_word(point)
            }
        })
    }

    fn is_boundary(&self, at: usize, unicode: bool) -> bool {
        let (floor, _) = self.word_window();
        let before = at > floor && self.at_word(at - 1, unicode);
        let after = self.at_word(at, unicode);
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
            Node::AnyChar(unix_lines) => match self.code_point_at(pos) {
                Some((point, width)) if !is_line_terminator(point, *unix_lines) => {
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
                // No snapshot around a branch. `java.util.regex` runs every
                // branch against ONE group array and puts a group back only
                // where the group's own tail sees its continuation fail, so a
                // capture made by a branch that then failed is still readable
                // — `(?=(a))?b|a` over "ab" reports group 1 as "a".
                for branch in branches {
                    if let Some(end) = self.run(branch, pos, caps, cont) {
                        return Some(end);
                    }
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
                ..
            } => self.repeat_possessive(node, *min, *max, pos, caps, cont),
            Node::Repeat {
                node,
                counted,
                min,
                max,
                kind,
            } => self.repeat(node, *counted, *min, *max, *kind, 0, None, pos, caps, cont),
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
            Node::End(unix_lines) => {
                // Java's `$` (without MULTILINE) also matches before a final
                // line terminator. Reaching one is what `requireEnd` reports:
                // more input could have changed the answer.
                self.hit_end.set(true);
                if pos == self.anchor_end() || self.at_final_terminator(pos, *unix_lines) {
                    self.require_end.set(true);
                    return self.resume(pos, caps, cont);
                }
                None
            }
            Node::PreviousEnd => {
                if pos == self.since {
                    return self.resume(pos, caps, cont);
                }
                None
            }
            // `\X` — take one code point, then every following one that does
            // NOT start a new cluster. It never gives any of them back.
            Node::Grapheme => {
                let Some((first, width)) = self.code_point_at(pos) else {
                    self.hit_end.set(true);
                    return None;
                };
                let mut before = first;
                let mut at = pos + width;
                while let Some((next, width)) = self.code_point_at(at) {
                    if unicode::grapheme_boundary(before, next) {
                        break;
                    }
                    before = next;
                    at += width;
                }
                self.resume(at, caps, cont)
            }
            Node::GraphemeBound => {
                let (floor, limit) = self.word_window();
                if pos == floor {
                    return self.resume(pos, caps, cont);
                }
                if pos < limit {
                    // Never INSIDE a surrogate pair, and never where the two
                    // code points belong to one cluster.
                    let inside = pos >= 1
                        && (0xD800..0xDC00).contains(&self.input[pos - 1])
                        && (0xDC00..0xE000).contains(&self.input[pos]);
                    let (Some(before), Some((after, _))) =
                        (self.point_before(pos), self.code_point_at(pos))
                    else {
                        return None;
                    };
                    if inside || !unicode::grapheme_boundary(before, after) {
                        return None;
                    }
                } else {
                    self.hit_end.set(true);
                    self.require_end.set(true);
                }
                self.resume(pos, caps, cont)
            }
            Node::InputEnd => {
                self.hit_end.set(true);
                if pos == self.anchor_end() {
                    self.require_end.set(true);
                    return self.resume(pos, caps, cont);
                }
                None
            }
            Node::InputEndBeforeFinalTerminator(unix_lines) => {
                self.hit_end.set(true);
                if pos == self.anchor_end() || self.at_final_terminator(pos, *unix_lines) {
                    self.require_end.set(true);
                    return self.resume(pos, caps, cont);
                }
                None
            }
            // `(?m)`: a line starts at the input's start and just after any
            // terminator; it ends at the input's end and just before one. A
            // CRLF pair is ONE terminator, so `$` sits before the CR.
            Node::LineStart(unix_lines) => {
                // Java's own comment: "Perl does not match ^ at end of input
                // even after newline". So the END of the input is never a line
                // start — which for an EMPTY input is position 0 too, and
                // `"".matches("(?m)^.*$")` is false because of it.
                if pos == self.anchor_end() {
                    return None;
                }
                let after_terminator = pos > self.anchor_start()
                    && is_line_terminator(u32::from(self.input[pos - 1]), *unix_lines)
                    // NOT between a CR and its LF: the pair is ONE terminator.
                    && !(self.input[pos - 1] == 0x0D && self.input.get(pos) == Some(&0x0A));
                if pos == self.anchor_start() || after_terminator {
                    return self.resume(pos, caps, cont);
                }
                None
            }
            Node::LineEnd(unix_lines) => {
                let before_terminator = self
                    .input
                    .get(pos)
                    .is_some_and(|unit| is_line_terminator(u32::from(*unit), *unix_lines))
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
            Node::WordBoundary(wanted, unicode) => {
                if self.is_boundary(pos, *unicode) == *wanted {
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
                let hit = match direction {
                    // Backwards: the body must END at `pos`, and it can begin
                    // no earlier than its widest match allows — or at the
                    // start of the input when that width is unbounded.
                    Look::Behind(width) => {
                        let earliest = match width {
                            Width::Fixed(width) => pos.saturating_sub(*width),
                            Width::Unbounded => 0,
                        };
                        (earliest..=pos)
                            .rev()
                            .any(|start| self.run(node, start, caps, &Cont::EndAt(pos)).is_some())
                    }
                    Look::Ahead => self.run(node, pos, caps, &Cont::Done).is_some(),
                };
                restore(self);
                // Whatever the body CAPTURED is kept, whether the lookaround
                // was positive or negative and whether or not it matched —
                // the body ran against the same group array the rest of the
                // match uses, which is why a group set inside a NEGATIVE
                // lookahead whose body matched (and which therefore failed
                // the branch) is still readable afterwards.
                if hit == *negated {
                    return None;
                }
                self.resume(pos, caps, cont)
            }
        }
    }

    /// Whether `pos` sits just before the input's final line terminator
    /// (`\n`, `\r\n`, or a lone `\r`).
    fn at_final_terminator(&self, pos: usize, unix_lines: bool) -> bool {
        let len = self.anchor_end();
        if pos == len {
            return false;
        }
        if pos + 1 == len && is_line_terminator(u32::from(self.input[pos]), unix_lines) {
            // NOT between a final CR and LF: the pair is ONE terminator, so
            // `$` fires before the CR and nowhere else. Treating the LF as its
            // own terminator gave `"a\r\n".replaceAll("$", "X")` an extra X.
            return !(pos >= 1 && self.input[pos] == 0x0A && self.input[pos - 1] == 0x0D);
        }
        !unix_lines && pos + 2 == len && self.input[pos] == 0x0D && self.input[pos + 1] == 0x0A
    }

    #[allow(clippy::too_many_arguments)]
    fn repeat(
        &self,
        node: &'a Node,
        counted: bool,
        min: u32,
        max: Option<u32>,
        kind: RepeatKind,
        done: u32,
        // Where the iteration that has just finished BEGAN, so the greedy
        // path can re-assert its group. `None` before any has run.
        since: Option<usize>,
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
                counted,
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
            && matches!(node, Node::Literal(_) | Node::Class(_) | Node::AnyChar(_))
        {
            let mut ends = vec![pos];
            let mut taken = 0u32;
            while max.is_none_or(|max| taken < max) {
                if !self.step() {
                    return None;
                }
                let end = *ends.last().expect("seeded with the start");
                let Some(next) = self.run(node, end, caps, &Cont::Done) else {
                    break;
                };
                if next == end {
                    break; // a zero-width body would loop forever
                }
                ends.push(next);
                taken += 1;
            }
            while taken >= min {
                let end = ends[taken as usize];
                if let Some(matched) = self.resume(end, caps, cont) {
                    return Some(matched);
                }
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
                    if counted {
                        Self::unwrite_empty(node, &saved, caps);
                    }
                    return Some(end);
                }
                let landed = self.resume(pos, caps, cont);
                // Only where an OPTIONAL iteration was taken. A fixed count
                // (`(X){3}`) never reaches the backing-off loop that does
                // this, and neither does a range that gave everything back.
                if landed.is_some() && counted && done > min {
                    Self::reassert(node, since, pos, caps);
                }
                landed
            }
            RepeatKind::Reluctant => {
                if let Some(end) = self.resume(pos, caps, cont) {
                    return Some(end);
                }
                take_more(caps)
            }
            // Possessive repeats never enter this path: `run` sends them to
            // `repeat_possessive`, and so no `Cont::Repeat` is ever built for
            // one. Resuming is the conservative fallback.
            RepeatKind::Possessive => self.resume(pos, caps, cont),
        }
    }

    /// An OPTIONAL repetition of a capturing group that consumes nothing —
    /// `((?!x))*`, `()*`, `(\b)*` — leaves the group UNSET where one that
    /// merely could consume nothing (`(x?)*`) leaves it empty. The same
    /// `GroupCurly` is behind both: a zero-length iteration writes the group,
    /// stops the loop, and is then undone by the restore on the way out, so
    /// only the iterations the MINIMUM required are still there.
    fn unwrite_empty(node: &'a Node, before: &Captures, caps: &mut Captures) {
        let Node::Group {
            index: Some(index),
            node: body,
        } = node
        else {
            return;
        };
        if fixed_width(body) == Some(0) {
            caps[*index] = before[*index];
        }
    }

    /// A greedy `(X){m,n}` over a FIXED-WIDTH capturing group writes the
    /// group's boundaries again once the rest of the pattern has matched —
    /// `java.util.regex` compiles that shape to a `GroupCurly`, whose backing
    /// off sets `groups[g] = [i - k, i)` AFTER its continuation returned true
    /// — but only while it still has iterations it could give back.
    /// So the loop that stopped EARLIEST has the last word, even though the
    /// match ran on past it: `((.){1,3})*` over "abcde" ends with group 2 as
    /// "c" — the third character, from the first pass — and not the "e" that
    /// the second pass wrote. A variable-width body compiles to a `Loop`
    /// instead, which does no such thing, so `fixed_width` decides.
    fn reassert(node: &'a Node, since: Option<usize>, pos: usize, caps: &mut Captures) {
        let Some(from) = since else { return };
        let Node::Group {
            index: Some(index),
            node: body,
        } = node
        else {
            return;
        };
        if fixed_width(body).is_some() {
            caps[*index] = Some((from, pos));
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
            // backtrack into — that independence is the "atomic" part. What
            // it captures is written straight through, the FAILED last
            // attempt included: `((a??a{2})?+x+?)*+` leaves the inner group
            // as "aa" even though nothing after it matched.
            let Some(end) = self.run(node, at, caps, &Cont::Done) else {
                break;
            };
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
                // Restored on the way back out. `java.util.regex` does NOT do
                // this — it writes group boundaries into one array and never
                // takes them back — but replicating that exactly would mean
                // replicating its backtracking ORDER too, since what a failed
                // branch leaves behind depends on which branches were tried.
                // The one place the difference shows is a capture inside a
                // lookaround, which is handled where lookarounds are.
                caps[*index] = previous;
                None
            }
            Cont::Repeat {
                node,
                counted,
                min,
                max,
                kind,
                done,
                from,
                parent,
            } => {
                // A body that consumed nothing would loop forever, so the
                // repetition stops there — and it stops even BELOW the
                // minimum, because repeating an empty match again would
                // change nothing, so one empty iteration stands for all the
                // ones still required. That is `java.util.regex`'s own rule,
                // and it decides which iteration a capture is left from:
                // `(a??){3}b` over "aab" runs empty, backs into "a", runs
                // empty, backs into "a", then runs empty and is DONE, so
                // group 1 is "" — where a loop that filled the minimum with
                // empty iterations first would leave "a" behind.
                if pos == *from {
                    return self.resume(pos, caps, parent);
                }
                self.repeat(
                    node,
                    *counted,
                    *min,
                    *max,
                    *kind,
                    *done,
                    Some(*from),
                    pos,
                    caps,
                    parent,
                )
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
    /// Where the PREVIOUS match ended — what `\G` matches at. `None` means
    /// there was none, and `\G` falls back to where the search begins.
    pub since: Option<usize>,
}

impl Bounds {
    #[must_use]
    pub fn whole(input: &[u16]) -> Bounds {
        Bounds {
            start: 0,
            end: input.len(),
            anchoring: true,
            transparent: false,
            since: None,
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

/// Whether a compiled pattern MENTIONS a supplementary code point or a
/// surrogate — `java.util.regex.Pattern.isSupplementary`, which counts a lone
/// surrogate as one. It is what picks `StartS` over `Start`, and so whether the
/// search walks its start positions by code point or by code unit.
fn mentions_supplementary(node: &Node) -> bool {
    let point = |value: u32| value >= 0x1_0000 || (0xD800..0xE000).contains(&value);
    match node {
        Node::Literal(unit) => point(u32::from(*unit)),
        Node::Class(class) => class.mentions_supplementary(),
        Node::Concat(nodes) | Node::Alt(nodes) => nodes.iter().any(mentions_supplementary),
        Node::Repeat { node, .. } | Node::Group { node, .. } | Node::Look { node, .. } => {
            mentions_supplementary(node)
        }
        _ => false,
    }
}

impl CharClass {
    /// Whether any single or range BOUND in the class is one — the bounds are
    /// what a JDK appends, and what it asks about.
    fn mentions_supplementary(&self) -> bool {
        let point = |value: u32| value >= 0x1_0000 || (0xD800..0xE000).contains(&value);
        self.items.iter().any(|item| match item {
            ClassItem::Single(single) => point(*single),
            ClassItem::Range(low, high) => point(*low) || point(*high),
            ClassItem::Nested(nested) => nested.mentions_supplementary(),
            ClassItem::Predefined(_) | ClassItem::Named { .. } => false,
        }) || self
            .intersections
            .iter()
            .any(CharClass::mentions_supplementary)
    }
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
        let steps_by_code_point = mentions_supplementary(&node);
        Ok(Regex {
            node,
            group_count: parser.groups,
            names: parser.names,
            steps_by_code_point,
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
            bounds.since.unwrap_or(from),
        );
        // ONE set of captures for the whole search, not one per start
        // position. `java.util.regex` clears its group array once per
        // `find()` and then walks the start positions itself, so a group that
        // captured during a FAILED attempt at an earlier position is still
        // set when a later position matches — `([abc])*+a*?1+` over
        // "bx xac 10 " matches "1" and reports group 1 as "c". Allocating
        // fresh captures per position is tidier and answers `null` there,
        // which is a different string on the screen.
        let mut caps: Captures = vec![None; self.group_count + 1];
        for start in from..=bounds.end {
            // A pattern that MENTIONS a supplementary code point or a
            // surrogate advances by code point, so the scan never begins
            // inside a pair — which is why `[\uD800-\uDFFF]`, and even a lone
            // low surrogate written as a literal, find nothing in an astral
            // character. A pattern that mentions neither steps by unit, which
            // is why `\X{2}` really does match a flag emoji one unit in.
            //
            // A position asked for OUTRIGHT is still tried either way:
            // `find()` after a zero-width match resumes one UNIT along, which
            // is how `"\u{1F600}".split("")` answers two lone surrogates.
            if self.steps_by_code_point
                && start > from
                && input
                    .get(start.wrapping_sub(1))
                    .is_some_and(|unit| (0xD800..0xDC00).contains(unit))
                && input
                    .get(start)
                    .is_some_and(|unit| (0xDC00..0xE000).contains(unit))
            {
                continue;
            }
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
            bounds.since.unwrap_or(bounds.start),
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
            bounds.since.unwrap_or(bounds.start),
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
