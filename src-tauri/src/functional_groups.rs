use crate::domain::{same_bond, Atom, Bond, Element, Molecule};
use crate::functional_group_patterns::{
    FunctionalGroupPattern, NeighborQuery, PatternAttachment, PatternElement, PatternRole,
    FUNCTIONAL_GROUP_PATTERNS,
};
use serde::Serialize;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum FunctionalGroupKind {
    BenzeneRing,
    CarboxylicAcid,
    SulfonicAcid,
    Amide,
    Nitrile,
    Ester,
    Aldehyde,
    Ketone,
    Alcohol,
    Amine,
    Alkene,
    Alkyne,
    Ether,
    Halogen,
    Nitro,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FunctionalGroupMatch {
    pub kind: FunctionalGroupKind,
    pub atom_ids: Vec<u32>,
    pub attachment_atom_id: Option<u32>,
    pub reference_atom_id: Option<u32>,
}

pub fn match_functional_groups(molecule: &Molecule) -> Vec<FunctionalGroupMatch> {
    let mut matches = Vec::new();
    for ring in find_benzene_rings(molecule) {
        matches.push(FunctionalGroupMatch {
            kind: FunctionalGroupKind::BenzeneRing,
            atom_ids: ring.to_vec(),
            attachment_atom_id: None,
            reference_atom_id: None,
        });
    }

    for pattern in FUNCTIONAL_GROUP_PATTERNS.iter() {
        matches.extend(match_pattern(molecule, pattern));
    }

    dedupe_matches(matches)
}

pub fn find_all_benzene_rings(molecule: &Molecule) -> Vec<Vec<u32>> {
    find_benzene_rings(molecule)
        .into_iter()
        .map(|ring| ring.to_vec())
        .collect()
}

pub fn ordered_benzene_ring_carbons(molecule: &Molecule) -> Option<Vec<u32>> {
    find_benzene_rings(molecule)
        .into_iter()
        .filter_map(|ring| {
            let ordered = order_benzene_ring_from_principal_group(molecule, ring)?;
            let score = numbering_score(molecule, &ordered);
            Some((score, ordered))
        })
        .min_by(|(left_score, left_ordered), (right_score, right_ordered)| {
            left_score
                .cmp(right_score)
                .then_with(|| left_ordered.cmp(right_ordered))
        })
        .map(|(_, ordered)| ordered)
}

fn order_benzene_ring_from_principal_group(
    molecule: &Molecule,
    ring: [u32; 6],
) -> Option<Vec<u32>> {
    let ring_set = ring.into_iter().collect::<HashSet<_>>();
    let best_start = ring
        .iter()
        .copied()
        .map(|atom_id| {
            (
                best_substituent_priority_at_ring_atom(molecule, &ring_set, atom_id),
                atom_id,
            )
        })
        .min_by(|(left_priority, left_atom), (right_priority, right_atom)| {
            left_priority
                .cmp(right_priority)
                .then_with(|| left_atom.cmp(right_atom))
        })?;

    if best_start.0 == u8::MAX {
        return None;
    }

    let cycle = rotate_cycle_to_start(&ring, best_start.1)?;
    let mut reversed = cycle.clone();
    reversed[1..].reverse();
    if numbering_score(molecule, &reversed) < numbering_score(molecule, &cycle) {
        Some(reversed)
    } else {
        Some(cycle)
    }
}

fn numbering_score(molecule: &Molecule, ordered_ring: &[u32]) -> Vec<(u8, usize, u32)> {
    let ring_set = ordered_ring.iter().copied().collect::<HashSet<_>>();
    ordered_ring
        .iter()
        .enumerate()
        .filter_map(|(index, atom_id)| {
            let priority = best_substituent_priority_at_ring_atom(molecule, &ring_set, *atom_id);
            (priority != u8::MAX).then_some((priority, index + 1, *atom_id))
        })
        .collect()
}

fn best_substituent_priority_at_ring_atom(
    molecule: &Molecule,
    ring_set: &HashSet<u32>,
    ring_atom_id: u32,
) -> u8 {
    let mut matches = match_functional_groups(molecule);
    
    // BenzeneRingのマッチを探す
    let benzene_matches: Vec<_> = matches.iter().filter(|m| m.kind == FunctionalGroupKind::BenzeneRing).collect();

    matches
        .into_iter()
        .filter_map(|group_match| {
            let attachment = group_match.attachment_atom_id?;
            
            // ベンゼン環同士の結合を考慮するため、kindのチェックを外す
            if attachment == ring_atom_id
                || group_match.atom_ids.iter().any(|atom_id| {
                    !ring_set.contains(atom_id) && is_bonded(molecule, *atom_id, ring_atom_id)
                })
            {
                Some(functional_group_priority(group_match.kind))
            } else {
                None
            }
        })
        .min()
        .unwrap_or(u8::MAX)
}

fn functional_group_priority(kind: FunctionalGroupKind) -> u8 {
    match kind {
        FunctionalGroupKind::CarboxylicAcid => 0,
        FunctionalGroupKind::SulfonicAcid => 1,
        FunctionalGroupKind::Amide => 2,
        FunctionalGroupKind::Nitrile => 3,
        FunctionalGroupKind::Ester => 4,
        FunctionalGroupKind::Aldehyde => 5,
        FunctionalGroupKind::Ketone => 6,
        FunctionalGroupKind::Alcohol => 7,
        FunctionalGroupKind::Amine => 8,
        FunctionalGroupKind::Alkene => 9,
        FunctionalGroupKind::Alkyne => 10,
        FunctionalGroupKind::Ether => 11,
        FunctionalGroupKind::Halogen => 12,
        FunctionalGroupKind::Nitro => 13,
        FunctionalGroupKind::BenzeneRing => 14, // 最低優先度として扱う
    }
}

fn find_benzene_rings(molecule: &Molecule) -> Vec<[u32; 6]> {
    let carbon_ids = molecule
        .atoms
        .iter()
        .filter(|atom| atom.element == Element::C)
        .map(|atom| atom.id)
        .collect::<Vec<_>>();
    let mut rings = Vec::new();
    let mut seen = HashSet::new();

    for start in carbon_ids {
        let mut path = vec![start];
        find_six_membered_carbon_cycles(molecule, start, start, &mut path, &mut rings, &mut seen);
    }

    rings
}

fn find_six_membered_carbon_cycles(
    molecule: &Molecule,
    start: u32,
    current: u32,
    path: &mut Vec<u32>,
    rings: &mut Vec<[u32; 6]>,
    seen: &mut HashSet<Vec<u32>>,
) {
    if path.len() == 6 {
        if is_bonded(molecule, current, start) && has_benzene_bond_pattern(molecule, path) {
            let mut key = path.clone();
            key.sort_unstable();
            if seen.insert(key) {
                rings.push(path.clone().try_into().expect("path length is fixed"));
            }
        }
        return;
    }

    for neighbor in carbon_neighbors(molecule, current) {
        if neighbor == start || path.contains(&neighbor) {
            continue;
        }
        path.push(neighbor);
        find_six_membered_carbon_cycles(molecule, start, neighbor, path, rings, seen);
        path.pop();
    }
}

fn has_benzene_bond_pattern(molecule: &Molecule, ring: &[u32]) -> bool {
    if ring.len() != 6 {
        return false;
    }
    let ring_bond_orders = (0..6)
        .filter_map(|index| bond_order(molecule, ring[index], ring[(index + 1) % 6]))
        .collect::<Vec<_>>();
    ring_bond_orders.len() == 6
        && ring_bond_orders
            .iter()
            .all(|order| *order == 1 || *order == 2)
        && ring_bond_orders.iter().filter(|order| **order == 2).count() == 3
        && ring_bond_orders
            .iter()
            .enumerate()
            .all(|(index, order)| *order != ring_bond_orders[(index + 1) % 6])
}

fn rotate_cycle_to_start(ring: &[u32; 6], start: u32) -> Option<Vec<u32>> {
    let index = ring.iter().position(|atom_id| *atom_id == start)?;
    Some((0..6).map(|offset| ring[(index + offset) % 6]).collect())
}

fn match_pattern(
    molecule: &Molecule,
    pattern: &FunctionalGroupPattern,
) -> Vec<FunctionalGroupMatch> {
    let mut matches = Vec::new();
    let mut assignment = HashMap::new();
    let mut used_atom_ids = HashSet::new();
    match_pattern_atom(
        molecule,
        pattern,
        0,
        &mut assignment,
        &mut used_atom_ids,
        &mut matches,
    );
    matches
}

fn match_pattern_atom(
    molecule: &Molecule,
    pattern: &FunctionalGroupPattern,
    pattern_atom_index: usize,
    assignment: &mut HashMap<PatternRole, u32>,
    used_atom_ids: &mut HashSet<u32>,
    matches: &mut Vec<FunctionalGroupMatch>,
) {
    if pattern_atom_index == pattern.atoms.len() {
        if let Some(group_match) = build_match_from_assignment(molecule, pattern, assignment) {
            matches.push(group_match);
        }
        return;
    }

    let pattern_atom = &pattern.atoms[pattern_atom_index];
    for atom in molecule
        .atoms
        .iter()
        .filter(|atom| pattern_element_matches(pattern_atom.element, atom))
    {
        if used_atom_ids.contains(&atom.id) {
            continue;
        }
        assignment.insert(pattern_atom.role.clone(), atom.id);
        used_atom_ids.insert(atom.id);
        if partial_assignment_matches_bonds(molecule, pattern, assignment) {
            match_pattern_atom(
                molecule,
                pattern,
                pattern_atom_index + 1,
                assignment,
                used_atom_ids,
                matches,
            );
        }
        used_atom_ids.remove(&atom.id);
        assignment.remove(&pattern_atom.role);
    }
}

fn build_match_from_assignment(
    molecule: &Molecule,
    pattern: &FunctionalGroupPattern,
    assignment: &HashMap<PatternRole, u32>,
) -> Option<FunctionalGroupMatch> {
    if !all_pattern_bonds_match(molecule, pattern, assignment)
        || !all_pattern_constraints_match(molecule, pattern, assignment)
    {
        return None;
    }
    let mut atom_ids = pattern
        .atoms
        .iter()
        .filter_map(|atom| assignment.get(&atom.role).copied())
        .collect::<Vec<_>>();
    atom_ids.sort_unstable();
    atom_ids.dedup();
    Some(FunctionalGroupMatch {
        kind: pattern.kind,
        atom_ids,
        attachment_atom_id: pattern
            .attachment
            .as_ref()
            .and_then(|attachment| resolve_pattern_attachment(molecule, assignment, attachment)),
        reference_atom_id: pattern
            .reference
            .as_ref()
            .and_then(|reference| assignment.get(reference).copied()),
    })
}

fn pattern_element_matches(pattern_element: PatternElement, atom: &Atom) -> bool {
    match pattern_element {
        PatternElement::Exact(element) => atom.element == element,
        PatternElement::AnyHalogen => matches!(
            atom.element,
            Element::F | Element::Cl | Element::Br | Element::I
        ),
    }
}

fn partial_assignment_matches_bonds(
    molecule: &Molecule,
    pattern: &FunctionalGroupPattern,
    assignment: &HashMap<PatternRole, u32>,
) -> bool {
    pattern.bonds.iter().all(|bond| {
        let Some(left) = assignment.get(&bond.left) else {
            return true;
        };
        let Some(right) = assignment.get(&bond.right) else {
            return true;
        };
        bond_order(molecule, *left, *right) == Some(bond.order)
    })
}

fn all_pattern_bonds_match(
    molecule: &Molecule,
    pattern: &FunctionalGroupPattern,
    assignment: &HashMap<PatternRole, u32>,
) -> bool {
    pattern.bonds.iter().all(|bond| {
        let Some(left) = assignment.get(&bond.left) else {
            return false;
        };
        let Some(right) = assignment.get(&bond.right) else {
            return false;
        };
        bond_order(molecule, *left, *right) == Some(bond.order)
    })
}

fn all_pattern_constraints_match(
    molecule: &Molecule,
    pattern: &FunctionalGroupPattern,
    assignment: &HashMap<PatternRole, u32>,
) -> bool {
    pattern.constraints.iter().all(|constraint| {
        neighbor_query_count(molecule, assignment, constraint) >= constraint.min_count
    })
}

fn neighbor_query_count(
    molecule: &Molecule,
    assignment: &HashMap<PatternRole, u32>,
    query: &NeighborQuery,
) -> usize {
    let Some(from_atom_id) = assignment.get(&query.from).copied() else {
        return 0;
    };
    let excluded_atom_ids = query
        .exclude
        .iter()
        .filter_map(|role| assignment.get(role).copied())
        .collect::<HashSet<_>>();

    molecule
        .bonds
        .iter()
        .filter(|bond| {
            bond.atom_ids.contains(&from_atom_id)
                && query
                    .bond_order
                    .is_none_or(|bond_order| bond.order == bond_order)
        })
        .filter_map(|bond| other_atom_id(bond, from_atom_id))
        .filter(|neighbor_id| !excluded_atom_ids.contains(neighbor_id))
        .filter(|neighbor_id| {
            molecule
                .atoms
                .iter()
                .find(|atom| atom.id == *neighbor_id)
                .is_some_and(|atom| pattern_element_matches(query.element, atom))
        })
        .count()
}

fn first_neighbor_query_match(
    molecule: &Molecule,
    assignment: &HashMap<PatternRole, u32>,
    query: &NeighborQuery,
) -> Option<u32> {
    let from_atom_id = assignment.get(&query.from).copied()?;
    let excluded_atom_ids = query
        .exclude
        .iter()
        .filter_map(|role| assignment.get(role).copied())
        .collect::<HashSet<_>>();

    molecule
        .bonds
        .iter()
        .filter(|bond| {
            bond.atom_ids.contains(&from_atom_id)
                && query
                    .bond_order
                    .is_none_or(|bond_order| bond.order == bond_order)
        })
        .filter_map(|bond| other_atom_id(bond, from_atom_id))
        .filter(|neighbor_id| !excluded_atom_ids.contains(neighbor_id))
        .find(|neighbor_id| {
            molecule
                .atoms
                .iter()
                .find(|atom| atom.id == *neighbor_id)
                .is_some_and(|atom| pattern_element_matches(query.element, atom))
        })
}

fn resolve_pattern_attachment(
    molecule: &Molecule,
    assignment: &HashMap<PatternRole, u32>,
    attachment: &PatternAttachment,
) -> Option<u32> {
    match attachment {
        PatternAttachment::Role(role) => assignment.get(role).copied(),
        PatternAttachment::Neighbor(query) => {
            first_neighbor_query_match(molecule, assignment, query)
        }
    }
}

fn dedupe_matches(matches: Vec<FunctionalGroupMatch>) -> Vec<FunctionalGroupMatch> {
    let mut seen = HashSet::new();
    matches
        .into_iter()
        .filter(|group_match| {
            let mut atom_ids = group_match.atom_ids.clone();
            atom_ids.sort_unstable();
            seen.insert((
                group_match.kind as u8,
                atom_ids,
                group_match.attachment_atom_id,
                group_match.reference_atom_id,
            ))
        })
        .collect()
}

pub fn get_ring_neighbors(molecule: &Molecule, ring: &[u32]) -> Vec<(u32, Vec<u32>)> {
    let ring_set: HashSet<u32> = ring.iter().copied().collect();
    ring.iter()
        .map(|&ring_atom_id| {
            let neighbors = molecule
                .bonds
                .iter()
                .filter(|b| b.atom_ids.contains(&ring_atom_id))
                .filter_map(|b| {
                    let neighbor = if b.atom_ids[0] == ring_atom_id { b.atom_ids[1] } else { b.atom_ids[0] };
                    if !ring_set.contains(&neighbor) { Some(neighbor) } else { None }
                })
                .collect();
            (ring_atom_id, neighbors)
        })
        .collect()
}

fn carbon_neighbors(molecule: &Molecule, atom_id: u32) -> Vec<u32> {
    element_neighbors(molecule, atom_id, Element::C)
}

pub fn element_neighbors(molecule: &Molecule, atom_id: u32, element: Element) -> Vec<u32> {
    molecule
        .bonds
        .iter()
        .filter(|bond| bond.atom_ids.contains(&atom_id))
        .filter_map(|bond| other_atom_id(bond, atom_id))
        .filter(|neighbor_id| atom_element(molecule, *neighbor_id) == Some(element))
        .collect()
}

fn is_bonded(molecule: &Molecule, first_atom_id: u32, second_atom_id: u32) -> bool {
    molecule
        .bonds
        .iter()
        .any(|bond| same_bond(bond.atom_ids, [first_atom_id, second_atom_id]))
}

fn bond_order(molecule: &Molecule, first_atom_id: u32, second_atom_id: u32) -> Option<u8> {
    molecule
        .bonds
        .iter()
        .find(|bond| same_bond(bond.atom_ids, [first_atom_id, second_atom_id]))
        .map(|bond| bond.order)
}

fn other_atom_id(bond: &Bond, atom_id: u32) -> Option<u32> {
    if bond.atom_ids[0] == atom_id {
        Some(bond.atom_ids[1])
    } else if bond.atom_ids[1] == atom_id {
        Some(bond.atom_ids[0])
    } else {
        None
    }
}

fn atom_element(molecule: &Molecule, atom_id: u32) -> Option<Element> {
    molecule
        .atoms
        .iter()
        .find(|atom| atom.id == atom_id)
        .map(|atom| atom.element)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Atom;

    fn atom(id: u32, element: Element) -> Atom {
        Atom {
            id,
            element,
            isotope: None,
            nuclear_spin: None,
            formal_charge: 0,
            position: [0.0, 0.0, 0.0],
        }
    }

    fn bond(id: u32, atom_ids: [u32; 2], order: u8) -> Bond {
        Bond {
            id,
            atom_ids,
            order,
        }
    }

    fn benzoic_acid() -> Molecule {
        Molecule {
            name: "benzoic acid".to_string(),
            atoms: vec![
                atom(1, Element::C),
                atom(2, Element::C),
                atom(3, Element::C),
                atom(4, Element::C),
                atom(5, Element::C),
                atom(6, Element::C),
                atom(7, Element::C),
                atom(8, Element::O),
                atom(9, Element::O),
                atom(10, Element::H),
            ],
            bonds: vec![
                bond(1, [1, 2], 2),
                bond(2, [2, 3], 1),
                bond(3, [3, 4], 2),
                bond(4, [4, 5], 1),
                bond(5, [5, 6], 2),
                bond(6, [6, 1], 1),
                bond(7, [1, 7], 1),
                bond(8, [7, 8], 2),
                bond(9, [7, 9], 1),
                bond(10, [9, 10], 1),
            ],
        }
    }

    fn nitro_chlorobenzoic_acid() -> Molecule {
        let mut molecule = benzoic_acid();
        molecule.atoms.extend([
            atom(11, Element::Cl),
            atom(12, Element::N),
            atom(13, Element::O),
            atom(14, Element::O),
        ]);
        molecule.bonds.extend([
            bond(11, [3, 11], 1),
            bond(12, [2, 12], 1),
            bond(13, [12, 13], 2),
            bond(14, [12, 14], 1),
        ]);
        molecule
    }

    #[test]
    fn matches_carboxylic_acid_and_benzene_ring() {
        let matches = match_functional_groups(&benzoic_acid());

        assert!(matches
            .iter()
            .any(|found| found.kind == FunctionalGroupKind::BenzeneRing));
        assert!(matches
            .iter()
            .any(|found| found.kind == FunctionalGroupKind::CarboxylicAcid));
    }

    #[test]
    fn match_includes_reference_atom_for_alpha_position() {
        let matches = match_functional_groups(&benzoic_acid());
        let carboxylic_acid = matches
            .iter()
            .find(|found| found.kind == FunctionalGroupKind::CarboxylicAcid)
            .expect("carboxylic acid should match");

        assert_eq!(carboxylic_acid.reference_atom_id, Some(7));
    }

    #[test]
    fn benzene_ring_order_starts_at_highest_priority_substituent() {
        assert_eq!(
            ordered_benzene_ring_carbons(&nitro_chlorobenzoic_acid()),
            Some(vec![1, 2, 3, 4, 5, 6])
        );
    }

    #[test]
    fn returns_none_without_benzene_ring() {
        let molecule = Molecule {
            name: "ethene".to_string(),
            atoms: vec![atom(1, Element::C), atom(2, Element::C)],
            bonds: vec![bond(1, [1, 2], 2)],
        };

        assert_eq!(ordered_benzene_ring_carbons(&molecule), None);
    }

    #[test]
    fn rejects_non_alternating_six_membered_carbon_ring() {
        let molecule = Molecule {
            name: "non alternating ring".to_string(),
            atoms: (1..=6).map(|id| atom(id, Element::C)).collect(),
            bonds: vec![
                bond(1, [1, 2], 2),
                bond(2, [2, 3], 2),
                bond(3, [3, 4], 2),
                bond(4, [4, 5], 1),
                bond(5, [5, 6], 1),
                bond(6, [6, 1], 1),
            ],
        };

        assert!(find_benzene_rings(&molecule).is_empty());
    }
}
