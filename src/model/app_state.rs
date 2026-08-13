use std::sync::Arc;

use tokio::sync::Mutex;

use tokio::sync::mpsc;

use crate::{
    model::task_with_result::TaskWithResult,
    service::{task_storage::TaskStorage, worker_register::WorkerRegister},
};

#[derive(Clone)]
pub struct AppState {
    pub task_tx: mpsc::Sender<TaskWithResult>,
    pub task_storage: Arc<Mutex<TaskStorage>>,
    pub worker_register: Arc<Mutex<WorkerRegister>>,
}
