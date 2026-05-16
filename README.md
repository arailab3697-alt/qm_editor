# QM Editor (Gaussian AI Input Editor)

QM Editor is a modern desktop application designed for creating Gaussian input files through a structured, AI-assisted interface. It combines interactive 3D molecule visualization with a domain-driven state model to ensure the generation of valid computational chemistry specifications.

## 🚀 Key Features

*   **Structured State Model:** Unlike traditional text editors, QM Editor treats Gaussian input as a rendered projection of a structured domain state, preventing syntax errors.
*   **AI-Assisted Command Generation:** A hybrid AI system (local heuristics + LLM) interprets natural language requests to propose state mutations.
*   **3D Molecule Visualization:** Interactive visualization and atom selection using 3Dmol.js.
*   **Molecule Import:** Support for `.xyz` and `.mol` file formats.
*   **Geometry Manipulation:** Precisely adjust bond lengths, angles, and dihedral angles.
*   **Real-time Validation:** Automated checks for charge and multiplicity consistency.
*   **Gaussian Input Preview:** Live rendering of the Gaussian `.com` file based on the current domain state.

## 🏗️ Architecture

The application follows a strict **Command-Reducer** pattern to manage state mutations, bridging a React frontend with a Rust backend using Tauri.

### Tech Stack

*   **Frontend:** React 19, TypeScript, Vite, Zustand (State), 3Dmol.js (Visualization).
*   **Backend:** Rust, Tauri v2.
*   **AI Integration:** [Rig](https://github.com/0xPlayground/rig) (Rust library), Google Gemini (default LLM provider).

### Core Design Principles

1.  **State is Sovereign:** The Gaussian input text is always derived from the internal domain model.
2.  **Semantic Commands:** All mutations must pass through validated semantic commands.
3.  **AI as a Proposer:** AI generates commands but never directly mutates the application state.
4.  **Shared Context:** The GUI and AI share the same semantic context (selected atoms, calculation specs).

## 📁 Project Structure

```text
.
├── specs/                  # Detailed functional and architectural specifications
├── src/                    # Frontend (React + TypeScript)
│   ├── app/                # App-level logic and State (Zustand)
│   ├── domain/             # Shared domain types (mirrored in Rust)
│   ├── App.tsx             # Main UI component and layout
│   └── main.tsx            # React entry point
└── src-tauri/              # Backend (Rust)
    └── src/
        ├── domain/         # Core logic: State, Reducer, Parsers, Renderer, Validation
        ├── ai.rs           # AI Provider integration (Gemini/Rig)
        ├── ai_commands.rs  # AI command processing
        ├── lib.rs          # Tauri command definitions and entry point
        ├── main.rs         # Minimal binary entry
        └── ...             # Utility modules (geometry, gaussian, templates, etc.)
```

## 🧠 AI Command Engine

The AI integration relies on a hybrid approach in `src-tauri/src/ai_commands.rs`:

1.  **Rule-based Parser (`propose_commands_by_rules`):** Uses heuristic-based regex and token matching for fast, predictable state mutations (e.g., "set method b3lyp").
2.  **LLM Integration:** If local parsing fails, the system sends a structured `AiContext` (containing selected atoms and current calculation parameters) to the LLM to generate a sequence of valid `Command` objects.

### Core Domain Structs

The system is defined by a strict set of domain types (shared across Rust and TypeScript):

*   **`AppState`**: The root of the application state, managing the `domain` (chemistry) and `ui` (viewport) state.
*   **`Command`**: An enum defining all allowed mutations, such as `SetMethod`, `SetBasis`, `SetCharge`, `SetBondLength`, etc.
*   **`AiContext`**: A summary of the current chemical specification and active selections, provided to the AI for informed decision-making.
*   **`CalculationSummary`**: Stores parameters like `JobType`, `Method`, `Basis`, `Solvent`, `Charge`, and `Multiplicity`.
*   **`AtomSummary`**: Represents the state of selected atoms (ID, element, position) used for geometric manipulations.

## 🔄 Core Workflows

### Command Dispatch
1.  Frontend captures user interaction (e.g., clicking an atom, typing in AI panel).
2.  A `Command` object is created.
3.  The command is sent to the Rust backend via Tauri `invoke`.
4.  The Rust `reduce` function applies the command to the current state and returns the new state.
5.  Frontend updates the Zustand store with the new state.

### AI Command Generation
1.  User enters a natural language request (e.g., "Set method to B3LYP and basis to 6-31G(d)").
2.  The request is sent to the backend along with the current `AiContext`.
3.  **Local Path:** A regex-based parser checks for simple, supported requests.
4.  **Remote Path:** If local parsing fails, the request is sent to an LLM (Gemini) with a structured system prompt.
5.  The AI returns a list of JSON-formatted commands.
6.  The user can review and apply these commands.

## 🛠️ Development Setup

### Prerequisites

*   [Node.js](https://nodejs.org/) (Latest LTS)
*   [Rust](https://www.rust-lang.org/) (Latest stable)
*   Tauri dependencies (see [Tauri setup guide](https://tauri.app/v1/guides/getting-started/prerequisites))

### Environment Variables

To use the AI assistant, set one of the following:

*   `GEMINI_API_KEY` or `GOOGLE_API_KEY`

Optional configuration:
*   `QM_EDITOR_AI_PROVIDER`: `google-gemini` (default)
*   `QM_EDITOR_GEMINI_MODEL`: `gemini-2.5-flash-lite` (default)

### Running the App

```bash
# Install dependencies
npm install

# Run in development mode
npm run tauri dev
```

## 📜 License

This project is private and intended for MVP development.
