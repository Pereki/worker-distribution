use crate::{
    model::{task::Task, worker::Worker},
    service::worker_register::{self, WorkerRegister},
};

pub struct WorkerDistributer {
    pub worker_register: WorkerRegister,
}

impl WorkerDistributer {
    pub fn new(worker_register: WorkerRegister) -> Self {
        WorkerDistributer { worker_register }
    }

    pub fn execute_task(&self, task: Task) {
        let default_worker = Worker::new(true, String::from("localhost"));

        let worker: &Worker = self
            .worker_register
            .get_available_worker()
            .unwrap_or(&default_worker);

        println!("Sending Task {} to worker {}", task.script, worker.ip);
    }
}
