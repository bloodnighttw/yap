use serde::{Deserialize, Serialize};

use crate::{
    components::{proxy::Proxy, input::Input},
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
        // Create the proxy component and get shared logs
        let proxy = Proxy::default();
        let filter = Input::default();
        
        
        let components: Vec<Box<dyn crate::framework::Component>> = vec![
            Box::new(proxy),
            Box::new(filter),
        ];
        
        // Create and run the runtime
        let mut runtime = Runtime::new(components, self.config.clone(), self.mode);
        runtime.run().await?;
        
        Ok(())
    }
}
