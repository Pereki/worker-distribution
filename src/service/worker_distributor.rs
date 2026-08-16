use std::sync::Arc;

use tokio::sync::{Mutex, mpsc::Receiver, watch::Receiver as WatchReceiver};

use reqwest::Client;

use crate::{
    model::{
        computing_result::ComputingResult,
        result_with_worker::ResultWithLoad,
        task::Task,
        task_with_result::{Status, TaskWithResult},
        worker::{Load, Worker},
    },
    service::{task_storage::TaskStorage, worker_register::WorkerRegister},
};

pub async fn distribute_task(
    worker_register: Arc<Mutex<WorkerRegister>>,
    worker_reeceiver: WatchReceiver<Vec<Worker>>,
    mut task_channel: Receiver<TaskWithResult>,
    task_storage: Arc<Mutex<TaskStorage>>,
) {
    while let Some(task) = task_channel.recv().await {
        tokio::spawn(handle_task(
            worker_register.clone(),
            worker_reeceiver.clone(),
            task_storage.clone(),
            task,
        ));
    }
}

async fn handle_task(
    worker_register: Arc<Mutex<WorkerRegister>>,
    mut worker_reeceiver: WatchReceiver<Vec<Worker>>,
    task_storage: Arc<Mutex<TaskStorage>>,
    task: TaskWithResult,
) {
    let worker = loop {
        if let Some(worker) = worker_reeceiver
            .borrow()
            .iter()
            .find(|w| w.is_available)
            .cloned()
        {
            break worker;
        }
        let _ = worker_reeceiver.changed().await;
    };

    set_worker_for_result(task_storage.clone(), &task.uuid, worker.clone()).await;
    set_status(task_storage.clone(), &task.uuid, Status::COMPUTING).await;

    println!("Sending Task {} to worker {}", task.task.code, worker.ip);

    let client = Client::new();
    let response = client
        .post(format!("{}/api/work", worker.ip))
        .json(&task.task)
        .send()
        .await;

    let computing_result: Option<ResultWithLoad> = match response {
        Ok(result) => {
            if result.status().is_success() {
                result.json::<ResultWithLoad>().await.ok()
            } else {
                eprintln!(
                    "Worker did not return a successfull response for {}",
                    &task.uuid
                );
                None
            }
        }
        Err(x) => {
            eprintln!("Networkerror in communication with worker: {}", x);
            None
        }
    };

    let handler_result = match computing_result {
        Some(res) => {
            worker_register
                .lock()
                .await
                .update_load(&worker.ip, res.load);
            match res.result {
                Some(x) => handle_success(x, task_storage.clone(), &task.uuid).await,
                None => {
                    eprintln!("No result could be found.");
                    handle_error(task_storage.clone(), &task.uuid).await
                }
            }
        }
        None => handle_error(task_storage.clone(), &task.uuid).await,
    };

    match handler_result {
        Err(_) => {
            eprintln!(
                "Handler did not register success or error correctly for task {}.",
                &task.uuid
            )
        }
        Ok(_) => {}
    }
}

async fn handle_success(
    result: ComputingResult,
    task_storage: Arc<Mutex<TaskStorage>>,
    uuid: &str,
) -> Result<(), ()> {
    task_storage.lock().await.update_result_of(uuid, result)?;
    set_status(task_storage, uuid, Status::FINISHED).await;
    Ok(())
}

async fn handle_error(task_storage: Arc<Mutex<TaskStorage>>, uuid: &str) -> Result<(), ()> {
    set_status(task_storage, uuid, Status::ERROR).await;
    Ok(())
}

async fn update_worker_load(
    worker_register: Arc<Mutex<WorkerRegister>>,
    worker_ip: &str,
    load: Load,
) {
    worker_register.lock().await.update_load(worker_ip, load);
}

async fn set_status(task_storage: Arc<Mutex<TaskStorage>>, uuid: &str, status: Status) {
    match task_storage.lock().await.update_satus_of(uuid, status) {
        Err(_) => eprint!("Could not change status of task {}.", uuid),
        Ok(_) => {}
    }
}

async fn set_worker_for_result(task_storage: Arc<Mutex<TaskStorage>>, uuid: &str, worker: Worker) {
    match task_storage
        .clone()
        .lock()
        .await
        .update_worker_of(uuid, worker)
    {
        Err(_) => eprint!("Error saving result for task {}", uuid),
        Ok(_) => {}
    }
}
