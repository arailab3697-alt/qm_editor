use crate::domain::{AiContext, AiDiagnostic, AiOutput, AiRepairPolicy, Command};
use crate::fragments::list_available_fragments;
use crate::templates::list_available_templates;

pub struct Checker {
    parser: Parser,
    validator: Validator,
}

impl Checker {
    pub fn new() -> Self {
        Self {
            parser: Parser,
            validator: Validator,
        }
    }

    pub fn check(&self, text: &str, context: &AiContext) -> Result<AiOutput, AiDiagnostic> {
        let output = self.parser.parse(text)?;
        self.validator.validate(&output, context)?;
        Ok(output)
    }
}

struct Parser;

impl Parser {
    fn parse(&self, text: &str) -> Result<AiOutput, AiDiagnostic> {
        let json_str = extract_json_object(text).ok_or_else(|| AiDiagnostic {
            diagnostics: vec!["No JSON object found in response.".to_string()],
            repair_policy: AiRepairPolicy {
                fix_error: true,
                fix_warning: false,
            },
        })?;

        serde_json::from_str::<AiOutput>(json_str)
            .map(|mut output| {
                for command in &mut output.result.commands {
                    match command {
                        Command::PlaceTemplate { template_name, .. } => {
                            *template_name = template_name.to_lowercase();
                        }
                        Command::AttachFragment { fragment_name, .. } => {
                            *fragment_name = fragment_name.to_lowercase();
                        }
                        Command::SubstituteByFragment { fragment_name, .. } => {
                            *fragment_name = fragment_name.to_lowercase();
                        }
                        _ => {}
                    }
                }
                output
            })
            .map_err(|e| AiDiagnostic {
                diagnostics: vec![format!("JSON parse error: {}", e)],
                repair_policy: AiRepairPolicy {
                    fix_error: true,
                    fix_warning: false,
                },
            })
    }
}

struct Validator;

impl Validator {
    fn validate(&self, output: &AiOutput, context: &AiContext) -> Result<(), AiDiagnostic> {
        let valid_ids: Vec<u32> = context
            .atom_index_map
            .iter()
            .map(|e| e.display_index)
            .collect();
        let templates = list_available_templates();
        let fragments = list_available_fragments();
        let mut errors = Vec::new();

        for command in &output.result.commands {
            match command {
                Command::PlaceTemplate { template_name, .. } => {
                    if !templates
                        .iter()
                        .any(|t| t.name.to_lowercase() == *template_name)
                    {
                        errors.push(format!("Template '{}' not found. Please check the spelling or list available templates.", template_name));
                    }
                }
                Command::AttachFragment { fragment_name, .. }
                | Command::SubstituteByFragment { fragment_name, .. } => {
                    if !fragments
                        .iter()
                        .any(|f| f.name.to_lowercase() == *fragment_name)
                    {
                        errors.push(format!("Fragment '{}' not found. Please check the spelling or consider combining other fragments.", fragment_name));
                    }
                }
                _ => {}
            }

            let ids = match command {
                Command::SetBondLength { atom_ids, .. } => Some(atom_ids.to_vec()),
                Command::SetBondAngle { atom_ids, .. } => Some(atom_ids.to_vec()),
                Command::SetDihedralAngle { atom_ids, .. } => Some(atom_ids.to_vec()),
                Command::SetAtomFormalCharge { atom_id, .. } => Some(vec![*atom_id]),
                Command::DeleteAtom { atom_id } => Some(vec![*atom_id]),
                Command::AddBond { atom_ids, .. } => Some(atom_ids.to_vec()),
                Command::AttachFragment { target_atom_id, .. } => Some(vec![*target_atom_id]),
                Command::SubstituteByFragment {
                    start_atom_id,
                    end_atom_id,
                    ..
                } => {
                    if start_atom_id == end_atom_id {
                        errors.push(format!("SubstituteByFragment: start_atom_id ({}) and end_atom_id ({}) must be different.", start_atom_id, end_atom_id));
                    }
                    Some(vec![*start_atom_id, *end_atom_id])
                }
                Command::ToggleAtomSelection { atom_id } => Some(vec![*atom_id]),
                _ => None,
            };

            if let Some(ids) = ids {
                for id in ids {
                    if !valid_ids.contains(&id) {
                        errors.push(format!(
                            "Atom ID {} is invalid. Available IDs are: {:?}",
                            id, valid_ids
                        ));
                    }
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(AiDiagnostic {
                diagnostics: errors,
                repair_policy: AiRepairPolicy {
                    fix_error: true,
                    fix_warning: false,
                },
            })
        }
    }
}

fn extract_json_object(text: &str) -> Option<&str> {
    let trimmed = text.trim();
    if trimmed.starts_with('{') && trimmed.ends_with('}') {
        return Some(trimmed);
    }

    let start = trimmed.find('{')?;
    let end = trimmed.rfind('}')?;
    (start < end).then(|| &trimmed[start..=end])
}
