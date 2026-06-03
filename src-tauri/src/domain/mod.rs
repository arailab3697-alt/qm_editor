use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppState {
    pub domain: DomainState,
    pub ui: UiState,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DomainState {
    pub chemical_spec: ChemicalSpec,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UiState {
    pub selected_atoms: Vec<u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChemicalSpec {
    pub molecule: Molecule,
    pub calculation: CalculationSpec,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Molecule {
    pub name: String,
    pub atoms: Vec<Atom>,
    pub bonds: Vec<Bond>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Atom {
    pub id: u32,
    pub element: Element,
    pub isotope: Option<MassNumber>,
    pub nuclear_spin: Option<TwiceSpin>,
    #[serde(default)]
    pub formal_charge: i32,
    pub position: [f64; 3],
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bond {
    pub id: u32,
    pub atom_ids: [u32; 2],
    pub order: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Electronegativity(pub f64);

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(transparent)]
pub struct MassNumber(pub u16);

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(transparent)]
pub struct TwiceSpin(pub u8);

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Element {
    H,
    #[serde(rename = "He")]
    He,
    #[serde(rename = "Li")]
    Li,
    #[serde(rename = "Be")]
    Be,
    B,
    C,
    N,
    O,
    F,
    #[serde(rename = "Ne")]
    Ne,
    #[serde(rename = "Na")]
    Na,
    #[serde(rename = "Mg")]
    Mg,
    #[serde(rename = "Al")]
    Al,
    #[serde(rename = "Si")]
    Si,
    P,
    S,
    #[serde(rename = "Cl")]
    Cl,
    #[serde(rename = "Ar")]
    Ar,
    K,
    #[serde(rename = "Ca")]
    Ca,
    #[serde(rename = "Sc")]
    Sc,
    #[serde(rename = "Ti")]
    Ti,
    V,
    #[serde(rename = "Cr")]
    Cr,
    #[serde(rename = "Mn")]
    Mn,
    #[serde(rename = "Fe")]
    Fe,
    #[serde(rename = "Co")]
    Co,
    #[serde(rename = "Ni")]
    Ni,
    #[serde(rename = "Cu")]
    Cu,
    #[serde(rename = "Zn")]
    Zn,
    #[serde(rename = "Ga")]
    Ga,
    #[serde(rename = "Ge")]
    Ge,
    #[serde(rename = "As")]
    As,
    #[serde(rename = "Se")]
    Se,
    #[serde(rename = "Br")]
    Br,
    #[serde(rename = "Kr")]
    Kr,
    #[serde(rename = "Rb")]
    Rb,
    #[serde(rename = "Sr")]
    Sr,
    Y,
    #[serde(rename = "Zr")]
    Zr,
    #[serde(rename = "Nb")]
    Nb,
    #[serde(rename = "Mo")]
    Mo,
    #[serde(rename = "Tc")]
    Tc,
    #[serde(rename = "Ru")]
    Ru,
    #[serde(rename = "Rh")]
    Rh,
    #[serde(rename = "Pd")]
    Pd,
    #[serde(rename = "Ag")]
    Ag,
    #[serde(rename = "Cd")]
    Cd,
    #[serde(rename = "In")]
    In,
    #[serde(rename = "Sn")]
    Sn,
    #[serde(rename = "Sb")]
    Sb,
    #[serde(rename = "Te")]
    Te,
    I,
    #[serde(rename = "Xe")]
    Xe,
    #[serde(rename = "Cs")]
    Cs,
    #[serde(rename = "Ba")]
    Ba,
    #[serde(rename = "La")]
    La,
    #[serde(rename = "Ce")]
    Ce,
    #[serde(rename = "Pr")]
    Pr,
    #[serde(rename = "Nd")]
    Nd,
    #[serde(rename = "Pm")]
    Pm,
    #[serde(rename = "Sm")]
    Sm,
    #[serde(rename = "Eu")]
    Eu,
    #[serde(rename = "Gd")]
    Gd,
    #[serde(rename = "Tb")]
    Tb,
    #[serde(rename = "Dy")]
    Dy,
    #[serde(rename = "Ho")]
    Ho,
    #[serde(rename = "Er")]
    Er,
    #[serde(rename = "Tm")]
    Tm,
    #[serde(rename = "Yb")]
    Yb,
    #[serde(rename = "Lu")]
    Lu,
    #[serde(rename = "Hf")]
    Hf,
    #[serde(rename = "Ta")]
    Ta,
    W,
    #[serde(rename = "Re")]
    Re,
    #[serde(rename = "Os")]
    Os,
    #[serde(rename = "Ir")]
    Ir,
    #[serde(rename = "Pt")]
    Pt,
    #[serde(rename = "Au")]
    Au,
    #[serde(rename = "Hg")]
    Hg,
    #[serde(rename = "Tl")]
    Tl,
    #[serde(rename = "Pb")]
    Pb,
    #[serde(rename = "Bi")]
    Bi,
    #[serde(rename = "Po")]
    Po,
    #[serde(rename = "At")]
    At,
    #[serde(rename = "Rn")]
    Rn,
    #[serde(rename = "Fr")]
    Fr,
    #[serde(rename = "Ra")]
    Ra,
    #[serde(rename = "Ac")]
    Ac,
    #[serde(rename = "Th")]
    Th,
    #[serde(rename = "Pa")]
    Pa,
    U,
    #[serde(rename = "Np")]
    Np,
    #[serde(rename = "Pu")]
    Pu,
    #[serde(rename = "Am")]
    Am,
    #[serde(rename = "Cm")]
    Cm,
    #[serde(rename = "Bk")]
    Bk,
    #[serde(rename = "Cf")]
    Cf,
    #[serde(rename = "Es")]
    Es,
    #[serde(rename = "Fm")]
    Fm,
    #[serde(rename = "Md")]
    Md,
    #[serde(rename = "No")]
    No,
    #[serde(rename = "Lr")]
    Lr,
    #[serde(rename = "Rf")]
    Rf,
    #[serde(rename = "Db")]
    Db,
    #[serde(rename = "Sg")]
    Sg,
    #[serde(rename = "Bh")]
    Bh,
    #[serde(rename = "Hs")]
    Hs,
    #[serde(rename = "Mt")]
    Mt,
    #[serde(rename = "Ds")]
    Ds,
    #[serde(rename = "Rg")]
    Rg,
    #[serde(rename = "Cn")]
    Cn,
    #[serde(rename = "Nh")]
    Nh,
    #[serde(rename = "Fl")]
    Fl,
    #[serde(rename = "Mc")]
    Mc,
    #[serde(rename = "Lv")]
    Lv,
    #[serde(rename = "Ts")]
    Ts,
    #[serde(rename = "Og")]
    Og,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalculationSpec {
    pub job_type: JobType,
    pub method: Method,
    pub basis: Basis,
    pub solvent: Option<Solvent>,
    pub charge: i32,
    pub multiplicity: u32,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum JobType {
    #[serde(rename = "opt")]
    Opt,
    #[serde(rename = "freq")]
    Freq,
    #[serde(rename = "opt+freq")]
    OptFreq,
    #[serde(rename = "ts")]
    Ts,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Method {
    B3LYP,
    WB97XD,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Basis {
    #[serde(rename = "6-31G(d)")]
    Six31Gd,
    #[serde(rename = "def2-SVP")]
    Def2Svp,
    #[serde(rename = "def2-TZVP")]
    Def2Tzvp,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Solvent {
    THF,
    Water,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PortType {
    #[serde(rename_all = "camelCase")]
    Atom { target_id: u32 },
    #[serde(rename_all = "camelCase")]
    Bond {
        start_atom_id: u32,
        end_atom_id: u32,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachPort {
    pub id: String,
    pub port_type: PortType,
    pub recommended_usage: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FragmentDefinitionFile {
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub template_name: String,
    pub attach_ports: Vec<AttachPort>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FragmentDefinition {
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub template_name: String,
    pub molecule: Molecule,
    pub attach_ports: Vec<AttachPort>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "SCREAMING_SNAKE_CASE",
    rename_all_fields = "camelCase"
)]
pub enum Command {
    // ... (rest of the commands)
    SetMethod {
        method: Method,
    },
    SetBasis {
        basis: Basis,
    },
    SetJobType {
        job_type: JobType,
    },
    SetSolvent {
        solvent: Option<Solvent>,
    },
    SetCharge {
        charge: i32,
    },
    SetMultiplicity {
        multiplicity: u32,
    },
    SetBondLength {
        atom_ids: [u32; 2],
        length: f64,
        #[serde(default)]
        mode: GeometryEditMode,
    },
    SetBondAngle {
        atom_ids: [u32; 3],
        angle: f64,
        #[serde(default)]
        mode: GeometryEditMode,
    },
    SetDihedralAngle {
        atom_ids: [u32; 4],
        angle: f64,
        #[serde(default)]
        mode: GeometryEditMode,
    },
    AddAtom {
        element: Element,
        position: [f64; 3],
        isotope: Option<MassNumber>,
        nuclear_spin: Option<TwiceSpin>,
        #[serde(default)]
        formal_charge: i32,
    },
    SetAtomFormalCharge {
        atom_id: u32,
        formal_charge: i32,
    },
    DeleteAtom {
        atom_id: u32,
    },
    AddBond {
        atom_ids: [u32; 2],
        order: u8,
    },
    DeleteBond {
        bond_id: u32,
    },
    PlaceTemplate {
        template_name: String,
        position: [f64; 3],
        direction: [f64; 3],
    },
    AttachFragment {
        fragment_name: String,
        target_atom_id: u32,
        rotation_angle: f64,
        orientation: [f64; 3],
    },
    SubstituteByFragment {
        fragment_name: String,
        start_atom_id: u32,
        end_atom_id: u32,
    },
    ReplaceAtom {
        atom_id: u32,
        element: Element,
    },
    SetMolecule {
        molecule: Molecule,
    },
    ToggleAtomSelection {
        atom_id: u32,
    },
    ClearSelection,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GeometryEditMode {
    #[default]
    AtomOnly,
    MoveOtherSide,
    MoveBothSides,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationMessage {
    pub level: ValidationLevel,
    pub message: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SubstituteByFragmentCompletion {
    pub start_atom_id: u32,
    pub end_atom_id: u32,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AtomSummary {
    pub display_index: u32,
    pub element: Element,
    pub isotope: Option<MassNumber>,
    pub nuclear_spin: Option<TwiceSpin>,
    pub formal_charge: i32,
    pub position: [f64; 3],
    pub chemical_context: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AtomIndexMapEntry {
    pub display_index: u32,
    pub atom_id: u32,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CalculationSummary {
    pub job_type: JobType,
    pub method: Method,
    pub basis: Basis,
    pub solvent: Option<Solvent>,
    pub charge: i32,
    pub multiplicity: u32,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiContext {
    pub selected_atoms: Vec<AtomSummary>,
    pub atom_index_map: Vec<AtomIndexMapEntry>,
    pub atom_context_map: std::collections::HashMap<u32, String>,
    pub calculation: CalculationSummary,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiResult {
    pub commands: Vec<Command>,
    pub explanation: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiOutput {
    pub result: AiResult,
    pub ignored_warning: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiDiagnostic {
    pub diagnostics: Vec<String>,
    pub repair_policy: AiRepairPolicy,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiRepairPolicy {
    pub fix_error: bool,
    pub fix_warning: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiProposal {
    pub commands: Vec<Command>,
    pub resolved_commands: Vec<Command>,
    pub explanation: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ValidationLevel {
    Error,
    Warning,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DoubleBondConfiguration {
    E,
    Z,
}

pub fn atom_index(molecule: &Molecule, atom_id: u32) -> Option<usize> {
    molecule.atoms.iter().position(|atom| atom.id == atom_id)
}

pub fn atom_position(molecule: &Molecule, atom_id: u32) -> Option<[f64; 3]> {
    molecule
        .atoms
        .iter()
        .find(|atom| atom.id == atom_id)
        .map(|atom| atom.position)
}

pub fn electronegativity(element: Element) -> Option<Electronegativity> {
    Some(Electronegativity(match element {
        Element::H => 2.20,
        Element::Li => 0.98,
        Element::Be => 1.57,
        Element::B => 2.04,
        Element::C => 2.55,
        Element::N => 3.04,
        Element::O => 3.44,
        Element::F => 3.98,
        Element::Cl => 3.16,
        Element::Br => 2.96,
        Element::I => 2.66,
        _ => return None,
    }))
}

pub fn atom_oxidation_number(molecule: &Molecule, atom_id: u32) -> Option<i32> {
    let atom = molecule.atoms.iter().find(|atom| atom.id == atom_id)?;
    let atom_electronegativity = electronegativity(atom.element)?;
    let mut oxidation_number = atom.formal_charge;

    for bond in molecule
        .bonds
        .iter()
        .filter(|bond| bond.atom_ids.contains(&atom_id))
    {
        let other_atom_id = if bond.atom_ids[0] == atom_id {
            bond.atom_ids[1]
        } else {
            bond.atom_ids[0]
        };
        let other_atom = molecule
            .atoms
            .iter()
            .find(|atom| atom.id == other_atom_id)?;
        let other_electronegativity = electronegativity(other_atom.element)?;
        let order = i32::from(bond.order);

        if atom_electronegativity < other_electronegativity {
            oxidation_number += order;
        } else if atom_electronegativity > other_electronegativity {
            oxidation_number -= order;
        }
    }

    Some(oxidation_number)
}

pub fn double_bond_e_z_configuration(
    molecule: &Molecule,
    first_atom_id: u32,
    second_atom_id: u32,
) -> Option<DoubleBondConfiguration> {
    let bond = molecule.bonds.iter().find(|bond| {
        bond.order == 2 && same_bond(bond.atom_ids, [first_atom_id, second_atom_id])
    })?;
    let [first_atom_id, second_atom_id] = bond.atom_ids;
    let first_priority = highest_priority_substituent(molecule, first_atom_id, second_atom_id)?;
    let second_priority = highest_priority_substituent(molecule, second_atom_id, first_atom_id)?;
    let first_position = atom_position(molecule, first_atom_id)?;
    let second_position = atom_position(molecule, second_atom_id)?;
    let first_substituent_position = atom_position(molecule, first_priority)?;
    let second_substituent_position = atom_position(molecule, second_priority)?;

    let axis = sub(second_position, first_position);
    let first_vector =
        perpendicular_component(sub(first_substituent_position, first_position), axis)?;
    let second_vector =
        perpendicular_component(sub(second_substituent_position, second_position), axis)?;
    let alignment = dot(first_vector, second_vector);

    if alignment.abs() <= 1e-9 {
        None
    } else if alignment > 0.0 {
        Some(DoubleBondConfiguration::Z)
    } else {
        Some(DoubleBondConfiguration::E)
    }
}

pub fn compare_cip_substituents(
    molecule: &Molecule,
    center_atom_id: u32,
    left_atom_id: u32,
    right_atom_id: u32,
) -> Option<Ordering> {
    if left_atom_id == right_atom_id {
        return Some(Ordering::Equal);
    }

    let max_depth = molecule.atoms.len().max(1);
    let mut left_frontier = vec![CipFrontierEntry {
        atom_id: left_atom_id,
        previous_atom_id: center_atom_id,
    }];
    let mut right_frontier = vec![CipFrontierEntry {
        atom_id: right_atom_id,
        previous_atom_id: center_atom_id,
    }];

    for _ in 0..max_depth {
        let mut left_layer = cip_layer(molecule, &left_frontier)?;
        let mut right_layer = cip_layer(molecule, &right_frontier)?;
        left_layer.sort_unstable_by(|left, right| right.cmp(left));
        right_layer.sort_unstable_by(|left, right| right.cmp(left));

        match left_layer.cmp(&right_layer) {
            Ordering::Equal => {}
            ordering => return Some(ordering),
        }

        left_frontier = next_cip_frontier(molecule, &left_frontier);
        right_frontier = next_cip_frontier(molecule, &right_frontier);
        if left_frontier.is_empty() && right_frontier.is_empty() {
            break;
        }
    }

    Some(Ordering::Equal)
}

pub fn atomic_number(element: Element) -> u16 {
    match element {
        Element::H => 1,
        Element::He => 2,
        Element::Li => 3,
        Element::Be => 4,
        Element::B => 5,
        Element::C => 6,
        Element::N => 7,
        Element::O => 8,
        Element::F => 9,
        Element::Ne => 10,
        Element::Na => 11,
        Element::Mg => 12,
        Element::Al => 13,
        Element::Si => 14,
        Element::P => 15,
        Element::S => 16,
        Element::Cl => 17,
        Element::Ar => 18,
        Element::K => 19,
        Element::Ca => 20,
        Element::Sc => 21,
        Element::Ti => 22,
        Element::V => 23,
        Element::Cr => 24,
        Element::Mn => 25,
        Element::Fe => 26,
        Element::Co => 27,
        Element::Ni => 28,
        Element::Cu => 29,
        Element::Zn => 30,
        Element::Ga => 31,
        Element::Ge => 32,
        Element::As => 33,
        Element::Se => 34,
        Element::Br => 35,
        Element::Kr => 36,
        Element::Rb => 37,
        Element::Sr => 38,
        Element::Y => 39,
        Element::Zr => 40,
        Element::Nb => 41,
        Element::Mo => 42,
        Element::Tc => 43,
        Element::Ru => 44,
        Element::Rh => 45,
        Element::Pd => 46,
        Element::Ag => 47,
        Element::Cd => 48,
        Element::In => 49,
        Element::Sn => 50,
        Element::Sb => 51,
        Element::Te => 52,
        Element::I => 53,
        Element::Xe => 54,
        Element::Cs => 55,
        Element::Ba => 56,
        Element::La => 57,
        Element::Ce => 58,
        Element::Pr => 59,
        Element::Nd => 60,
        Element::Pm => 61,
        Element::Sm => 62,
        Element::Eu => 63,
        Element::Gd => 64,
        Element::Tb => 65,
        Element::Dy => 66,
        Element::Ho => 67,
        Element::Er => 68,
        Element::Tm => 69,
        Element::Yb => 70,
        Element::Lu => 71,
        Element::Hf => 72,
        Element::Ta => 73,
        Element::W => 74,
        Element::Re => 75,
        Element::Os => 76,
        Element::Ir => 77,
        Element::Pt => 78,
        Element::Au => 79,
        Element::Hg => 80,
        Element::Tl => 81,
        Element::Pb => 82,
        Element::Bi => 83,
        Element::Po => 84,
        Element::At => 85,
        Element::Rn => 86,
        Element::Fr => 87,
        Element::Ra => 88,
        Element::Ac => 89,
        Element::Th => 90,
        Element::Pa => 91,
        Element::U => 92,
        Element::Np => 93,
        Element::Pu => 94,
        Element::Am => 95,
        Element::Cm => 96,
        Element::Bk => 97,
        Element::Cf => 98,
        Element::Es => 99,
        Element::Fm => 100,
        Element::Md => 101,
        Element::No => 102,
        Element::Lr => 103,
        Element::Rf => 104,
        Element::Db => 105,
        Element::Sg => 106,
        Element::Bh => 107,
        Element::Hs => 108,
        Element::Mt => 109,
        Element::Ds => 110,
        Element::Rg => 111,
        Element::Cn => 112,
        Element::Nh => 113,
        Element::Fl => 114,
        Element::Mc => 115,
        Element::Lv => 116,
        Element::Ts => 117,
        Element::Og => 118,
    }
}

pub fn same_bond(left: [u32; 2], right: [u32; 2]) -> bool {
    (left[0] == right[0] && left[1] == right[1]) || (left[0] == right[1] && left[1] == right[0])
}

#[derive(Clone, Copy)]
struct CipFrontierEntry {
    atom_id: u32,
    previous_atom_id: u32,
}

fn highest_priority_substituent(
    molecule: &Molecule,
    center_atom_id: u32,
    excluded_atom_id: u32,
) -> Option<u32> {
    let substituents = molecule
        .bonds
        .iter()
        .filter_map(|bond| {
            if bond.atom_ids[0] == center_atom_id && bond.atom_ids[1] != excluded_atom_id {
                Some(bond.atom_ids[1])
            } else if bond.atom_ids[1] == center_atom_id && bond.atom_ids[0] != excluded_atom_id {
                Some(bond.atom_ids[0])
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    let mut highest = *substituents.first()?;
    for substituent in substituents.iter().skip(1) {
        match compare_cip_substituents(molecule, center_atom_id, *substituent, highest)? {
            Ordering::Greater => highest = *substituent,
            Ordering::Equal => return None,
            Ordering::Less => {}
        }
    }
    Some(highest)
}

fn cip_layer(molecule: &Molecule, frontier: &[CipFrontierEntry]) -> Option<Vec<(u16, u16)>> {
    frontier
        .iter()
        .map(|entry| {
            molecule
                .atoms
                .iter()
                .find(|atom| atom.id == entry.atom_id)
                .map(cip_atom_key)
        })
        .collect()
}

fn cip_atom_key(atom: &Atom) -> (u16, u16) {
    (
        atomic_number(atom.element),
        atom.isotope.map_or(0, |mass| mass.0),
    )
}

fn next_cip_frontier(molecule: &Molecule, frontier: &[CipFrontierEntry]) -> Vec<CipFrontierEntry> {
    let mut next = Vec::new();
    for entry in frontier {
        for bond in molecule
            .bonds
            .iter()
            .filter(|bond| bond.atom_ids.contains(&entry.atom_id))
        {
            let next_atom_id = if bond.atom_ids[0] == entry.atom_id {
                bond.atom_ids[1]
            } else {
                bond.atom_ids[0]
            };
            if next_atom_id == entry.previous_atom_id {
                continue;
            }
            for _ in 0..bond.order.max(1) {
                next.push(CipFrontierEntry {
                    atom_id: next_atom_id,
                    previous_atom_id: entry.atom_id,
                });
            }
        }
    }
    next
}

fn sub(left: [f64; 3], right: [f64; 3]) -> [f64; 3] {
    [left[0] - right[0], left[1] - right[1], left[2] - right[2]]
}

fn dot(left: [f64; 3], right: [f64; 3]) -> f64 {
    left[0] * right[0] + left[1] * right[1] + left[2] * right[2]
}

fn scale(vector: [f64; 3], scalar: f64) -> [f64; 3] {
    [vector[0] * scalar, vector[1] * scalar, vector[2] * scalar]
}

fn length_squared(vector: [f64; 3]) -> f64 {
    dot(vector, vector)
}

fn perpendicular_component(vector: [f64; 3], axis: [f64; 3]) -> Option<[f64; 3]> {
    let axis_length_squared = length_squared(axis);
    if axis_length_squared <= f64::EPSILON {
        return None;
    }
    let projected = scale(axis, dot(vector, axis) / axis_length_squared);
    let perpendicular = sub(vector, projected);
    (length_squared(perpendicular) > 1e-12).then_some(perpendicular)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn atom(id: u32, element: Element, position: [f64; 3]) -> Atom {
        Atom {
            id,
            element,
            isotope: None,
            nuclear_spin: None,
            formal_charge: 0,
            position,
        }
    }

    fn bond(id: u32, atom_ids: [u32; 2], order: u8) -> Bond {
        Bond {
            id,
            atom_ids,
            order,
        }
    }

    fn dichloroethene(second_chlorine_position: [f64; 3]) -> Molecule {
        Molecule {
            name: "dichloroethene".to_string(),
            atoms: vec![
                atom(1, Element::C, [0.0, 0.0, 0.0]),
                atom(2, Element::C, [1.0, 0.0, 0.0]),
                atom(3, Element::Cl, [0.0, 1.0, 0.0]),
                atom(4, Element::H, [0.0, -1.0, 0.0]),
                atom(5, Element::Cl, second_chlorine_position),
                atom(6, Element::H, [1.0, 1.0, 0.0]),
            ],
            bonds: vec![
                bond(1, [1, 2], 2),
                bond(2, [1, 3], 1),
                bond(3, [1, 4], 1),
                bond(4, [2, 5], 1),
                bond(5, [2, 6], 1),
            ],
        }
    }

    #[test]
    fn cip_priority_uses_atomic_number() {
        let molecule = dichloroethene([1.0, -1.0, 0.0]);

        assert_eq!(
            compare_cip_substituents(&molecule, 1, 3, 4),
            Some(Ordering::Greater)
        );
    }

    #[test]
    fn double_bond_configuration_returns_e_for_opposite_high_priority_substituents() {
        let molecule = dichloroethene([1.0, -1.0, 0.0]);

        assert_eq!(
            double_bond_e_z_configuration(&molecule, 1, 2),
            Some(DoubleBondConfiguration::E)
        );
    }

    #[test]
    fn double_bond_configuration_returns_z_for_same_side_high_priority_substituents() {
        let molecule = dichloroethene([1.0, 1.0, 0.0]);

        assert_eq!(
            double_bond_e_z_configuration(&molecule, 1, 2),
            Some(DoubleBondConfiguration::Z)
        );
    }

    #[test]
    fn double_bond_configuration_returns_none_when_priority_ties() {
        let mut molecule = dichloroethene([1.0, -1.0, 0.0]);
        molecule.atoms[2].element = Element::H;

        assert_eq!(double_bond_e_z_configuration(&molecule, 1, 2), None);
    }
}

pub mod fragment_test;
