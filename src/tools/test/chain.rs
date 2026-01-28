//! Chain testing (pop test)

use rmcp::model::CallToolResult;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::PopMcpResult;
use crate::executor::PopExecutor;
use crate::tools::common::{error_result, success_result};

/// Parameters for the test_chain tool.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[schemars(extend("properties" = {}))]
pub struct TestChainParams {
    /// Path to the chain project directory.
    #[schemars(description = "Path to the chain project directory")]
    pub path: String,
}

impl TestChainParams {
    /// Validate the parameters
    fn validate(&self) -> Result<(), String> {
        if self.path.is_empty() {
            return Err("Path cannot be empty".to_owned());
        }
        Ok(())
    }
}

/// Build command arguments for test_chain
fn build_test_chain_args(params: &TestChainParams) -> Vec<&str> {
    vec!["test", "--path", params.path.as_str()]
}

/// Execute test_chain tool
pub fn test_chain(executor: &PopExecutor, params: TestChainParams) -> PopMcpResult<CallToolResult> {
    params
        .validate()
        .map_err(crate::error::PopMcpError::InvalidInput)?;

    let args = build_test_chain_args(&params);

    match executor.execute(&args) {
        Ok(output) => Ok(success_result(format!("Tests completed!\n\n{}", output))),
        Err(e) => Ok(error_result(format!("Tests failed: {}", e))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_rejects_empty_path() {
        let params = TestChainParams {
            path: String::new(),
        };
        assert!(params.validate().is_err());
    }

    #[test]
    fn validate_accepts_valid_path() {
        let params = TestChainParams {
            path: "./my_chain".to_owned(),
        };
        assert!(params.validate().is_ok());
    }

    #[test]
    fn build_args_basic() {
        let params = TestChainParams {
            path: "./my_chain".to_owned(),
        };
        let args = build_test_chain_args(&params);
        assert_eq!(args, vec!["test", "--path", "./my_chain"]);
    }
}
