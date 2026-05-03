//! Compatibility mapping for OpenAI-style `/v1/chat/completions` providers.
//!
//! Codex internally uses Responses-shaped items. This module converts the
//! subset needed by OpenAI-compatible Chat Completions backends: messages,
//! function tool calls, and tool outputs.

use codex_api::ChatCompletionsRequest;
use codex_protocol::models::ContentItem;
use codex_protocol::models::FunctionCallOutputPayload;
use codex_protocol::models::ImageDetail;
use codex_protocol::models::ResponseItem;
use codex_protocol::openai_models::ModelInfo;
use codex_tools::ResponsesApiNamespaceTool;
use codex_tools::ResponsesApiTool;
use codex_tools::ToolSpec;
use serde_json::Value;
use serde_json::json;
use tracing::debug;

use crate::client_common::Prompt;

pub(crate) fn build_chat_completions_request(
    prompt: &Prompt,
    model_info: &ModelInfo,
) -> Result<ChatCompletionsRequest, serde_json::Error> {
    let mut messages = Vec::new();
    if !prompt.base_instructions.text.trim().is_empty() {
        messages.push(json!({
            "role": "system",
            "content": prompt.base_instructions.text,
        }));
    }
    messages.extend(chat_messages_from_response_items(
        &prompt.get_formatted_input(),
    ));

    let tools = create_tools_json_for_chat_completions(&prompt.tools)?;
    let has_tools = !tools.is_empty();
    Ok(ChatCompletionsRequest {
        model: model_info.slug.clone(),
        messages,
        tools,
        tool_choice: has_tools.then(|| "auto".to_string()),
        parallel_tool_calls: has_tools && prompt.parallel_tool_calls,
        stream: true,
    })
}

fn chat_messages_from_response_items(items: &[ResponseItem]) -> Vec<Value> {
    let mut messages = Vec::new();
    let mut pending_tool_calls = Vec::new();

    for item in items {
        match item {
            ResponseItem::Message { role, content, .. } => {
                flush_tool_calls(&mut messages, &mut pending_tool_calls);
                messages.push(json!({
                    "role": role,
                    "content": chat_content_from_content_items(content),
                }));
            }
            ResponseItem::FunctionCall {
                name,
                namespace,
                arguments,
                call_id,
                ..
            } => {
                pending_tool_calls.push(chat_tool_call(
                    call_id,
                    &chat_tool_name(namespace.as_deref(), name),
                    arguments,
                ));
            }
            ResponseItem::CustomToolCall {
                call_id,
                name,
                input,
                ..
            } => {
                pending_tool_calls.push(chat_tool_call(call_id, name, input));
            }
            ResponseItem::FunctionCallOutput { call_id, output }
            | ResponseItem::CustomToolCallOutput {
                call_id, output, ..
            } => {
                flush_tool_calls(&mut messages, &mut pending_tool_calls);
                messages.push(json!({
                    "role": "tool",
                    "tool_call_id": call_id,
                    "content": tool_output_to_text(output),
                }));
            }
            ResponseItem::Reasoning { .. }
            | ResponseItem::LocalShellCall { .. }
            | ResponseItem::ToolSearchCall { .. }
            | ResponseItem::ToolSearchOutput { .. }
            | ResponseItem::WebSearchCall { .. }
            | ResponseItem::ImageGenerationCall { .. }
            | ResponseItem::Compaction { .. }
            | ResponseItem::Other => {}
        }
    }

    flush_tool_calls(&mut messages, &mut pending_tool_calls);
    messages
}

fn flush_tool_calls(messages: &mut Vec<Value>, pending_tool_calls: &mut Vec<Value>) {
    if pending_tool_calls.is_empty() {
        return;
    }

    messages.push(json!({
        "role": "assistant",
        "content": "",
        "tool_calls": std::mem::take(pending_tool_calls),
    }));
}

fn chat_content_from_content_items(content: &[ContentItem]) -> Value {
    let has_image = content
        .iter()
        .any(|item| matches!(item, ContentItem::InputImage { .. }));
    if !has_image {
        return Value::String(content_items_to_text(content));
    }

    Value::Array(
        content
            .iter()
            .map(|item| match item {
                ContentItem::InputText { text } | ContentItem::OutputText { text } => json!({
                    "type": "text",
                    "text": text,
                }),
                ContentItem::InputImage { image_url, detail } => {
                    let mut image_url_json = json!({ "url": image_url });
                    if let Some(detail) = detail {
                        image_url_json["detail"] = image_detail_value(*detail);
                    }
                    json!({
                        "type": "image_url",
                        "image_url": image_url_json,
                    })
                }
            })
            .collect(),
    )
}

fn content_items_to_text(content: &[ContentItem]) -> String {
    content
        .iter()
        .filter_map(|item| match item {
            ContentItem::InputText { text } | ContentItem::OutputText { text } => {
                Some(text.as_str())
            }
            ContentItem::InputImage { .. } => None,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn image_detail_value(detail: ImageDetail) -> Value {
    serde_json::to_value(detail).unwrap_or(Value::String("auto".to_string()))
}

fn chat_tool_call(call_id: &str, name: &str, arguments: &str) -> Value {
    json!({
        "id": call_id,
        "type": "function",
        "function": {
            "name": name,
            "arguments": arguments,
        },
    })
}

fn tool_output_to_text(output: &FunctionCallOutputPayload) -> String {
    output.body.to_text().unwrap_or_default()
}

fn chat_tool_name(namespace: Option<&str>, name: &str) -> String {
    match namespace {
        Some(namespace) => format!("{namespace}{name}"),
        None => name.to_string(),
    }
}

fn create_tools_json_for_chat_completions(
    tools: &[ToolSpec],
) -> Result<Vec<Value>, serde_json::Error> {
    let mut tools_json = Vec::new();
    for tool in tools {
        match tool {
            ToolSpec::Function(tool) => {
                tools_json.push(chat_function_tool(tool.name.clone(), tool)?);
            }
            ToolSpec::Namespace(namespace) => {
                for tool in &namespace.tools {
                    match tool {
                        ResponsesApiNamespaceTool::Function(tool) => {
                            tools_json.push(chat_function_tool(
                                chat_tool_name(Some(namespace.name.as_str()), &tool.name),
                                tool,
                            )?);
                        }
                    }
                }
            }
            ToolSpec::ToolSearch { .. }
            | ToolSpec::LocalShell {}
            | ToolSpec::ImageGeneration { .. }
            | ToolSpec::WebSearch { .. }
            | ToolSpec::Freeform(_) => {
                debug!(
                    tool = tool.name(),
                    "dropping non-function tool for chat completions compatibility"
                );
            }
        }
    }

    Ok(tools_json)
}

fn chat_function_tool(name: String, tool: &ResponsesApiTool) -> Result<Value, serde_json::Error> {
    let parameters = serde_json::to_value(&tool.parameters)?;
    Ok(json!({
        "type": "function",
        "function": {
            "name": name,
            "description": tool.description,
            "parameters": parameters,
        },
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use codex_protocol::models::FunctionCallOutputPayload;
    use codex_tools::ResponsesApiTool;
    use pretty_assertions::assert_eq;

    #[test]
    fn maps_messages_tool_calls_and_outputs() {
        let items = vec![
            ResponseItem::Message {
                id: None,
                role: "user".to_string(),
                content: vec![ContentItem::InputText {
                    text: "hello".to_string(),
                }],
                phase: None,
            },
            ResponseItem::FunctionCall {
                id: None,
                name: "shell".to_string(),
                namespace: None,
                arguments: "{\"cmd\":\"ls\"}".to_string(),
                call_id: "call_1".to_string(),
            },
            ResponseItem::FunctionCallOutput {
                call_id: "call_1".to_string(),
                output: FunctionCallOutputPayload::from_text("ok".to_string()),
            },
        ];

        assert_eq!(
            chat_messages_from_response_items(&items),
            vec![
                json!({"role": "user", "content": "hello"}),
                json!({
                    "role": "assistant",
                    "content": "",
                    "tool_calls": [{
                        "id": "call_1",
                        "type": "function",
                        "function": {
                            "name": "shell",
                            "arguments": "{\"cmd\":\"ls\"}",
                        },
                    }],
                }),
                json!({"role": "tool", "tool_call_id": "call_1", "content": "ok"}),
            ]
        );
    }

    #[test]
    fn maps_function_tools_to_chat_shape() {
        let tools =
            create_tools_json_for_chat_completions(&[ToolSpec::Function(ResponsesApiTool {
                name: "shell".to_string(),
                description: "Run a shell command".to_string(),
                strict: false,
                defer_loading: None,
                parameters: serde_json::from_value(json!({
                    "type": "object",
                    "properties": {},
                }))
                .expect("schema should deserialize"),
                output_schema: None,
            })])
            .expect("tools should serialize");

        assert_eq!(
            tools,
            vec![json!({
                "type": "function",
                "function": {
                    "name": "shell",
                    "description": "Run a shell command",
                    "parameters": {
                        "type": "object",
                        "properties": {},
                    },
                },
            })]
        );
    }
}
