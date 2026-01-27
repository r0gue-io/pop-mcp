//! Chain calls (pop call chain)

use rmcp::model::CallToolResult;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::{PopMcpError, PopMcpResult};
use crate::executor::PopExecutor;
use crate::tools::common::{error_result, success_result};

/// Type hints for formatting arguments in chain calls.
const TYPE_HINTS: &str = r#"

━━━ Type Hints ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

MultiAddress: Wrap SS58 address in Id(), e.g., Id(5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY)
Option<T>: Pass value directly for Some, omit argument for None
Vec<u8>/[u8]: Pass as string (auto-converted to hex), e.g., "hello" becomes 0x68656c6c6f
Vec<AccountId32>: KNOWN BUG - currently broken in pop-cli. Avoid if possible.
Balance: Use string with full precision (e.g., "1000000000000" for 1 unit with 12 decimals)
Dev accounts: //Alice, //Bob, //Charlie, //Dave, //Eve (use with suri)
Dev accounts (SS58): //Alice=5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY, //Bob=5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty
"#;

/// Parameters for the call_chain tool.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[schemars(extend("properties" = {}))]
pub struct CallChainParams {
    /// WebSocket URL of the chain node.
    #[schemars(description = "WebSocket URL of the chain node (e.g., ws://localhost:9944)")]
    pub url: String,

    /// Pallet name containing the extrinsic, storage item, or constant.
    #[schemars(
        description = "Pallet name (e.g., 'system', 'balances'). Use with metadata=true to list all pallets."
    )]
    pub pallet: Option<String>,

    /// Function name: extrinsic, storage key, or constant name.
    #[schemars(
        description = "Extrinsic, storage key, or constant name to call. Not allowed with metadata=true."
    )]
    pub function: Option<String>,

    /// Arguments for the call.
    #[schemars(description = "Arguments for the call as space-separated values")]
    pub args: Option<Vec<String>>,

    /// Secret key URI for signing transactions.
    #[schemars(
        description = "Secret key URI for signing (e.g., '//Alice'). Required for transactions, not for queries/constants."
    )]
    pub suri: Option<String>,

    /// Execute with root origin via sudo pallet.
    #[schemars(
        description = "Execute with root origin via sudo pallet. Not allowed with metadata=true."
    )]
    pub sudo: Option<bool>,

    /// Display chain metadata instead of executing a call.
    #[schemars(
        description = "Display chain metadata. Use alone to list all pallets, or with pallet to show pallet details (extrinsics, storage, constants). Cannot be used with function, args, suri, or sudo."
    )]
    pub metadata: Option<bool>,
}

impl CallChainParams {
    /// Validate parameters
    pub fn validate(&self) -> Result<(), String> {
        let metadata_mode = self.metadata.unwrap_or(false);

        if metadata_mode {
            // In metadata mode, these are not allowed
            if self.function.is_some() {
                return Err("Cannot use 'function' with metadata=true".to_owned());
            }
            if self.args.is_some() {
                return Err("Cannot use 'args' with metadata=true".to_owned());
            }
            if self.suri.is_some() {
                return Err("Cannot use 'suri' with metadata=true".to_owned());
            }
            if self.sudo.unwrap_or(false) {
                return Err("Cannot use 'sudo' with metadata=true".to_owned());
            }
        } else {
            // In call mode, pallet and function are required
            if self.pallet.is_none() {
                return Err("'pallet' is required when metadata is not set".to_owned());
            }
            if self.function.is_none() {
                return Err("'function' is required when metadata is not set".to_owned());
            }
        }

        Ok(())
    }
}

/// Build command arguments for call_chain
fn build_call_chain_args(params: &CallChainParams) -> Vec<String> {
    let mut args = vec!["call".to_owned(), "chain".to_owned()];

    args.push("--url".to_owned());
    args.push(params.url.clone());

    let metadata_mode = params.metadata.unwrap_or(false);

    if metadata_mode {
        args.push("--metadata".to_owned());
        if let Some(ref pallet) = params.pallet {
            args.push("--pallet".to_owned());
            args.push(pallet.clone());
        }
    } else {
        // Normal call mode
        if let Some(ref pallet) = params.pallet {
            args.push("--pallet".to_owned());
            args.push(pallet.clone());
        }

        if let Some(ref function) = params.function {
            args.push("--function".to_owned());
            args.push(function.clone());
        }

        if let Some(ref call_args) = params.args {
            if !call_args.is_empty() {
                args.push("--args".to_owned());
                args.extend(call_args.iter().cloned());
            }
        }

        if let Some(ref suri) = params.suri {
            args.push("--suri".to_owned());
            args.push(suri.clone());
        }

        if params.sudo.unwrap_or(false) {
            args.push("--sudo".to_owned());
        }

        // Always skip confirmation for non-interactive use
        args.push("-y".to_owned());
    }

    args
}

/// Check if output contains error indicators from pop CLI
fn is_error_output(output: &str) -> bool {
    let error_indicators = [
        "Error:",
        "error:",
        "Failed to",
        "failed to",
        "Unable to",
        "Pallet with name",    // "Pallet with name X not found"
        "not found in pallet", // "Call with name X not found in pallet Y"
    ];
    error_indicators
        .iter()
        .any(|indicator| output.contains(indicator))
}

/// Execute call_chain tool
pub fn call_chain(executor: &PopExecutor, params: CallChainParams) -> PopMcpResult<CallToolResult> {
    params.validate().map_err(PopMcpError::InvalidInput)?;

    let args = build_call_chain_args(&params);
    let args_refs: Vec<&str> = args.iter().map(String::as_str).collect();

    let metadata_mode = params.metadata.unwrap_or(false);

    match executor.execute(&args_refs) {
        Ok(output) => {
            // In metadata mode, check for specific pallet not found error
            // In call mode, check for general error indicators
            let is_error = if metadata_mode {
                output.contains("Pallet with name") && output.contains("not found")
            } else {
                is_error_output(&output)
            };

            if is_error {
                Ok(error_result(format!("Chain call failed:\n\n{}", output)))
            } else if metadata_mode {
                Ok(success_result(format!(
                    "Chain metadata\n\n{}{}",
                    output, TYPE_HINTS
                )))
            } else {
                Ok(success_result(format!(
                    "Chain call successful!\n\n{}",
                    output
                )))
            }
        }
        Err(e) => Ok(error_result(format!("Chain call failed: {}", e))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_rejects_function_with_metadata() {
        let params = CallChainParams {
            url: "ws://localhost:9944".to_owned(),
            pallet: Some("system".to_owned()),
            function: Some("account".to_owned()),
            args: None,
            suri: None,
            sudo: None,
            metadata: Some(true),
        };
        assert!(params.validate().is_err());
    }

    #[test]
    fn validate_rejects_args_with_metadata() {
        let params = CallChainParams {
            url: "ws://localhost:9944".to_owned(),
            pallet: Some("system".to_owned()),
            function: None,
            args: Some(vec!["arg1".to_owned()]),
            suri: None,
            sudo: None,
            metadata: Some(true),
        };
        assert!(params.validate().is_err());
    }

    #[test]
    fn validate_rejects_suri_with_metadata() {
        let params = CallChainParams {
            url: "ws://localhost:9944".to_owned(),
            pallet: None,
            function: None,
            args: None,
            suri: Some("//Alice".to_owned()),
            sudo: None,
            metadata: Some(true),
        };
        assert!(params.validate().is_err());
    }

    #[test]
    fn validate_rejects_sudo_with_metadata() {
        let params = CallChainParams {
            url: "ws://localhost:9944".to_owned(),
            pallet: None,
            function: None,
            args: None,
            suri: None,
            sudo: Some(true),
            metadata: Some(true),
        };
        assert!(params.validate().is_err());
    }

    #[test]
    fn validate_rejects_missing_pallet_in_call_mode() {
        let params = CallChainParams {
            url: "ws://localhost:9944".to_owned(),
            pallet: None,
            function: Some("account".to_owned()),
            args: None,
            suri: None,
            sudo: None,
            metadata: None,
        };
        assert!(params.validate().is_err());
    }

    #[test]
    fn validate_rejects_missing_function_in_call_mode() {
        let params = CallChainParams {
            url: "ws://localhost:9944".to_owned(),
            pallet: Some("system".to_owned()),
            function: None,
            args: None,
            suri: None,
            sudo: None,
            metadata: None,
        };
        assert!(params.validate().is_err());
    }

    #[test]
    fn validate_accepts_metadata_mode_no_pallet() {
        let params = CallChainParams {
            url: "ws://localhost:9944".to_owned(),
            pallet: None,
            function: None,
            args: None,
            suri: None,
            sudo: None,
            metadata: Some(true),
        };
        assert!(params.validate().is_ok());
    }

    #[test]
    fn validate_accepts_metadata_mode_with_pallet() {
        let params = CallChainParams {
            url: "ws://localhost:9944".to_owned(),
            pallet: Some("system".to_owned()),
            function: None,
            args: None,
            suri: None,
            sudo: None,
            metadata: Some(true),
        };
        assert!(params.validate().is_ok());
    }

    #[test]
    fn validate_accepts_call_mode() {
        let params = CallChainParams {
            url: "ws://localhost:9944".to_owned(),
            pallet: Some("system".to_owned()),
            function: Some("remark".to_owned()),
            args: Some(vec!["0x1234".to_owned()]),
            suri: Some("//Alice".to_owned()),
            sudo: None,
            metadata: None,
        };
        assert!(params.validate().is_ok());
    }

    #[test]
    fn build_args_metadata_list_pallets() {
        let params = CallChainParams {
            url: "ws://localhost:9944".to_owned(),
            pallet: None,
            function: None,
            args: None,
            suri: None,
            sudo: None,
            metadata: Some(true),
        };
        let args = build_call_chain_args(&params);
        assert_eq!(
            args,
            vec![
                "call",
                "chain",
                "--url",
                "ws://localhost:9944",
                "--metadata"
            ]
        );
    }

    #[test]
    fn build_args_metadata_inspect_pallet() {
        let params = CallChainParams {
            url: "ws://localhost:9944".to_owned(),
            pallet: Some("System".to_owned()),
            function: None,
            args: None,
            suri: None,
            sudo: None,
            metadata: Some(true),
        };
        let args = build_call_chain_args(&params);
        assert_eq!(
            args,
            vec![
                "call",
                "chain",
                "--url",
                "ws://localhost:9944",
                "--metadata",
                "--pallet",
                "System"
            ]
        );
    }

    #[test]
    fn build_args_query() {
        let params = CallChainParams {
            url: "ws://localhost:9944".to_owned(),
            pallet: Some("system".to_owned()),
            function: Some("account".to_owned()),
            args: Some(vec![
                "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY".to_owned()
            ]),
            suri: None,
            sudo: None,
            metadata: None,
        };
        let args = build_call_chain_args(&params);
        assert_eq!(
            args,
            vec![
                "call",
                "chain",
                "--url",
                "ws://localhost:9944",
                "--pallet",
                "system",
                "--function",
                "account",
                "--args",
                "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
                "-y"
            ]
        );
    }

    #[test]
    fn build_args_transaction_with_sudo() {
        let params = CallChainParams {
            url: "ws://localhost:9944".to_owned(),
            pallet: Some("system".to_owned()),
            function: Some("remark".to_owned()),
            args: Some(vec!["0x1234".to_owned()]),
            suri: Some("//Alice".to_owned()),
            sudo: Some(true),
            metadata: None,
        };
        let args = build_call_chain_args(&params);
        assert_eq!(
            args,
            vec![
                "call",
                "chain",
                "--url",
                "ws://localhost:9944",
                "--pallet",
                "system",
                "--function",
                "remark",
                "--args",
                "0x1234",
                "--suri",
                "//Alice",
                "--sudo",
                "-y"
            ]
        );
    }

    #[test]
    fn build_args_constant_no_args() {
        let params = CallChainParams {
            url: "ws://localhost:9944".to_owned(),
            pallet: Some("balances".to_owned()),
            function: Some("ExistentialDeposit".to_owned()),
            args: None,
            suri: None,
            sudo: None,
            metadata: None,
        };
        let args = build_call_chain_args(&params);
        assert_eq!(
            args,
            vec![
                "call",
                "chain",
                "--url",
                "ws://localhost:9944",
                "--pallet",
                "balances",
                "--function",
                "ExistentialDeposit",
                "-y"
            ]
        );
    }
}
