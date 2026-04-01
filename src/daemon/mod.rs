// src/daemon/mod.rs

//! Winch Daemon core module
//! Contains state management, graph structures, scheduling, and error handling.

pub mod state;
pub mod graph;
pub mod scheduler;
pub mod errors;

use crate::daemon::state::ProjectState;
use crate::daemon::graph::ModuleGraph;
use crate::daemon::scheduler::Scheduler;
use crate::daemon::errors::DaemonError;

use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use walkdir::WalkDir;
use anyhow::Result;
use std::path::PathBuf;

/// Handles building multiple projects concurrently.
/// Placeholder function: implement the build logic here.
pub async fn handle_build_multi() -> Result<()> {
    // Example pseudo-code
    let root = PathBuf::from(".");
    
    for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_dir() {
            // Perform build logic per directory
            println!("Found project directory: {:?}", path);
        }
    }

    Ok(())
}

/// Initializes a filesystem watcher for projects
pub fn init_watcher(projects: &[PathBuf]) -> Result<RecommendedWatcher> {
    let mut watcher: RecommendedWatcher = RecommendedWatcher::new(
        move |res| match res {
            Ok(event) => println!("Filesystem event: {:?}", event),
            Err(err) => eprintln!("Watch error: {:?}", err),
        },
        notify::Config::default(),
    )?;

    for project in projects {
        watcher.watch(&project.join("Cargo.toml"), RecursiveMode::NonRecursive)?;
        watcher.watch(&project.join("src"), RecursiveMode::Recursive)?;
    }

    Ok(watcher)
}
