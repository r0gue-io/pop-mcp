//! Contract deployment (`pop up <contract>`)

use rmcp::model::CallToolResult;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::{PopMcpError, PopMcpResult};
use crate::executor::PopExecutor;
use crate::tools::common::{error_result, success_result};

/// Parameters for the deploy_contract tool.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[schemars(extend("properties" = {}))]
pub struct DeployContractParams {
    /// Path to the contract directory.
    #[schemars(
        description = "Path to the contract directory (e.g., './my_contract' or 'my_contract')"
    )]
    pub path: String,
    /// Constructor function to call.
    #[schemars(description = "Constructor function to call")]
    pub constructor: Option<String>,
    /// Constructor arguments as space-separated values.
    #[schemars(description = "Constructor arguments as space-separated values")]
    pub args: Option<String>,
    /// Initial balance to transfer to the contract (in tokens).
    #[schemars(description = "Initial balance to transfer to the contract (in tokens)")]
    pub value: Option<String>,
    /// Whether to submit an extrinsic for on-chain execution.
    #[schemars(description = "Submit an extrinsic for on-chain execution")]
    pub execute: Option<bool>,
    /// WebSocket URL of the node.
    #[schemars(description = "WebSocket URL of the node")]
    pub url: Option<String>,
}

/// Build command arguments for deploy_contract
fn build_deploy_contract_args<'a>(
    params: &'a DeployContractParams,
    stored_url: Option<&'a str>,
    effective_suri: Option<&'a str>,
) -> Vec<&'a str> {
    let mut args = vec!["up", params.path.as_str(), "-y"];

    if let Some(ref constructor) = params.constructor {
        args.push("--constructor");
        args.push(constructor.as_str());
    }

    if let Some(ref contract_args) = params.args {
        args.push("--args");
        for arg in contract_args.split_whitespace() {
            args.push(arg);
        }
    }

    if let Some(ref value) = params.value {
        args.push("--value");
        args.push(value.as_str());
    }

    if params.execute.unwrap_or(false) {
        args.push("--execute");
    }

    if let Some(suri) = effective_suri {
        args.push("--suri");
        args.push(suri);
    }

    // Use provided URL or fall back to stored URL
    if let Some(ref url) = params.url {
        args.push("--url");
        args.push(url.as_str());
    } else if let Some(url) = stored_url {
        args.push("--url");
        args.push(url);
    }

    args
}

/// Execute deploy_contract tool
pub fn deploy_contract(
    executor: &PopExecutor,
    params: DeployContractParams,
    stored_url: Option<&str>,
) -> PopMcpResult<CallToolResult> {
    // Read suri from PRIVATE_KEY environment variable
    let suri = crate::get_default_suri();
    if params.execute.unwrap_or(false) && suri.is_none() {
        return Err(PopMcpError::InvalidInput(
            "PRIVATE_KEY environment variable is required when execute=true".to_owned(),
        ));
    }
    let args = build_deploy_contract_args(&params, stored_url, suri.as_deref());

    match executor.execute(&args) {
        Ok(output) => Ok(success_result(output)),
        Err(e) => Ok(error_result(format!("Deployment failed:\n\n{}", e))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_args_minimal() {
        let params = DeployContractParams {
            path: "./my_contract".to_owned(),
            constructor: None,
            args: None,
            value: None,
            execute: None,
            url: None,
        };
        let args = build_deploy_contract_args(&params, None, None);
        assert_eq!(args, vec!["up", "./my_contract", "-y"]);
    }

    #[test]
    fn build_args_with_env_suri() {
        let params = DeployContractParams {
            path: "./my_contract".to_owned(),
            constructor: Some("new".to_owned()),
            args: Some("100 true".to_owned()),
            value: Some("1000".to_owned()),
            execute: Some(true),
            url: Some("ws://localhost:9944".to_owned()),
        };
        let args = build_deploy_contract_args(&params, None, Some("//Alice"));
        assert_eq!(
            args,
            vec![
                "up",
                "./my_contract",
                "-y",
                "--constructor",
                "new",
                "--args",
                "100",
                "true",
                "--value",
                "1000",
                "--execute",
                "--suri",
                "//Alice",
                "--url",
                "ws://localhost:9944",
            ]
        );
    }

    #[test]
    fn build_args_uses_stored_url_fallback() {
        let params = DeployContractParams {
            path: "./my_contract".to_owned(),
            constructor: None,
            args: None,
            value: None,
            execute: None,
            url: None,
        };
        let args = build_deploy_contract_args(&params, Some("ws://stored:9944"), None);
        assert_eq!(
            args,
            vec!["up", "./my_contract", "-y", "--url", "ws://stored:9944"]
        );
    }
}
