use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use regex::Regex;
use std::fs;

#[derive(Debug, Default)]
pub struct ModuleGraph {
    pub nodes: HashSet<PathBuf>,
    pub edges: HashMap<PathBuf, HashSet<PathBuf>>, // parent -> dependents
}

impl ModuleGraph {
    pub fn new() -> Self {
        ModuleGraph {
            nodes: HashSet::new(),
            edges: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, path: PathBuf) {
        self.nodes.insert(path.clone());
        self.edges.entry(path).or_default();
    }

    pub fn add_edge(&mut self, from: PathBuf, to: PathBuf) {
        self.edges.entry(from).or_default().insert(to);
    }

    pub fn scan_project(project: &Path) -> Self {
        let mut graph = ModuleGraph::new();
        let mod_regex = Regex::new(r"mod\s+([a-zA-Z0-9_]+)").unwrap();

        let src_path = project.join("src");
        if src_path.exists() {
            for entry in walkdir::WalkDir::new(&src_path).into_iter().filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_file() && path.extension().map(|s| s == "rs").unwrap_or(false) {
                    graph.add_node(path.to_path_buf());
                    if let Ok(content) = fs::read_to_string(path) {
                        for cap in mod_regex.captures_iter(&content) {
                            let mod_name = &cap[1];
                            let mod_path = path.parent().unwrap().join(format!("{}.rs", mod_name));
                            if mod_path.exists() {
                                graph.add_edge(path.to_path_buf(), mod_path);
                            }
                        }
                    }
                }
            }
        }

        graph
    }
}
