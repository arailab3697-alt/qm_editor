pub mod ai;
pub mod ai_commands;
pub mod domain;
pub mod fragments;
pub mod functional_group_patterns;
pub mod functional_groups;
pub mod gaussian;
pub mod geometry;
pub mod parser;
pub mod reducer;
pub mod templates;
pub mod validation;

use ai_commands::{build_ai_context, resolve_atom_references};
use domain::{
    AiContext, AiProposal, AppState, ChemicalSpec, Command, FragmentDefinition, Molecule,
    SubstituteByFragmentCompletion, ValidationMessage, YoloPlanStep, YoloStepHistoryEntry,
    YoloStepProposal,
};
use fragments::list_available_fragments;
use functional_groups::{
    match_functional_groups, ordered_benzene_ring_carbons, FunctionalGroupMatch,
};
use gaussian::render_gaussian;
use parser::parse_molecule_file;
use reducer::{infer_substitute_by_fragment_completion, initial_app_state, reduce};
use templates::{list_available_templates, TemplateSummary};
use validation::validate_chemical_spec;

#[tauri::command]
#[specta::specta]
fn list_available_templates_tauri() -> Vec<TemplateSummary> {
    list_available_templates()
}

#[tauri::command]
#[specta::specta]
fn list_available_fragments_tauri() -> Vec<FragmentDefinition> {
    list_available_fragments()
}

#[tauri::command]
#[specta::specta]
fn inspect_fragment_tauri(fragment_name: String) -> Option<FragmentDefinition> {
    list_available_fragments()
        .into_iter()
        .find(|f| f.name == fragment_name)
}

#[tauri::command]
#[specta::specta]
fn infer_substitute_by_fragment_completion_tauri(
    molecule: Molecule,
    selected_atom_id: u32,
) -> Option<SubstituteByFragmentCompletion> {
    infer_substitute_by_fragment_completion(&molecule, selected_atom_id)
}

#[tauri::command]
#[specta::specta]
fn match_functional_groups_tauri(molecule: Molecule) -> Vec<FunctionalGroupMatch> {
    match_functional_groups(&molecule)
}

#[tauri::command]
#[specta::specta]
fn ordered_benzene_ring_carbons_tauri(molecule: Molecule) -> Option<Vec<u32>> {
    ordered_benzene_ring_carbons(&molecule)
}

#[tauri::command]
#[specta::specta]
fn get_initial_app_state() -> AppState {
    initial_app_state()
}
// ... (rest of the file)

#[tauri::command]
#[specta::specta]
fn apply_command(state: AppState, command: Command) -> AppState {
    reduce(state, command)
}

#[tauri::command]
#[specta::specta]
fn apply_commands(state: AppState, commands: Vec<Command>) -> AppState {
    commands.into_iter().fold(state, |current_state, command| {
        reduce(current_state, command)
    })
}

#[tauri::command]
#[specta::specta]
fn parse_molecule_file_tauri(file_name: String, text: String) -> Result<Molecule, String> {
    parse_molecule_file(&file_name, &text)
}

#[tauri::command]
#[specta::specta]
fn render_gaussian_tauri(spec: ChemicalSpec) -> String {
    render_gaussian(&spec)
}

#[tauri::command]
#[specta::specta]
fn validate_chemical_spec_tauri(spec: ChemicalSpec) -> Vec<ValidationMessage> {
    validate_chemical_spec(&spec)
}

#[tauri::command]
#[specta::specta]
fn build_ai_context_tauri(state: AppState) -> AiContext {
    build_ai_context(&state)
}

#[tauri::command]
#[specta::specta]
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

#[tauri::command]
#[specta::specta]
fn plan_yolo_steps_tauri(input: String) -> Vec<YoloPlanStep> {
    ai::build_yolo_plan(&input)
}

#[tauri::command]
#[specta::specta]
async fn propose_yolo_step_tauri(
    input: String,
    state: AppState,
    screenshot: Option<String>,
    plan: Vec<YoloPlanStep>,
    step: YoloPlanStep,
    history: Vec<YoloStepHistoryEntry>,
) -> Result<YoloStepProposal, String> {
    let context = build_ai_context(&state);
    let (prompt, result) = ai::propose_yolo_step_commands(
        &input, &state, &context, screenshot, &plan, &step, &history,
    )
    .await?;
    let resolved_commands = resolve_atom_references(result.commands.clone(), &context)?;
    Ok(YoloStepProposal {
        prompt,
        commands: result.commands,
        resolved_commands,
        explanation: result.explanation,
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let specta_builder = tauri_specta::Builder::<tauri::Wry>::new().commands(tauri_specta::collect_commands![
        list_available_templates_tauri,
        list_available_fragments_tauri,
        inspect_fragment_tauri,
        infer_substitute_by_fragment_completion_tauri,
        match_functional_groups_tauri,
        ordered_benzene_ring_carbons_tauri,
        get_initial_app_state,
        apply_command,
        apply_commands,
        parse_molecule_file_tauri,
        render_gaussian_tauri,
        validate_chemical_spec_tauri,
        build_ai_context_tauri,
        propose_commands_via_ai_tauri,
        plan_yolo_steps_tauri,
        propose_yolo_step_tauri
    ]);

    #[cfg(debug_assertions)]
    specta_builder
        .export(specta_typescript::Typescript::default(), "../src/bindings.ts")
        .expect("Failed to export typescript bindings");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(specta_builder.invoke_handler())
        .setup(move |app| {
            specta_builder.mount_events(app);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
