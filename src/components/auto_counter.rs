use ratatui::text::Span;
use std::sync::{Arc, Mutex};

use crate::framework::Updater;

pub struct AutoCounter {
    count: Arc<Mutex<u64>>,
    updater: Option<Updater>,
    task_handle: Option<tokio::task::JoinHandle<()>>,
}

impl AutoCounter {
    pub fn new() -> Self {
        Self {
            count: Arc::new(Mutex::new(0)),
            updater: None,
            task_handle: None,
        }
    }
}


impl crate::framework::Component for AutoCounter {
    fn component_will_mount(&mut self, _config: crate::config::Config) -> color_eyre::Result<()> {
        Ok(())
    }

    fn component_did_mount(&mut self, _area: ratatui::layout::Size, updater: Updater) -> color_eyre::Result<()> {
        self.updater = Some(updater.clone());
        let updater_clone = updater.clone();
        let count_clone = self.count.clone();
        self.task_handle = Some(tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
            loop {
                interval.tick().await;
                // Increment the counter
                if let Ok(mut count) = count_clone.lock() {
                    *count += 1;
                }
                // Trigger re-render
                updater_clone.update();
            }
        }));
        Ok(())
    }
    
    fn render(&mut self, frame: &mut ratatui::Frame, area: ratatui::prelude::Rect) -> color_eyre::Result<()> {
        let count_value = self.count.lock().unwrap_or_else(|e| e.into_inner());
        let format = format!("Count: {}", *count_value);
        let paragraph = ratatui::widgets::Paragraph::new(Span::from(format));
        frame.render_widget(paragraph, area);
        Ok(())
    }
    
}

impl Drop for AutoCounter {
    fn drop(&mut self) {
        if let Some(handle) = self.task_handle.take() {
            handle.abort();
        }
    }
}