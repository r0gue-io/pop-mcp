//! Contract build (pop build)

use rmcp::model::CallToolResult;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::PopMcpResult;
use crate::executor::CommandExecutor;
use crate::tools::helpers::{error_result, success_result};

// Parameters

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct BuildContractParams {
    #[schemars(description = "Path to the contract directory")]
    pub path: String,
    #[schemars(description = "Build in release mode with optimizations")]
    pub release: Option<bool>,
}

impl BuildContractParams {
    /// Validate the parameters
    pub fn validate(&self) -> Result<(), String> {
        if self.path.is_empty() {
            return Err("Path cannot be empty".to_string());
        }
        Ok(())
    }
}

/// Build command arguments for build_contract
pub fn build_build_contract_args<'a>(params: &'a BuildContractParams) -> Vec<&'a str> {
    let mut args = vec!["build", "--path", params.path.as_str()];

    if params.release.unwrap_or(false) {
        args.push("--release");
    }

    args
}

/// Execute build_contract tool
pub fn build_contract<E: CommandExecutor>(
    executor: &E,
    params: BuildContractParams,
) -> PopMcpResult<CallToolResult> {
    params
        .validate()
        .map_err(|e| crate::error::PopMcpError::InvalidInput(e))?;

    let args = build_build_contract_args(&params);

    match executor.execute(&args) {
        Ok(output) => Ok(success_result(format!("Build successful!\n\n{}", output))),
        Err(e) => Ok(error_result(format!("Build failed: {}", e))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::PopExecutor;
    use crate::tools::new::contract::{create_contract, CreateContractParams};
    use rmcp::model::RawContent;
    use std::env;
    use tempfile::tempdir;

    fn content_text(result: &rmcp::model::CallToolResult) -> String {
        result
            .content
            .last()
            .and_then(|c| match &c.raw {
                RawContent::Text(t) => Some(t.text.clone()),
                _ => None,
            })
            .unwrap_or_default()
    }

    #[test]
    fn test_validate_rejects_empty_path() {
        let params = BuildContractParams {
            path: "".to_string(),
            release: None,
        };
        assert!(params.validate().is_err());
    }

    #[test]
    fn test_build_args() {
        let params = BuildContractParams {
            path: "./my_contract".to_string(),
            release: Some(true),
        };
        let args = build_build_contract_args(&params);
        assert_eq!(args, vec!["build", "--path", "./my_contract", "--release"]);
    }

    #[test]
    fn test_build_nonexistent_path() {
        let executor = PopExecutor::new();
        let params = BuildContractParams {
            path: "/nonexistent/path/to/contract".to_string(),
            release: None,
        };

        let result = build_contract(&executor, params).unwrap();
        assert!(result.is_error.unwrap_or(false));

        let text = content_text(&result);
        assert!(text.contains("Build failed"));
    }

    #[test]
    fn test_build_contract() {
        let executor = PopExecutor::new();

        let dir = tempdir().unwrap();
        env::set_current_dir(dir.path()).expect("Failed to change dir");

        // First create a contract
        let contract_name = "build_test";
        let create_params = CreateContractParams {
            name: contract_name.to_string(),
            template: "standard".to_string(),
        };
        let create_result = create_contract(&executor, create_params);
        assert!(create_result.is_ok());

        // Build the contract
        let contract_path = dir.path().join(contract_name);
        let build_params = BuildContractParams {
            path: contract_path.to_string_lossy().to_string(),
            release: None,
        };

        let result = build_contract(&executor, build_params).unwrap();
        assert!(!result.is_error.unwrap());

        let text = content_text(&result);
        assert!(text.contains("Build successful"));

        // Verify build artifacts exist
        assert!(contract_path.join("target/ink").exists());
    }
}
