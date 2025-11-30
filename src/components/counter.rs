use ratatui::{text::Span};

use crate::components::Component;


#[derive(Debug, Clone, PartialEq)]
pub struct Counter {
    count: i32,
}

impl Counter {
    pub fn new() -> Self {
        Self { count: 0 }
    }
}

impl Default for Counter {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for Counter {
    
    fn handle_key_event(&mut self, key: crossterm::event::KeyEvent) -> color_eyre::Result<Option<crate::action::Action>> {
        // if we press i, increase the valuem, when press e, decress
        match key.code {
            crossterm::event::KeyCode::Char('i') => {
                self.count += 1;
            }
            crossterm::event::KeyCode::Char('e') => {
                self.count -= 1;
            }
            _ => {}
        }
        
        Ok(None)
    }
    
    fn draw(&mut self, frame: &mut ratatui::Frame, area: ratatui::prelude::Rect) -> color_eyre::Result<()> {
        let format = format!("Count: {}", self.count);
        let paragraph = ratatui::widgets::Paragraph::new(Span::from(format));
        frame.render_widget(paragraph, area);
        Ok(())
    }
}