//! Base parser utilities and shared functionality

use crate::ast::*;
use crate::lexer::{Token, TokenType};

/// Base parser trait with common utilities
pub trait BaseParser {
    fn tokens(&self) -> &Vec<Token>;
    fn current(&self) -> usize;
    fn set_current(&mut self, pos: usize);
    
    /// Check if we're at the end of tokens
    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::Eof
    }
    
    /// Peek at current token without consuming
    fn peek(&self) -> &Token {
        &self.tokens()[self.current()]
    }
    
    /// Get previous token
    fn previous(&self) -> &Token {
        &self.tokens()[self.current() - 1]
    }
    
    /// Advance to next token
    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.set_current(self.current() + 1);
        }
        self.previous()
    }
    
    /// Check if current token matches any of the given types
    fn check(&self, token_type: &TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            std::mem::discriminant(&self.peek().token_type) == std::mem::discriminant(token_type)
        }
    }
    
    /// Match and consume tokens if they match
    fn match_tokens(&mut self, token_types: &[TokenType]) -> bool {
        for token_type in token_types {
            if self.check(token_type) {
                self.advance();
                return true;
            }
        }
        false
    }
    
    /// Consume a specific token type or return error
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
    
    /// Skip newlines and comments
    fn skip_newlines_and_comments(&mut self) {
        while !self.is_at_end() {
            match &self.peek().token_type {
                TokenType::Newline | TokenType::Comment(_) => {
                    self.advance();
                }
                _ => break,
            }
        }
    }
    
    /// Look ahead for 'is' token (for assignments)
    fn peek_ahead_for_is(&self) -> bool {
        let mut pos = self.current() + 1;
        while pos < self.tokens().len() {
            match &self.tokens()[pos].token_type {
                TokenType::Is => return true,
                TokenType::Newline | TokenType::Comment(_) => pos += 1,
                _ => return false,
            }
        }
        false
    }
    
    /// Look ahead for parentheses (for function calls)
    fn peek_ahead_for_paren(&self) -> bool {
        let mut pos = self.current() + 1;
        while pos < self.tokens().len() {
            match &self.tokens()[pos].token_type {
                TokenType::LeftParen => return true,
                TokenType::Newline | TokenType::Comment(_) => pos += 1,
                _ => return false,
            }
        }
        false
    }
}

/// Parser context for sharing state between modules
pub struct ParserContext {
    pub tokens: Vec<Token>,
    pub current: usize,
}

impl ParserContext {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }
}

impl BaseParser for ParserContext {
    fn tokens(&self) -> &Vec<Token> {
        &self.tokens
    }
    
    fn current(&self) -> usize {
        self.current
    }
    
    fn set_current(&mut self, pos: usize) {
        self.current = pos;
    }
}