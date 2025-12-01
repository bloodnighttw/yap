use std::fmt::Display;

use tokio::sync::mpsc::UnboundedSender;

#[derive(Clone, Debug)]
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

impl Display for Updater {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Updater")
    }
}

impl PartialEq for Updater {
    fn eq(&self, other: &Self) -> bool {
        self.tx.same_channel(&other.tx)
    }
}

impl Eq for Updater {}