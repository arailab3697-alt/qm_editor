pub mod ai;
pub mod domain;
pub mod reducer;
pub mod validation;
pub mod parser;
pub mod gaussian;
pub mod geometry;
pub mod ai_commands;
pub mod templates;
pub mod fragments;

use domain::{AppState, ChemicalSpec, Command, Molecule, ValidationMessage, AiContext, AiResult, FragmentDefinition};
use reducer::{initial_app_state, reduce};
use parser::parse_molecule_file;
use gaussian::render_gaussian;
use validation::validate_chemical_spec;
use ai_commands::{build_ai_context, propose_commands_by_rules};
use fragments::list_available_fragments;

#[tauri::command]
fn list_available_fragments_tauri() -> Vec<FragmentDefinition> {
    list_available_fragments()
}

#[tauri::command]
fn inspect_fragment_tauri(fragment_name: String) -> Option<FragmentDefinition> {
    list_available_fragments().into_iter().find(|f| f.name == fragment_name)
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
fn build_ai_context_tauri(state: AppState, screenshot: Option<String>) -> AiContext {
    build_ai_context(&state, screenshot)
}

#[tauri::command]
async fn propose_commands_via_ai_tauri(
    input: String,
    state: AppState,
    screenshot: Option<String>,
) -> Result<AiResult, String> {
    let context = build_ai_context(&state, screenshot);
    let result = ai::propose_commands_via_ai(&input, &state, &context).await;
    result
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            list_available_fragments_tauri,
            inspect_fragment_tauri,
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
