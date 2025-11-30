use ratatui::{prelude::*, widgets::*};
use tokio::sync::mpsc::UnboundedSender;
use tracing::info;

use super::Component;
use crate::{action::Action, config::Config};

#[derive(Default)]
pub struct Home {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
}

impl Home {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Component for Home {
    fn component_will_mount(&mut self, tx: UnboundedSender<Action>, config: Config) -> color_eyre::Result<()> {
        info!("Home::constructor - Initializing component");
        self.command_tx = Some(tx);
        self.config = config;
        Ok(())
    }

    fn component_did_mount(&mut self, area: ratatui::layout::Size) -> color_eyre::Result<()> {
        info!("Home::componentDidMount - Component mounted with area: {:?}", area);
        Ok(())
    }

    fn component_will_unmount(&mut self) -> color_eyre::Result<()> {
        info!("Home::componentWillUnmount - Cleaning up resources");
        // Drop the sender to signal shutdown
        self.command_tx = None;
        Ok(())
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) -> color_eyre::Result<()> {
        frame.render_widget(Paragraph::new("hello world"), area);
        Ok(())
    }
}

impl Drop for Home {
    fn drop(&mut self) {
        info!("Home::drop - Component being dropped");
    }
}
