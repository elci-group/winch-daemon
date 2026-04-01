use anyhow::Result;
use notify::{Watcher, RecommendedWatcher, RecursiveMode, EventKind};
use std::sync::{Arc, Mutex, mpsc::channel};
use std::sync::atomic::{AtomicBool, Ordering};
use std::collections::HashSet;
use std::path::PathBuf;
use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::mpsc as tokio_mpsc;

pub async fn watch_directories(directories: Vec<PathBuf>, home_dir: PathBuf) -> Result<()> {
    let (tx, rx) = channel();
    let watcher: RecommendedWatcher = Watcher::new(tx, notify::Config::default())?;
    let watcher = Arc::new(Mutex::new(watcher));

    // Channel for rebuild events from the blocking loop
    let (rebuild_tx, mut rebuild_rx) = tokio_mpsc::channel::<PathBuf>(100);

    // Track initially watched directories
    let mut initial_watched = HashSet::new();
    let total_dirs = directories.len();

    // Initialize watcher with project directories
    {
        let mut w = watcher.lock().unwrap();
        eprintln!("⏳ Initializing file watchers for {} directories...", total_dirs);
        for dir in directories.iter() {
            w.watch(dir, RecursiveMode::Recursive)?;
            initial_watched.insert(dir.clone());
        }

        // Watch home directory (non-recursive) for new project discovery
        w.watch(&home_dir, RecursiveMode::NonRecursive)?;

        // Watch common top-level subdirectories where projects might appear (recursively)
        for candidate in &["src", "projects", "repos", "workspace", "code", "dev"] {
            let p = home_dir.join(candidate);
            if p.is_dir() {
                let _ = w.watch(&p, RecursiveMode::Recursive);
            }
        }
    }

    eprintln!("✅ Watching {} project directories", total_dirs);

    // Set up shutdown signal handling
    let shutdown = Arc::new(AtomicBool::new(false));
    let shutdown_clone = shutdown.clone();
    let watcher_clone = watcher.clone();
    let rebuild_tx_clone = rebuild_tx.clone();

    // Spawn the blocking event loop in a background task
    let event_loop_handle = tokio::task::spawn_blocking(move || {
        let mut watched_dirs = initial_watched;

        loop {
            // Check shutdown flag periodically
            if shutdown_clone.load(Ordering::Relaxed) {
                eprintln!("📤 Shutting down file watcher...");
                break;
            }

            // Use recv_timeout for periodic shutdown checks
            match rx.recv_timeout(std::time::Duration::from_millis(500)) {
                Ok(Ok(event)) => {
                    // Check for new Cargo.toml (project discovery)
                    for path in &event.paths {
                        if path.file_name() == Some("Cargo.toml".as_ref()) {
                            if matches!(event.kind, EventKind::Create(_)) {
                                if let Some(parent) = path.parent() {
                                    let parent = parent.to_path_buf();
                                    if !watched_dirs.contains(&parent) {
                                        eprintln!("✨ New Rust project detected: {:?}", parent);
                                        if let Ok(mut w) = watcher_clone.lock() {
                                            if w.watch(&parent, RecursiveMode::Recursive).is_ok() {
                                                watched_dirs.insert(parent);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Check for changes in existing projects (Modify/Remove events)
                    for path in &event.paths {
                        if (path.ends_with("Cargo.toml") || path.ends_with("Cargo.lock"))
                            && !matches!(event.kind, EventKind::Create(_))
                        {
                            eprintln!("⚠️ Change detected: {:?}", path);
                            // Queue rebuild request for async handler
                            let _ = rebuild_tx_clone.blocking_send(path.clone());
                        }
                    }
                }
                Ok(Err(e)) => eprintln!("Watcher error: {:?}", e),
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    // Timeout is expected; loop continues to check shutdown flag
                    continue;
                }
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    eprintln!("Watcher channel disconnected");
                    break;
                }
            }
        }
    });

    // Async side: wait for SIGTERM or SIGINT, and process rebuild events
    let mut sigterm = signal(SignalKind::terminate())?;
    let mut sigint = signal(SignalKind::interrupt())?;
    let mut rebuild_requests = 0;

    loop {
        tokio::select! {
            _ = sigterm.recv() => {
                eprintln!("📬 Received SIGTERM, initiating graceful shutdown...");
                break;
            }
            _ = sigint.recv() => {
                eprintln!("📬 Received SIGINT, initiating graceful shutdown...");
                break;
            }
            Some(path) = rebuild_rx.recv() => {
                rebuild_requests += 1;
                eprintln!("📝 Rebuild queued for {:?} (total: {})", path, rebuild_requests);
                // In the future, this could trigger actual rebuild logic
            }
        }
    }

    // Signal the blocking loop to stop
    shutdown.store(true, Ordering::Relaxed);

    // Wait for the blocking loop to finish
    let _ = event_loop_handle.await;

    eprintln!("✅ File watcher stopped");
    Ok(())
}
