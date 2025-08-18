//! Structural parsing module - modules, data, forms, screens, fragments

use crate::ast::*;
use crate::lexer::TokenType;
use super::base::{BaseParser, ParserContext};
use super::statements::StatementParser;

pub struct StructureParser;

impl StructureParser {
    /// Parse structural definitions
    pub fn parse_structure(ctx: &mut ParserContext) -> ParseResult<Node> {
        match &ctx.peek().token_type {
            TokenType::Module => Self::parse_module(ctx),
            TokenType::Data => Self::parse_data_definition(ctx),
            TokenType::Layout => Self::parse_layout_definition(ctx),
            TokenType::Form => Self::parse_form_definition(ctx),
            TokenType::Screen => Self::parse_screen_definition(ctx),
            TokenType::Fragment => Self::parse_fragment_definition(ctx),
            _ => Err(ParseError {
                message: "Not a structural definition".to_string(),
                line: ctx.peek().line,
                column: ctx.peek().column,
            })
        }
    }
    
    /// Parse module definition
    fn parse_module(ctx: &mut ParserContext) -> ParseResult<Node> {
        let line = ctx.peek().line;
        ctx.consume(&TokenType::Module, "Expected 'module'")?;
        
        let name = match &ctx.advance().token_type {
            TokenType::Identifier(name) => name.clone(),
            _ => return Err(ParseError {
                message: "Expected module name".to_string(),
                line: ctx.previous().line,
                column: ctx.previous().column,
            }),
        };
        
        let mut body = Vec::new();
        
        while !ctx.check(&TokenType::EndModule) && !ctx.is_at_end() {
            ctx.skip_newlines_and_comments();
            if ctx.check(&TokenType::EndModule) || ctx.is_at_end() {
                break;
            }
            body.push(StatementParser::parse_statement(ctx)?);
        }
        
        ctx.consume(&TokenType::EndModule, "Expected 'end module'")?;
        
        Ok(Node::ModuleDefinition(ModuleDefinition {
            name,
            body,
            line_number: Some(line),
        }))
    }
    
    /// Parse data definition
    fn parse_data_definition(ctx: &mut ParserContext) -> ParseResult<Node> {
        let line = ctx.peek().line;
        ctx.consume(&TokenType::Data, "Expected 'data'")?;
        
        let name = match &ctx.advance().token_type {
            TokenType::Identifier(name) => name.clone(),
            _ => return Err(ParseError {
                message: "Expected data type name".to_string(),
                line: ctx.previous().line,
                column: ctx.previous().column,
            }),
        };
        
        let mut fields = Vec::new();
        
        while !ctx.check(&TokenType::EndData) && !ctx.is_at_end() {
            ctx.skip_newlines_and_comments();
            if ctx.check(&TokenType::EndData) || ctx.is_at_end() {
                break;
            }
            
            // Parse field: "field_name is field_type"
            let field_name = match &ctx.advance().token_type {
                TokenType::Identifier(name) => name.clone(),
                _ => return Err(ParseError {
                    message: "Expected field name".to_string(),
                    line: ctx.previous().line,
                    column: ctx.previous().column,
                }),
            };
            
            ctx.consume(&TokenType::Is, "Expected 'is'")?;
            
            let field_type = match &ctx.advance().token_type {
                TokenType::Identifier(type_name) => type_name.clone(),
                _ => return Err(ParseError {
                    message: "Expected field type".to_string(),
                    line: ctx.previous().line,
                    column: ctx.previous().column,
                }),
            };
            
            fields.push(DataField {
                name: field_name,
                field_type,
                annotations: Vec::new(),
                line_number: Some(ctx.previous().line),
            });
        }
        
        ctx.consume(&TokenType::EndData, "Expected 'end data'")?;
        
        Ok(Node::DataDefinition(DataDefinition {
            name,
            fields,
            storage_type: None,
            line_number: Some(line),
        }))
    }
    
    /// Parse layout definition
    fn parse_layout_definition(ctx: &mut ParserContext) -> ParseResult<Node> {
        let line = ctx.peek().line;
        ctx.consume(&TokenType::Layout, "Expected 'layout'")?;
        
        let name = match &ctx.advance().token_type {
            TokenType::Identifier(name) => name.clone(),
            _ => return Err(ParseError {
                message: "Expected layout name".to_string(),
                line: ctx.previous().line,
                column: ctx.previous().column,
            }),
        };
        
        let slots = Vec::new();
        
        while !ctx.check(&TokenType::EndLayout) && !ctx.is_at_end() {
            ctx.skip_newlines_and_comments();
            if ctx.check(&TokenType::EndLayout) || ctx.is_at_end() {
                break;
            }
            // TODO: Parse layout content
            ctx.advance();
        }
        
        ctx.consume(&TokenType::EndLayout, "Expected 'end layout'")?;
        
        Ok(Node::FragmentDefinition(FragmentDefinition {
            name,
            slots,
            attributes: Vec::new(),
            classes: Vec::new(),
            styles: None,
            line_number: Some(line),
        }))
    }
    
    /// Parse form definition
    fn parse_form_definition(ctx: &mut ParserContext) -> ParseResult<Node> {
        let line = ctx.peek().line;
        ctx.consume(&TokenType::Form, "Expected 'form'")?;
        
        let name = match &ctx.advance().token_type {
            TokenType::Identifier(name) => name.clone(),
            _ => return Err(ParseError {
                message: "Expected form name".to_string(),
                line: ctx.previous().line,
                column: ctx.previous().column,
            }),
        };
        
        let mut children = Vec::new();
        
        while !ctx.check(&TokenType::EndForm) && !ctx.is_at_end() {
            ctx.skip_newlines_and_comments();
            if ctx.check(&TokenType::EndForm) || ctx.is_at_end() {
                break;
            }
            children.push(StatementParser::parse_statement(ctx)?);
        }
        
        ctx.consume(&TokenType::EndForm, "Expected 'end form'")?;
        
        Ok(Node::FormDefinition(FormDefinition {
            name,
            children,
            attributes: Vec::new(),
            classes: Vec::new(),
            styles: None,
            line_number: Some(line),
        }))
    }
    
    /// Parse screen definition
    fn parse_screen_definition(ctx: &mut ParserContext) -> ParseResult<Node> {
        let line = ctx.peek().line;
        ctx.consume(&TokenType::Screen, "Expected 'screen'")?;
        
        let name = match &ctx.advance().token_type {
            TokenType::Identifier(name) => name.clone(),
            _ => return Err(ParseError {
                message: "Expected screen name".to_string(),
                line: ctx.previous().line,
                column: ctx.previous().column,
            }),
        };
        
        let fragments = Vec::new();
        
        while !ctx.check(&TokenType::EndScreen) && !ctx.is_at_end() {
            ctx.skip_newlines_and_comments();
            if ctx.check(&TokenType::EndScreen) || ctx.is_at_end() {
                break;
            }
            // TODO: Parse screen content
            ctx.advance();
        }
        
        ctx.consume(&TokenType::EndScreen, "Expected 'end screen'")?;
        
        Ok(Node::ScreenDefinition(ScreenDefinition {
            name,
            fragments,
            attributes: Vec::new(),
            classes: Vec::new(),
            styles: None,
            line_number: Some(line),
        }))
    }
    
    /// Parse fragment definition
    fn parse_fragment_definition(ctx: &mut ParserContext) -> ParseResult<Node> {
        let line = ctx.peek().line;
        ctx.consume(&TokenType::Fragment, "Expected 'fragment'")?;
        
        let name = match &ctx.advance().token_type {
            TokenType::Identifier(name) => name.clone(),
            _ => return Err(ParseError {
                message: "Expected fragment name".to_string(),
                line: ctx.previous().line,
                column: ctx.previous().column,
            }),
        };
        
        let slots = Vec::new();
        
        while !ctx.check(&TokenType::EndFragment) && !ctx.is_at_end() {
            ctx.skip_newlines_and_comments();
            if ctx.check(&TokenType::EndFragment) || ctx.is_at_end() {
                break;
            }
            // TODO: Parse fragment content
            ctx.advance();
        }
        
        ctx.consume(&TokenType::EndFragment, "Expected 'end fragment'")?;
        
        Ok(Node::FragmentDefinition(FragmentDefinition {
            name,
            slots,
            attributes: Vec::new(),
            classes: Vec::new(),
            styles: None,
            line_number: Some(line),
        }))
    }
}