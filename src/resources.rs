use rmcp::model::{CallToolResult, Content};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum DocScope {
    Ink,
    Pop,
    Xcm,
    All,
}

// Embed documentation files directly in the binary at compile time
const INK_LLMS_DOC: &str = include_str!("../.claude/docs/ink-llms.txt");
const INK_TECHNICAL_DOC: &str = include_str!("../.claude/docs/ink-technical-guide.txt");
const POP_CLI_DOC: &str = include_str!("../.claude/docs/pop-cli-comprehensive-guide.txt");
const XCM_COMPREHENSIVE_DOC: &str = include_str!("../.claude/docs/xcm-comprehensive-guide.txt");
const XCM_INK_EXAMPLES_DOC: &str = include_str!("../.claude/docs/xcm-ink-examples-guide.txt");

struct DocFile {
    name: &'static str,
    content: &'static str,
    uri: &'static str,
    scope: &'static str,
}

const DOC_FILES: &[DocFile] = &[
    DocFile {
        name: "ink! Comprehensive Guide",
        content: INK_LLMS_DOC,
        uri: "ink://docs/llm-guide",
        scope: "ink",
    },
    DocFile {
        name: "ink! Technical Guide",
        content: INK_TECHNICAL_DOC,
        uri: "ink://docs/technical-guide",
        scope: "ink",
    },
    DocFile {
        name: "Pop CLI Comprehensive Guide",
        content: POP_CLI_DOC,
        uri: "pop://docs/cli-guide",
        scope: "pop",
    },
    DocFile {
        name: "XCM Comprehensive Guide",
        content: XCM_COMPREHENSIVE_DOC,
        uri: "xcm://docs/comprehensive-guide",
        scope: "xcm",
    },
    DocFile {
        name: "XCM ink! Examples Guide",
        content: XCM_INK_EXAMPLES_DOC,
        uri: "xcm://docs/ink-examples",
        scope: "xcm",
    },
];

pub async fn search_docs(query: &str, scope: Option<DocScope>) -> Result<CallToolResult> {
    let search_scope = match scope {
        Some(DocScope::Ink) => "ink",
        Some(DocScope::Pop) => "pop",
        Some(DocScope::Xcm) => "xcm",
        Some(DocScope::All) | None => "all",
    };

    let mut results = String::new();
    let query_lower = query.to_lowercase();
    let mut found_matches = false;

    for doc in DOC_FILES {
        // Filter by scope
        if search_scope != "all" && doc.scope != search_scope {
            continue;
        }

        // Documentation is embedded in the binary at compile time
        let content = doc.content;

        // Search for matches
        let lines: Vec<&str> = content.lines().collect();
        let mut matches_in_doc = Vec::new();

        for (i, line) in lines.iter().enumerate() {
            if line.to_lowercase().contains(&query_lower) {
                found_matches = true;

                // Get context (2 lines before and after)
                let start = i.saturating_sub(2);
                let end = (i + 3).min(lines.len());

                let context = lines[start..end].join("\n");
                matches_in_doc.push(format!("Line {}:\n{}\n---", i + 1, context));

                // Limit to 5 matches per document
                if matches_in_doc.len() >= 5 {
                    break;
                }
            }
        }

        if !matches_in_doc.is_empty() {
            results.push_str(&format!("\n## {} ({})\n\n", doc.name, doc.uri));
            results.push_str(&matches_in_doc.join("\n\n"));
            results.push_str("\n\n");
        }
    }

    let response = if found_matches {
        format!(
            "Search results for \"{}\" in {} documentation:\n{}\n\nTo read full documentation, use the resource URIs shown above.",
            query, search_scope, results
        )
    } else {
        format!(
            "No results found for \"{}\" in {} documentation.\n\nAvailable documentation:\n{}\n\nTry different keywords or read the full documentation using the resource URIs.",
            query,
            search_scope,
            DOC_FILES.iter()
                .filter(|d| search_scope == "all" || d.scope == search_scope)
                .map(|d| format!("- {}: {}", d.name, d.uri))
                .collect::<Vec<_>>()
                .join("\n")
        )
    };

    Ok(CallToolResult::success(vec![Content::text(response)]))
}
