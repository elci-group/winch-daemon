// src/daemon/scheduler.rs

use std::collections::HashSet;
use std::path::PathBuf;
use tokio::time::{Duration, Instant};

pub struct Scheduler {
    pub debounce_ms: u64,
    pub build_affected_only: bool,
    pending: HashSet<PathBuf>,
    deadline: Option<Instant>,
}

impl Scheduler {
    /// Create a new scheduler with debounce settings
    pub fn new(debounce_ms: u64, build_affected_only: bool) -> Self {
        Self {
            debounce_ms,
            build_affected_only,
            pending: HashSet::new(),
            deadline: None,
        }
    }

    /// Add a changed path and reset/set the debounce deadline
    pub fn add(&mut self, path: PathBuf) {
        self.pending.insert(path);
        self.deadline = Some(Instant::now() + Duration::from_millis(self.debounce_ms));
    }

    /// Check if the quiet window has elapsed
    pub fn is_due(&self) -> bool {
        self.deadline.map(|d| Instant::now() >= d).unwrap_or(false)
    }

    /// Get the deadline for use with tokio::time::sleep_until
    pub fn deadline(&self) -> Option<Instant> {
        self.deadline
    }

    /// Drain and return all pending paths, clear deadline
    pub fn take_pending(&mut self) -> HashSet<PathBuf> {
        self.deadline = None;
        std::mem::take(&mut self.pending)
    }

    /// Check if there are any pending changes
    pub fn has_pending(&self) -> bool {
        !self.pending.is_empty()
    }
}
