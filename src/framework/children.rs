use ratatui::layout::Size;

use super::{action::Action, components::Component};
use crate::{config::Config, framework::Updater, tui::Event};

/// `Children` trait provides React-like children functionality for components.
///
/// This trait allows components to contain and manage child components,
/// similar to how React components can have children through props.children.
pub trait Children {
    /// Get mutable references to children components.
    /// Similar to React's props.children. Override this to provide children.
    ///
    /// # Returns
    ///
    /// * `Vec<&mut Box<dyn Component>>` - Mutable references to child components.
    fn children(&mut self) -> Vec<&mut Box<dyn Component>> {
        Vec::new()
    }

    /// Helper method to propagate constructor to all children.
    /// Call this in your component_will_mount if you have children.
    fn children_will_mount(&mut self, config: Config) -> color_eyre::Result<()> {
        for child in self.children().iter_mut() {
            child.component_will_mount(config.clone())?;
        }
        Ok(())
    }

    /// Helper method to propagate mount to all children.
    /// Call this in your component_did_mount if you have children.
    fn children_did_mount(&mut self, area: Size, updater: Updater) -> color_eyre::Result<()> {
        for child in self.children().iter_mut() {
            child.component_did_mount(area, updater.clone())?;
        }
        Ok(())
    }

    /// Helper method to propagate events to all children.
    /// Call this in your handle_events if you want children to receive events.
    fn propagate_events(&mut self, event: Option<Event>) -> color_eyre::Result<Vec<Action>> {
        let mut actions = Vec::new();
        for child in self.children().iter_mut() {
            if let Some(action) = child.handle_events(event.clone())? {
                actions.push(action);
            }
        }
        Ok(actions)
    }

}
