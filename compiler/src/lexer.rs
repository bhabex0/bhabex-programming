//! Bhabex Lexer - High-performance tokenizer using logos
//! 
//! Copyright BHABEX 2026 | Securely designed & developed by bxLabs | Protected by bxSecurity | Powered by BHABEX®

use logos::{Logos, Lexer as LogosLexer, Span as LogosSpan};
use std::fmt;
use crate::diagnostics::{Diagnostic, DiagnosticSeverity, DiagnosticBag};

/// Source code location span
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}

impl Span {
    pub fn new(start: usize, end: usize, line: usize, column: usize) -> Self {
        Self { start, end, line, column }
    }

    pub fn merge(self, other: Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
            line: self.line,
            column: self.column,
        }
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

impl From<LogosSpan> for Span {
    fn from(span: LogosSpan) -> Self {
        Span {
            start: span.start,
            end: span.end,
            line: 0,
            column: 0,
        }
    }
}

/// Token kinds for Bhabex language
#[derive(Logos, Debug, Clone, PartialEq, Eq, Hash)]
#[logos(skip r"[ \t\r\n\f]+", skip r"//[^\n]*", skip r"/\*([^*]|\*[^/])*\*/")]
pub enum TokenKind {
    // Keywords
    #[token("fn")] Fn,
    #[token("let")] Let,
    #[token("mut")] Mut,
    #[token("const")] Const,
    #[token("struct")] Struct,
    #[token("enum")] Enum,
    #[token("trait")] Trait,
    #[token("impl")] Impl,
    #[token("type")] Type,
    #[token("if")] If,
    #[token("else")] Else,
    #[token("match")] Match,
    #[token("for")] For,
    #[token("while")] While,
    #[token("loop")] Loop,
    #[token("break")] Break,
    #[token("continue")] Continue,
    #[token("return")] Return,
    #[token("async")] Async,
    #[token("await")] Await,
    #[token("spawn")] Spawn,
    #[token("try")] Try,
    #[token("catch")] Catch,
    #[token("throw")] Throw,
    #[token("import")] Import,
    #[token("export")] Export,
    #[token("pub")] Pub,
    #[token("use")] Use,
    #[token("as")] As,
    #[token("in")] In,
    #[token("where")] Where,
    #[token("Self")] Self_,
    #[token("self")] SelfLower,
    #[token("super")] Super,
    #[token("crate")] Crate,
    #[token("macro")] Macro,
    #[token("macro_rules")] MacroRules,

    // Types
    #[token("i8")] I8,
    #[token("i16")] I16,
    #[token("i32")] I32,
    #[token("i64")] I64,
    #[token("i128")] I128,
    #[token("isize")] Isize,
    #[token("u8")] U8,
    #[token("u16")] U16,
    #[token("u32")] U32,
    #[token("u64")] U64,
    #[token("u128")] U128,
    #[token("usize")] Usize,
    #[token("f32")] F32,
    #[token("f64")] F64,
    #[token("bool")] Bool,
    #[token("char")] CharToken,
    #[token("str")] Str,
    #[token("string")] String_,
    #[token("never")] Never,
    #[token("()")] Unit,

    // Literals
    #[token("true")] True,
    #[token("false")] False,
    #[token("null")] Null,

    // Operators
    #[token("+")] Plus,
    #[token("-")] Minus,
    #[token("*")] Star,
    #[token("/")] Slash,
    #[token("%")] Percent,
    #[token("**")] Pow,
    #[token("+=")] PlusEq,
    #[token("-=")] MinusEq,
    #[token("*=")] StarEq,
    #[token("/=")] SlashEq,
    #[token("%=")] PercentEq,
    #[token("==")] EqEq,
    #[token("!=")] Ne,
    #[token("<")] Lt,
    #[token(">")] Gt,
    #[token("<=")] Le,
    #[token(">=")] Ge,
    #[token("&&")] And,
    #[token("||")] Or,
    #[token("!")] Not,
    #[token("&")] BitAnd,
    #[token("|")] BitOr,
    #[token("^")] BitXor,
    #[token("~")] BitNot,
    #[token("<<")] Shl,
    #[token(">>")] Shr,
    #[token("<<=")] ShlEq,
    #[token(">>=")] ShrEq,
    #[token("&=")] BitAndEq,
    #[token("|=")] BitOrEq,
    #[token("^=")] BitXorEq,
    #[token("=")] Assign,
    #[token("->")] Arrow,
    #[token("=>")] FatArrow,
    #[token(".")] Dot,
    #[token("..")] Range,
    #[token("..=")] RangeInclusive,
    #[token("...")] Spread,
    #[token(",")] Comma,
    #[token(";")] Semi,
    #[token(":")] Colon,
    #[token("::")] DoubleColon,
    #[token("?")] Question,
    #[token("!")] Bang,

    // Delimiters
    #[token("(")] LParen,
    #[token(")")] RParen,
    #[token("{")] LBrace,
    #[token("}")] RBrace,
    #[token("[")] LBracket,
    #[token("]")] RBracket,
    #[token("@")] At,
    #[token("#")] Hash,
    #[token("$")] Dollar,

    // Identifiers and literals
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*")] Ident,

    #[regex(r"[0-9][0-9_]*\.[0-9][0-9_]*([eE][+-]?[0-9][0-9_]*)?")] Float,

    #[regex(r"0x[0-9a-fA-F_]+")] HexInt,

    #[regex(r"0o[0-7_]+")] OctInt,

    #[regex(r"0b[01_]+")] BinInt,

    #[regex(r"[0-9][0-9_]*")] Int,

    #[regex(r#""([^"\\]|\\.)*""#)] String,

    #[regex(r"'([^'\\]|\\.)*'")] Char,

    #[regex("r#*\"[^\"]*\"#*")] RawString,

    // Unknown/Error
    Error,
}

/// Token with metadata
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
    pub text: String,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span, text: String) -> Self {
        Self { kind, span, text }
    }

    pub fn is_eof(&self) -> bool {
        matches!(self.kind, TokenKind::Error) && self.text.is_empty()
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}(\"{}\") @ {}", self.kind, self.text, self.span)
    }
}

/// Source code location span
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}

impl Span {
    pub fn new(start: usize, end: usize, line: usize, column: usize) -> Self {
        Self { start, end, line, column }
    }

    pub fn merge(self, other: Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
            line: self.line,
            column: self.column,
        }
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

impl From<LogosSpan> for Span {
    fn from(span: LogosSpan) -> Self {
        Span {
            start: span.start,
            end: span.end,
            line: 0,
            column: 0,
        }
    }
}

/// Lexer for Bhabex source code
pub struct Lexer<'a> {
    source: &'a str,
    logos_lexer: LogosLexer<'a, TokenKind>,
    line_starts: Vec<usize>,
    diagnostics: DiagnosticBag,
}

impl<'a> Lexer<'a> {
    /// Create a new lexer for the given source code
    pub fn new(source: &'a str) -> Self {
        let line_starts = Self::compute_line_starts(source);
        let logos_lexer = TokenKind::lexer(source);
        
        Self {
            source,
            logos_lexer,
            line_starts,
            diagnostics: DiagnosticBag::new(),
        }
    }

    /// Compute line start positions for span calculation
    fn compute_line_starts(source: &str) -> Vec<usize> {
        let mut starts = vec![0];
        for (i, ch) in source.char_indices() {
            if ch == '\n' {
                starts.push(i + 1);
            }
        }
        starts
    }

    /// Convert logos span to our span with line/column
    fn span_from_logos(&self, logos_span: LogosSpan) -> Span {
        let start = logos_span.start;
        let end = logos_span.end;
        
        // Binary search for line
        let line = self.line_starts.binary_search(&start).unwrap_or_else(|i| i - 1);
        let line_start = self.line_starts[line];
        let column = start - line_start;
        
        Span::new(start, end, line + 1, column + 1)
    }

    /// Tokenize the entire source
    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        
        while let Some(token_result) = self.logos_lexer.next() {
            let logos_span = self.logos_lexer.span();
            let span = self.span_from_logos(logos_span);
            
            match token_result {
                Ok(kind) => {
                    let text = &self.source[logos_span.start..logos_span.end];
                    tokens.push(Token::new(kind, span, text.to_string()));
                }
                Err(_) => {
                    let text = &self.source[logos_span.start..logos_span.end];
                    self.diagnostics.add(Diagnostic::new(
                        DiagnosticSeverity::Error,
                        format!("Invalid token: \"{}\"", text),
                        span,
                    ));
                    tokens.push(Token::new(TokenKind::Error, span, text.to_string()));
                }
            }
        }
        
        // Add EOF token
        let eof_span = Span::new(
            self.source.len(),
            self.source.len(),
            self.line_starts.len(),
            1,
        );
        tokens.push(Token::new(TokenKind::Error, eof_span, String::new()));
        
        tokens
    }

    /// Get collected diagnostics
    pub fn diagnostics(&self) -> &DiagnosticBag {
        &self.diagnostics
    }

    /// Get source text
    pub fn source(&self) -> &str {
        self.source
    }
}

/// Token with metadata
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
    pub text: String,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span, text: String) -> Self {
        Self { kind, span, text }
    }

    pub fn is_eof(&self) -> bool {
        matches!(self.kind, TokenKind::Error) && self.text.is_empty()
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}(\"{}\") @ {}", self.kind, self.text, self.span)
    }
}

/// Source code location span
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}

impl Span {
    pub fn new(start: usize, end: usize, line: usize, column: usize) -> Self {
        Self { start, end, line, column }
    }

    pub fn merge(self, other: Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
            line: self.line,
            column: self.column,
        }
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

impl From<LogosSpan> for Span {
    fn from(span: LogosSpan) -> Self {
        Span {
            start: span.start,
            end: span.end,
            line: 0,
            column: 0,
        }
    }
}

/// Convenience function to tokenize source
pub fn tokenize(source: &str) -> (Vec<Token>, DiagnosticBag) {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize();
    let diagnostics = lexer.diagnostics().clone();
    (tokens, diagnostics)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_tokens() {
        let source = "fn main() { let x = 42; }";
        let (tokens, _) = tokenize(source);
        
        assert_eq!(tokens[0].kind, TokenKind::Fn);
        assert_eq!(tokens[1].kind, TokenKind::Ident);
        assert_eq!(tokens[2].kind, TokenKind::LParen);
        assert_eq!(tokens[3].kind, TokenKind::RParen);
        assert_eq!(tokens[4].kind, TokenKind::LBrace);
        assert_eq!(tokens[5].kind, TokenKind::Let);
        assert_eq!(tokens[6].kind, TokenKind::Ident);
        assert_eq!(tokens[7].kind, TokenKind::Assign);
        assert_eq!(tokens[8].kind, TokenKind::Int);
        assert_eq!(tokens[9].kind, TokenKind::Semi);
        assert_eq!(tokens[10].kind, TokenKind::RBrace);
    }

    #[test]
    fn test_keywords() {
        let keywords = vec![
            ("fn", TokenKind::Fn),
            ("let", TokenKind::Let),
            ("mut", TokenKind::Mut),
            ("const", TokenKind::Const),
            ("struct", TokenKind::Struct),
            ("enum", TokenKind::Enum),
            ("trait", TokenKind::Trait),
            ("impl", TokenKind::Impl),
            ("if", TokenKind::If),
            ("else", TokenKind::Else),
            ("match", TokenKind::Match),
            ("for", TokenKind::For),
            ("while", TokenKind::While),
            ("loop", TokenKind::Loop),
            ("break", TokenKind::Break),
            ("continue", TokenKind::Continue),
            ("return", TokenKind::Return),
            ("async", TokenKind::Async),
            ("await", TokenKind::Await),
            ("import", TokenKind::Import),
            ("export", TokenKind::Export),
            ("pub", TokenKind::Pub),
            ("use", TokenKind::Use),
        ];

        for (kw, expected) in keywords {
            let (tokens, _) = tokenize(kw);
            assert_eq!(tokens[0].kind, expected, "Failed for keyword: {}", kw);
        }
    }

    #[test]
    fn test_operators() {
        let operators = vec![
            ("+", TokenKind::Plus),
            ("-", TokenKind::Minus),
            ("*", TokenKind::Star),
            ("/", TokenKind::Slash),
            ("%", TokenKind::Percent),
            ("**", TokenKind::Pow),
            ("+=", TokenKind::PlusEq),
            ("==", TokenKind::EqEq),
            ("!=", TokenKind::Ne),
            ("<=", TokenKind::Le),
            (">=", TokenKind::Ge),
            ("&&", TokenKind::And),
            ("||", TokenKind::Or),
            ("->", TokenKind::Arrow),
            ("=>", TokenKind::FatArrow),
            ("..", TokenKind::Range),
            ("..=", TokenKind::RangeInclusive),
        ];

        for (op, expected) in operators {
            let (tokens, _) = tokenize(op);
            assert_eq!(tokens[0].kind, expected, "Failed for operator: {}", op);
        }
    }

    #[test]
    fn test_literals() {
        let (tokens, _) = tokenize("42");
        assert_eq!(tokens[0].kind, TokenKind::Int);
        assert_eq!(tokens[0].text, "42");

        let (tokens, _) = tokenize("3.14");
        assert_eq!(tokens[0].kind, TokenKind::Float);

        let (tokens, _) = tokenize(r#""hello""#);
        assert_eq!(tokens[0].kind, TokenKind::String);

        let (tokens, _) = tokenize("'a'");
        assert_eq!(tokens[0].kind, TokenKind::Char);

        let (tokens, _) = tokenize("true");
        assert_eq!(tokens[0].kind, TokenKind::True);

        let (tokens, _) = tokenize("false");
        assert_eq!(tokens[0].kind, TokenKind::False);
    }

    #[test]
    fn test_identifiers() {
        let (tokens, _) = tokenize("my_var");
        assert_eq!(tokens[0].kind, TokenKind::Ident);
        assert_eq!(tokens[0].text, "my_var");

        let (tokens, _) = tokenize("_private");
        assert_eq!(tokens[0].kind, TokenKind::Ident);

        let (tokens, _) = tokenize("CamelCase");
        assert_eq!(tokens[0].kind, TokenKind::Ident);
    }

    #[test]
    fn test_complex_expression() {
        let source = "fn add(a: i32, b: i32) -> i32 { a + b }";
        let (tokens, _) = tokenize(source);
        
        let kinds: Vec<_> = tokens.iter().map(|t| t.kind.clone()).collect();
        assert!(kinds.contains(&TokenKind::Fn));
        assert!(kinds.contains(&TokenKind::Ident));
        assert!(kinds.contains(&TokenKind::Colon));
        assert!(kinds.contains(&TokenKind::I32));
        assert!(kinds.contains(&TokenKind::Arrow));
        assert!(kinds.contains(&TokenKind::Plus));
    }

    #[test]
    fn test_comments_skipped() {
        let source = "fn main() { // comment\nlet x = 1; /* block comment */ }";
        let (tokens, _) = tokenize(source);
        
        for token in &tokens {
            assert_ne!(token.text, "// comment");
            assert_ne!(token.text, "/* block comment */");
        }
    }

    #[test]
    fn test_raw_strings() {
        let source = "r#\"hello\"#";
        let (tokens, _) = tokenize(source);
        assert_eq!(tokens[0].kind, TokenKind::RawString);
    }

    #[test]
    fn test_number_formats() {
        let (tokens, _) = tokenize("0xFF");
        assert_eq!(tokens[0].kind, TokenKind::HexInt);

        let (tokens, _) = tokenize("0o77");
        assert_eq!(tokens[0].kind, TokenKind::OctInt);

        let (tokens, _) = tokenize("0b1010");
        assert_eq!(tokens[0].kind, TokenKind::BinInt);
    }
}