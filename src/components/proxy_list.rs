use ratatui::{prelude::*, widgets::*};
use tracing::info;

use super::Component;
use super::proxy::SharedLogs;
use crate::{config::Config, framework::Updater};

pub struct ProxyList {
    logs: SharedLogs,
    _updater: Option<Updater>,
}

impl ProxyList {
    pub fn new(logs: SharedLogs) -> Self {
        Self {
            logs,
            _updater: None,
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
        self._updater = Some(updater);
        Ok(())
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

        // Create the list widget
        let list = List::new(items)
            .block(
                Block::default()
                    .title("HTTP Proxy Log (Last 100 requests on port 9999)")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .style(Style::default().fg(Color::White));

        frame.render_widget(list, area);
        Ok(())
    }
}
