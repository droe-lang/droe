//! Statement parsing module - basic statements like display, assignment, conditionals, loops

use crate::ast::*;
use crate::lexer::TokenType;
use crate::expressions::ExpressionParser;
use super::base::{BaseParser, ParserContext};
use super::ui_components::UIComponentParser;
use super::database::DatabaseParser;
use super::structures::StructureParser;

pub struct StatementParser;

impl StatementParser {
    /// Parse any statement
    pub fn parse_statement(ctx: &mut ParserContext) -> ParseResult<Node> {
        // Skip newlines and comments
        ctx.skip_newlines_and_comments();
        
        if ctx.is_at_end() {
            return Err(ParseError {
                message: "Unexpected end of input".to_string(),
                line: ctx.peek().line,
                column: ctx.peek().column,
            });
        }
        
        match &ctx.peek().token_type {
            // Structural definitions
            TokenType::Module | TokenType::Data | TokenType::Layout | 
            TokenType::Form | TokenType::Screen | TokenType::Fragment => {
                StructureParser::parse_structure(ctx)
            }
            
            // Database and API statements
            TokenType::Call | TokenType::Fetch | TokenType::Update | 
            TokenType::Delete | TokenType::Db | TokenType::Serve => {
                DatabaseParser::parse_data_statement(ctx)
            }
            
            // UI Components
            TokenType::Title | TokenType::Text | TokenType::Input | TokenType::Textarea |
            TokenType::Dropdown | TokenType::Toggle | TokenType::Checkbox | TokenType::Radio |
            TokenType::Button | TokenType::Image | TokenType::Video | TokenType::Audio |
            TokenType::Slot => {
                UIComponentParser::parse_component(ctx)
            }
            
            // Basic statements
            TokenType::Action => Self::parse_action_definition(ctx),
            TokenType::Task => Self::parse_task_definition(ctx),
            TokenType::Display => Self::parse_display_statement(ctx),
            TokenType::Set => Self::parse_set_statement(ctx),
            TokenType::When => Self::parse_when_statement(ctx),
            TokenType::While => Self::parse_while_loop(ctx),
            TokenType::For => Self::parse_for_loop(ctx),
            TokenType::Give => Self::parse_return_statement(ctx),
            TokenType::Include => Self::parse_include_statement(ctx),
            
            TokenType::Identifier(_) => {
                // Could be assignment or action invocation
                if ctx.peek_ahead_for_is() {
                    Self::parse_assignment(ctx)
                } else if ctx.peek_ahead_for_paren() {
                    Self::parse_action_invocation(ctx)
                } else {
                    Err(ParseError {
                        message: format!("Unexpected identifier: {}", ctx.peek().lexeme),
                        line: ctx.peek().line,
                        column: ctx.peek().column,
                    })
                }
            }
            _ => Err(ParseError {
                message: format!("Unexpected token: {}", ctx.peek().lexeme),
                line: ctx.peek().line,
                column: ctx.peek().column,
            }),
        }
    }
    
    /// Parse action definition
    fn parse_action_definition(ctx: &mut ParserContext) -> ParseResult<Node> {
        let line = ctx.peek().line;
        ctx.consume(&TokenType::Action, "Expected 'action'")?;
        
        let name = match &ctx.advance().token_type {
            TokenType::Identifier(name) => name.clone(),
            _ => return Err(ParseError {
                message: "Expected action name".to_string(),
                line: ctx.previous().line,
                column: ctx.previous().column,
            }),
        };
        
        let mut body = Vec::new();
        
        while !ctx.check(&TokenType::EndAction) && !ctx.is_at_end() {
            ctx.skip_newlines_and_comments();
            if ctx.check(&TokenType::EndAction) || ctx.is_at_end() {
                break;
            }
            body.push(Self::parse_statement(ctx)?);
        }
        
        ctx.consume(&TokenType::EndAction, "Expected 'end action'")?;
        
        Ok(Node::ActionDefinition(ActionDefinition {
            name,
            body,
            line_number: Some(line),
        }))
    }
    
    /// Parse task definition
    fn parse_task_definition(ctx: &mut ParserContext) -> ParseResult<Node> {
        let line = ctx.peek().line;
        ctx.consume(&TokenType::Task, "Expected 'task'")?;
        
        let name = match &ctx.advance().token_type {
            TokenType::Identifier(name) => name.clone(),
            _ => return Err(ParseError {
                message: "Expected task name".to_string(),
                line: ctx.previous().line,
                column: ctx.previous().column,
            }),
        };
        
        let mut body = Vec::new();
        
        while !ctx.check(&TokenType::EndTask) && !ctx.is_at_end() {
            ctx.skip_newlines_and_comments();
            if ctx.check(&TokenType::EndTask) || ctx.is_at_end() {
                break;
            }
            body.push(Self::parse_statement(ctx)?);
        }
        
        ctx.consume(&TokenType::EndTask, "Expected 'end task'")?;
        
        Ok(Node::TaskAction(TaskAction {
            name,
            parameters: Vec::new(),
            body,
            line_number: Some(line),
        }))
    }
    
    /// Parse display statement
    fn parse_display_statement(ctx: &mut ParserContext) -> ParseResult<Node> {
        let line = ctx.peek().line;
        ctx.consume(&TokenType::Display, "Expected 'display'")?;
        
        let expression = Self::parse_expression(ctx)?;
        
        Ok(Node::DisplayStatement(DisplayStatement {
            expression: Box::new(expression),
            line_number: Some(line),
        }))
    }
    
    /// Parse set statement
    fn parse_set_statement(ctx: &mut ParserContext) -> ParseResult<Node> {
        let line = ctx.peek().line;
        ctx.consume(&TokenType::Set, "Expected 'set'")?;
        
        let variable = match &ctx.advance().token_type {
            TokenType::Identifier(name) => name.clone(),
            _ => return Err(ParseError {
                message: "Expected variable name".to_string(),
                line: ctx.previous().line,
                column: ctx.previous().column,
            }),
        };
        
        ctx.consume(&TokenType::To, "Expected 'to'")?;
        
        let value = Self::parse_expression(ctx)?;
        
        Ok(Node::Assignment(Assignment {
            variable,
            value: Box::new(value),
            line_number: Some(line),
        }))
    }
    
    /// Parse assignment
    fn parse_assignment(ctx: &mut ParserContext) -> ParseResult<Node> {
        let line = ctx.peek().line;
        
        let variable = match &ctx.advance().token_type {
            TokenType::Identifier(name) => name.clone(),
            _ => return Err(ParseError {
                message: "Expected variable name".to_string(),
                line: ctx.previous().line,
                column: ctx.previous().column,
            }),
        };
        
        ctx.consume(&TokenType::Is, "Expected 'is'")?;
        
        let value = Self::parse_expression(ctx)?;
        
        Ok(Node::Assignment(Assignment {
            variable,
            value: Box::new(value),
            line_number: Some(line),
        }))
    }
    
    /// Parse when statement (conditional)
    fn parse_when_statement(ctx: &mut ParserContext) -> ParseResult<Node> {
        let line = ctx.peek().line;
        ctx.consume(&TokenType::When, "Expected 'when'")?;
        
        let condition = Self::parse_expression(ctx)?;
        
        ctx.consume(&TokenType::Then, "Expected 'then'")?;
        
        let mut then_body = Vec::new();
        let mut else_body = None;
        
        while !ctx.check(&TokenType::EndWhen) && !ctx.check(&TokenType::Otherwise) && !ctx.is_at_end() {
            ctx.skip_newlines_and_comments();
            if ctx.check(&TokenType::EndWhen) || ctx.check(&TokenType::Otherwise) || ctx.is_at_end() {
                break;
            }
            then_body.push(Self::parse_statement(ctx)?);
        }
        
        if ctx.check(&TokenType::Otherwise) {
            ctx.advance(); // consume 'otherwise'
            let mut otherwise_body = Vec::new();
            
            while !ctx.check(&TokenType::EndWhen) && !ctx.is_at_end() {
                ctx.skip_newlines_and_comments();
                if ctx.check(&TokenType::EndWhen) || ctx.is_at_end() {
                    break;
                }
                otherwise_body.push(Self::parse_statement(ctx)?);
            }
            
            else_body = Some(otherwise_body);
        }
        
        ctx.consume(&TokenType::EndWhen, "Expected 'end when'")?;
        
        Ok(Node::IfStatement(IfStatement {
            condition: Box::new(condition),
            then_body,
            else_body,
            line_number: Some(line),
        }))
    }
    
    /// Parse while loop
    fn parse_while_loop(ctx: &mut ParserContext) -> ParseResult<Node> {
        let line = ctx.peek().line;
        ctx.consume(&TokenType::While, "Expected 'while'")?;
        
        let condition = Self::parse_expression(ctx)?;
        
        let mut body = Vec::new();
        
        while !ctx.check(&TokenType::EndWhile) && !ctx.is_at_end() {
            ctx.skip_newlines_and_comments();
            if ctx.check(&TokenType::EndWhile) || ctx.is_at_end() {
                break;
            }
            body.push(Self::parse_statement(ctx)?);
        }
        
        ctx.consume(&TokenType::EndWhile, "Expected 'end while'")?;
        
        Ok(Node::WhileLoop(WhileLoop {
            condition: Box::new(condition),
            body,
            line_number: Some(line),
        }))
    }
    
    /// Parse for loop
    fn parse_for_loop(ctx: &mut ParserContext) -> ParseResult<Node> {
        let line = ctx.peek().line;
        ctx.consume(&TokenType::For, "Expected 'for'")?;
        ctx.consume(&TokenType::Each, "Expected 'each'")?;
        
        let variable = match &ctx.advance().token_type {
            TokenType::Identifier(name) => name.clone(),
            _ => return Err(ParseError {
                message: "Expected variable name".to_string(),
                line: ctx.previous().line,
                column: ctx.previous().column,
            }),
        };
        
        ctx.consume(&TokenType::In, "Expected 'in'")?;
        
        let iterable = Self::parse_expression(ctx)?;
        
        let mut body = Vec::new();
        
        while !ctx.check(&TokenType::EndFor) && !ctx.is_at_end() {
            ctx.skip_newlines_and_comments();
            if ctx.check(&TokenType::EndFor) || ctx.is_at_end() {
                break;
            }
            body.push(Self::parse_statement(ctx)?);
        }
        
        ctx.consume(&TokenType::EndFor, "Expected 'end for'")?;
        
        Ok(Node::ForEachLoop(ForEachLoop {
            variable,
            iterable: Box::new(iterable),
            body,
            line_number: Some(line),
        }))
    }
    
    /// Parse return statement
    fn parse_return_statement(ctx: &mut ParserContext) -> ParseResult<Node> {
        let line = ctx.peek().line;
        ctx.consume(&TokenType::Give, "Expected 'give'")?;
        
        let expression = Self::parse_expression(ctx)?;
        
        Ok(Node::ReturnStatement(ReturnStatement {
            expression: Box::new(expression),
            return_type: "give".to_string(),
            line_number: Some(line),
        }))
    }
    
    /// Parse include statement
    fn parse_include_statement(ctx: &mut ParserContext) -> ParseResult<Node> {
        let line = ctx.peek().line;
        ctx.consume(&TokenType::Include, "Expected 'include'")?;
        
        let file_path = match &ctx.advance().token_type {
            TokenType::StringLiteral(path) => path.clone(),
            _ => return Err(ParseError {
                message: "Expected file path string".to_string(),
                line: ctx.previous().line,
                column: ctx.previous().column,
            }),
        };
        
        Ok(Node::IncludeStatement(IncludeStatement {
            module_name: file_path.clone(),
            file_path,
            line_number: Some(line),
        }))
    }
    
    /// Parse action invocation
    fn parse_action_invocation(ctx: &mut ParserContext) -> ParseResult<Node> {
        let line = ctx.peek().line;
        
        let action_name = match &ctx.advance().token_type {
            TokenType::Identifier(name) => name.clone(),
            _ => return Err(ParseError {
                message: "Expected action name".to_string(),
                line: ctx.previous().line,
                column: ctx.previous().column,
            }),
        };
        
        ctx.consume(&TokenType::LeftParen, "Expected '('")?;
        
        let mut arguments = Vec::new();
        
        while !ctx.check(&TokenType::RightParen) && !ctx.is_at_end() {
            arguments.push(Self::parse_expression(ctx)?);
            
            if !ctx.check(&TokenType::RightParen) {
                ctx.consume(&TokenType::Comma, "Expected ',' between arguments")?;
            }
        }
        
        ctx.consume(&TokenType::RightParen, "Expected ')'")?;
        
        Ok(Node::ActionInvocationWithArgs(ActionInvocationWithArgs {
            module_name: None,
            action_name,
            arguments,
            line_number: Some(line),
        }))
    }
    
    /// Parse expression (delegated to expression parser for now)
    fn parse_expression(ctx: &mut ParserContext) -> ParseResult<Node> {
        // For now, use a simple expression parsing
        // In the future, we could integrate the ExpressionParser here
        match &ctx.advance().token_type {
            TokenType::BooleanLiteral(value) => Ok(Node::Literal(Literal {
                value: LiteralValue::Boolean(*value),
                literal_type: "boolean".to_string(),
                line_number: Some(ctx.previous().line),
            })),
            TokenType::NumberLiteral(value) => Ok(Node::Literal(Literal {
                value: if value.fract() == 0.0 {
                    LiteralValue::Integer(*value as i64)
                } else {
                    LiteralValue::Float(*value)
                },
                literal_type: "number".to_string(),
                line_number: Some(ctx.previous().line),
            })),
            TokenType::StringLiteral(value) => Ok(Node::Literal(Literal {
                value: LiteralValue::String(value.clone()),
                literal_type: "string".to_string(),
                line_number: Some(ctx.previous().line),
            })),
            TokenType::Identifier(name) => Ok(Node::Identifier(Identifier {
                name: name.clone(),
                line_number: Some(ctx.previous().line),
            })),
            _ => Err(ParseError {
                message: format!("Unexpected token in expression: {}", ctx.previous().lexeme),
                line: ctx.previous().line,
                column: ctx.previous().column,
            }),
        }
    }
}