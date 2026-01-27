//! MCP Resources for Pop CLI type documentation

use rmcp::model::{AnnotateAble, RawResource, Resource, ResourceContents};

/// URI for the type hints resource
pub const TYPE_HINTS_URI: &str = "pop://docs/type-hints";

/// Minimal documentation for Substrate/ink! types used in call_chain
const TYPE_HINTS_CONTENT: &str = include_str!("../docs/type-hints.txt");

/// List all available resources
pub fn list_resources() -> Vec<Resource> {
    vec![RawResource {
        uri: TYPE_HINTS_URI.to_owned(),
        name: "type-hints".to_owned(),
        title: Some("Substrate Type Hints".to_owned()),
        description: Some(
            "Type formatting hints for call_chain tool (MultiAddress, Option, Vec, Balance)"
                .to_owned(),
        ),
        mime_type: Some("text/plain".to_owned()),
        size: Some(TYPE_HINTS_CONTENT.len() as u32),
        icons: None,
    }
    .no_annotation()]
}

/// Read a resource by URI
pub fn read_resource(uri: &str) -> Option<ResourceContents> {
    if uri == TYPE_HINTS_URI {
        Some(ResourceContents::text(TYPE_HINTS_CONTENT, TYPE_HINTS_URI))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_resources_returns_type_hints() {
        let resources = list_resources();
        assert_eq!(resources.len(), 1);
        assert_eq!(resources[0].uri, TYPE_HINTS_URI);
    }

    #[test]
    fn read_resource_returns_content_for_valid_uri() {
        let content = read_resource(TYPE_HINTS_URI);
        assert!(content.is_some());
        if let Some(ResourceContents::TextResourceContents { text, .. }) = content {
            assert!(text.contains("MultiAddress"));
            assert!(text.contains("//Alice"));
        }
    }

    #[test]
    fn read_resource_returns_none_for_invalid_uri() {
        let content = read_resource("pop://invalid");
        assert!(content.is_none());
    }
}
