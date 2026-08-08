use crate::model::worker::Worker;

pub struct WorkerRegister {
    workers: Vec<Worker>,
}

impl WorkerRegister {
    pub fn new() -> Self {
        Self {
            workers: Vec::new(),
        }
    }

    pub fn register(&mut self, worker: Worker) -> Result<(), ()> {
        self.workers.push(worker);
        Ok(())
    }

    pub fn get_available_worker(&self) -> Option<&Worker> {
        self.workers.iter().find(|worker| worker.is_available)
    }
}
