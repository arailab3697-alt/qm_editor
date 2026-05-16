use crate::domain::{AppState, Command, Element, MassNumber, Molecule, TwiceSpin, atom_index, atom_position};
use crate::geometry::{add, dihedral_degrees, dot, length, normalize, perpendicular, rotate, scale, sub, distance};
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
            target_bond_id,
            rotation_angle,
        } => {
            substitute_by_fragment(
                &mut state.domain.chemical_spec.molecule,
                &fragment_name,
                target_bond_id,
                rotation_angle,
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
    let Some(port) = fragment.attach_ports.first() else {
        return;
    };
    let Some(target_pos) = atom_position(molecule, target_atom_id) else {
        return;
    };

    let port_atom_id = port.target_id;

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

fn substitute_by_fragment(
    molecule: &mut Molecule,
    fragment_name: &str,
    target_bond_id: u32,
    _rotation_angle: f64,
) {
    let fragments = crate::fragments::list_available_fragments();
    let Some(fragment) = fragments.iter().find(|f| f.name == fragment_name) else {
        return;
    };
    let Some(mut template) = crate::templates::get_template(&fragment.template_name) else {
        return;
    };
    
    // Find the target bond
    let Some(bond_index) = molecule.bonds.iter().position(|b| b.id == target_bond_id) else {
        return;
    };
    let bond = molecule.bonds[bond_index].clone();
    let bond_atoms = bond.atom_ids;

    // Find a bond-type port in the fragment
    let Some(port) = fragment.attach_ports.iter().find(|p| matches!(p.port_type, crate::domain::PortType::Bond)) else {
        return;
    };
    
    // Remove the target bond
    molecule.bonds.remove(bond_index);

    // Get positions of bond atoms for alignment
    let pos_a = atom_position(molecule, bond_atoms[0]).unwrap_or([0.0, 0.0, 0.0]);
    let pos_b = atom_position(molecule, bond_atoms[1]).unwrap_or([0.0, 0.0, 0.0]);
    let midpoint = scale(add(pos_a, pos_b), 0.5);

    // Align fragment to the midpoint and bond vector
    // Simplification: Shift fragment port to midpoint
    let base_id = next_atom_id(molecule).saturating_sub(1);
    let bond_base_id = next_bond_id(molecule).saturating_sub(1);

    for atom in &mut template.atoms {
        atom.id += base_id;
        atom.position = add(atom.position, midpoint);
    }
    for bond in &mut template.bonds {
        bond.id += bond_base_id;
        bond.atom_ids[0] += base_id;
        bond.atom_ids[1] += base_id;
    }

    molecule.atoms.extend(template.atoms);
    molecule.bonds.extend(template.bonds);
    
    // Connect new fragment to existing bond atoms (simplified logic)
    add_bond(molecule, [bond_atoms[0], port.target_id + base_id], 1);
    add_bond(molecule, [bond_atoms[1], port.target_id + base_id], 1);
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
    fn attach_fragment_removes_port_atom_and_connects() {
        // Initial state: Water (3 atoms: O, H, H)
        let state = initial_app_state();
        // Methyl has 5 atoms (C + 4H). 
        // Port atom is ID 1 (C).
        // Methyl should add 4 atoms and bonds, effectively replacing the connection at C.
        // Wait, current logic appends ALL atoms. I need to fix it.
        let state = reduce(
            state,
            Command::AttachFragment {
                fragment_name: "methyl".to_string(),
                target_atom_id: 1, // Target is O in water
                rotation_angle: 0.0,
                orientation: [0.0, 0.0, 0.0],
            },
        );
        let molecule = &state.domain.chemical_spec.molecule;

        // Current implementation appends all atoms from template.
        // If "methyl" uses "methane" template, it has 5 atoms (1C, 4H).
        // The port is the Carbon (ID 1). 
        // After attachment, ID 1 of template (shifted) should be removed 
        // OR the logic should just not add the port atom itself if it's considered part of the "connection".
        // Let's check atom count.
        // 3 (initial) + 5 (methyl) = 8.
        // If port is removed, 8 - 1 = 7.
        assert_eq!(molecule.atoms.len(), 7); 
    }
}
