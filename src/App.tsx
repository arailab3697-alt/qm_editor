import { useEffect, useRef, useState } from "react";
import { createViewer, type AtomSpec, type GLViewer } from "3dmol";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import { useAppStore } from "./app/store";
import type { AICommand, AIResult } from "./domain/commands";
import {
  supportedBases,
  supportedElements,
  supportedJobTypes,
  supportedMethods,
  supportedSolvents,
  type Element,
  type Molecule,
  type Solvent,
  type ValidationMessage,
} from "./domain/chemicalSpec";

type TemplateSummary = {
  name: string;
  displayName: string;
  description: string;
};

type FragmentSummary = {
  name: string;
  displayName: string;
  description: string;
};

type SubstituteByFragmentCompletion = {
  startAtomId: number;
  endAtomId: number;
};

function App() {
  return <EditorShell />;
}

function EditorShell() {
  const { state, loadInitialState } = useAppStore();
  const [gaussian, setGaussian] = useState("");
  const [messages, setMessages] = useState<ValidationMessage[]>([]);

  useEffect(() => {
    void loadInitialState();
  }, [loadInitialState]);

  useEffect(() => {
    if (!state) return;
    const spec = state.domain.chemicalSpec;
    void invoke<string>("render_gaussian_tauri", { spec }).then(setGaussian);
    void invoke<ValidationMessage[]>("validate_chemical_spec_tauri", { spec }).then(setMessages);
  }, [state]);

  if (!state) {
    return (
      <main className="app-shell">
        <section className="viewer-panel">Loading editor...</section>
      </main>
    );
  }

  const spec = state.domain.chemicalSpec;

  return (
    <main className="app-shell">
      <header className="tool-shell">
        <div className="tool-head">
          <div>
            <p className="eyebrow">Gaussian Input IDE</p>
            <h1>DFT Input File Editor</h1>
          </div>
          <ImportControl />
        </div>
        <CalculationToolbar messages={messages} />
      </header>

      <section className="workspace" aria-label="Editor workspace">
        <div className="left-workspace">
          <MoleculeViewer />

          <section className="preview-panel" aria-label="GJF file preview">
            <div className="panel-heading">
              <h2>GJF Preview</h2>
              <span>{spec.molecule.atoms.length} atoms</span>
            </div>
            <pre>{gaussian}</pre>
          </section>
        </div>

        <AIAssistant />
      </section>
    </main>
  );
}

function ImportControl() {
  const { dispatchCommand } = useAppStore();
  const [error, setError] = useState("");

  async function importFile(file: File | undefined) {
    if (!file) return;
    try {
      const text = await file.text();
      const molecule = await invoke<Molecule>("parse_molecule_file_tauri", { fileName: file.name, text });
      await dispatchCommand({ type: "SET_MOLECULE", molecule });
      setError("");
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : "Failed to import molecule.");
    }
  }

  return (
    <div className="import-control">
      <label className="file-button">
        Import XYZ/MOL
        <input
          type="file"
          accept=".xyz,.mol"
          onChange={(event) => void importFile(event.currentTarget.files?.[0])}
        />
      </label>
      {error ? <span className="inline-error">{error}</span> : null}
    </div>
  );
}

function MoleculeViewer() {
  const { state, dispatchCommand } = useAppStore();
  if (!state) return null;
  const { molecule } = state.domain.chemicalSpec;
  const selected = state.ui.selectedAtoms;
  const containerRef = useRef<HTMLDivElement | null>(null);
  const viewerRef = useRef<GLViewer | null>(null);

  useEffect(() => {
    const container = containerRef.current;
    if (!container) return;

    if (!viewerRef.current) {
      viewerRef.current = createViewer(container, { backgroundColor: "#f8fafc" });
    }

    const viewer = viewerRef.current;
    viewer.removeAllModels();
    viewer.removeAllLabels();
    viewer.addModel(moleculeToMol(molecule), "mol");
    viewer.setStyle({}, { stick: { radius: 0.15 }, sphere: { scale: 0.34 } });

    molecule.atoms.forEach((atom, index) => {
      viewer.addLabel(formatAtomLabel(index + 1, atom.formalCharge), {
        position: { x: atom.position[0], y: atom.position[1], z: atom.position[2] },
        backgroundColor: "white",
        backgroundOpacity: 0.5,
        fontSize: 12,
        fontColor: "black",
      });
    });

    for (const atomId of selected) {
      const atomIndex = molecule.atoms.findIndex((atom) => atom.id === atomId);
      if (atomIndex < 0) continue;
      viewer.setStyle(
        { index: atomIndex },
        { stick: { radius: 0.2, color: "#c27a22" }, sphere: { scale: 0.46, color: "#f4b13d" } },
      );
    }
    viewer.setClickable({}, true, (atom: AtomSpec) => {
      const atomId = atom.index === undefined ? undefined : molecule.atoms[atom.index]?.id;
      if (atomId !== undefined) void dispatchCommand({ type: "TOGGLE_ATOM_SELECTION", atomId });
    });
    viewer.zoomTo();
    viewer.render();
    viewer.resize();
  }, [dispatchCommand, molecule, selected]);

  return (
    <section className="viewer-panel" aria-label="Molecule viewer">
      <div className="panel-heading">
        <div>
          <h2>{molecule.name}</h2>
          <p>{formatSelectedDisplayAtoms(molecule, selected)}</p>
        </div>
        <button type="button" onClick={() => void dispatchCommand({ type: "CLEAR_SELECTION" })}>
          Clear
        </button>
      </div>

      <div ref={containerRef} className="molecule-canvas" role="img" aria-label="3D molecule viewer" />
    </section>
  );
}

function CalculationToolbar({ messages }: { messages: ValidationMessage[] }) {
  const { state, dispatchCommand } = useAppStore();
  if (!state) return null;
  const calculation = state.domain.chemicalSpec.calculation;

  return (
    <section className="toolbar" aria-label="Editor toolbar">
      <div className="toolbar-section calculation-tools">
        <div className="tool-section-heading">
          <h2>Calculation</h2>
        </div>

        <div className="form-grid">
          <SelectField
            label="Job type"
            value={calculation.jobType}
            options={supportedJobTypes}
            onChange={(jobType) => void dispatchCommand({ type: "SET_JOB_TYPE", jobType })}
          />
          <SelectField
            label="Method"
            value={calculation.method}
            options={supportedMethods}
            onChange={(method) => void dispatchCommand({ type: "SET_METHOD", method })}
          />
          <SelectField
            label="Basis"
            value={calculation.basis}
            options={supportedBases}
            onChange={(basis) => void dispatchCommand({ type: "SET_BASIS", basis })}
          />
          <label>
            Solvent
            <select
              value={calculation.solvent ?? ""}
              onChange={(event) =>
                void dispatchCommand({
                  type: "SET_SOLVENT",
                  solvent: event.currentTarget.value ? (event.currentTarget.value as Solvent) : undefined,
                })
              }
            >
              <option value="">Gas phase</option>
              {supportedSolvents.map((solvent) => (
                <option key={solvent} value={solvent}>
                  {solvent}
                </option>
              ))}
            </select>
          </label>
          <NumberField
            label="Charge"
            value={calculation.charge}
            onChange={(charge) => void dispatchCommand({ type: "SET_CHARGE", charge })}
          />
          <NumberField
            label="Multiplicity"
            min={1}
            value={calculation.multiplicity}
            onChange={(multiplicity) => void dispatchCommand({ type: "SET_MULTIPLICITY", multiplicity })}
          />
        </div>
      </div>

      <MoleculeEditor />

      <TemplateFragmentTools />

      <GeometryEditor />

      <div className="validation-list toolbar-section">
        {messages.length === 0 ? (
          <p className="valid">Ready to render Gaussian input.</p>
        ) : (
          messages.map((message) => (
            <p key={message.message} className={message.level}>
              {message.message}
            </p>
          ))
        )}
      </div>
    </section>
  );
}

function MoleculeEditor() {
  const { state, dispatchCommand, applyCommands } = useAppStore();
  const [element, setElement] = useState<Element>("C");
  const [x, setX] = useState("0");
  const [y, setY] = useState("0");
  const [z, setZ] = useState("0");
  const [isotope, setIsotope] = useState("");
  const [nuclearSpin, setNuclearSpin] = useState("");
  const [formalCharge, setFormalCharge] = useState("0");
  const [selectedFormalCharge, setSelectedFormalCharge] = useState("0");
  const [bondOrder, setBondOrder] = useState<1 | 2 | 3>(1);
  const molecule = state?.domain.chemicalSpec.molecule;
  const selected = state?.ui.selectedAtoms ?? [];

  useEffect(() => {
    if (!molecule || selected.length !== 1) return;
    const atom = molecule.atoms.find((candidate) => candidate.id === selected[0]);
    if (atom) setSelectedFormalCharge(String(atom.formalCharge));
  }, [molecule, selected]);

  if (!state || !molecule) return null;
  const coordinates = [Number(x), Number(y), Number(z)] as [number, number, number];
  const canAddAtom =
    coordinates.every(Number.isFinite) &&
    isOptionalInteger(isotope, 1) &&
    isOptionalInteger(nuclearSpin, 0) &&
    isInteger(formalCharge);
  const canSetFormalCharge = selected.length > 0 && isInteger(selectedFormalCharge);
  const selectedBondIds = molecule.bonds
    .filter((bond) => bond.atomIds.every((atomId) => selected.includes(atomId)))
    .map((bond) => bond.id);

  function addAtom() {
    if (!canAddAtom) return;
    void dispatchCommand({
      type: "ADD_ATOM",
      element,
      position: coordinates,
      isotope: isotope === "" ? undefined : Number(isotope),
      nuclearSpin: nuclearSpin === "" ? undefined : Number(nuclearSpin),
      formalCharge: Number(formalCharge),
    });
  }

  function deleteSelectedAtoms() {
    void applyCommands(selected.map((atomId) => ({ type: "DELETE_ATOM", atomId })));
  }

  return (
    <div className="molecule-editor" aria-label="Molecule edit menu">
      <div className="tool-section-heading">
        <h3>Molecule Edit</h3>
        <span>{molecule.bonds.length} bonds</span>
      </div>

      <div className="atom-editor-grid">
        <SelectField label="Element" value={element} options={supportedElements} onChange={setElement} />
        <NumberTextField label="X" value={x} step="0.001" onChange={setX} />
        <NumberTextField label="Y" value={y} step="0.001" onChange={setY} />
        <NumberTextField label="Z" value={z} step="0.001" onChange={setZ} />
        <NumberTextField label="Isotope" value={isotope} min="1" step="1" onChange={setIsotope} />
        <NumberTextField label="2I" value={nuclearSpin} min="0" step="1" onChange={setNuclearSpin} />
        <NumberTextField label="Formal charge" value={formalCharge} step="1" onChange={setFormalCharge} />
      </div>

      <div className="editor-actions">
        <button type="button" disabled={!canAddAtom} onClick={addAtom}>
          Add Atom
        </button>
        <button type="button" disabled={selected.length === 0} onClick={deleteSelectedAtoms}>
          Delete Selected Atoms
        </button>
      </div>

      <div className="bond-actions">
        <label>
          Selected charge
          <input
            type="number"
            step="1"
            value={selectedFormalCharge}
            disabled={selected.length === 0}
            onChange={(event) => setSelectedFormalCharge(event.currentTarget.value)}
          />
        </label>
        <button
          type="button"
          disabled={!canSetFormalCharge}
          onClick={() =>
            void applyCommands(
              selected.map((atomId) => ({
                type: "SET_ATOM_FORMAL_CHARGE",
                atomId,
                formalCharge: Number(selectedFormalCharge),
              })),
            )
          }
        >
          Set Formal Charge
        </button>
        <label>
          Bond order
          <select value={bondOrder} onChange={(event) => setBondOrder(Number(event.currentTarget.value) as 1 | 2 | 3)}>
            <option value={1}>1</option>
            <option value={2}>2</option>
            <option value={3}>3</option>
          </select>
        </label>
        <button
          type="button"
          disabled={selected.length < 2}
          onClick={() =>
            void dispatchCommand({
              type: "ADD_BOND",
              atomIds: [selected[0], selected[1]],
              order: bondOrder,
            })
          }
        >
          Add Bond
        </button>
        <button
          type="button"
          disabled={selectedBondIds.length === 0}
          onClick={() => void applyCommands(selectedBondIds.map((bondId) => ({ type: "DELETE_BOND", bondId })))}
        >
          Delete Selected Bonds
        </button>
      </div>
    </div>
  );
}

function TemplateFragmentTools() {
  const { state, dispatchCommand } = useAppStore();
  const [templates, setTemplates] = useState<TemplateSummary[]>([]);
  const [fragments, setFragments] = useState<FragmentSummary[]>([]);
  const [templateName, setTemplateName] = useState("");
  const [fragmentName, setFragmentName] = useState("");
  const [x, setX] = useState("0");
  const [y, setY] = useState("0");
  const [z, setZ] = useState("0");
  const [substitutionCompletion, setSubstitutionCompletion] = useState<SubstituteByFragmentCompletion | null>(null);

  useEffect(() => {
    void invoke<TemplateSummary[]>("list_available_templates_tauri").then((availableTemplates) => {
      setTemplates(availableTemplates);
      setTemplateName((current) => current || availableTemplates[0]?.name || "");
    });
    void invoke<FragmentSummary[]>("list_available_fragments_tauri").then((availableFragments) => {
      setFragments(availableFragments);
      setFragmentName((current) => current || availableFragments[0]?.name || "");
    });
  }, []);

  useEffect(() => {
    if (!state || state.ui.selectedAtoms.length !== 1) {
      setSubstitutionCompletion(null);
      return;
    }

    const molecule = state.domain.chemicalSpec.molecule;
    const selectedAtomId = state.ui.selectedAtoms[0];
    void invoke<SubstituteByFragmentCompletion | null>("infer_substitute_by_fragment_completion_tauri", {
      molecule,
      selectedAtomId,
    }).then(setSubstitutionCompletion);
  }, [state]);

  if (!state) return null;

  const selected = state.ui.selectedAtoms;
  const position = [Number(x), Number(y), Number(z)] as [number, number, number];
  const canPlaceTemplate = Boolean(templateName) && position.every(Number.isFinite);
  const canAttachFragment = Boolean(fragmentName) && selected.length >= 1;
  const substitutionAtomIds =
    selected.length >= 2
      ? ({ startAtomId: selected[0], endAtomId: selected[1] } satisfies SubstituteByFragmentCompletion)
      : substitutionCompletion;
  const canSubstituteFragment = Boolean(fragmentName) && substitutionAtomIds !== null;

  return (
    <div className="template-fragment-tools" aria-label="Template and fragment tools">
      <div className="tool-section-heading">
        <h3>Templates / Fragments</h3>
        <span>{formatSelectedDisplayAtoms(state.domain.chemicalSpec.molecule, selected)}</span>
      </div>

      <div className="template-grid">
        <label>
          Template
          <select value={templateName} onChange={(event) => setTemplateName(event.currentTarget.value)}>
            {templates.map((template) => (
              <option key={template.name} value={template.name}>
                {template.displayName}
              </option>
            ))}
          </select>
        </label>
        <NumberTextField label="X" value={x} step="0.001" onChange={setX} />
        <NumberTextField label="Y" value={y} step="0.001" onChange={setY} />
        <NumberTextField label="Z" value={z} step="0.001" onChange={setZ} />
      </div>

      <div className="editor-actions">
        <button
          type="button"
          disabled={!canPlaceTemplate}
          onClick={() =>
            void dispatchCommand({
              type: "PLACE_TEMPLATE",
              templateName,
              position,
              direction: [1, 0, 0],
            })
          }
        >
          Add Template
        </button>
      </div>

      <div className="fragment-actions">
        <label>
          Fragment
          <select value={fragmentName} onChange={(event) => setFragmentName(event.currentTarget.value)}>
            {fragments.map((fragment) => (
              <option key={fragment.name} value={fragment.name}>
                {fragment.displayName}
              </option>
            ))}
          </select>
        </label>
        <button
          type="button"
          disabled={!canAttachFragment}
          onClick={() =>
            void dispatchCommand({
              type: "ATTACH_FRAGMENT",
              fragmentName,
              targetAtomId: selected[0],
              rotationAngle: 0,
              orientation: [1, 0, 0],
            })
          }
        >
          Attach Fragment
        </button>
        <button
          type="button"
          disabled={!canSubstituteFragment}
          onClick={() =>
            substitutionAtomIds &&
            void dispatchCommand({
              type: "SUBSTITUTE_BY_FRAGMENT",
              fragmentName,
              startAtomId: substitutionAtomIds.startAtomId,
              endAtomId: substitutionAtomIds.endAtomId,
            })
          }
        >
          Substitute Fragment
        </button>
      </div>
    </div>
  );
}

function GeometryEditor() {
  const { state, dispatchCommand } = useAppStore();
  const [bondLength, setBondLength] = useState("");
  const [bondAngle, setBondAngle] = useState("");
  const [dihedralAngle, setDihedralAngle] = useState("");
  const molecule = state?.domain.chemicalSpec.molecule;
  const selected = state?.ui.selectedAtoms ?? [];
  const bondAtomIds = selected.length >= 2 ? ([selected[0], selected[1]] as [number, number]) : null;
  const angleAtomIds = selected.length >= 3 ? ([selected[0], selected[1], selected[2]] as [number, number, number]) : null;
  const dihedralAtomIds =
    selected.length >= 4 ? ([selected[0], selected[1], selected[2], selected[3]] as [number, number, number, number]) : null;

  useEffect(() => {
    if (!molecule) return;
    const lengthValue = selected.length >= 2 ? measureBondLength(molecule, selected[0], selected[1]) : undefined;
    const angleValue = selected.length >= 3 ? measureBondAngle(molecule, selected[0], selected[1], selected[2]) : undefined;
    const dihedralValue =
      selected.length >= 4 ? measureDihedralAngle(molecule, selected[0], selected[1], selected[2], selected[3]) : undefined;
    setBondLength(formatMeasure(lengthValue));
    setBondAngle(formatMeasure(angleValue));
    setDihedralAngle(formatMeasure(dihedralValue));
  }, [molecule, selected]);

  if (!state) return null;

  return (
    <div className="geometry-editor" aria-label="Geometry edit menu">
      <div className="tool-section-heading">
        <h3>Geometry Edit</h3>
        <span>Select 2, 3, or 4 atoms in order</span>
      </div>
      <div className="geometry-grid">
        <label>
          Bond length
          <div className="inline-field">
            <input
              type="number"
              step="0.001"
              min="0.001"
              value={bondLength}
              disabled={selected.length < 2}
              onChange={(event) => setBondLength(event.currentTarget.value)}
            />
            <button
              type="button"
              disabled={selected.length < 2 || !Number.isFinite(Number(bondLength)) || Number(bondLength) <= 0}
              onClick={() =>
                bondAtomIds &&
                void dispatchCommand({
                  type: "SET_BOND_LENGTH",
                  atomIds: bondAtomIds,
                  length: Number(bondLength),
                })
              }
            >
              SetBondLength
            </button>
          </div>
        </label>
        <label>
          Bond angle
          <div className="inline-field">
            <input
              type="number"
              step="0.1"
              min="0"
              max="180"
              value={bondAngle}
              disabled={selected.length < 3}
              onChange={(event) => setBondAngle(event.currentTarget.value)}
            />
            <button
              type="button"
              disabled={selected.length < 3 || !isAngleInput(bondAngle)}
              onClick={() =>
                angleAtomIds &&
                void dispatchCommand({
                  type: "SET_BOND_ANGLE",
                  atomIds: angleAtomIds,
                  angle: Number(bondAngle),
                })
              }
            >
              SetBondAngle
            </button>
          </div>
        </label>
        <label>
          Dihedral angle
          <div className="inline-field">
            <input
              type="number"
              step="0.1"
              value={dihedralAngle}
              disabled={selected.length < 4}
              onChange={(event) => setDihedralAngle(event.currentTarget.value)}
            />
            <button
              type="button"
              disabled={selected.length < 4 || !Number.isFinite(Number(dihedralAngle))}
              onClick={() =>
                dihedralAtomIds &&
                void dispatchCommand({
                  type: "SET_DIHEDRAL_ANGLE",
                  atomIds: dihedralAtomIds,
                  angle: Number(dihedralAngle),
                })
              }
            >
              SetDihedralAngle
            </button>
          </div>
        </label>
      </div>
    </div>
  );
}

function SelectField<T extends string>({
  label,
  value,
  options,
  onChange,
}: {
  label: string;
  value: T;
  options: readonly T[];
  onChange: (value: T) => void;
}) {
  return (
    <label>
      {label}
      <select value={value} onChange={(event) => onChange(event.currentTarget.value as T)}>
        {options.map((option) => (
          <option key={option} value={option}>
            {option}
          </option>
        ))}
      </select>
    </label>
  );
}

function NumberField({
  label,
  value,
  min,
  onChange,
}: {
  label: string;
  value: number;
  min?: number;
  onChange: (value: number) => void;
}) {
  return (
    <label>
      {label}
      <input
        type="number"
        min={min}
        value={value}
        onChange={(event) => onChange(Number(event.currentTarget.value))}
      />
    </label>
  );
}

function NumberTextField({
  label,
  value,
  min,
  step,
  onChange,
}: {
  label: string;
  value: string;
  min?: string;
  step?: string;
  onChange: (value: string) => void;
}) {
  return (
    <label>
      {label}
      <input
        type="number"
        min={min}
        step={step}
        value={value}
        onChange={(event) => onChange(event.currentTarget.value)}
      />
    </label>
  );
}

type YoloStepStatus = "pending" | "running" | "applied" | "skipped" | "failed";

type YoloStep = {
  id: number;
  goal: string;
  status: YoloStepStatus;
  prompt?: string;
  beforeScreenshot?: string;
  afterScreenshot?: string;
  commands?: AICommand[];
  explanation?: string;
  error?: string;
};

function AIAssistant() {
  const { state, applyCommands, undo, canUndo } = useAppStore();
  const [request, setRequest] = useState("");
  const [result, setResult] = useState<AIResult | null>(null);
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);
  const [yoloRunning, setYoloRunning] = useState(false);
  const [yoloSteps, setYoloSteps] = useState<YoloStep[]>([]);

  function captureScreenshot() {
    const canvas = document.querySelector<HTMLCanvasElement>(".molecule-canvas canvas");
    return canvas?.toDataURL("image/png");
  }

  function generateCommands() {
    if (!state || loading) return;
    setError("");
    setLoading(true);
    setResult(null);

    const screenshot = captureScreenshot();
    void invoke<AIResult>("propose_commands_via_ai_tauri", {
      input: request,
      state,
      screenshot,
    })
      .then(setResult)
      .catch((caught) => {
        setResult(null);
        setError(typeof caught === "string" ? caught : "Failed to generate AI commands.");
      })
      .finally(() => {
        setLoading(false);
      });
  }

  async function applyAICommands() {
    if (!result || result.resolvedCommands.length === 0) return;
    await applyCommands(result.resolvedCommands);
    setResult(null);
  }

  async function runYoloMode() {
    if (!state || loading || yoloRunning || !request.trim()) return;
    setError("");
    setResult(null);
    setLoading(true);
    setYoloRunning(true);

    const plan = buildYoloPlan(request);
    setYoloSteps(plan);

    const history: string[] = [];
    try {
      for (const step of plan) {
        const liveState = useAppStore.getState().state;
        if (!liveState) throw new Error("Editor state is not available.");

        const beforeScreenshot = captureScreenshot();
        const prompt = buildYoloStepPrompt(request, step, plan, history);
        setYoloSteps((current) =>
          updateYoloStep(current, step.id, { status: "running", prompt, beforeScreenshot }),
        );

        const proposal = await invoke<AIResult>("propose_commands_via_ai_tauri", {
          input: prompt,
          state: liveState,
          screenshot: beforeScreenshot,
        });

        if (proposal.resolvedCommands.length > 0) {
          await applyCommands(proposal.resolvedCommands);
          await waitForMoleculeRender();
        }

        const afterScreenshot = captureScreenshot();
        const status: YoloStepStatus = proposal.resolvedCommands.length > 0 ? "applied" : "skipped";
        history.push(
          `Step ${step.id} (${step.goal}) ${status}: ${proposal.explanation}. Commands: ${JSON.stringify(
            proposal.commands,
          )}`,
        );
        setYoloSteps((current) =>
          updateYoloStep(current, step.id, {
            status,
            afterScreenshot,
            commands: proposal.commands,
            explanation: proposal.explanation,
          }),
        );
      }
    } catch (caught) {
      const message = caught instanceof Error ? caught.message : typeof caught === "string" ? caught : "YOLO mode failed.";
      setError(message);
      setYoloSteps((current) => {
        const running = current.find((step) => step.status === "running");
        return running ? updateYoloStep(current, running.id, { status: "failed", error: message }) : current;
      });
    } finally {
      setLoading(false);
      setYoloRunning(false);
    }
  }

  return (
    <section className="assistant-panel" aria-label="AI assistant">
      <div className="panel-heading">
        <h2>AI Assistant</h2>
        <button type="button" disabled={!canUndo()} onClick={undo}>
          Undo
        </button>
      </div>
      <textarea
        value={request}
        onChange={(event) => setRequest(event.currentTarget.value)}
        placeholder="Set WB97XD with def2-TZVP in THF, or set selected bond length to 1.42"
        disabled={loading}
      />
      <div className="assistant-actions">
        <button type="button" onClick={generateCommands} disabled={loading || !request.trim()}>
          {loading && !yoloRunning ? "Generating..." : "Generate Commands"}
        </button>
        <button type="button" onClick={runYoloMode} disabled={loading || yoloRunning || !request.trim()}>
          {yoloRunning ? "YOLO Running..." : "YOLO"}
        </button>
        <button type="button" disabled={loading || !result || result.resolvedCommands.length === 0} onClick={() => void applyAICommands()}>
          Apply Commands
        </button>
      </div>
      {result ? (
        <div className="ai-output">
          <p>
            {result.explanation}
          </p>
          <pre>{JSON.stringify({ commands: result.commands, explanation: result.explanation }, null, 2)}</pre>
        </div>
      ) : null}
      {yoloSteps.length > 0 ? <YoloRunLog steps={yoloSteps} /> : null}
      {error ? <p className="inline-error">{error}</p> : null}
    </section>
  );
}

function YoloRunLog({ steps }: { steps: YoloStep[] }) {
  return (
    <div className="yolo-run-log" aria-label="YOLO mode run log">
      <div className="tool-section-heading">
        <h3>YOLO Plan</h3>
        <span>{steps.filter((step) => step.status === "applied" || step.status === "skipped").length}/{steps.length}</span>
      </div>
      <ol>
        {steps.map((step) => (
          <li key={step.id} className={`yolo-step yolo-step-${step.status}`}>
            <div className="yolo-step-head">
              <strong>{step.goal}</strong>
              <span>{step.status}</span>
            </div>
            {step.explanation ? <p>{step.explanation}</p> : null}
            {step.error ? <p className="inline-error">{step.error}</p> : null}
            {step.prompt ? (
              <details>
                <summary>Input sent after current screenshot</summary>
                <pre>{step.prompt}</pre>
              </details>
            ) : null}
            <div className="yolo-screenshots">
              {step.beforeScreenshot ? <ScreenshotThumb label="Before" src={step.beforeScreenshot} /> : null}
              {step.afterScreenshot ? <ScreenshotThumb label="After apply" src={step.afterScreenshot} /> : null}
            </div>
            {step.commands ? <pre>{JSON.stringify(step.commands, null, 2)}</pre> : null}
          </li>
        ))}
      </ol>
    </div>
  );
}

function ScreenshotThumb({ label, src }: { label: string; src: string }) {
  return (
    <figure>
      <img src={src} alt={`${label} molecule screenshot`} />
      <figcaption>{label}</figcaption>
    </figure>
  );
}

function buildYoloPlan(request: string): YoloStep[] {
  const explicitTasks = request
    .split(/[\n。.!?]+/)
    .map((part) => part.trim())
    .filter(Boolean)
    .slice(0, 6);
  const goals = explicitTasks.length > 1 ? explicitTasks : [request.trim(), "Verify the applied editor state and make one corrective command batch if needed"];

  return goals.map((goal, index) => ({
    id: index + 1,
    goal,
    status: "pending",
  }));
}

function buildYoloStepPrompt(originalRequest: string, step: YoloStep, plan: YoloStep[], history: string[]) {
  return [
    "YOLO MODE: act as an agentic Gaussian input editor.",
    "You are executing one subtask now. Use the provided screenshot plus current state, propose only the commands for this subtask, and keep the explanation concise.",
    "If this subtask is already satisfied in the current state, return zero commands and explain that it is already satisfied.",
    "Do not ask the user for confirmation in YOLO mode; choose the safest chemically reasonable command batch available.",
    `Original request: ${originalRequest}`,
    `Plan: ${plan.map((item) => `${item.id}. ${item.goal}`).join(" | ")}`,
    `Current subtask ${step.id}/${plan.length}: ${step.goal}`,
    history.length > 0 ? `Previous applied subtasks: ${history.join("\n")}` : "Previous applied subtasks: none",
    "Return the normal raw JSON command response for this one subtask only.",
  ].join("\n");
}

function updateYoloStep(steps: YoloStep[], id: number, patch: Partial<YoloStep>) {
  return steps.map((step) => (step.id === id ? { ...step, ...patch } : step));
}

function waitForMoleculeRender() {
  return new Promise<void>((resolve) => {
    requestAnimationFrame(() => requestAnimationFrame(() => resolve()));
  });
}

function formatMeasure(value: number | undefined) {
  return value === undefined || !Number.isFinite(value) ? "" : value.toFixed(3);
}

function isAngleInput(value: string) {
  const numeric = Number(value);
  return Number.isFinite(numeric) && numeric >= 0 && numeric <= 180;
}

function isOptionalInteger(value: string, min: number) {
  if (value === "") return true;
  const numeric = Number(value);
  return Number.isInteger(numeric) && numeric >= min;
}

function isInteger(value: string) {
  return Number.isInteger(Number(value));
}

function formatAtomLabel(displayIndex: number, formalCharge: number) {
  formalCharge = formalCharge ?? 0;
  if (formalCharge === 0) return String(displayIndex);
  const magnitude = Math.abs(formalCharge);
  const sign = formalCharge > 0 ? "+" : "-";
  return `${displayIndex} ${magnitude === 1 ? sign : `${magnitude}${sign}`}`;
}

function formatSelectedDisplayAtoms(molecule: Molecule, selectedAtomIds: number[]) {
  if (selectedAtomIds.length === 0) return "No selection";
  const displayIds = selectedAtomIds
    .map((atomId) => {
      const displayIndex = molecule.atoms.findIndex((atom) => atom.id === atomId);
      return displayIndex < 0 ? undefined : displayIndex + 1;
    })
    .filter((displayIndex): displayIndex is number => displayIndex !== undefined);

  return displayIds.length === 0 ? "No selection" : `Selected: ${displayIds.join(", ")}`;
}

function measureBondLength(molecule: Molecule, firstId: number, secondId: number) {
  const first = findAtomPosition(molecule, firstId);
  const second = findAtomPosition(molecule, secondId);
  return first && second ? vectorLength(subtract(second, first)) : undefined;
}

function measureBondAngle(molecule: Molecule, firstId: number, centerId: number, thirdId: number) {
  const first = findAtomPosition(molecule, firstId);
  const center = findAtomPosition(molecule, centerId);
  const third = findAtomPosition(molecule, thirdId);
  if (!first || !center || !third) return undefined;
  const firstVector = subtract(first, center);
  const thirdVector = subtract(third, center);
  const denominator = vectorLength(firstVector) * vectorLength(thirdVector);
  if (denominator === 0) return undefined;
  return Math.acos(clamp(dot(firstVector, thirdVector) / denominator, -1, 1)) * (180 / Math.PI);
}

function measureDihedralAngle(molecule: Molecule, firstId: number, secondId: number, thirdId: number, fourthId: number) {
  const first = findAtomPosition(molecule, firstId);
  const second = findAtomPosition(molecule, secondId);
  const third = findAtomPosition(molecule, thirdId);
  const fourth = findAtomPosition(molecule, fourthId);
  if (!first || !second || !third || !fourth) return undefined;
  const b0 = subtract(second, first);
  const b1 = subtract(third, second);
  const b2 = subtract(fourth, third);
  const n1 = normalize(cross(b0, b1));
  const n2 = normalize(cross(b1, b2));
  const b1Unit = normalize(b1);
  if (!n1 || !n2 || !b1Unit) return undefined;
  const m1 = cross(n1, b1Unit);
  return Math.atan2(dot(m1, n2), dot(n1, n2)) * (180 / Math.PI);
}

function findAtomPosition(molecule: Molecule, atomId: number) {
  return molecule.atoms.find((atom) => atom.id === atomId)?.position;
}

function subtract(a: [number, number, number], b: [number, number, number]): [number, number, number] {
  return [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
}

function dot(a: [number, number, number], b: [number, number, number]) {
  return a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
}

function cross(a: [number, number, number], b: [number, number, number]): [number, number, number] {
  return [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]];
}

function vectorLength(vector: [number, number, number]) {
  return Math.sqrt(dot(vector, vector));
}

function normalize(vector: [number, number, number]) {
  const length = vectorLength(vector);
  return length === 0 ? undefined : ([vector[0] / length, vector[1] / length, vector[2] / length] as [number, number, number]);
}

function clamp(value: number, min: number, max: number) {
  return Math.min(max, Math.max(min, value));
}

function moleculeToMol(molecule: Molecule) {
  const atomIndexById = new Map(molecule.atoms.map((atom, index) => [atom.id, index + 1]));
  const bondLines = molecule.bonds
    .map((bond) => {
      const firstIndex = atomIndexById.get(bond.atomIds[0]);
      const secondIndex = atomIndexById.get(bond.atomIds[1]);
      if (firstIndex === undefined || secondIndex === undefined) return undefined;
      return `${formatMolInteger(firstIndex, 3)}${formatMolInteger(secondIndex, 3)}${formatMolInteger(bond.order, 3)}  0  0  0  0`;
    })
    .filter((line): line is string => line !== undefined);
  const chargeLines = formatMolChargeLines(molecule);

  return [
    molecule.name,
    "qm_editor",
    "",
    `${formatMolInteger(molecule.atoms.length, 3)}${formatMolInteger(bondLines.length, 3)}  0  0  0  0            999 V2000`,
    ...molecule.atoms.map(({ element, position }) => {
      const [x, y, z] = position;
      return `${formatMolFloat(x)}${formatMolFloat(y)}${formatMolFloat(z)} ${element.padEnd(3, " ")} 0  0  0  0  0  0  0  0  0  0  0  0`;
    }),
    ...bondLines,
    ...chargeLines,
    "M  END",
    "",
  ].join("\n");
}

function formatMolChargeLines(molecule: Molecule) {
  const entries = molecule.atoms
    .map((atom, index) => ({ atomIndex: index + 1, formalCharge: atom.formalCharge ?? 0 }))
    .filter((entry) => entry.formalCharge !== 0);

  const lines: string[] = [];
  for (let index = 0; index < entries.length; index += 8) {
    const chunk = entries.slice(index, index + 8);
    const values = chunk
      .map((entry) => `${formatMolInteger(entry.atomIndex, 4)}${formatMolInteger(entry.formalCharge, 4)}`)
      .join("");
    lines.push(`M  CHG${formatMolInteger(chunk.length, 3)}${values}`);
  }
  return lines;
}

function formatMolFloat(value: number) {
  return value.toFixed(4).padStart(10, " ");
}

function formatMolInteger(value: number, width: number) {
  return String(value).padStart(width, " ");
}

export default App;
