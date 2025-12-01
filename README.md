# yap

[![CI](https://github.com//ratatui-hello-world/workflows/CI/badge.svg)](https://github.com//ratatui-hello-world/actions)

A React-like TUI framework built with Ratatui, featuring component lifecycle, children composition, and event-driven rendering.

## Component Lifecycle

This framework implements a React-like component lifecycle with the following phases:

### Lifecycle Methods

```rust
pub trait Component {
    // 1. Component Will Mount - called once when component is created, before mounting
    //    Initialize component state, setup children
    fn component_will_mount(&mut self, config: Config) -> Result<()>
    
    // 2. Component Did Mount - called after first render, provides Updater for triggering re-renders
    fn component_did_mount(&mut self, area: Size, updater: Updater) -> Result<()>
    
    // 3. Handle Events - called when user input or system events occur
    fn handle_events(&mut self, event: Option<Event>) -> Result<Option<Action>>
    fn handle_key_event(&mut self, key: KeyEvent) -> Result<Option<Action>>
    fn handle_mouse_event(&mut self, mouse: MouseEvent) -> Result<Option<Action>>
    
    // 4. Render - called when UI needs to be drawn (REQUIRED)
    fn render(&mut self, frame: &mut Frame, area: Rect) -> Result<()>
}
```

### Component Lifecycle Flow

```mermaid
flowchart TD
    Start([Component Created]) --> Runtime[Runtime::new<br/>Create component vector]
    Runtime --> RunStart[Runtime::run starts]
    RunStart --> TuiInit[TUI Initialize & Enter]
    
    TuiInit --> WillMount[component_will_mount<br/>runtime.rs:56<br/>Pass config, initialize state]
    WillMount --> InitRender[Send Action::Render<br/>runtime.rs:60]
    InitRender --> CreateUpdater[Create Updater<br/>runtime.rs:61]
    
    CreateUpdater --> DidMount[component_did_mount<br/>runtime.rs:67<br/>Pass size & updater]
    DidMount --> EventLoop{Event Loop<br/>tokio::select!}
    
    EventLoop -->|TUI Event| ProcessEvent[process_event<br/>runtime.rs:79]
    ProcessEvent --> HandleEvent[component.handle_events<br/>runtime.rs:131]
    HandleEvent -->|Returns Action?| ActionQueue[Send to action_tx]
    
    EventLoop -->|Async Action| ActionReceived[action_rx.recv<br/>runtime.rs:91]
    
    ActionQueue --> HandleLifecycle[handle_lifecycle<br/>runtime.rs:82/94<br/>Process all pending actions]
    ActionReceived --> HandleLifecycle
    
    HandleLifecycle -->|Action::Render| RenderCall[render components<br/>runtime.rs:169/185]
    HandleLifecycle -->|Action::Resize| ResizeHandle[handle_resize<br/>runtime.rs:166]
    HandleLifecycle -->|Action::Quit| SetQuit[should_quit = true]
    HandleLifecycle -->|Action::Suspend| SetSuspend[should_suspend = true]
    
    RenderCall --> ComponentRender[component.render<br/>runtime.rs:189<br/>Draw to terminal]
    ResizeHandle --> RenderCall
    ComponentRender --> EventLoop
    
    EventLoop -->|should_quit| Cleanup[Cleanup Phase]
    Cleanup --> TuiExit[tui.exit<br/>runtime.rs:116]
    TuiExit --> End([Runtime Ends])
    
    style WillMount fill:#e1f5ff
    style DidMount fill:#e1f5ff
    style HandleEvent fill:#fff4e1
    style ComponentRender fill:#ffe1f5
    style EventLoop fill:#f0f0f0
```

### Detailed Event Processing Flow

```mermaid
sequenceDiagram
    participant U as User Input
    participant T as TUI
    participant R as Runtime
    participant C as Component
    participant A as Action Channel
    
    Note over R: Initialization Phase
    R->>C: component_will_mount(config)
    C-->>R: Ok()
    R->>A: Send Action::Render
    R->>C: component_did_mount(size, updater)
    C-->>R: Ok()
    
    Note over R,T: Event Loop (tokio::select!)
    
    U->>T: Key Press / Mouse Event
    T->>R: next_event() returns Event
    R->>R: process_event(event)
    R->>C: handle_events(Some(event))
    C->>C: handle_key_event / handle_mouse_event
    
    alt Component needs re-render
        C-->>R: Some(Action::Render)
        R->>A: Send action to channel
    else No action needed
        C-->>R: None
    end
    
    R->>R: handle_lifecycle() - drain action_rx
    
    loop For each pending action
        A-->>R: Receive action
        alt Action::Render
            R->>C: render(frame, area)
            C->>T: Draw widgets to frame
        else Action::Resize
            R->>R: handle_resize(w, h)
            R->>C: render(frame, area)
        else Action::Quit
            R->>R: should_quit = true
        end
    end
    
    Note over R: Loop continues until should_quit
```

### setState Pattern

Components trigger re-renders by sending `Action::Render` when state changes:

```rust
pub struct Counter {
    count: i32,
    action_tx: Option<UnboundedSender<Action>>,
}

impl Counter {
    fn set_count(&mut self, new_count: i32) {
        self.count = new_count;
        // Trigger re-render when state changes
        if let Some(tx) = &self.action_tx {
            let _ = tx.send(Action::Render);
        }
    }
}

impl Component for Counter {
    fn component_will_mount(&mut self, tx: UnboundedSender<Action>, _config: Config) -> Result<()> {
        self.action_tx = Some(tx); // Store for setState
        Ok(())
    }
    
    fn handle_key_event(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        match key.code {
            KeyCode::Char('h') => self.set_count(self.count + 1), // setState pattern
            KeyCode::Char('e') => self.set_count(self.count - 1),
            _ => {}
        }
        Ok(None)
    }
}
```

### Children Composition

Components can contain children, similar to React:

```rust
let mut container = Container::new("My Container");
container.with_children(vec![
    Box::new(Home::new()),
    Box::new(Counter::default()),
]);
```

Children receive lifecycle events automatically through helper methods:
- `init_children()` - Initialize children in component_will_mount
- `mount_children()` - Mount children in component_did_mount
- `propagate_events()` - Pass events to children in handle_events
- `unmount_children()` - Cleanup children in component_will_unmount

### Event-Driven Rendering

**No tick rate or frame rate** - rendering only occurs when:
1. Initial mount (`app.rs:66`)
2. Component calls `action_tx.send(Action::Render)` (setState pattern)
3. Window resize (`app.rs:156`)
4. Resume from suspend (`app.rs:87`)

This makes the framework efficient - the UI only updates when state actually changes.
