use crate::domain::{AiContext, AiResult, Command, GeometryEditMode, Method, JobType, Basis, Solvent, AtomSummary, CalculationSummary, AtomIndexMapEntry, atom_index, atom_position};
use crate::gaussian::method_name;
use crate::geometry::{dihedral_degrees, sub, rotate};

pub fn build_ai_context(state: &crate::domain::AppState) -> AiContext {
    let molecule = &state.domain.chemical_spec.molecule;
    let calculation = &state.domain.chemical_spec.calculation;
    let atom_index_map = molecule
        .atoms
        .iter()
        .enumerate()
        .map(|(index, atom)| AtomIndexMapEntry {
            display_index: index as u32 + 1,
            atom_id: atom.id,
        })
        .collect::<Vec<_>>();
    let selected_atoms = state
        .ui
        .selected_atoms
        .iter()
        .filter_map(|atom_id| {
            molecule
                .atoms
                .iter()
                .enumerate()
                .find(|(_, atom)| atom.id == *atom_id)
                .map(|(index, atom)| AtomSummary {
                    display_index: index as u32 + 1,
                    element: atom.element,
                    isotope: atom.isotope,
                    nuclear_spin: atom.nuclear_spin,
                    position: atom.position,
                })
        })
        .collect::<Vec<_>>();

    AiContext {
        selected_atoms,
        atom_index_map,
        calculation: CalculationSummary {
            job_type: calculation.job_type,
            method: calculation.method,
            basis: calculation.basis,
            solvent: calculation.solvent,
            charge: calculation.charge,
            multiplicity: calculation.multiplicity,
        },
    }
}

pub fn propose_commands_by_rules(input: &str, context: &AiContext) -> AiResult {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return AiResult {
            commands: Vec::new(),
            explanation: "No request was provided.".to_string(),
        };
    }

    if let Some(result) = parse_json_ai_result(trimmed) {
        return result;
    }

    // Strict grammar check: starts with "set"
    let normalized = trimmed.to_ascii_lowercase();
    if !normalized.starts_with("set") {
        return AiResult {
            commands: Vec::new(),
            explanation: "Invalid syntax. Commands must start with 'set'.".to_string(),
        };
    }

    // Tokenize after 'set'
    let content = &normalized[3..].trim();
    let tokens: Vec<&str> = content.split_whitespace().collect();
    
    let mut commands = Vec::new();

    for token in tokens {
        if token.contains("b3lyp") {
            commands.push(Command::SetMethod { method: Method::B3LYP });
        } else if token.contains("wb97xd") {
            commands.push(Command::SetMethod { method: Method::WB97XD });
        } else if token.contains("6-31g(d)") {
            commands.push(Command::SetBasis { basis: Basis::Six31Gd });
        } else if token.contains("def2-svp") {
            commands.push(Command::SetBasis { basis: Basis::Def2Svp });
        } else if token.contains("def2-tzvp") {
            commands.push(Command::SetBasis { basis: Basis::Def2Tzvp });
        } else if token.contains("thf") {
            commands.push(Command::SetSolvent { solvent: Some(Solvent::THF) });
        } else if token.contains("water") {
            commands.push(Command::SetSolvent { solvent: Some(Solvent::Water) });
        } else if token.contains("no_solvent") || token.contains("gas_phase") {
            commands.push(Command::SetSolvent { solvent: None });
        } else if let Some(job_type) = infer_job_type_by_rules(token) {
            commands.push(Command::SetJobType { job_type });
        } else if let Some(charge) = parse_kv_by_rules(token, "charge") {
            commands.push(Command::SetCharge { charge });
        } else if let Some(mult) = parse_kv_by_rules(token, "multiplicity")
            .or_else(|| parse_kv_by_rules(token, "mult"))
        {
            if let Ok(multiplicity) = u32::try_from(mult) {
                commands.push(Command::SetMultiplicity { multiplicity });
            }
        }
    }
    
    // Geometry commands might span multiple tokens, handled separately if not strictly positional
    if let Some(command) = infer_geometry_command_by_rules(&normalized, context) {
        commands.push(command);
    }

    let unique_commands = dedupe_commands_by_rules(commands);
    let explanation = if unique_commands.is_empty() {
        "Valid 'set' command detected, but no recognizable parameters found.".to_string()
    } else {
        format!("Proposed {} command(s).", unique_commands.len())
    };

    AiResult {
        commands: unique_commands,
        explanation,
    }
}

fn parse_kv_by_rules(token: &str, keyword: &str) -> Option<i32> {
    if token.starts_with(keyword) {
        let val = token.trim_start_matches(keyword).trim_matches(|c| c == ':' || c == '=');
        return val.parse::<i32>().ok();
    }
    None
}


pub fn parse_ai_result_json(text: &str) -> Result<AiResult, String> {
    let parsed = serde_json::from_str::<AiResult>(text).map_err(|error| error.to_string())?;
    let commands = parsed
        .commands
        .into_iter()
        // .filter(is_ai_command)
        .collect::<Vec<_>>();
    Ok(AiResult {
        commands,
        explanation: if parsed.explanation.is_empty() {
            "Parsed JSON commands.".to_string()
        } else {
            parsed.explanation
        },
    })
}

fn parse_json_ai_result(text: &str) -> Option<AiResult> {
    parse_ai_result_json(text).ok()
}

// fn is_ai_command(command: &Command) -> bool {
//     matches!(
//         command,
//         Command::SetMethod { .. }
//             | Command::SetBasis { .. }
//             | Command::SetJobType { .. }
//             | Command::SetSolvent { .. }
//             | Command::SetCharge { .. }
//             | Command::SetMultiplicity { .. }
//             | Command::SetBondLength { .. }
//             | Command::SetBondAngle { .. }
//             | Command::SetDihedralAngle { .. }
//             | Command::AddAtom { .. }
//             | Command::DeleteAtom { .. }
//             | Command::AddBond { .. }
//             | Command::DeleteBond { .. }
//     )
// }

fn infer_job_type_by_rules(text: &str) -> Option<JobType> {
    if text.contains("transition state")
        || text.split_whitespace().any(|token| token == "ts")
    {
        return Some(JobType::Ts);
    }

    let has_opt = text.contains("opt")
        || text.contains("optimize")
        || text.contains("optimization");
    let has_freq = text.contains("freq") || text.contains("frequency");
    match (has_opt, has_freq) {
        (true, true) => Some(JobType::OptFreq),
        (true, false) => Some(JobType::Opt),
        (false, true) => Some(JobType::Freq),
        (false, false) => None,
    }
}

fn parse_number_after_by_rules(text: &str, keyword: &str) -> Option<i32> {
    let words = text.split_whitespace().collect::<Vec<_>>();
    for (index, word) in words.iter().enumerate() {
        if *word == keyword {
            let next = words.get(index + 1)?;
            let numeric = next.trim_matches(|char: char| {
                char == ':' || char == '=' || char == ','
            });
            if let Ok(value) = numeric.parse::<i32>() {
                return Some(value);
            }
        }

        if let Some(rest) = word.strip_prefix(keyword) {
            let numeric = rest.trim_matches(|char: char| {
                char == ':' || char == '=' || char == ','
            });
            if !numeric.is_empty() {
                if let Ok(value) = numeric.parse::<i32>() {
                    return Some(value);
                }
            }
        }
    }
    None
}

fn infer_geometry_command_by_rules(text: &str, context: &AiContext) -> Option<Command> {
    let value = parse_geometry_value_by_rules(text)?;
    let selected = context
        .selected_atoms
        .iter()
        .filter_map(|atom| display_index_to_atom_id(context, atom.display_index))
        .collect::<Vec<_>>();

    if (text.contains("dihedral") || text.contains("torsion"))
        && selected.len() >= 4
    {
        return Some(Command::SetDihedralAngle {
            atom_ids: [selected[0], selected[1], selected[2], selected[3]],
            angle: value,
            mode: GeometryEditMode::AtomOnly,
        });
    }
    if (text.contains("bond angle") || text.contains("angle"))
        && selected.len() >= 3
    {
        return Some(Command::SetBondAngle {
            atom_ids: [selected[0], selected[1], selected[2]],
            angle: value,
            mode: GeometryEditMode::AtomOnly,
        });
    }
    if (text.contains("bond length") || text.contains("distance"))
        && selected.len() >= 2
    {
        return Some(Command::SetBondLength {
            atom_ids: [selected[0], selected[1]],
            length: value,
            mode: GeometryEditMode::AtomOnly,
        });
    }

    None
}

pub fn resolve_atom_references(commands: Vec<Command>, context: &AiContext) -> Result<Vec<Command>, String> {
    commands
        .into_iter()
        .map(|command| resolve_command_atom_references(command, context))
        .collect()
}

fn resolve_command_atom_references(command: Command, context: &AiContext) -> Result<Command, String> {
    let resolved = match command {
        Command::SetBondLength { atom_ids, length, mode } => Command::SetBondLength {
            atom_ids: resolve_pair(atom_ids, context)?,
            length,
            mode,
        },
        Command::SetBondAngle { atom_ids, angle, mode } => Command::SetBondAngle {
            atom_ids: resolve_triplet(atom_ids, context)?,
            angle,
            mode,
        },
        Command::SetDihedralAngle { atom_ids, angle, mode } => Command::SetDihedralAngle {
            atom_ids: resolve_quartet(atom_ids, context)?,
            angle,
            mode,
        },
        Command::DeleteAtom { atom_id } => Command::DeleteAtom {
            atom_id: resolve_display_index(context, atom_id)?,
        },
        Command::AddBond { atom_ids, order } => Command::AddBond {
            atom_ids: resolve_pair(atom_ids, context)?,
            order,
        },
        Command::AttachFragment {
            fragment_name,
            target_atom_id,
            rotation_angle,
            orientation,
        } => Command::AttachFragment {
            fragment_name,
            target_atom_id: resolve_display_index(context, target_atom_id)?,
            rotation_angle,
            orientation,
        },
        Command::SubstituteByFragment {
            fragment_name,
            start_atom_id,
            end_atom_id,
        } => Command::SubstituteByFragment {
            fragment_name,
            start_atom_id: resolve_display_index(context, start_atom_id)?,
            end_atom_id: resolve_display_index(context, end_atom_id)?,
        },
        Command::ToggleAtomSelection { atom_id } => Command::ToggleAtomSelection {
            atom_id: resolve_display_index(context, atom_id)?,
        },
        other => other,
    };
    Ok(resolved)
}

fn resolve_pair(atom_ids: [u32; 2], context: &AiContext) -> Result<[u32; 2], String> {
    Ok([
        resolve_display_index(context, atom_ids[0])?,
        resolve_display_index(context, atom_ids[1])?,
    ])
}

fn resolve_triplet(atom_ids: [u32; 3], context: &AiContext) -> Result<[u32; 3], String> {
    Ok([
        resolve_display_index(context, atom_ids[0])?,
        resolve_display_index(context, atom_ids[1])?,
        resolve_display_index(context, atom_ids[2])?,
    ])
}

fn resolve_quartet(atom_ids: [u32; 4], context: &AiContext) -> Result<[u32; 4], String> {
    Ok([
        resolve_display_index(context, atom_ids[0])?,
        resolve_display_index(context, atom_ids[1])?,
        resolve_display_index(context, atom_ids[2])?,
        resolve_display_index(context, atom_ids[3])?,
    ])
}

fn resolve_display_index(context: &AiContext, display_index: u32) -> Result<u32, String> {
    display_index_to_atom_id(context, display_index)
        .ok_or_else(|| format!("Unknown display atom index: {display_index}"))
}

fn display_index_to_atom_id(context: &AiContext, display_index: u32) -> Option<u32> {
    context
        .atom_index_map
        .iter()
        .find(|entry| entry.display_index == display_index)
        .map(|entry| entry.atom_id)
}

fn parse_geometry_value_by_rules(text: &str) -> Option<f64> {
    text.split(|char: char| {
        matches!(
            char,
            ' ' | '　' | ':' | '=' | ',' | ';' | '(' | ')' | '[' | ']'
        )
    })
    .filter_map(|part| {
        let trimmed = part.trim_matches(|char: char| {
            matches!(char, 'a' | 'A' | '°' | 'Å' | 'Å')
        });
        if trimmed.is_empty() {
            return None;
        }
        // Handle cases like "1.42オングストローム"
        let numeric_part = if let Some(index) = trimmed.find(|c: char| !c.is_ascii_digit() && c != '.') {
            &trimmed[..index]
        } else {
            trimmed
        };
        numeric_part.parse::<f64>().ok()
    })
    .last()
}

fn dedupe_commands_by_rules(commands: Vec<Command>) -> Vec<Command> {
    let mut unique = Vec::new();
    let mut method = None;
    let mut basis = None;
    let mut job_type = None;
    let mut solvent = None;
    let mut charge = None;
    let mut multiplicity = None;

    for command in commands {
        match command {
            Command::SetMethod { .. } => method = Some(command),
            Command::SetBasis { .. } => basis = Some(command),
            Command::SetJobType { .. } => job_type = Some(command),
            Command::SetSolvent { .. } => solvent = Some(command),
            Command::SetCharge { .. } => charge = Some(command),
            Command::SetMultiplicity { .. } => multiplicity = Some(command),
            Command::SetBondLength { .. }
            | Command::SetBondAngle { .. }
            | Command::SetDihedralAngle { .. }
            | Command::AddAtom { .. }
            | Command::DeleteAtom { .. }
            | Command::AddBond { .. }
            | Command::DeleteBond { .. }
            | Command::PlaceTemplate { .. }
            | Command::AttachFragment { .. }
            | Command::SubstituteByFragment { .. } => unique.push(command),
            Command::SetMolecule { .. }
            | Command::ToggleAtomSelection { .. }
            | Command::ClearSelection => {}
        }
    }

    unique.extend(
        [method, basis, job_type, solvent, charge, multiplicity]
            .into_iter()
            .flatten(),
    );
    unique
}
