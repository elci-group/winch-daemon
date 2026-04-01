# Agent Guide: Winch Daemon

Winch Daemon is a Rust-based system-wide dependency management tool. It scans for Rust projects, monitors them for changes, and orchestrates builds/dependency updates.

## Essential Commands

The project uses standard Cargo commands:

- **Build**: `cargo build`
- **Run**: `cargo run` (Requires Rust projects in the user's home directory or subdirectories)
- **Test**: `cargo test`
- **Check**: `cargo check`
- **Lint**: `cargo clippy`
- **Format**: `cargo fmt`

## Project Structure

- `src/main.rs`: Entry point. Handles home directory scanning and initializes components.
- `src/daemon/`: Core logic for state management, scheduling, and error handling.
  - `state.rs`: Project state tracking.
  - `graph.rs`: Dependency graph management.
  - `scheduler.rs`: Build scheduling.
- `src/system_monitor.rs`: Filesystem watching using the `notify` crate.
- `src/version_map.rs`: SQLite-backed storage for crate versions (`~/.winch/version_map.sqlite`).
- `src/resolver.rs`: Logic for resolving dependencies and versions.
- `src/workspace.rs`: Cargo workspace handling.
- Other modules (`advisor.rs`, `drift.rs`, `fingerprint.rs`, `intent.rs`, etc.) support the orchestration and analysis logic.

## Code Patterns & Conventions

- **Error Handling**: Uses `anyhow::Result` for application-level error management.
- **Asynchronous Code**: Uses `tokio` for the async runtime.
- **Filesystem**: Heavy use of `std::path::PathBuf` and `walkdir` for project discovery.
- **Persistence**: Uses `rusqlite` for local state tracking.
- **Module Structure**: Prefer nested modules (e.g., `daemon/mod.rs` exposing sub-modules).

## Key Components

### Project Discovery
The `find_rust_projects` function in `main.rs` recursively scans the home directory for `Cargo.toml` files.

### Filesystem Monitoring
`system_monitor.rs` uses `notify` to watch project directories. It triggers `daemon::handle_build_multi()` when `Cargo.toml` or `Cargo.lock` changes.

### Version Mapping
`VersionMap` (in `version_map.rs`) manages a SQLite database to track successful builds and versions across projects.

## Important Gotchas

- **Home Directory Dependency**: The daemon defaults to scanning `dirs::home_dir()` or `/home/adminx`. Ensure appropriate permissions or test environments.
- **Database Path**: The SQLite database is stored at `~/.winch/version_map.sqlite`.
- **Phase 3 Development**: As noted in `main.rs`, the project is in "Phase 3", meaning some functions (like `handle_build_multi`) might still be placeholders or in early stages.
- **Blocking Rx**: The watcher loop in `system_monitor.rs` uses a blocking `rx.recv()`. Be careful when integrating with other async tasks.

## Testing Approach

- Currently, the project relies on standard Rust unit and integration tests (`cargo test`).
- Check `src/` for `#[cfg(test)]` blocks within modules.
