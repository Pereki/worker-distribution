mod model;
mod service;
use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    response::IntoResponse,
    routing::{get, post},
};
use clap::Parser;
use local_ip_address::local_ip;
use reqwest::{Client, StatusCode};
use serde_json::json;
use tokio::sync::{Mutex, mpsc};
use uuid::Uuid;

use crate::{
    model::{
        app_state::AppState,
        argument::{Argument, ServerType},
        task::Task,
        task_with_result::TaskWithResult,
        worker::Worker,
    },
    service::{
        execute_utils::exec, health_check, task_storage::TaskStorage, worker_distributor,
        worker_register::WorkerRegister,
    },
};

#[tokio::main]
async fn main() {
    let arguments: Argument = Argument::parse();
    let (task_tx, task_rx) = mpsc::channel::<TaskWithResult>(64);
    let (worker_tx, worker_rx) = tokio::sync::watch::channel(Vec::new());

    let task_storage = Arc::new(Mutex::new(TaskStorage::new()));
    let worker_register = Arc::new(Mutex::new(WorkerRegister::new(worker_tx)));
    let app_state = AppState {
        task_storage: task_storage.clone(),
        task_tx,
        worker_register: worker_register.clone(),
    };

    let app = Router::<AppState>::new()
        .route("/api/distribute", post(distribute))
        .route("/api/work", post(work))
        .route("/api/register", post(register))
        .route("/api/result/{uuid}", get(result))
        .route("/api/health", get(health))
        .with_state(app_state);

    if arguments.server_type == ServerType::WORKER {
        println!("Registering on Distributor...");
        let client = Client::new();

        let worker = Worker::new(
            true,
            String::from(format!("http://{}:{}", local_ip().unwrap(), arguments.port)),
        );
        let _ = client
            .post(format!(
                "http://{}/api/register",
                arguments.distributor_ip.unwrap()
            ))
            .json(&worker)
            .send()
            .await;
    }

    tokio::spawn(worker_distributor::distribute_task(
        worker_register.clone(),
        worker_rx,
        task_rx,
        task_storage.clone(),
    ));

    tokio::spawn(health_check::check_health(worker_register.clone()));

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", arguments.port))
        .await
        .unwrap();
    println!("Server started successfully at 0.0.0.0:{}", arguments.port);
    axum::serve(listener, app).await.unwrap();
}

async fn distribute(
    State(app_state): State<AppState>,
    Json(task): Json<Task>,
) -> impl IntoResponse {
    let task_with_result = TaskWithResult {
        uuid: Uuid::new_v4().to_string(),
        task,
        result: Option::None,
        status: model::task_with_result::Status::QUEUED,
        worker: Option::None,
    };

    app_state
        .task_storage
        .lock()
        .await
        .task_hashmap
        .insert(task_with_result.uuid.clone(), task_with_result.clone());

    app_state.task_tx.send(task_with_result.clone()).await;
    Json(task_with_result)
}

async fn work(Json(task): Json<Task>) -> impl IntoResponse {
    let response = exec(task).await.expect("failed");
    let json_response = json!(response);
    Json(json_response)
}

async fn register(
    State(app_state): State<AppState>,
    Json(worker): Json<Worker>,
) -> impl IntoResponse {
    println!("registering worker {}...", worker.ip);
    match app_state.worker_register.lock().await.register(worker) {
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
        Ok(_) => StatusCode::OK,
    }
}

async fn result(
    State(app_state): State<AppState>,
    Path(uuid): Path<String>,
) -> Result<Json<TaskWithResult>, StatusCode> {
    let storage = app_state.task_storage.lock().await;
    match storage.task_hashmap.get(&uuid) {
        Some(task) => Ok(Json(task.clone())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

async fn health() -> StatusCode {
    StatusCode::OK
}
