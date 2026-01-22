//! Chain creation (pop new chain)

use rmcp::model::CallToolResult;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::PopMcpResult;
use crate::executor::PopExecutor;
use crate::tools::common::{error_result, success_result};

/// Parameters for the create_chain tool.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[schemars(extend("properties" = {}))]
pub struct CreateChainParams {
    /// Name of the chain project.
    #[schemars(
        description = "Name of the chain project directory (alphanumeric characters and underscores only)"
    )]
    pub name: String,

    /// Provider for the chain template.
    #[schemars(description = "Template provider: 'pop', 'openzeppelin', or 'parity'")]
    pub provider: String,

    /// Template to use for the chain.
    #[schemars(
        description = "Full template path: 'r0gue-io/base-parachain', 'r0gue-io/assets-parachain', 'r0gue-io/contracts-parachain' (pop), 'openzeppelin/generic-template', 'openzeppelin/evm-template' (openzeppelin), 'paritytech/polkadot-sdk-parachain-template' (parity)"
    )]
    pub template: String,

    /// Token symbol for the chain (Pop templates only).
    #[schemars(
        description = "Native token symbol (default: 'UNIT') - only applies to Pop templates"
    )]
    pub symbol: Option<String>,

    /// Token decimals for the chain (Pop templates only).
    #[schemars(description = "Token decimals (default: 12) - only applies to Pop templates")]
    pub decimals: Option<u8>,
}

impl CreateChainParams {
    /// Validate the chain name and parameters.
    fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Chain name cannot be empty".to_owned());
        }
        if !self.name.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(
                "Chain names can only contain alphanumeric characters and underscores".to_owned(),
            );
        }

        // Validate provider
        let valid_providers = ["pop", "openzeppelin", "parity"];
        if !valid_providers.contains(&self.provider.to_lowercase().as_str()) {
            return Err(format!(
                "Invalid provider '{}'. Valid providers: pop, openzeppelin, parity",
                self.provider
            ));
        }

        // Validate template matches provider
        let provider_lower = self.provider.to_lowercase();
        let template_lower = self.template.to_lowercase();

        let valid = match provider_lower.as_str() {
            "pop" => template_lower.starts_with("r0gue-io/"),
            "openzeppelin" => template_lower.starts_with("openzeppelin/"),
            "parity" => template_lower.starts_with("paritytech/"),
            _ => false,
        };

        if !valid {
            return Err(format!(
                "Template '{}' does not match provider '{}'. Use a template from the correct provider.",
                self.template, self.provider
            ));
        }

        Ok(())
    }
}

/// Build command arguments for create_chain.
fn build_create_chain_args(params: &CreateChainParams) -> Vec<String> {
    let mut args = vec![
        "new".to_owned(),
        "chain".to_owned(),
        params.name.clone(),
        params.provider.clone(),
        "--template".to_owned(),
        params.template.clone(),
    ];

    // Add symbol if provided (Pop templates only)
    if let Some(ref symbol) = params.symbol {
        args.push("--symbol".to_owned());
        args.push(symbol.clone());
    }

    // Add decimals if provided (Pop templates only)
    if let Some(decimals) = params.decimals {
        args.push("--decimals".to_owned());
        args.push(decimals.to_string());
    }

    args
}

/// Execute create_chain tool.
pub fn create_chain(
    executor: &PopExecutor,
    params: CreateChainParams,
) -> PopMcpResult<CallToolResult> {
    // Validate parameters
    params
        .validate()
        .map_err(crate::error::PopMcpError::InvalidInput)?;

    let args = build_create_chain_args(&params);
    let args_refs: Vec<&str> = args.iter().map(String::as_str).collect();

    match executor.execute(&args_refs) {
        Ok(output) => {
            // Check for common error patterns in output
            if output.contains("directory already exists")
                || output.contains("doesn't support")
                || output.contains("incorrect initial endowment")
            {
                Ok(error_result(format!("Failed to create chain: {}", output)))
            } else {
                Ok(success_result(format!(
                    "Successfully created chain project: {}\n\nNext steps:\n\
                    1. cd {}\n\
                    2. pop build --release\n\
                    3. pop up network -f ./network.toml\n\n{}",
                    params.name, params.name, output
                )))
            }
        }
        Err(e) => Ok(error_result(format!("Failed to create chain: {}", e))),
    }
}

#[cfg(test)]
#[allow(clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn validate_allows_valid_names() {
        for name in ["my_chain", "chain123", "test_chain_v2"] {
            let params = CreateChainParams {
                name: (*name).to_owned(),
                provider: "pop".to_owned(),
                template: "r0gue-io/base-parachain".to_owned(),
                symbol: None,
                decimals: None,
            };
            assert!(params.validate().is_ok(), "Expected {} to be valid", name);
        }
    }

    #[test]
    fn validate_rejects_invalid_names() {
        for name in ["", "my-chain", "my chain", "my@chain", "my.chain"] {
            let params = CreateChainParams {
                name: (*name).to_owned(),
                provider: "pop".to_owned(),
                template: "r0gue-io/base-parachain".to_owned(),
                symbol: None,
                decimals: None,
            };
            assert!(
                params.validate().is_err(),
                "Expected {} to be invalid",
                name
            );
        }
    }

    #[test]
    fn validate_rejects_invalid_provider() {
        let params = CreateChainParams {
            name: "my_chain".to_owned(),
            provider: "invalid".to_owned(),
            template: "r0gue-io/base-parachain".to_owned(),
            symbol: None,
            decimals: None,
        };
        let result = params.validate();
        assert!(result.is_err());
        assert!(
            result.err().is_some_and(|e| e.contains("Invalid provider")),
            "Expected error to contain 'Invalid provider'"
        );
    }

    #[test]
    fn validate_rejects_mismatched_provider_template() {
        // Pop provider with OpenZeppelin template
        let params = CreateChainParams {
            name: "my_chain".to_owned(),
            provider: "pop".to_owned(),
            template: "openzeppelin/generic-template".to_owned(),
            symbol: None,
            decimals: None,
        };
        let result = params.validate();
        assert!(result.is_err());
        assert!(
            result
                .err()
                .is_some_and(|e| e.contains("does not match provider")),
            "Expected error to contain 'does not match provider'"
        );
    }

    #[test]
    fn validate_accepts_all_valid_combinations() {
        let valid_combinations = [
            ("pop", "r0gue-io/base-parachain"),
            ("pop", "r0gue-io/assets-parachain"),
            ("pop", "r0gue-io/contracts-parachain"),
            ("openzeppelin", "openzeppelin/generic-template"),
            ("openzeppelin", "openzeppelin/evm-template"),
            ("parity", "paritytech/polkadot-sdk-parachain-template"),
        ];

        for (provider, template) in valid_combinations {
            let params = CreateChainParams {
                name: "my_chain".to_owned(),
                provider: provider.to_owned(),
                template: template.to_owned(),
                symbol: None,
                decimals: None,
            };
            assert!(
                params.validate().is_ok(),
                "Expected ({}, {}) to be valid",
                provider,
                template
            );
        }
    }

    #[test]
    fn build_args_includes_required_params() {
        let params = CreateChainParams {
            name: "my_chain".to_owned(),
            provider: "pop".to_owned(),
            template: "r0gue-io/base-parachain".to_owned(),
            symbol: None,
            decimals: None,
        };
        let args = build_create_chain_args(&params);
        assert_eq!(
            args,
            vec![
                "new",
                "chain",
                "my_chain",
                "pop",
                "--template",
                "r0gue-io/base-parachain"
            ]
        );
    }

    #[test]
    fn build_args_includes_optional_symbol_and_decimals() {
        let params = CreateChainParams {
            name: "my_chain".to_owned(),
            provider: "pop".to_owned(),
            template: "r0gue-io/base-parachain".to_owned(),
            symbol: Some("TOKEN".to_owned()),
            decimals: Some(18),
        };
        let args = build_create_chain_args(&params);
        assert_eq!(
            args,
            vec![
                "new",
                "chain",
                "my_chain",
                "pop",
                "--template",
                "r0gue-io/base-parachain",
                "--symbol",
                "TOKEN",
                "--decimals",
                "18"
            ]
        );
    }
}
