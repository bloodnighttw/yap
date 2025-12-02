use color_eyre::eyre::Ok;
use crossterm::event::KeyEvent;
use ratatui::prelude::Rect;
use tokio::sync::mpsc;
use tracing::{debug, info};

use super::{action::Action, components::Component};
use crate::{
    app::Mode,
    config::Config,
    framework::Updater,
    tui::{Event, Tui},
};

/// Runtime manages the execution of components and handles the application lifecycle.
///
/// This is similar to the React runtime that manages the component tree and handles
/// the lifecycle events, event processing, and rendering.
pub struct Runtime {
    components: Vec<Box<dyn Component>>,
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
    /// 5. Cleanup TUI
    pub async fn run(&mut self) -> color_eyre::Result<()> {
        let mut tui = Tui::new()?;
        tui.enter()?;

        info!("Initializing components (constructor phase)");
        for component in self.components.iter_mut() {
            component.component_will_mount(self.config.clone())?;
        }

        // Initial render
        self.action_tx.send(Action::Render)?;
        let updater = Updater::new(self.action_tx.clone());

        info!("Mounting components (componentDidMount phase)");
        let size = tui.size()?;
        for component in self.components.iter_mut() {
            component.component_did_mount(size, updater.clone())?;
        }

        // a tickless event loop
        loop {
            let stop = tokio::select! {
                // Wait for input events from TUI
                Some(event) = tui.next_event() => {
                    // Handle the event
                    self.process_event(event)?;
                    Ok(false)
                }

                // Also check for actions that may come from async tasks
                Some(action) = self.action_rx.recv() => {
                    // Put the action back and process all pending actions
                    let stop = self.batch_actions(&mut tui, action)?;
                    Ok(stop)
                }
            }?;

            tracing::info!("Event loop");

            if stop {
                break;
            }
        }

        tui.exit()?;
        Ok(())
    }

    fn process_event(&mut self, event: Event) -> color_eyre::Result<()> {
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

    // if the batch result is to stopped rendering and exit, return true
    fn batch_actions(&mut self, tui: &mut Tui, action: Action) -> color_eyre::Result<bool> {
        let mut resize: Option<(u16, u16)> = match action {
            Action::Resize(w, h) => Some((w, h)),
            _ => None,
        };
        let mut need_render = action == Action::Render;
        let quit = action == Action::Quit;
        let mut suspend = action == Action::Suspend;
        let mut resume = action == Action::Resume;

        while let Result::Ok(action) = self.action_rx.try_recv() {
            if action != Action::Render {
                debug!("{action:?}");
            }

            match action {
                Action::Quit => {
                    return Ok(true);
                }
                Action::Suspend => {
                    suspend = true;
                }
                Action::Resume => {
                    resume = true;
                }
                Action::Resize(w, h) => {
                    resize = Some((w, h));
                }
                Action::Render => {
                    // Render action is explicit, so render immediately
                    need_render = true;
                }
                _ => {}
            }
        }

        if quit {
            return Ok(true);
        }

        if let Some((w, h)) = resize {
            self.handle_resize(tui, w, h)?;
        }

        if suspend {
            tui.suspend()?;
            // wait until resume
            self.action_tx.send(Action::Resume)?;
        }

        if resume {
            tui.resume()?;
            tui.clear()?;
            self.render(tui)?;
        }

        if need_render {
            self.render(tui)?;
        }

        Ok(false)
    }

    fn handle_resize(&mut self, tui: &mut Tui, w: u16, h: u16) -> color_eyre::Result<()> {
        tui.resize(Rect::new(0, 0, w, h))?;
        self.render(tui)?;
        Ok(())
    }

    fn render(&mut self, tui: &mut Tui) -> color_eyre::Result<()> {
        tui.draw(|frame| {
            for component in self.components.iter_mut() {
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
