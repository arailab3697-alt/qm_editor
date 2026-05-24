use crate::domain::{Atom, Bond, Element, Molecule};
use crate::geometry::{covalent_radius, distance};

pub fn parse_molecule_file(file_name: &str, text: &str) -> Result<Molecule, String> {
    match file_name.rsplit('.').next().map(str::to_ascii_lowercase) {
        Some(extension) if extension == "xyz" => parse_xyz(file_name, text),
        Some(extension) if extension == "mol" => parse_mol(file_name, text),
        _ => Err("Unsupported molecule file. Import .xyz or .mol.".to_string()),
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

pub fn parse_element(value: &str) -> Result<Element, String> {
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

pub fn element_symbol(element: Element) -> &'static str {
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
        Element::Sc => "Sc",
        Element::Ti => "Ti",
        Element::V => "V",
        Element::Cr => "Cr",
        Element::Mn => "Mn",
        Element::Fe => "Fe",
        Element::Co => "Co",
        Element::Ni => "Ni",
        Element::Cu => "Cu",
        Element::Zn => "Zn",
        Element::Ga => "Ga",
        Element::Ge => "Ge",
        Element::As => "As",
        Element::Se => "Se",
        Element::Br => "Br",
        Element::Kr => "Kr",
        Element::Rb => "Rb",
        Element::Sr => "Sr",
        Element::Y => "Y",
        Element::Zr => "Zr",
        Element::Nb => "Nb",
        Element::Mo => "Mo",
        Element::Tc => "Tc",
        Element::Ru => "Ru",
        Element::Rh => "Rh",
        Element::Pd => "Pd",
        Element::Ag => "Ag",
        Element::Cd => "Cd",
        Element::In => "In",
        Element::Sn => "Sn",
        Element::Sb => "Sb",
        Element::Te => "Te",
        Element::I => "I",
        Element::Xe => "Xe",
        Element::Cs => "Cs",
        Element::Ba => "Ba",
        Element::La => "La",
        Element::Ce => "Ce",
        Element::Pr => "Pr",
        Element::Nd => "Nd",
        Element::Pm => "Pm",
        Element::Sm => "Sm",
        Element::Eu => "Eu",
        Element::Gd => "Gd",
        Element::Tb => "Tb",
        Element::Dy => "Dy",
        Element::Ho => "Ho",
        Element::Er => "Er",
        Element::Tm => "Tm",
        Element::Yb => "Yb",
        Element::Lu => "Lu",
        Element::Hf => "Hf",
        Element::Ta => "Ta",
        Element::W => "W",
        Element::Re => "Re",
        Element::Os => "Os",
        Element::Ir => "Ir",
        Element::Pt => "Pt",
        Element::Au => "Au",
        Element::Hg => "Hg",
        Element::Tl => "Tl",
        Element::Pb => "Pb",
        Element::Bi => "Bi",
        Element::Po => "Po",
        Element::At => "At",
        Element::Rn => "Rn",
        Element::Fr => "Fr",
        Element::Ra => "Ra",
        Element::Ac => "Ac",
        Element::Th => "Th",
        Element::Pa => "Pa",
        Element::U => "U",
        Element::Np => "Np",
        Element::Pu => "Pu",
        Element::Am => "Am",
        Element::Cm => "Cm",
        Element::Bk => "Bk",
        Element::Cf => "Cf",
        Element::Es => "Es",
        Element::Fm => "Fm",
        Element::Md => "Md",
        Element::No => "No",
        Element::Lr => "Lr",
        Element::Rf => "Rf",
        Element::Db => "Db",
        Element::Sg => "Sg",
        Element::Bh => "Bh",
        Element::Hs => "Hs",
        Element::Mt => "Mt",
        Element::Ds => "Ds",
        Element::Rg => "Rg",
        Element::Cn => "Cn",
        Element::Nh => "Nh",
        Element::Fl => "Fl",
        Element::Mc => "Mc",
        Element::Lv => "Lv",
        Element::Ts => "Ts",
        Element::Og => "Og",
    }
}

pub fn safe_name(name: &str) -> String {
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
