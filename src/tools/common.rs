//! Common helper functions for tool implementations

use rmcp::model::{CallToolResult, Content, RawContent};

/// Deserialize an `Option<String>` that also accepts JSON booleans.
///
/// MCP clients may send `false`/`true` as JSON booleans when the schema lacks
/// property type info. This converts them to the strings `"false"`/`"true"`.
pub(crate) fn deserialize_stringy_bool<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize as _;
    let value = Option::<serde_json::Value>::deserialize(deserializer)?;
    match value {
        None => Ok(None),
        Some(serde_json::Value::String(s)) => Ok(Some(s)),
        Some(serde_json::Value::Bool(b)) => Ok(Some(b.to_string())),
        Some(serde_json::Value::Number(n)) => Ok(Some(n.to_string())),
        Some(other) => Err(serde::de::Error::custom(format!(
            "expected string or bool, got {other}"
        ))),
    }
}

/// Create a success result with the given text
pub(crate) fn success_result(text: impl Into<String>) -> CallToolResult {
    CallToolResult::success(vec![Content::text(text.into())])
}

/// Create an error result with the given text
pub(crate) fn error_result(text: impl Into<String>) -> CallToolResult {
    CallToolResult::error(vec![Content::text(text.into())])
}

/// Extract text content from a CallToolResult
pub fn extract_text(result: &CallToolResult) -> Option<String> {
    result.content.first().and_then(|c| match &c.raw {
        RawContent::Text(t) => Some(t.text.clone()),
        _ => None,
    })
}

/// Extract text content from a CallToolResult, returning empty string if missing.
#[cfg(test)]
pub(crate) fn content_text(result: &CallToolResult) -> String {
    extract_text(result).unwrap_or_default()
}
