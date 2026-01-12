//! Clean tools (pop clean)

use rmcp::model::CallToolResult;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::PopMcpResult;
use crate::executor::PopExecutor;
use crate::tools::common::{error_result, success_result};

/// Parameters for the clean_nodes tool.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct CleanNodesParams {
    /// Process IDs of nodes to stop.
    pub pids: Vec<u32>,
}

/// Stop running local nodes using pop clean node --pid <pid...>
pub fn clean_nodes(
    executor: &PopExecutor,
    params: CleanNodesParams,
) -> PopMcpResult<CallToolResult> {
    if params.pids.is_empty() {
        return Ok(error_result("At least one pid is required"));
    }

    let mut args = vec!["clean".to_owned(), "node".to_owned(), "--pid".to_owned()];
    args.extend(params.pids.iter().map(ToString::to_string));
    let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();

    match executor.execute(&arg_refs) {
        Ok(output) => Ok(success_result(format!(
            "Nodes cleaned for pids: {}\n\n{}",
            params
                .pids
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(" "),
            output
        ))),
        Err(e) => Ok(error_result(format!("Failed to clean nodes: {}", e))),
    }
}
