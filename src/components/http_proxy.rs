use crate::framework::{Component, ExternalEvent, Message};
use chrono::Local;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use tokio::sync::mpsc;

#[derive(Clone)]
pub struct HttpRequest {
    pub method: String,
    pub uri: String,
    pub timestamp: String,
}

impl ExternalEvent for HttpRequest {}

#[derive(Clone)]
pub enum ProxyMsg {
    NewRequest,
    Clear,
    Quit,
}

impl Message for ProxyMsg {}

pub struct HttpProxy {
    requests: Vec<HttpRequest>,
    should_quit: bool,
    max_requests: usize,
}

impl HttpProxy {
    pub fn new() -> Self {
        Self {
            requests: Vec::new(),
            should_quit: false,
            max_requests: 100,
        }
    }
}

impl Component for HttpProxy {
    type Msg = ProxyMsg;
    type ExtEvent = HttpRequest;

    fn on_key(&mut self, key: KeyEvent) -> Option<Self::Msg> {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => Some(ProxyMsg::Quit),
            KeyCode::Char('c') => Some(ProxyMsg::Clear),
            _ => None,
        }
    }

    fn on_external_event(&mut self, event: Self::ExtEvent) -> Option<Self::Msg> {
        // Add new HTTP request to the list
        self.requests.push(event);
        
        // Keep only last N requests
        if self.requests.len() > self.max_requests {
            self.requests.remove(0);
        }
        
        Some(ProxyMsg::NewRequest)
    }

    fn render(&self, frame: &mut Frame) {
        let area = frame.area();

        // Title
        let title = Paragraph::new(vec![
            Line::from(vec![
                Span::styled("HTTP Proxy Monitor", Style::default().fg(Color::Cyan)),
            ]),
            Line::from(vec![
                Span::raw("Listening on http://127.0.0.1:8080 | "),
                Span::styled("Press 'c' to clear", Style::default().fg(Color::Yellow)),
                Span::raw(" | "),
                Span::styled("Press 'q' to quit", Style::default().fg(Color::Red)),
            ]),
        ]);

        let title_area = Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: 3,
        };

        frame.render_widget(title, title_area);

        // Request list
        let list_items: Vec<ListItem> = self
            .requests
            .iter()
            .rev() // Show newest first
            .map(|req| {
                let content = format!(
                    "[{}] {} {}",
                    req.timestamp, req.method, req.uri
                );
                
                let color = match req.method.as_str() {
                    "GET" => Color::Green,
                    "POST" => Color::Blue,
                    "PUT" => Color::Yellow,
                    "DELETE" => Color::Red,
                    _ => Color::White,
                };
                
                ListItem::new(content).style(Style::default().fg(color))
            })
            .collect();

        let list = List::new(list_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("Requests ({})", self.requests.len()))
                    .border_style(Style::default().fg(Color::Cyan)),
            );

        let list_area = Rect {
            x: area.x,
            y: area.y + 3,
            width: area.width,
            height: area.height - 3,
        };

        frame.render_widget(list, list_area);
    }

    fn handle_message(&mut self, msg: Self::Msg) {
        match msg {
            ProxyMsg::NewRequest => {
                // Request already added in on_external_event
            }
            ProxyMsg::Clear => {
                self.requests.clear();
            }
            ProxyMsg::Quit => {
                self.should_quit = true;
            }
        }
    }

    fn should_quit(&self) -> bool {
        self.should_quit
    }
}
