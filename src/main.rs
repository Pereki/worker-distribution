mod model;
mod service;

use axum::{Json, Router, response::IntoResponse, routing::get};
use serde_json::json;

use crate::{
    model::task::Task,
    service::{
        worker_distributer::{self, WorkerDistributer},
        worker_register::WorkerRegister,
    },
};

#[tokio::main]
async fn main() {
    // build our application with a single route
    let app = Router::new().route("/api/exec", get(exec));

    // listen globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Server started successfully at 0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}

async fn exec() -> impl IntoResponse {
    let worker_distributer = worker_distributer::WorkerDistributer::new(WorkerRegister::new());

    worker_distributer.execute_task(Task::new(
        String::from("echo \"hello\""),
        model::languages::Language::SHELL,
    ));
    let json_response = json!({"status": "ok"});
    Json(json_response)
}
