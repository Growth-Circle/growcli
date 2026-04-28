//! GrowthCircle Responses API compatibility helpers.
//!
//! The GrowthCircle endpoint does not accept unnamed tool types (e.g.
//! `local_shell`) that the upstream OpenAI Responses API supports.  This
//! module filters those out before sending the request.

use codex_api::Provider as ApiProvider;
use codex_tools::create_tools_json_for_responses_api;
use serde_json::Value;
use tracing::warn;

fn provider_uses_growthcircle_compat(provider: &ApiProvider) -> bool {
    provider.name.eq_ignore_ascii_case("growthcircle")
        || provider.base_url.contains("ai.growthcircle.id")
}

pub(crate) fn create_tools_json_for_provider(
    provider: &ApiProvider,
    tools: &[codex_tools::ToolSpec],
) -> std::result::Result<Vec<Value>, serde_json::Error> {
    let tools = create_tools_json_for_responses_api(tools)?;

    if !provider_uses_growthcircle_compat(provider) {
        return Ok(tools);
    }

    Ok(tools
        .into_iter()
        .filter(|tool| {
            let has_name = tool
                .get("name")
                .and_then(Value::as_str)
                .is_some_and(|name: &str| !name.trim().is_empty());
            if has_name {
                return true;
            }

            let tool_type = tool
                .get("type")
                .and_then(Value::as_str)
                .unwrap_or("<unknown>");
            warn!(
                tool_type,
                "dropping unnamed tool for GrowthCircle Responses compatibility"
            );
            false
        })
        .collect())
}
