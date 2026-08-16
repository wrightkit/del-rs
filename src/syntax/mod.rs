//! Syntax layer: lexer, token stream, AST, recoverable parser.

pub mod ast;
pub mod lexer;
pub mod parser;
pub mod token;

use crate::span::{FileId, Span};

/// Result of lexing + parsing a single source file.
pub struct ParseOutput {
    pub tokens: Vec<token::Token>,
    pub ast: ast::AstFile,
    pub diagnostics: Vec<crate::diagnostics::Diagnostic>,
}

/// Lex and parse `text` (belonging to `file`).
pub fn parse_source(file: FileId, text: &str) -> ParseOutput {
    let (tokens, mut diagnostics) = lexer::lex(file, text);
    let (ast, parse_diags) = parser::parse(&tokens, file, text);
    diagnostics.extend(parse_diags);
    ParseOutput {
        tokens,
        ast,
        diagnostics,
    }
}

/// Total span of a file.
pub fn file_span(file: FileId, text: &str) -> Span {
    Span::new(file, 0, text.len() as u32)
}
