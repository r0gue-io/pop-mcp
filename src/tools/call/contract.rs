//! Contract calls (pop call contract)

use rmcp::model::CallToolResult;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::PopMcpResult;
use crate::executor::PopExecutor;
use crate::tools::common::{error_result, success_result};

// Parameters

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct CallContractParams {
    #[schemars(description = "Path to the contract directory (needed for contract metadata)")]
    pub path: String,
    #[schemars(description = "Contract address")]
    pub contract: String,
    #[schemars(description = "Message/method to call")]
    pub message: String,
    #[schemars(description = "Method arguments as space-separated values")]
    pub args: Option<String>,
    #[schemars(description = "Value to transfer with the call (in tokens)")]
    pub value: Option<String>,
    #[schemars(description = "Submit an extrinsic for on-chain execution")]
    pub execute: Option<bool>,
    #[schemars(description = "Secret key URI for signing")]
    pub suri: Option<String>,
    #[schemars(description = "WebSocket URL of the node")]
    pub url: Option<String>,
}

/// Build command arguments for call_contract
pub fn build_call_contract_args(params: &CallContractParams) -> Vec<&str> {
    let mut args = vec![
        "call",
        "contract",
        "--path",
        params.path.as_str(),
        "--contract",
        params.contract.as_str(),
        "--message",
        params.message.as_str(),
        "-y",
    ];

    // Split space-separated arguments
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

    if let Some(ref suri) = params.suri {
        args.push("--suri");
        args.push(suri.as_str());
    }

    if let Some(ref url) = params.url {
        args.push("--url");
        args.push(url.as_str());
    }

    if params.execute.unwrap_or(false) {
        args.push("--execute");
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
    let args = build_call_contract_args(&params);

    match executor.execute(&args) {
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
    fn build_args_variants() {
        struct Case {
            name: &'static str,
            params: CallContractParams,
            expected: Vec<&'static str>,
        }

        let cases = vec![
            Case {
                name: "minimal",
                params: CallContractParams {
                    path: "./my_contract".to_string(),
                    contract: "0x1234".to_string(),
                    message: "get".to_string(),
                    args: None,
                    value: None,
                    execute: None,
                    suri: None,
                    url: None,
                },
                expected: vec![
                    "call",
                    "contract",
                    "--path",
                    "./my_contract",
                    "--contract",
                    "0x1234",
                    "--message",
                    "get",
                    "-y",
                ],
            },
            Case {
                name: "args_value_suri_execute_url",
                params: CallContractParams {
                    path: "./p".to_string(),
                    contract: "0xabc".to_string(),
                    message: "transfer".to_string(),
                    args: Some("0x5678 100".to_string()),
                    value: Some("10".to_string()),
                    execute: Some(true),
                    suri: Some("//Alice".to_string()),
                    url: Some("ws://explicit:9944".to_string()),
                },
                expected: vec![
                    "call",
                    "contract",
                    "--path",
                    "./p",
                    "--contract",
                    "0xabc",
                    "--message",
                    "transfer",
                    "-y",
                    "--args",
                    "0x5678",
                    "100",
                    "--value",
                    "10",
                    "--suri",
                    "//Alice",
                    "--url",
                    "ws://explicit:9944",
                    "--execute",
                ],
            },
        ];

        for case in cases {
            let args = build_call_contract_args(&case.params);
            assert_eq!(args, case.expected, "case {}", case.name);
        }
    }
}
