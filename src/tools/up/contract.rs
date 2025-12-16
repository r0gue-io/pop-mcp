//! Contract deployment (pop up <contract>)

use rmcp::model::CallToolResult;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::PopMcpResult;
use crate::executor::CommandExecutor;
use crate::tools::common::{error_result, success_result};

// Parameters

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct DeployContractParams {
    #[schemars(
        description = "Path to the contract directory (e.g., './my_contract' or 'my_contract')"
    )]
    pub path: String,
    #[schemars(description = "Constructor function to call")]
    pub constructor: Option<String>,
    #[schemars(description = "Constructor arguments as space-separated values")]
    pub args: Option<String>,
    #[schemars(description = "Initial balance to transfer to the contract (in tokens)")]
    pub value: Option<String>,
    #[schemars(description = "Submit an extrinsic for on-chain execution")]
    pub execute: Option<bool>,
    #[schemars(description = "Secret key URI for signing")]
    pub suri: Option<String>,
    #[schemars(description = "WebSocket URL of the node")]
    pub url: Option<String>,
}

/// Build command arguments for deploy_contract
pub fn build_deploy_contract_args<'a>(
    params: &'a DeployContractParams,
    stored_url: Option<&'a str>,
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

    if let Some(ref suri) = params.suri {
        args.push("--suri");
        args.push(suri.as_str());
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
pub fn deploy_contract<E: CommandExecutor>(
    executor: &E,
    params: DeployContractParams,
    stored_url: Option<&str>,
) -> PopMcpResult<CallToolResult> {
    let args = build_deploy_contract_args(&params, stored_url);

    match executor.execute(&args) {
        Ok(output) => Ok(success_result(output)),
        Err(e) => Ok(error_result(format!("Deployment failed:\n\n{}", e))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_args_variants() {
        struct Case {
            name: &'static str,
            params: DeployContractParams,
            stored_url: Option<&'static str>,
            expected: Vec<&'static str>,
        }

        let cases = vec![
            Case {
                name: "minimal",
                params: DeployContractParams {
                    path: "./my_contract".to_string(),
                    constructor: None,
                    args: None,
                    value: None,
                    execute: None,
                    suri: None,
                    url: None,
                },
                stored_url: None,
                expected: vec!["up", "./my_contract", "-y"],
            },
            Case {
                name: "full_with_explicit_url",
                params: DeployContractParams {
                    path: "./my_contract".to_string(),
                    constructor: Some("new".to_string()),
                    args: Some("100 true".to_string()),
                    value: Some("1000".to_string()),
                    execute: Some(true),
                    suri: Some("//Alice".to_string()),
                    url: Some("ws://explicit:9944".to_string()),
                },
                stored_url: Some("ws://stored:9944"),
                expected: vec![
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
                    "ws://explicit:9944",
                ],
            },
            Case {
                name: "stored_url_fallback",
                params: DeployContractParams {
                    path: "./my_contract".to_string(),
                    constructor: None,
                    args: None,
                    value: None,
                    execute: None,
                    suri: None,
                    url: None,
                },
                stored_url: Some("ws://stored:9944"),
                expected: vec!["up", "./my_contract", "-y", "--url", "ws://stored:9944"],
            },
        ];

        for case in cases {
            let args = build_deploy_contract_args(&case.params, case.stored_url);
            assert_eq!(args, case.expected, "case {}", case.name);
        }
    }
}
