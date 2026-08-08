mod model;
mod service;

use std::sync::Arc;

use axum::{Json, Router, extract::State, response::IntoResponse, routing::post};
use clap::Parser;
use reqwest::{Client, StatusCode};
use serde_json::json;
use tokio::sync::Mutex;

use crate::{
    model::{
        argument::{Argument, ServerType},
        task::Task,
        worker::Worker,
    },
    service::{
        execute_utils::ExecuteUtils,
        worker_distributor::{self, WorkerDistributor},
        worker_register::WorkerRegister,
    },
};

#[tokio::main]
async fn main() {
    let arguments: Argument = Argument::parse();

    let worker_distributer = Arc::new(Mutex::new(worker_distributor::WorkerDistributor::new(
        WorkerRegister::new(),
    )));

    let app = Router::<Arc<Mutex<WorkerDistributor>>>::new()
        .route("/api/distribute", post(distribute))
        .route("/api/work", post(work))
        .route("/api/register", post(register))
        .with_state(worker_distributer);

    if arguments.server_type == ServerType::WORKER {
        println!("Registering on Distributor...");
        let client = Client::new();

        let worker = Worker::new(
            true,
            String::from(format!("http://localhost:{}", arguments.port)),
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

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", arguments.port))
        .await
        .unwrap();
    println!("Server started successfully at 0.0.0.0:{}", arguments.port);
    axum::serve(listener, app).await.unwrap();
}

async fn distribute(
    State(worker_distributer): State<Arc<Mutex<WorkerDistributor>>>,
    Json(task): Json<Task>,
) -> impl IntoResponse {
    let resp = worker_distributer.lock().await.distribute_task(task).await;
    Json(resp)
}

async fn work(Json(task): Json<Task>) -> impl IntoResponse {
    let response = ExecuteUtils::exec(task).expect("failed");
    let json_response = json!(response);
    Json(json_response)
}

async fn register(
    State(worker_distributer): State<Arc<Mutex<WorkerDistributor>>>,
    Json(worker): Json<Worker>,
) -> impl IntoResponse {
    println!("registering worker {}...", worker.ip);
    match worker_distributer
        .lock()
        .await
        .worker_register
        .register(worker)
    {
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
        Ok(_) => StatusCode::OK,
    }
}
