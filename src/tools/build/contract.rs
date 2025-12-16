//! Contract build (pop build)

use rmcp::model::CallToolResult;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::PopMcpResult;
use crate::executor::CommandExecutor;
use crate::tools::common::{error_result, success_result};

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
}
