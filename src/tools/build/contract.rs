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
pub fn build_build_contract_args(params: &BuildContractParams) -> Vec<&str> {
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
        .map_err(crate::error::PopMcpError::InvalidInput)?;

    let args = build_build_contract_args(&params);

    match executor.execute(&args) {
        Ok(_output) => Ok(success_result("Build successful!")),
        Err(e) => Ok(error_result(format!("Build failed: {}", e))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::PopExecutor;
    use crate::tools::helpers::{content_text, create_standard_contract, pop_available};
    use serial_test::serial;

    #[test]
    fn validate_rejects_empty_path() {
        let params = BuildContractParams {
            path: "".to_string(),
            release: None,
        };
        assert!(params.validate().is_err());
    }

    #[test]
    fn build_args_include_release_flag() {
        let params = BuildContractParams {
            path: "./my_contract".to_string(),
            release: Some(true),
        };
        let args = build_build_contract_args(&params);
        assert_eq!(args, vec!["build", "--path", "./my_contract", "--release"]);
    }

    #[test]
    fn build_nonexistent_path() {
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
    #[serial]
    fn build_contract_success_creates_artifacts() {
        let executor = PopExecutor::new();
        if !pop_available(&executor) {
            return;
        }

        let fixture = create_standard_contract(&executor, "build_test");

        let build_params = BuildContractParams {
            path: fixture.path.to_string_lossy().to_string(),
            release: None,
        };

        let result = build_contract(&executor, build_params).unwrap();
        assert!(result.is_error.is_some());
        assert!(!content_text(&result).is_empty());

        // Verify build artifacts exist
        assert!(fixture.path.join("target/ink").exists());
    }
}
