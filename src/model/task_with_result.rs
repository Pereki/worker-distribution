use crate::model::{computing_result::ComputingResult, task::Task, worker::Worker};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
pub struct TaskWithResult {
    pub uuid: String,
    pub task: Task,
    pub result: Option<ComputingResult>,
    pub status: Status,
    pub worker: Option<Worker>,
}

#[derive(Deserialize, Serialize, Clone)]
pub enum Status {
    REQUESTED,
    QUEUED,
    COMPUTING,
    FINISHED,
}

impl TaskWithResult {
    pub fn new(
        task: Task,
        result: Option<ComputingResult>,
        uuid: String,
        status: Status,
        worker: Option<Worker>,
    ) -> Self {
        Self {
            task,
            result,
            uuid,
            status,
            worker,
        }
    }
}
