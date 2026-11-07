//! Bhabex Compiler
//!
//! Copyright BHABEX 2026 | Securely designed & developed by bxLabs | Protected by bxSecurity | Powered by BHABEX®

pub mod lexer;
pub mod parser;
pub mod ast;
pub mod bytecode;
pub mod vm;
pub mod compiler;
pub mod diagnostics;
pub mod symbols;

pub use lexer::{Lexer, Token, TokenKind, Span, tokenize};
pub use parser::{Parser, ParseError};
pub use ast::*;
pub use bytecode::{Bytecode, Instruction, Constant, Function, Class};
pub use vm::{VM, VMError, Value};
pub use compiler::Compiler;
pub use diagnostics::{Diagnostic, DiagnosticSeverity, DiagnosticBag};
pub use symbols::{SymbolTable, Symbol, SymbolKind};