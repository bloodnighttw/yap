use serde::{Deserialize, Serialize};

use crate::{
    components::{auto_counter::AutoCounter, container::Container, counter::Counter, home::Home},
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
        // Demonstrate children pattern: wrap Home and Counter in a Container
        let mut main_container = Container::new("Main Container");
        main_container.with_children(vec![
            Box::new(Home::new()),
            Box::new(Counter::default()),
            Box::new(AutoCounter::new())
        ]);
        
        let components: Vec<Box<dyn crate::framework::Component>> = vec![
            Box::new(main_container),
        ];
        
        // Create and run the runtime
        let mut runtime = Runtime::new(components, self.config.clone(), self.mode);
        runtime.run().await?;
        
        Ok(())
    }
}
