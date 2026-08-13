use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
pub struct ComputingResult {
    pub stdout: String,
    pub stderr: String,
    pub duration: u128,
    pub bytecode: Option<String>,
}

impl ComputingResult {
    pub fn new(stdout: String, stderr: String, duration: u128, bytecode: Option<String>) -> Self {
        ComputingResult {
            stdout,
            stderr,
            duration,
            bytecode,
        }
    }
}
