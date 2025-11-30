use ratatui::{text::Span};
use tokio::sync::mpsc::UnboundedSender;
use tracing::{debug, info};

use crate::{framework::Action, components::Component, config::Config};


#[derive(Debug)]
pub struct Counter {
    count: i32,
    initial_count: i32,
    action_tx: Option<UnboundedSender<Action>>,
}

impl Counter {
    pub fn new() -> Self {
        Self { 
            count: 0,
            initial_count: 0,
            action_tx: None,
        }
    }

    /// setState-like method to update count and trigger re-render
    fn set_count(&mut self, new_count: i32) {
        self.count = new_count;
        // Trigger re-render when state changes
        if let Some(tx) = &self.action_tx {
            let _ = tx.send(Action::Render);
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
    fn component_will_mount(&mut self, tx: UnboundedSender<Action>, _config: Config) -> color_eyre::Result<()> {
        info!("Counter::component_will_mount - Initializing component");
        self.action_tx = Some(tx);
        Ok(())
    }

    fn component_did_mount(&mut self, area: ratatui::layout::Size) -> color_eyre::Result<()> {
        info!("Counter::componentDidMount - Component mounted with area: {:?}", area);
        self.initial_count = self.count;
        Ok(())
    }

    fn component_will_unmount(&mut self) -> color_eyre::Result<()> {
        info!("Counter::componentWillUnmount - Final count: {}, changed by: {}", 
              self.count, self.count - self.initial_count);
        self.action_tx = None;
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