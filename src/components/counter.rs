use crate::framework::{Component, ExternalEvent, Message};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{widgets::Paragraph, Frame};

#[derive(Clone)]
pub enum CounterMsg {
    Increment,
    Decrement,
    Quit,
}

impl Message for CounterMsg {}

// Counter has no external events
pub enum NoExtEvent {}
impl ExternalEvent for NoExtEvent {}

pub struct Counter {
    should_quit: bool,
    count: i32,
}

impl Counter {
    pub fn new() -> Self {
        Self {
            should_quit: false,
            count: 0,
        }
    }

    pub fn count(&self) -> i32 {
        self.count
    }
}

impl Component for Counter {
    type Msg = CounterMsg;
    type ExtEvent = NoExtEvent;

    fn on_key(&mut self, key: KeyEvent) -> Option<Self::Msg> {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => Some(CounterMsg::Quit),
            KeyCode::Char('i') => Some(CounterMsg::Increment),
            KeyCode::Char('d') => Some(CounterMsg::Decrement),
            _ => None,
        }
    }

    fn on_external_event(&mut self, _event: Self::ExtEvent) -> Option<Self::Msg> {
        None // Counter doesn't handle external events
    }

    fn render(&self, frame: &mut Frame) {
        let text = format!(
            "Counter: {}\n\nPress 'i' to increment\nPress 'd' to decrement\nPress 'q' or ESC to quit",
            self.count
        );
        let paragraph = Paragraph::new(text);
        frame.render_widget(paragraph, frame.area());
    }

    fn handle_message(&mut self, msg: Self::Msg) {
        match msg {
            CounterMsg::Increment => {
                self.count += 1;
            }
            CounterMsg::Decrement => {
                self.count -= 1;
            }
            CounterMsg::Quit => {
                self.should_quit = true;
            }
        }
    }

    fn should_quit(&self) -> bool {
        self.should_quit
    }
}
