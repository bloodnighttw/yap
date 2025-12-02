use ratatui::{prelude::*, widgets::*};
use tracing::info;

use super::{Children, Component};
use crate::{config::Config, framework::{Action, Updater}, tui::Event};

/// A container component that can hold and render children.
/// Similar to a <div> in React that wraps other components.
pub struct Container {
    updater: Option<Updater>,
    config: Config,
    children: Vec<Box<dyn Component>>,
    title: String,
    border_style: Style,
}

#[allow(dead_code)]
impl Container {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            updater: None,
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

impl Children for Container {
    fn children(&mut self) -> Vec<&mut Box<dyn Component>> {
        self.children.iter_mut().collect()
    }
}

impl Component for Container {
    fn component_will_mount(&mut self, config: Config) -> color_eyre::Result<()> {
        info!("Container::component_will_mount - Initializing container '{}'", self.title);
        self.config = config.clone();
        
        // Initialize all children
        self.children_will_mount(config)?;
        Ok(())
    }

    fn component_did_mount(&mut self, area: ratatui::layout::Size, updater: Updater) -> color_eyre::Result<()> {
        info!("Container::componentDidMount - Container '{}' mounted with area: {:?}", self.title, area);
        self.updater = Some(updater.clone());
        // Mount all children
        self.children_did_mount(area, updater)?;
        Ok(())
    }

    fn handle_events(&mut self, event: Option<Event>) -> color_eyre::Result<Option<Action>> {
        // Propagate events to children
        let actions = self.propagate_events(event)?;
        
        // Return first action if any
        Ok(actions.into_iter().next())
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
