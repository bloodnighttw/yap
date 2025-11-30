use ratatui::{prelude::*, widgets::*};
use tokio::sync::mpsc::UnboundedSender;
use tracing::info;

use super::Component;
use crate::{action::Action, config::Config, tui::Event};

/// A container component that can hold and render children.
/// Similar to a <div> in React that wraps other components.
pub struct Container {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    children: Vec<Box<dyn Component>>,
    title: String,
    border_style: Style,
}

impl Container {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            command_tx: None,
            config: Config::default(),
            children: Vec::new(),
            title: title.into(),
            border_style: Style::default().fg(Color::Cyan),
        }
    }

    /// Set multiple children at once
    pub fn with_children(&mut self, children: Vec<Box<dyn Component>>) -> &mut Self {
        self.children = children;
        self
    }
}

impl Component for Container {
    fn component_will_mount(&mut self, tx: UnboundedSender<Action>, config: Config) -> color_eyre::Result<()> {
        info!("Container::constructor - Initializing container '{}'", self.title);
        self.command_tx = Some(tx.clone());
        self.config = config.clone();
        
        // Initialize all children
        self.init_children(tx, config)?;
        Ok(())
    }

    fn component_did_mount(&mut self, area: ratatui::layout::Size) -> color_eyre::Result<()> {
        info!("Container::componentDidMount - Container '{}' mounted with area: {:?}", self.title, area);
        
        // Mount all children
        self.mount_children(area)?;
        Ok(())
    }

    fn children(&mut self) -> Vec<&mut Box<dyn Component>> {
        self.children.iter_mut().collect()
    }

    fn handle_events(&mut self, event: Option<Event>) -> color_eyre::Result<Option<Action>> {
        // Propagate events to children
        let actions = self.propagate_events(event)?;
        
        // Return first action if any
        Ok(actions.into_iter().next())
    }

    fn component_will_unmount(&mut self) -> color_eyre::Result<()> {
        info!("Container::componentWillUnmount - Cleaning up container '{}'", self.title);
        
        // Unmount all children first
        self.unmount_children()?;
        
        // Drop the sender to signal shutdown
        self.command_tx = None;
        Ok(())
    }
    

    fn render(&mut self, frame: &mut Frame, area: Rect) -> color_eyre::Result<()> {
        // Create a block with border for the container
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(self.border_style)
            .title(self.title.as_str());
        
        // Calculate inner area for children
        let inner_area = block.inner(area);
        
        // Render the container border
        frame.render_widget(block, area);
        
        // Render each child on its own line
        let mut y_offset = 0;
        for child in self.children.iter_mut() {
            if y_offset < inner_area.height {
                let child_area = Rect {
                    x: inner_area.x,
                    y: inner_area.y + y_offset,
                    width: inner_area.width,
                    height: 1,
                };
                child.render(frame, child_area)?;
                y_offset += 1;
            }
        }
        
        Ok(())
    }
}

impl Drop for Container {
    fn drop(&mut self) {
        info!("Container::drop - Container '{}' being dropped", self.title);
    }
}
