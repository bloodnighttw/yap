mod framework;
mod proxy_server;
mod components {
    pub mod counter;
    pub mod input;
    pub mod app;
    pub mod http_proxy;
}

use color_eyre::eyre::Result;
use components::http_proxy::HttpProxy;
use crossterm::event::{Event as CrosstermEvent, EventStream, KeyEvent};
use framework::Component;
use futures::StreamExt;
use ratatui::DefaultTerminal;
use tokio::sync::mpsc;

// ============================================================================
// Event-Driven Runtime
// ============================================================================

pub enum RuntimeEvent<E> {
    Key(KeyEvent),
    External(E),
}

pub struct Runtime<C: Component> {
    component: C,
    event_rx: mpsc::UnboundedReceiver<RuntimeEvent<C::ExtEvent>>,
}

impl<C: Component> Runtime<C> {
    pub fn new(component: C) -> (Self, mpsc::UnboundedSender<RuntimeEvent<C::ExtEvent>>) {
        let (event_tx, event_rx) = mpsc::unbounded_channel();

        (
            Self {
                component,
                event_rx,
            },
            event_tx,
        )
    }

    /// Start event-driven runtime (NO LOOP!)
    pub async fn run(mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        // Wait for events and react to them
        while let Some(event) = self.event_rx.recv().await {
            let msg = match event {
                RuntimeEvent::Key(key) => self.component.on_key(key),
                RuntimeEvent::External(ext) => self.component.on_external_event(ext),
            };

            // Handle emitted message
            if let Some(msg) = msg {
                self.component.handle_message(msg);
            }

            // Render after each event
            terminal.draw(|frame| self.component.render(frame))?;

            // Check if should quit
            if self.component.should_quit() {
                break;
            }
        }

        Ok(())
    }
}

/// Spawn keyboard event producer
pub fn spawn_keyboard_events<E: Send + 'static>(tx: mpsc::UnboundedSender<RuntimeEvent<E>>) {
    tokio::spawn(async move {
        let mut event_stream = EventStream::new();

        while let Some(event) = event_stream.next().await {
            if let Ok(CrosstermEvent::Key(key)) = event {
                if tx.send(RuntimeEvent::Key(key)).is_err() {
                    break;
                }
            }
        }
    });
}

// ============================================================================
// Main Entry Point
// ============================================================================

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let mut terminal = ratatui::init();
    let result = run(&mut terminal).await;
    ratatui::restore();
    result
}

async fn run(terminal: &mut DefaultTerminal) -> Result<()> {
    // Create channels for HTTP requests
    let (http_req_tx, mut http_req_rx) = tokio::sync::mpsc::unbounded_channel();

    // Create HTTP Proxy component
    let http_proxy = HttpProxy::new();
    let (runtime, event_tx) = Runtime::new(http_proxy);

    // Spawn keyboard event producer
    spawn_keyboard_events(event_tx.clone());

    // Forward HTTP requests as external events
    let event_tx_clone = event_tx.clone();
    tokio::spawn(async move {
        while let Some(http_req) = http_req_rx.recv().await {
            if event_tx_clone.send(RuntimeEvent::External(http_req)).is_err() {
                break;
            }
        }
    });

    // Start HTTP proxy server
    tokio::spawn(proxy_server::start_proxy_server(http_req_tx));

    // Run the event-driven runtime (NO LOOP!)
    runtime.run(terminal).await
}
