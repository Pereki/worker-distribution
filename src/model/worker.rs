#[derive(serde::Deserialize, serde::Serialize)]
pub struct Worker {
    pub is_available: bool,
    pub ip: String,
}

impl Worker {
    pub fn new(is_available: bool, ip: String) -> Self {
        Worker { is_available, ip }
    }
}
