// ============================================================================
// React-like Component Framework
// ============================================================================

use crossterm::event::KeyEvent;
use ratatui::Frame;

/// Messages that components emit
pub trait Message: Clone + Send + 'static {}

/// External events that can be sent to components from outside
pub trait ExternalEvent: Send + 'static {}

/// React-like Component trait
pub trait Component: Send {
    type Msg: Message;
    type ExtEvent: ExternalEvent;

    /// Handle keyboard events and optionally emit messages
    fn on_key(&mut self, key: KeyEvent) -> Option<Self::Msg>;

    /// Handle external events (from MPSC channels, network, etc.)
    fn on_external_event(&mut self, event: Self::ExtEvent) -> Option<Self::Msg>;

    /// Render the component
    fn render(&self, frame: &mut Frame);

    /// Handle messages (like Redux reducers)
    fn handle_message(&mut self, msg: Self::Msg);

    /// Check if component should quit
    fn should_quit(&self) -> bool {
        false
    }
}
