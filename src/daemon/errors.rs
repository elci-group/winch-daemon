// src/daemon/errors.rs

pub struct DaemonError {
    pub message: String,
}

impl DaemonError {
    pub fn new(msg: &str) -> Self {
        DaemonError {
            message: msg.to_string(),
        }
    }
}
