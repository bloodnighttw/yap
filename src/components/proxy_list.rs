use ratatui::{prelude::*, widgets::*};
use tracing::info;
use crossterm::event::{KeyCode, KeyEvent};

use super::Component;
use super::proxy::{SharedLogs, Proxy};
use crate::{config::Config, framework::{Updater, Action}};

pub struct ProxyList {
    logs: SharedLogs,
    updater: Option<Updater>,
    scroll_state: ScrollbarState,
    scroll_offset: usize,
    selected_index: usize,
    items_len: usize,
    show_popup: bool,
    visible_height: usize,
}

impl ProxyList {
    pub fn new(logs: SharedLogs) -> Self {
        Self {
            logs,
            updater: None,
            scroll_state: ScrollbarState::default(),
            scroll_offset: 0,
            selected_index: 0,
            items_len: 0,
            show_popup: false,
            visible_height: 10,
        }
    }


}

impl Component for ProxyList {
    fn component_will_mount(&mut self, _config: Config) -> color_eyre::Result<()> {
        info!("ProxyList::component_will_mount - Initializing component");
        Ok(())
    }

    fn component_did_mount(
        &mut self,
        _area: ratatui::layout::Size,
        updater: Updater,
    ) -> color_eyre::Result<()> {
        info!("ProxyList::component_did_mount");
        self.updater = Some(updater);
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> color_eyre::Result<Option<Action>> {
        if self.show_popup {
            // Handle popup keys
            match key.code {
                KeyCode::Esc | KeyCode::Char('q') => {
                    self.show_popup = false;
                    if let Some(updater) = &self.updater {
                        updater.update();
                    }
                }
                _ => {}
            }
            return Ok(None);
        }

        match key.code {
            KeyCode::Down | KeyCode::Char('j') => {
                // Move selection down
                if self.selected_index < self.items_len.saturating_sub(1) {
                    self.selected_index = self.selected_index.saturating_add(1);
                    
                    // Update scroll if needed - keep selection in visible area
                    let max_visible = self.scroll_offset + self.visible_height.saturating_sub(1);
                    if self.selected_index > max_visible {
                        self.scroll_offset = self.selected_index.saturating_sub(self.visible_height.saturating_sub(1));
                    }
                    
                    self.scroll_state = self.scroll_state.position(self.scroll_offset);
                    
                    // Trigger re-render
                    if let Some(updater) = &self.updater {
                        updater.update();
                    }
                }
                Ok(None)
            }
            KeyCode::Up | KeyCode::Char('k') => {
                // Move selection up
                if self.selected_index > 0 {
                    self.selected_index = self.selected_index.saturating_sub(1);
                    
                    // Update scroll if needed
                    if self.selected_index < self.scroll_offset {
                        self.scroll_offset = self.selected_index;
                    }
                    
                    self.scroll_state = self.scroll_state.position(self.scroll_offset);
                    
                    // Trigger re-render
                    if let Some(updater) = &self.updater {
                        updater.update();
                    }
                }
                Ok(None)
            }
            KeyCode::Enter => {
                // Open popup for selected item
                let logs_snapshot = if let Ok(logs) = self.logs.try_read() {
                    logs.iter().cloned().collect::<Vec<_>>()
                } else {
                    vec![]
                };

                if self.selected_index < logs_snapshot.len() {
                    // Show popup - content will be loaded during render
                    self.show_popup = true;
                    
                    if let Some(updater) = &self.updater {
                        updater.update();
                    }
                }
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    fn render(
        &mut self,
        frame: &mut ratatui::Frame,
        area: ratatui::prelude::Rect,
    ) -> color_eyre::Result<()> {
        // Update visible height based on area (subtract 2 for borders)
        self.visible_height = area.height.saturating_sub(2) as usize;
        
        // Try to read logs non-blocking and clone the data
        let logs_snapshot = if let Ok(logs) = self.logs.try_read() {
            logs.iter().cloned().collect::<Vec<_>>()
        } else {
            vec![]
        };
        
        // Create list items from logs snapshot
        let items: Vec<ListItem> = if logs_snapshot.is_empty() {
            vec![ListItem::new(Line::from(Span::styled(
                "Waiting for requests...",
                Style::default().fg(Color::Gray),
            )))]
        } else {
            logs_snapshot
                .iter()
                .enumerate()
                .map(|(idx, log)| {
                    let time = log.timestamp.format("%H:%M:%S");
                    let line = Line::from(vec![
                        Span::styled(
                            format!("[{}] ", time),
                            Style::default().fg(Color::Gray),
                        ),
                        Span::styled(
                            format!("{:8} ", log.method),
                            Style::default().fg(match log.method.as_str() {
                                "GET" => Color::Green,
                                "POST" => Color::Blue,
                                "CONNECT" => Color::Magenta,
                                _ => Color::Yellow,
                            }),
                        ),
                        Span::raw(&log.uri),
                    ]);
                    
                    let style = if idx == self.selected_index {
                        Style::default().bg(Color::DarkGray)
                    } else {
                        Style::default()
                    };
                    
                    ListItem::new(line).style(style)
                })
                .collect()
        };

        let old_items_len = self.items_len;
        self.items_len = items.len();
        
        // Auto-scroll to bottom if user was at the bottom and new items were added
        let was_at_bottom = old_items_len > 0 && self.selected_index == old_items_len.saturating_sub(1);
        if was_at_bottom && self.items_len > old_items_len {
            self.selected_index = self.items_len.saturating_sub(1);
            // Update scroll to keep selection visible
            if self.items_len > self.visible_height {
                self.scroll_offset = self.items_len.saturating_sub(self.visible_height);
            }
        } else {
            // If not at bottom, just ensure selected_index is within bounds
            if self.selected_index >= self.items_len && self.items_len > 0 {
                self.selected_index = self.items_len.saturating_sub(1);
            }
        }
        
        // Update scroll state based on content length
        self.scroll_state = self.scroll_state.content_length(self.items_len.saturating_sub(1));
        
        // Create the list widget with stateful rendering
        let list = List::new(items)
            .block(
                Block::default()
                    .title("HTTP Proxy Log (↑/↓ navigate, Enter to view, ESC/q to close)")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .style(Style::default().fg(Color::White))
            .scroll_padding(1);

        // Create a stateful list to support scrolling
        let mut list_state = ListState::default()
            .with_selected(Some(self.selected_index))
            .with_offset(self.scroll_offset);
        frame.render_stateful_widget(list, area, &mut list_state);
        
        // Render scrollbar
        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));
        
        frame.render_stateful_widget(
            scrollbar,
            area.inner(Margin {
                vertical: 1,
                horizontal: 0,
            }),
            &mut self.scroll_state,
        );
        
        // Render popup if needed
        if self.show_popup {
            self.render_popup(frame, area, &logs_snapshot)?;
        }
        
        Ok(())
    }
}

impl ProxyList {
    fn render_popup(
        &mut self,
        frame: &mut ratatui::Frame,
        area: ratatui::prelude::Rect,
        logs_snapshot: &[super::proxy::HttpLog],
    ) -> color_eyre::Result<()> {
        // Create a centered popup
        let popup_area = centered_rect(90, 90, area);
        
        // Load file content synchronously for rendering
        let (status, url, body) = if self.selected_index < logs_snapshot.len() {
            let log = &logs_snapshot[self.selected_index];
            let file_path = Proxy::uri_to_file_path(&log.uri);
            
            match std::fs::read_to_string(&file_path) {
                Ok(content) => {
                    let mut status = String::from("Unknown");
                    let mut body = String::new();
                    let mut in_body = false;
                    
                    for line in content.lines() {
                        if line.starts_with("Status:") {
                            status = line.trim_start_matches("Status:").trim().to_string();
                        } else if line.starts_with("Response Body:") {
                            in_body = true;
                        } else if in_body {
                            body.push_str(line);
                            body.push('\n');
                        }
                    }
                    
                    (status, log.uri.clone(), body.trim().to_string())
                }
                Err(e) => (
                    "Error".to_string(),
                    log.uri.clone(),
                    format!("Failed to load file: {}", e),
                ),
            }
        } else {
            ("Unknown".to_string(), "".to_string(), "".to_string())
        };
        
        // Create popup content
        let popup_block = Block::default()
            .title(format!("Response - Status: {} | {}", status, url))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow));
        
        let text = Paragraph::new(body)
            .block(popup_block)
            .wrap(Wrap { trim: false })
            .scroll((0, 0));
        
        // Clear the area and render popup
        frame.render_widget(Clear, popup_area);
        frame.render_widget(text, popup_area);
        
        Ok(())
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
