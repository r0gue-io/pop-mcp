//! Chain build (pop build)

use rmcp::model::CallToolResult;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::PopMcpResult;
use crate::executor::PopExecutor;
use crate::tools::common::{error_result, success_result};

/// Parameters for the build_chain tool.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[schemars(extend("properties" = {}))]
pub struct BuildChainParams {
    /// Path to the chain project directory.
    #[schemars(description = "Path to the chain project directory")]
    pub path: String,
    /// Whether to build in release mode (default: true).
    #[schemars(description = "Build in release mode with optimizations (default: true)")]
    pub release: Option<bool>,
}

impl BuildChainParams {
    /// Validate the parameters
    fn validate(&self) -> Result<(), String> {
        if self.path.is_empty() {
            return Err("Path cannot be empty".to_owned());
        }
        Ok(())
    }
}

/// Build command arguments for build_chain
fn build_build_chain_args(params: &BuildChainParams) -> Vec<&str> {
    let mut args = vec!["build", "--path", params.path.as_str()];

    // Default to release mode for chains (production builds)
    if params.release.unwrap_or(true) {
        args.push("--release");
    }

    args
}

/// Execute build_chain tool
pub fn build_chain(
    executor: &PopExecutor,
    params: BuildChainParams,
) -> PopMcpResult<CallToolResult> {
    params
        .validate()
        .map_err(crate::error::PopMcpError::InvalidInput)?;

    let args = build_build_chain_args(&params);

    match executor.execute(&args) {
        Ok(_output) => Ok(success_result("Chain build successful!")),
        Err(e) => Ok(error_result(format!("Chain build failed: {}", e))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_rejects_empty_path() {
        let params = BuildChainParams {
            path: String::new(),
            release: None,
        };
        assert!(params.validate().is_err());
    }

    #[test]
    fn validate_accepts_valid_path() {
        let params = BuildChainParams {
            path: "./my_chain".to_owned(),
            release: None,
        };
        assert!(params.validate().is_ok());
    }

    #[test]
    fn build_args_defaults_to_release() {
        let params = BuildChainParams {
            path: "./my_chain".to_owned(),
            release: None,
        };
        let args = build_build_chain_args(&params);
        assert_eq!(args, vec!["build", "--path", "./my_chain", "--release"]);
    }

    #[test]
    fn build_args_respects_release_true() {
        let params = BuildChainParams {
            path: "./my_chain".to_owned(),
            release: Some(true),
        };
        let args = build_build_chain_args(&params);
        assert_eq!(args, vec!["build", "--path", "./my_chain", "--release"]);
    }

    #[test]
    fn build_args_respects_release_false() {
        let params = BuildChainParams {
            path: "./my_chain".to_owned(),
            release: Some(false),
        };
        let args = build_build_chain_args(&params);
        assert_eq!(args, vec!["build", "--path", "./my_chain"]);
    }
}
