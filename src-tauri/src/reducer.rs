use crate::domain::{AppState, Command, Element, MassNumber, Molecule, TwiceSpin, atom_index, atom_position, Atom, Bond};
use crate::geometry::{add, dihedral_degrees, dot, length, normalize, perpendicular, rotate, scale, sub, distance, rotation_from_to, rotate_vec};
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
                            position: [0.0, 0.0, 0.0],
                        },
                        crate::domain::Atom {
                            id: 2,
                            element: Element::H,
                            isotope: None,
                            nuclear_spin: None,
                            position: [0.758, 0.586, 0.0],
                        },
                        crate::domain::Atom {
                            id: 3,
                            element: Element::H,
                            isotope: None,
                            nuclear_spin: None,
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
        Command::SetBondLength { atom_ids, length } => {
            set_bond_length(&mut state.domain.chemical_spec.molecule, atom_ids, length);
        }
        Command::SetBondAngle { atom_ids, angle } => {
            set_bond_angle(&mut state.domain.chemical_spec.molecule, atom_ids, angle);
        }
        Command::SetDihedralAngle { atom_ids, angle } => {
            set_dihedral_angle(&mut state.domain.chemical_spec.molecule, atom_ids, angle);
        }
        Command::AddAtom {
            element,
            position,
            isotope,
            nuclear_spin,
        } => add_atom(
            &mut state.domain.chemical_spec.molecule,
            element,
            position,
            isotope,
            nuclear_spin,
        ),
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
        Command::ClearSelection => state.ui.selected_atoms.clear(),
    }
    state
}

fn set_bond_length(molecule: &mut Molecule, atom_ids: [u32; 2], length: f64) {
    if !length.is_finite() || length <= 0.0 {
        return;
    }
    let Some(anchor) = atom_position(molecule, atom_ids[0]) else {
        return;
    };
    let Some(moving_index) = atom_index(molecule, atom_ids[1]) else {
        return;
    };
    let direction = sub(molecule.atoms[moving_index].position, anchor);
    let Some(unit) = normalize(direction) else {
        return;
    };
    molecule.atoms[moving_index].position = add(anchor, scale(unit, length));
}

fn set_bond_angle(molecule: &mut Molecule, atom_ids: [u32; 3], angle: f64) {
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
    molecule.atoms[moving_index].position = add(center, new_vector);
}

fn set_dihedral_angle(molecule: &mut Molecule, atom_ids: [u32; 4], angle: f64) {
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
    molecule.atoms[moving_index].position = add(third, rotate(sub(moving, third), axis, delta));
}

fn add_atom(
    molecule: &mut Molecule,
    element: Element,
    position: [f64; 3],
    isotope: Option<MassNumber>,
    nuclear_spin: Option<TwiceSpin>,
) {
    if !position.iter().all(|coordinate| coordinate.is_finite()) {
        return;
    }
    molecule.atoms.push(crate::domain::Atom {
        id: next_atom_id(molecule),
        element,
        isotope,
        nuclear_spin,
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
        .filter_map(|p| if let crate::domain::PortType::Atom { target_id } = p.port_type { Some(target_id) } else { None })
        .next() else {
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

pub fn substitute_by_fragment(
    molecule: &mut Molecule,
    fragment_name: &str,
    start_atom_id: u32,
    end_atom_id: u32,
) {
    use std::collections::{HashMap, HashSet};

    let fragments = crate::fragments::list_available_fragments();
    println!("substitute by fragment");
    println!("{:?}, {:?}", fragments, fragment_name);

    let Some(fragment) = fragments.iter().find(|f| f.name == fragment_name) else {
        return;
    };

    let Some(mut template) = crate::templates::get_template(&fragment.template_name) else {
        return;
    };

    // ------------------------------------------------------------
    // Find target bond A(start) -- B(end)
    // ------------------------------------------------------------

    let Some(target_bond_index) = molecule.bonds.iter().position(|b|
        (b.atom_ids[0] == start_atom_id && b.atom_ids[1] == end_atom_id)
            || (b.atom_ids[0] == end_atom_id && b.atom_ids[1] == start_atom_id)
    ) else {
        return;
    };

    // ------------------------------------------------------------
    // Find fragment port bond X -- Y
    //
    // consumed_atom = X
    // retained_atom = Y
    // ------------------------------------------------------------

    let Some((consumed_atom_id, retained_atom_id)) = fragment
        .attach_ports
        .iter()
        .find_map(|p| {
            if let crate::domain::PortType::Bond {
                start_atom_id,
                end_atom_id,
            } = p.port_type
            {
                Some((start_atom_id, end_atom_id))
            } else {
                None
            }
        })
    else {
        return;
    };

    // ------------------------------------------------------------
    // Get positions
    // ------------------------------------------------------------

    let Some(target_start_pos) = atom_position(molecule, start_atom_id) else {
        return;
    };

    let Some(target_end_pos) = atom_position(molecule, end_atom_id) else {
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
    // consumed -> retained
    // to
    // target_start -> target_end
    // ------------------------------------------------------------

    let target_vec = normalize(sub(target_end_pos, target_start_pos)).expect("target bond must be valid");
    let fragment_vec = normalize(sub(retained_pos, consumed_pos)).expect("fragment bond must be valid");

    let rotation = rotation_from_to(fragment_vec, target_vec);

    // Rotate around consumed atom
    for atom in &mut template.atoms {
        let local = sub(atom.position, consumed_pos);

        let rotated = rotate_vec(rotation, local);

        atom.position = add(rotated, consumed_pos);
    }

    // Translate consumed atom onto target_start
    let transformed_consumed_pos = template
        .atoms
        .iter()
        .find(|a| a.id == consumed_atom_id)
        .map(|a| a.position)
        .unwrap_or(consumed_pos);

    let shift = sub(target_start_pos, transformed_consumed_pos);

    for atom in &mut template.atoms {
        atom.position = add(atom.position, shift);
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
            element: atom.element.clone(),
            position: atom.position,
            isotope: None,
            nuclear_spin: None,
        });

        next_atom += 1;
    }

    // ------------------------------------------------------------
    // Add fragment bonds except port bond
    // ------------------------------------------------------------

    for bond in &template.bonds {
        let a = bond.atom_ids[0];
        let b = bond.atom_ids[1];

        let is_port_bond =
            (a == consumed_atom_id && b == retained_atom_id)
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
    // A -- retained(fragment)
    // ------------------------------------------------------------

    let Some(&mapped_retained_atom) = mapping.get(&retained_atom_id) else {
        return;
    };

    molecule.bonds.push(Bond {
        id: next_bond,
        atom_ids: [start_atom_id, mapped_retained_atom],
        order: 1,
    });

    // ------------------------------------------------------------
    // Remove target end atom B
    // ------------------------------------------------------------

    let removed_atom_id = end_atom_id;

    molecule.bonds.retain(|b| {
        b.atom_ids[0] != removed_atom_id
            && b.atom_ids[1] != removed_atom_id
    });

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

    molecule
        .atoms
        .retain(|a| connected.contains(&a.id));
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
                length
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
            },
        );
        let first = atom_position_for(&state, 2);
        let center = atom_position_for(&state, 1);
        let third = atom_position_for(&state, 3);
        let measured = angle_degrees(first, center, third).expect("angle should be measurable");

        assert!((measured - 120.0).abs() < 1e-9);
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
        assert_eq!(atom.position, [1.0, 2.0, 3.0]);
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
        assert!(molecule.atoms.iter().any(|a| a.element == Element::C && a.position == [10.0, 0.0, 0.0]));
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
                start_atom_id: 1, // Bond between 1 and 2
                end_atom_id: 2,
            },
        );
        let molecule = &state.domain.chemical_spec.molecule;

        // Original bonds were: 1-2, 1-3. After substitution:
        // Bond 1-2 removed (length 1).
        // New bonds from fragment added.
        // Need to verify bond 1-2 is gone.
        assert!(molecule.bonds.iter().all(|b| !(b.atom_ids[0] == 1 && b.atom_ids[1] == 2) && !(b.atom_ids[0] == 2 && b.atom_ids[1] == 1)));
        assert_eq!(molecule.atoms.len(), 3 - 1 + 5 - 1); // 3 original + 5 from methane
    }
}
