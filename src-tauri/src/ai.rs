use rig::{completion::{Prompt, ToolDefinition}, prelude::*, providers::gemini, tool::Tool};
use serde::{Deserialize, Serialize};

use crate::domain::{AiContext, AiResult, AppState};
use crate::ai_commands::{propose_commands_by_rules as propose_by_rules, parse_ai_result_json};
use crate::templates;

const DEFAULT_GEMINI_MODEL: &str = "gemini-2.5-flash";
// const DEFAULT_GEMINI_MODEL: &str = "gemini-2.5-flash-lite";
// const DEFAULT_GEMINI_MODEL: &str = "gemma-4-26b-a4b-it";

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

#[derive(Deserialize, Serialize)]
struct ListTemplatesArgs {}

#[derive(Deserialize, Serialize)]
struct InspectTemplateArgs {
    name: String,
}

#[derive(Debug, thiserror::Error)]
#[error("Template error")]
struct TemplateError;

#[derive(Serialize, Deserialize)]
struct ListTemplates;

impl Tool for ListTemplates {
    const NAME: &'static str = "list_available_templates";
    type Error = TemplateError;
    type Args = ListTemplatesArgs;
    type Output = Vec<templates::TemplateSummary>;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "list_available_templates".to_string(),
            description: "List all available chemical templates that can be added to the molecule.".to_string(),
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
    type Error = TemplateError;
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

pub async fn propose_commands_via_ai(
    input: &str,
    _state: &AppState,
    context: &AiContext,
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
        AiProvider::GoogleGemini => propose_with_gemini(input, _state, context).await,
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
) -> Result<AiResult, String> {
    let api_key = std::env::var("GEMINI_API_KEY")
        .or_else(|_| std::env::var("GOOGLE_API_KEY"))
        .map_err(|_| "Set GEMINI_API_KEY or GOOGLE_API_KEY to use the AI assistant.".to_string())?;
    let client = gemini::Client::new(api_key).map_err(|error| error.to_string())?;
    let model = std::env::var("QM_EDITOR_GEMINI_MODEL")
        .unwrap_or_else(|_| DEFAULT_GEMINI_MODEL.to_string());
    let max_turns = 3;
    let agent = client
        .agent(model)
        .preamble(system_prompt())
        .tool(ListTemplates)
        .tool(InspectTemplate)
        .temperature(0.0)
        .build();
    let prompt = build_prompt(input, state, context)?;
    let response = agent
        .prompt(prompt)
        .max_turns(max_turns)
        .await
        .map_err(|error| error.to_string())?;
    println!("gemini says: {:?}", response);
    let json = extract_json_object(&response)
        .ok_or_else(|| "AI response did not contain a JSON object.".to_string())?;

    parse_ai_result_json(json)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PromptPayload<'a> {
    request: &'a str,
    state: &'a AppState,
    context: &'a AiContext,
}

fn build_prompt(input: &str, state: &AppState, context: &AiContext) -> Result<String, String> {
    let payload = PromptPayload {
        request: input,
        state,
        context,
    };
    serde_json::to_string_pretty(&payload).map_err(|error| error.to_string())
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
{"commands":[],"explanation":"reason"}

Return exactly one JSON object with this shape:
{"commands":[],"explanation":"short explanation"}

Allowed command types and their fields:
- SET_METHOD: {"type": "SET_METHOD", "method": "B3LYP" | "WB97XD"}
- SET_BASIS: {"type": "SET_BASIS", "basis": "6-31G(d)" | "def2-SVP" | "def2-TZVP"}
- SET_JOB_TYPE: {"type": "SET_JOB_TYPE", "jobType": "opt" | "freq" | "opt+freq" | "ts"}
- SET_SOLVENT: {"type": "SET_SOLVENT", "solvent": "THF" | "Water" | null}
- SET_CHARGE: {"type": "SET_CHARGE", "charge": number}
- SET_MULTIPLICITY: {"type": "SET_MULTIPLICITY", "multiplicity": number}
- SET_BOND_LENGTH: {"type": "SET_BOND_LENGTH", "atomIds": [id1, id2], "length": number}
- SET_BOND_ANGLE: {"type": "SET_BOND_ANGLE", "atomIds": [id1, id2, id3], "angle": number}
- SET_DIHEDRAL_ANGLE: {"type": "SET_DIHEDRAL_ANGLE", "atomIds": [id1, id2, id3, id4], "angle": number}
- ADD_ATOM: {"type": "ADD_ATOM", "element": string, "position": [x, y, z], "isotope"?: number, "nuclearSpin"?: number}
- DELETE_ATOM: {"type": "DELETE_ATOM", "atomId": number}
- ADD_BOND: {"type": "ADD_BOND", "atomIds": [id1, id2], "order": 1 | 2 | 3}
- DELETE_BOND: {"type": "DELETE_BOND", "bondId": number}
- PLACE_TEMPLATE: {"type": "PLACE_TEMPLATE", "templateName": string, "position": [x, y, z], "direction": [dx, dy, dz]}

Use camelCase fields exactly as shown. For geometry changes (length, angle, dihedral), use IDs from the provided selectedAtoms if they match the required count.
Before ANY PLACE_TEMPLATE command, you MUST call 'list_available_templates' to see what is available.
You can use 'inspect_template' to get detail of the template.
Never include markdown, comments, or extra text in your final JSON response."#
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
    use crate::domain::{self, Command, Method};
    use crate::reducer;
    use crate::ai_commands;

    #[test]
    fn uses_local_parser_for_supported_short_request() {
        let state = reducer::initial_app_state();
        let context = ai_commands::build_ai_context(&state, None);
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
    fn pattern_matching_is_exhaustive() {
        let cmd = Command::PlaceTemplate {
            template_name: "test".to_string(),
            position: [0.0, 0.0, 0.0],
            direction: [0.0, 0.0, 0.0],
        };
        match cmd {
            Command::SetMethod { .. } => {}
            Command::SetBasis { .. } => {}
            Command::SetJobType { .. } => {}
            Command::SetSolvent { .. } => {}
            Command::SetCharge { .. } => {}
            Command::SetMultiplicity { .. } => {}
            Command::SetBondLength { .. } => {}
            Command::SetBondAngle { .. } => {}
            Command::SetDihedralAngle { .. } => {}
            Command::AddAtom { .. } => {}
            Command::DeleteAtom { .. } => {}
            Command::AddBond { .. } => {}
            Command::DeleteBond { .. } => {}
            Command::PlaceTemplate { .. } => {}
            Command::SetMolecule { .. } => {}
            Command::ToggleAtomSelection { .. } => {}
            Command::ClearSelection => {}
        }
    }
}
