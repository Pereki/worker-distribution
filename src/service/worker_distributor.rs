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
        task_storage
            .lock()
            .await
            .update_worker_of(&task.uuid, worker.clone());

        task_storage.lock().await.update_satus_of(
            &task.uuid,
            crate::model::task_with_result::Status::COMPUTING,
        );
        println!("Sending Task {} to worker {}", task.task.code, worker.ip);

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
            .update_result_of(&task.uuid, response.unwrap().json().await.unwrap());

        task_storage
            .lock()
            .await
            .update_satus_of(&task.uuid, crate::model::task_with_result::Status::FINISHED);

        worker_register.lock().await.unlock_worker(&worker.ip);
    }
}
