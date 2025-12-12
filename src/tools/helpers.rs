//! Helper functions for tool implementations

use rmcp::model::{CallToolResult, Content};

/// Create a success result with the given text
pub fn success_result(text: impl Into<String>) -> CallToolResult {
    CallToolResult::success(vec![Content::text(text.into())])
}

/// Create an error result with the given text
pub fn error_result(text: impl Into<String>) -> CallToolResult {
    CallToolResult::error(vec![Content::text(text.into())])
}
