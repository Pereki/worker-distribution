use std::collections::HashMap;

use crate::model::{
    computing_result::ComputingResult,
    task_with_result::{Status, TaskWithResult},
    worker::Worker,
};

pub struct TaskStorage {
    pub task_hashmap: HashMap<String, TaskWithResult>,
}

impl TaskStorage {
    pub fn new() -> Self {
        Self {
            task_hashmap: HashMap::new(),
        }
    }

    pub fn update_satus_of(&mut self, task_uuid: &str, status: Status) -> Result<(), ()> {
        self.task_hashmap.get_mut(task_uuid).unwrap().status = status;
        Ok(())
    }

    pub fn update_worker_of(&mut self, task_uuid: &str, worker: Worker) -> Result<(), ()> {
        self.task_hashmap.get_mut(task_uuid).unwrap().worker = Option::Some(worker);
        Ok(())
    }

    pub fn update_result_of(&mut self, task_uuid: &str, result: ComputingResult) -> Result<(), ()> {
        self.task_hashmap.get_mut(task_uuid).unwrap().result = Option::Some(result);
        Ok(())
    }
}
