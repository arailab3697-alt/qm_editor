use serde::{Deserialize, Serialize};

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
    pub position: [f64; 3],
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bond {
    pub id: u32,
    pub atom_ids: [u32; 2],
    pub order: u8,
}

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
    #[serde(rename = "Fe")]
    Fe,
    #[serde(rename = "Cu")]
    Cu,
    #[serde(rename = "Zn")]
    Zn,
    #[serde(rename = "Br")]
    Br,
    I,
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
    pub position: [f64; 3],
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

pub fn same_bond(left: [u32; 2], right: [u32; 2]) -> bool {
    (left[0] == right[0] && left[1] == right[1]) || (left[0] == right[1] && left[1] == right[0])
}

pub mod fragment_test;
