use crate::domain::{
    AiContext, AiResult, AtomIndexMapEntry, AtomSummary, Basis, CalculationSummary, Command, Element,
    GeometryEditMode, JobType, Method, Solvent,
};
use crate::functional_groups::{
    element_neighbors, find_all_benzene_rings, get_ring_neighbors, match_functional_groups,
    FunctionalGroupKind,
};
use std::collections::{HashMap, HashSet};

pub fn build_ai_context(state: &crate::domain::AppState) -> AiContext {
    let molecule = &state.domain.chemical_spec.molecule;
    let calculation = &state.domain.chemical_spec.calculation;

    let fg_matches = match_functional_groups(molecule);
    let mut atom_to_contexts: HashMap<u32, Vec<String>> = HashMap::new();

    // 1. 基本的な官能基マッチングの情報を格納
    for m in &fg_matches {
        let kind_str = format!("{:?}", m.kind);
        let mut atoms_to_mark = m.atom_ids.clone();

        // Alcoholの場合、水素もIn_Alcoholに含める
        if m.kind == FunctionalGroupKind::Alcohol {
            if let Some(o_id) = m.reference_atom_id {
                let h_neighbors = element_neighbors(molecule, o_id, Element::H);
                atoms_to_mark.extend(h_neighbors);
            }
        }

        for &id in &atoms_to_mark {
            atom_to_contexts
                .entry(id)
                .or_default()
                .push(format!("In_{}", kind_str));
        }
    }

    // 2. ベンゼン環の周辺原子へのコンテキスト伝播
    // すべてのベンゼン環を検出して処理する
    let benzene_matches: Vec<_> = fg_matches
        .iter()
        .filter(|m| m.kind == FunctionalGroupKind::BenzeneRing)
        .collect();

    for m in benzene_matches {
        // m.atom_ids は環の炭素6つ
        let ordered_ring = m.atom_ids.clone(); // 単純化: 順序付けは必須ではない場合もあるが、位置情報に依存するなら必要
        
        for (idx, (_ring_atom_id, neighbors)) in get_ring_neighbors(molecule, &ordered_ring)
            .into_iter()
            .enumerate()
        {
            for neighbor_id in neighbors {
                atom_to_contexts
                    .entry(neighbor_id)
                    .or_default()
                    .push(format!("BenzeneRing_{}th_position", idx + 1));
            }
        }
    }

    // ... (1. 基本的な官能基マッチングの情報を格納)
    // ...
    // 2. ベンゼン環の周辺原子へのコンテキスト伝播
    // ...

    let atom_index_map = molecule
        .atoms
        .iter()
        .enumerate()
        .map(|(index, atom)| AtomIndexMapEntry {
            display_index: index as u32 + 1,
            atom_id: atom.id,
        })
        .collect::<Vec<_>>();

    // atom_id -> display_index のマップを作成
    let id_to_display: HashMap<u32, u32> = atom_index_map
        .iter()
        .map(|e| (e.atom_id, e.display_index))
        .collect();

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
                    formal_charge: atom.formal_charge,
                    position: atom.position,
                    chemical_context: atom_to_contexts.get(&atom.id).map(|ctxs| ctxs.join(", ")),
                })
        })
        .collect::<Vec<_>>();

    // atom_id -> display_index -> context に変換
    let atom_context_map: HashMap<u32, String> = atom_to_contexts
        .into_iter()
        .filter_map(|(id, ctxs)| {
            let display_index = id_to_display.get(&id)?;
            Some((*display_index, ctxs.join(", ")))
        })
        .collect();

    AiContext {
        selected_atoms,
        atom_index_map,
        atom_context_map,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reducer::{self, infer_substitute_by_fragment_completion};

    #[test]
    fn local_geometry_rule_outputs_display_atom_ids() {
        let mut state = reducer::initial_app_state();
        state.domain.chemical_spec.molecule.atoms[0].id = 10;
        state.domain.chemical_spec.molecule.atoms[1].id = 20;
        state.ui.selected_atoms = vec![10, 20];
        let context = build_ai_context(&state);

        let result = propose_commands_by_rules("set bond length 1.42", &context);

        assert!(matches!(
            result.commands.as_slice(),
            [Command::SetBondLength {
                atom_ids: [1, 2],
                ..
            }]
        ));
        let resolved =
            resolve_atom_references(result.commands, &context).expect("ids should resolve");
        assert!(matches!(
            resolved.as_slice(),
            [Command::SetBondLength {
                atom_ids: [10, 20],
                ..
            }]
        ));
    }

    #[test]
    fn build_ai_context_includes_chemical_context() {
        let mut state = reducer::initial_app_state();
        // Setup benzoic acid-like structure for testing
        state.domain.chemical_spec.molecule.atoms = vec![
            crate::domain::Atom { id: 1, element: crate::domain::Element::C, isotope: None, nuclear_spin: None, formal_charge: 0, position: [0.0, 0.0, 0.0] },
            crate::domain::Atom { id: 2, element: crate::domain::Element::O, isotope: None, nuclear_spin: None, formal_charge: 0, position: [1.2, 0.0, 0.0] },
            crate::domain::Atom { id: 3, element: crate::domain::Element::O, isotope: None, nuclear_spin: None, formal_charge: 0, position: [0.0, 1.3, 0.0] },
            crate::domain::Atom { id: 4, element: crate::domain::Element::H, isotope: None, nuclear_spin: None, formal_charge: 0, position: [0.0, 2.2, 0.0] },
        ];
        state.domain.chemical_spec.molecule.bonds = vec![
            crate::domain::Bond { id: 1, atom_ids: [1, 2], order: 2 },
            crate::domain::Bond { id: 2, atom_ids: [1, 3], order: 1 },
            crate::domain::Bond { id: 3, atom_ids: [3, 4], order: 1 },
        ];
        state.ui.selected_atoms = vec![1];
        
        let context = build_ai_context(&state);
        let atom = &context.selected_atoms[0];
        assert_eq!(atom.display_index, 1);
        assert!(atom.chemical_context.as_ref().is_some_and(|fg| fg.contains("CarboxylicAcid")));
    }

    #[test]
    fn build_ai_context_includes_alcohol_hydrogen() {
        let mut state = reducer::initial_app_state();
        // Setup methanol: C-O-H
        state.domain.chemical_spec.molecule.atoms = vec![
            crate::domain::Atom { id: 1, element: crate::domain::Element::C, isotope: None, nuclear_spin: None, formal_charge: 0, position: [0.0, 0.0, 0.0] },
            crate::domain::Atom { id: 2, element: crate::domain::Element::O, isotope: None, nuclear_spin: None, formal_charge: 0, position: [1.2, 0.0, 0.0] },
            crate::domain::Atom { id: 3, element: crate::domain::Element::H, isotope: None, nuclear_spin: None, formal_charge: 0, position: [1.2, 0.9, 0.0] },
        ];
        state.domain.chemical_spec.molecule.bonds = vec![
            crate::domain::Bond { id: 1, atom_ids: [1, 2], order: 1 },
            crate::domain::Bond { id: 2, atom_ids: [2, 3], order: 1 },
        ];
        state.ui.selected_atoms = vec![3]; // Select H
        
        let context = build_ai_context(&state);
        let h_atom_id = 3;
        // Need to find display index for H atom (id 3)
        let display_h = context.atom_index_map.iter().find(|e| e.atom_id == h_atom_id).unwrap().display_index;
        
        let context_str = context.atom_context_map.get(&display_h).expect("Should have context");
        assert!(context_str.contains("In_Alcohol"));
    }

    fn build_4_hydroxybiphenyl() -> crate::domain::AppState {
        use crate::reducer::{reduce, initial_app_state};
        use crate::domain::{Command, Element};

        let state = initial_app_state();
        let state = reduce(state, Command::SubstituteByFragment { fragment_name: "phenyl".to_string(), start_atom_id: 2, end_atom_id: 1 });
        let context = build_ai_context(&state);

        let para_position = context
            .atom_context_map
            .iter()
            .find(|(_, v)| **v == "BenzeneRing_4th_position")
            .map(|(k, _)| *k)
            .unwrap();


        let para_position = display_index_to_atom_id(&context, para_position).unwrap();

        let para_carbon = infer_substitute_by_fragment_completion(
            &state.domain.chemical_spec.molecule,
            para_position
        )
            .unwrap()
            .end_atom_id;

        reduce(state, Command::SubstituteByFragment { fragment_name: "phenyl".to_string(), start_atom_id: para_position, end_atom_id: para_carbon })
    }

    #[test]
    fn build_ai_context_includes_biphenyl_context() {
        let state = build_4_hydroxybiphenyl();
        let context = build_ai_context(&state);
        
        let neighbor_contexts = context
            .atom_context_map
            .values()
            .filter(|v| v.contains("BenzeneRing_"))
            .count();

        assert_eq!(neighbor_contexts, 12, "Should have marked exactly 12 substituent atoms with BenzeneRing_nth_position context. Found: {}", neighbor_contexts);
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
            commands.push(Command::SetMethod {
                method: Method::B3LYP,
            });
        } else if token.contains("wb97xd") {
            commands.push(Command::SetMethod {
                method: Method::WB97XD,
            });
        } else if token.contains("6-31g(d)") {
            commands.push(Command::SetBasis {
                basis: Basis::Six31Gd,
            });
        } else if token.contains("def2-svp") {
            commands.push(Command::SetBasis {
                basis: Basis::Def2Svp,
            });
        } else if token.contains("def2-tzvp") {
            commands.push(Command::SetBasis {
                basis: Basis::Def2Tzvp,
            });
        } else if token.contains("thf") {
            commands.push(Command::SetSolvent {
                solvent: Some(Solvent::THF),
            });
        } else if token.contains("water") {
            commands.push(Command::SetSolvent {
                solvent: Some(Solvent::Water),
            });
        } else if token.contains("no_solvent") || token.contains("gas_phase") {
            commands.push(Command::SetSolvent { solvent: None });
        } else if let Some(job_type) = infer_job_type_by_rules(token) {
            commands.push(Command::SetJobType { job_type });
        } else if let Some(charge) = parse_kv_by_rules(token, "charge") {
            commands.push(Command::SetCharge { charge });
        } else if let Some(mult) =
            parse_kv_by_rules(token, "multiplicity").or_else(|| parse_kv_by_rules(token, "mult"))
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
        let val = token
            .trim_start_matches(keyword)
            .trim_matches(|c| c == ':' || c == '=');
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
//             | Command::ReplaceAtom { .. }
//     )
// }

fn infer_job_type_by_rules(text: &str) -> Option<JobType> {
    if text.contains("transition state") || text.split_whitespace().any(|token| token == "ts") {
        return Some(JobType::Ts);
    }

    let has_opt =
        text.contains("opt") || text.contains("optimize") || text.contains("optimization");
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
            let numeric = next.trim_matches(|char: char| char == ':' || char == '=' || char == ',');
            if let Ok(value) = numeric.parse::<i32>() {
                return Some(value);
            }
        }

        if let Some(rest) = word.strip_prefix(keyword) {
            let numeric = rest.trim_matches(|char: char| char == ':' || char == '=' || char == ',');
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
        .map(|atom| atom.display_index)
        .collect::<Vec<_>>();

    if (text.contains("dihedral") || text.contains("torsion")) && selected.len() >= 4 {
        return Some(Command::SetDihedralAngle {
            atom_ids: [selected[0], selected[1], selected[2], selected[3]],
            angle: value,
            mode: GeometryEditMode::AtomOnly,
        });
    }
    if (text.contains("bond angle") || text.contains("angle")) && selected.len() >= 3 {
        return Some(Command::SetBondAngle {
            atom_ids: [selected[0], selected[1], selected[2]],
            angle: value,
            mode: GeometryEditMode::AtomOnly,
        });
    }
    if (text.contains("bond length") || text.contains("distance")) && selected.len() >= 2 {
        return Some(Command::SetBondLength {
            atom_ids: [selected[0], selected[1]],
            length: value,
            mode: GeometryEditMode::AtomOnly,
        });
    }

    None
}

pub fn resolve_atom_references(
    commands: Vec<Command>,
    context: &AiContext,
) -> Result<Vec<Command>, String> {
    commands
        .into_iter()
        .map(|command| resolve_command_atom_references(command, context))
        .collect()
}

fn resolve_command_atom_references(
    command: Command,
    context: &AiContext,
) -> Result<Command, String> {
    let resolved = match command {
        Command::SetBondLength {
            atom_ids,
            length,
            mode,
        } => Command::SetBondLength {
            atom_ids: resolve_pair(atom_ids, context)?,
            length,
            mode,
        },
        Command::SetBondAngle {
            atom_ids,
            angle,
            mode,
        } => Command::SetBondAngle {
            atom_ids: resolve_triplet(atom_ids, context)?,
            angle,
            mode,
        },
        Command::SetDihedralAngle {
            atom_ids,
            angle,
            mode,
        } => Command::SetDihedralAngle {
            atom_ids: resolve_quartet(atom_ids, context)?,
            angle,
            mode,
        },
        Command::DeleteAtom { atom_id } => Command::DeleteAtom {
            atom_id: resolve_display_index(context, atom_id)?,
        },
        Command::SetAtomFormalCharge {
            atom_id,
            formal_charge,
        } => Command::SetAtomFormalCharge {
            atom_id: resolve_display_index(context, atom_id)?,
            formal_charge,
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
        Command::ReplaceAtom { atom_id, element } => Command::ReplaceAtom {
            atom_id: resolve_display_index(context, atom_id)?,
            element,
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
        let trimmed = part.trim_matches(|char: char| matches!(char, 'a' | 'A' | '°' | 'Å' | 'Å'));
        if trimmed.is_empty() {
            return None;
        }
        // Handle cases like "1.42オングストローム"
        let numeric_part =
            if let Some(index) = trimmed.find(|c: char| !c.is_ascii_digit() && c != '.') {
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
            | Command::SetAtomFormalCharge { .. }
            | Command::DeleteAtom { .. }
            | Command::AddBond { .. }
            | Command::DeleteBond { .. }
            | Command::PlaceTemplate { .. }
            | Command::AttachFragment { .. }
            | Command::SubstituteByFragment { .. }
            | Command::ReplaceAtom { .. } => unique.push(command),
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
