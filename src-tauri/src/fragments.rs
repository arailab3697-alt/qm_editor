use crate::domain::{FragmentDefinition, FragmentDefinitionFile, Molecule};
use std::fs;
use std::path::Path;

pub fn list_available_fragments() -> Vec<FragmentDefinition> {
    let mut fragments = Vec::new();
    let dir = Path::new("fragments");

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            if let Ok(content) = fs::read_to_string(entry.path()) {
                if let Ok(file_def) = serde_json::from_str::<FragmentDefinitionFile>(&content) {
                    let template_path =
                        Path::new("templates").join(format!("{}.json", file_def.template_name));
                    if let Ok(template_content) = fs::read_to_string(template_path) {
                        if let Ok(template_json) =
                            serde_json::from_str::<serde_json::Value>(&template_content)
                        {
                            if let Ok(molecule) = serde_json::from_value::<Molecule>(
                                template_json["molecule"].clone(),
                            ) {
                                fragments.push(FragmentDefinition {
                                    name: file_def.name,
                                    display_name: file_def.display_name,
                                    description: file_def.description,
                                    template_name: file_def.template_name,
                                    molecule,
                                    attach_ports: file_def.attach_ports,
                                });
                            }
                        }
                    }
                }
            }
        }
    }
    fragments
}
