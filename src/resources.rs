//! Resource handlers for Pop MCP Server

use anyhow::Result;
use rmcp::model::{CallToolResult, Content, ListResourcesResult, ReadResourceResult, Resource};
use rmcp::ErrorData as McpError;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct SearchDocumentationParams {
    #[schemars(description = "Search query or topic")]
    pub query: String,
    #[schemars(
        description = "Limit search to specific documentation: 'ink', 'pop', 'xcm', 'dedot', or 'all'"
    )]
    pub scope: Option<String>,
}

pub async fn search_documentation(
    params: SearchDocumentationParams,
) -> Result<CallToolResult, McpError> {
    search_docs(&params.query, params.scope)
        .await
        .map_err(|e| McpError::internal_error(e.to_string(), None))
}

const INK_LLMS_DOC: &str = include_str!("../.claude/docs/ink-llms.txt");
const POP_CLI_DOC: &str = include_str!("../.claude/docs/pop-cli-llms.txt");
const XCM_COMPREHENSIVE_DOC: &str = include_str!("../.claude/docs/xcm-comprehensive-guide.txt");
const XCM_INK_EXAMPLES_DOC: &str = include_str!("../.claude/docs/xcm-ink-examples-guide.txt");

const ENDPOINTS_DOC: &str = r#"POLKADOT NETWORK ENDPOINTS

TESTNET (RECOMMENDED FOR DEVELOPMENT)
--------------------------------------
PassetHub Testnet (ParaID 1111) - DEFAULT
- Network: PassetHub Testnet
- RPC: wss://testnet-passet-hub.polkadot.io
- Explorer: https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Ftestnet-passet-hub.polkadot.io
- Native Token: PAS (Paseo native token)
- Use Case: ink! smart contract development, XCM testing, asset swaps
- Status: Active and maintained

IMPORTANT: PassetHub is the ONLY network you should use for contract development and testing.
All deployment, testing, and interaction commands MUST use this endpoint.

Example deployment:
```bash
pop up --url wss://testnet-passet-hub.polkadot.io --use-wallet
```

Example contract call:
```bash
pop call contract --path . --url wss://testnet-passet-hub.polkadot.io \
  --contract 0xYOUR_CONTRACT_ADDRESS \
  --message your_method \
  --use-wallet
```

LOCAL DEVELOPMENT
-----------------
For local testing with zombienet or dev nodes:
- Local Node: ws://127.0.0.1:9944
- Use Case: Testing before deploying to testnet

MAINNET (PRODUCTION ONLY)
--------------------------
DO NOT use mainnet unless explicitly requested by the user.
Mainnet endpoints are not included here to prevent accidental production deployments.
"#;

enum ResourceSource {
    Embedded(&'static str),
    External(&'static str),
}

struct ResourceFile {
    name: &'static str,
    source: ResourceSource,
    uri: &'static str,
    scope: &'static str,
}

const RESOURCES: &[ResourceFile] = &[
    ResourceFile {
        name: "ink! Comprehensive Guide",
        source: ResourceSource::Embedded(INK_LLMS_DOC),
        uri: "ink://docs/llm-guide",
        scope: "ink",
    },
    ResourceFile {
        name: "Pop CLI Comprehensive Guide",
        source: ResourceSource::Embedded(POP_CLI_DOC),
        uri: "pop://docs/cli-guide",
        scope: "pop",
    },
    ResourceFile {
        name: "XCM Comprehensive Guide",
        source: ResourceSource::Embedded(XCM_COMPREHENSIVE_DOC),
        uri: "xcm://docs/comprehensive-guide",
        scope: "xcm",
    },
    ResourceFile {
        name: "XCM ink! Examples Guide",
        source: ResourceSource::Embedded(XCM_INK_EXAMPLES_DOC),
        uri: "xcm://docs/ink-examples",
        scope: "xcm",
    },
    ResourceFile {
        name: "Dedot & Typink Documentation",
        source: ResourceSource::External("https://docs.dedot.dev/llms-full.txt"),
        uri: "dedot://docs/full-guide",
        scope: "dedot",
    },
    ResourceFile {
        name: "Polkadot Network Endpoints",
        source: ResourceSource::Embedded(ENDPOINTS_DOC),
        uri: "polkadot://endpoints",
        scope: "network",
    },
];

async fn fetch_external_doc(url: &str) -> Result<String> {
    let response = reqwest::get(url).await?;
    let text = response.text().await?;
    Ok(text)
}

pub async fn search_docs(query: &str, scope: Option<String>) -> Result<CallToolResult> {
    let search_scope = match scope.as_deref() {
        Some("ink") => "ink",
        Some("pop") => "pop",
        Some("xcm") => "xcm",
        Some("dedot") => "dedot",
        Some("all") | None => "all",
        Some(other) => {
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "Invalid scope '{}'. Valid options: ink, pop, xcm, dedot, all",
                other
            ))]));
        }
    };

    let mut results = String::new();
    let query_lower = query.to_lowercase();
    let mut found_matches = false;

    for res in RESOURCES {
        if search_scope != "all" && res.scope != search_scope {
            continue;
        }

        let content = match &res.source {
            ResourceSource::Embedded(text) => text.to_string(),
            ResourceSource::External(url) => match fetch_external_doc(url).await {
                Ok(text) => text,
                Err(e) => {
                    results.push_str(&format!(
                        "\n## {} ({})\n\nNote: Failed to fetch: {}\n\n",
                        res.name, res.uri, e
                    ));
                    continue;
                }
            },
        };

        let lines: Vec<&str> = content.lines().collect();
        let mut matches_in_doc = Vec::new();

        for (i, line) in lines.iter().enumerate() {
            if line.to_lowercase().contains(&query_lower) {
                found_matches = true;
                let start = i.saturating_sub(2);
                let end = (i + 3).min(lines.len());
                let context = lines[start..end].join("\n");
                matches_in_doc.push(format!("Line {}:\n{}\n---", i + 1, context));
                if matches_in_doc.len() >= 5 {
                    break;
                }
            }
        }

        if !matches_in_doc.is_empty() {
            results.push_str(&format!("\n## {} ({})\n\n", res.name, res.uri));
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
            "No results found for \"{}\" in {} documentation.\n\nAvailable resources:\n{}\n\nTry different keywords or read the full documentation using the resource URIs.",
            query,
            search_scope,
            RESOURCES
                .iter()
                .filter(|r| search_scope == "all" || r.scope == search_scope)
                .map(|r| format!("- {}: {}", r.name, r.uri))
                .collect::<Vec<_>>()
                .join("\n")
        )
    };

    Ok(CallToolResult::success(vec![Content::text(response)]))
}

pub async fn list_resources() -> Result<ListResourcesResult> {
    let resources: Vec<Resource> = RESOURCES
        .iter()
        .map(|res| {
            Resource::new(
                rmcp::model::RawResource {
                    uri: res.uri.to_string(),
                    name: res.name.to_string(),
                    description: Some(format!("{} resource", res.scope)),
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
    for res in RESOURCES {
        if res.uri == uri {
            let content = match &res.source {
                ResourceSource::Embedded(text) => text.to_string(),
                ResourceSource::External(url) => fetch_external_doc(url).await?,
            };

            return Ok(ReadResourceResult {
                contents: vec![rmcp::model::ResourceContents::text(
                    content,
                    uri.to_string(),
                )],
            });
        }
    }

    Err(anyhow::anyhow!("Resource not found: {}", uri))
}
