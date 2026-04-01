use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Deserialize, Default, Debug)]
pub struct WinchConfig {
    #[serde(default)]
    pub watch_dirs: Vec<PathBuf>,
    pub db_path: Option<String>,
    pub build_command: Option<String>,
}

pub fn load(home_dir: &Path) -> WinchConfig {
    let config_path = home_dir.join(".winch/config.toml");
    match std::fs::read_to_string(&config_path)
        .ok()
        .and_then(|s| toml::from_str::<WinchConfig>(&s).ok())
    {
        Some(cfg) => {
            eprintln!("📋 Loaded config from {:?}", config_path);
            if !cfg.watch_dirs.is_empty() {
                eprintln!("   Watch dirs: {} configured", cfg.watch_dirs.len());
            }
            cfg
        }
        None => {
            eprintln!("📋 No config found at {:?}, using defaults", config_path);
            WinchConfig::default()
        }
    }
}
