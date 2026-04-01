mod daemon;
mod resolver;
mod notifier;
mod version_map;
mod workspace;
mod system_monitor;
mod config;

use anyhow::Result;
use std::path::{Path, PathBuf};
use std::fs;
use std::process::Command;

/// Get the current inotify max_user_watches limit
fn get_inotify_limit() -> u64 {
    match fs::read_to_string("/proc/sys/fs/inotify/max_user_watches") {
        Ok(content) => content.trim().parse().unwrap_or(8192),
        Err(_) => 8192,
    }
}

/// Increase inotify limit for daemon operation
fn increase_inotify_limit() -> Result<u64> {
    let original_limit = get_inotify_limit();
    let new_limit = 524288u64;

    if original_limit >= new_limit {
        return Ok(original_limit);
    }

    // Try to increase via sudo
    let output = Command::new("sudo")
        .args(&[
            "bash", "-c",
            &format!("echo {} > /proc/sys/fs/inotify/max_user_watches", new_limit)
        ])
        .output()?;

    if output.status.success() {
        eprintln!("📈 Increased inotify limit from {} to {}", original_limit, new_limit);
        Ok(original_limit)
    } else {
        eprintln!("⚠️ Could not increase inotify limit (may require sudo). Current limit: {}", original_limit);
        Ok(original_limit)
    }
}

/// Restore inotify limit to original value
fn restore_inotify_limit(original_limit: u64) -> Result<()> {
    let output = Command::new("sudo")
        .args(&[
            "bash", "-c",
            &format!("echo {} > /proc/sys/fs/inotify/max_user_watches", original_limit)
        ])
        .output()?;

    if output.status.success() {
        eprintln!("📉 Restored inotify limit to {}", original_limit);
    }
    Ok(())
}

/// Recursively scan a directory for Rust projects (Cargo.toml) with depth limit
fn find_rust_projects(base: &Path) -> Vec<PathBuf> {
    find_rust_projects_impl(base, 0, 5)
}

fn find_rust_projects_impl(base: &Path, depth: usize, max_depth: usize) -> Vec<PathBuf> {
    let mut projects = Vec::new();

    if !base.exists() || depth > max_depth {
        return projects;
    }

    let entries = match fs::read_dir(base) {
        Ok(e) => e,
        Err(_) => return projects,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            // Skip certain directories that typically don't contain projects
            let skip = path
                .file_name()
                .and_then(|n| n.to_str())
                .map(|s| {
                    s.starts_with('.')
                        || s == "node_modules"
                        || s == "target"
                        || s == ".cargo"
                        || s == ".git"
                })
                .unwrap_or(false);

            if !skip {
                projects.extend(find_rust_projects_impl(&path, depth + 1, max_depth));
            }
        } else if path.is_file() && path.file_name() == Some("Cargo.toml".as_ref()) {
            projects.push(path);
        }
    }

    projects
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Starting Winch System-Wide Daemon (Phase 3)...");
    
    // --- Home directory ---
    let home_dir = dirs::home_dir().unwrap_or(PathBuf::from("/home/adminx"));

    // --- Load configuration ---
    let cfg = config::load(&home_dir);

    // --- Detect Rust projects ---
    let rust_projects = find_rust_projects(&home_dir);
    if rust_projects.is_empty() {
        println!("⚠️ No Rust projects found in {}", home_dir.display());
    } else {
        println!("🔍 Found {} Rust projects", rust_projects.len());
        for p in &rust_projects {
            println!("   📦 {:?}", p);
        }
    }

    // --- Prepare watch directories ---
    let watch_dirs: Vec<PathBuf> = rust_projects
        .iter()
        .map(|p| p.parent().unwrap().to_path_buf())
        .collect();

    // --- Ensure version map database exists ---
    let db_path = cfg.db_path
        .map(PathBuf::from)
        .unwrap_or_else(|| home_dir.join(".winch/version_map.sqlite"));
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let _vm = version_map::VersionMap::new(db_path.to_str().unwrap())?;

    // --- Increase inotify limit for watching many directories ---
    let original_limit = increase_inotify_limit().unwrap_or(8192);

    // --- Prepare build command ---
    let build_command = cfg.build_command
        .unwrap_or_else(|| "cargo build".to_string());
    eprintln!("⚙️  Build command: {}", build_command);

    // --- Start watching (with cleanup on exit) ---
    let result = system_monitor::watch_directories(watch_dirs, home_dir, cfg.watch_dirs, build_command).await;

    // --- Restore inotify limit ---
    let _ = restore_inotify_limit(original_limit);

    result
}
