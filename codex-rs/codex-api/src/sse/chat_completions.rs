use crate::common::ResponseEvent;
use crate::common::ResponseStream;
use crate::error::ApiError;
use crate::telemetry::SseTelemetry;
use codex_client::ByteStream;
use codex_client::StreamResponse;
use codex_protocol::models::ContentItem;
use codex_protocol::models::ResponseItem;
use codex_protocol::protocol::TokenUsage;
use eventsource_stream::Eventsource;
use futures::StreamExt;
use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::Instant;
use tokio::time::timeout;
use tracing::debug;
use tracing::trace;

const REQUEST_ID_HEADER: &str = "x-request-id";

pub fn spawn_chat_completions_stream(
    stream_response: StreamResponse,
    idle_timeout: Duration,
    telemetry: Option<Arc<dyn SseTelemetry>>,
) -> ResponseStream {
    let upstream_request_id = stream_response
        .headers
        .get(REQUEST_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(ToString::to_string);
    let (tx_event, rx_event) = mpsc::channel::<Result<ResponseEvent, ApiError>>(1600);
    tokio::spawn(process_chat_completions_sse(
        stream_response.bytes,
        tx_event,
        idle_timeout,
        telemetry,
    ));
    ResponseStream {
        rx_event,
        upstream_request_id,
    }
}

#[derive(Debug, Default)]
struct ChatCompletionsState {
    response_id: Option<String>,
    text: String,
    tool_calls: BTreeMap<u32, PartialToolCall>,
    usage: Option<TokenUsage>,
    finished: bool,
}

#[derive(Debug, Default)]
struct PartialToolCall {
    id: Option<String>,
    name: Option<String>,
    arguments: String,
}

impl ChatCompletionsState {
    fn apply_chunk(&mut self, chunk: ChatCompletionChunk) -> Vec<ResponseEvent> {
        if let Some(id) = chunk.id {
            self.response_id = Some(id);
        }
        if let Some(usage) = chunk.usage {
            self.usage = Some(usage.into());
        }

        let mut events = Vec::new();
        for choice in chunk.choices {
            if !matches!(choice.index, None | Some(0)) {
                continue;
            }

            if let Some(delta) = choice.delta.content.and_then(delta_content_to_string)
                && !delta.is_empty()
            {
                self.text.push_str(&delta);
                events.push(ResponseEvent::OutputTextDelta(delta));
            }

            for tool_call in choice.delta.tool_calls.unwrap_or_default() {
                let index = tool_call.index.unwrap_or(0);
                let pending = self.tool_calls.entry(index).or_default();
                if let Some(id) = tool_call.id {
                    pending.id = Some(id);
                }
                if let Some(function) = tool_call.function {
                    if let Some(name) = function.name {
                        pending.name = Some(name);
                    }
                    if let Some(arguments) = function.arguments
                        && !arguments.is_empty()
                    {
                        pending.arguments.push_str(&arguments);
                        let item_id = pending
                            .id
                            .clone()
                            .unwrap_or_else(|| format!("tool_call_{index}"));
                        events.push(ResponseEvent::ToolCallInputDelta {
                            item_id,
                            call_id: pending.id.clone(),
                            delta: arguments,
                        });
                    }
                }
            }

            if choice.finish_reason.is_some() {
                self.finished = true;
            }
        }

        events
    }

    fn terminal_events(&mut self) -> Vec<ResponseEvent> {
        let mut events = Vec::new();
        if !self.text.is_empty() {
            events.push(ResponseEvent::OutputItemDone(ResponseItem::Message {
                id: None,
                role: "assistant".to_string(),
                content: vec![ContentItem::OutputText {
                    text: std::mem::take(&mut self.text),
                }],
                phase: None,
            }));
        }

        let has_tool_calls = !self.tool_calls.is_empty();
        let tool_calls = std::mem::take(&mut self.tool_calls);
        for (index, tool_call) in tool_calls {
            let name = tool_call
                .name
                .unwrap_or_else(|| format!("tool_call_{index}"));
            let call_id = tool_call.id.unwrap_or_else(|| format!("tool_call_{index}"));
            events.push(ResponseEvent::OutputItemDone(ResponseItem::FunctionCall {
                id: None,
                name,
                namespace: None,
                arguments: tool_call.arguments,
                call_id,
            }));
        }

        events.push(ResponseEvent::Completed {
            response_id: self.response_id.clone().unwrap_or_default(),
            token_usage: self.usage.clone(),
            end_turn: Some(!has_tool_calls),
        });
        events
    }
}

#[derive(Debug, Deserialize)]
struct ChatCompletionChunk {
    id: Option<String>,
    #[serde(default)]
    choices: Vec<ChatCompletionChoice>,
    usage: Option<ChatCompletionUsage>,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionChoice {
    index: Option<u32>,
    #[serde(default)]
    delta: ChatCompletionDelta,
    finish_reason: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct ChatCompletionDelta {
    content: Option<Value>,
    tool_calls: Option<Vec<ChatCompletionToolCallDelta>>,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionToolCallDelta {
    index: Option<u32>,
    id: Option<String>,
    function: Option<ChatCompletionFunctionDelta>,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionFunctionDelta {
    name: Option<String>,
    arguments: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionUsage {
    prompt_tokens: Option<i64>,
    completion_tokens: Option<i64>,
    total_tokens: Option<i64>,
    prompt_tokens_details: Option<ChatPromptTokensDetails>,
    completion_tokens_details: Option<ChatCompletionTokensDetails>,
}

#[derive(Debug, Deserialize)]
struct ChatPromptTokensDetails {
    cached_tokens: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionTokensDetails {
    reasoning_tokens: Option<i64>,
}

impl From<ChatCompletionUsage> for TokenUsage {
    fn from(value: ChatCompletionUsage) -> Self {
        let input_tokens = value.prompt_tokens.unwrap_or(0);
        let output_tokens = value.completion_tokens.unwrap_or(0);
        TokenUsage {
            input_tokens,
            cached_input_tokens: value
                .prompt_tokens_details
                .and_then(|details| details.cached_tokens)
                .unwrap_or(0),
            output_tokens,
            reasoning_output_tokens: value
                .completion_tokens_details
                .and_then(|details| details.reasoning_tokens)
                .unwrap_or(0),
            total_tokens: value
                .total_tokens
                .unwrap_or(input_tokens.saturating_add(output_tokens)),
        }
    }
}

fn delta_content_to_string(value: Value) -> Option<String> {
    match value {
        Value::String(text) => Some(text),
        Value::Array(parts) => {
            let text = parts
                .into_iter()
                .filter_map(|part| part.get("text").and_then(Value::as_str).map(str::to_string))
                .collect::<Vec<_>>()
                .join("");
            (!text.is_empty()).then_some(text)
        }
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::Object(_) => None,
    }
}

pub async fn process_chat_completions_sse(
    stream: ByteStream,
    tx_event: mpsc::Sender<Result<ResponseEvent, ApiError>>,
    idle_timeout: Duration,
    telemetry: Option<Arc<dyn SseTelemetry>>,
) {
    let mut stream = stream.eventsource();
    let mut state = ChatCompletionsState::default();

    loop {
        let start = Instant::now();
        let response = timeout(idle_timeout, stream.next()).await;
        if let Some(t) = telemetry.as_ref() {
            t.on_sse_poll(&response, start.elapsed());
        }
        let sse = match response {
            Ok(Some(Ok(sse))) => sse,
            Ok(Some(Err(e))) => {
                debug!("chat completions SSE error: {e:#}");
                let _ = tx_event.send(Err(ApiError::Stream(e.to_string()))).await;
                return;
            }
            Ok(None) => {
                if state.finished {
                    send_terminal_events(&tx_event, &mut state).await;
                } else {
                    let _ = tx_event
                        .send(Err(ApiError::Stream(
                            "stream closed before chat completion finished".into(),
                        )))
                        .await;
                }
                return;
            }
            Err(_) => {
                let _ = tx_event
                    .send(Err(ApiError::Stream("idle timeout waiting for SSE".into())))
                    .await;
                return;
            }
        };

        trace!("chat completions SSE event: {}", &sse.data);
        if sse.data.trim() == "[DONE]" {
            state.finished = true;
            send_terminal_events(&tx_event, &mut state).await;
            return;
        }

        let chunk: ChatCompletionChunk = match serde_json::from_str(&sse.data) {
            Ok(chunk) => chunk,
            Err(e) => {
                debug!(
                    "failed to parse chat completions SSE event: {e}, data: {}",
                    &sse.data
                );
                continue;
            }
        };

        for event in state.apply_chunk(chunk) {
            if tx_event.send(Ok(event)).await.is_err() {
                return;
            }
        }

        if state.finished {
            send_terminal_events(&tx_event, &mut state).await;
            return;
        }
    }
}

async fn send_terminal_events(
    tx_event: &mpsc::Sender<Result<ResponseEvent, ApiError>>,
    state: &mut ChatCompletionsState,
) {
    for event in state.terminal_events() {
        if tx_event.send(Ok(event)).await.is_err() {
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accumulates_text_and_tool_calls() {
        let mut state = ChatCompletionsState::default();

        let text_events = state.apply_chunk(ChatCompletionChunk {
            id: Some("chatcmpl-1".to_string()),
            choices: vec![ChatCompletionChoice {
                index: Some(0),
                delta: ChatCompletionDelta {
                    content: Some(Value::String("hello".to_string())),
                    tool_calls: None,
                },
                finish_reason: None,
            }],
            usage: None,
        });
        let tool_events = state.apply_chunk(ChatCompletionChunk {
            id: Some("chatcmpl-1".to_string()),
            choices: vec![ChatCompletionChoice {
                index: Some(0),
                delta: ChatCompletionDelta {
                    content: None,
                    tool_calls: Some(vec![ChatCompletionToolCallDelta {
                        index: Some(0),
                        id: Some("call_1".to_string()),
                        function: Some(ChatCompletionFunctionDelta {
                            name: Some("shell".to_string()),
                            arguments: Some("{\"cmd\":".to_string()),
                        }),
                    }]),
                },
                finish_reason: None,
            }],
            usage: None,
        });
        let _ = state.apply_chunk(ChatCompletionChunk {
            id: Some("chatcmpl-1".to_string()),
            choices: vec![ChatCompletionChoice {
                index: Some(0),
                delta: ChatCompletionDelta {
                    content: None,
                    tool_calls: Some(vec![ChatCompletionToolCallDelta {
                        index: Some(0),
                        id: None,
                        function: Some(ChatCompletionFunctionDelta {
                            name: None,
                            arguments: Some("\"ls\"}".to_string()),
                        }),
                    }]),
                },
                finish_reason: Some("tool_calls".to_string()),
            }],
            usage: Some(ChatCompletionUsage {
                prompt_tokens: Some(7),
                completion_tokens: Some(5),
                total_tokens: Some(12),
                prompt_tokens_details: None,
                completion_tokens_details: None,
            }),
        });

        assert!(matches!(
            text_events.as_slice(),
            [ResponseEvent::OutputTextDelta(delta)] if delta == "hello"
        ));
        assert!(matches!(
            tool_events.as_slice(),
            [ResponseEvent::ToolCallInputDelta { delta, .. }] if delta == "{\"cmd\":"
        ));

        let terminal = state.terminal_events();
        assert!(matches!(
            terminal.as_slice(),
            [
                ResponseEvent::OutputItemDone(ResponseItem::Message { .. }),
                ResponseEvent::OutputItemDone(ResponseItem::FunctionCall {
                    name,
                    arguments,
                    call_id,
                    ..
                }),
                ResponseEvent::Completed {
                    response_id,
                    token_usage: Some(TokenUsage { total_tokens: 12, .. }),
                    end_turn: Some(false),
                }
            ] if name == "shell"
                && arguments == "{\"cmd\":\"ls\"}"
                && call_id == "call_1"
                && response_id == "chatcmpl-1"
        ));
    }
}
