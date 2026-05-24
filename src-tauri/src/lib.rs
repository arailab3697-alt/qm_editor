pub mod ai;
pub mod ai_commands;
pub mod domain;
pub mod fragments;
pub mod gaussian;
pub mod geometry;
pub mod parser;
pub mod reducer;
pub mod templates;
pub mod validation;

use ai_commands::{build_ai_context, resolve_atom_references};
use domain::{
    AiContext, AiProposal, AppState, ChemicalSpec, Command, FragmentDefinition, Molecule,
    SubstituteByFragmentCompletion, ValidationMessage,
};
use fragments::list_available_fragments;
use gaussian::render_gaussian;
use parser::parse_molecule_file;
use reducer::{infer_substitute_by_fragment_completion, initial_app_state, reduce};
use templates::{list_available_templates, TemplateSummary};
use validation::validate_chemical_spec;

#[tauri::command]
fn list_available_templates_tauri() -> Vec<TemplateSummary> {
    list_available_templates()
}

#[tauri::command]
fn list_available_fragments_tauri() -> Vec<FragmentDefinition> {
    list_available_fragments()
}

#[tauri::command]
fn inspect_fragment_tauri(fragment_name: String) -> Option<FragmentDefinition> {
    list_available_fragments()
        .into_iter()
        .find(|f| f.name == fragment_name)
}

#[tauri::command]
fn infer_substitute_by_fragment_completion_tauri(
    molecule: Molecule,
    selected_atom_id: u32,
) -> Option<SubstituteByFragmentCompletion> {
    infer_substitute_by_fragment_completion(&molecule, selected_atom_id)
}

#[tauri::command]
fn get_initial_app_state() -> AppState {
    initial_app_state()
}
// ... (rest of the file)

#[tauri::command]
fn apply_command(state: AppState, command: Command) -> AppState {
    reduce(state, command)
}

#[tauri::command]
fn apply_commands(state: AppState, commands: Vec<Command>) -> AppState {
    commands.into_iter().fold(state, |current_state, command| {
        reduce(current_state, command)
    })
}

#[tauri::command]
fn parse_molecule_file_tauri(file_name: String, text: String) -> Result<Molecule, String> {
    parse_molecule_file(&file_name, &text)
}

#[tauri::command]
fn render_gaussian_tauri(spec: ChemicalSpec) -> String {
    render_gaussian(&spec)
}

#[tauri::command]
fn validate_chemical_spec_tauri(spec: ChemicalSpec) -> Vec<ValidationMessage> {
    validate_chemical_spec(&spec)
}

#[tauri::command]
fn build_ai_context_tauri(state: AppState) -> AiContext {
    build_ai_context(&state)
}

#[tauri::command]
async fn propose_commands_via_ai_tauri(
    input: String,
    state: AppState,
    screenshot: Option<String>,
) -> Result<AiProposal, String> {
    let context = build_ai_context(&state);
    let result = ai::propose_commands_via_ai(&input, &state, &context, screenshot).await?;
    let resolved_commands = resolve_atom_references(result.commands.clone(), &context)?;
    Ok(AiProposal {
        commands: result.commands,
        resolved_commands,
        explanation: result.explanation,
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            list_available_templates_tauri,
            list_available_fragments_tauri,
            inspect_fragment_tauri,
            infer_substitute_by_fragment_completion_tauri,
            get_initial_app_state,
            apply_command,
            apply_commands,
            parse_molecule_file_tauri,
            render_gaussian_tauri,
            validate_chemical_spec_tauri,
            build_ai_context_tauri,
            propose_commands_via_ai_tauri
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
