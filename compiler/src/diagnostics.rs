//! Diagnostics and error reporting for Bhabex
//! 
//! Copyright BHABEX 2026 | Securely designed & developed by bxLabs | Protected by bxSecurity | Powered by BHABEX®

use std::fmt;
use crate::lexer::Span;

/// Severity of a diagnostic
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DiagnosticSeverity {
    /// An error that prevents compilation
    Error,
    /// A warning that doesn't prevent compilation
    Warning,
    /// An informational message
    Info,
    /// A hint/suggestion
    Hint,
}

impl fmt::Display for DiagnosticSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DiagnosticSeverity::Error => write!(f, "error"),
            DiagnosticSeverity::Warning => write!(f, "warning"),
            DiagnosticSeverity::Info => write!(f, "info"),
            DiagnosticSeverity::Hint => write!(f, "hint"),
        }
    }
}

/// A single diagnostic message
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub span: Span,
    pub code: Option<String>,
    pub help: Option<String>,
    pub notes: Vec<(Span, String)>,
}

impl Diagnostic {
    pub fn new(severity: DiagnosticSeverity, message: impl Into<String>, span: Span) -> Self {
        Self {
            severity,
            message: message.into(),
            span,
            code: None,
            help: None,
            notes: Vec::new(),
        }
    }

    pub fn error(message: impl Into<String>, span: Span) -> Self {
        Self::new(DiagnosticSeverity::Error, message, span)
    }

    pub fn warning(message: impl Into<String>, span: Span) -> Self {
        Self::new(DiagnosticSeverity::Warning, message, span)
    }

    pub fn info(message: impl Into<String>, span: Span) -> Self {
        Self::new(DiagnosticSeverity::Info, message, span)
    }

    pub fn hint(message: impl Into<String>, span: Span) -> Self {
        Self::new(DiagnosticSeverity::Hint, message, span)
    }

    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }

    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    pub fn with_note(mut self, span: Span, message: impl Into<String>) -> Self {
        self.notes.push((span, message.into()));
        self
    }
}

/// Collection of diagnostics
#[derive(Debug, Clone, Default)]
pub struct DiagnosticBag {
    diagnostics: Vec<Diagnostic>,
}

impl DiagnosticBag {
    pub fn new() -> Self {
        Self { diagnostics: Vec::new() }
    }

    pub fn add(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    pub fn extend(&mut self, other: DiagnosticBag) {
        self.diagnostics.extend(other.diagnostics);
    }

    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(|d| d.severity == DiagnosticSeverity::Error)
    }

    pub fn errors(&self) -> Vec<&Diagnostic> {
        self.diagnostics.iter().filter(|d| d.severity == DiagnosticSeverity::Error).collect()
    }

    pub fn warnings(&self) -> Vec<&Diagnostic> {
        self.diagnostics.iter().filter(|d| d.severity == DiagnosticSeverity::Warning).collect()
    }

    pub fn all(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    pub fn len(&self) -> usize {
        self.diagnostics.len()
    }

    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }

    pub fn clear(&mut self) {
        self.diagnostics.clear();
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let severity_str = match self.severity {
            DiagnosticSeverity::Error => "error",
            DiagnosticSeverity::Warning => "warning",
            DiagnosticSeverity::Info => "info",
            DiagnosticSeverity::Hint => "hint",
        };

        if let Some(code) = &self.code {
            write!(f, "[{}] {}: {}", severity_str, code, self.message)?;
        } else {
            write!(f, "[{}] {}", severity_str, self.message)?;
        }

        write!(f, " @ {}", self.span)?;

        if let Some(help) = &self.help {
            write!(f, "\n  help: {}", help)?;
        }

        for (span, note) in &self.notes {
            write!(f, "\n  @ {}: {}", span, note)?;
        }

        Ok(())
    }
}