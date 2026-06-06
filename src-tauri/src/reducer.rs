use crate::domain::{
    atom_index, atom_position, AppState, Atom, Bond, Command, Element, GeometryEditMode,
    MassNumber, Molecule, SubstituteByFragmentCompletion, TwiceSpin,
};
use crate::geometry::{
    add, dihedral_degrees, distance, dot, length, normalize, perpendicular, rotate, rotate_vec,
    rotation_from_to, scale, sub,
};
use crate::templates;

pub fn initial_app_state() -> AppState {
    AppState {
        domain: crate::domain::DomainState {
            chemical_spec: crate::domain::ChemicalSpec {
                molecule: Molecule {
                    name: "Water".to_string(),
                    atoms: vec![
                        crate::domain::Atom {
                            id: 1,
                            element: Element::O,
                            isotope: None,
                            nuclear_spin: None,
                            formal_charge: 0,
                            position: [0.0, 0.0, 0.0],
                        },
                        crate::domain::Atom {
                            id: 2,
                            element: Element::H,
                            isotope: None,
                            nuclear_spin: None,
                            formal_charge: 0,
                            position: [0.758, 0.586, 0.0],
                        },
                        crate::domain::Atom {
                            id: 3,
                            element: Element::H,
                            isotope: None,
                            nuclear_spin: None,
                            formal_charge: 0,
                            position: [-0.758, 0.586, 0.0],
                        },
                    ],
                    bonds: vec![
                        crate::domain::Bond {
                            id: 1,
                            atom_ids: [1, 2],
                            order: 1,
                        },
                        crate::domain::Bond {
                            id: 2,
                            atom_ids: [1, 3],
                            order: 1,
                        },
                    ],
                },
                calculation: crate::domain::CalculationSpec {
                    job_type: crate::domain::JobType::Opt,
                    method: crate::domain::Method::B3LYP,
                    basis: crate::domain::Basis::Six31Gd,
                    solvent: None,
                    charge: 0,
                    multiplicity: 1,
                },
            },
        },
        ui: crate::domain::UiState {
            selected_atoms: Vec::new(),
        },
    }
}

pub fn reduce(mut state: AppState, command: Command) -> AppState {
    match command {
        Command::SetMethod { method } => state.domain.chemical_spec.calculation.method = method,
        Command::SetBasis { basis } => state.domain.chemical_spec.calculation.basis = basis,
        Command::SetJobType { job_type } => {
            state.domain.chemical_spec.calculation.job_type = job_type
        }
        Command::SetSolvent { solvent } => state.domain.chemical_spec.calculation.solvent = solvent,
        Command::SetCharge { charge } => state.domain.chemical_spec.calculation.charge = charge,
        Command::SetMultiplicity { multiplicity } => {
            state.domain.chemical_spec.calculation.multiplicity = multiplicity
        }
        Command::SetBondLength {
            atom_ids,
            length,
            mode,
        } => {
            set_bond_length(
                &mut state.domain.chemical_spec.molecule,
                atom_ids,
                length,
                mode,
            );
        }
        Command::SetBondAngle {
            atom_ids,
            angle,
            mode,
        } => {
            set_bond_angle(
                &mut state.domain.chemical_spec.molecule,
                atom_ids,
                angle,
                mode,
            );
        }
        Command::SetDihedralAngle {
            atom_ids,
            angle,
            mode,
        } => {
            set_dihedral_angle(
                &mut state.domain.chemical_spec.molecule,
                atom_ids,
                angle,
                mode,
            );
        }
        Command::AddAtom {
            element,
            position,
            isotope,
            nuclear_spin,
            formal_charge,
        } => add_atom(
            &mut state.domain.chemical_spec.molecule,
            element,
            position,
            isotope,
            nuclear_spin,
            formal_charge,
        ),
        Command::SetAtomFormalCharge {
            atom_id,
            formal_charge,
        } => {
            if let Some(atom_index) = atom_index(&state.domain.chemical_spec.molecule, atom_id) {
                state.domain.chemical_spec.molecule.atoms[atom_index].formal_charge = formal_charge;
            }
        }
        Command::DeleteAtom { atom_id } => {
            delete_atom(&mut state.domain.chemical_spec.molecule, atom_id);
            state
                .ui
                .selected_atoms
                .retain(|selected| *selected != atom_id);
        }
        Command::AddBond { atom_ids, order } => {
            add_bond(&mut state.domain.chemical_spec.molecule, atom_ids, order);
        }
        Command::DeleteBond { bond_id } => {
            state
                .domain
                .chemical_spec
                .molecule
                .bonds
                .retain(|bond| bond.id != bond_id);
        }
        Command::PlaceTemplate {
            template_name,
            position,
            direction,
        } => {
            place_template(
                &mut state.domain.chemical_spec.molecule,
                &template_name,
                position,
                direction,
            );
        }
        Command::AttachFragment {
            fragment_name,
            target_atom_id,
            rotation_angle,
            orientation,
        } => {
            attach_fragment(
                &mut state.domain.chemical_spec.molecule,
                &fragment_name,
                target_atom_id,
                rotation_angle,
                orientation,
            );
        }
        Command::SubstituteByFragment {
            fragment_name,
            start_atom_id,
            end_atom_id,
        } => {
            substitute_by_fragment(
                &mut state.domain.chemical_spec.molecule,
                &fragment_name,
                start_atom_id,
                end_atom_id,
            );
            let atom_ids: std::collections::HashSet<u32> = state
                .domain
                .chemical_spec
                .molecule
                .atoms
                .iter()
                .map(|atom| atom.id)
                .collect();
            state
                .ui
                .selected_atoms
                .retain(|atom_id| atom_ids.contains(atom_id));
        }
        Command::SetMolecule { molecule } => {
            state.domain.chemical_spec.molecule = molecule;
            state.ui.selected_atoms.clear();
        }
        Command::ToggleAtomSelection { atom_id } => {
            if let Some(index) = state.ui.selected_atoms.iter().position(|id| *id == atom_id) {
                state.ui.selected_atoms.remove(index);
            } else {
                state.ui.selected_atoms.push(atom_id);
            }
        }
        Command::ReplaceAtom { atom_id, element } => {
            if let Some(index) = atom_index(&state.domain.chemical_spec.molecule, atom_id) {
                let atom = &mut state.domain.chemical_spec.molecule.atoms[index];
                atom.element = element;
                atom.isotope = None;
                atom.nuclear_spin = None;
            }
        }
        Command::ClearSelection => state.ui.selected_atoms.clear(),
    }
    state
}

fn set_bond_length(
    molecule: &mut Molecule,
    atom_ids: [u32; 2],
    length: f64,
    mode: GeometryEditMode,
) {
    if !length.is_finite() || length <= 0.0 {
        return;
    }
    let Some(anchor_index) = atom_index(molecule, atom_ids[0]) else {
        return;
    };
    let Some(moving_index) = atom_index(molecule, atom_ids[1]) else {
        return;
    };
    let anchor = molecule.atoms[anchor_index].position;
    let direction = sub(molecule.atoms[moving_index].position, anchor);
    let Some(unit) = normalize(direction) else {
        return;
    };
    let target = add(anchor, scale(unit, length));
    let delta = sub(target, molecule.atoms[moving_index].position);
    apply_length_or_dihedral_motion(molecule, atom_ids[0], atom_ids[1], delta, mode);
}

fn set_bond_angle(molecule: &mut Molecule, atom_ids: [u32; 3], angle: f64, mode: GeometryEditMode) {
    if !angle.is_finite() || !(0.0..=180.0).contains(&angle) {
        return;
    }
    let Some(first) = atom_position(molecule, atom_ids[0]) else {
        return;
    };
    let Some(center) = atom_position(molecule, atom_ids[1]) else {
        return;
    };
    let Some(moving_index) = atom_index(molecule, atom_ids[2]) else {
        return;
    };
    let moving = molecule.atoms[moving_index].position;
    let Some(axis_to_first) = normalize(sub(first, center)) else {
        return;
    };
    let moving_vector = sub(moving, center);
    let moving_length = length(moving_vector);
    if moving_length <= f64::EPSILON {
        return;
    }

    let projected = sub(
        moving_vector,
        scale(axis_to_first, dot(moving_vector, axis_to_first)),
    );
    let side = normalize(projected).unwrap_or_else(|| perpendicular(axis_to_first));
    let radians = angle.to_radians();
    let new_vector = scale(
        add(
            scale(axis_to_first, radians.cos()),
            scale(side, radians.sin()),
        ),
        moving_length,
    );
    let target = add(center, new_vector);
    let delta = sub(target, molecule.atoms[moving_index].position);
    apply_angle_motion(molecule, atom_ids[1], atom_ids[0], atom_ids[2], delta, mode);
}

fn set_dihedral_angle(
    molecule: &mut Molecule,
    atom_ids: [u32; 4],
    angle: f64,
    mode: GeometryEditMode,
) {
    if !angle.is_finite() {
        return;
    }
    let Some(first) = atom_position(molecule, atom_ids[0]) else {
        return;
    };
    let Some(second) = atom_position(molecule, atom_ids[1]) else {
        return;
    };
    let Some(third) = atom_position(molecule, atom_ids[2]) else {
        return;
    };
    let Some(moving_index) = atom_index(molecule, atom_ids[3]) else {
        return;
    };
    let moving = molecule.atoms[moving_index].position;
    let Some(current) = dihedral_degrees(first, second, third, moving) else {
        return;
    };
    let delta = (angle - current).to_radians();
    let Some(axis) = normalize(sub(third, second)) else {
        return;
    };
    let target = add(third, rotate(sub(moving, third), axis, delta));
    let delta_vec = sub(target, molecule.atoms[moving_index].position);
    apply_length_or_dihedral_motion(molecule, atom_ids[1], atom_ids[2], delta_vec, mode);
}

fn apply_length_or_dihedral_motion(
    molecule: &mut Molecule,
    first_atom: u32,
    second_atom: u32,
    delta: [f64; 3],
    mode: GeometryEditMode,
) {
    match mode {
        GeometryEditMode::AtomOnly => {
            if let Some(second_idx) = atom_index(molecule, second_atom) {
                molecule.atoms[second_idx].position =
                    add(molecule.atoms[second_idx].position, delta);
            }
        }
        GeometryEditMode::MoveOtherSide => {
            let Some(moving_ids) = connected_component_without_bond(
                molecule,
                second_atom,
                first_atom,
                [first_atom, second_atom],
            ) else {
                return;
            };
            translate_atoms(molecule, &moving_ids, delta);
        }
        GeometryEditMode::MoveBothSides => {
            let Some(second_side) = connected_component_without_bond(
                molecule,
                second_atom,
                first_atom,
                [first_atom, second_atom],
            ) else {
                return;
            };
            let Some(first_side) = connected_component_without_bond(
                molecule,
                first_atom,
                second_atom,
                [first_atom, second_atom],
            ) else {
                return;
            };
            translate_atoms(molecule, &second_side, scale(delta, 0.5));
            translate_atoms(molecule, &first_side, scale(delta, -0.5));
        }
    }
}

fn apply_angle_motion(
    molecule: &mut Molecule,
    center_atom: u32,
    fixed_atom: u32,
    moving_atom: u32,
    delta: [f64; 3],
    mode: GeometryEditMode,
) {
    match mode {
        GeometryEditMode::AtomOnly => {
            if let Some(moving_idx) = atom_index(molecule, moving_atom) {
                molecule.atoms[moving_idx].position =
                    add(molecule.atoms[moving_idx].position, delta);
            }
        }
        GeometryEditMode::MoveOtherSide => {
            let Some(moving_ids) = connected_component_without_bond(
                molecule,
                moving_atom,
                center_atom,
                [center_atom, moving_atom],
            ) else {
                return;
            };
            translate_atoms(molecule, &moving_ids, delta);
        }
        GeometryEditMode::MoveBothSides => {
            let Some(moving_side) = connected_component_without_bond(
                molecule,
                moving_atom,
                center_atom,
                [center_atom, moving_atom],
            ) else {
                return;
            };
            let Some(fixed_side) = connected_component_without_bond(
                molecule,
                fixed_atom,
                center_atom,
                [center_atom, fixed_atom],
            ) else {
                return;
            };
            translate_atoms(molecule, &moving_side, scale(delta, 0.5));
            translate_atoms(molecule, &fixed_side, scale(delta, -0.5));
        }
    }
}

fn translate_atoms(molecule: &mut Molecule, atom_ids: &[u32], delta: [f64; 3]) {
    for atom_id in atom_ids {
        if let Some(idx) = atom_index(molecule, *atom_id) {
            molecule.atoms[idx].position = add(molecule.atoms[idx].position, delta);
        }
    }
}

fn connected_component_without_bond(
    molecule: &Molecule,
    start: u32,
    blocked: u32,
    blocked_bond: [u32; 2],
) -> Option<Vec<u32>> {
    use std::collections::{HashSet, VecDeque};
    let mut visited: HashSet<u32> = HashSet::new();
    let mut queue = VecDeque::new();
    visited.insert(start);
    queue.push_back(start);

    while let Some(current) = queue.pop_front() {
        for bond in &molecule.bonds {
            if crate::domain::same_bond(bond.atom_ids, blocked_bond) {
                continue;
            }
            let next = if bond.atom_ids[0] == current {
                bond.atom_ids[1]
            } else if bond.atom_ids[1] == current {
                bond.atom_ids[0]
            } else {
                continue;
            };
            if visited.insert(next) {
                queue.push_back(next);
            }
        }
    }
    if visited.contains(&blocked) {
        None
    } else {
        Some(visited.into_iter().collect())
    }
}

fn add_atom(
    molecule: &mut Molecule,
    element: Element,
    position: [f64; 3],
    isotope: Option<MassNumber>,
    nuclear_spin: Option<TwiceSpin>,
    formal_charge: i32,
) {
    if !position.iter().all(|coordinate| coordinate.is_finite()) {
        return;
    }
    molecule.atoms.push(crate::domain::Atom {
        id: next_atom_id(molecule),
        element,
        isotope,
        nuclear_spin,
        formal_charge,
        position,
    });
}

fn delete_atom(molecule: &mut Molecule, atom_id: u32) {
    molecule.atoms.retain(|atom| atom.id != atom_id);
    molecule
        .bonds
        .retain(|bond| !bond.atom_ids.contains(&atom_id));
}

fn add_bond(molecule: &mut Molecule, atom_ids: [u32; 2], order: u8) {
    if atom_ids[0] == atom_ids[1] || !(1..=3).contains(&order) {
        return;
    }
    if atom_index(molecule, atom_ids[0]).is_none() || atom_index(molecule, atom_ids[1]).is_none() {
        return;
    }
    if molecule
        .bonds
        .iter()
        .any(|bond| crate::domain::same_bond(bond.atom_ids, atom_ids))
    {
        return;
    }
    molecule.bonds.push(crate::domain::Bond {
        id: next_bond_id(molecule),
        atom_ids,
        order,
    });
}

fn place_template(molecule: &mut Molecule, name: &str, position: [f64; 3], _direction: [f64; 3]) {
    let Some(mut template) = templates::get_template(name) else {
        return;
    };

    let base_id = next_atom_id(molecule).saturating_sub(1);
    let bond_base_id = next_bond_id(molecule).saturating_sub(1);

    for atom in &mut template.atoms {
        atom.id += base_id;
        atom.position = add(atom.position, position);
    }
    for bond in &mut template.bonds {
        bond.id += bond_base_id;
        bond.atom_ids[0] += base_id;
        bond.atom_ids[1] += base_id;
    }

    molecule.atoms.extend(template.atoms);
    molecule.bonds.extend(template.bonds);
}

fn next_atom_id(molecule: &Molecule) -> u32 {
    molecule
        .atoms
        .iter()
        .map(|atom| atom.id)
        .max()
        .unwrap_or(0)
        .saturating_add(1)
}

fn next_bond_id(molecule: &Molecule) -> u32 {
    molecule
        .bonds
        .iter()
        .map(|bond| bond.id)
        .max()
        .unwrap_or(0)
        .saturating_add(1)
}

fn attach_fragment(
    molecule: &mut Molecule,
    fragment_name: &str,
    target_atom_id: u32,
    _rotation_angle: f64,
    _orientation: [f64; 3],
) {
    let fragments = crate::fragments::list_available_fragments();

    let Some(fragment) = fragments.iter().find(|f| f.name == fragment_name) else {
        return;
    };

    let Some(mut template) = crate::templates::get_template(&fragment.template_name) else {
        return;
    };

    let Some(port_atom_id) = fragment
        .attach_ports
        .iter()
        .filter_map(|p| {
            if let crate::domain::PortType::Atom { target_id } = p.port_type {
                Some(target_id)
            } else {
                None
            }
        })
        .next()
    else {
        return;
    };

    let Some(target_pos) = atom_position(molecule, target_atom_id) else {
        return;
    };

    // Access atoms before moving to new collection
    let port_atom_pos = template
        .atoms
        .iter()
        .find(|a| a.id == port_atom_id)
        .map(|a| a.position)
        .unwrap_or([0.0, 0.0, 0.0]);

    let mut template_atoms = template.atoms;
    template_atoms.retain(|a| a.id != port_atom_id);

    // Shift atoms
    let shift = sub(target_pos, port_atom_pos);

    for atom in &mut template_atoms {
        atom.position = add(atom.position, shift);
    }

    let base_id = next_atom_id(molecule).saturating_sub(1);
    let bond_base_id = next_bond_id(molecule).saturating_sub(1);

    for atom in &mut template_atoms {
        atom.id += base_id;
    }
    for bond in &mut template.bonds {
        bond.id += bond_base_id;
        bond.atom_ids[0] += base_id;
        bond.atom_ids[1] += base_id;
    }

    molecule.atoms.extend(template_atoms);
    molecule.bonds.extend(template.bonds);

    add_bond(molecule, [target_atom_id, port_atom_id + base_id], 1);
}

pub fn infer_substitute_by_fragment_completion(
    molecule: &Molecule,
    selected_atom_id: u32,
) -> Option<SubstituteByFragmentCompletion> {
    let mut connected_bonds = molecule
        .bonds
        .iter()
        .filter(|bond| bond.atom_ids.contains(&selected_atom_id));

    let bond = connected_bonds.next()?;
    if connected_bonds.next().is_some() {
        return None;
    }

    let neighbor_atom_id = if bond.atom_ids[0] == selected_atom_id {
        bond.atom_ids[1]
    } else {
        bond.atom_ids[0]
    };

    Some(SubstituteByFragmentCompletion {
        start_atom_id: selected_atom_id,
        end_atom_id: neighbor_atom_id,
    })
}

pub fn substitute_by_fragment(
    molecule: &mut Molecule,
    fragment_name: &str,
    start_atom_id: u32,
    end_atom_id: u32,
) {
    use std::collections::{HashMap, HashSet};

    let fragments = crate::fragments::list_available_fragments();
    println!("{:?}", fragments);

    let Some(fragment) = fragments.iter().find(|f| f.name == fragment_name) else {
        return;
    };

    let Some(mut template) = crate::templates::get_template(&fragment.template_name) else {
        return;
    };

    // ------------------------------------------------------------
    // Find target bond A(start/consumed) -- B(end/retained)
    // ------------------------------------------------------------

    let Some(target_bond_index) = molecule.bonds.iter().position(|b| {
        (b.atom_ids[0] == start_atom_id && b.atom_ids[1] == end_atom_id)
            || (b.atom_ids[0] == end_atom_id && b.atom_ids[1] == start_atom_id)
    }) else {
        return;
    };

    // ------------------------------------------------------------
    // Find fragment port bond X -- Y
    //
    // retained_atom = X
    // consumed_atom = Y
    // ------------------------------------------------------------

    let Some((consumed_atom_id, retained_atom_id)) = fragment.attach_ports.iter().find_map(|p| {
        if let crate::domain::PortType::Bond {
            start_atom_id,
            end_atom_id,
        } = p.port_type
        {
            Some((end_atom_id, start_atom_id))
        } else {
            None
        }
    }) else {
        return;
    };

    // ------------------------------------------------------------
    // Get positions
    // ------------------------------------------------------------

    let Some(target_consumed_pos) = atom_position(molecule, start_atom_id) else {
        return;
    };

    let Some(target_retained_pos) = atom_position(molecule, end_atom_id) else {
        return;
    };

    let Some(consumed_pos) = template
        .atoms
        .iter()
        .find(|a| a.id == consumed_atom_id)
        .map(|a| a.position)
    else {
        return;
    };

    let Some(retained_pos) = template
        .atoms
        .iter()
        .find(|a| a.id == retained_atom_id)
        .map(|a| a.position)
    else {
        return;
    };

    // ------------------------------------------------------------
    // Build transform
    //
    // Align:
    // retained -> consumed
    // to
    // target_consumed -> target_retained
    // ------------------------------------------------------------

    let target_vec = normalize(sub(target_consumed_pos, target_retained_pos))
        .expect("target bond must be valid");
    let fragment_vec =
        normalize(sub(retained_pos, consumed_pos)).expect("fragment bond must be valid");

    let base_rotation = rotation_from_to(fragment_vec, target_vec);

    // Optimize rotation around bond axis (target_vec)
    let mut best_angle = 0.0;
    let mut min_repulsion = f64::MAX;

    // Two-pass search: 16 steps (22.5 deg) then 16 steps around the best angle
    let mut current_center = 0.0;
    let mut step = 22.5 * std::f64::consts::PI / 180.0;
    
    for _pass in 0..2 {
        let mut best_pass_angle = current_center;
        
        for i in 0..16 {
            let angle = current_center - 8.0 * step + (i as f64) * step;
            
            let mut total_repulsion = 0.0;
            
            // Calculate trial positions
            for atom in &template.atoms {
                if atom.id == consumed_atom_id { continue; }
                
                let local = sub(atom.position, consumed_pos);
                let rotated_base = rotate_vec(base_rotation, local);
                let rotated = rotate(rotated_base, target_vec, angle);
                let trial_pos = add(rotated, sub(target_retained_pos, rotate_vec(base_rotation, sub(retained_pos, consumed_pos))));
                
                for mol_atom in &molecule.atoms {
                    total_repulsion += crate::geometry::repulsion_potential(
                        trial_pos, 
                        atom.element, 
                        mol_atom.position, 
                        mol_atom.element
                    );
                }
            }
            
            if total_repulsion < min_repulsion {
                min_repulsion = total_repulsion;
                best_pass_angle = angle;
            }
        }
        current_center = best_pass_angle;
        step /= 16.0; // Refine step
        best_angle = current_center;
    }

    // Apply best rotation and placement

    // Apply best rotation and placement
    for atom in &mut template.atoms {
        let local = sub(atom.position, consumed_pos);
        let rotated_base = rotate_vec(base_rotation, local);
        let rotated = rotate(rotated_base, target_vec, best_angle);
        
        let shift = sub(target_consumed_pos, rotate_vec(base_rotation, sub(retained_pos, consumed_pos)));
        atom.position = add(rotated, shift);
    }

    // ------------------------------------------------------------
    // Prepare ID mapping
    //
    // consumed fragment atom is NOT copied
    // ------------------------------------------------------------

    let mut mapping: HashMap<u32, u32> = HashMap::new();

    let mut next_atom = next_atom_id(molecule);
    let mut next_bond = next_bond_id(molecule);

    for atom in &template.atoms {
        if atom.id == consumed_atom_id {
            continue;
        }

        mapping.insert(atom.id, next_atom);

        molecule.atoms.push(Atom {
            id: next_atom,
            element: atom.element,
            position: atom.position,
            isotope: atom.isotope,
            nuclear_spin: atom.nuclear_spin,
            formal_charge: atom.formal_charge,
        });

        next_atom += 1;
    }

    // ------------------------------------------------------------
    // Add fragment bonds except port bond
    // ------------------------------------------------------------

    for bond in &template.bonds {
        let a = bond.atom_ids[0];
        let b = bond.atom_ids[1];

        let is_port_bond = (a == consumed_atom_id && b == retained_atom_id)
            || (a == retained_atom_id && b == consumed_atom_id);

        if is_port_bond {
            continue;
        }

        // Skip bonds involving consumed atom
        if a == consumed_atom_id || b == consumed_atom_id {
            continue;
        }

        let Some(&mapped_a) = mapping.get(&a) else {
            continue;
        };

        let Some(&mapped_b) = mapping.get(&b) else {
            continue;
        };

        molecule.bonds.push(Bond {
            id: next_bond,
            atom_ids: [mapped_a, mapped_b],
            order: bond.order,
        });

        next_bond += 1;
    }

    // ------------------------------------------------------------
    // Remove target bond A -- B
    // ------------------------------------------------------------

    molecule.bonds.remove(target_bond_index);

    // ------------------------------------------------------------
    // Create new bond:
    //
    // B -- retained(fragment)
    // ------------------------------------------------------------

    let Some(&mapped_retained_atom) = mapping.get(&retained_atom_id) else {
        return;
    };

    molecule.bonds.push(Bond {
        id: next_bond,
        atom_ids: [end_atom_id, mapped_retained_atom],
        order: 1,
    });

    // ------------------------------------------------------------
    // Remove target start atom A
    // ------------------------------------------------------------

    let removed_atom_id = start_atom_id;

    molecule
        .bonds
        .retain(|b| b.atom_ids[0] != removed_atom_id && b.atom_ids[1] != removed_atom_id);

    molecule.atoms.retain(|a| a.id != removed_atom_id);

    // ------------------------------------------------------------
    // Optional cleanup:
    // remove isolated atoms
    // ------------------------------------------------------------

    let connected: HashSet<u32> = molecule
        .bonds
        .iter()
        .flat_map(|b| [b.atom_ids[0], b.atom_ids[1]])
        .collect();

    molecule.atoms.retain(|a| connected.contains(&a.id));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn atom_position_for(state: &AppState, atom_id: u32) -> [f64; 3] {
        state
            .domain
            .chemical_spec
            .molecule
            .atoms
            .iter()
            .find(|atom| atom.id == atom_id)
            .expect("atom should exist")
            .position
    }

    #[test]
    fn deserializes_geometry_commands_from_frontend_shape() {
        let command: Command =
            serde_json::from_str(r#"{"type":"SET_BOND_LENGTH","atomIds":[1,2],"length":1.42}"#)
                .expect("command should deserialize");

        assert!(matches!(
            command,
            Command::SetBondLength {
                atom_ids: [1, 2],
                length,
                ..
            } if (length - 1.42).abs() < 1e-12
        ));
    }

    #[test]
    fn bond_length_command_updates_coordinates() {
        let state = reduce(
            initial_app_state(),
            Command::SetBondLength {
                atom_ids: [1, 2],
                length: 1.42,
                mode: GeometryEditMode::AtomOnly,
            },
        );

        assert!(
            (distance(atom_position_for(&state, 1), atom_position_for(&state, 2)) - 1.42).abs()
                < 1e-9
        );
    }

    fn angle_degrees(a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> Option<f64> {
        let ba = sub(a, b);
        let bc = sub(c, b);
        let denominator = length(ba) * length(bc);
        if denominator <= f64::EPSILON {
            return None;
        }
        Some(
            (dot(ba, bc) / denominator)
                .clamp(-1.0, 1.0)
                .acos()
                .to_degrees(),
        )
    }

    #[test]
    fn bond_angle_command_updates_coordinates() {
        let state = reduce(
            initial_app_state(),
            Command::SetBondAngle {
                atom_ids: [2, 1, 3],
                angle: 120.0,
                mode: GeometryEditMode::AtomOnly,
            },
        );
        let first = atom_position_for(&state, 2);
        let center = atom_position_for(&state, 1);
        let third = atom_position_for(&state, 3);
        let measured = angle_degrees(first, center, third).expect("angle should be measurable");

        assert!((measured - 120.0).abs() < 1e-9);
    }


    fn build_four_atom_chain() -> Molecule {
        Molecule {
            name: "chain".to_string(),
            atoms: vec![
                Atom { id: 1, element: Element::C, isotope: None, nuclear_spin: None, position: [0.0, 0.0, 0.0] },
                Atom { id: 2, element: Element::C, isotope: None, nuclear_spin: None, position: [1.0, 0.0, 0.0] },
                Atom { id: 3, element: Element::C, isotope: None, nuclear_spin: None, position: [2.0, 1.0, 0.0] },
                Atom { id: 4, element: Element::C, isotope: None, nuclear_spin: None, position: [3.0, 1.0, 1.0] },
            ],
            bonds: vec![
                Bond { id: 1, atom_ids: [1, 2], order: 1 },
                Bond { id: 2, atom_ids: [2, 3], order: 1 },
                Bond { id: 3, atom_ids: [3, 4], order: 1 },
            ],
        }
    }


    #[test]
    fn dihedral_angle_command_updates_coordinates() {
        let state = reduce(
            initial_app_state(),
            Command::SetMolecule { molecule: build_four_atom_chain() },
        );

        let state = reduce(
            state,
            Command::SetDihedralAngle {
                atom_ids: [1, 2, 3, 4],
                angle: 60.0,
                mode: GeometryEditMode::AtomOnly,
            },
        );

        let a = atom_position_for(&state, 1);
        let b = atom_position_for(&state, 2);
        let c = atom_position_for(&state, 3);
        let d = atom_position_for(&state, 4);
        let measured = dihedral_degrees(a, b, c, d).expect("dihedral should be measurable");

        assert!((measured - 60.0).abs() < 1e-7);
    }

    #[test]
    fn add_atom_preserves_isotope_and_nuclear_spin() {
        let state = reduce(
            initial_app_state(),
            Command::AddAtom {
                element: Element::C,
                position: [1.0, 2.0, 3.0],
                isotope: Some(MassNumber(13)),
                nuclear_spin: Some(TwiceSpin(1)),
                formal_charge: -1,
            },
        );
        let atom = state
            .domain
            .chemical_spec
            .molecule
            .atoms
            .iter()
            .find(|atom| atom.id == 4)
            .expect("new atom should exist");

        assert_eq!(atom.element, Element::C);
        assert_eq!(atom.isotope, Some(MassNumber(13)));
        assert_eq!(atom.nuclear_spin, Some(TwiceSpin(1)));
        assert_eq!(atom.formal_charge, -1);
        assert_eq!(atom.position, [1.0, 2.0, 3.0]);
    }

    #[test]
    fn set_atom_formal_charge_updates_selected_atom() {
        let state = reduce(
            initial_app_state(),
            Command::SetAtomFormalCharge {
                atom_id: 1,
                formal_charge: 1,
            },
        );

        assert_eq!(
            state.domain.chemical_spec.molecule.atoms[0].formal_charge,
            1
        );
    }

    #[test]
    fn delete_atom_removes_connected_bonds_and_selection() {
        let state = reduce(
            initial_app_state(),
            Command::ToggleAtomSelection { atom_id: 2 },
        );
        let state = reduce(state, Command::DeleteAtom { atom_id: 2 });
        let molecule = &state.domain.chemical_spec.molecule;

        assert!(molecule.atoms.iter().all(|atom| atom.id != 2));
        assert!(molecule
            .bonds
            .iter()
            .all(|bond| !bond.atom_ids.contains(&2)));
        assert!(state.ui.selected_atoms.is_empty());
    }

    #[test]
    fn add_bond_rejects_duplicate_bonds() {
        let state = reduce(
            initial_app_state(),
            Command::AddBond {
                atom_ids: [2, 1],
                order: 2,
            },
        );

        assert_eq!(state.domain.chemical_spec.molecule.bonds.len(), 2);
    }

    #[test]
    fn place_template_inserts_atoms_and_bonds() {
        let state = reduce(
            initial_app_state(),
            Command::PlaceTemplate {
                template_name: "methane".to_string(),
                position: [10.0, 0.0, 0.0],
                direction: [0.0, 0.0, 1.0],
            },
        );
        let molecule = &state.domain.chemical_spec.molecule;
        assert_eq!(molecule.atoms.len(), 3 + 5); // 3 original + 5 from methane
        assert!(molecule
            .atoms
            .iter()
            .any(|a| a.element == Element::C && a.position == [10.0, 0.0, 0.0]));
    }

    #[test]
    fn substitute_by_fragment_removes_bond_and_connects_fragment() {
        // Initial state has a bond between 1 and 2
        let state = initial_app_state();
        // Assuming we have a fragment "methyl" that has a bond-type port
        // The implementation finds a bond-type port and connects to start_atom_id/end_atom_id
        let state = reduce(
            state,
            Command::SubstituteByFragment {
                fragment_name: "methyl".to_string(),
                start_atom_id: 2, // Replace atom 2, keep atom 1
                end_atom_id: 1,
            },
        );
        let molecule = &state.domain.chemical_spec.molecule;

        // Original bonds were: 1-2, 1-3. After substitution:
        // Bond 1-2 is removed.
        // New bonds from fragment added.
        // Need to verify bond 1-2 is gone.
        assert!(molecule
            .bonds
            .iter()
            .all(|b| !(b.atom_ids[0] == 1 && b.atom_ids[1] == 2)
                && !(b.atom_ids[0] == 2 && b.atom_ids[1] == 1)));
        assert_eq!(molecule.atoms.len(), 3 - 1 + 5 - 1); // 3 original + 5 from methane
    }

    #[test]
    fn substitute_by_fragment_removes_deleted_atom_from_selection() {
        let state = reduce(
            initial_app_state(),
            Command::ToggleAtomSelection { atom_id: 1 },
        );
        let state = reduce(state, Command::ToggleAtomSelection { atom_id: 2 });
        let state = reduce(
            state,
            Command::SubstituteByFragment {
                fragment_name: "methyl".to_string(),
                start_atom_id: 2,
                end_atom_id: 1,
            },
        );

        assert_eq!(state.ui.selected_atoms, vec![1]);
    }

    #[test]
    fn replace_atom_updates_element_and_clears_isotope() {
        let mut state = initial_app_state();
        state.domain.chemical_spec.molecule.atoms[0].isotope = Some(MassNumber(18));
        
        let state = reduce(
            state,
            Command::ReplaceAtom {
                atom_id: 1,
                element: Element::N,
            },
        );

        let atom = &state.domain.chemical_spec.molecule.atoms[0];
        assert_eq!(atom.element, Element::N);
        assert_eq!(atom.isotope, None);
    }

    #[test]
    fn infer_substitute_by_fragment_completion_uses_selected_atom_as_start_atom() {
        let molecule = initial_app_state().domain.chemical_spec.molecule;

        assert_eq!(
            infer_substitute_by_fragment_completion(&molecule, 2),
            Some(SubstituteByFragmentCompletion {
                start_atom_id: 2,
                end_atom_id: 1,
            })
        );
    }

    #[test]
    fn infer_substitute_by_fragment_completion_rejects_ambiguous_single_atom() {
        let molecule = initial_app_state().domain.chemical_spec.molecule;

        assert_eq!(infer_substitute_by_fragment_completion(&molecule, 1), None);
    }
}
