//! Diagnostic reporting for the Droe compiler

use serde::{Deserialize, Serialize};
use crate::ast::ParseError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub severity: Severity,
    pub message: String,
    pub line: usize,
    pub character: usize,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Severity {
    Error,
    Warning,
    Information,
    Hint,
}

impl Diagnostic {
    pub fn error(message: String, line: usize, character: usize) -> Self {
        Self {
            severity: Severity::Error,
            message,
            line,
            character,
            source: "droe-compiler".to_string(),
        }
    }
    
    pub fn warning(message: String, line: usize, character: usize) -> Self {
        Self {
            severity: Severity::Warning,
            message,
            line,
            character,
            source: "droe-compiler".to_string(),
        }
    }
    
    pub fn from_parse_error(error: ParseError) -> Self {
        Self::error(error.message, error.line, error.column)
    }
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Error => write!(f, "error"),
            Severity::Warning => write!(f, "warning"),
            Severity::Information => write!(f, "information"),
            Severity::Hint => write!(f, "hint"),
        }
    }
}