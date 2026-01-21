//! Chain/node management (pop up ink-node)

use rmcp::model::{CallToolResult, Content};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::PopMcpResult;
use crate::executor::PopExecutor;
use crate::tools::common::error_result;

/// Parameters for the up_ink_node tool.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[schemars(extend("properties" = {}))]
pub struct UpInkNodeParams {
    /// The port to be used for the ink! node (default: 9944).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ink_node_port: Option<u16>,
    /// The port to be used for the Ethereum RPC node (default: 8545).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eth_rpc_port: Option<u16>,
}

/// Parse the output to extract WebSocket URL.
///
/// Looks for lines like `url: ws://localhost:9944/` in the output.
fn parse_ws_url(output: &str) -> Option<String> {
    for line in output.lines() {
        // Strip common prefixes (pipe chars from formatted output)
        let trimmed = line.trim().trim_start_matches('│').trim();
        // Look for "url: ws://..." pattern
        if trimmed.starts_with("url:") && trimmed.contains("ws://") {
            if let Some(start) = trimmed.find("ws://") {
                return Some(trimmed[start..].trim_end_matches('/').to_owned());
            }
        }
    }
    None
}

/// Parse the output to extract PIDs from the `kill -9` hint.
fn parse_pids(output: &str) -> Option<Vec<u32>> {
    for line in output.lines() {
        let trimmed = line.trim().trim_start_matches('│').trim();
        if let Some(start) = trimmed.find("kill -9") {
            let after = &trimmed[start + "kill -9".len()..];
            let mut pids = Vec::new();
            for token in after.split_whitespace() {
                let token = token.trim_matches(|c: char| !c.is_ascii_digit());
                if token.is_empty() {
                    continue;
                }
                if let Ok(pid) = token.parse::<u32>() {
                    pids.push(pid);
                }
            }
            if !pids.is_empty() {
                return Some(pids);
            }
        }
    }
    None
}

/// Execute up_ink_node tool (pop up ink-node)
///
/// Returns the websocket URL on success (e.g., "ws://localhost:9944")
pub fn up_ink_node(
    executor: &PopExecutor,
    params: UpInkNodeParams,
) -> PopMcpResult<CallToolResult> {
    let mut args = vec!["up", "ink-node", "-y", "--detach"];

    let ink_port_str;
    if let Some(port) = params.ink_node_port {
        ink_port_str = port.to_string();
        args.push("-i");
        args.push(&ink_port_str);
    }

    let eth_port_str;
    if let Some(port) = params.eth_rpc_port {
        eth_port_str = port.to_string();
        args.push("-e");
        args.push(&eth_port_str);
    }

    match executor.execute(&args) {
        Ok(output) => match parse_ws_url(&output) {
            Some(url) => {
                let mut content = vec![Content::text(url)];
                if let Some(pids) = parse_pids(&output) {
                    let pid_text = pids
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(" ");
                    content.push(Content::text(format!("pids: {}", pid_text)));
                }
                Ok(CallToolResult::success(content))
            }
            None => Ok(error_result("Failed to parse websocket URL from output")),
        },
        Err(e) => Ok(error_result(e.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ws_url_extracts_localhost_url() {
        let output = r#"
┌   Pop CLI : Launch a local Ink! node
│
⚙  Local node started successfully:
│  portal: https://polkadot.js.org/apps/?rpc=ws://localhost:9944/#/explorer
│  url: ws://localhost:9944/
│  logs: tail -f /var/folders/32/t119h4g16mq5jrlm7f4_shhm0000gp/T/.tmpDGAoYa
│
⚙  Ethereum RPC node started successfully:
│  url: ws://localhost:8545
│  logs: tail -f /var/folders/32/t119h4g16mq5jrlm7f4_shhm0000gp/T/.tmptLAPcC
│
└  ✅ Ink! node bootstrapped successfully. Run `kill -9 11040 11253` to terminate it.
"#;
        let url = parse_ws_url(output);
        assert_eq!(url, Some("ws://localhost:9944".to_owned()));
    }

    #[test]
    fn parse_ws_url_returns_none_when_missing() {
        let output = "Some error occurred";
        let url = parse_ws_url(output);
        assert_eq!(url, None);
    }
}
