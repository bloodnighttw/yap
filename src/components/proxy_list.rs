use ratatui::{prelude::*, widgets::*};
use tracing::info;
use crossterm::event::{KeyCode, KeyEvent};

use super::Component;
use super::proxy::SharedLogs;
use crate::{config::Config, framework::{Updater, Action}};

pub struct ProxyList {
    logs: SharedLogs,
    updater: Option<Updater>,
    scroll_state: ScrollbarState,
    scroll_offset: usize,
    items_len: usize,
}

impl ProxyList {
    pub fn new(logs: SharedLogs) -> Self {
        Self {
            logs,
            updater: None,
            scroll_state: ScrollbarState::default(),
            scroll_offset: 0,
            items_len: 0,
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
        match key.code {
            KeyCode::Down => {
                // Scroll down
                if self.scroll_offset < self.items_len.saturating_sub(1) {
                    self.scroll_offset = self.scroll_offset.saturating_add(1);
                    self.scroll_state = self.scroll_state.position(self.scroll_offset);
                    
                    // Trigger re-render
                    if let Some(updater) = &self.updater {
                        updater.update();
                    }
                }
                Ok(None)
            }
            KeyCode::Up => {
                // Scroll up
                if self.scroll_offset > 0 {
                    self.scroll_offset = self.scroll_offset.saturating_sub(1);
                    self.scroll_state = self.scroll_state.position(self.scroll_offset);
                    
                    // Trigger re-render
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
        // Try to read logs non-blocking and clone the data
        let logs_snapshot = if let Ok(logs) = self.logs.try_read() {
            logs.iter().cloned().collect::<Vec<_>>()
        } else {
            vec![]
        };
        
        // Create list items from logs snapshot
        let items: Vec<Line> = if logs_snapshot.is_empty() {
            vec![Line::from(Span::styled(
                "Waiting for requests...",
                Style::default().fg(Color::Gray),
            ))]
        } else {
            logs_snapshot
                .iter()
                .map(|log| {
                    let time = log.timestamp.format("%H:%M:%S");
                    Line::from(vec![
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
                    ])
                })
                .collect()
        };

        self.items_len = items.len();
        
        // Update scroll state based on content length
        self.scroll_state = self.scroll_state.content_length(self.items_len.saturating_sub(1));
        
        // Create the list widget with stateful rendering
        let list = List::new(items)
            .block(
                Block::default()
                    .title("HTTP Proxy Log (Last 100 requests on port 9999)")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .style(Style::default().fg(Color::White))
            .scroll_padding(1);

        // Create a stateful list to support scrolling
        let mut list_state = ListState::default().with_offset(self.scroll_offset);
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
        
        Ok(())
    }
}
