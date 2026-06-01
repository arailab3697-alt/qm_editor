import type { Basis, Element, JobType, Method, Molecule, Solvent } from "./chemicalSpec";

export type Command =
  | {
      type: "SET_BOND_LENGTH";
      atomIds: [number, number];
      length: number;
      mode?: "ATOM_ONLY" | "MOVE_OTHER_SIDE" | "MOVE_BOTH_SIDES";
    }
  | {
      type: "SET_BOND_ANGLE";
      atomIds: [number, number, number];
      angle: number;
      mode?: "ATOM_ONLY" | "MOVE_OTHER_SIDE" | "MOVE_BOTH_SIDES";
    }
  | {
      type: "SET_DIHEDRAL_ANGLE";
      atomIds: [number, number, number, number];
      angle: number;
      mode?: "ATOM_ONLY" | "MOVE_OTHER_SIDE" | "MOVE_BOTH_SIDES";
    }
  | { type: "SET_METHOD"; method: Method }
  | { type: "SET_BASIS"; basis: Basis }
  | { type: "SET_JOB_TYPE"; jobType: JobType }
  | { type: "SET_SOLVENT"; solvent?: Solvent }
  | { type: "SET_CHARGE"; charge: number }
  | { type: "SET_MULTIPLICITY"; multiplicity: number }
  | {
      type: "ADD_ATOM";
      element: Element;
      position: [number, number, number];
      isotope?: number;
      nuclearSpin?: number;
      formalCharge?: number;
    }
  | { type: "SET_ATOM_FORMAL_CHARGE"; atomId: number; formalCharge: number }
  | { type: "DELETE_ATOM"; atomId: number }
  | { type: "ADD_BOND"; atomIds: [number, number]; order: 1 | 2 | 3 }
  | { type: "DELETE_BOND"; bondId: number }
  | { type: "PLACE_TEMPLATE"; templateName: string; position: [number, number, number]; direction: [number, number, number] }
  | { type: "ATTACH_FRAGMENT"; fragmentName: string; targetAtomId: number; rotationAngle: number; orientation: [number, number, number] }
  | { type: "SUBSTITUTE_BY_FRAGMENT"; fragmentName: string; startAtomId: number; endAtomId: number }
  | { type: "SET_MOLECULE"; molecule: Molecule }
  | { type: "TOGGLE_ATOM_SELECTION"; atomId: number }
  | { type: "CLEAR_SELECTION" };

export type AICommand = Exclude<
  Command,
  { type: "SET_MOLECULE" } | { type: "TOGGLE_ATOM_SELECTION" } | { type: "CLEAR_SELECTION" }
>;

export type AIResult = {
  commands: AICommand[];
  resolvedCommands: AICommand[];
  explanation: string;
};

export const commandSchema = {
  oneOf: [
    { type: "object", properties: { type: { const: "SET_METHOD" }, method: { enum: ["B3LYP", "WB97XD"] } }, required: ["type", "method"], additionalProperties: false },
    { type: "object", properties: { type: { const: "SET_BASIS" }, basis: { enum: ["6-31G(d)", "def2-SVP", "def2-TZVP"] } }, required: ["type", "basis"], additionalProperties: false },
    { type: "object", properties: { type: { const: "SET_JOB_TYPE" }, jobType: { enum: ["opt", "freq", "opt+freq", "ts"] } }, required: ["type", "jobType"], additionalProperties: false },
    { type: "object", properties: { type: { const: "SET_SOLVENT" }, solvent: { enum: ["THF", "Water", null] } }, required: ["type", "solvent"], additionalProperties: false },
    { type: "object", properties: { type: { const: "SET_CHARGE" }, charge: { type: "number" } }, required: ["type", "charge"], additionalProperties: false },
    { type: "object", properties: { type: { const: "SET_MULTIPLICITY" }, multiplicity: { type: "number" } }, required: ["type", "multiplicity"], additionalProperties: false },
    { type: "object", properties: { type: { const: "SET_BOND_LENGTH" }, atomIds: { type: "array", items: { type: "number" }, minItems: 2, maxItems: 2 }, length: { type: "number" }, mode: { enum: ["ATOM_ONLY", "MOVE_OTHER_SIDE", "MOVE_BOTH_SIDES"] } }, required: ["type", "atomIds", "length"], additionalProperties: false },
    { type: "object", properties: { type: { const: "SET_BOND_ANGLE" }, atomIds: { type: "array", items: { type: "number" }, minItems: 3, maxItems: 3 }, angle: { type: "number" }, mode: { enum: ["ATOM_ONLY", "MOVE_OTHER_SIDE", "MOVE_BOTH_SIDES"] } }, required: ["type", "atomIds", "angle"], additionalProperties: false },
    { type: "object", properties: { type: { const: "SET_DIHEDRAL_ANGLE" }, atomIds: { type: "array", items: { type: "number" }, minItems: 4, maxItems: 4 }, angle: { type: "number" }, mode: { enum: ["ATOM_ONLY", "MOVE_OTHER_SIDE", "MOVE_BOTH_SIDES"] } }, required: ["type", "atomIds", "angle"], additionalProperties: false },
    { type: "object", properties: { type: { const: "ADD_ATOM" }, element: { type: "string" }, position: { type: "array", items: { type: "number" }, minItems: 3, maxItems: 3 }, isotope: { type: "number" }, nuclearSpin: { type: "number" }, formalCharge: { type: "number" } }, required: ["type", "element", "position"], additionalProperties: false },
    { type: "object", properties: { type: { const: "SET_ATOM_FORMAL_CHARGE" }, atomId: { type: "number" }, formalCharge: { type: "number" } }, required: ["type", "atomId", "formalCharge"], additionalProperties: false },
    { type: "object", properties: { type: { const: "DELETE_ATOM" }, atomId: { type: "number" } }, required: ["type", "atomId"], additionalProperties: false },
    { type: "object", properties: { type: { const: "ADD_BOND" }, atomIds: { type: "array", items: { type: "number" }, minItems: 2, maxItems: 2 }, order: { enum: [1, 2, 3] } }, required: ["type", "atomIds", "order"], additionalProperties: false },
    { type: "object", properties: { type: { const: "DELETE_BOND" }, bondId: { type: "number" } }, required: ["type", "bondId"], additionalProperties: false },
    { type: "object", properties: { type: { const: "PLACE_TEMPLATE" }, templateName: { type: "string" }, position: { type: "array", items: { type: "number" }, minItems: 3, maxItems: 3 }, direction: { type: "array", items: { type: "number" }, minItems: 3, maxItems: 3 } }, required: ["type", "templateName", "position", "direction"], additionalProperties: false },
    { type: "object", properties: { type: { const: "ATTACH_FRAGMENT" }, fragmentName: { type: "string" }, targetAtomId: { type: "number" }, rotationAngle: { type: "number" }, orientation: { type: "array", items: { type: "number" }, minItems: 3, maxItems: 3 } }, required: ["type", "fragmentName", "targetAtomId", "rotationAngle", "orientation"], additionalProperties: false },
    { type: "object", properties: { type: { const: "SUBSTITUTE_BY_FRAGMENT" }, fragmentName: { type: "string" }, startAtomId: { type: "number" }, endAtomId: { type: "number" } }, required: ["type", "fragmentName", "startAtomId", "endAtomId"], additionalProperties: false },
    { type: "object", properties: { type: { const: "SET_MOLECULE" }, molecule: { type: "object" } }, required: ["type", "molecule"], additionalProperties: false },
    { type: "object", properties: { type: { const: "TOGGLE_ATOM_SELECTION" }, atomId: { type: "number" } }, required: ["type", "atomId"], additionalProperties: false },
    { type: "object", properties: { type: { const: "CLEAR_SELECTION" } }, required: ["type"], additionalProperties: false },
  ],
} as const;

export const aiResultSchema = {
  type: "object",
  properties: {
    commands: { type: "array", items: commandSchema },
    resolvedCommands: { type: "array", items: commandSchema },
    explanation: { type: "string" },
  },
  required: ["commands", "resolvedCommands", "explanation"],
  additionalProperties: false,
} as const;
