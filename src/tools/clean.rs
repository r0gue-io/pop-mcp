//! Clean tools (pop clean)

use rmcp::model::CallToolResult;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::PopMcpResult;
use crate::executor::CommandExecutor;
use crate::tools::common::{error_result, success_result};

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct CleanNodesParams {}

/// Stop all running local nodes using pop clean node --all
pub fn clean_nodes<E: CommandExecutor>(
    executor: &E,
    _params: CleanNodesParams,
) -> PopMcpResult<CallToolResult> {
    let args = ["clean", "node", "--all"];

    match executor.execute(&args) {
        Ok(output) => Ok(success_result(format!("Nodes cleaned!\n\n{}", output))),
        Err(e) => Ok(error_result(format!("Failed to clean nodes: {}", e))),
    }
}
