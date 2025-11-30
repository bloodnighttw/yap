use crate::components::counter::{Counter, CounterMsg};
use crate::components::input::{Input, InputMsg};
use crate::framework::{Component, ExternalEvent, Message};
use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::Paragraph,
    Frame,
};

#[derive(Clone)]
pub enum AppMsg {
    CounterMsg(CounterMsg),
    InputMsg(InputMsg),
    Quit,
}

impl Message for AppMsg {}

// App has no external events
pub enum NoExtEvent {}
impl ExternalEvent for NoExtEvent {}

pub struct App {
    counter: Counter,
    input: Input,
    should_quit: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            counter: Counter::new(),
            input: Input::new("Type something here..."),
            should_quit: false,
        }
    }
}

impl Component for App {
    type Msg = AppMsg;
    type ExtEvent = NoExtEvent;

    fn on_key(&mut self, key: KeyEvent) -> Option<Self::Msg> {
        // If input is focused, only input gets the keys (except Tab/ESC)
        if self.input.is_focused() {
            if let Some(input_msg) = self.input.on_key(key) {
                match input_msg {
                    InputMsg::Quit => return Some(AppMsg::Quit),
                    other => return Some(AppMsg::InputMsg(other)),
                }
            }
        } else {
            // Input not focused, forward to both components
            // Input first (for Tab to focus)
            if let Some(input_msg) = self.input.on_key(key) {
                match input_msg {
                    InputMsg::Quit => return Some(AppMsg::Quit),
                    other => return Some(AppMsg::InputMsg(other)),
                }
            }
            
            // Then counter
            if let Some(counter_msg) = self.counter.on_key(key) {
                match counter_msg {
                    CounterMsg::Quit => return Some(AppMsg::Quit),
                    other => return Some(AppMsg::CounterMsg(other)),
                }
            }
        }

        None
    }

    fn on_external_event(&mut self, _event: Self::ExtEvent) -> Option<Self::Msg> {
        None // App doesn't handle external events
    }

    fn render(&self, frame: &mut Frame) {
        // Split screen into two sections
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(50),  // Top half for counter
                Constraint::Percentage(50),  // Bottom half for input
            ])
            .split(frame.area());

        // Render counter in top half
        self.render_counter(frame, chunks[0]);

        // Render input in bottom half
        self.render_input(frame, chunks[1]);
    }

    fn handle_message(&mut self, msg: Self::Msg) {
        match msg {
            AppMsg::CounterMsg(counter_msg) => {
                self.counter.handle_message(counter_msg);
            }
            AppMsg::InputMsg(input_msg) => {
                self.input.handle_message(input_msg);
            }
            AppMsg::Quit => {
                self.should_quit = true;
            }
        }
    }

    fn should_quit(&self) -> bool {
        self.should_quit || self.counter.should_quit() || self.input.should_quit()
    }
}

impl App {
    fn render_counter(&self, frame: &mut Frame, area: Rect) {
        // Inline counter rendering logic
        let text = format!(
            "=== COUNTER (Top Half) ===\n\nCounter: {}\n\nPress 'i' to increment\nPress 'd' to decrement",
            self.counter.count()
        );
        let paragraph = Paragraph::new(text);
        frame.render_widget(paragraph, area);
    }

    fn render_input(&self, frame: &mut Frame, area: Rect) {
        // Inline input rendering logic
        use ratatui::style::{Color, Modifier, Style};
        use ratatui::widgets::{Block, Borders};

        let input_area = Rect {
            x: area.x + area.width / 4,
            y: area.y + 2,
            width: area.width / 2,
            height: 3,
        };

        let border_style = if self.input.is_focused() {
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .title(if self.input.is_focused() {
                "[FOCUSED] Input (Bottom Half)"
            } else {
                "[Press Tab to focus] Input (Bottom Half)"
            });

        let display_value = self.input.value();
        let text = if self.input.is_focused() {
            format!("{}|", display_value)
        } else {
            display_value.to_string()
        };

        let paragraph = Paragraph::new(text).block(block);
        frame.render_widget(paragraph, input_area);

        // Instructions
        let instructions = Paragraph::new(
            "Counter: i/d to change | Input: Tab to focus, type when focused | ESC: Quit"
        )
        .style(Style::default().fg(Color::Yellow));

        let instruction_area = Rect {
            x: area.x,
            y: area.y + area.height - 1,
            width: area.width,
            height: 1,
        };

        frame.render_widget(instructions, instruction_area);
    }
}
