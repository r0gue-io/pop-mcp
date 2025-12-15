//! Contract testing (pop test)

use rmcp::model::CallToolResult;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::PopMcpResult;
use crate::executor::CommandExecutor;
use crate::tools::helpers::{error_result, success_result};

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
    use crate::executor::PopExecutor;
    use crate::tools::helpers::{content_text, create_standard_contract, pop_available};
    use serial_test::serial;

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
        assert!(args.contains(&"--e2e"));
    }

    #[test]
    #[serial]
    fn contract_success_and_e2e() {
        let executor = PopExecutor::new();
        if !pop_available(&executor) {
            return;
        }

        let contract = create_standard_contract(&executor, "test_contract_e2e");

        // Normal test
        let params = TestContractParams {
            path: contract.path.to_string_lossy().to_string(),
            e2e: false,
        };
        let result = test_contract(&executor, params).unwrap();
        assert_eq!(result.is_error, Some(false));
        assert!(content_text(&result).contains("Tests completed!"));

        // E2E test
        let params_e2e = TestContractParams {
            path: contract.path.to_string_lossy().to_string(),
            e2e: true,
        };
        let result_e2e = test_contract(&executor, params_e2e).unwrap();
        assert_eq!(result_e2e.is_error, Some(false));
        assert!(content_text(&result_e2e).contains("Tests completed!"));
    }

    #[test]
    fn contract_failure() {
        let executor = PopExecutor::new();
        let params = TestContractParams {
            path: "./my_contract".to_string(),
            e2e: false,
        };

        let result = test_contract(&executor, params).unwrap();
        assert!(result.is_error.unwrap());

        let text = content_text(&result);
        assert!(text.contains("Tests failed"));
    }

    #[test]
    #[serial]
    fn contract_nonexistent_path() {
        let executor = PopExecutor::new();
        let params = TestContractParams {
            path: "/nonexistent/path/to/contract".to_string(),
            e2e: false,
        };

        let result = test_contract(&executor, params).unwrap();
        assert!(result.is_error.unwrap());

        let text = content_text(&result);
        assert!(text.contains("Tests failed"));
    }
}
