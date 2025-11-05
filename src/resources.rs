use rmcp::model::{CallToolResult, Content};
use std::fs;
use std::path::PathBuf;
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

const DOCS_DIR: &str = ".claude/docs";

struct DocFile {
    name: &'static str,
    path: &'static str,
    uri: &'static str,
    scope: &'static str,
}

const DOC_FILES: &[DocFile] = &[
    DocFile {
        name: "ink! Comprehensive Guide",
        path: "ink-llms.txt",
        uri: "ink://docs/llm-guide",
        scope: "ink",
    },
    DocFile {
        name: "ink! Technical Guide",
        path: "ink-technical-guide.txt",
        uri: "ink://docs/technical-guide",
        scope: "ink",
    },
    DocFile {
        name: "Pop CLI Comprehensive Guide",
        path: "pop-cli-comprehensive-guide.txt",
        uri: "pop://docs/cli-guide",
        scope: "pop",
    },
    DocFile {
        name: "XCM Comprehensive Guide",
        path: "xcm-comprehensive-guide.txt",
        uri: "xcm://docs/comprehensive-guide",
        scope: "xcm",
    },
    DocFile {
        name: "XCM ink! Examples Guide",
        path: "xcm-ink-examples-guide.txt",
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

        // Read the document
        let doc_path = PathBuf::from(DOCS_DIR).join(doc.path);
        let content = match fs::read_to_string(&doc_path) {
            Ok(content) => content,
            Err(_) => continue, // Skip files that can't be read
        };

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
