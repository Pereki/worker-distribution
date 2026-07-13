use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub enum Language {
    SHELL,
}
