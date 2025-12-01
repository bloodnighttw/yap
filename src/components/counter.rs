use ratatui::{text::Span};
use tracing::{debug, info};

use crate::{components::Component, config::Config, framework::{Action, Updater}};


pub struct Counter {
    count: i32,
    initial_count: i32,
    updater: Option<Updater>
}

impl Counter {
    pub fn new() -> Self {
        Self { 
            count: 0,
            initial_count: 0,
            updater: None,
        }
    }

    /// setState-like method to update count and trigger re-render
    fn set_count(&mut self, new_count: i32) {
        self.count = new_count;
        // Trigger re-render when state changes
        if let Some(updater) = &self.updater {
            updater.update();
        } else {
            unreachable!("Counter::set_count - action_tx is None, cannot trigger re-render, the component might not be mounted");
        }
    }
}

impl Default for Counter {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for Counter {
    fn component_will_mount(&mut self, _config: Config) -> color_eyre::Result<()> {
        info!("Counter::component_will_mount - Initializing component");
        Ok(())
    }

    fn component_did_mount(&mut self, area: ratatui::layout::Size, updater: Updater) -> color_eyre::Result<()> {
        info!("Counter::componentDidMount - Component mounted with area: {:?}", area);
        self.initial_count = self.count;
        self.updater = Some(updater);
        Ok(())
    }
    
    fn handle_key_event(&mut self, key: crossterm::event::KeyEvent) -> color_eyre::Result<Option<Action>> {
        use crossterm::event::{KeyCode, KeyModifiers};
        debug!("Counter received key: {:?}, modifiers: {:?}", key.code, key.modifiers);
        match (key.code, key.modifiers) {
            (KeyCode::Char('h'), m) if m.contains(KeyModifiers::CONTROL) => {
                self.set_count(self.count + 1);
            }
            (KeyCode::Char('e'), m) if m.contains(KeyModifiers::CONTROL) => {
                self.set_count(self.count - 1);
            }
            _ => {}
        }
        
        Ok(None)
    }
    
    fn render(&mut self, frame: &mut ratatui::Frame, area: ratatui::prelude::Rect) -> color_eyre::Result<()> {
        let format = format!("Count: {} (Ctrl+h to increment, Ctrl+e to decrement)", self.count);
        let paragraph = ratatui::widgets::Paragraph::new(Span::from(format));
        frame.render_widget(paragraph, area);
        Ok(())
    }
}