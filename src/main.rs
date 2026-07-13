mod model;
mod service;

use axum::{
    Json, Router,
    response::IntoResponse,
    routing::{get, post},
};
use serde_json::json;

use crate::{
    model::task::Task,
    service::{
        execute_utils::ExecuteUtils,
        worker_distributer::{self, WorkerDistributer},
        worker_register::WorkerRegister,
    },
};

#[tokio::main]
async fn main() {
    // build our application with a single route
    let app = Router::new()
        .route("/api/exec", get(exec))
        .route("/api/work", post(work));

    // listen globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Server started successfully at 0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}

async fn exec() -> impl IntoResponse {
    let worker_distributer = worker_distributer::WorkerDistributer::new(WorkerRegister::new());

    let resp = worker_distributer
        .execute_task(Task::new(
            String::from("echo \"test\""),
            model::languages::Language::SHELL,
        ))
        .await;
    Json(resp)
}

async fn work(Json(task): Json<Task>) -> impl IntoResponse {
    let response = ExecuteUtils::exec(task).expect("failed");
    let json_response = json!(response);
    Json(json_response)
}
