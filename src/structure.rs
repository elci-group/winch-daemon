pub struct ModuleGraph {
    pub nodes: Vec<String>,
    pub edges: Vec<(String, String)>,
}
pub fn build_graph(project: &Path) -> Result<ModuleGraph>;
