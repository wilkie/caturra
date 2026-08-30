//! Lexer for Java source (JLS §3).
//!
//! Produces a token stream for the parser. Covers the CSA subset plus
//! everything cheap to support alongside it: all Java keywords and
//! operators, integer/floating/char/string/boolean/null literals, and
//! both comment forms.

use crate::diagnostics::{Diagnostic, SourcePosition, SourceSpan};

/// The kind of a token.
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Identifier(String),
    Keyword(Keyword),
    IntLiteral(i64),
    LongLiteral(i64),
    FloatLiteral(f32),
    DoubleLiteral(f64),
    StringLiteral(String),
    /// The literal's UTF-16 code unit, which may be an unpaired surrogate.
    CharLiteral(u16),
    BooleanLiteral(bool),
    NullLiteral,
    /// Operators and punctuation, e.g. `+`, `==`, `{`.
    Symbol(&'static str),
}

/// Java keywords (JLS §3.9) — reserved words only; `true`/`false`/`null`
/// are literals.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum Keyword {
    Abstract,
    Assert,
    Boolean,
    Break,
    Byte,
    Case,
    Catch,
    Char,
    Class,
    Const,
    Continue,
    Default,
    Do,
    Double,
    Else,
    Enum,
    Extends,
    Final,
    Finally,
    Float,
    For,
    Goto,
    If,
    Implements,
    Import,
    Instanceof,
    Int,
    Interface,
    Long,
    Native,
    New,
    Package,
    Private,
    Protected,
    Public,
    Return,
    Short,
    Static,
    Strictfp,
    Super,
    Switch,
    Synchronized,
    This,
    Throw,
    Throws,
    Transient,
    Try,
    Var,
    Void,
    Volatile,
    While,
}

fn keyword_from_str(word: &str) -> Option<Keyword> {
    use Keyword::{
        Abstract, Assert, Boolean, Break, Byte, Case, Catch, Char, Class, Const, Continue, Default,
        Do, Double, Else, Enum, Extends, Final, Finally, Float, For, Goto, If, Implements, Import,
        Instanceof, Int, Interface, Long, Native, New, Package, Private, Protected, Public, Return,
        Short, Static, Strictfp, Super, Switch, Synchronized, This, Throw, Throws, Transient, Try,
        Var, Void, Volatile, While,
    };
    Some(match word {
        "abstract" => Abstract,
        "assert" => Assert,
        "boolean" => Boolean,
        "break" => Break,
        "byte" => Byte,
        "case" => Case,
        "catch" => Catch,
        "char" => Char,
        "class" => Class,
        "const" => Const,
        "continue" => Continue,
        "default" => Default,
        "do" => Do,
        "double" => Double,
        "else" => Else,
        "enum" => Enum,
        "extends" => Extends,
        "final" => Final,
        "finally" => Finally,
        "float" => Float,
        "for" => For,
        "goto" => Goto,
        "if" => If,
        "implements" => Implements,
        "import" => Import,
        "instanceof" => Instanceof,
        "int" => Int,
        "interface" => Interface,
        "long" => Long,
        "native" => Native,
        "new" => New,
        "package" => Package,
        "private" => Private,
        "protected" => Protected,
        "public" => Public,
        "return" => Return,
        "short" => Short,
        "static" => Static,
        "strictfp" => Strictfp,
        "super" => Super,
        "switch" => Switch,
        "synchronized" => Synchronized,
        "this" => This,
        "throw" => Throw,
        "throws" => Throws,
        "transient" => Transient,
        "try" => Try,
        "var" => Var,
        "void" => Void,
        "volatile" => Volatile,
        "while" => While,
        _ => return None,
    })
}

/// A token with its location in the source.
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: SourceSpan,
}

/// Multi-character symbols, longest first so maximal munch works by
/// scanning the table in order (JLS §3.2).
const SYMBOLS: &[&str] = &[
    ">>>=", "<<=", ">>=", ">>>", "...", "==", "!=", "<=", ">=", "&&", "||", "++", "--", "+=", "-=",
    "*=", "/=", "%=", "&=", "|=", "^=", "<<", ">>", "->", "::", "+", "-", "*", "/", "%", "=", "<",
    ">", "!", "&", "|", "^", "~", "?", ":", ";", ",", ".", "(", ")", "{", "}", "[", "]", "@",
];

struct Lexer<'a> {
    path: &'a str,
    chars: Vec<char>,
    pos: usize,
    line: u32,
    column: u32,
    tokens: Vec<Token>,
    errors: Vec<Diagnostic>,
}

/// JLS §3.3, step ONE of translation: every eligible `\uXXXX` escape in the
/// source becomes the character it denotes BEFORE anything is lexed. That is
/// not a string-literal feature — it is a property of the source text — so
/// `\u0022` really does close a string literal, `\u000A` really is a line
/// terminator (ending a `//` comment, and making a char literal illegal),
/// and `\u0061bc` really is the identifier `abc`.
///
/// An escape is eligible only when its backslash is preceded by an EVEN
/// number of backslashes: `"\\u0041"` is a backslash followed by `u0041`,
/// not an `A`. Any number of `u`s may follow the backslash.
///
/// A SURROGATE pair of escapes combines into the one supplementary
/// character it spells. A lone surrogate escape is left in place for the
/// literal lexer, which is the only context where it means anything (and
/// where it still renders as U+FFFD — caturra's tokens hold Rust `char`s,
/// which cannot represent an unpaired surrogate).
fn translate_unicode_escapes(path: &str, text: &str, errors: &mut Vec<Diagnostic>) -> Vec<char> {
    let chars: Vec<char> = text.chars().collect();
    let mut out: Vec<char> = Vec::with_capacity(chars.len());
    // Where a run of backslashes began, so the eligibility of the NEXT one
    // is decided by how many precede it.
    let mut eligible = true;
    let mut index = 0usize;
    // Track the position in the ORIGINAL text for diagnostics.
    let (mut line, mut column) = (1u32, 1u32);
    // The escape at `index`, as (code unit, index just past it).
    let escape_at = |index: usize| -> Option<(u32, usize)> {
        if chars.get(index) != Some(&'\\') || chars.get(index + 1) != Some(&'u') {
            return None;
        }
        let mut at = index + 1;
        while chars.get(at) == Some(&'u') {
            at += 1;
        }
        let mut value = 0u32;
        for offset in 0..4 {
            let digit = chars.get(at + offset)?.to_digit(16)?;
            value = value * 16 + digit;
        }
        Some((value, at + 4))
    };
    while index < chars.len() {
        let current = chars[index];
        if current == '\\' && eligible && chars.get(index + 1) == Some(&'u') {
            match escape_at(index) {
                Some((value, next)) => {
                    // A high surrogate followed by a low one is a single
                    // supplementary character.
                    let paired = (0xD800..=0xDBFF)
                        .contains(&value)
                        .then(|| escape_at(next))
                        .flatten();
                    if let Some((low, after)) = paired
                        && (0xDC00..=0xDFFF).contains(&low)
                    {
                        let combined = 0x10000 + ((value - 0xD800) << 10) + (low - 0xDC00);
                        if let Some(character) = char::from_u32(combined) {
                            out.push(character);
                            index = after;
                            eligible = true;
                            continue;
                        }
                    }
                    // An unpaired surrogate has no `char`: leave the escape
                    // for the literal lexer, the only place it can mean
                    // anything.
                    if let Some(character) = char::from_u32(value) {
                        out.push(character);
                    } else {
                        out.extend(&chars[index..next]);
                    }
                    index = next;
                    eligible = true;
                    continue;
                }
                None => {
                    errors.push(Diagnostic::error(
                        path,
                        "illegal unicode escape",
                        SourceSpan {
                            start: SourcePosition { line, column },
                            end: SourcePosition { line, column },
                        },
                    ));
                }
            }
        }
        if current == '\\' {
            eligible = !eligible;
        } else {
            eligible = true;
        }
        if current == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
        out.push(current);
        index += 1;
    }
    out
}

/// Tokenize a source file. Always returns the tokens it could produce;
/// lexical problems are reported as diagnostics alongside.
#[must_use]
pub fn lex(path: &str, text: &str) -> (Vec<Token>, Vec<Diagnostic>) {
    let mut errors = Vec::new();
    let chars = translate_unicode_escapes(path, text, &mut errors);
    let mut lexer = Lexer {
        path,
        chars,
        pos: 0,
        line: 1,
        column: 1,
        tokens: Vec::new(),
        errors,
    };
    lexer.run();
    (lexer.tokens, lexer.errors)
}

impl Lexer<'_> {
    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn peek_at(&self, offset: usize) -> Option<char> {
        self.chars.get(self.pos + offset).copied()
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.pos += 1;
        if c == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        Some(c)
    }

    fn position(&self) -> SourcePosition {
        SourcePosition {
            line: self.line,
            column: self.column,
        }
    }

    fn error(&mut self, message: impl Into<String>, start: SourcePosition) {
        let span = SourceSpan {
            start,
            end: self.position(),
        };
        self.errors
            .push(Diagnostic::error(self.path, message, span));
    }

    /// Whether the last token is a `-` that must be UNARY: nothing before it,
    /// or something that cannot end an expression (`(`, `,`, `=`, an operator,
    /// `return`, …). A literal, identifier, or closing bracket before the `-`
    /// makes it binary subtraction instead.
    fn trailing_unary_minus(&self) -> bool {
        let [.., before, minus] = self.tokens.as_slice() else {
            // A lone leading `-` (the whole file starts with it) is unary too.
            return matches!(
                self.tokens.as_slice(),
                [Token {
                    kind: TokenKind::Symbol("-"),
                    ..
                }]
            );
        };
        if minus.kind != TokenKind::Symbol("-") {
            return false;
        }
        !matches!(
            before.kind,
            TokenKind::Identifier(_)
                | TokenKind::IntLiteral(_)
                | TokenKind::LongLiteral(_)
                | TokenKind::FloatLiteral(_)
                | TokenKind::DoubleLiteral(_)
                | TokenKind::CharLiteral(_)
                | TokenKind::StringLiteral(_)
                | TokenKind::Symbol(")" | "]")
        )
    }

    fn push(&mut self, kind: TokenKind, start: SourcePosition) {
        let span = SourceSpan {
            start,
            end: self.position(),
        };
        self.tokens.push(Token { kind, span });
    }

    fn run(&mut self) {
        while let Some(c) = self.peek() {
            let start = self.position();
            if c.is_whitespace() {
                self.bump();
            } else if c == '/' && self.peek_at(1) == Some('/') {
                while self.peek().is_some_and(|c| c != '\n') {
                    self.bump();
                }
            } else if c == '/' && self.peek_at(1) == Some('*') {
                self.block_comment(start);
            } else if c.is_alphabetic() || c == '_' || c == '$' {
                self.word(start);
            } else if c.is_ascii_digit()
                || (c == '.' && self.peek_at(1).is_some_and(|d| d.is_ascii_digit()))
            {
                self.number(start);
            } else if c == '"' {
                self.string_literal(start);
            } else if c == '\'' {
                self.char_literal(start);
            } else if !self.symbol(start) {
                self.bump();
                self.error(format!("unexpected character '{c}'"), start);
            }
        }
    }

    fn block_comment(&mut self, start: SourcePosition) {
        self.bump();
        self.bump();
        loop {
            match self.bump() {
                None => {
                    self.error("unterminated block comment", start);
                    return;
                }
                Some('*') if self.peek() == Some('/') => {
                    self.bump();
                    return;
                }
                Some(_) => {}
            }
        }
    }

    fn word(&mut self, start: SourcePosition) {
        let mut word = String::new();
        while self
            .peek()
            .is_some_and(|c| c.is_alphanumeric() || c == '_' || c == '$')
        {
            word.push(self.bump().expect("peeked"));
        }
        let kind = match word.as_str() {
            "true" => TokenKind::BooleanLiteral(true),
            "false" => TokenKind::BooleanLiteral(false),
            "null" => TokenKind::NullLiteral,
            _ => keyword_from_str(&word).map_or(TokenKind::Identifier(word), TokenKind::Keyword),
        };
        self.push(kind, start);
    }

    #[allow(clippy::too_many_lines)] // one literal grammar
    fn number(&mut self, start: SourcePosition) {
        // Hex, binary, and octal integer literals (with underscores).
        // Java's rules: `0x1F`, `0b1010`, and a leading zero followed
        // by digits is octal (`0777`); a lone `0`, `0.5`, and `0e3`
        // stay decimal.
        if self.peek() == Some('0') {
            let radix = match self.peek_at(1) {
                Some('x' | 'X') => Some((16u32, 2usize)),
                Some('b' | 'B') => Some((2, 2)),
                Some(c) if c.is_ascii_digit() || c == '_' => Some((8, 1)),
                _ => None,
            };
            if let Some((radix, prefix_len)) = radix {
                for _ in 0..prefix_len {
                    self.bump();
                }
                // A HEX FLOAT (`0x1.fp3`): hex digits, an optional fraction,
                // and a MANDATORY binary exponent. Decided by lookahead,
                // since the integer scan below would stop at the `.` and
                // leave `.fp3` to be read as a field access.
                if radix == 16 {
                    let mut ahead = 0usize;
                    while self
                        .peek_at(ahead)
                        .is_some_and(|c| c.is_ascii_hexdigit() || c == '_')
                    {
                        ahead += 1;
                    }
                    if matches!(self.peek_at(ahead), Some('.' | 'p' | 'P')) {
                        self.hex_float(start);
                        return;
                    }
                }
                let mut digits = String::new();
                while self
                    .peek()
                    .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
                {
                    digits.push(self.bump().expect("peeked"));
                }
                if !self.check_underscores(&digits, radix == 16, start) {
                    return;
                }
                let mut cleaned = digits.replace('_', "");
                let is_long = cleaned.ends_with('L') || cleaned.ends_with('l');
                if is_long {
                    cleaned.pop();
                }
                if is_long {
                    // Long literals may fill all 64 bits.
                    match u64::from_str_radix(&cleaned, radix) {
                        Ok(value) => {
                            self.push(TokenKind::LongLiteral(value.cast_signed()), start);
                        }
                        Err(_) => {
                            self.error(
                                format!("integer literal '0{digits}' is malformed or out of range"),
                                start,
                            );
                        }
                    }
                    return;
                }
                // Hex/binary literals may fill all 32 bits (0xFFFFFFFF
                // is -1); parse unsigned then reinterpret.
                match u32::from_str_radix(&cleaned, radix) {
                    Ok(value) => {
                        self.push(TokenKind::IntLiteral(i64::from(value.cast_signed())), start);
                    }
                    Err(_) => {
                        self.error(
                            format!("integer literal '0{digits}' is malformed or out of range"),
                            start,
                        );
                    }
                }
                return;
            }
        }
        let mut digits = String::new();
        let mut is_double = false;
        // A leading `.` (no integer part): `.5` is `0.5`.
        if self.peek() == Some('.') {
            digits.push('0');
        }
        while self.peek().is_some_and(|c| c.is_ascii_digit() || c == '_') {
            digits.push(self.bump().expect("peeked"));
        }
        // The fraction digits are OPTIONAL (JLS §3.10.2): `5.`, `5.d` and
        // `5.e2` are all doubles. Only a number can reach here — a `.` that
        // begins a member access follows an identifier, not a digit — so
        // consuming it unconditionally cannot swallow a field selector.
        if self.peek() == Some('.') {
            is_double = true;
            digits.push(self.bump().expect("peeked"));
            while self.peek().is_some_and(|c| c.is_ascii_digit() || c == '_') {
                digits.push(self.bump().expect("peeked"));
            }
        }
        // Exponents: `1e10`, `2.5E-3`. Only when followed by digits (or
        // a signed digit), so `1exit` stays `1` + identifier.
        if matches!(self.peek(), Some('e' | 'E')) {
            let exponent_digits_follow = self.peek_at(1).is_some_and(|c| c.is_ascii_digit())
                || (matches!(self.peek_at(1), Some('+' | '-'))
                    && self.peek_at(2).is_some_and(|c| c.is_ascii_digit()));
            if exponent_digits_follow {
                is_double = true;
                digits.push(self.bump().expect("peeked"));
                if matches!(self.peek(), Some('+' | '-')) {
                    digits.push(self.bump().expect("peeked"));
                }
                while self.peek().is_some_and(|c| c.is_ascii_digit() || c == '_') {
                    digits.push(self.bump().expect("peeked"));
                }
            }
        }
        // Type suffixes: L (long), d/D (double), f/F (float — not yet
        // a supported type).
        let mut force_long = false;
        match self.peek() {
            Some('L' | 'l') if !is_double => {
                self.bump();
                force_long = true;
            }
            Some('d' | 'D') => {
                self.bump();
                is_double = true;
            }
            Some('f' | 'F') => {
                self.bump();
                if !self.check_underscores(&digits, false, start) {
                    return;
                }
                let digits = digits.replace('_', "");
                match digits.parse::<f32>() {
                    Ok(value) if self.float_in_range(f64::from(value), &digits, start) => {
                        self.push(TokenKind::FloatLiteral(value), start);
                    }
                    Ok(_) => {}
                    Err(_) => {
                        self.error(format!("invalid float literal '{digits}'"), start);
                    }
                }
                return;
            }
            _ => {}
        }
        if !self.check_underscores(&digits, false, start) {
            return;
        }
        let digits = digits.replace('_', "");
        if force_long {
            match digits.parse::<i64>() {
                Ok(value) => self.push(TokenKind::LongLiteral(value), start),
                // JLS §3.10.1: `9223372036854775808L` — one past `Long.MAX_VALUE`
                // — is legal in EXACTLY one place: as the operand of unary
                // minus, where the pair spells `Long.MIN_VALUE`. The minus is
                // unary when what precedes it cannot end an expression, which
                // is the same test every lexer uses to split the two minuses.
                // The `-` token is folded away and the literal becomes MIN.
                Err(_) if digits == "9223372036854775808" && self.trailing_unary_minus() => {
                    let minus = self.tokens.pop().expect("checked by trailing_unary_minus");
                    self.tokens.push(Token {
                        kind: TokenKind::LongLiteral(i64::MIN),
                        span: SourceSpan {
                            start: minus.span.start,
                            end: self.position(),
                        },
                    });
                }
                // javac's own wording, which names no number: the caret it
                // prints under the literal is what points at it.
                Err(_) => {
                    let _ = &digits;
                    self.error(String::from("integer number too large"), start);
                }
            }
        } else if is_double {
            match digits.parse::<f64>() {
                Ok(value) if self.float_in_range(value, &digits, start) => {
                    self.push(TokenKind::DoubleLiteral(value), start);
                }
                Ok(_) => {}
                Err(_) => self.error(format!("invalid floating-point literal '{digits}'"), start),
            }
        } else {
            // javac's own wording for a literal its type cannot hold, which
            // names no number: the caret it prints under the literal is what
            // points at it.
            if let Ok(value) = digits.parse::<i64>() {
                self.push(TokenKind::IntLiteral(value), start);
            } else {
                self.error(String::from("integer number too large"), start);
            }
        }
    }

    /// JLS §3.10.2: a floating literal that does not FIT its type is a
    /// compile error — "too large" when it rounds to an infinity, and "too
    /// small" when a nonzero significand rounds all the way to zero (`0e-400`
    /// is fine: its significand is zero). Reports the error and answers
    /// whether the literal is usable.
    fn float_in_range(&mut self, value: f64, text: &str, start: SourcePosition) -> bool {
        if value.is_infinite() {
            self.error("floating point number too large", start);
            return false;
        }
        let significand = text
            .split_once(['e', 'E', 'p', 'P'])
            .map_or(text, |(mantissa, _)| mantissa);
        let nonzero = significand
            .chars()
            .any(|c| c.is_ascii_hexdigit() && c != '0');
        if value == 0.0 && nonzero {
            self.error("floating point number too small", start);
            return false;
        }
        true
    }

    /// A hexadecimal floating-point literal (`0x1.fp3` = 15.5), positioned
    /// just past the `0x`. The binary `p` exponent is mandatory — `0x1.f`
    /// alone is javac's "malformed floating point literal".
    fn hex_float(&mut self, start: SourcePosition) {
        let mut integer = String::new();
        while self
            .peek()
            .is_some_and(|c| c.is_ascii_hexdigit() || c == '_')
        {
            integer.push(self.bump().expect("peeked"));
        }
        let mut fraction = String::new();
        if self.peek() == Some('.') {
            self.bump();
            while self
                .peek()
                .is_some_and(|c| c.is_ascii_hexdigit() || c == '_')
            {
                fraction.push(self.bump().expect("peeked"));
            }
        }
        let malformed = "malformed floating point literal";
        if !matches!(self.peek(), Some('p' | 'P')) {
            self.error(malformed, start);
            return;
        }
        self.bump();
        let mut exponent = String::new();
        if matches!(self.peek(), Some('+' | '-')) {
            exponent.push(self.bump().expect("peeked"));
        }
        while self.peek().is_some_and(|c| c.is_ascii_digit() || c == '_') {
            exponent.push(self.bump().expect("peeked"));
        }
        let is_float = match self.peek() {
            Some('f' | 'F') => {
                self.bump();
                true
            }
            Some('d' | 'D') => {
                self.bump();
                false
            }
            _ => false,
        };
        let (integer, fraction) = (integer.replace('_', ""), fraction.replace('_', ""));
        let exponent = exponent.replace('_', "");
        if (integer.is_empty() && fraction.is_empty()) || exponent.is_empty() {
            self.error(malformed, start);
            return;
        }
        let Ok(exponent) = exponent.parse::<i32>() else {
            self.error(malformed, start);
            return;
        };
        let mut value = 0f64;
        for c in integer.chars() {
            value = value * 16.0 + f64::from(c.to_digit(16).expect("hex digit"));
        }
        let mut scale = 1.0 / 16.0;
        for c in fraction.chars() {
            value += f64::from(c.to_digit(16).expect("hex digit")) * scale;
            scale /= 16.0;
        }
        value *= 2f64.powi(exponent);
        // The significand for the range check is the hex digits themselves.
        let significand = format!("{integer}{fraction}");
        if is_float {
            #[allow(clippy::cast_possible_truncation)]
            let narrowed = value as f32;
            if self.float_in_range(f64::from(narrowed), &significand, start) {
                self.push(TokenKind::FloatLiteral(narrowed), start);
            }
            return;
        }
        if self.float_in_range(value, &significand, start) {
            self.push(TokenKind::DoubleLiteral(value), start);
        }
    }

    /// JLS §3.10.1: an underscore may appear only BETWEEN digits — never at
    /// either end of a run, and never touching a radix prefix, a decimal
    /// point, an exponent marker or a type suffix. caturra stripped them
    /// anywhere, so `1_`, `0x_FF` and `10_L` all compiled.
    fn check_underscores(&mut self, text: &str, hex: bool, start: SourcePosition) -> bool {
        let chars: Vec<char> = text.chars().collect();
        let digit = |c: Option<&char>| {
            c.is_some_and(|c| {
                if hex {
                    c.is_ascii_hexdigit()
                } else {
                    c.is_ascii_digit()
                }
            })
        };
        for (at, c) in chars.iter().enumerate() {
            if *c != '_' {
                continue;
            }
            // Runs of underscores are legal INSIDE a number (`1__0`), so
            // look past them on both sides.
            let mut before = at;
            while before > 0 && chars[before - 1] == '_' {
                before -= 1;
            }
            let mut after = at;
            while chars.get(after + 1) == Some(&'_') {
                after += 1;
            }
            if !(before > 0 && digit(chars.get(before - 1)) && digit(chars.get(after + 1))) {
                self.error("illegal underscore", start);
                return false;
            }
        }
        true
    }

    /// An escape as a `char`, for a STRING literal — whose token is a Rust
    /// `String`, so an unpaired surrogate there still becomes U+FFFD.
    fn escape(&mut self, start: SourcePosition) -> Option<char> {
        let unit = self.escape_unit(start)?;
        Some(char::from_u32(unit).unwrap_or('\u{FFFD}'))
    }

    /// An escape as its raw code UNIT. A `\uD83D` is a perfectly legal char
    /// literal denoting an unpaired surrogate, and no Rust `char` can hold
    /// one — so a char literal is carried as a number all the way through.
    fn escape_unit(&mut self, start: SourcePosition) -> Option<u32> {
        match self.bump() {
            Some('n') => Some(0x0A),
            Some('t') => Some(0x09),
            Some('r') => Some(0x0D),
            Some('b') => Some(0x08),
            Some('f') => Some(0x0C),
            // OCTAL escapes (JLS §3.10.6): one to three octal digits, at
            // most \377 — a three-digit form needs a leading 0-3, so `\400`
            // is `\40` followed by a literal '0'. `\0` is just its
            // one-digit case.
            Some(first @ '0'..='7') => {
                let mut value = first.to_digit(8).expect("octal digit");
                let mut digits = 1;
                while digits < 3
                    && let Some(next) = self.peek()
                    && let Some(digit) = next.to_digit(8)
                {
                    // The third digit only fits when the first is 0-3.
                    if digits == 2 && first > '3' {
                        break;
                    }
                    value = value * 8 + digit;
                    self.bump();
                    digits += 1;
                }
                Some(value)
            }
            Some('\\') => Some(u32::from(b'\\')),
            Some('\'') => Some(u32::from(b'\'')),
            Some('"') => Some(u32::from(b'"')),
            Some('u') => {
                let mut code = String::new();
                for _ in 0..4 {
                    match self.bump() {
                        Some(c) if c.is_ascii_hexdigit() => code.push(c),
                        _ => {
                            self.error("malformed \\u escape (needs 4 hex digits)", start);
                            return None;
                        }
                    }
                }
                Some(u32::from_str_radix(&code, 16).expect("hex digits"))
            }
            Some(other) => {
                self.error(format!("unknown escape sequence '\\{other}'"), start);
                None
            }
            None => {
                self.error("unterminated escape sequence", start);
                None
            }
        }
    }

    fn string_literal(&mut self, start: SourcePosition) {
        self.bump();
        let mut value = String::new();
        loop {
            match self.peek() {
                None | Some('\n') => {
                    // javac's word is "unclosed", and its four literal
                    // messages are four DIFFERENT sentences: an unclosed
                    // string, an unclosed character, an EMPTY character, and a
                    // newline inside one. A student searches the sentence.
                    self.error("unclosed string literal", start);
                    return;
                }
                Some('"') => {
                    self.bump();
                    self.push(TokenKind::StringLiteral(value), start);
                    return;
                }
                Some('\\') => {
                    self.bump();
                    if let Some(c) = self.escape(start) {
                        value.push(c);
                    }
                }
                Some(_) => value.push(self.bump().expect("peeked")),
            }
        }
    }

    fn char_literal(&mut self, start: SourcePosition) {
        self.bump();
        let value: Option<u32> = match self.peek() {
            // `''` is EMPTY; a newline (or the end of the file) inside one is
            // an illegal line end. javac tells them apart, and so must this.
            Some('\'') => {
                self.bump();
                self.error("empty character literal", start);
                return;
            }
            None | Some('\n') => {
                self.bump();
                self.error("illegal line end in character literal", start);
                return;
            }
            Some('\\') => {
                self.bump();
                self.escape_unit(start)
            }
            Some(_) => self.bump().map(u32::from),
        };
        if self.peek() == Some('\'') {
            self.bump();
            if let Some(unit) = value {
                self.push(
                    TokenKind::CharLiteral(u16::try_from(unit).unwrap_or(u16::MAX)),
                    start,
                );
            }
        } else {
            self.error("unclosed character literal", start);
        }
    }

    fn symbol(&mut self, start: SourcePosition) -> bool {
        for symbol in SYMBOLS {
            let matches = symbol
                .chars()
                .enumerate()
                .all(|(i, expected)| self.peek_at(i) == Some(expected));
            if matches {
                for _ in 0..symbol.len() {
                    self.bump();
                }
                self.push(TokenKind::Symbol(symbol), start);
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(text: &str) -> Vec<TokenKind> {
        let (tokens, errors) = lex("Test.java", text);
        assert!(errors.is_empty(), "unexpected errors: {errors:?}");
        tokens.into_iter().map(|t| t.kind).collect()
    }

    #[test]
    fn lexes_hello_world() {
        let source = r#"
            public class Main {
                public static void main(String[] args) {
                    System.out.println("Hello, World!");
                }
            }
        "#;
        let tokens = kinds(source);
        assert!(tokens.contains(&TokenKind::Keyword(Keyword::Class)));
        assert!(tokens.contains(&TokenKind::Identifier(String::from("println"))));
        assert!(tokens.contains(&TokenKind::StringLiteral(String::from("Hello, World!"))));
        assert!(tokens.contains(&TokenKind::Symbol("[")));
    }

    #[test]
    fn lexes_literals() {
        assert_eq!(
            kinds("42 3.5 'a' '\\n' true false null"),
            vec![
                TokenKind::IntLiteral(42),
                TokenKind::DoubleLiteral(3.5),
                TokenKind::CharLiteral(u16::from(b'a')),
                TokenKind::CharLiteral(0x0A),
                TokenKind::BooleanLiteral(true),
                TokenKind::BooleanLiteral(false),
                TokenKind::NullLiteral,
            ]
        );
    }

    #[test]
    fn lexes_exponent_literals() {
        assert_eq!(
            kinds("1e3 2.5E-2 1e+2"),
            vec![
                TokenKind::DoubleLiteral(1000.0),
                TokenKind::DoubleLiteral(0.025),
                TokenKind::DoubleLiteral(100.0),
            ]
        );
        // 'e' not followed by digits stays an identifier boundary.
        assert_eq!(
            kinds("1e"),
            vec![
                TokenKind::IntLiteral(1),
                TokenKind::Identifier(String::from("e")),
            ]
        );
    }

    #[test]
    fn maximal_munch_on_operators() {
        assert_eq!(
            kinds("a >>= b >= c > d"),
            vec![
                TokenKind::Identifier(String::from("a")),
                TokenKind::Symbol(">>="),
                TokenKind::Identifier(String::from("b")),
                TokenKind::Symbol(">="),
                TokenKind::Identifier(String::from("c")),
                TokenKind::Symbol(">"),
                TokenKind::Identifier(String::from("d")),
            ]
        );
    }

    #[test]
    fn member_access_after_int_is_not_a_double() {
        assert_eq!(
            kinds("x.size()"),
            vec![
                TokenKind::Identifier(String::from("x")),
                TokenKind::Symbol("."),
                TokenKind::Identifier(String::from("size")),
                TokenKind::Symbol("("),
                TokenKind::Symbol(")"),
            ]
        );
    }

    #[test]
    fn comments_are_skipped() {
        assert_eq!(
            kinds("int x; // trailing\n/* block\n comment */ int y;"),
            vec![
                TokenKind::Keyword(Keyword::Int),
                TokenKind::Identifier(String::from("x")),
                TokenKind::Symbol(";"),
                TokenKind::Keyword(Keyword::Int),
                TokenKind::Identifier(String::from("y")),
                TokenKind::Symbol(";"),
            ]
        );
    }

    #[test]
    fn reports_unterminated_string_with_location() {
        let (_, errors) = lex("Main.java", "String s = \"oops;\nint x;");
        assert_eq!(errors.len(), 1);
        assert_eq!(
            errors[0].span.unwrap().start,
            SourcePosition {
                line: 1,
                column: 12
            }
        );
    }

    #[test]
    fn tracks_line_and_column() {
        let (tokens, _) = lex("Main.java", "int\n  x;");
        assert_eq!(tokens[1].span.start, SourcePosition { line: 2, column: 3 });
    }
}
