use reqwest::Client;

use crate::{
    model::{result::Result, task::Task, worker::Worker},
    service::worker_register::WorkerRegister,
};

pub struct WorkerDistributor {
    pub worker_register: WorkerRegister,
}

impl WorkerDistributor {
    pub fn new(worker_register: WorkerRegister) -> Self {
        WorkerDistributor { worker_register }
    }

    pub async fn distribute_task(&self, task: Task) -> Result {
        let default_worker = Worker::new(true, String::from("http://localhost:3000"));

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
