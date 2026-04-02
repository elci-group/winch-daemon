# Winch Daemon

A system-wide Rust dependency monitoring daemon that watches your home directory for Rust projects and automatically tracks dependency changes. Runs as a persistent systemd user service with dynamic project discovery.

## Features

- **🎯 Single Instance**: Runs once as a systemd user service, not per-shell
- **🔍 Dynamic Discovery**: Automatically detects newly created Rust projects without restart
- **📡 Real-time Monitoring**: Watches for Cargo.toml and Cargo.lock changes
- **🛑 Graceful Shutdown**: Handles SIGTERM/SIGINT cleanly
- **📊 Auto-Restart**: Systemd restarts on failure
- **📝 Journal Logging**: Integrated with systemd journal for easy log viewing
- **⚡ Optimized Scanning**: Depth-limited filesystem traversal with smart directory skipping

## Installation

### 1. Build the binary

```bash
cd ~/winch-daemon
cargo build --release
```

The binary is built to: `~/winch-daemon/target/release/winch_daemon`

### 2. Install systemd service

```bash
mkdir -p ~/.config/systemd/user
cp ~/.config/systemd/user/winch-daemon.service ~/.config/systemd/user/winch-daemon.service
systemctl --user daemon-reload
```

Note: The service file is already created at `~/.config/systemd/user/winch-daemon.service`

### 3. Enable and start

```bash
# Enable auto-start at login
systemctl --user enable winch-daemon.service

# Start the daemon now
systemctl --user start winch-daemon.service

# Verify it's running
systemctl --user status winch-daemon.service
```

## Usage

### View logs

```bash
# Follow logs in real-time
journalctl --user -u winch-daemon -f

# View last 50 lines
journalctl --user -u winch-daemon -n 50

# View logs since boot
journalctl --user -u winch-daemon --since boot

# Filter for specific events
journalctl --user -u winch-daemon | grep "New Rust project"
```

### Manage the service

```bash
# Check status
systemctl --user status winch-daemon.service

# Stop the daemon
systemctl --user stop winch-daemon.service

# Restart the daemon
systemctl --user restart winch-daemon.service

# View resource usage
systemctl --user status winch-daemon.service
```

### Disable auto-start

```bash
systemctl --user disable winch-daemon.service
```

To remove from startup but keep the service available for manual starting.

## How It Works

### Project Discovery

1. **Initial Scan**: On startup, recursively scans `$HOME` (max depth 5) for all `Cargo.toml` files
2. **Watch Initialization**: Adds each project directory to the file watcher
3. **Dynamic Discovery**: Monitors `$HOME` and common subdirectories (`projects/`, `src/`, `repos/`, `workspace/`, `code/`, `dev/`) for new `Cargo.toml` files
4. **Auto-Add**: When a new project is created or cloned, it's automatically added to the watch list

### File Monitoring

- Uses `notify` crate with inotify backend (Linux)
- Watches all registered directories recursively for file events
- Filters for `Cargo.toml` and `Cargo.lock` changes only
- **Create events**: Trigger project discovery (new projects)
- **Modify events**: Queue rebuild requests (dependency changes)

### Graceful Shutdown

- Catches SIGTERM (from `systemctl stop`) and SIGINT (Ctrl+C)
- Sets shutdown flag
- Blocking event loop exits at next timeout (500ms max)
- Restores inotify limit to original value on exit
- Async runtime waits for loop to complete before returning

## Architecture

```
main.rs
  ├─ find_rust_projects()      → Recursively scan home dir for projects
  ├─ increase_inotify_limit()  → Bump OS file watch limit
  └─ watch_directories()       → Main event loop (async)
       │
       ├─ Arc<Mutex<RecommendedWatcher>>  → Thread-safe watcher
       ├─ HashSet<PathBuf>                → Track watched directories
       ├─ tokio::task::spawn_blocking()   → Run blocking event loop
       │    └─ recv_timeout(500ms)        → Check shutdown flag periodically
       └─ tokio::select! on SIGTERM/SIGINT
            └─ Trigger shutdown → restore inotify → exit
```

### Key Components

- **system_monitor.rs**: File watcher setup, event loop, shutdown handling
- **main.rs**: Project discovery, inotify limit management, entry point
- **daemon/**: Placeholder for rebuild logic (currently stubs)

## Configuration

### Inotify Limit

The daemon automatically increases the OS inotify limit on startup and restores it on exit. For systems with `fs.inotify.max_user_watches=524288` already set in sysctl, no action is needed.

To manually set a persistent limit:

```bash
echo fs.inotify.max_user_watches=524288 | sudo tee -a /etc/sysctl.conf
sudo sysctl -p
```

### Watch Directories

The daemon watches:
1. All discovered Rust project directories (via `Cargo.toml`)
2. `$HOME` (non-recursively)
3. Common subdirectories (recursively):
   - `~/projects/`
   - `~/src/`
   - `~/repos/`
   - `~/workspace/`
   - `~/code/`
   - `~/dev/`

To add more watched locations, edit `src/system_monitor.rs` in the `watch_directories()` function and rebuild.

## Troubleshooting

### Daemon not running after login

Check if systemd service is enabled:
```bash
systemctl --user is-enabled winch-daemon.service
```

If not enabled, run:
```bash
systemctl --user enable winch-daemon.service
systemctl --user start winch-daemon.service
```

### "Too many open files" error

Increase the inotify limit:
```bash
echo fs.inotify.max_user_watches=524288 | sudo tee -a /etc/sysctl.conf
sudo sysctl -p
```

Then restart the daemon:
```bash
systemctl --user restart winch-daemon.service
```

### Projects not being discovered

1. Verify the project directory is under `$HOME` or in a watched subdirectory
2. Check that the directory has a `Cargo.toml` file
3. View logs: `journalctl --user -u winch-daemon -f`
4. Manually trigger restart: `systemctl --user restart winch-daemon.service`

### Daemon using too much memory

Check the number of watched directories:
```bash
journalctl --user -u winch-daemon | grep "Watching"
```

If too many, consider:
1. Reducing depth limit in `find_rust_projects()`
2. Adding more directories to the skip list (e.g., `.cargo`, vendor dirs)
3. Restricting watched paths in `src/system_monitor.rs`

### High CPU usage

The daemon should be idle most of the time. High CPU may indicate:
1. Rapid file changes in watched directories
2. Excessive filesystem events (build artifacts, cache)

Check active events:
```bash
journalctl --user -u winch-daemon -f | grep "Change detected"
```

## Development

### Building

```bash
cargo build --release
```

### Running locally (not via systemd)

```bash
./target/release/winch_daemon
```

Press Ctrl+C to stop.

### Testing dynamic discovery

```bash
# In another terminal
mkdir ~/projects/test-project
touch ~/projects/test-project/Cargo.toml

# View logs
journalctl --user -u winch-daemon -f | grep "New Rust"
```

### Viewing all project discoveries on startup

```bash
journalctl --user -u winch-daemon | grep "📦"
```

## Systemd Service Details

**Service File**: `~/.config/systemd/user/winch-daemon.service`

- **Type**: Simple (long-running process)
- **Restart**: on-failure (restarts if exit code is non-zero)
- **RestartSec**: 5 seconds
- **TimeoutStopSec**: 10 seconds (max time to shut down gracefully)
- **KillSignal**: SIGTERM (allows clean shutdown)
- **Logging**: systemd journal

## Future Enhancements

- [ ] Implement actual rebuild triggering in `daemon::handle_build_multi()`
- [ ] Add configuration file for custom watch paths
- [ ] Structural analysis (fingerprinting, drift detection)
- [ ] Smart rebuild scheduling (batch, priority)
- [ ] Webhook notifications on changes
- [ ] Metrics/telemetry collection

## License

Licensed under the same license as the elci-group projects.

## Contributing

1. Build: `cargo build --release`
2. Test: `./target/release/winch_daemon` or `systemctl --user restart winch-daemon.service`
3. View logs: `journalctl --user -u winch-daemon -f`
4. Commit and push to GitHub
