use crossterm::event::KeyEvent;
use ratatui::prelude::Rect;
use tokio::sync::mpsc;
use tracing::{debug, info};

use super::{action::Action, components::Component};
use crate::{
    app::Mode,
    config::Config,
    tui::{Event, Tui},
};

/// Runtime manages the execution of components and handles the application lifecycle.
///
/// This is similar to the React runtime that manages the component tree and handles
/// the lifecycle events, event processing, and rendering.
pub struct Runtime {
    components: Vec<Box<dyn Component>>,
    should_quit: bool,
    should_suspend: bool,
    action_tx: mpsc::UnboundedSender<Action>,
    action_rx: mpsc::UnboundedReceiver<Action>,
    config: Config,
    mode: Mode,
}

impl Runtime {
    /// Create a new Runtime with the given components and configuration.
    pub fn new(components: Vec<Box<dyn Component>>, config: Config, mode: Mode) -> Self {
        let (action_tx, action_rx) = mpsc::unbounded_channel();
        
        Self {
            components,
            should_quit: false,
            should_suspend: false,
            action_tx,
            action_rx,
            config,
            mode,
        }
    }

    /// Run the runtime loop.
    ///
    /// This method handles the full lifecycle:
    /// 1. Initialize TUI
    /// 2. Mount components (component_will_mount, component_did_mount)
    /// 3. Run event loop (handle events, process actions, render)
    /// 4. Unmount components (component_will_unmount)
    /// 5. Cleanup TUI
    pub async fn run(&mut self) -> color_eyre::Result<()> {
        let mut tui = Tui::new()?;
        tui.enter()?;

        // React-like lifecycle: constructor phase
        info!("Initializing components (constructor phase)");
        for component in self.components.iter_mut() {
            component.component_will_mount(self.action_tx.clone(), self.config.clone())?;
        }
        
        // Initial render
        self.action_tx.send(Action::Render)?;

        // React-like lifecycle: componentDidMount phase
        info!("Mounting components (componentDidMount phase)");
        let size = tui.size()?;
        for component in self.components.iter_mut() {
            component.component_did_mount(size)?;
        }

        let action_tx = self.action_tx.clone();
        loop {
            self.handle_events(&mut tui).await?;
            self.handle_lifecycle(&mut tui)?;
            
            if self.should_suspend {
                tui.suspend()?;
                action_tx.send(Action::Resume)?;
                action_tx.send(Action::ClearScreen)?;
                tui.enter()?;
                // Trigger render after resume
                action_tx.send(Action::Render)?;
            } else if self.should_quit {
                tui.stop()?;
                break;
            }
        }
        
        // React-like lifecycle: componentWillUnmount phase
        info!("Unmounting components (componentWillUnmount phase)");
        for component in self.components.iter_mut() {
            component.component_will_unmount()?;
        }
        
        tui.exit()?;
        Ok(())
    }

    async fn handle_events(&mut self, tui: &mut Tui) -> color_eyre::Result<()> {
        let Some(event) = tui.next_event().await else {
            return Ok(());
        };
        let action_tx = self.action_tx.clone();
        match event {
            Event::Quit => action_tx.send(Action::Quit)?,
            Event::Resize(x, y) => action_tx.send(Action::Resize(x, y))?,
            Event::Key(key) => self.handle_key_event(key)?,
            _ => {}
        }
        for component in self.components.iter_mut() {
            if let Some(action) = component.handle_events(Some(event.clone()))? {
                action_tx.send(action)?;
            }
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> color_eyre::Result<()> {
        let action_tx = self.action_tx.clone();
        let Some(keymap) = self.config.keybindings.get(&self.mode) else {
            return Ok(());
        };
        if let Some(action) = keymap.get(&vec![key]) {
            info!("Got action: {action:?}");
            action_tx.send(action.clone())?;
        }
        Ok(())
    }

    fn handle_lifecycle(&mut self, tui: &mut Tui) -> color_eyre::Result<()> {
        while let Ok(action) = self.action_rx.try_recv() {
            if action != Action::Render {
                debug!("{action:?}");
            }
            
            match action {
                Action::Quit => self.should_quit = true,
                Action::Suspend => self.should_suspend = true,
                Action::Resume => self.should_suspend = false,
                Action::ClearScreen => tui.terminal.clear()?,
                Action::Resize(w, h) => self.handle_resize(tui, w, h)?,
                Action::Render => self.render(tui)?,
                _ => {}
            }
        }
        Ok(())
    }

    fn handle_resize(&mut self, tui: &mut Tui, w: u16, h: u16) -> color_eyre::Result<()> {
        tui.resize(Rect::new(0, 0, w, h))?;
        self.render(tui)?;
        Ok(())
    }

    fn render(&mut self, tui: &mut Tui) -> color_eyre::Result<()> {
        tui.draw(|frame| {
            for component in self.components.iter_mut() {
                // React-like lifecycle: render method
                if let Err(err) = component.render(frame, frame.area()) {
                    let _ = self
                        .action_tx
                        .send(Action::Error(format!("Failed to render: {:?}", err)));
                }
            }
        })?;
        Ok(())
    }

}
