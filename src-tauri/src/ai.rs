use rig::{
    completion::{Prompt, ToolDefinition},
    prelude::*,
    providers::gemini,
    tool::Tool,
};
use serde::{Deserialize, Serialize};

mod checker;
use checker::Checker;

use crate::ai_commands::{parse_ai_result_json, propose_commands_by_rules as propose_by_rules};
use crate::domain::{
    AiContext, AiResult, AppState, AtomSummary, CalculationSpec, CalculationSummary, Element,
    FragmentDefinition, MassNumber, TwiceSpin,
};
use crate::fragments;
use crate::templates;

#[derive(Deserialize, Serialize)]
struct ListTemplatesArgs {}

#[derive(Deserialize, Serialize)]
struct InspectTemplateArgs {
    name: String,
}

#[derive(Deserialize, Serialize)]
struct ListFragmentsArgs {}

#[derive(Deserialize, Serialize)]
struct InspectFragmentArgs {
    fragment_name: String,
}
#[derive(Deserialize, Serialize)]
struct UseScreenshotArgs {}

#[derive(Debug, thiserror::Error)]
#[error("Tool error")]
struct ToolError;

#[derive(Serialize, Deserialize)]
struct ListTemplates;

impl Tool for ListTemplates {
    const NAME: &'static str = "list_available_templates";
    type Error = ToolError;
    type Args = ListTemplatesArgs;
    type Output = Vec<templates::TemplateSummary>;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "list_available_templates".to_string(),
            description: "List all available chemical templates that can be added to the molecule."
                .to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        }
    }

    async fn call(&self, _args: Self::Args) -> Result<Self::Output, Self::Error> {
        Ok(templates::list_available_templates())
    }
}

#[derive(Serialize, Deserialize)]
struct InspectTemplate;

impl Tool for InspectTemplate {
    const NAME: &'static str = "inspect_template";
    type Error = ToolError;
    type Args = InspectTemplateArgs;
    type Output = Option<templates::TemplateDefinition>;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "inspect_template".to_string(),
            description: "Get detailed information about a specific template.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "name": { "type": "string", "description": "The name of the template." }
                },
                "required": ["name"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        Ok(templates::get_template_definition(&args.name))
    }
}

#[derive(Serialize, Deserialize)]
struct ListFragments;

impl Tool for ListFragments {
    const NAME: &'static str = "list_available_fragments";
    type Error = ToolError;
    type Args = ListFragmentsArgs;
    type Output = Vec<FragmentDefinition>;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "list_available_fragments".to_string(),
            description: "List all available fragments for structural composition.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        }
    }

    async fn call(&self, _args: Self::Args) -> Result<Self::Output, Self::Error> {
        Ok(fragments::list_available_fragments())
    }
}

#[derive(Serialize, Deserialize)]
struct InspectFragment;

impl Tool for InspectFragment {
    const NAME: &'static str = "inspect_fragment";
    type Error = ToolError;
    type Args = InspectFragmentArgs;
    type Output = Option<FragmentDefinition>;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "inspect_fragment".to_string(),
            description: "Inspect a fragment structure. Use an 'atom' type port for 'ATTACH_FRAGMENT' and 'bond' type for 'SUBSTITUTE_BY_FRAGMENT'.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "fragmentName": { "type": "string", "description": "The name of the fragment." }
                },
                "required": ["fragmentName"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        Ok(fragments::list_available_fragments()
            .into_iter()
            .find(|f| f.name == args.fragment_name))
    }
}

#[derive(Serialize, Deserialize)]
struct UseScreenshotContext;

impl Tool for UseScreenshotContext {
    const NAME: &'static str = "use_screenshot_context";
    type Error = ToolError;
    type Args = UseScreenshotArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "use_screenshot_context".to_string(),
            description: "Use this tool when visual context from the latest molecule canvas screenshot is needed.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        }
    }

    async fn call(&self, _args: Self::Args) -> Result<Self::Output, Self::Error> {
        Ok("Screenshot context is available in request.screenshot when provided.".to_string())
    }
}

// const DEFAULT_GEMINI_MODEL: &str = "gemini-2.5-flash";
const DEFAULT_GEMINI_MODEL: &str = "gemini-3.1-flash-lite-preview";

#[derive(Clone, Copy, Debug)]
enum AiProvider {
    GoogleGemini,
}

impl AiProvider {
    fn from_env() -> Result<Self, String> {
        match std::env::var("QM_EDITOR_AI_PROVIDER")
            .unwrap_or_else(|_| "google-gemini".to_string())
            .trim()
            .to_ascii_lowercase()
            .as_str()
        {
            "" | "google" | "google-gemini" | "gemini" => Ok(Self::GoogleGemini),
            provider => Err(format!("Unsupported AI provider '{provider}'.")),
        }
    }
}

pub async fn propose_commands_via_ai(
    input: &str,
    _state: &AppState,
    context: &AiContext,
    screenshot: Option<String>,
) -> Result<AiResult, String> {
    if input.trim().is_empty() {
        return Ok(propose_by_rules(input, context));
    }

    if let Ok(result) = parse_ai_result_json(input.trim()) {
        return Ok(result);
    }

    if let Some(result) = local_result_for_supported_request(input, context) {
        return Ok(result);
    }

    match AiProvider::from_env()? {
        AiProvider::GoogleGemini => propose_with_gemini(input, _state, context, screenshot).await,
    }
}

fn local_result_for_supported_request(input: &str, context: &AiContext) -> Option<AiResult> {
    let result = propose_by_rules(input, context);
    (!result.commands.is_empty()).then_some(result)
}

async fn propose_with_gemini(
    input: &str,
    state: &AppState,
    context: &AiContext,
    screenshot: Option<String>,
) -> Result<AiResult, String> {
    let api_key = std::env::var("GEMINI_API_KEY")
        .or_else(|_| std::env::var("GOOGLE_API_KEY"))
        .map_err(|_| "Set GEMINI_API_KEY or GOOGLE_API_KEY to use the AI assistant.".to_string())?;
    let client = gemini::Client::new(api_key).map_err(|error| error.to_string())?;
    let model = std::env::var("QM_EDITOR_GEMINI_MODEL")
        .unwrap_or_else(|_| DEFAULT_GEMINI_MODEL.to_string());
    let max_turns = 10;
    let agent = client
        .agent(model)
        .preamble(system_prompt())
        .tool(ListTemplates)
        .tool(InspectTemplate)
        .tool(ListFragments)
        .tool(InspectFragment)
        .tool(UseScreenshotContext)
        .temperature(0.0)
        .build();

    let mut prompt = build_prompt(input, state, context, screenshot)?;
    let checker = Checker::new();

    let retry_count = 3;

    for attempt in 0..retry_count {
        let response = agent
            .prompt(prompt.clone())
            .max_turns(max_turns)
            .await
            .map_err(|error| error.to_string())?;

        // for debug, do not erase this
        println!("gemini says (attempt {}): {:?}", attempt + 1, response);

        match checker.check(&response, context) {
            Ok(output) => return Ok(output.result),
            Err(diag) => {
                if attempt == 2 {
                    return Err(format!(
                        "AI failed to produce valid output after 3 attempts. Diagnostics: {:?}",
                        diag.diagnostics
                    ));
                }
                prompt = format!(
                    "Your previous response failed validation. Please fix it according to these diagnostics:\n{}\nRepair Policy: {:?}",
                    serde_json::to_string_pretty(&diag.diagnostics).unwrap_or_else(|_| "Error serializing diagnostics".to_string()),
                    diag.repair_policy
                );
            }
        }
    }

    Err("Unexpected end of validation loop".to_string())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PromptPayload<'a> {
    request: &'a str,
    state: AiVisibleState<'a>,
    context: AiVisibleContext,
    screenshot: Option<&'a str>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AiVisibleState<'a> {
    domain: AiVisibleDomain<'a>,
    ui: AiVisibleUi,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AiVisibleDomain<'a> {
    chemical_spec: AiVisibleChemicalSpec<'a>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AiVisibleChemicalSpec<'a> {
    molecule: AiVisibleMolecule,
    calculation: &'a CalculationSpec,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AiVisibleMolecule {
    name: String,
    atoms: Vec<AiVisibleAtom>,
    bonds: Vec<AiVisibleBond>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AiVisibleAtom {
    id: u32,
    element: Element,
    isotope: Option<MassNumber>,
    nuclear_spin: Option<TwiceSpin>,
    formal_charge: i32,
    position: [f64; 3],
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AiVisibleBond {
    id: u32,
    atom_ids: [u32; 2],
    order: u8,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AiVisibleUi {
    selected_atoms: Vec<u32>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AiVisibleContext {
    selected_atoms: Vec<AtomSummary>,
    atom_index_map: Vec<AiVisibleAtomIndexMapEntry>,
    atom_context_map: std::collections::HashMap<u32, String>,
    calculation: CalculationSummary,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AiVisibleAtomIndexMapEntry {
    display_index: u32,
    atom_id: u32,
}

fn build_prompt(
    input: &str,
    state: &AppState,
    context: &AiContext,
    screenshot: Option<String>,
) -> Result<String, String> {
    let payload = PromptPayload {
        request: input,
        state: build_ai_visible_state(state),
        context: build_ai_visible_context(context),
        screenshot: screenshot.as_deref(),
    };
    println!("{:?}", payload.context.atom_context_map);
    serde_json::to_string_pretty(&payload).map_err(|error| error.to_string())
}

fn build_ai_visible_context(context: &AiContext) -> AiVisibleContext {
    AiVisibleContext {
        selected_atoms: context.selected_atoms.clone(),
        atom_index_map: context
            .atom_index_map
            .iter()
            .map(|entry| AiVisibleAtomIndexMapEntry {
                display_index: entry.display_index,
                atom_id: entry.display_index,
            })
            .collect(),
        atom_context_map: context.atom_context_map.clone(),
        calculation: context.calculation.clone(),
    }
}

fn build_ai_visible_state(state: &AppState) -> AiVisibleState<'_> {
    let molecule = &state.domain.chemical_spec.molecule;
    let display_index_for_atom_id = molecule
        .atoms
        .iter()
        .enumerate()
        .map(|(index, atom)| (atom.id, index as u32 + 1))
        .collect::<std::collections::HashMap<_, _>>();

    let atoms = molecule
        .atoms
        .iter()
        .enumerate()
        .map(|(index, atom)| AiVisibleAtom {
            id: index as u32 + 1,
            element: atom.element,
            isotope: atom.isotope,
            nuclear_spin: atom.nuclear_spin,
            formal_charge: atom.formal_charge,
            position: atom.position,
        })
        .collect();

    let bonds = molecule
        .bonds
        .iter()
        .enumerate()
        .filter_map(|(index, bond)| {
            let first_atom_id = display_index_for_atom_id.get(&bond.atom_ids[0])?;
            let second_atom_id = display_index_for_atom_id.get(&bond.atom_ids[1])?;
            Some(AiVisibleBond {
                id: index as u32 + 1,
                atom_ids: [*first_atom_id, *second_atom_id],
                order: bond.order,
            })
        })
        .collect();

    let selected_atoms = state
        .ui
        .selected_atoms
        .iter()
        .filter_map(|atom_id| display_index_for_atom_id.get(atom_id).copied())
        .collect();

    AiVisibleState {
        domain: AiVisibleDomain {
            chemical_spec: AiVisibleChemicalSpec {
                molecule: AiVisibleMolecule {
                    name: molecule.name.clone(),
                    atoms,
                    bonds,
                },
                calculation: &state.domain.chemical_spec.calculation,
            },
        },
        ui: AiVisibleUi { selected_atoms },
    }
}

fn system_prompt() -> &'static str {
    r#"You are a JSON command generator for a molecular Gaussian editor.

Your response MUST be valid raw JSON.

Do NOT wrap the JSON in markdown code fences.
Do NOT output ```json.
Do NOT output explanations before or after the JSON.
Do NOT output natural language outside the JSON object.

The FIRST character of your response MUST be '{'.
The LAST character of your response MUST be '}'.

If you cannot fulfill the request, return:
{"result":{"commands":[],"explanation":"reason"},"ignoredWarning":[]}

Return exactly one JSON object with this shape:
{
  "result": {
    "commands": [],
    "explanation": "short explanation"
  },
  "ignoredWarning": []
}

Allowed command types and their fields:
- SET_METHOD: {"type": "SET_METHOD", "method": "B3LYP" | "WB97XD"}
- SET_BASIS: {"type": "SET_BASIS", "basis": "6-31G(d)" | "def2-SVP" | "def2-TZVP"}
- SET_JOB_TYPE: {"type": "SET_JOB_TYPE", "jobType": "opt" | "freq" | "opt+freq" | "ts"}
- SET_SOLVENT: {"type": "SET_SOLVENT", "solvent": "THF" | "Water" | null}
- SET_CHARGE: {"type": "SET_CHARGE", "charge": number}
- SET_MULTIPLICITY: {"type": "SET_MULTIPLICITY", "multiplicity": number}
- SET_BOND_LENGTH: {"type": "SET_BOND_LENGTH", "atomIds": [id1, id2], "length": number, "mode"?: "ATOM_ONLY" | "MOVE_OTHER_SIDE" | "MOVE_BOTH_SIDES"}
- SET_BOND_ANGLE: {"type": "SET_BOND_ANGLE", "atomIds": [id1, id2, id3], "angle": number, "mode"?: "ATOM_ONLY" | "MOVE_OTHER_SIDE" | "MOVE_BOTH_SIDES"}
- SET_DIHEDRAL_ANGLE: {"type": "SET_DIHEDRAL_ANGLE", "atomIds": [id1, id2, id3, id4], "angle": number, "mode"?: "ATOM_ONLY" | "MOVE_OTHER_SIDE" | "MOVE_BOTH_SIDES"}
- ADD_ATOM: {"type": "ADD_ATOM", "element": string, "position": [x, y, z], "isotope"?: number, "nuclearSpin"?: number}
- SET_ATOM_FORMAL_CHARGE: {"type": "SET_ATOM_FORMAL_CHARGE", "atomId": number, "formalCharge": number}
- DELETE_ATOM: {"type": "DELETE_ATOM", "atomId": number}
- ADD_BOND: {"type": "ADD_BOND", "atomIds": [id1, id2], "order": 1 | 2 | 3}
- DELETE_BOND: {"type": "DELETE_BOND", "bondId": number}
- PLACE_TEMPLATE: {"type": "PLACE_TEMPLATE", "templateName": string, "position": [x, y, z], "direction": [dx, dy, dz]}
- ATTACH_FRAGMENT: {"type": "ATTACH_FRAGMENT", "fragmentName": string, "targetAtomId": number, "rotationAngle": number, "orientation": [x, y, z]}
- SUBSTITUTE_BY_FRAGMENT: {"type": "SUBSTITUTE_BY_FRAGMENT", "fragmentName": string, "startAtomId": number, "endAtomId": number}; startAtomId is the existing atom to remove/replace, endAtomId is the bonded existing atom to keep/connect.

Use camelCase fields exactly as shown. 
- Always list available templates/fragments first if the user wants to add/substitute them.
- Use 'inspect_template' for standard molecules.
- Use 'inspect_fragment' to decide between ATTACH_FRAGMENT and SUBSTITUTE_BY_FRAGMENT.
- If a fragment has a 'bond' type port, SUBSTITUTE_BY_FRAGMENT must be used as the preferred method for integrating fragments.
- Only use ATTACH_FRAGMENT if a 'bond' type port is not available and an 'atom' type port exists.
- SUBSTITUTE_BY_FRAGMENT provides a more chemically accurate integration by replacing existing bonds.
- Never include markdown, comments, or extra text in your final JSON response.
- The state shown to you uses display IDs only: molecule.atoms[].id, molecule.bonds[].atomIds, and ui.selectedAtoms are all 1-based display atom IDs in current atom order.
- The context also uses display IDs only. You MUST use these display IDs in all atom reference fields of output commands; the application resolves them to stable internal atomId values after your response."#
}

fn extract_json_object(text: &str) -> Option<&str> {
    let trimmed = text.trim();
    if trimmed.starts_with('{') && trimmed.ends_with('}') {
        return Some(trimmed);
    }

    let start = trimmed.find('{')?;
    let end = trimmed.rfind('}')?;
    (start < end).then(|| &trimmed[start..=end])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai_commands;
    use crate::domain::{self, Command, Method};
    use crate::reducer;

    #[test]
    fn uses_local_parser_for_supported_short_request() {
        let state = reducer::initial_app_state();
        let context = ai_commands::build_ai_context(&state);
        let result = local_result_for_supported_request("set wb97xd", &context)
            .expect("supported short request should be handled locally");

        assert!(matches!(
            result.commands.as_slice(),
            [Command::SetMethod {
                method: Method::WB97XD
            }]
        ));
    }

    #[test]
    fn prompt_state_uses_display_atom_ids_for_atoms_bonds_and_selection() {
        let mut state = reducer::initial_app_state();
        state.domain.chemical_spec.molecule.atoms[0].id = 10;
        state.domain.chemical_spec.molecule.atoms[1].id = 20;
        state.domain.chemical_spec.molecule.atoms[2].id = 30;
        state.domain.chemical_spec.molecule.bonds[0].atom_ids = [10, 20];
        state.domain.chemical_spec.molecule.bonds[1].atom_ids = [10, 30];
        state.ui.selected_atoms = vec![20];

        let context = ai_commands::build_ai_context(&state);
        let prompt = build_prompt("inspect", &state, &context, None).expect("prompt should build");
        let json: serde_json::Value = serde_json::from_str(&prompt).expect("prompt should be JSON");

        assert_eq!(
            json["state"]["domain"]["chemicalSpec"]["molecule"]["atoms"][0]["id"],
            1
        );
        assert_eq!(
            json["state"]["domain"]["chemicalSpec"]["molecule"]["atoms"][1]["id"],
            2
        );
        assert_eq!(
            json["state"]["domain"]["chemicalSpec"]["molecule"]["bonds"][0]["atomIds"],
            serde_json::json!([1, 2])
        );
        assert_eq!(
            json["state"]["domain"]["chemicalSpec"]["molecule"]["bonds"][1]["atomIds"],
            serde_json::json!([1, 3])
        );
        assert_eq!(json["state"]["ui"]["selectedAtoms"], serde_json::json!([2]));
        assert_eq!(
            json["context"]["atomIndexMap"],
            serde_json::json!([
                { "displayIndex": 1, "atomId": 1 },
                { "displayIndex": 2, "atomId": 2 },
                { "displayIndex": 3, "atomId": 3 }
            ])
        );
    }

    #[test]
    fn prompt_context_includes_chemical_context() {
        let mut state = reducer::initial_app_state();
        // Setup benzoic acid-like structure for testing
        state.domain.chemical_spec.molecule.atoms = vec![
            crate::domain::Atom { id: 1, element: crate::domain::Element::C, isotope: None, nuclear_spin: None, formal_charge: 0, position: [0.0, 0.0, 0.0] },
            crate::domain::Atom { id: 2, element: crate::domain::Element::O, isotope: None, nuclear_spin: None, formal_charge: 0, position: [1.2, 0.0, 0.0] },
            crate::domain::Atom { id: 3, element: crate::domain::Element::O, isotope: None, nuclear_spin: None, formal_charge: 0, position: [0.0, 1.3, 0.0] },
            crate::domain::Atom { id: 4, element: crate::domain::Element::H, isotope: None, nuclear_spin: None, formal_charge: 0, position: [0.0, 2.2, 0.0] },
        ];
        state.domain.chemical_spec.molecule.bonds = vec![
            crate::domain::Bond { id: 1, atom_ids: [1, 2], order: 2 },
            crate::domain::Bond { id: 2, atom_ids: [1, 3], order: 1 },
            crate::domain::Bond { id: 3, atom_ids: [3, 4], order: 1 },
        ];
        state.ui.selected_atoms = vec![1];

        let context = ai_commands::build_ai_context(&state);
        let prompt = build_prompt("inspect", &state, &context, None).expect("prompt should build");
        let json: serde_json::Value = serde_json::from_str(&prompt).expect("prompt should be JSON");

        let selected_atom = &json["context"]["selectedAtoms"][0];
        assert!(selected_atom["chemicalContext"].is_string());
        assert!(selected_atom["chemicalContext"].as_str().unwrap().contains("CarboxylicAcid"));
    }
}
