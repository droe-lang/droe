//! Metadata annotation parsing module

use crate::ast::*;
use crate::lexer::TokenType;
use super::base::{BaseParser, ParserContext};

pub struct MetadataParser;

impl MetadataParser {
    /// Parse metadata annotation: @target mobile
    pub fn parse_metadata(ctx: &mut ParserContext) -> ParseResult<MetadataAnnotation> {
        let line = ctx.peek().line;
        
        // Parse @key format (lexer treats @target as single identifier)
        let full_annotation = match &ctx.advance().token_type {
            TokenType::Identifier(s) if s.starts_with('@') => s.clone(),
            _ => return Err(ParseError {
                message: "Expected metadata annotation starting with '@'".to_string(),
                line: ctx.previous().line,
                column: ctx.previous().column,
            }),
        };
        
        // Extract key by removing '@' prefix
        let key = full_annotation[1..].to_string();
        
        // Parse the value (optional)
        let value = if !ctx.is_at_end() && !ctx.check(&TokenType::Newline) {
            match &ctx.peek().token_type {
                TokenType::StringLiteral(_) => {
                    if let TokenType::StringLiteral(val) = &ctx.advance().token_type {
                        MetadataValue::String(val.clone())
                    } else {
                        MetadataValue::String(String::new())
                    }
                }
                TokenType::Identifier(_) => {
                    if let TokenType::Identifier(val) = &ctx.advance().token_type {
                        MetadataValue::String(val.clone())
                    } else {
                        MetadataValue::String(String::new())
                    }
                }
                _ => MetadataValue::String(String::new())
            }
        } else {
            MetadataValue::String(String::new())
        };
        
        Ok(MetadataAnnotation {
            key,
            value,
            line_number: Some(line),
        })
    }
    
    /// Check if current token starts a metadata annotation
    pub fn is_metadata_start(ctx: &ParserContext) -> bool {
        // Check if line starts with @ followed by identifier
        let mut check_pos = ctx.current();
        if check_pos < ctx.tokens().len() {
            if let TokenType::Identifier(s) = &ctx.tokens()[check_pos].token_type {
                if s.starts_with('@') {
                    return true;
                }
            }
        }
        false
    }
}