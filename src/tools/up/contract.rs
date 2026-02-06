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
    #[serde(
        default,
        deserialize_with = "crate::tools::common::deserialize_stringy_bool"
    )]
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
fn build_deploy_contract_args(
    params: &DeployContractParams,
    stored_url: Option<&str>,
) -> Vec<String> {
    let mut args = vec!["up".to_owned(), params.path.clone(), "-y".to_owned()];

    if let Some(ref constructor) = params.constructor {
        args.push("--constructor".to_owned());
        args.push(constructor.clone());
    }

    if let Some(ref contract_args) = params.args {
        args.push("--args".to_owned());
        for arg in contract_args.split_whitespace() {
            args.push(arg.to_owned());
        }
    }

    if let Some(ref value) = params.value {
        args.push("--value".to_owned());
        args.push(value.clone());
    }

    if params.execute.unwrap_or(false) {
        args.push("--execute".to_owned());
    }

    // Use provided URL or fall back to stored URL
    if let Some(ref url) = params.url {
        args.push("--url".to_owned());
        args.push(url.clone());
    } else if let Some(url) = stored_url {
        args.push("--url".to_owned());
        args.push(url.to_owned());
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
    let suri = crate::read_private_key_suri();
    if params.execute.unwrap_or(false) && suri.is_none() {
        return Err(PopMcpError::InvalidInput(
            "PRIVATE_KEY environment variable is required when execute=true".to_owned(),
        ));
    }
    let mut args = build_deploy_contract_args(&params, stored_url);
    if params.execute.unwrap_or(false) {
        if let Some(suri) = suri {
            args.push("--suri".to_owned());
            args.push(suri);
        }
    }
    let args_refs: Vec<&str> = args.iter().map(String::as_str).collect();

    match executor.execute(&args_refs) {
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
        let args = build_deploy_contract_args(&params, None);
        assert_eq!(
            args,
            vec!["up".to_owned(), "./my_contract".to_owned(), "-y".to_owned()]
        );
    }

    #[test]
    fn build_args_with_execute() {
        let params = DeployContractParams {
            path: "./my_contract".to_owned(),
            constructor: Some("new".to_owned()),
            args: Some("100 true".to_owned()),
            value: Some("1000".to_owned()),
            execute: Some(true),
            url: Some("ws://localhost:9944".to_owned()),
        };
        let args = build_deploy_contract_args(&params, None);
        assert_eq!(
            args,
            vec![
                "up".to_owned(),
                "./my_contract".to_owned(),
                "-y".to_owned(),
                "--constructor".to_owned(),
                "new".to_owned(),
                "--args".to_owned(),
                "100".to_owned(),
                "true".to_owned(),
                "--value".to_owned(),
                "1000".to_owned(),
                "--execute".to_owned(),
                "--url".to_owned(),
                "ws://localhost:9944".to_owned(),
            ]
        );
    }

    #[test]
    #[allow(clippy::panic)]
    fn deserialize_args_from_json_bool() {
        let json = r#"{"path": "./flipper", "args": false}"#;
        let params: DeployContractParams = match serde_json::from_str(json) {
            Ok(params) => params,
            Err(err) => panic!("failed to deserialize args bool: {err}"),
        };
        assert_eq!(params.args, Some("false".to_owned()));
    }

    #[test]
    #[allow(clippy::panic)]
    fn deserialize_args_from_json_string() {
        let json = r#"{"path": "./flipper", "args": "true"}"#;
        let params: DeployContractParams = match serde_json::from_str(json) {
            Ok(params) => params,
            Err(err) => panic!("failed to deserialize args string: {err}"),
        };
        assert_eq!(params.args, Some("true".to_owned()));
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
        let args = build_deploy_contract_args(&params, Some("ws://stored:9944"));
        assert_eq!(
            args,
            vec![
                "up".to_owned(),
                "./my_contract".to_owned(),
                "-y".to_owned(),
                "--url".to_owned(),
                "ws://stored:9944".to_owned()
            ]
        );
    }
}
