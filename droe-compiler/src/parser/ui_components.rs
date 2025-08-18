//! UI Components parsing module

use crate::ast::*;
use crate::lexer::TokenType;
use super::base::{BaseParser, ParserContext};

pub struct UIComponentParser;

impl UIComponentParser {
    /// Parse UI component based on token type
    pub fn parse_component(ctx: &mut ParserContext) -> ParseResult<Node> {
        match &ctx.peek().token_type {
            TokenType::Title => Self::parse_title_component(ctx),
            TokenType::Text => Self::parse_text_component(ctx),
            TokenType::Input => Self::parse_input_component(ctx),
            TokenType::Textarea => Self::parse_textarea_component(ctx),
            TokenType::Dropdown => Self::parse_dropdown_component(ctx),
            TokenType::Toggle => Self::parse_toggle_component(ctx),
            TokenType::Checkbox => Self::parse_checkbox_component(ctx),
            TokenType::Radio => Self::parse_radio_component(ctx),
            TokenType::Button => Self::parse_button_component(ctx),
            TokenType::Image => Self::parse_image_component(ctx),
            TokenType::Video => Self::parse_video_component(ctx),
            TokenType::Audio => Self::parse_audio_component(ctx),
            TokenType::Slot => Self::parse_slot_component(ctx),
            _ => Err(ParseError {
                message: "Not a UI component".to_string(),
                line: ctx.peek().line,
                column: ctx.peek().column,
            })
        }
    }
    
    fn parse_title_component(ctx: &mut ParserContext) -> ParseResult<Node> {
        let line = ctx.peek().line;
        ctx.consume(&TokenType::Title, "Expected 'title'")?;
        
        let text = match &ctx.advance().token_type {
            TokenType::StringLiteral(text) => text.clone(),
            _ => return Err(ParseError {
                message: "Expected title text".to_string(),
                line: ctx.previous().line,
                column: ctx.previous().column,
            }),
        };
        
        Ok(Node::TitleComponent(TitleComponent {
            text,
            attributes: Vec::new(),
            classes: Vec::new(),
            styles: None,
            line_number: Some(line),
        }))
    }
    
    fn parse_text_component(ctx: &mut ParserContext) -> ParseResult<Node> {
        let line = ctx.peek().line;
        ctx.consume(&TokenType::Text, "Expected 'text'")?;
        
        let text = match &ctx.advance().token_type {
            TokenType::StringLiteral(text) => text.clone(),
            _ => return Err(ParseError {
                message: "Expected text content".to_string(),
                line: ctx.previous().line,
                column: ctx.previous().column,
            }),
        };
        
        Ok(Node::TextComponent(TextComponent {
            text,
            attributes: Vec::new(),
            classes: Vec::new(),
            styles: None,
            line_number: Some(line),
        }))
    }
    
    fn parse_input_component(ctx: &mut ParserContext) -> ParseResult<Node> {
        let line = ctx.peek().line;
        ctx.consume(&TokenType::Input, "Expected 'input'")?;
        
        let input_type = match &ctx.advance().token_type {
            TokenType::StringLiteral(input_type) => input_type.clone(),
            TokenType::Identifier(input_type) => input_type.clone(),
            _ => "text".to_string(),
        };
        
        Ok(Node::InputComponent(InputComponent {
            input_type,
            binding: None,
            attributes: Vec::new(),
            element_id: None,
            classes: Vec::new(),
            styles: None,
            line_number: Some(line),
        }))
    }
    
    fn parse_textarea_component(ctx: &mut ParserContext) -> ParseResult<Node> {
        let line = ctx.peek().line;
        ctx.consume(&TokenType::Textarea, "Expected 'textarea'")?;
        
        Ok(Node::TextareaComponent(TextareaComponent {
            label: None,
            placeholder: None,
            rows: None,
            binding: None,
            attributes: Vec::new(),
            classes: Vec::new(),
            styles: None,
            element_id: None,
            line_number: Some(line),
        }))
    }
    
    fn parse_dropdown_component(ctx: &mut ParserContext) -> ParseResult<Node> {
        let line = ctx.peek().line;
        ctx.consume(&TokenType::Dropdown, "Expected 'dropdown'")?;
        
        Ok(Node::DropdownComponent(DropdownComponent {
            label: None,
            options: Vec::new(),
            binding: None,
            attributes: Vec::new(),
            element_id: None,
            classes: Vec::new(),
            styles: None,
            line_number: Some(line),
        }))
    }
    
    fn parse_toggle_component(ctx: &mut ParserContext) -> ParseResult<Node> {
        let line = ctx.peek().line;
        ctx.consume(&TokenType::Toggle, "Expected 'toggle'")?;
        
        Ok(Node::ToggleComponent(ToggleComponent {
            binding: None,
            attributes: Vec::new(),
            classes: Vec::new(),
            styles: None,
            element_id: None,
            line_number: Some(line),
        }))
    }
    
    fn parse_checkbox_component(ctx: &mut ParserContext) -> ParseResult<Node> {
        let line = ctx.peek().line;
        ctx.consume(&TokenType::Checkbox, "Expected 'checkbox'")?;
        
        Ok(Node::CheckboxComponent(CheckboxComponent {
            text: None,
            binding: None,
            attributes: Vec::new(),
            element_id: None,
            classes: Vec::new(),
            styles: None,
            line_number: Some(line),
        }))
    }
    
    fn parse_radio_component(ctx: &mut ParserContext) -> ParseResult<Node> {
        let line = ctx.peek().line;
        ctx.consume(&TokenType::Radio, "Expected 'radio'")?;
        
        Ok(Node::RadioComponent(RadioComponent {
            text: None,
            value: None,
            binding: None,
            attributes: Vec::new(),
            element_id: None,
            classes: Vec::new(),
            styles: None,
            line_number: Some(line),
        }))
    }
    
    fn parse_button_component(ctx: &mut ParserContext) -> ParseResult<Node> {
        let line = ctx.peek().line;
        ctx.consume(&TokenType::Button, "Expected 'button'")?;
        
        let text = match &ctx.advance().token_type {
            TokenType::StringLiteral(text) => text.clone(),
            _ => return Err(ParseError {
                message: "Expected button text".to_string(),
                line: ctx.previous().line,
                column: ctx.previous().column,
            }),
        };
        
        Ok(Node::ButtonComponent(ButtonComponent {
            text,
            action: None,
            attributes: Vec::new(),
            classes: Vec::new(),
            styles: None,
            line_number: Some(line),
        }))
    }
    
    fn parse_image_component(ctx: &mut ParserContext) -> ParseResult<Node> {
        let line = ctx.peek().line;
        ctx.consume(&TokenType::Image, "Expected 'image'")?;
        
        let src = match &ctx.advance().token_type {
            TokenType::StringLiteral(src) => src.clone(),
            _ => return Err(ParseError {
                message: "Expected image source".to_string(),
                line: ctx.previous().line,
                column: ctx.previous().column,
            }),
        };
        
        Ok(Node::ImageComponent(ImageComponent {
            src,
            alt: None,
            attributes: Vec::new(),
            classes: Vec::new(),
            styles: None,
            line_number: Some(line),
        }))
    }
    
    fn parse_video_component(ctx: &mut ParserContext) -> ParseResult<Node> {
        let line = ctx.peek().line;
        ctx.consume(&TokenType::Video, "Expected 'video'")?;
        
        let src = match &ctx.advance().token_type {
            TokenType::StringLiteral(src) => src.clone(),
            _ => return Err(ParseError {
                message: "Expected video source".to_string(),
                line: ctx.previous().line,
                column: ctx.previous().column,
            }),
        };
        
        Ok(Node::VideoComponent(VideoComponent {
            src,
            controls: true,
            autoplay: false,
            loop_video: false,
            muted: false,
            attributes: Vec::new(),
            classes: Vec::new(),
            styles: None,
            line_number: Some(line),
        }))
    }
    
    fn parse_audio_component(ctx: &mut ParserContext) -> ParseResult<Node> {
        let line = ctx.peek().line;
        ctx.consume(&TokenType::Audio, "Expected 'audio'")?;
        
        let src = match &ctx.advance().token_type {
            TokenType::StringLiteral(src) => src.clone(),
            _ => return Err(ParseError {
                message: "Expected audio source".to_string(),
                line: ctx.previous().line,
                column: ctx.previous().column,
            }),
        };
        
        Ok(Node::AudioComponent(AudioComponent {
            src,
            controls: true,
            autoplay: false,
            loop_audio: false,
            attributes: Vec::new(),
            classes: Vec::new(),
            styles: None,
            line_number: Some(line),
        }))
    }
    
    fn parse_slot_component(ctx: &mut ParserContext) -> ParseResult<Node> {
        let line = ctx.peek().line;
        ctx.consume(&TokenType::Slot, "Expected 'slot'")?;
        
        let name = match &ctx.advance().token_type {
            TokenType::StringLiteral(name) => name.clone(),
            TokenType::Identifier(name) => name.clone(),
            _ => return Err(ParseError {
                message: "Expected slot name".to_string(),
                line: ctx.previous().line,
                column: ctx.previous().column,
            }),
        };
        
        Ok(Node::SlotComponent(SlotComponent {
            name,
            default_content: Vec::new(),
            attributes: Vec::new(),
            classes: Vec::new(),
            styles: None,
            line_number: Some(line),
        }))
    }
}