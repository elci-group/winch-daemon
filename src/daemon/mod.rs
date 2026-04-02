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
/// Spawns a tokio task for each project with the specified build command.
pub async fn handle_build_multi(projects: Vec<PathBuf>, build_command: String) -> Result<()> {
    let parts: Vec<&str> = build_command.split_whitespace().collect();
    let (cmd, args) = if !parts.is_empty() {
        (parts[0], parts[1..].iter().map(|s| s.to_string()).collect::<Vec<_>>())
    } else {
        ("cargo", vec!["build".to_string()])
    };

    for project in projects {
        let cmd = cmd.to_string();
        let args = args.clone();
        tokio::spawn(async move {
            eprintln!("🔨 Building {:?}...", project);
            let result = tokio::process::Command::new(&cmd)
                .args(&args)
                .current_dir(&project)
                .output()
                .await;

            match result {
                Ok(output) if output.status.success() => {
                    eprintln!("✅ Build succeeded: {:?}", project);
                }
                Ok(output) => {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    eprintln!("❌ Build failed: {:?}\n{}", project, stderr);
                }
                Err(e) => {
                    eprintln!("❌ Build error: {:?}: {}", project, e);
                }
            }
        });
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
