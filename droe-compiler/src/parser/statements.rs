//! Statement parsing module - basic statements like display, assignment, conditionals, loops

use crate::ast::*;
use crate::lexer::TokenType;
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
    
    /// Parse set statement - supports both 'set var to value' and 'set var which is type to value'
    fn parse_set_statement(ctx: &mut ParserContext) -> ParseResult<Node> {
        let line = ctx.peek().line;
        ctx.consume(&TokenType::Set, "Expected 'set'")?;
        
        // Collect all tokens until we find 'to' to reconstruct the full line text
        let mut content_tokens = Vec::new();
        while !ctx.check(&TokenType::To) && !ctx.is_at_end() {
            content_tokens.push(ctx.advance().lexeme.clone());
        }
        
        if !ctx.check(&TokenType::To) {
            return Err(ParseError {
                message: "Invalid set statement - missing 'to'".to_string(),
                line,
                column: 0, // Use 0 since we don't track specific column within the set statement
            });
        }
        
        let var_part = content_tokens.join(" ");
        ctx.advance(); // consume 'to'
        
        // Parse the value expression
        let value = Self::parse_expression(ctx)?;
        
        // Extract variable name and type declaration
        let (variable, declared_type) = if var_part.contains(" which is ") {
            let parts: Vec<&str> = var_part.split(" which is ").collect();
            let var_name = parts[0].trim().to_string();
            let type_name = if parts.len() > 1 {
                Some(parts[1].trim().to_string())
            } else {
                None
            };
            (var_name, type_name)
        } else {
            (var_part.trim().to_string(), None)
        };
        
        if variable.is_empty() {
            return Err(ParseError {
                message: "Expected variable name".to_string(),
                line,
                column: 0,
            });
        }
        
        Ok(Node::Assignment(Assignment {
            variable,
            declared_type,
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
            declared_type: None, // Simple assignment without type declaration
            value: Box::new(value),
            line_number: Some(line),
        }))
    }
    
    /// Parse when statement (conditional) with support for else-if chains
    fn parse_when_statement(ctx: &mut ParserContext) -> ParseResult<Node> {
        let line = ctx.peek().line;
        ctx.consume(&TokenType::When, "Expected 'when'")?;
        
        let condition = Self::parse_expression(ctx)?;
        
        ctx.consume(&TokenType::Then, "Expected 'then'")?;
        
        let mut then_body = Vec::new();
        let mut elseif_clauses = Vec::new();
        let mut else_body = None;
        
        // Parse the main 'then' body
        while !ctx.check(&TokenType::EndWhen) && !ctx.check(&TokenType::Otherwise) && !ctx.is_at_end() {
            ctx.skip_newlines_and_comments();
            if ctx.check(&TokenType::EndWhen) || ctx.check(&TokenType::Otherwise) || ctx.is_at_end() {
                break;
            }
            then_body.push(Self::parse_statement(ctx)?);
        }
        
        // Parse else-if clauses (otherwise when ...)
        while ctx.check(&TokenType::Otherwise) {
            ctx.advance(); // consume 'otherwise'
            ctx.skip_newlines_and_comments();
            
            // Check if this is "otherwise when" (else-if) or just "otherwise" (else)
            if ctx.check(&TokenType::When) {
                // This is an else-if clause
                let elseif_line = ctx.peek().line;
                ctx.advance(); // consume 'when'
                
                let elseif_condition = Self::parse_expression(ctx)?;
                ctx.consume(&TokenType::Then, "Expected 'then' after 'otherwise when'")?;
                
                let mut elseif_body = Vec::new();
                
                while !ctx.check(&TokenType::EndWhen) && !ctx.check(&TokenType::Otherwise) && !ctx.is_at_end() {
                    ctx.skip_newlines_and_comments();
                    if ctx.check(&TokenType::EndWhen) || ctx.check(&TokenType::Otherwise) || ctx.is_at_end() {
                        break;
                    }
                    elseif_body.push(Self::parse_statement(ctx)?);
                }
                
                elseif_clauses.push(ElseIfClause {
                    condition: Box::new(elseif_condition),
                    body: elseif_body,
                    line_number: Some(elseif_line),
                });
            } else {
                // This is the final 'otherwise' (else) clause
                let mut otherwise_body = Vec::new();
                
                while !ctx.check(&TokenType::EndWhen) && !ctx.is_at_end() {
                    ctx.skip_newlines_and_comments();
                    if ctx.check(&TokenType::EndWhen) || ctx.is_at_end() {
                        break;
                    }
                    otherwise_body.push(Self::parse_statement(ctx)?);
                }
                
                else_body = Some(otherwise_body);
                break; // Final else clause, exit the loop
            }
        }
        
        ctx.consume(&TokenType::EndWhen, "Expected 'end when'")?;
        
        Ok(Node::IfStatement(IfStatement {
            condition: Box::new(condition),
            then_body,
            elseif_clauses,
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
        
        // Check if this is character iteration by looking ahead
        let is_char_iteration = if let TokenType::Identifier(name) = &ctx.peek().token_type {
            name == "char"
        } else if let TokenType::Char = &ctx.peek().token_type {
            true
        } else {
            false
        };
        
        if is_char_iteration {
            ctx.advance(); // consume 'char' (either as identifier or keyword)
            ctx.consume(&TokenType::In, "Expected 'in'")?;
            
            let string_expr = Self::parse_expression(ctx)?;
            
            // For character iteration, the variable is typically a single character variable
            // We'll use a default name or allow custom naming in the future
            let variable = "char".to_string();
            
            let mut body = Vec::new();
            
            while !ctx.check(&TokenType::EndFor) && !ctx.is_at_end() {
                ctx.skip_newlines_and_comments();
                if ctx.check(&TokenType::EndFor) || ctx.is_at_end() {
                    break;
                }
                body.push(Self::parse_statement(ctx)?);
            }
            
            ctx.consume(&TokenType::EndFor, "Expected 'end for'")?;
            
            return Ok(Node::ForEachCharLoop(ForEachCharLoop {
                variable,
                string_expr: Box::new(string_expr),
                body,
                line_number: Some(line),
            }));
        }
        
        // Regular for each loop
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
    
    /// Parse expression with support for logical operators and grouping
    fn parse_expression(ctx: &mut ParserContext) -> ParseResult<Node> {
        Self::parse_logical_or(ctx)
    }
    
    /// Parse logical OR expressions (lowest precedence)
    fn parse_logical_or(ctx: &mut ParserContext) -> ParseResult<Node> {
        let mut expr = Self::parse_logical_and(ctx)?;
        
        while ctx.check(&TokenType::Or) {
            ctx.advance(); // consume 'or'
            let right = Self::parse_logical_and(ctx)?;
            expr = Node::BinaryOp(BinaryOp {
                left: Box::new(expr),
                operator: "or".to_string(),
                right: Box::new(right),
                line_number: Some(ctx.previous().line),
            });
        }
        
        Ok(expr)
    }
    
    /// Parse logical AND expressions (higher precedence than OR)
    fn parse_logical_and(ctx: &mut ParserContext) -> ParseResult<Node> {
        let mut expr = Self::parse_comparison(ctx)?;
        
        while ctx.check(&TokenType::And) {
            ctx.advance(); // consume 'and'
            let right = Self::parse_comparison(ctx)?;
            expr = Node::BinaryOp(BinaryOp {
                left: Box::new(expr),
                operator: "and".to_string(),
                right: Box::new(right),
                line_number: Some(ctx.previous().line),
            });
        }
        
        Ok(expr)
    }
    
    /// Parse comparison expressions
    fn parse_comparison(ctx: &mut ParserContext) -> ParseResult<Node> {
        let left = Self::parse_primary(ctx)?;
        
        // Check for comparison operators
        let operator = match &ctx.peek().token_type {
            TokenType::IsGreaterThan => "is greater than",
            TokenType::IsLessThan => "is less than",
            TokenType::IsGreaterThanOrEqualTo => "is greater than or equal to",
            TokenType::IsLessThanOrEqualTo => "is less than or equal to",
            TokenType::Equals => "equals",
            TokenType::DoesNotEqual => "does not equal",
            TokenType::Is => "is",
            _ => return Ok(left), // Not a comparison
        };
        
        ctx.advance(); // consume operator
        let right = Self::parse_primary(ctx)?;
        
        Ok(Node::BinaryOp(BinaryOp {
            left: Box::new(left),
            operator: operator.to_string(),
            right: Box::new(right),
            line_number: Some(ctx.previous().line),
        }))
    }
    
    /// Parse primary expressions (literals, identifiers, parenthesized expressions)
    fn parse_primary(ctx: &mut ParserContext) -> ParseResult<Node> {
        // Handle parenthesized expressions
        if ctx.check(&TokenType::LeftParen) {
            ctx.advance(); // consume '('
            let expr = Self::parse_logical_or(ctx)?; // Parse full expression inside parentheses
            ctx.consume(&TokenType::RightParen, "Expected ')' after expression")?;
            return Ok(expr);
        }
        
        // Check if it's a string literal that needs interpolation
        let is_interpolation = if let TokenType::StringLiteral(value) = &ctx.peek().token_type {
            value.contains('[') && value.contains(']')
        } else {
            false
        };
        
        if is_interpolation {
            if let TokenType::StringLiteral(value) = &ctx.advance().token_type {
                return Self::parse_string_interpolation(value.clone(), ctx.previous().line);
            }
        }
        
        // Parse literals and identifiers
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
    
    /// Parse string interpolation
    fn parse_string_interpolation(content: String, line: usize) -> ParseResult<Node> {
        let mut parts = Vec::new();
        let mut current_pos = 0;

        while current_pos < content.len() {
            // Find next interpolation
            if let Some(start) = content[current_pos..].find('[') {
                let abs_start = current_pos + start;
                
                // Add literal part before interpolation
                if abs_start > current_pos {
                    parts.push(Node::Literal(Literal {
                        value: LiteralValue::String(content[current_pos..abs_start].to_string()),
                        literal_type: "string".to_string(),
                        line_number: Some(line),
                    }));
                }
                
                // Find end of interpolation
                if let Some(end) = content[abs_start + 1..].find(']') {
                    let abs_end = abs_start + 1 + end;
                    let var_name = &content[abs_start + 1..abs_end];
                    
                    // Add the variable reference
                    parts.push(Node::Identifier(Identifier {
                        name: var_name.trim().to_string(),
                        line_number: Some(line),
                    }));
                    
                    current_pos = abs_end + 1;
                } else {
                    // No closing bracket found
                    return Err(ParseError {
                        message: "Unclosed interpolation bracket '['".to_string(),
                        line,
                        column: 0,
                    });
                }
            } else {
                // No more interpolations, add remaining text
                if current_pos < content.len() {
                    parts.push(Node::Literal(Literal {
                        value: LiteralValue::String(content[current_pos..].to_string()),
                        literal_type: "string".to_string(),
                        line_number: Some(line),
                    }));
                }
                break;
            }
        }

        Ok(Node::StringInterpolation(StringInterpolation {
            parts,
            line_number: Some(line),
        }))
    }
}