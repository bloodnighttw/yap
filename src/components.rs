use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::{
    Frame,
    layout::{Rect, Size},
};
use tokio::sync::mpsc::UnboundedSender;

use crate::{action::Action, config::Config, tui::Event};

pub mod home;
pub mod counter;
pub mod container;

/// `Component` is a trait that represents a visual and interactive element of the user interface.
///
/// Implementors of this trait can be registered with the main application loop and will be able to
/// receive events, update state, and be rendered on the screen.
/// 
/// This trait follows React-like lifecycle methods for predictable component behavior.
pub trait Component {
    /// Called once when the component is first created, before mounting.
    /// Similar to React's constructor. Use this to initialize state.
    ///
    /// # Arguments
    ///
    /// * `tx` - An unbounded sender that can send actions.
    /// * `config` - Configuration settings.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - An Ok result or an error.
    fn component_will_mount(&mut self, tx: UnboundedSender<Action>, config: Config) -> color_eyre::Result<()> {
        let _ = (tx, config); // to appease clippy
        Ok(())
    }

    /// Get mutable references to children components.
    /// Similar to React's props.children. Override this to provide children.
    ///
    /// # Returns
    ///
    /// * `Vec<&mut Box<dyn Component>>` - Mutable references to child components.
    fn children(&mut self) -> Vec<&mut Box<dyn Component>> {
        Vec::new()
    }

    /// Called immediately after the component is mounted to the DOM.
    /// Similar to React's componentDidMount. Use this for initialization that requires DOM or side effects.
    ///
    /// # Arguments
    ///
    /// * `area` - Rectangular area the component is mounted within.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - An Ok result or an error.
    fn component_did_mount(&mut self, area: Size) -> color_eyre::Result<()> {
        let _ = area; // to appease clippy
        Ok(())
    }

    /// Called immediately before the component is unmounted and destroyed.
    /// Similar to React's componentWillUnmount. Use this for cleanup.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - An Ok result or an error.
    fn component_will_unmount(&mut self) -> color_eyre::Result<()> {
        Ok(())
    }

    /// Handle incoming events and produce actions if necessary.
    ///
    /// # Arguments
    ///
    /// * `event` - An optional event to be processed.
    ///
    /// # Returns
    ///
    /// * `Result<Option<Action>>` - An action to be processed or none.
    fn handle_events(&mut self, event: Option<Event>) -> color_eyre::Result<Option<Action>> {
        let action = match event {
            Some(Event::Key(key_event)) => self.handle_key_event(key_event)?,
            Some(Event::Mouse(mouse_event)) => self.handle_mouse_event(mouse_event)?,
            _ => None,
        };
        Ok(action)
    }

    /// Handle key events and produce actions if necessary.
    ///
    /// # Arguments
    ///
    /// * `key` - A key event to be processed.
    ///
    /// # Returns
    ///
    /// * `Result<Option<Action>>` - An action to be processed or none.
    fn handle_key_event(&mut self, key: KeyEvent) -> color_eyre::Result<Option<Action>> {
        let _ = key; // to appease clippy
        Ok(None)
    }

    /// Handle mouse events and produce actions if necessary.
    ///
    /// # Arguments
    ///
    /// * `mouse` - A mouse event to be processed.
    ///
    /// # Returns
    ///
    /// * `Result<Option<Action>>` - An action to be processed or none.
    fn handle_mouse_event(&mut self, mouse: MouseEvent) -> color_eyre::Result<Option<Action>> {
        let _ = mouse; // to appease clippy
        Ok(None)
    }

    /// Render the component on the screen. (REQUIRED)
    /// Similar to React's render method.
    ///
    /// # Arguments
    ///
    /// * `frame` - A frame used for rendering.
    /// * `area` - The area in which the component should be drawn.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - An Ok result or an error.
    fn render(&mut self, frame: &mut Frame, area: Rect) -> color_eyre::Result<()>;

    /// Helper method to propagate constructor to all children.
    /// Call this in your constructor if you have children.
    fn init_children(&mut self, tx: UnboundedSender<Action>, config: Config) -> color_eyre::Result<()> {
        for child in self.children().iter_mut() {
            child.component_will_mount(tx.clone(), config.clone())?;
        }
        Ok(())
    }

    /// Helper method to propagate mount to all children.
    /// Call this in your component_did_mount if you have children.
    fn mount_children(&mut self, area: Size) -> color_eyre::Result<()> {
        for child in self.children().iter_mut() {
            child.component_did_mount(area)?;
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

    /// Helper method to propagate unmount to all children.
    /// Call this in your component_will_unmount if you have children.
    fn unmount_children(&mut self) -> color_eyre::Result<()> {
        for child in self.children().iter_mut() {
            child.component_will_unmount()?;
        }
        Ok(())
    }
}
