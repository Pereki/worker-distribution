use tokio::sync::watch::Sender;

use crate::model::worker::{Load, Worker};

pub struct WorkerRegister {
    pub workers: Vec<Worker>,
    worker_sender: Sender<Vec<Worker>>,
}

impl WorkerRegister {
    pub fn new(worker_sender: Sender<Vec<Worker>>) -> Self {
        Self {
            workers: Vec::new(),
            worker_sender: worker_sender,
        }
    }

    pub fn register(&mut self, worker: Worker) {
        self.workers.push(worker);
        self.notify();
    }

    pub fn get_available_worker(&self) -> Option<&Worker> {
        self.workers
            .iter()
            .find(|worker| worker.is_available && !worker.is_dead)
    }

    pub fn lock_worker(&mut self, worker_ip: &str) {
        self.workers
            .iter_mut()
            .find(|worker| worker.ip.eq(worker_ip))
            .unwrap()
            .is_available = false;

        self.notify();
    }

    pub fn unlock_worker(&mut self, worker_ip: &str) {
        self.workers
            .iter_mut()
            .find(|worker| worker.ip.eq(worker_ip))
            .unwrap()
            .is_available = true;

        self.notify();
    }

    pub fn worker_unavailable(&mut self, worker_ip: &str) {
        self.workers
            .iter_mut()
            .find(|worker| worker.ip.eq(worker_ip))
            .unwrap()
            .is_dead = true;

        self.notify();
    }

    pub fn worker_available(&mut self, worker_ip: &str) {
        self.workers
            .iter_mut()
            .find(|worker| worker.ip.eq(worker_ip))
            .unwrap()
            .is_dead = false;

        self.notify();
    }

    pub fn update_load(&mut self, worker_ip: &str, load: Load) {
        println!(
            "Updating load of worker {} with cpuload {} and memoryload {}",
            &worker_ip, &load.cpu_usage, &load.memory_usage
        );

        let worker = self
            .workers
            .iter_mut()
            .find(|worker| worker.ip.eq(worker_ip))
            .unwrap();

        if load.memory_usage > 80.00 || load.cpu_usage > 80.00 {
            worker.is_available = false;
        } else {
            worker.is_available = true;
        }

        worker.load = load;

        self.notify();
    }

    pub fn notify(&self) {
        match self.worker_sender.send(
            self.workers
                .iter()
                .filter(|worker| worker.is_available)
                .filter(|worker| !worker.is_dead)
                .cloned()
                .collect(),
        ) {
            Ok(_) => {}
            Err(x) => println!("Error when notifying subscribers: {}", x),
        }
    }
}
