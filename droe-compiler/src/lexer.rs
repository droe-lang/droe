//! Lexical analysis for Droe DSL

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Keywords
    Module,
    EndModule,
    Data,
    EndData,
    Action,
    EndAction,
    Task,
    EndTask,
    When,
    Then,
    Otherwise,
    EndWhen,
    While,
    EndWhile,
    For,
    Each,
    In,
    EndFor,
    Give,
    Display,
    Set,
    To,
    Is,
    Include,
    From,
    
    // Word-based operators
    Plus,
    Minus,
    Times,
    DividedBy,
    DoesNotEqual,
    IsLessThan,
    IsGreaterThan,
    IsLessThanOrEqualTo,
    IsGreaterThanOrEqualTo,
    
    // Database and API keywords
    Call,
    Fetch,
    Update,
    Delete,
    Db,
    Serve,
    Using,
    Headers,
    EndHeaders,
    
    // UI Component keywords
    Title,
    Text,
    Input,
    Textarea,
    Dropdown,
    Toggle,
    Checkbox,
    Radio,
    Button,
    Image,
    Video,
    Audio,
    Slot,
    
    // Layout keywords
    Layout,
    EndLayout,
    Form,
    EndForm,
    Screen,
    EndScreen,
    Fragment,
    EndFragment,
    
    // Format keyword
    Format,
    
    // Operators
    Equals,
    NotEquals,
    GreaterThan,
    LessThan,
    GreaterEquals,
    LessEquals,
    Multiply,
    Divide,
    And,
    Or,
    Not,
    
    // Delimiters
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Semicolon,
    Colon,
    
    // Literals
    StringLiteral(String),
    NumberLiteral(f64),
    BooleanLiteral(bool),
    Identifier(String),
    
    // Special
    Newline,
    Whitespace(String),
    Comment(String),
    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub line: usize,
    pub column: usize,
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: String, line: usize, column: usize) -> Self {
        Self {
            token_type,
            lexeme,
            line,
            column,
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} '{}' at {}:{}", self.token_type, self.lexeme, self.line, self.column)
    }
}

pub struct Lexer {
    source: Vec<char>,
    current: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Self {
            source: source.chars().collect(),
            current: 0,
            line: 1,
            column: 1,
        }
    }
    
    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        
        while !self.is_at_end() {
            if let Some(token) = self.next_token() {
                tokens.push(token);
            }
        }
        
        tokens.push(Token::new(
            TokenType::Eof,
            String::new(),
            self.line,
            self.column,
        ));
        
        tokens
    }
    
    fn next_token(&mut self) -> Option<Token> {
        self.skip_whitespace();
        
        if self.is_at_end() {
            return None;
        }
        
        let start_line = self.line;
        let start_column = self.column;
        let c = self.advance();
        
        let token_type = match c {
            '(' => TokenType::LeftParen,
            ')' => TokenType::RightParen,
            '[' => TokenType::LeftBracket,
            ']' => TokenType::RightBracket,
            '{' => TokenType::LeftBrace,
            '}' => TokenType::RightBrace,
            ',' => TokenType::Comma,
            '.' => TokenType::Dot,
            ';' => TokenType::Semicolon,
            ':' => TokenType::Colon,
            '+' => TokenType::Plus,
            '-' => TokenType::Minus,
            '*' => TokenType::Multiply,
            '/' => {
                if self.peek() == '/' {
                    return self.comment();
                }
                TokenType::Divide
            }
            '=' => {
                if self.peek() == '=' {
                    self.advance();
                    TokenType::Equals
                } else {
                    // Single '=' is used in assignments like "x is value" 
                    return self.identifier_or_keyword();
                }
            }
            '!' => {
                if self.peek() == '=' {
                    self.advance();
                    TokenType::NotEquals
                } else {
                    TokenType::Not
                }
            }
            '>' => {
                if self.peek() == '=' {
                    self.advance();
                    TokenType::GreaterEquals
                } else {
                    TokenType::GreaterThan
                }
            }
            '<' => {
                if self.peek() == '=' {
                    self.advance();
                    TokenType::LessEquals
                } else {
                    TokenType::LessThan
                }
            }
            '"' => return self.string_literal(),
            '\'' => return self.string_literal(),
            '\n' => {
                self.line += 1;
                self.column = 1;
                TokenType::Newline
            }
            _ => {
                if c.is_ascii_digit() {
                    return self.number_literal();
                } else if c.is_ascii_alphabetic() || c == '_' {
                    return self.identifier_or_keyword();
                } else {
                    // Skip unknown characters
                    return None;
                }
            }
        };
        
        Some(Token::new(
            token_type,
            c.to_string(),
            start_line,
            start_column,
        ))
    }
    
    fn string_literal(&mut self) -> Option<Token> {
        let start_line = self.line;
        let start_column = self.column - 1; // Account for opening quote
        let quote_char = self.previous();
        let mut value = String::new();
        let mut lexeme = quote_char.to_string();
        
        while !self.is_at_end() && self.peek() != quote_char {
            if self.peek() == '\n' {
                self.line += 1;
                self.column = 1;
            }
            let c = self.advance();
            value.push(c);
            lexeme.push(c);
        }
        
        if self.is_at_end() {
            return None; // Unterminated string
        }
        
        // Consume closing quote
        lexeme.push(self.advance());
        
        Some(Token::new(
            TokenType::StringLiteral(value),
            lexeme,
            start_line,
            start_column,
        ))
    }
    
    fn number_literal(&mut self) -> Option<Token> {
        let start_line = self.line;
        let start_column = self.column - 1;
        let mut lexeme = self.previous().to_string();
        
        while !self.is_at_end() && self.peek().is_ascii_digit() {
            lexeme.push(self.advance());
        }
        
        // Handle decimal point
        if !self.is_at_end() && self.peek() == '.' && self.peek_next().is_ascii_digit() {
            lexeme.push(self.advance()); // consume '.'
            while !self.is_at_end() && self.peek().is_ascii_digit() {
                lexeme.push(self.advance());
            }
        }
        
        let value = lexeme.parse::<f64>().unwrap_or(0.0);
        
        Some(Token::new(
            TokenType::NumberLiteral(value),
            lexeme,
            start_line,
            start_column,
        ))
    }
    
    fn identifier_or_keyword(&mut self) -> Option<Token> {
        let start_line = self.line;
        let start_column = self.column - 1;
        let mut lexeme = self.previous().to_string();
        
        while !self.is_at_end() && (self.peek().is_ascii_alphanumeric() || self.peek() == '_') {
            lexeme.push(self.advance());
        }
        
        let token_type = match lexeme.as_str() {
            "module" => TokenType::Module,
            "end" => {
                // Look ahead to see what kind of end this is
                self.skip_whitespace();
                if !self.is_at_end() {
                    let mut next_word = String::new();
                    let saved_pos = self.current;
                    let saved_line = self.line;
                    let saved_column = self.column;
                    
                    while !self.is_at_end() && (self.peek().is_ascii_alphanumeric() || self.peek() == '_') {
                        next_word.push(self.advance());
                    }
                    
                    // Restore position
                    self.current = saved_pos;
                    self.line = saved_line;
                    self.column = saved_column;
                    
                    match next_word.as_str() {
                        "module" => {
                            // Consume "module"
                            for _ in 0..6 { self.advance(); }
                            lexeme = "end module".to_string();
                            TokenType::EndModule
                        }
                        "data" => {
                            for _ in 0..4 { self.advance(); }
                            lexeme = "end data".to_string();
                            TokenType::EndData
                        }
                        "action" => {
                            for _ in 0..6 { self.advance(); }
                            lexeme = "end action".to_string();
                            TokenType::EndAction
                        }
                        "task" => {
                            for _ in 0..4 { self.advance(); }
                            lexeme = "end task".to_string();
                            TokenType::EndTask
                        }
                        "when" => {
                            for _ in 0..4 { self.advance(); }
                            lexeme = "end when".to_string();
                            TokenType::EndWhen
                        }
                        "while" => {
                            for _ in 0..5 { self.advance(); }
                            lexeme = "end while".to_string();
                            TokenType::EndWhile
                        }
                        "for" => {
                            for _ in 0..3 { self.advance(); }
                            lexeme = "end for".to_string();
                            TokenType::EndFor
                        }
                        "layout" => {
                            for _ in 0..6 { self.advance(); }
                            lexeme = "end layout".to_string();
                            TokenType::EndLayout
                        }
                        "form" => {
                            for _ in 0..4 { self.advance(); }
                            lexeme = "end form".to_string();
                            TokenType::EndForm
                        }
                        "screen" => {
                            for _ in 0..6 { self.advance(); }
                            lexeme = "end screen".to_string();
                            TokenType::EndScreen
                        }
                        "fragment" => {
                            for _ in 0..8 { self.advance(); }
                            lexeme = "end fragment".to_string();
                            TokenType::EndFragment
                        }
                        "headers" => {
                            for _ in 0..7 { self.advance(); }
                            lexeme = "end headers".to_string();
                            TokenType::EndHeaders
                        }
                        _ => TokenType::Identifier(lexeme.clone()),
                    }
                } else {
                    TokenType::Identifier(lexeme.clone())
                }
            }
            "data" => TokenType::Data,
            "action" => TokenType::Action,
            "task" => TokenType::Task,
            "when" => TokenType::When,
            "then" => TokenType::Then,
            "otherwise" => TokenType::Otherwise,
            "while" => TokenType::While,
            "for" => TokenType::For,
            "each" => TokenType::Each,
            "in" => TokenType::In,
            "give" => TokenType::Give,
            "display" => TokenType::Display,
            "set" => TokenType::Set,
            "to" => TokenType::To,
            "is" => TokenType::Is,
            "include" => TokenType::Include,
            "from" => TokenType::From,
            "equals" => TokenType::Equals,
            "and" => TokenType::And,
            "or" => TokenType::Or,
            "not" => TokenType::Not,
            "true" => TokenType::BooleanLiteral(true),
            "false" => TokenType::BooleanLiteral(false),
            
            // Word-based operators
            "plus" => TokenType::Plus,
            "minus" => TokenType::Minus,
            "times" => TokenType::Times,
            "divided" => {
                // Check for "divided by"
                self.skip_whitespace();
                if self.match_word("by") {
                    lexeme = "divided by".to_string();
                    TokenType::DividedBy
                } else {
                    TokenType::Identifier(lexeme.clone())
                }
            }
            "does" => {
                // Check for "does not equal"
                self.skip_whitespace();
                if self.match_word("not") {
                    self.skip_whitespace();
                    if self.match_word("equal") {
                        lexeme = "does not equal".to_string();
                        TokenType::DoesNotEqual
                    } else {
                        TokenType::Identifier(lexeme.clone())
                    }
                } else {
                    TokenType::Identifier(lexeme.clone())
                }
            }
            
            // Database and API keywords
            "call" => TokenType::Call,
            "fetch" => TokenType::Fetch,
            "update" => TokenType::Update,
            "delete" => TokenType::Delete,
            "db" => TokenType::Db,
            "serve" => TokenType::Serve,
            "using" => TokenType::Using,
            "headers" => TokenType::Headers,
            
            // UI Component keywords
            "title" => TokenType::Title,
            "text" => TokenType::Text,
            "input" => TokenType::Input,
            "textarea" => TokenType::Textarea,
            "dropdown" => TokenType::Dropdown,
            "toggle" => TokenType::Toggle,
            "checkbox" => TokenType::Checkbox,
            "radio" => TokenType::Radio,
            "button" => TokenType::Button,
            "image" => TokenType::Image,
            "video" => TokenType::Video,
            "audio" => TokenType::Audio,
            "slot" => TokenType::Slot,
            
            // Layout keywords
            "layout" => TokenType::Layout,
            "form" => TokenType::Form,
            "screen" => TokenType::Screen,
            "fragment" => TokenType::Fragment,
            
            // Format keyword
            "Format" => TokenType::Format,
            
            _ => TokenType::Identifier(lexeme.clone()),
        };
        
        Some(Token::new(token_type, lexeme, start_line, start_column))
    }
    
    fn comment(&mut self) -> Option<Token> {
        let start_line = self.line;
        let start_column = self.column - 1;
        let mut lexeme = "//".to_string();
        
        // Skip the second '/'
        self.advance();
        
        while !self.is_at_end() && self.peek() != '\n' {
            lexeme.push(self.advance());
        }
        
        Some(Token::new(
            TokenType::Comment(lexeme.clone()),
            lexeme,
            start_line,
            start_column,
        ))
    }
    
    fn skip_whitespace(&mut self) {
        while !self.is_at_end() {
            match self.peek() {
                ' ' | '\r' | '\t' => {
                    self.advance();
                }
                _ => break,
            }
        }
    }
    
    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }
    
    fn advance(&mut self) -> char {
        let c = self.source[self.current];
        self.current += 1;
        self.column += 1;
        c
    }
    
    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.source[self.current]
        }
    }
    
    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            '\0'
        } else {
            self.source[self.current + 1]
        }
    }
    
    fn previous(&self) -> char {
        if self.current == 0 {
            '\0'
        } else {
            self.source[self.current - 1]
        }
    }
    
    fn match_word(&mut self, word: &str) -> bool {
        let saved_pos = self.current;
        let saved_line = self.line;
        let saved_column = self.column;
        
        for expected_char in word.chars() {
            if self.is_at_end() || self.peek() != expected_char {
                // Restore position
                self.current = saved_pos;
                self.line = saved_line;
                self.column = saved_column;
                return false;
            }
            self.advance();
        }
        
        // Check that the word ends at a word boundary
        if !self.is_at_end() && (self.peek().is_ascii_alphanumeric() || self.peek() == '_') {
            // Restore position
            self.current = saved_pos;
            self.line = saved_line;
            self.column = saved_column;
            return false;
        }
        
        true
    }
}