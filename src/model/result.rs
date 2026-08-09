use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
pub struct Result {
    pub result: String,
}

impl Result {
    pub fn new(result: String) -> Self {
        Result { result }
    }
}
