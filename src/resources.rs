use rmcp::model::{CallToolResult, Content, ReadResourceResult, ListResourcesResult, Resource};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum DocScope {
    Ink,
    Pop,
    Xcm,
    Dedot,
    All,
}

// Embed documentation files directly in the binary at compile time
const INK_LLMS_DOC: &str = include_str!("../.claude/docs/ink-llms.txt");
const INK_TECHNICAL_DOC: &str = include_str!("../.claude/docs/ink-technical-guide.txt");
const POP_CLI_DOC: &str = include_str!("../.claude/docs/pop-cli-comprehensive-guide.txt");
const XCM_COMPREHENSIVE_DOC: &str = include_str!("../.claude/docs/xcm-comprehensive-guide.txt");
const XCM_INK_EXAMPLES_DOC: &str = include_str!("../.claude/docs/xcm-ink-examples-guide.txt");

enum DocSource {
    Embedded(&'static str),
    External(&'static str), // URL
}

struct DocFile {
    name: &'static str,
    source: DocSource,
    uri: &'static str,
    scope: &'static str,
}

const DOC_FILES: &[DocFile] = &[
    DocFile {
        name: "ink! Comprehensive Guide",
        source: DocSource::Embedded(INK_LLMS_DOC),
        uri: "ink://docs/llm-guide",
        scope: "ink",
    },
    DocFile {
        name: "ink! Technical Guide",
        source: DocSource::Embedded(INK_TECHNICAL_DOC),
        uri: "ink://docs/technical-guide",
        scope: "ink",
    },
    DocFile {
        name: "Pop CLI Comprehensive Guide",
        source: DocSource::Embedded(POP_CLI_DOC),
        uri: "pop://docs/cli-guide",
        scope: "pop",
    },
    DocFile {
        name: "XCM Comprehensive Guide",
        source: DocSource::Embedded(XCM_COMPREHENSIVE_DOC),
        uri: "xcm://docs/comprehensive-guide",
        scope: "xcm",
    },
    DocFile {
        name: "XCM ink! Examples Guide",
        source: DocSource::Embedded(XCM_INK_EXAMPLES_DOC),
        uri: "xcm://docs/ink-examples",
        scope: "xcm",
    },
    DocFile {
        name: "Dedot & Typink Documentation",
        source: DocSource::External("https://docs.dedot.dev/llms-full.txt"),
        uri: "dedot://docs/full-guide",
        scope: "dedot",
    },
];

async fn fetch_external_doc(url: &str) -> Result<String> {
    let response = reqwest::get(url).await?;
    let text = response.text().await?;
    Ok(text)
}

pub async fn search_docs(query: &str, scope: Option<DocScope>) -> Result<CallToolResult> {
    let search_scope = match scope {
        Some(DocScope::Ink) => "ink",
        Some(DocScope::Pop) => "pop",
        Some(DocScope::Xcm) => "xcm",
        Some(DocScope::Dedot) => "dedot",
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

        // Get documentation content (either embedded or external)
        let content = match &doc.source {
            DocSource::Embedded(text) => text.to_string(),
            DocSource::External(url) => {
                match fetch_external_doc(url).await {
                    Ok(text) => text,
                    Err(e) => {
                        // If fetch fails, add a note and continue
                        results.push_str(&format!("\n## {} ({})\n\nNote: Failed to fetch external documentation: {}\n\n", doc.name, doc.uri, e));
                        continue;
                    }
                }
            }
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

pub async fn list_resources() -> Result<ListResourcesResult> {
    let resources: Vec<Resource> = DOC_FILES
        .iter()
        .map(|doc| {
            Resource::new(
                rmcp::model::RawResource {
                    uri: doc.uri.to_string(),
                    name: doc.name.to_string(),
                    description: Some(format!("{} documentation for Polkadot development", doc.scope)),
                    mime_type: Some("text/plain".to_string()),
                    title: None,
                    size: None,
                    icons: None,
                },
                None,
            )
        })
        .collect();

    Ok(ListResourcesResult {
        resources,
        next_cursor: None,
    })
}

pub async fn read_resource(uri: &str) -> Result<ReadResourceResult> {
    for doc in DOC_FILES {
        if doc.uri == uri {
            let content = match &doc.source {
                DocSource::Embedded(text) => text.to_string(),
                DocSource::External(url) => fetch_external_doc(url).await?,
            };

            return Ok(ReadResourceResult {
                contents: vec![rmcp::model::ResourceContents::text(content, uri.to_string())],
            });
        }
    }

    Err(anyhow::anyhow!("Resource not found: {}", uri))
}
