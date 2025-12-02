use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Debug, Clone, PartialEq, Eq, Display, Serialize, Deserialize)]
pub enum Action {
    Render,
    Resize(u16, u16),
    Suspend,
    Resume,
    Quit,
    Error(String),
}
