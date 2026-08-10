use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
pub enum Language {
    SHELL,
    JAVA,
}
