//! Chain/node management (pop up ink-node)

use rmcp::model::CallToolResult;

use crate::error::PopMcpResult;
use crate::executor::CommandExecutor;
use crate::tools::helpers::{error_result, success_result};

/// Result of launching an ink node
#[derive(Debug, Clone)]
pub struct LaunchNodeResult {
    pub websocket_url: Option<String>,
}

/// Parse the output from launch_ink_node to extract WebSocket URL
pub fn parse_launch_output(output: &str) -> LaunchNodeResult {
    let mut ws_url = String::from("ws://localhost:9944");

    for line in output.lines() {
        if line.contains("rpc=ws://") {
            if let Some(start) = line.find("rpc=ws://") {
                let url_part = &line[start + 4..]; // Skip "rpc="
                if let Some(end) = url_part.find(['#', ' ', '&']) {
                    ws_url = url_part[..end].trim_end_matches('/').to_string();
                }
            }
        }
    }

    LaunchNodeResult {
        websocket_url: Some(ws_url),
    }
}

/// Execute launch_ink_node tool (pop up ink-node)
pub fn launch_ink_node<E: CommandExecutor>(
    executor: &E,
) -> PopMcpResult<(CallToolResult, LaunchNodeResult)> {
    let args = ["up", "ink-node", "-y", "--detach"];

    match executor.execute(&args) {
        Ok(output) => {
            let result = parse_launch_output(&output);
            Ok((success_result(output), result))
        }
        Err(e) => Ok((
            error_result(e.to_string()),
            LaunchNodeResult {
                websocket_url: None,
            },
        )),
    }
}

/// Stop all running local nodes using pop clean
pub fn stop_nodes<E: CommandExecutor>(executor: &E) -> PopMcpResult<CallToolResult> {
    let args = ["clean", "node", "--all"];

    match executor.execute(&args) {
        Ok(output) => Ok(success_result(format!("Nodes stopped!\n\n{}", output))),
        Err(e) => Ok(error_result(format!("Failed to stop nodes: {}", e))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::PopExecutor;
    use rmcp::model::RawContent;

    fn content_text(result: &rmcp::model::CallToolResult) -> String {
        result
            .content
            .last()
            .and_then(|c| match &c.raw {
                RawContent::Text(t) => Some(t.text.clone()),
                _ => None,
            })
            .unwrap_or_default()
    }

    #[test]
    fn test_parse_launch_output_with_url() {
        let output = r#"
Node started!
Portal: https://polkadot.js.org/apps/?rpc=ws://127.0.0.1:9944#/explorer
"#;
        let result = parse_launch_output(output);
        assert_eq!(
            result.websocket_url,
            Some("ws://127.0.0.1:9944".to_string())
        );
    }

    #[test]
    fn test_parse_launch_output_default_url() {
        let output = "Node started!";
        let result = parse_launch_output(output);
        assert_eq!(
            result.websocket_url,
            Some("ws://localhost:9944".to_string())
        );
    }

    #[test]
    fn test_launch_and_stop_ink_node() {
        let executor = PopExecutor::new();

        // Launch ink-node
        let (result, node_result) = launch_ink_node(&executor).unwrap();
        assert!(!result.is_error.unwrap());

        let text = content_text(&result);
        assert!(text.contains("successfully"));

        // Verify we got a websocket URL
        assert!(node_result.websocket_url.is_some());

        // Stop the node
        let stop_result = stop_nodes(&executor).unwrap();
        assert!(!stop_result.is_error.unwrap());
    }
}
