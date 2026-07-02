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
    DoubleLiteral(f64),
    StringLiteral(String),
    CharLiteral(char),
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

/// Tokenize a source file. Always returns the tokens it could produce;
/// lexical problems are reported as diagnostics alongside.
#[must_use]
pub fn lex(path: &str, text: &str) -> (Vec<Token>, Vec<Diagnostic>) {
    let mut lexer = Lexer {
        path,
        chars: text.chars().collect(),
        pos: 0,
        line: 1,
        column: 1,
        tokens: Vec::new(),
        errors: Vec::new(),
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
            } else if c.is_ascii_digit() {
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

    fn number(&mut self, start: SourcePosition) {
        let mut digits = String::new();
        let mut is_double = false;
        while self.peek().is_some_and(|c| c.is_ascii_digit() || c == '_') {
            digits.push(self.bump().expect("peeked"));
        }
        // A '.' starts a fraction only when followed by a digit, so
        // `list.size()` and `1..2` don't confuse the lexer.
        if self.peek() == Some('.') && self.peek_at(1).is_some_and(|c| c.is_ascii_digit()) {
            is_double = true;
            digits.push(self.bump().expect("peeked"));
            while self.peek().is_some_and(|c| c.is_ascii_digit() || c == '_') {
                digits.push(self.bump().expect("peeked"));
            }
        }
        // TODO: exponents (1e9), hex/binary/octal literals, and the
        // L/f/d suffixes.
        let digits = digits.replace('_', "");
        if is_double {
            match digits.parse::<f64>() {
                Ok(value) => self.push(TokenKind::DoubleLiteral(value), start),
                Err(_) => self.error(format!("invalid floating-point literal '{digits}'"), start),
            }
        } else {
            match digits.parse::<i64>() {
                Ok(value) => self.push(TokenKind::IntLiteral(value), start),
                Err(_) => self.error(format!("integer literal '{digits}' is out of range"), start),
            }
        }
    }

    fn escape(&mut self, start: SourcePosition) -> Option<char> {
        match self.bump() {
            Some('n') => Some('\n'),
            Some('t') => Some('\t'),
            Some('r') => Some('\r'),
            Some('b') => Some('\u{8}'),
            Some('f') => Some('\u{c}'),
            Some('0') => Some('\0'),
            Some('\\') => Some('\\'),
            Some('\'') => Some('\''),
            Some('"') => Some('"'),
            // TODO: \uXXXX escapes and octal escapes.
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
                    self.error("unterminated string literal", start);
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
        let value = match self.peek() {
            None | Some('\n' | '\'') => {
                self.bump();
                self.error("empty or unterminated character literal", start);
                return;
            }
            Some('\\') => {
                self.bump();
                self.escape(start)
            }
            Some(_) => self.bump(),
        };
        if self.peek() == Some('\'') {
            self.bump();
            if let Some(c) = value {
                self.push(TokenKind::CharLiteral(c), start);
            }
        } else {
            self.error("unterminated character literal (expected closing ')", start);
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
                TokenKind::CharLiteral('a'),
                TokenKind::CharLiteral('\n'),
                TokenKind::BooleanLiteral(true),
                TokenKind::BooleanLiteral(false),
                TokenKind::NullLiteral,
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
