//! Database and API parsing module

use crate::ast::*;
use crate::lexer::TokenType;
use super::base::{BaseParser, ParserContext};

pub struct DatabaseParser;

impl DatabaseParser {
    /// Parse database and API statements
    pub fn parse_data_statement(ctx: &mut ParserContext) -> ParseResult<Node> {
        match &ctx.peek().token_type {
            TokenType::Call | TokenType::Fetch | TokenType::Update | TokenType::Delete => {
                Self::parse_api_call(ctx)
            }
            TokenType::Db => Self::parse_database_statement(ctx),
            TokenType::Serve => Self::parse_serve_statement(ctx),
            _ => Err(ParseError {
                message: "Not a database or API statement".to_string(),
                line: ctx.peek().line,
                column: ctx.peek().column,
            })
        }
    }
    
    /// Parse API call statements: call GET "/api/users"
    fn parse_api_call(ctx: &mut ParserContext) -> ParseResult<Node> {
        let line = ctx.peek().line;
        let verb = match &ctx.advance().token_type {
            TokenType::Call => "call",
            TokenType::Fetch => "fetch", 
            TokenType::Update => "update",
            TokenType::Delete => "delete",
            _ => "call",
        }.to_string();
        
        let method = match &ctx.advance().token_type {
            TokenType::Identifier(method) => method.clone(),
            _ => return Err(ParseError {
                message: "Expected HTTP method".to_string(),
                line: ctx.previous().line,
                column: ctx.previous().column,
            }),
        };
        
        let endpoint = match &ctx.advance().token_type {
            TokenType::StringLiteral(endpoint) => endpoint.clone(),
            TokenType::Identifier(endpoint) => endpoint.clone(),
            _ => return Err(ParseError {
                message: "Expected endpoint".to_string(),
                line: ctx.previous().line,
                column: ctx.previous().column,
            }),
        };
        
        // TODO: Parse headers, payload, response variable
        Ok(Node::ApiCallStatement(ApiCallStatement {
            verb,
            endpoint,
            method,
            payload: None,
            headers: Vec::new(),
            response_variable: None,
            line_number: Some(line),
        }))
    }
    
    /// Parse database statements: db create users
    fn parse_database_statement(ctx: &mut ParserContext) -> ParseResult<Node> {
        let line = ctx.peek().line;
        ctx.consume(&TokenType::Db, "Expected 'db'")?;
        
        let operation = match &ctx.advance().token_type {
            TokenType::Identifier(op) => op.clone(),
            _ => return Err(ParseError {
                message: "Expected database operation".to_string(),
                line: ctx.previous().line,
                column: ctx.previous().column,
            }),
        };
        
        let entity_name = match &ctx.advance().token_type {
            TokenType::Identifier(name) => name.clone(),
            _ => return Err(ParseError {
                message: "Expected entity name".to_string(),
                line: ctx.previous().line,
                column: ctx.previous().column,
            }),
        };
        
        Ok(Node::DatabaseStatement(DatabaseStatement {
            operation,
            entity_name,
            conditions: Vec::new(),
            fields: Vec::new(),
            return_var: None,
            line_number: Some(line),
        }))
    }
    
    /// Parse serve statements: serve GET "/api/endpoint"
    fn parse_serve_statement(ctx: &mut ParserContext) -> ParseResult<Node> {
        let line = ctx.peek().line;
        ctx.consume(&TokenType::Serve, "Expected 'serve'")?;
        
        let method = match &ctx.advance().token_type {
            TokenType::Identifier(method) => method.clone(),
            _ => return Err(ParseError {
                message: "Expected HTTP method".to_string(),
                line: ctx.previous().line,
                column: ctx.previous().column,
            }),
        };
        
        let endpoint = match &ctx.advance().token_type {
            TokenType::StringLiteral(endpoint) => endpoint.clone(),
            TokenType::Identifier(endpoint) => endpoint.clone(),
            _ => return Err(ParseError {
                message: "Expected endpoint".to_string(),
                line: ctx.previous().line,
                column: ctx.previous().column,
            }),
        };
        
        Ok(Node::ServeStatement(ServeStatement {
            method,
            endpoint,
            body: Vec::new(),
            params: Vec::new(),
            accept_type: None,
            response_action: None,
            line_number: Some(line),
        }))
    }
}