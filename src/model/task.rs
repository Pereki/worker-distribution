use crate::model::languages::Language;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
pub struct Task {
    pub code: String,
    pub lang: Language,
}

impl Task {
    pub fn new(code: String, lang: Language) -> Self {
        Task { code, lang }
    }
}
