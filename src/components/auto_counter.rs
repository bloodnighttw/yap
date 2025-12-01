use ratatui::text::Span;
use tokio::time::sleep;
use std::{sync::{Arc, atomic::{AtomicU64, Ordering}}, time::Duration};

use crate::framework::Updater;

#[derive(Default)]
pub struct AutoCounter {
    count: Arc<AtomicU64>,
    updater: Option<Updater>,
    task_handle: Option<tokio::task::JoinHandle<()>>,
}

impl crate::framework::Component for AutoCounter {

    fn component_did_mount(&mut self, _area: ratatui::layout::Size, updater: Updater) -> color_eyre::Result<()> {
        self.updater = Some(updater.clone());
        let updater_clone = updater.clone();
        let count_clone = self.count.clone();
        self.task_handle = Some(tokio::spawn(async move {
            loop {
                sleep(Duration::from_secs(1)).await;
                
                // Increment the counter
                count_clone.fetch_add(1, Ordering::Relaxed);
                
                // Trigger re-render
                updater_clone.update();
            }
        }));
        Ok(())
    }
    
    fn render(&mut self, frame: &mut ratatui::Frame, area: ratatui::prelude::Rect) -> color_eyre::Result<()> {
        let count_value = self.count.load(Ordering::Relaxed);
        let format = format!("Count: {}", count_value);
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