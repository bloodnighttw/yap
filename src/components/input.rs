use color_eyre::eyre::Ok;

use crate::framework::{Action, Component, Updater};

#[derive(Clone, Default, Debug)]
pub struct Input {
    hostname: String,
    cursor_position: usize,
    updater: Option<Updater>,
}

impl Component for Input {
    fn component_did_mount(
        &mut self,
        _area: ratatui::prelude::Size,
        updater: Updater,
    ) -> color_eyre::Result<()> {
        self.updater = Some(updater);
        Ok(())
    }

    fn render(
        &mut self,
        frame: &mut ratatui::Frame,
        area: ratatui::prelude::Rect,
    ) -> color_eyre::Result<()> {
        // Draw the input text
        let input = ratatui::widgets::Paragraph::new(self.hostname.as_str());
        frame.render_widget(input, area);

        // Set the native cursor position
        frame.set_cursor_position((area.x + self.cursor_position as u16, area.y));

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
                    self.cursor_position += c.len_utf8();
                }
                crossterm::event::KeyCode::Backspace => {
                    if self.cursor_position > 0 {
                        // Find the previous character boundary
                        let mut new_pos = self.cursor_position - 1;
                        while new_pos > 0 && !self.hostname.is_char_boundary(new_pos) {
                            new_pos -= 1;
                        }
                        self.hostname.remove(new_pos);
                        self.cursor_position = new_pos;
                    }
                }
                crossterm::event::KeyCode::Left => {
                    if self.cursor_position > 0 {
                        // Move to previous character boundary
                        let mut new_pos = self.cursor_position.saturating_sub(1);
                        while new_pos > 0 && !self.hostname.is_char_boundary(new_pos) {
                            new_pos -= 1;
                        }
                        self.cursor_position = new_pos;
                    }
                }
                crossterm::event::KeyCode::Right => {
                    if self.cursor_position < self.hostname.len() {
                        // Move to next character boundary
                        let mut new_pos = self.cursor_position + 1;
                        while new_pos < self.hostname.len() && !self.hostname.is_char_boundary(new_pos) {
                            new_pos += 1;
                        }
                        self.cursor_position = new_pos.min(self.hostname.len());
                    }
                }
                crossterm::event::KeyCode::Home => {
                    self.cursor_position = 0;
                }
                crossterm::event::KeyCode::End => {
                    self.cursor_position = self.hostname.len();
                }
                crossterm::event::KeyCode::Delete => {
                    if self.cursor_position < self.hostname.len() {
                        self.hostname.remove(self.cursor_position);
                    }
                }
                _ => {}
            }
        }
        Ok(Action::Render.into())
    }
}
