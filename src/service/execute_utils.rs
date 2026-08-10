use base64::Engine;

use std::process::Command;

use tokio::fs::{self, DirBuilder};
use tokio::time::Instant;
use uuid::Uuid;

use crate::model::computing_result::ComputingResult;
use crate::model::languages::Language::SHELL;
use crate::model::task::Task;

pub async fn exec(task: Task) -> Option<ComputingResult> {
    println!("Executing task {}", task.code);
    let start = Instant::now();
    if matches!(task.lang, SHELL) {
        let output = Command::new("sh")
            .arg("-c")
            .arg(task.code)
            .output()
            .expect("Failed to execute command");

        return Some(ComputingResult::new(
            String::from_utf8(output.stdout).unwrap(),
            String::from_utf8(output.stderr).unwrap(),
            start.elapsed().as_millis(),
            Option::None,
        ));
    }

    let uuid = Uuid::new_v4();

    let file_path = std::env::temp_dir().join(uuid.to_string());
    fs::create_dir_all(&file_path).await.unwrap();
    fs::write(&file_path.join("Main.java"), task.code).await;

    let javac_command = Command::new("javac")
        .args(["-d", file_path.to_str().unwrap()])
        .arg(&file_path.join("Main.java"))
        .output()
        .unwrap();

    if (!javac_command.status.success()) {
        return Some(ComputingResult::new(
            String::from_utf8(javac_command.stdout).unwrap(),
            String::from_utf8(javac_command.stderr).unwrap(),
            start.elapsed().as_millis(),
            Option::None,
        ));
    }

    let java_run = Command::new("java")
        .args(["-cp", &file_path.to_str().unwrap()])
        .arg("Main")
        .output()
        .unwrap();

    Some(ComputingResult::new(
        String::from_utf8(java_run.stdout).unwrap(),
        String::from_utf8(java_run.stderr).unwrap(),
        start.elapsed().as_millis(),
        Option::Some(
            base64::engine::general_purpose::STANDARD
                .encode(std::fs::read(&file_path.join("Main.class").to_str().unwrap()).unwrap()),
        ),
    ))
}
