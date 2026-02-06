//! Clean tools (pop clean)

use rmcp::model::CallToolResult;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::PopMcpResult;
use crate::executor::PopExecutor;
use crate::tools::common::{error_result, success_result};

/// Parameters for the clean_nodes tool.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[schemars(extend("properties" = {}))]
pub struct CleanNodesParams {
    /// Process IDs of nodes to stop.
    pub pids: Vec<u32>,
}

/// Parameters for the clean_network tool.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[schemars(extend("properties" = {}))]
pub struct CleanNetworkParams {
    /// Path to the network base directory or zombie.json.
    #[schemars(description = "Path to the network base directory or zombie.json")]
    pub path: Option<String>,
    /// Stop all running networks without prompting (default: false).
    #[schemars(description = "Stop all running networks without prompting (default: false)")]
    pub all: Option<bool>,
    /// Keep the network state on disk after shutdown (default: false).
    #[schemars(description = "Keep the network state on disk after shutdown (default: false)")]
    pub keep_state: Option<bool>,
}

impl CleanNetworkParams {
    fn validate(&self) -> Result<(), String> {
        let all = self.all.unwrap_or(false);
        let has_path = self.path.as_ref().is_some_and(|p| !p.trim().is_empty());
        if all && has_path {
            return Err("Provide either 'all' or 'path', not both".to_owned());
        }
        if !all && !has_path {
            return Err("Provide either 'all' or a non-empty 'path'".to_owned());
        }
        Ok(())
    }
}

fn build_clean_network_args(params: &CleanNetworkParams) -> Vec<String> {
    let mut args = vec!["clean".to_owned(), "network".to_owned()];

    if params.all.unwrap_or(false) {
        args.push("--all".to_owned());
    }

    if params.keep_state.unwrap_or(false) {
        args.push("--keep-state".to_owned());
    }

    if let Some(path) = params.path.as_ref() {
        if !path.trim().is_empty() {
            args.push(path.clone());
        }
    }

    args
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

/// Stop running networks using pop clean network.
pub fn clean_network(
    executor: &PopExecutor,
    params: CleanNetworkParams,
) -> PopMcpResult<CallToolResult> {
    if let Err(message) = params.validate() {
        return Ok(error_result(message));
    }

    let args = build_clean_network_args(&params);
    let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();

    match executor.execute(&arg_refs) {
        Ok(output) => Ok(success_result(output)),
        Err(e) => Ok(error_result(format!("Failed to clean network: {}", e))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_network_rejects_missing_path_and_all() {
        let params = CleanNetworkParams {
            path: None,
            all: None,
            keep_state: None,
        };
        assert!(params.validate().is_err());
    }

    #[test]
    fn clean_network_rejects_all_and_path() {
        let params = CleanNetworkParams {
            path: Some("/tmp/zombie.json".to_owned()),
            all: Some(true),
            keep_state: None,
        };
        assert!(params.validate().is_err());
    }

    #[test]
    fn clean_network_accepts_path() {
        let params = CleanNetworkParams {
            path: Some("/tmp/zombie.json".to_owned()),
            all: None,
            keep_state: None,
        };
        assert!(params.validate().is_ok());
    }

    #[test]
    fn build_clean_network_args_with_path() {
        let params = CleanNetworkParams {
            path: Some("/tmp/zombie.json".to_owned()),
            all: None,
            keep_state: Some(true),
        };
        let args = build_clean_network_args(&params);
        assert_eq!(
            args,
            vec![
                "clean".to_owned(),
                "network".to_owned(),
                "--keep-state".to_owned(),
                "/tmp/zombie.json".to_owned()
            ]
        );
    }

    #[test]
    fn build_clean_network_args_with_all() {
        let params = CleanNetworkParams {
            path: None,
            all: Some(true),
            keep_state: None,
        };
        let args = build_clean_network_args(&params);
        assert_eq!(
            args,
            vec!["clean".to_owned(), "network".to_owned(), "--all".to_owned()]
        );
    }
}
