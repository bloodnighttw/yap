use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{
    components::{input::Input, proxy::Proxy, proxy_list::ProxyList},
    framework::{Children, Component},
};

pub struct Layout {
    children: Vec<Box<dyn Component>>,
}

impl Default for Layout {
    fn default() -> Self {
        // Create shared filter state
        let filter = Arc::new(RwLock::new(String::new()));
        
        // Create the proxy component and get shared logs
        let proxy = Proxy::default();
        let log = proxy.get_logs();
        
        // Create components with shared state
        let input = Input::new(filter.clone());
        let proxy_list = ProxyList::new(log, filter);

        Self {
            children: vec![
                Box::new(proxy), 
                Box::new(proxy_list),
                Box::new(input), 
            ],
        }
    }
}

impl Children for Layout {
    fn children(&mut self) -> Vec<&mut Box<dyn super::Component>> {
        self.children.iter_mut().collect()
    }
}

impl Component for Layout {
    fn component_will_mount(&mut self, config: crate::config::Config) -> color_eyre::Result<()> {
        self.children_will_mount(config)
    }

    fn component_did_mount(
        &mut self,
        area: ratatui::prelude::Size,
        updater: crate::framework::Updater,
    ) -> color_eyre::Result<()> {
        self.children_did_mount(area, updater)
    }

    fn handle_events(
        &mut self,
        event: Option<crate::tui::Event>,
    ) -> color_eyre::Result<Option<crate::framework::Action>> {
        let action = self.propagate_events(event)?;
        Ok(action.into_iter().next())
    }

    fn render(
        &mut self,
        frame: &mut ratatui::Frame,
        area: ratatui::prelude::Rect,
    ) -> color_eyre::Result<()> {
        // on top we render one line for the input
        let input_area = ratatui::prelude::Rect {
            x: area.x,
            y: 0,
            width: area.width,
            height: area.height - 1,
        };
        self.children[1].render(frame, input_area)?;

        // render proxy list on remaining area
        let proxy_area = ratatui::prelude::Rect {
            x: area.x,
            y: area.height - 1,
            width: area.width,
            height: 1,
        };

        self.children[2].render(frame, proxy_area)?;

        Ok(())
    }
}
