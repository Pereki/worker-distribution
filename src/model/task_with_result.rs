use crate::model::{result::Result, task::Task};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
pub struct TaskWithResult {
    pub uuid: String,
    pub task: Task,
    pub result: Option<Result>,
}

impl TaskWithResult {
    pub fn new(task: Task, result: Option<Result>, uuid: String) -> Self {
        Self { task, result, uuid }
    }
}
