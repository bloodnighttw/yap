use ratatui::{text::Span};
use tracing::{debug, info};

use crate::components::Component;


#[derive(Debug, Clone, PartialEq)]
pub struct Counter {
    count: i32,
    initial_count: i32,
}

impl Counter {
    pub fn new() -> Self {
        Self { 
            count: 0,
            initial_count: 0,
        }
    }
}

impl Default for Counter {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for Counter {
    fn component_did_mount(&mut self, area: ratatui::layout::Size) -> color_eyre::Result<()> {
        info!("Counter::componentDidMount - Component mounted with area: {:?}", area);
        self.initial_count = self.count;
        Ok(())
    }

    fn component_will_unmount(&mut self) -> color_eyre::Result<()> {
        info!("Counter::componentWillUnmount - Final count: {}, changed by: {}", 
              self.count, self.count - self.initial_count);
        Ok(())
    }
    
    fn handle_key_event(&mut self, key: crossterm::event::KeyEvent) -> color_eyre::Result<Option<crate::action::Action>> {
        use crossterm::event::{KeyCode, KeyModifiers};
        debug!("Counter received key: {:?}, modifiers: {:?}", key.code, key.modifiers);
        match (key.code, key.modifiers) {
            (KeyCode::Char('h'), m) if m.contains(KeyModifiers::CONTROL) => {
                self.count += 1;
            }
            (KeyCode::Char('e'), m) if m.contains(KeyModifiers::CONTROL) => {
                self.count -= 1;
            }
            _ => {}
        }
        
        Ok(None)
    }
    
    fn render(&mut self, frame: &mut ratatui::Frame, area: ratatui::prelude::Rect) -> color_eyre::Result<()> {
        let format = format!("Count: {}", self.count);
        let paragraph = ratatui::widgets::Paragraph::new(Span::from(format));
        frame.render_widget(paragraph, area);
        Ok(())
    }
}