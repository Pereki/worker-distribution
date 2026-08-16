#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct Worker {
    pub is_available: bool,
    pub load: Load,
    pub is_dead: bool,
    pub ip: String,
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct Load {
    pub cpu_usage: f64,
    pub memory_usage: f64,
}

impl Worker {
    pub fn new(is_available: bool, is_dead: bool, ip: String, load: Load) -> Self {
        Worker {
            is_available,
            ip,
            is_dead,
            load,
        }
    }
}

impl Load {
    pub fn new(cpu_usage: f64, memory_usage: f64) -> Load {
        Load {
            cpu_usage,
            memory_usage,
        }
    }
}
