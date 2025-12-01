use ratatui::{prelude::*, widgets::*};
use tracing::info;

use super::Component;
use crate::{config::Config, framework::{Updater}};

#[derive(Default)]
pub struct Home {
    updater: Option<Updater>,
    config: Config,
}

impl Component for Home {
    fn component_will_mount(&mut self, config: Config) -> color_eyre::Result<()> {
        info!("Home::component_will_mount - Initializing component");
        self.config = config;
        Ok(())
    }

    fn component_did_mount(&mut self, _area: ratatui::layout::Size, updater: Updater) -> color_eyre::Result<()> {
        self.updater = Some(updater);
        Ok(())
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) -> color_eyre::Result<()> {
        frame.render_widget(Paragraph::new("hello world"), area);
        Ok(())
    }
}
