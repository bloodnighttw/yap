use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::{
    Frame,
    layout::{Rect, Size},
};
use tokio::sync::mpsc::UnboundedSender;
use tracing::debug;

use crate::{action::Action, config::Config, tui::Event};

pub mod fps;
pub mod home;
pub mod counter;

/// A wrapper that provides automatic cleanup via Drop trait.
/// Similar to React's component lifecycle, this ensures componentWillUnmount is called.
pub struct ComponentWrapper<T: Component> {
    component: T,
    mounted: bool,
}

impl<T: Component> ComponentWrapper<T> {
    pub fn new(component: T) -> Self {
        Self {
            component,
            mounted: false,
        }
    }

    pub fn mount(&mut self, area: Size) -> color_eyre::Result<()> {
        if !self.mounted {
            self.component.component_did_mount(area)?;
            self.mounted = true;
            debug!("Component mounted");
        }
        Ok(())
    }

    pub fn get_mut(&mut self) -> &mut T {
        &mut self.component
    }

    pub fn is_mounted(&self) -> bool {
        self.mounted
    }
}

impl<T: Component> Drop for ComponentWrapper<T> {
    fn drop(&mut self) {
        if self.mounted {
            debug!("Component unmounting, running cleanup");
            if let Err(e) = self.component.component_will_unmount() {
                eprintln!("Error during component cleanup: {}", e);
            }
        }
    }
}

impl<T: Component> std::ops::Deref for ComponentWrapper<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.component
    }
}

impl<T: Component> std::ops::DerefMut for ComponentWrapper<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.component
    }
}

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
    fn constructor(&mut self, tx: UnboundedSender<Action>, config: Config) -> color_eyre::Result<()> {
        let _ = (tx, config); // to appease clippy
        Ok(())
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

    /// Called before the component is updated with new props or state.
    /// Similar to React's shouldComponentUpdate. Return false to prevent update.
    ///
    /// # Arguments
    ///
    /// * `action` - An action that may trigger a component update.
    ///
    /// # Returns
    ///
    /// * `bool` - Whether the component should update.
    fn should_component_update(&mut self, action: &Action) -> bool {
        let _ = action; // to appease clippy
        true
    }

    /// Called to update component state based on an action.
    /// Similar to React's componentDidUpdate / getDerivedStateFromProps combined.
    ///
    /// # Arguments
    ///
    /// * `action` - An action that may modify the state of the component.
    ///
    /// # Returns
    ///
    /// * `Result<Option<Action>>` - An action to be processed or none.
    fn component_did_update(&mut self, action: Action) -> color_eyre::Result<Option<Action>> {
        let _ = action; // to appease clippy
        Ok(None)
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
}
