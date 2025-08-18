//! Advanced expression parsing for Droe DSL

use crate::ast::*;
use crate::lexer::{Token, TokenType};
// use std::collections::HashMap;

/// Expression parser that handles advanced expression constructs
pub struct ExpressionParser {
    tokens: Vec<Token>,
    current: usize,
}

impl ExpressionParser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    /// Parse a complete expression from tokens
    pub fn parse_expression(&mut self) -> ParseResult<Node> {
        self.parse_logical_or()
    }

    /// Parse logical OR expressions
    fn parse_logical_or(&mut self) -> ParseResult<Node> {
        let mut expr = self.parse_logical_and()?;

        while self.match_tokens(&[TokenType::Or]) {
            let operator = "or".to_string();
            let right = self.parse_logical_and()?;
            expr = Node::BinaryOp(BinaryOp {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
                line_number: Some(self.previous().line),
            });
        }

        Ok(expr)
    }

    /// Parse logical AND expressions
    fn parse_logical_and(&mut self) -> ParseResult<Node> {
        let mut expr = self.parse_equality()?;

        while self.match_tokens(&[TokenType::And]) {
            let operator = "and".to_string();
            let right = self.parse_equality()?;
            expr = Node::BinaryOp(BinaryOp {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
                line_number: Some(self.previous().line),
            });
        }

        Ok(expr)
    }

    /// Parse equality expressions (equals, does not equal)
    fn parse_equality(&mut self) -> ParseResult<Node> {
        let mut expr = self.parse_comparison()?;

        while self.match_tokens(&[TokenType::Equals, TokenType::DoesNotEqual, TokenType::NotEquals]) {
            let operator = match self.previous().token_type {
                TokenType::Equals => "equals",
                TokenType::DoesNotEqual => "does not equal",
                TokenType::NotEquals => "not equals",
                _ => "equals",
            }.to_string();
            let right = self.parse_comparison()?;
            expr = Node::BinaryOp(BinaryOp {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
                line_number: Some(self.previous().line),
            });
        }

        Ok(expr)
    }

    /// Parse comparison expressions
    fn parse_comparison(&mut self) -> ParseResult<Node> {
        let mut expr = self.parse_term()?;

        while self.match_tokens(&[
            TokenType::GreaterThan,
            TokenType::GreaterEquals,
            TokenType::LessThan,
            TokenType::LessEquals,
            TokenType::IsGreaterThan,
            TokenType::IsLessThan,
            TokenType::IsGreaterThanOrEqualTo,
            TokenType::IsLessThanOrEqualTo,
        ]) {
            let operator = match self.previous().token_type {
                TokenType::GreaterThan => ">",
                TokenType::GreaterEquals => ">=",
                TokenType::LessThan => "<",
                TokenType::LessEquals => "<=",
                TokenType::IsGreaterThan => "is greater than",
                TokenType::IsLessThan => "is less than",
                TokenType::IsGreaterThanOrEqualTo => "is greater than or equal to",
                TokenType::IsLessThanOrEqualTo => "is less than or equal to",
                _ => ">",
            }.to_string();
            let right = self.parse_term()?;
            expr = Node::BinaryOp(BinaryOp {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
                line_number: Some(self.previous().line),
            });
        }

        Ok(expr)
    }

    /// Parse addition and subtraction (word-based and symbol-based)
    fn parse_term(&mut self) -> ParseResult<Node> {
        let mut expr = self.parse_factor()?;

        while self.match_tokens(&[TokenType::Minus, TokenType::Plus]) {
            let operator = match self.previous().token_type {
                TokenType::Plus => "add",
                TokenType::Minus => "subtract",
                _ => "add",
            }.to_string();
            let right = self.parse_factor()?;
            expr = Node::ArithmeticOp(ArithmeticOp {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
                line_number: Some(self.previous().line),
            });
        }

        Ok(expr)
    }

    /// Parse multiplication and division (word-based and symbol-based)
    fn parse_factor(&mut self) -> ParseResult<Node> {
        let mut expr = self.parse_unary()?;

        while self.match_tokens(&[TokenType::Divide, TokenType::Multiply, TokenType::Times, TokenType::DividedBy]) {
            let operator = match self.previous().token_type {
                TokenType::Multiply => "multiply",
                TokenType::Divide => "divide",
                TokenType::Times => "multiply",
                TokenType::DividedBy => "divide",
                _ => "multiply",
            }.to_string();
            let right = self.parse_unary()?;
            expr = Node::ArithmeticOp(ArithmeticOp {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
                line_number: Some(self.previous().line),
            });
        }

        Ok(expr)
    }

    /// Parse unary expressions
    fn parse_unary(&mut self) -> ParseResult<Node> {
        if self.match_tokens(&[TokenType::Not, TokenType::Minus]) {
            let operator = self.previous().lexeme.clone();
            let right = self.parse_unary()?;
            return Ok(Node::BinaryOp(BinaryOp {
                left: Box::new(Node::Literal(Literal {
                    value: LiteralValue::Boolean(false),
                    literal_type: "boolean".to_string(),
                    line_number: Some(self.previous().line),
                })),
                operator,
                right: Box::new(right),
                line_number: Some(self.previous().line),
            }));
        }

        self.parse_call()
    }

    /// Parse function calls and Format expressions
    fn parse_call(&mut self) -> ParseResult<Node> {
        let mut expr = self.parse_primary()?;

        loop {
            if self.match_tokens(&[TokenType::LeftParen]) {
                // This is a function call - for now we only support Format
                if let Node::Identifier(ref id) = expr {
                    if id.name == "Format" {
                        expr = self.parse_format_call()?;
                    } else {
                        return Err(ParseError {
                            message: format!("Unknown function: {}", id.name),
                            line: self.previous().line,
                            column: self.previous().column,
                        });
                    }
                } else {
                    return Err(ParseError {
                        message: "Invalid function call".to_string(),
                        line: self.previous().line,
                        column: self.previous().column,
                    });
                }
            } else if self.match_tokens(&[TokenType::Dot]) {
                // Property access
                let property = match &self.advance().token_type {
                    TokenType::Identifier(name) => name.clone(),
                    _ => return Err(ParseError {
                        message: "Expected property name after '.'".to_string(),
                        line: self.previous().line,
                        column: self.previous().column,
                    }),
                };
                
                expr = Node::PropertyAccess(PropertyAccess {
                    object: Box::new(expr),
                    property,
                    line_number: Some(self.previous().line),
                });
            } else if self.match_tokens(&[TokenType::LeftBracket]) {
                // Array access
                let index = self.parse_expression()?;
                self.consume(&TokenType::RightBracket, "Expected ']' after array index")?;
                
                expr = Node::PropertyAccess(PropertyAccess {
                    object: Box::new(expr),
                    property: format!("[{}]", match index {
                        Node::Literal(Literal { value: LiteralValue::Integer(i), .. }) => i.to_string(),
                        Node::Identifier(Identifier { name, .. }) => name,
                        _ => "0".to_string(),
                    }),
                    line_number: Some(self.previous().line),
                });
            } else {
                break;
            }
        }

        Ok(expr)
    }

    /// Parse Format function call: Format(expression, "pattern")
    fn parse_format_call(&mut self) -> ParseResult<Node> {
        let expression = self.parse_expression()?;
        self.consume(&TokenType::Comma, "Expected ',' after format expression")?;
        
        let format_pattern = match &self.advance().token_type {
            TokenType::StringLiteral(pattern) => pattern.clone(),
            _ => return Err(ParseError {
                message: "Expected format pattern string".to_string(),
                line: self.previous().line,
                column: self.previous().column,
            }),
        };
        
        self.consume(&TokenType::RightParen, "Expected ')' after format pattern")?;
        
        Ok(Node::FormatExpression(FormatExpression {
            expression: Box::new(expression),
            format_pattern,
            line_number: Some(self.previous().line),
        }))
    }

    /// Parse primary expressions (literals, identifiers, parenthesized expressions, arrays)
    fn parse_primary(&mut self) -> ParseResult<Node> {
        let token = self.advance().token_type.clone();
        match &token {
            TokenType::BooleanLiteral(value) => Ok(Node::Literal(Literal {
                value: LiteralValue::Boolean(*value),
                literal_type: "boolean".to_string(),
                line_number: Some(self.previous().line),
            })),
            TokenType::NumberLiteral(value) => Ok(Node::Literal(Literal {
                value: if value.fract() == 0.0 {
                    LiteralValue::Integer(*value as i64)
                } else {
                    LiteralValue::Float(*value)
                },
                literal_type: "number".to_string(),
                line_number: Some(self.previous().line),
            })),
            TokenType::StringLiteral(value) => {
                // Check for string interpolation
                if value.contains('[') && value.contains(']') {
                    self.parse_string_interpolation(value.clone())
                } else {
                    Ok(Node::Literal(Literal {
                        value: LiteralValue::String(value.clone()),
                        literal_type: "string".to_string(),
                        line_number: Some(self.previous().line),
                    }))
                }
            }
            TokenType::Identifier(name) => Ok(Node::Identifier(Identifier {
                name: name.clone(),
                line_number: Some(self.previous().line),
            })),
            TokenType::Format => Ok(Node::Identifier(Identifier {
                name: "Format".to_string(),
                line_number: Some(self.previous().line),
            })),
            TokenType::LeftParen => {
                let expr = self.parse_expression()?;
                self.consume(&TokenType::RightParen, "Expected ')' after expression")?;
                Ok(expr)
            }
            TokenType::LeftBracket => self.parse_array_literal(),
            _ => Err(ParseError {
                message: format!("Unexpected token in expression: {}", self.previous().lexeme),
                line: self.previous().line,
                column: self.previous().column,
            }),
        }
    }

    /// Parse array literal: [element1, element2, ...]
    fn parse_array_literal(&mut self) -> ParseResult<Node> {
        let mut elements = Vec::new();

        if !self.check(&TokenType::RightBracket) {
            loop {
                elements.push(self.parse_expression()?);
                
                if !self.match_tokens(&[TokenType::Comma]) {
                    break;
                }
            }
        }

        self.consume(&TokenType::RightBracket, "Expected ']' after array elements")?;

        Ok(Node::ArrayLiteral(ArrayLiteral {
            elements,
            line_number: Some(self.previous().line),
        }))
    }

    /// Parse string interpolation: "Hello [name]"
    fn parse_string_interpolation(&mut self, content: String) -> ParseResult<Node> {
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
                        line_number: Some(self.previous().line),
                    }));
                }
                
                // Find end of interpolation
                if let Some(end) = content[abs_start + 1..].find(']') {
                    let abs_end = abs_start + 1 + end;
                    let expr_str = &content[abs_start + 1..abs_end];
                    
                    // Parse the interpolated expression as an identifier for now
                    // In a real implementation, we'd tokenize and parse this properly
                    parts.push(Node::Identifier(Identifier {
                        name: expr_str.trim().to_string(),
                        line_number: Some(self.previous().line),
                    }));
                    
                    current_pos = abs_end + 1;
                } else {
                    return Err(ParseError {
                        message: "Unclosed interpolation in string".to_string(),
                        line: self.previous().line,
                        column: self.previous().column,
                    });
                }
            } else {
                // No more interpolations, add rest as literal
                if current_pos < content.len() {
                    parts.push(Node::Literal(Literal {
                        value: LiteralValue::String(content[current_pos..].to_string()),
                        literal_type: "string".to_string(),
                        line_number: Some(self.previous().line),
                    }));
                }
                break;
            }
        }

        Ok(Node::StringInterpolation(StringInterpolation {
            parts,
            line_number: Some(self.previous().line),
        }))
    }

    // Utility methods
    fn match_tokens(&mut self, token_types: &[TokenType]) -> bool {
        for token_type in token_types {
            if self.check(token_type) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn check(&self, token_type: &TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            std::mem::discriminant(&self.peek().token_type) == std::mem::discriminant(token_type)
        }
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::Eof
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn consume(&mut self, token_type: &TokenType, message: &str) -> ParseResult<&Token> {
        if self.check(token_type) {
            Ok(self.advance())
        } else {
            Err(ParseError {
                message: message.to_string(),
                line: self.peek().line,
                column: self.peek().column,
            })
        }
    }
}