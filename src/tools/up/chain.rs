//! Chain/node management (pop up ink-node)

use rmcp::model::CallToolResult;

use crate::error::PopMcpResult;
use crate::executor::CommandExecutor;
use crate::tools::helpers::{error_result, success_result};

/// Parse the output to extract WebSocket URL
///
/// Looks for lines like `url: ws://localhost:9944/` in the output
fn parse_ws_url(output: &str) -> Option<String> {
    for line in output.lines() {
        // Strip common prefixes (pipe chars from formatted output)
        let trimmed = line.trim().trim_start_matches('│').trim();
        // Look for "url: ws://..." pattern (the local node URL on port 9944)
        if trimmed.starts_with("url:") && trimmed.contains("ws://") && trimmed.contains(":9944") {
            if let Some(start) = trimmed.find("ws://") {
                return Some(trimmed[start..].trim_end_matches('/').to_string());
            }
        }
    }
    None
}

/// Execute up_ink_node tool (pop up ink-node)
///
/// Returns the websocket URL on success (e.g., "ws://localhost:9944")
pub fn up_ink_node<E: CommandExecutor>(executor: &E) -> PopMcpResult<CallToolResult> {
    let args = ["up", "ink-node", "-y", "--detach"];

    match executor.execute(&args) {
        Ok(output) => match parse_ws_url(&output) {
            Some(url) => Ok(success_result(url)),
            None => Ok(error_result("Failed to parse websocket URL from output")),
        },
        Err(e) => Ok(error_result(e.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::PopExecutor;
    use crate::tools::clean::clean_nodes;
    use crate::tools::helpers::{extract_text, pop_available, test_utils::is_port_in_use};

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
        assert_eq!(url, Some("ws://localhost:9944".to_string()));
    }

    #[test]
    fn parse_ws_url_returns_none_when_missing() {
        let output = "Some error occurred";
        let url = parse_ws_url(output);
        assert_eq!(url, None);
    }

    #[test]
    fn up_ink_node_launches_node() {
        let executor = PopExecutor::new();
        if !pop_available(&executor) {
            return;
        }

        // Launch ink-node
        let result = up_ink_node(&executor).unwrap();
        assert!(!result.is_error.unwrap());

        // Verify result contains the websocket URL
        let url = extract_text(&result).unwrap();
        assert_eq!(url, "ws://localhost:9944");

        // Verify port 9944 is in use
        assert!(
            is_port_in_use(9944),
            "Port 9944 should be in use after launch"
        );

        // Clean up
        let _ = clean_nodes(&executor);
    }
}
