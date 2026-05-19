use rig::{completion::{Prompt, ToolDefinition}, prelude::*, providers::gemini, tool::Tool};
use serde::{Deserialize, Serialize};

use crate::domain::{AiContext, AiResult, AppState, FragmentDefinition};
use crate::ai_commands::{propose_commands_by_rules as propose_by_rules, parse_ai_result_json};
use crate::templates;
use crate::fragments;

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
        Ok(fragments::list_available_fragments().into_iter().find(|f| f.name == args.fragment_name))
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
    let prompt = build_prompt(input, state, context, screenshot)?;
    let response = agent
        .prompt(prompt)
        .max_turns(max_turns)
        .await
        .map_err(|error| error.to_string())?;

    // for debug, do not erase this
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
    screenshot: Option<&'a str>,
}

fn build_prompt(input: &str, state: &AppState, context: &AiContext, screenshot: Option<String>) -> Result<String, String> {
    let payload = PromptPayload {
        request: input,
        state,
        context,
        screenshot: screenshot.as_deref(),
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
- SET_BOND_LENGTH: {"type": "SET_BOND_LENGTH", "atomIds": [id1, id2], "length": number, "mode"?: "ATOM_ONLY" | "MOVE_OTHER_SIDE" | "MOVE_BOTH_SIDES"}
- SET_BOND_ANGLE: {"type": "SET_BOND_ANGLE", "atomIds": [id1, id2, id3], "angle": number, "mode"?: "ATOM_ONLY" | "MOVE_OTHER_SIDE" | "MOVE_BOTH_SIDES"}
- SET_DIHEDRAL_ANGLE: {"type": "SET_DIHEDRAL_ANGLE", "atomIds": [id1, id2, id3, id4], "angle": number, "mode"?: "ATOM_ONLY" | "MOVE_OTHER_SIDE" | "MOVE_BOTH_SIDES"}
- ADD_ATOM: {"type": "ADD_ATOM", "element": string, "position": [x, y, z], "isotope"?: number, "nuclearSpin"?: number}
- DELETE_ATOM: {"type": "DELETE_ATOM", "atomId": number}
- ADD_BOND: {"type": "ADD_BOND", "atomIds": [id1, id2], "order": 1 | 2 | 3}
- DELETE_BOND: {"type": "DELETE_BOND", "bondId": number}
- PLACE_TEMPLATE: {"type": "PLACE_TEMPLATE", "templateName": string, "position": [x, y, z], "direction": [dx, dy, dz]}
- ATTACH_FRAGMENT: {"type": "ATTACH_FRAGMENT", "fragmentName": string, "targetAtomId": number, "rotationAngle": number, "orientation": [x, y, z]}
- SUBSTITUTE_BY_FRAGMENT: {"type": "SUBSTITUTE_BY_FRAGMENT", "fragmentName": string, "startAtomId": number, "endAtomId": number}

Use camelCase fields exactly as shown. 
- Always list available templates/fragments first if the user wants to add/substitute them.
- Use 'inspect_template' for standard molecules.
- Use 'inspect_fragment' to decide between ATTACH_FRAGMENT and SUBSTITUTE_BY_FRAGMENT.
- If a fragment has a 'bond' type port, SUBSTITUTE_BY_FRAGMENT must be used as the preferred method for integrating fragments.
- Only use ATTACH_FRAGMENT if a 'bond' type port is not available and an 'atom' type port exists.
- SUBSTITUTE_BY_FRAGMENT provides a more chemically accurate integration by replacing existing bonds.
- Never include markdown, comments, or extra text in your final JSON response.
- The context includes atomIndexMap as a two-layer ID map: displayIndex (1-based atom order shown to users / Gaussian rows) -> atomId (stable internal ID). You MUST use displayIndex values in all atom reference fields of output commands; the application resolves them to atomId during command interpretation."#
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
}
