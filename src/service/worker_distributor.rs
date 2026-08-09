use std::sync::Arc;

use tokio::sync::{Mutex, mpsc::Receiver, watch::Receiver as WatchReceiver};

use reqwest::Client;

use crate::{
    model::{task_with_result::TaskWithResult, worker::Worker},
    service::{task_storage::TaskStorage, worker_register::WorkerRegister},
};

pub async fn distribute_task(
    worker_register: Arc<Mutex<WorkerRegister>>,
    mut worker_reeceiver: WatchReceiver<Vec<Worker>>,
    mut task_channel: Receiver<TaskWithResult>,
    task_storage: Arc<Mutex<TaskStorage>>,
) {
    while let Some(task) = task_channel.recv().await {
        let worker = loop {
            if let Some(worker) = worker_reeceiver
                .borrow()
                .iter()
                .find(|w| w.is_available)
                .cloned()
            {
                break worker;
            }
            worker_reeceiver.changed().await;
        };

        println!("Sending Task {} to worker {}", task.task.script, worker.ip);

        worker_register.lock().await.lock_worker(&worker.ip);
        let client = Client::new();

        let response = client
            .post(format!("{}/api/work", worker.ip))
            .json(&task.task)
            .send()
            .await;

        task_storage
            .lock()
            .await
            .task_hashmap
            .get_mut(&task.uuid)
            .unwrap()
            .result = response.unwrap().json().await.unwrap();

        worker_register.lock().await.unlock_worker(&worker.ip);
    }
}
