use serde::{Deserialize, Serialize};

use crate::{
    components::{input::Input, layout::Layout, proxy::Proxy},
    config::Config,
    framework::Runtime,
};

pub struct App {
    config: Config,
    mode: Mode,
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Mode {
    #[default]
    Home,
}

impl App {
    pub fn new() -> color_eyre::Result<Self> {
        Ok(Self {
            config: Config::new()?,
            mode: Mode::Home,
        })
    }

    pub async fn run(&mut self) -> color_eyre::Result<()> {
        
        
        let components: Vec<Box<dyn crate::framework::Component>> = vec![
            Box::new(Layout::default())
        ];
        
        // Create and run the runtime
        let mut runtime = Runtime::new(components, self.config.clone(), self.mode);
        runtime.run().await?;
        
        Ok(())
    }
}
