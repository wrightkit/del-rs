//! Token kinds and token stream.

use crate::span::Span;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TokenKind {
    // Trivia (retained; skipped by the parser, discoverable for tooling).
    Whitespace,
    LineComment,
    BlockComment,
    DocComment,

    // Literals.
    Int,
    Real,
    Str,
    Bool,

    // Identifiers and keywords.
    Ident,
    KwRule,
    KwDefine,
    KwGlobalVar,
    KwPlayerVar,
    KwIf,
    KwElse,
    KwFor,
    KwForeach,
    KwWhile,
    KwSwitch,
    KwCase,
    KwDefault,
    KwBreak,
    KwContinue,
    KwReturn,
    KwClass,
    KwStruct,
    KwEnum,
    KwConstructor,
    KwNew,
    KwDelete,
    KwIn,
    KwRef,
    KwRecursive,
    KwAsync,
    KwConst,
    KwImport,
    KwAs,
    KwIs,
    KwPublic,
    KwPrivate,
    KwProtected,
    KwStatic,
    KwVirtual,
    KwOverride,
    KwSingle,
    KwThis,
    KwRoot,
    KwTrue,
    KwFalse,
    KwNull,
    KwType,
    KwDisabled,
    KwPersist,
    KwVoid,
    KwJson,

    // Punctuation and operators.
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Semicolon,
    Colon,
    Dot,
    DotDot,
    Arrow,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Caret,
    PlusPlus,
    MinusMinus,
    PlusEq,
    MinusEq,
    StarEq,
    SlashEq,
    PercentEq,
    CaretEq,
    Eq,
    EqEq,
    Bang,
    BangEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
    AmpAmp,
    PipePipe,
    Pipe,
    Tilde,
    Question,
    At,

    // Recovery.
    Error,
    Eof,
}

impl TokenKind {
    pub fn is_trivia(&self) -> bool {
        matches!(
            self,
            TokenKind::Whitespace
                | TokenKind::LineComment
                | TokenKind::BlockComment
                | TokenKind::DocComment
        )
    }

    /// Keyword names for diagnostics ("expected keyword ...").
    pub fn describe(&self) -> &'static str {
        match self {
            TokenKind::Int => "integer",
            TokenKind::Real => "number",
            TokenKind::Str => "string",
            TokenKind::Ident => "identifier",
            TokenKind::LParen => "'('",
            TokenKind::RParen => "')'",
            TokenKind::LBrace => "'{'",
            TokenKind::RBrace => "'}'",
            TokenKind::LBracket => "'['",
            TokenKind::RBracket => "']'",
            TokenKind::Comma => "','",
            TokenKind::Semicolon => "';'",
            TokenKind::Colon => "':'",
            TokenKind::Dot => "'.'",
            TokenKind::DotDot => "'..'",
            TokenKind::Arrow => "'=>'",
            TokenKind::Plus => "'+'",
            TokenKind::Minus => "'-'",
            TokenKind::Star => "'*'",
            TokenKind::Slash => "'/'",
            TokenKind::Percent => "'%'",
            TokenKind::Caret => "'^'",
            TokenKind::Eq => "'='",
            TokenKind::EqEq => "'=='",
            TokenKind::Bang => "'!'",
            TokenKind::BangEq => "'!='",
            TokenKind::Lt => "'<'",
            TokenKind::Gt => "'>'",
            TokenKind::LtEq => "'<='",
            TokenKind::GtEq => "'>='",
            TokenKind::AmpAmp => "'&&'",
            TokenKind::PipePipe => "'||'",
            TokenKind::Pipe => "'|'",
            TokenKind::Tilde => "'~'",
            TokenKind::Question => "'?'",
            TokenKind::Eof => "end of file",
            _ => "token",
        }
    }
}

/// Lexical form of a string token.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StrForm {
    /// `"..."` or `'...'`
    Plain,
    /// `@"..."` or `@'...'`
    Localized,
    /// `$"..."` or `$'...'`
    Interpolated,
}

/// Interpolated-string hole (opening `{` / closing `}` markers and the
/// expression token slice inside).
#[derive(Clone, Debug)]
pub struct InterpHole {
    pub open: Span,
    pub close: Span,
    pub tokens: Vec<Token>,
}

#[derive(Clone, Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
    /// Present only for `Str` tokens.
    pub str_form: Option<StrForm>,
    /// Holes for interpolated strings (empty for other kinds).
    pub holes: Vec<InterpHole>,
    /// `true`/`false` value for `Bool` tokens.
    pub bool_value: Option<bool>,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Token {
        Token {
            kind,
            span,
            str_form: None,
            holes: Vec::new(),
            bool_value: None,
        }
    }
}
