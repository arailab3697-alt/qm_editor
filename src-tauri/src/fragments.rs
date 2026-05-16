use crate::domain::FragmentDefinition;
use std::fs;
use std::path::Path;

pub fn list_available_fragments() -> Vec<FragmentDefinition> {
    let mut fragments = Vec::new();
    let dir = Path::new("fragments");

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            if let Ok(content) = fs::read_to_string(entry.path()) {
                if let Ok(def) = serde_json::from_str::<FragmentDefinition>(&content) {
                    fragments.push(def);
                }
            }
        }
    }
    fragments
}
