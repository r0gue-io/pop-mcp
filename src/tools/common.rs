//! Common helper functions for tool implementations

use rmcp::model::{CallToolResult, Content, RawContent};

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
