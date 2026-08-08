use std::process::Command;

use crate::model::result::Result;
use crate::model::task::Task;

pub struct ExecuteUtils {}

impl ExecuteUtils {
    pub fn new() -> ExecuteUtils {
        ExecuteUtils {}
    }

    pub fn exec(task: Task) -> Option<Result> {
        let output = Command::new("sh")
            .arg("-c")
            .arg(task.script)
            .output()
            .expect("Failed to execute command");
        let str = String::from_utf8(output.stdout).unwrap();

        Some(Result::new(str))
    }
}
