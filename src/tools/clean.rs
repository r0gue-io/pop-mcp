//! Clean tools (pop clean)

use rmcp::model::CallToolResult;

use crate::error::PopMcpResult;
use crate::executor::CommandExecutor;
use crate::tools::helpers::{error_result, success_result};

/// Stop all running local nodes using pop clean node --all
pub fn clean_nodes<E: CommandExecutor>(executor: &E) -> PopMcpResult<CallToolResult> {
    let args = ["clean", "node", "--all"];

    match executor.execute(&args) {
        Ok(output) => Ok(success_result(format!("Nodes cleaned!\n\n{}", output))),
        Err(e) => Ok(error_result(format!("Failed to clean nodes: {}", e))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::PopExecutor;
    use crate::tools::helpers::{extract_text, pop_available, test_utils::is_port_in_use};
    use crate::tools::up::chain::up_ink_node;

    #[test]
    fn clean_nodes_stops_running_node() {
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

        // Clean nodes
        let result = clean_nodes(&executor).unwrap();
        assert!(!result.is_error.unwrap());

        // Verify port 9944 is no longer in use
        assert!(
            !is_port_in_use(9944),
            "Port 9944 should be free after clean"
        );
    }
}
