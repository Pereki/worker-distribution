use crate::model::languages::Language;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
pub struct Task {
    pub script: String,
    pub lang: Language,
}

impl Task {
    pub fn new(script: String, lang: Language) -> Self {
        Task { script, lang }
    }
}
