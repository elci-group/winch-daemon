use serde::{Serialize, Deserialize};
use std::{path::Path, fs};
use syn::{File, Item, ItemMod, ItemStruct, ItemEnum, ItemTrait};

#[derive(Debug, Serialize, Deserialize)]
pub struct Fingerprint {
    pub modules: Vec<String>,
    pub structs: Vec<String>,
    pub enums: Vec<String>,
    pub traits: Vec<String>,
}

pub fn fingerprint_project(path: &Path) -> anyhow::Result<Fingerprint> {
    let mut fp = Fingerprint {
        modules: vec![],
        structs: vec![],
        enums: vec![],
        traits: vec![],
    };

    for entry in walkdir::WalkDir::new(path) {
        let entry = entry?;
        if entry.path().extension().map(|s| s == "rs").unwrap_or(false) {
            let code = fs::read_to_string(entry.path())?;
            if let Ok(ast) = syn::parse_file(&code) {
                extract_items(&mut fp, &ast);
            }
        }
    }

    Ok(fp)
}

fn extract_items(fp: &mut Fingerprint, ast: &File) {
    for item in &ast.items {
        match item {
            Item::Mod(ItemMod { ident, .. }) => fp.modules.push(ident.to_string()),
            Item::Struct(ItemStruct { ident, .. }) => fp.structs.push(ident.to_string()),
            Item::Enum(ItemEnum { ident, .. }) => fp.enums.push(ident.to_string()),
            Item::Trait(ItemTrait { ident, .. }) => fp.traits.push(ident.to_string()),
            _ => {}
        }
    }
}
