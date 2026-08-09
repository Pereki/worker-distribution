use std::collections::HashMap;

use crate::model::task_with_result::TaskWithResult;

pub struct TaskStorage {
    pub task_hashmap: HashMap<String, TaskWithResult>,
}

impl TaskStorage {
    pub fn new() -> Self {
        Self {
            task_hashmap: HashMap::new(),
        }
    }
}
