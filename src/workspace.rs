use anyhow::Result;
use std::process::Command;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct WorkspaceMember {
    pub name: String,
    pub path: String,
    pub dependencies: Vec<String>,
}

pub fn get_workspace_members() -> Result<HashMap<String, WorkspaceMember>> {
    let output = Command::new("cargo")
        .args(&["metadata", "--format-version", "1", "--no-deps"])
        .output()?;

    let metadata: Value = serde_json::from_slice(&output.stdout)?;
    let mut members = HashMap::new();

    if let Some(packages) = metadata["packages"].as_array() {
        for pkg in packages {
            let name = pkg["name"].as_str().unwrap_or("").to_string();
            let path = pkg["manifest_path"].as_str().unwrap_or("").to_string();
            let deps = pkg["dependencies"]
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|d| d["name"].as_str().map(|s| s.to_string()))
                .collect();

            members.insert(name.clone(), WorkspaceMember { name, path, dependencies: deps });
        }
    }

    Ok(members)
}
