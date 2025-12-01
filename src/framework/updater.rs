use tokio::sync::mpsc::UnboundedSender;

#[derive(Clone)]
pub struct Updater {
    tx: UnboundedSender<super::action::Action>,
}

impl Updater {
    
    pub fn new(tx: UnboundedSender<super::action::Action>) -> Self {
        Self { tx }
    }
    
    pub fn update(&self) {
        let _ = self.tx.send(super::Action::Render);
    }
}