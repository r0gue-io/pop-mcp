//! Contract calls (pop call contract)

use rmcp::model::CallToolResult;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::PopMcpResult;
use crate::executor::CommandExecutor;
use crate::tools::helpers::{error_result, success_result};

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
pub fn build_call_contract_args<'a>(
    params: &'a CallContractParams,
    stored_url: Option<&'a str>,
) -> Vec<&'a str> {
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

    // Use provided URL or fall back to stored URL
    if let Some(ref url) = params.url {
        args.push("--url");
        args.push(url.as_str());
    } else if let Some(url) = stored_url {
        args.push("--url");
        args.push(url);
    }

    if params.execute.unwrap_or(false) {
        args.push("--execute");
    }

    args
}

/// Execute call_contract tool
pub fn call_contract<E: CommandExecutor>(
    executor: &E,
    params: CallContractParams,
    stored_url: Option<&str>,
) -> PopMcpResult<CallToolResult> {
    let args = build_call_contract_args(&params, stored_url);

    match executor.execute(&args) {
        Ok(output) => Ok(success_result(format!(
            "Contract call successful!\n\n{}",
            output
        ))),
        Err(e) => Ok(error_result(format!("Contract call failed: {}", e))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::test_utils::MockExecutor;

    #[test]
    fn test_build_args_basic() {
        let params = CallContractParams {
            path: "./my_contract".to_string(),
            contract: "0x1234".to_string(),
            message: "get".to_string(),
            args: None,
            value: None,
            execute: None,
            suri: None,
            url: None,
        };
        let args = build_call_contract_args(&params, None);

        assert!(args.contains(&"call"));
        assert!(args.contains(&"contract"));
        assert!(args.contains(&"--contract"));
        assert!(args.contains(&"0x1234"));
        assert!(args.contains(&"--message"));
        assert!(args.contains(&"get"));
    }

    #[test]
    fn test_build_args_with_split_args() {
        let params = CallContractParams {
            path: "./my_contract".to_string(),
            contract: "0x1234".to_string(),
            message: "transfer".to_string(),
            args: Some("0x5678 100".to_string()),
            value: None,
            execute: Some(true),
            suri: None,
            url: None,
        };
        let args = build_call_contract_args(&params, Some("ws://localhost:9944"));

        // Args should be split
        assert!(args.contains(&"0x5678"));
        assert!(args.contains(&"100"));
        assert!(args.contains(&"--execute"));
    }

    #[test]
    fn test_build_args_stored_url() {
        let params = CallContractParams {
            path: "./my_contract".to_string(),
            contract: "0x1234".to_string(),
            message: "get".to_string(),
            args: None,
            value: None,
            execute: None,
            suri: None,
            url: None,
        };
        let args = build_call_contract_args(&params, Some("ws://stored:9944"));

        assert!(args.contains(&"ws://stored:9944"));
    }

    #[test]
    fn test_call_contract_success() {
        let executor = MockExecutor::success("Result: 42");
        let params = CallContractParams {
            path: "./my_contract".to_string(),
            contract: "0x1234".to_string(),
            message: "get".to_string(),
            args: None,
            value: None,
            execute: None,
            suri: None,
            url: None,
        };

        let result = call_contract(&executor, params, None).unwrap();
        assert!(!result.is_error.unwrap_or(true));
    }

    #[test]
    fn test_call_contract_failure() {
        let executor = MockExecutor::failure("Contract not found");
        let params = CallContractParams {
            path: "./my_contract".to_string(),
            contract: "0x1234".to_string(),
            message: "get".to_string(),
            args: None,
            value: None,
            execute: None,
            suri: None,
            url: None,
        };

        let result = call_contract(&executor, params, None).unwrap();
        assert!(result.is_error.unwrap_or(false));
    }
}
