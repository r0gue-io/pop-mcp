//! Contract calls (pop call contract)

use rmcp::model::CallToolResult;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::{PopMcpError, PopMcpResult};
use crate::executor::PopExecutor;
use crate::tools::common::{error_result, success_result};

/// Parameters for the call_contract tool.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[schemars(extend("properties" = {}))]
pub struct CallContractParams {
    /// Path to the contract directory (needed for metadata).
    #[schemars(description = "Path to the contract directory (needed for contract metadata)")]
    pub path: String,
    /// Contract address to call.
    #[schemars(description = "Contract address")]
    pub contract: String,
    /// Message/method to call on the contract.
    #[schemars(description = "Message/method to call")]
    pub message: String,
    /// Method arguments as space-separated values.
    #[schemars(description = "Method arguments as space-separated values")]
    #[serde(
        default,
        deserialize_with = "crate::tools::common::deserialize_stringy_bool"
    )]
    pub args: Option<String>,
    /// Value to transfer with the call (in tokens).
    #[schemars(description = "Value to transfer with the call (in tokens)")]
    pub value: Option<String>,
    /// Whether to submit an extrinsic for on-chain execution.
    #[schemars(description = "Submit an extrinsic for on-chain execution")]
    pub execute: Option<bool>,
    /// WebSocket URL of the node.
    #[schemars(description = "WebSocket URL of the node")]
    pub url: Option<String>,
}

/// Build command arguments for call_contract
fn build_call_contract_args(params: &CallContractParams) -> Vec<String> {
    let mut args = vec![
        "call".to_owned(),
        "contract".to_owned(),
        "--path".to_owned(),
        params.path.clone(),
        "--contract".to_owned(),
        params.contract.clone(),
        "--message".to_owned(),
        params.message.clone(),
        "-y".to_owned(),
    ];

    // Split space-separated arguments
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

    if let Some(ref url) = params.url {
        args.push("--url".to_owned());
        args.push(url.clone());
    }

    if params.execute.unwrap_or(false) {
        args.push("--execute".to_owned());
    }

    args
}

/// Check if output contains error indicators from pop CLI
fn is_error_output(output: &str) -> bool {
    let error_indicators = [
        "Unable to",
        "Error:",
        "error:",
        "Failed to",
        "failed to",
        "not connected",
        "Contract not found",
    ];
    error_indicators
        .iter()
        .any(|indicator| output.contains(indicator))
}

/// Execute call_contract tool
pub fn call_contract(
    executor: &PopExecutor,
    params: CallContractParams,
) -> PopMcpResult<CallToolResult> {
    // Read suri from PRIVATE_KEY environment variable
    let suri = crate::read_private_key_suri();
    if params.execute.unwrap_or(false) && suri.is_none() {
        return Err(PopMcpError::InvalidInput(
            "PRIVATE_KEY environment variable is required when execute=true".to_owned(),
        ));
    }
    let mut args = build_call_contract_args(&params);
    if params.execute.unwrap_or(false) {
        if let Some(suri) = suri {
            args.push("--suri".to_owned());
            args.push(suri);
        }
    }
    let args_refs: Vec<&str> = args.iter().map(String::as_str).collect();

    match executor.execute(&args_refs) {
        Ok(output) => {
            // Check if the output contains error indicators even if exit code was 0
            if is_error_output(&output) {
                Ok(error_result(format!("Contract call failed:\n\n{}", output)))
            } else {
                Ok(success_result(format!(
                    "Contract call successful!\n\n{}",
                    output
                )))
            }
        }
        Err(e) => Ok(error_result(format!("Contract call failed: {}", e))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_args_minimal() {
        let params = CallContractParams {
            path: "./my_contract".to_owned(),
            contract: "0x1234".to_owned(),
            message: "get".to_owned(),
            args: None,
            value: None,
            execute: None,
            url: None,
        };
        let args = build_call_contract_args(&params);
        assert_eq!(
            args,
            vec![
                "call".to_owned(),
                "contract".to_owned(),
                "--path".to_owned(),
                "./my_contract".to_owned(),
                "--contract".to_owned(),
                "0x1234".to_owned(),
                "--message".to_owned(),
                "get".to_owned(),
                "-y".to_owned(),
            ]
        );
    }

    #[test]
    fn build_args_with_execute() {
        let params = CallContractParams {
            path: "./p".to_owned(),
            contract: "0xabc".to_owned(),
            message: "transfer".to_owned(),
            args: Some("0x5678 100".to_owned()),
            value: Some("10".to_owned()),
            execute: Some(true),
            url: Some("ws://localhost:9944".to_owned()),
        };
        let args = build_call_contract_args(&params);
        assert_eq!(
            args,
            vec![
                "call".to_owned(),
                "contract".to_owned(),
                "--path".to_owned(),
                "./p".to_owned(),
                "--contract".to_owned(),
                "0xabc".to_owned(),
                "--message".to_owned(),
                "transfer".to_owned(),
                "-y".to_owned(),
                "--args".to_owned(),
                "0x5678".to_owned(),
                "100".to_owned(),
                "--value".to_owned(),
                "10".to_owned(),
                "--url".to_owned(),
                "ws://localhost:9944".to_owned(),
                "--execute".to_owned(),
            ]
        );
    }
}
