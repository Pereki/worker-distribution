use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Result {
    pub result: String,
}

impl Result {
    pub fn new(result: String) -> Self {
        Result { result }
    }
}
