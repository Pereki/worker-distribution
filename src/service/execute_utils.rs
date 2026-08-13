use base64::Engine;

use std::error::Error;
use std::io;
use std::path::PathBuf;
use std::process::{Command, Output};

use tokio::fs::{self};
use tokio::time::Instant;
use uuid::Uuid;

use crate::model::computing_result::ComputingResult;
use crate::model::languages::Language::{JAVA, SHELL};
use crate::model::task::Task;

const MAIN_JAVA_FILE_NAME: &str = "Main.java";
const MAIN_CLASS_FILE_NAME: &str = "Main.class";
const MAIN_CLASS_NAME: &str = "Main";
const JAVA_COMMAND: &str = "java";
const JAVAC_COMMAND: &str = "javac";
const CLASSPATH_ARGUMENT: &str = "-cp";
const DIRECTORY_ARGUMENT: &str = "-d";
const SHELL_COMMAND: &str = "sh";
const INLINE_SCRIPT_ARGUMENT: &str = "-c";

pub async fn exec(task: Task) -> Option<ComputingResult> {
    println!("Executing task {}", task.code);
    match task.lang {
        JAVA => compile_java(&task).await,
        SHELL => compile_shell(&task),
    }
}

fn compile_shell(task: &Task) -> Option<ComputingResult> {
    let start = Instant::now();
    let output = match Command::new(SHELL_COMMAND)
        .arg(INLINE_SCRIPT_ARGUMENT)
        .arg(task.code.clone())
        .output()
    {
        Ok(x) => x,
        Err(er) => {
            eprintln!("Error running shellscript: {}", er);
            return None;
        }
    };

    match generate_output(output, start.elapsed().as_millis(), None) {
        Err(_) => {
            return None;
        }
        Ok(x) => {
            return Some(x);
        }
    }
}

async fn create_dir_for_java_task(task: &Task, uuid: &str) -> Result<PathBuf, Box<dyn Error>> {
    let file_path = std::env::temp_dir().join(uuid);
    fs::create_dir_all(&file_path).await?;
    fs::write(&file_path.join(MAIN_JAVA_FILE_NAME), &task.code).await?;
    Ok(file_path)
}

fn run_java_compiler(file_path: &PathBuf) -> io::Result<Output> {
    Command::new(JAVAC_COMMAND)
        .args([DIRECTORY_ARGUMENT, file_path.to_str().unwrap()])
        .arg(&file_path.join(MAIN_JAVA_FILE_NAME))
        .output()
}

fn run_java(file_path: &PathBuf) -> io::Result<Output> {
    Command::new(JAVA_COMMAND)
        .args([CLASSPATH_ARGUMENT, &file_path.to_str().unwrap()])
        .arg(MAIN_CLASS_NAME)
        .output()
}

fn generate_output(
    output: Output,
    elapsed_time_in_millis: u128,
    bytecode: Option<String>,
) -> Result<ComputingResult, Box<dyn Error>> {
    let stdtout = match String::from_utf8(output.stdout) {
        Ok(x) => x,
        Err(er) => {
            eprintln!("Error getting output of shell task: {}", er);
            return Err(er.into());
        }
    };

    let stderr = match String::from_utf8(output.stderr) {
        Ok(x) => x,
        Err(er) => {
            eprintln!("Error getting output of shell task: {}", er);
            return Err(er.into());
        }
    };

    return Ok(ComputingResult::new(
        stdtout,
        stderr,
        elapsed_time_in_millis,
        bytecode,
    ));
}

async fn compile_java(task: &Task) -> Option<ComputingResult> {
    let start = Instant::now();
    let uuid = Uuid::new_v4();

    let file_path = match create_dir_for_java_task(&task, &uuid.to_string()).await {
        Ok(file) => file,
        Err(er) => {
            eprintln!("Could not create dir for java: {}", er);
            return None;
        }
    };

    let javac_command = match run_java_compiler(&file_path) {
        Ok(x) => x,
        Err(e) => {
            eprintln!("Error running java compiler: {}", e);
            return None;
        }
    };

    if !javac_command.status.success() {
        return generate_output(javac_command, start.elapsed().as_millis(), None).ok();
    }

    let java_run = match run_java(&file_path) {
        Ok(x) => x,
        Err(er) => {
            eprintln!("Could not run java: {}", er);
            return None;
        }
    };

    let optional_bytecode = match &file_path.join(MAIN_CLASS_FILE_NAME).to_str() {
        None => None,
        Some(x) => match std::fs::read(x) {
            Err(er) => {
                eprintln!("Could not read bytecode: {}", er);
                None
            }
            Ok(y) => Some(base64::engine::general_purpose::STANDARD.encode(y)),
        },
    };

    return generate_output(java_run, start.elapsed().as_millis(), optional_bytecode).ok();
}
