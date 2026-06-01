export type JobType = "opt" | "freq" | "opt+freq" | "ts";
export type Method = "B3LYP" | "WB97XD";
export type Basis = "6-31G(d)" | "def2-SVP" | "def2-TZVP";
export type Solvent = "THF" | "Water";
export type Element =
  | "H"
  | "He"
  | "Li"
  | "Be"
  | "B"
  | "C"
  | "N"
  | "O"
  | "F"
  | "Ne"
  | "Na"
  | "Mg"
  | "Al"
  | "Si"
  | "P"
  | "S"
  | "Cl"
  | "Ar"
  | "K"
  | "Ca"
  | "Fe"
  | "Cu"
  | "Zn"
  | "Br"
  | "I";

export type Atom = {
  id: number;
  element: Element;
  isotope?: number;
  nuclearSpin?: number;
  formalCharge: number;
  position: [number, number, number];
};

export type Bond = {
  id: number;
  atomIds: [number, number];
  order: 1 | 2 | 3;
};

export type Molecule = {
  name: string;
  atoms: Atom[];
  bonds: Bond[];
};

export type CalculationSpec = {
  jobType: JobType;
  method: Method;
  basis: Basis;
  solvent?: Solvent;
  charge: number;
  multiplicity: number;
};

export type ChemicalSpec = {
  molecule: Molecule;
  calculation: CalculationSpec;
};

export type DomainState = {
  chemicalSpec: ChemicalSpec;
};

export type UIState = {
  selectedAtoms: number[];
};

export type AppState = {
  domain: DomainState;
  ui: UIState;
};

export type ValidationMessage = {
  level: "error" | "warning";
  message: string;
};

export const supportedJobTypes: JobType[] = ["opt", "freq", "opt+freq", "ts"];
export const supportedMethods: Method[] = ["B3LYP", "WB97XD"];
export const supportedBases: Basis[] = ["6-31G(d)", "def2-SVP", "def2-TZVP"];
export const supportedSolvents: Solvent[] = ["THF", "Water"];
export const supportedElements: Element[] = [
  "H",
  "He",
  "Li",
  "Be",
  "B",
  "C",
  "N",
  "O",
  "F",
  "Ne",
  "Na",
  "Mg",
  "Al",
  "Si",
  "P",
  "S",
  "Cl",
  "Ar",
  "K",
  "Ca",
  "Fe",
  "Cu",
  "Zn",
  "Br",
  "I",
];

export const emptyMolecule: Molecule = {
  name: "Untitled molecule",
  atoms: [],
  bonds: [],
};

export const defaultChemicalSpec: ChemicalSpec = {
  molecule: {
    name: "Water",
    atoms: [
      { id: 1, element: "O", formalCharge: 0, position: [0, 0, 0] },
      { id: 2, element: "H", formalCharge: 0, position: [0.758, 0.586, 0] },
      { id: 3, element: "H", formalCharge: 0, position: [-0.758, 0.586, 0] },
    ],
    bonds: [
      { id: 1, atomIds: [1, 2], order: 1 },
      { id: 2, atomIds: [1, 3], order: 1 },
    ],
  },
  calculation: {
    jobType: "opt",
    method: "B3LYP",
    basis: "6-31G(d)",
    charge: 0,
    multiplicity: 1,
  },
};

export const initialAppState: AppState = {
  domain: { chemicalSpec: defaultChemicalSpec },
  ui: { selectedAtoms: [] },
};
