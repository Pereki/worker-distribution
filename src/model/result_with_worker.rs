use crate::model::{computing_result::ComputingResult, worker::Load};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
pub struct ResultWithLoad {
    pub result: Option<ComputingResult>,
    pub load: Load,
}

impl ResultWithLoad {
    pub fn new(result: Option<ComputingResult>, load: Load) -> Self {
        Self { result, load }
    }
}
