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
#[serde(
    tag = "type",
    rename_all = "SCREAMING_SNAKE_CASE",
    rename_all_fields = "camelCase"
)]
pub enum Command {
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
    },
    SetBondAngle {
        atom_ids: [u32; 3],
        angle: f64,
    },
    SetDihedralAngle {
        atom_ids: [u32; 4],
        angle: f64,
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
    SetMolecule {
        molecule: Molecule,
    },
    ToggleAtomSelection {
        atom_id: u32,
    },
    ClearSelection,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationMessage {
    pub level: ValidationLevel,
    pub message: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AtomSummary {
    pub id: u32,
    pub element: Element,
    pub isotope: Option<MassNumber>,
    pub nuclear_spin: Option<TwiceSpin>,
    pub position: [f64; 3],
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
    pub calculation: CalculationSummary,
    pub screenshot: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiResult {
    pub commands: Vec<Command>,
    pub explanation: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ValidationLevel {
    Error,
    Warning,
}

pub fn initial_app_state() -> AppState {
    AppState {
        domain: DomainState {
            chemical_spec: ChemicalSpec {
                molecule: Molecule {
                    name: "Water".to_string(),
                    atoms: vec![
                        Atom {
                            id: 1,
                            element: Element::O,
                            isotope: None,
                            nuclear_spin: None,
                            position: [0.0, 0.0, 0.0],
                        },
                        Atom {
                            id: 2,
                            element: Element::H,
                            isotope: None,
                            nuclear_spin: None,
                            position: [0.758, 0.586, 0.0],
                        },
                        Atom {
                            id: 3,
                            element: Element::H,
                            isotope: None,
                            nuclear_spin: None,
                            position: [-0.758, 0.586, 0.0],
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
                },
                calculation: CalculationSpec {
                    job_type: JobType::Opt,
                    method: Method::B3LYP,
                    basis: Basis::Six31Gd,
                    solvent: None,
                    charge: 0,
                    multiplicity: 1,
                },
            },
        },
        ui: UiState {
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

pub fn parse_molecule_file(file_name: &str, text: &str) -> Result<Molecule, String> {
    match file_name.rsplit('.').next().map(str::to_ascii_lowercase) {
        Some(extension) if extension == "xyz" => parse_xyz(file_name, text),
        Some(extension) if extension == "mol" => parse_mol(file_name, text),
        _ => Err("Unsupported molecule file. Import .xyz or .mol.".to_string()),
    }
}

pub fn render_gaussian(spec: &ChemicalSpec) -> String {
    let calculation = &spec.calculation;
    let molecule = &spec.molecule;
    let mut route = vec![
        route_job(calculation.job_type).to_string(),
        format!(
            "{}/{}",
            method_name(calculation.method),
            basis_name(calculation.basis)
        ),
    ];
    if let Some(solvent) = calculation.solvent {
        route.push(format!("SCRF=(Solvent={})", solvent_name(solvent)));
    }

    let coordinates = molecule
        .atoms
        .iter()
        .map(|atom| {
            format!(
                "{:<2} {:>12.6} {:>12.6} {:>12.6}",
                element_symbol(atom.element),
                atom.position[0],
                atom.position[1],
                atom.position[2]
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        "%chk={}.chk\n# {}\n\n{}\n\n{} {}\n{}\n\n",
        safe_name(&molecule.name),
        route.join(" "),
        if molecule.name.is_empty() {
            "Gaussian input"
        } else {
            &molecule.name
        },
        calculation.charge,
        calculation.multiplicity,
        coordinates
    )
}

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

    messages
}

pub fn build_ai_context(state: &AppState, screenshot: Option<String>) -> AiContext {
    let molecule = &state.domain.chemical_spec.molecule;
    let calculation = &state.domain.chemical_spec.calculation;
    let selected_atoms = state
        .ui
        .selected_atoms
        .iter()
        .filter_map(|atom_id| molecule.atoms.iter().find(|atom| atom.id == *atom_id))
        .map(|atom| AtomSummary {
            id: atom.id,
            element: atom.element,
            isotope: atom.isotope,
            nuclear_spin: atom.nuclear_spin,
            position: atom.position,
        })
        .collect::<Vec<_>>();

    AiContext {
        selected_atoms,
        calculation: CalculationSummary {
            job_type: calculation.job_type,
            method: calculation.method,
            basis: calculation.basis,
            solvent: calculation.solvent,
            charge: calculation.charge,
            multiplicity: calculation.multiplicity,
        },
        screenshot,
    }
}

pub fn propose_ai_commands(input: &str, context: &AiContext) -> AiResult {
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

    let normalized = trimmed.to_ascii_lowercase();
    let mut commands = Vec::new();

    if normalized.contains("b3lyp") {
        commands.push(Command::SetMethod {
            method: Method::B3LYP,
        });
    }
    if normalized.contains("wb97xd") {
        commands.push(Command::SetMethod {
            method: Method::WB97XD,
        });
    }

    if normalized.contains("6-31g(d)") {
        commands.push(Command::SetBasis {
            basis: Basis::Six31Gd,
        });
    }
    if normalized.contains("def2-svp") {
        commands.push(Command::SetBasis {
            basis: Basis::Def2Svp,
        });
    }
    if normalized.contains("def2-tzvp") {
        commands.push(Command::SetBasis {
            basis: Basis::Def2Tzvp,
        });
    }

    if normalized.contains("thf") {
        commands.push(Command::SetSolvent {
            solvent: Some(Solvent::THF),
        });
    }
    if normalized.contains("water") {
        commands.push(Command::SetSolvent {
            solvent: Some(Solvent::Water),
        });
    }
    if normalized.contains("no solvent") || normalized.contains("gas phase") {
        commands.push(Command::SetSolvent { solvent: None });
    }

    if let Some(job_type) = infer_job_type(&normalized) {
        commands.push(Command::SetJobType { job_type });
    }
    if let Some(charge) = parse_number_after(&normalized, "charge") {
        commands.push(Command::SetCharge { charge });
    }
    if let Some(multiplicity) = parse_number_after(&normalized, "multiplicity")
        .or_else(|| parse_number_after(&normalized, "mult"))
        .and_then(|value| u32::try_from(value).ok())
    {
        commands.push(Command::SetMultiplicity { multiplicity });
    }
    if let Some(command) = infer_geometry_command(&normalized, context) {
        commands.push(command);
    }

    let unique_commands = dedupe_ai_commands(commands);
    let explanation = if unique_commands.is_empty() {
        "No supported changes were found. Try mentioning method, basis, job type, solvent, charge, multiplicity, bond length, bond angle, or dihedral angle."
            .to_string()
    } else {
        format!(
            "Proposed {} command(s) from the request. Current method is {}.",
            unique_commands.len(),
            method_name(context.calculation.method)
        )
    };

    AiResult {
        commands: unique_commands,
        explanation,
    }
}

fn parse_xyz(file_name: &str, text: &str) -> Result<Molecule, String> {
    let lines = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    let atom_count = lines
        .first()
        .ok_or_else(|| "XYZ file must start with an atom count.".to_string())?
        .parse::<usize>()
        .map_err(|_| "XYZ file must start with an atom count.".to_string())?;

    let mut atoms = Vec::with_capacity(atom_count);
    for (index, line) in lines.iter().skip(2).take(atom_count).enumerate() {
        let parts = line.split_whitespace().collect::<Vec<_>>();
        if parts.len() < 4 {
            return Err(format!("Invalid XYZ atom line {}.", index + 3));
        }
        atoms.push(Atom {
            id: (index + 1) as u32,
            element: parse_element(parts[0])?,
            isotope: None,
            nuclear_spin: None,
            position: [
                parse_coord(parts[1], "XYZ coordinates must be numeric.")?,
                parse_coord(parts[2], "XYZ coordinates must be numeric.")?,
                parse_coord(parts[3], "XYZ coordinates must be numeric.")?,
            ],
        });
    }

    if atoms.len() != atom_count {
        return Err("XYZ file ended before all atoms were read.".to_string());
    }

    Ok(Molecule {
        name: lines
            .get(1)
            .map(|line| line.to_string())
            .filter(|line| !line.is_empty())
            .unwrap_or_else(|| strip_extension(file_name)),
        bonds: infer_bonds(&atoms),
        atoms,
    })
}

fn parse_mol(file_name: &str, text: &str) -> Result<Molecule, String> {
    let lines = text.lines().collect::<Vec<_>>();
    let counts = lines
        .get(3)
        .ok_or_else(|| "MOL file is missing a counts line.".to_string())?;
    let atom_count = counts
        .get(0..3)
        .unwrap_or("")
        .trim()
        .parse::<usize>()
        .map_err(|_| "MOL counts line is invalid.".to_string())?;
    let bond_count = counts
        .get(3..6)
        .unwrap_or("")
        .trim()
        .parse::<usize>()
        .map_err(|_| "MOL counts line is invalid.".to_string())?;

    let mut atoms = Vec::with_capacity(atom_count);
    for (index, line) in lines.iter().skip(4).take(atom_count).enumerate() {
        let parts = line.split_whitespace().collect::<Vec<_>>();
        if parts.len() < 4 {
            return Err(format!("Invalid MOL atom line {}.", index + 5));
        }
        atoms.push(Atom {
            id: (index + 1) as u32,
            element: parse_element(parts[3])?,
            isotope: None,
            nuclear_spin: None,
            position: [
                parse_coord(parts[0], "MOL coordinates must be numeric.")?,
                parse_coord(parts[1], "MOL coordinates must be numeric.")?,
                parse_coord(parts[2], "MOL coordinates must be numeric.")?,
            ],
        });
    }

    let mut bonds = Vec::with_capacity(bond_count);
    for (index, line) in lines
        .iter()
        .skip(4 + atom_count)
        .take(bond_count)
        .enumerate()
    {
        let parts = line.split_whitespace().collect::<Vec<_>>();
        if parts.len() < 3 {
            return Err(format!("Invalid MOL bond line {}.", index + atom_count + 5));
        }
        let first = parts[0]
            .parse::<u32>()
            .map_err(|_| format!("Invalid MOL bond line {}.", index + atom_count + 5))?;
        let second = parts[1]
            .parse::<u32>()
            .map_err(|_| format!("Invalid MOL bond line {}.", index + atom_count + 5))?;
        let order = parts[2].parse::<u8>().unwrap_or(1).clamp(1, 3);
        bonds.push(Bond {
            id: (index + 1) as u32,
            atom_ids: [first, second],
            order,
        });
    }

    Ok(Molecule {
        name: lines
            .first()
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty())
            .unwrap_or_else(|| strip_extension(file_name)),
        atoms,
        bonds,
    })
}

fn infer_bonds(atoms: &[Atom]) -> Vec<Bond> {
    let mut bonds = Vec::new();
    for first_index in 0..atoms.len() {
        for second_index in (first_index + 1)..atoms.len() {
            let first = &atoms[first_index];
            let second = &atoms[second_index];
            let threshold = covalent_radius(first.element) + covalent_radius(second.element) + 0.45;
            if distance(first.position, second.position) <= threshold {
                bonds.push(Bond {
                    id: (bonds.len() + 1) as u32,
                    atom_ids: [first.id, second.id],
                    order: 1,
                });
            }
        }
    }
    bonds
}

pub fn parse_ai_result_json(text: &str) -> Result<AiResult, String> {
    let parsed = serde_json::from_str::<AiResult>(text).map_err(|error| error.to_string())?;
    let commands = parsed
        .commands
        .into_iter()
        .filter(is_ai_command)
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

fn is_ai_command(command: &Command) -> bool {
    matches!(
        command,
        Command::SetMethod { .. }
            | Command::SetBasis { .. }
            | Command::SetJobType { .. }
            | Command::SetSolvent { .. }
            | Command::SetCharge { .. }
            | Command::SetMultiplicity { .. }
            | Command::SetBondLength { .. }
            | Command::SetBondAngle { .. }
            | Command::SetDihedralAngle { .. }
            | Command::AddAtom { .. }
            | Command::DeleteAtom { .. }
            | Command::AddBond { .. }
            | Command::DeleteBond { .. }
    )
}

fn infer_job_type(text: &str) -> Option<JobType> {
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

fn parse_number_after(text: &str, keyword: &str) -> Option<i32> {
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

fn infer_geometry_command(text: &str, context: &AiContext) -> Option<Command> {
    let value = parse_geometry_value(text)?;
    let selected = context
        .selected_atoms
        .iter()
        .map(|atom| atom.id)
        .collect::<Vec<_>>();

    if (text.contains("dihedral") || text.contains("torsion"))
        && selected.len() >= 4
    {
        return Some(Command::SetDihedralAngle {
            atom_ids: [selected[0], selected[1], selected[2], selected[3]],
            angle: value,
        });
    }
    if (text.contains("bond angle") || text.contains("angle"))
        && selected.len() >= 3
    {
        return Some(Command::SetBondAngle {
            atom_ids: [selected[0], selected[1], selected[2]],
            angle: value,
        });
    }
    if (text.contains("bond length") || text.contains("distance"))
        && selected.len() >= 2
    {
        return Some(Command::SetBondLength {
            atom_ids: [selected[0], selected[1]],
            length: value,
        });
    }

    None
}

fn parse_geometry_value(text: &str) -> Option<f64> {
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

fn dedupe_ai_commands(commands: Vec<Command>) -> Vec<Command> {
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
            | Command::DeleteBond { .. } => unique.push(command),
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
    molecule.atoms.push(Atom {
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
        .any(|bond| same_bond(bond.atom_ids, atom_ids))
    {
        return;
    }
    molecule.bonds.push(Bond {
        id: next_bond_id(molecule),
        atom_ids,
        order,
    });
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

fn same_bond(left: [u32; 2], right: [u32; 2]) -> bool {
    (left[0] == right[0] && left[1] == right[1]) || (left[0] == right[1] && left[1] == right[0])
}

fn atom_index(molecule: &Molecule, atom_id: u32) -> Option<usize> {
    molecule.atoms.iter().position(|atom| atom.id == atom_id)
}

fn atom_position(molecule: &Molecule, atom_id: u32) -> Option<[f64; 3]> {
    molecule
        .atoms
        .iter()
        .find(|atom| atom.id == atom_id)
        .map(|atom| atom.position)
}

fn distance(a: [f64; 3], b: [f64; 3]) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

fn dihedral_degrees(a: [f64; 3], b: [f64; 3], c: [f64; 3], d: [f64; 3]) -> Option<f64> {
    let b0 = sub(b, a);
    let b1 = sub(c, b);
    let b2 = sub(d, c);
    let n1 = normalize(cross(b0, b1))?;
    let n2 = normalize(cross(b1, b2))?;
    let m1 = cross(n1, normalize(b1)?);
    Some(dot(m1, n2).atan2(dot(n1, n2)).to_degrees())
}

fn add(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn scale(vector: [f64; 3], scalar: f64) -> [f64; 3] {
    [vector[0] * scalar, vector[1] * scalar, vector[2] * scalar]
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn length(vector: [f64; 3]) -> f64 {
    dot(vector, vector).sqrt()
}

fn normalize(vector: [f64; 3]) -> Option<[f64; 3]> {
    let vector_length = length(vector);
    if vector_length <= f64::EPSILON {
        None
    } else {
        Some(scale(vector, 1.0 / vector_length))
    }
}

fn rotate(vector: [f64; 3], axis: [f64; 3], radians: f64) -> [f64; 3] {
    let cos = radians.cos();
    let sin = radians.sin();
    add(
        add(scale(vector, cos), scale(cross(axis, vector), sin)),
        scale(axis, dot(axis, vector) * (1.0 - cos)),
    )
}

fn perpendicular(vector: [f64; 3]) -> [f64; 3] {
    let candidate = if vector[0].abs() < 0.9 {
        [1.0, 0.0, 0.0]
    } else {
        [0.0, 1.0, 0.0]
    };
    normalize(cross(vector, candidate)).unwrap_or([0.0, 0.0, 1.0])
}

fn covalent_radius(element: Element) -> f64 {
    match element {
        Element::H => 0.31,
        Element::C => 0.76,
        Element::N => 0.71,
        Element::O => 0.66,
        Element::F => 0.57,
        Element::P => 1.07,
        Element::S => 1.05,
        Element::Cl => 1.02,
        Element::Br => 1.20,
        Element::I => 1.39,
        _ => 0.75,
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

fn route_job(job_type: JobType) -> &'static str {
    match job_type {
        JobType::Opt => "Opt",
        JobType::Freq => "Freq",
        JobType::OptFreq => "Opt Freq",
        JobType::Ts => "Opt=(TS,CalcFC,NoEigenTest)",
    }
}

fn method_name(method: Method) -> &'static str {
    match method {
        Method::B3LYP => "B3LYP",
        Method::WB97XD => "WB97XD",
    }
}

fn basis_name(basis: Basis) -> &'static str {
    match basis {
        Basis::Six31Gd => "6-31G(d)",
        Basis::Def2Svp => "def2-SVP",
        Basis::Def2Tzvp => "def2-TZVP",
    }
}

fn solvent_name(solvent: Solvent) -> &'static str {
    match solvent {
        Solvent::THF => "THF",
        Solvent::Water => "Water",
    }
}

fn parse_element(value: &str) -> Result<Element, String> {
    match normalize_element(value).as_str() {
        "H" => Ok(Element::H),
        "He" => Ok(Element::He),
        "Li" => Ok(Element::Li),
        "Be" => Ok(Element::Be),
        "B" => Ok(Element::B),
        "C" => Ok(Element::C),
        "N" => Ok(Element::N),
        "O" => Ok(Element::O),
        "F" => Ok(Element::F),
        "Ne" => Ok(Element::Ne),
        "Na" => Ok(Element::Na),
        "Mg" => Ok(Element::Mg),
        "Al" => Ok(Element::Al),
        "Si" => Ok(Element::Si),
        "P" => Ok(Element::P),
        "S" => Ok(Element::S),
        "Cl" => Ok(Element::Cl),
        "Ar" => Ok(Element::Ar),
        "K" => Ok(Element::K),
        "Ca" => Ok(Element::Ca),
        "Fe" => Ok(Element::Fe),
        "Cu" => Ok(Element::Cu),
        "Zn" => Ok(Element::Zn),
        "Br" => Ok(Element::Br),
        "I" => Ok(Element::I),
        element => Err(format!("Unsupported element '{element}'.")),
    }
}

fn element_symbol(element: Element) -> &'static str {
    match element {
        Element::H => "H",
        Element::He => "He",
        Element::Li => "Li",
        Element::Be => "Be",
        Element::B => "B",
        Element::C => "C",
        Element::N => "N",
        Element::O => "O",
        Element::F => "F",
        Element::Ne => "Ne",
        Element::Na => "Na",
        Element::Mg => "Mg",
        Element::Al => "Al",
        Element::Si => "Si",
        Element::P => "P",
        Element::S => "S",
        Element::Cl => "Cl",
        Element::Ar => "Ar",
        Element::K => "K",
        Element::Ca => "Ca",
        Element::Fe => "Fe",
        Element::Cu => "Cu",
        Element::Zn => "Zn",
        Element::Br => "Br",
        Element::I => "I",
    }
}

fn safe_name(name: &str) -> String {
    let normalized = name
        .trim()
        .chars()
        .map(|char| {
            if char.is_ascii_alphanumeric() || char == '_' || char == '-' {
                char
            } else {
                '_'
            }
        })
        .collect::<String>();
    if normalized.is_empty() {
        "molecule".to_string()
    } else {
        normalized
    }
}

fn normalize_element(value: &str) -> String {
    let mut chars = value.chars();
    match chars.next() {
        Some(first) => {
            let rest = chars.as_str().to_ascii_lowercase();
            format!("{}{}", first.to_ascii_uppercase(), rest)
        }
        None => String::new(),
    }
}

fn parse_coord(value: &str, message: &str) -> Result<f64, String> {
    value.parse::<f64>().map_err(|_| message.to_string())
}

fn strip_extension(file_name: &str) -> String {
    file_name
        .rsplit_once('.')
        .map(|(name, _)| name)
        .unwrap_or(file_name)
        .to_string()
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
}
