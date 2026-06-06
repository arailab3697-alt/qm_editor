use crate::domain::Molecule;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
#[serde(rename_all = "camelCase")]
pub struct TemplateDefinition {
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub molecule: Molecule,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
#[serde(rename_all = "camelCase")]
pub struct TemplateSummary {
    pub name: String,
    pub display_name: String,
    pub description: String,
}

pub fn get_template(name: &str) -> Option<Molecule> {
    let path = format!("templates/{}.json", name);
    let content = fs::read_to_string(path).ok()?;
    let def: TemplateDefinition = serde_json::from_str(&content).ok()?;
    Some(def.molecule)
}

pub fn get_template_definition(name: &str) -> Option<TemplateDefinition> {
    let path = format!("templates/{}.json", name);
    let content = fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

pub fn list_available_templates() -> Vec<TemplateSummary> {
    let mut templates = Vec::new();
    let dir = Path::new("templates");

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            if let Some(content) = fs::read_to_string(entry.path()).ok() {
                if let Ok(def) = serde_json::from_str::<TemplateDefinition>(&content) {
                    templates.push(TemplateSummary {
                        name: def.name,
                        display_name: def.display_name,
                        description: def.description,
                    });
                }
            }
        }
    }
    templates
}
