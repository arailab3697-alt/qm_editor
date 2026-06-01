use super::domain::{
    atom_oxidation_number, electronegativity, ChemicalSpec, Element, ValidationLevel,
    ValidationMessage,
};

pub fn validate_chemical_spec(spec: &ChemicalSpec) -> Vec<ValidationMessage> {
    let mut messages = Vec::new();
    let molecule = &spec.molecule;
    let calculation = &spec.calculation;

    if molecule.atoms.is_empty() {
        messages.push(error("Molecule must contain at least one atom."));
    }

    if calculation.multiplicity < 1 {
        messages.push(error("Multiplicity must be a positive integer."));
    }

    let formal_charge_sum = molecule
        .atoms
        .iter()
        .map(|atom| atom.formal_charge)
        .sum::<i32>();
    if !molecule.atoms.is_empty() && formal_charge_sum != calculation.charge {
        messages.push(warning(&format!(
            "Sum of atom formal charges ({formal_charge_sum}) differs from total charge ({}).",
            calculation.charge
        )));
    }

    let charge_parity = calculation.charge.unsigned_abs() % 2;
    let electron_parity = molecule.atoms.iter().fold(charge_parity, |parity, atom| {
        (parity + valence_parity(atom.element)) % 2
    });
    let unpaired_parity = (calculation.multiplicity - 1) % 2;
    if !molecule.atoms.is_empty() && electron_parity != unpaired_parity {
        messages.push(warning(
            "Charge and multiplicity look inconsistent for common valence parity.",
        ));
    }

    for (index, atom) in molecule.atoms.iter().enumerate() {
        if electronegativity(atom.element).is_none() {
            continue;
        }
        let Some(oxidation_number) = atom_oxidation_number(molecule, atom.id) else {
            continue;
        };
        if !usual_oxidation_numbers(atom.element).contains(&oxidation_number) {
            messages.push(warning(&format!(
                "Atom {} ({:?}) has unusual oxidation number {oxidation_number}.",
                index + 1,
                atom.element
            )));
        }
    }

    messages
}

fn error(message: &str) -> ValidationMessage {
    ValidationMessage {
        level: ValidationLevel::Error,
        message: message.to_string(),
    }
}

fn warning(message: &str) -> ValidationMessage {
    ValidationMessage {
        level: ValidationLevel::Warning,
        message: message.to_string(),
    }
}

fn valence_parity(element: Element) -> u32 {
    match element {
        Element::H
        | Element::B
        | Element::N
        | Element::F
        | Element::P
        | Element::Cl
        | Element::Br
        | Element::I => 1,
        _ => 0,
    }
}

fn usual_oxidation_numbers(element: Element) -> &'static [i32] {
    match element {
        Element::H => &[-1, 1],
        Element::Li => &[1],
        Element::Be => &[2],
        Element::B => &[3],
        Element::C => &[-4, -3, -2, -1, 0, 1, 2, 3, 4],
        Element::N => &[-3, -2, -1, 0, 1, 2, 3, 4, 5],
        Element::O => &[-2, -1, 0, 1, 2],
        Element::F => &[-1],
        Element::Cl | Element::Br | Element::I => &[-1, 0, 1, 3, 5, 7],
        _ => &[],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Atom, Bond, CalculationSpec, JobType, Method, Molecule};

    fn spec_for(molecule: Molecule) -> ChemicalSpec {
        ChemicalSpec {
            molecule,
            calculation: CalculationSpec {
                job_type: JobType::Opt,
                method: Method::B3LYP,
                basis: crate::domain::Basis::Six31Gd,
                solvent: None,
                charge: 0,
                multiplicity: 1,
            },
        }
    }

    #[test]
    fn oxidation_number_uses_electronegativity_and_bond_order() {
        let molecule = Molecule {
            name: "water".to_string(),
            atoms: vec![
                Atom {
                    id: 1,
                    element: Element::O,
                    isotope: None,
                    nuclear_spin: None,
                    formal_charge: 0,
                    position: [0.0, 0.0, 0.0],
                },
                Atom {
                    id: 2,
                    element: Element::H,
                    isotope: None,
                    nuclear_spin: None,
                    formal_charge: 0,
                    position: [0.0, 0.0, 0.0],
                },
                Atom {
                    id: 3,
                    element: Element::H,
                    isotope: None,
                    nuclear_spin: None,
                    formal_charge: 0,
                    position: [0.0, 0.0, 0.0],
                },
            ],
            bonds: vec![
                Bond {
                    id: 1,
                    atom_ids: [1, 2],
                    order: 1,
                },
                Bond {
                    id: 2,
                    atom_ids: [1, 3],
                    order: 1,
                },
            ],
        };

        assert_eq!(atom_oxidation_number(&molecule, 1), Some(-2));
        assert_eq!(atom_oxidation_number(&molecule, 2), Some(1));
    }

    #[test]
    fn validator_warns_for_unusual_oxidation_number() {
        let molecule = Molecule {
            name: "oxygen fluoride".to_string(),
            atoms: vec![
                Atom {
                    id: 1,
                    element: Element::O,
                    isotope: None,
                    nuclear_spin: None,
                    formal_charge: 0,
                    position: [0.0, 0.0, 0.0],
                },
                Atom {
                    id: 2,
                    element: Element::F,
                    isotope: None,
                    nuclear_spin: None,
                    formal_charge: 0,
                    position: [0.0, 0.0, 0.0],
                },
            ],
            bonds: vec![Bond {
                id: 1,
                atom_ids: [1, 2],
                order: 3,
            }],
        };

        let messages = validate_chemical_spec(&spec_for(molecule));

        assert!(messages
            .iter()
            .any(|message| message.message.contains("unusual oxidation number 3")));
    }
}
