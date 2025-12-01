use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::{
    Frame,
    layout::{Rect, Size},
};

use super::action::Action;
use crate::{config::Config, framework::Updater, tui::Event};

/// `Component` is a trait that represents a visual and interactive element of the user interface.
///
/// Implementors of this trait can be registered with the main application loop and will be able to
/// receive events, update state, and be rendered on the screen.
/// 
/// This trait follows React-like lifecycle methods for predictable component behavior.
pub trait Component {
    /// Called once when the component is first created, before mounting.
    /// you should initialize any component here, because this is called only once.
    /// and you will need to progregate the component lifecycle to any children here as well.
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration settings.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - An Ok result or an error.
    fn component_will_mount(&mut self,config: Config) -> color_eyre::Result<()> {
        let _ = config; // to appease clippy
        Ok(())
    }

    /// Called immediately after the component is mounted to the DOM.
    /// Similar to React's componentDidMount. Use this for initialization that requires DOM or side effects.
    ///
    /// # Arguments
    ///
    /// * `area` - Rectangular area the component is mounted within.
    /// * `updater` - An updater that can be used to trigger re-renders, the component can only be
    /// update after mounting, so this is provided here.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - An Ok result or an error.
    fn component_did_mount(&mut self, area: Size, updater: Updater) -> color_eyre::Result<()> {
        let _ = (area, updater); // to appease clippy
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
}
