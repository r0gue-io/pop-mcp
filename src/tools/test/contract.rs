//! Contract testing (pop test)

use rmcp::model::CallToolResult;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::PopMcpResult;
use crate::executor::CommandExecutor;
use crate::tools::common::{error_result, success_result};

// Parameters

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct TestContractParams {
    #[schemars(description = "Path to the contract directory")]
    pub path: String,
    #[schemars(description = "Run end-to-end tests")]
    pub e2e: bool,
}

impl TestContractParams {
    /// Validate the parameters
    pub fn validate(&self) -> Result<(), String> {
        if self.path.is_empty() {
            return Err("Path cannot be empty".to_string());
        }
        Ok(())
    }
}

/// Build command arguments for test_contract
pub fn build_test_contract_args(params: &TestContractParams) -> Vec<&str> {
    let mut args = vec!["test", "--path", params.path.as_str()];

    if params.e2e {
        args.push("--e2e");
    }

    args
}

/// Execute test_contract tool
pub fn test_contract<E: CommandExecutor>(
    executor: &E,
    params: TestContractParams,
) -> PopMcpResult<CallToolResult> {
    params
        .validate()
        .map_err(crate::error::PopMcpError::InvalidInput)?;

    let args = build_test_contract_args(&params);

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
        let params = TestContractParams {
            path: "".to_string(),
            e2e: false,
        };
        assert!(params.validate().is_err());
    }

    #[test]
    fn build_args_basic() {
        let params = TestContractParams {
            path: "./my_contract".to_string(),
            e2e: false,
        };
        let args = build_test_contract_args(&params);
        assert_eq!(args, vec!["test", "--path", "./my_contract"]);
    }

    #[test]
    fn build_args_e2e() {
        let params = TestContractParams {
            path: "./my_contract".to_string(),
            e2e: true,
        };
        let args = build_test_contract_args(&params);
        assert_eq!(args, vec!["test", "--path", "./my_contract", "--e2e"]);
    }
}
