use tokio::sync::watch::Sender;

use crate::model::worker::Worker;

pub struct WorkerRegister {
    workers: Vec<Worker>,
    worker_sender: Sender<Vec<Worker>>,
}

impl WorkerRegister {
    pub fn new(worker_sender: Sender<Vec<Worker>>) -> Self {
        Self {
            workers: Vec::new(),
            worker_sender: worker_sender,
        }
    }

    pub fn register(&mut self, worker: Worker) -> Result<(), ()> {
        self.workers.push(worker);
        self.worker_sender.send(
            self.workers
                .iter()
                .filter(|worker| worker.is_available)
                .cloned()
                .collect(),
        );
        Ok(())
    }

    pub fn get_available_worker(&self) -> Option<&Worker> {
        self.workers.iter().find(|worker| worker.is_available)
    }

    pub fn lock_worker(&mut self, worker_ip: &str) {
        self.workers
            .iter_mut()
            .find(|worker| worker.ip.eq(worker_ip))
            .unwrap()
            .is_available = false;

        self.worker_sender.send(
            self.workers
                .iter()
                .filter(|worker| worker.is_available)
                .cloned()
                .collect(),
        );
    }

    pub fn unlock_worker(&mut self, worker_ip: &str) {
        self.workers
            .iter_mut()
            .find(|worker| worker.ip.eq(worker_ip))
            .unwrap()
            .is_available = true;

        self.worker_sender.send(
            self.workers
                .iter()
                .filter(|worker| worker.is_available)
                .cloned()
                .collect(),
        );
    }
}
