//! Modular parser for Droe DSL
//! 
//! This module provides a clean, modular parser architecture inspired by the Python implementation.
//! Each sub-module handles a specific aspect of parsing:
//! 
//! - `base`: Common utilities and base parser trait
//! - `statements`: Basic statements (display, assignment, conditionals, loops)
//! - `structures`: Structural definitions (modules, data, forms, screens)
//! - `ui_components`: UI component parsing
//! - `database`: Database and API statement parsing
//! - `metadata`: Metadata annotation parsing

pub mod base;
pub mod statements;
pub mod structures;
pub mod ui_components;
pub mod database;
pub mod metadata;

use crate::ast::*;
use crate::lexer::Lexer;
use self::base::{BaseParser, ParserContext};
use self::statements::StatementParser;
use self::metadata::MetadataParser;

/// Main parser that orchestrates all parsing modules
pub struct Parser {
    // Keep for backward compatibility
}

impl Parser {
    pub fn new() -> Self {
        Self {}
    }
    
    /// Parse source code into a Program AST
    pub fn parse(&self, source: &str) -> ParseResult<Program> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();
        
        let mut ctx = ParserContext::new(tokens);
        self.parse_program(&mut ctx)
    }
    
    /// Parse the entire program
    fn parse_program(&self, ctx: &mut ParserContext) -> ParseResult<Program> {
        let mut statements = Vec::new();
        let mut metadata = Vec::new();
        
        while !ctx.is_at_end() {
            ctx.skip_newlines_and_comments();
            
            if ctx.is_at_end() {
                break;
            }
            
            // Check for metadata annotations
            if MetadataParser::is_metadata_start(ctx) {
                if let Ok(meta) = MetadataParser::parse_metadata(ctx) {
                    metadata.push(meta.clone());
                    statements.push(Node::MetadataAnnotation(meta));
                    continue;
                }
            }
            
            // Parse regular statements
            match StatementParser::parse_statement(ctx) {
                Ok(stmt) => statements.push(stmt),
                Err(e) => return Err(e),
            }
        }
        
        Ok(Program {
            statements,
            metadata,
            included_modules: None,
            line_number: Some(1),
        })
    }
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}