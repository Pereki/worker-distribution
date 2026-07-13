use std::fmt::format;

use axum::Json;
use reqwest::Client;

use crate::{
    model::{result::Result, task::Task, worker::Worker},
    service::worker_register::{self, WorkerRegister},
};

pub struct WorkerDistributer {
    pub worker_register: WorkerRegister,
}

impl WorkerDistributer {
    pub fn new(worker_register: WorkerRegister) -> Self {
        WorkerDistributer { worker_register }
    }

    pub async fn execute_task(&self, task: Task) -> Result {
        let default_worker = Worker::new(true, String::from("localhost"));

        let worker: &Worker = self
            .worker_register
            .get_available_worker()
            .unwrap_or(&default_worker);

        println!("Sending Task {} to worker {}", task.script, worker.ip);

        let client = Client::new();

        let response = client
            .post(format!("{}/api/work", worker.ip))
            .json(&task)
            .send()
            .await;

        let typed_resp: Result = response.unwrap().json().await.unwrap();

        typed_resp
    }
}
