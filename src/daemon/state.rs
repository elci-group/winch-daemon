use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    path::Path,
};

use twox_hash::XxHash64;
use std::hash::Hasher;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct FileState {
    pub hash: u64,
    pub last_built: Option<String>, // timestamp or version
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ProjectState {
    pub files: HashMap<String, FileState>,
}

impl ProjectState {
    pub fn new() -> Self {
        ProjectState { files: HashMap::new() }
    }

    pub fn hash_file(path: &Path) -> u64 {
        let bytes = fs::read(path).unwrap_or_default();
        let mut hasher = XxHash64::default();
        hasher.write(&bytes);
        hasher.finish()
    }

    pub fn update_file(&mut self, path: &Path) -> bool {
        let hash = Self::hash_file(path);
        let file_str = path.to_string_lossy().to_string();
        match self.files.get(&file_str) {
            Some(state) if state.hash == hash => false,
            _ => {
                self.files.insert(file_str, FileState { hash, last_built: None });
                true
            }
        }
    }
}
