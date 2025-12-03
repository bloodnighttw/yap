use color_eyre::eyre::Ok;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use std::time::{Duration, Instant};

use crate::framework::{Action, Component, Updater};

#[derive(Clone, Debug)]
pub struct Filter {
    hostname: String,
    cursor_position: usize,
    cursor_visible: bool,
    last_blink: Instant,
    updater: Option<Updater>,
}

impl Default for Filter {
    fn default() -> Self {
        Self {
            hostname: String::default(),
            cursor_position: 0,
            cursor_visible: true,
            last_blink: Instant::now(),
            updater: None,
        }
    }
}

impl Component for Filter {
    
    fn component_did_mount(&mut self, _area: ratatui::prelude::Size, updater: Updater) -> color_eyre::Result<()> {
        self.updater = Some(updater.clone());
        
        // Spawn a background task for cursor blinking
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(530));
            loop {
                interval.tick().await;
                updater.update();
            }
        });
        
        Ok(())
    }
    
    fn render(
        &mut self,
        frame: &mut ratatui::Frame,
        area: ratatui::prelude::Rect,
    ) -> color_eyre::Result<()> {
        // Update cursor blink state
        const BLINK_INTERVAL: Duration = Duration::from_millis(530);
        if self.last_blink.elapsed() >= BLINK_INTERVAL {
            self.cursor_visible = !self.cursor_visible;
            self.last_blink = Instant::now();
        }
        
        // Build the text with block cursor
        let mut spans = Vec::new();
        
        if self.hostname.is_empty() {
            // Empty input - show block cursor as a space with background
            if self.cursor_visible {
                spans.push(Span::styled(
                    " ",
                    Style::default().bg(Color::White).fg(Color::Black),
                ));
            } else {
                spans.push(Span::raw(" "));
            }
        } else {
            let chars: Vec<char> = self.hostname.chars().collect();
            
            // Text before cursor
            if self.cursor_position > 0 {
                let before: String = chars[..self.cursor_position].iter().collect();
                spans.push(Span::raw(before));
            }
            
            // Character at cursor position (or space if at end)
            if self.cursor_visible {
                let cursor_char = if self.cursor_position < chars.len() {
                    chars[self.cursor_position].to_string()
                } else {
                    " ".to_string()
                };
                
                spans.push(Span::styled(
                    cursor_char,
                    Style::default().bg(Color::White).fg(Color::Black),
                ));
            } else {
                // When cursor is hidden, still show the character normally
                if self.cursor_position < chars.len() {
                    spans.push(Span::raw(chars[self.cursor_position].to_string()));
                } else {
                    spans.push(Span::raw(" "));
                }
            }
            
            // Text after cursor
            if self.cursor_position + 1 < chars.len() {
                let after: String = chars[self.cursor_position + 1..].iter().collect();
                spans.push(Span::raw(after));
            }
        }

        let line = Line::from(spans);
        let input = ratatui::widgets::Paragraph::new(line);

        frame.render_widget(input, area);

        Ok(())
    }

    fn handle_key_event(
        &mut self,
        key: crossterm::event::KeyEvent,
    ) -> color_eyre::Result<Option<crate::framework::Action>> {
        // when push any key without modifier, add the character to the hostname
        // When push backspace, remove the last character from the hostname
        if key.modifiers.is_empty() {
            match key.code {
                crossterm::event::KeyCode::Char(c) => {
                    self.hostname.insert(self.cursor_position, c);
                    self.cursor_position += 1;
                    self.reset_cursor_blink();
                }
                crossterm::event::KeyCode::Backspace => {
                    if self.cursor_position > 0 {
                        self.hostname.remove(self.cursor_position - 1);
                        self.cursor_position -= 1;
                        self.reset_cursor_blink();
                    }
                }
                crossterm::event::KeyCode::Left => {
                    if self.cursor_position > 0 {
                        self.cursor_position -= 1;
                        self.reset_cursor_blink();
                    }
                }
                crossterm::event::KeyCode::Right => {
                    if self.cursor_position < self.hostname.len() {
                        self.cursor_position += 1;
                        self.reset_cursor_blink();
                    }
                }
                crossterm::event::KeyCode::Home => {
                    self.cursor_position = 0;
                    self.reset_cursor_blink();
                }
                crossterm::event::KeyCode::End => {
                    self.cursor_position = self.hostname.len();
                    self.reset_cursor_blink();
                }
                _ => {}
            }
        }
        Ok(Action::Render.into())
    }
}

impl Filter {
    fn reset_cursor_blink(&mut self) {
        self.cursor_visible = true;
        self.last_blink = Instant::now();
    }
}
