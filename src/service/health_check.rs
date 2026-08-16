use std::{sync::Arc, time::Duration};

use reqwest::Client;
use tokio::{sync::Mutex, task::JoinSet};

use crate::{model::worker::Load, service::worker_register::WorkerRegister};

pub async fn check_health(worker_register: Arc<Mutex<WorkerRegister>>) {
    loop {
        tokio::time::sleep(Duration::from_secs(10)).await;
        let mut health_test_set = JoinSet::new();
        for worker in worker_register.lock().await.workers.iter() {
            health_test_set.spawn(curl_worker(worker.ip.clone(), worker_register.clone()));
        }

        health_test_set.join_all().await;
    }
}

pub async fn curl_worker(worker_ip: String, worker_register: Arc<Mutex<WorkerRegister>>) {
    println!("Requesting health of {}...", &worker_ip);
    let client = Client::new();

    let availability = match client.get(format!("{}/api/health", worker_ip)).send().await {
        Ok(x) => match x.status().is_success() {
            true => match x.json::<Load>().await.ok() {
                Some(el) => {
                    worker_register.lock().await.update_load(&worker_ip, el);
                    true
                }
                None => false,
            },
            false => false,
        },
        Err(_) => false,
    };

    if !availability {
        println!("Worker {} is not available.", worker_ip);
        worker_register
            .lock()
            .await
            .worker_unavailable(worker_ip.as_str());
    } else {
        println!("Worker {} is available.", worker_ip);
        worker_register
            .lock()
            .await
            .worker_available(worker_ip.as_str());
    }
}
