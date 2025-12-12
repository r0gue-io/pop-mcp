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
    pub e2e: Option<bool>,
    #[schemars(description = "Path to local node for e2e tests")]
    pub node: Option<String>,
}

/// Build command arguments for test_contract
pub fn build_test_contract_args<'a>(params: &'a TestContractParams) -> Vec<&'a str> {
    let mut args = vec!["test", "--path", params.path.as_str()];

    if params.e2e.unwrap_or(false) {
        args.push("--e2e");
    }

    if let Some(ref node) = params.node {
        args.push("--node");
        args.push(node.as_str());
    }

    args
}

/// Execute test_contract tool
pub fn test_contract<E: CommandExecutor>(
    executor: &E,
    params: TestContractParams,
) -> PopMcpResult<CallToolResult> {
    let args = build_test_contract_args(&params);

    match executor.execute(&args) {
        Ok(output) => Ok(success_result(format!("Tests completed!\n\n{}", output))),
        Err(e) => Ok(error_result(format!("Tests failed: {}", e))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::test_utils::MockExecutor;

    #[test]
    fn test_build_args_basic() {
        let params = TestContractParams {
            path: "./my_contract".to_string(),
            e2e: None,
            node: None,
        };
        let args = build_test_contract_args(&params);
        assert_eq!(args, vec!["test", "--path", "./my_contract"]);
    }

    #[test]
    fn test_build_args_e2e() {
        let params = TestContractParams {
            path: "./my_contract".to_string(),
            e2e: Some(true),
            node: None,
        };
        let args = build_test_contract_args(&params);
        assert!(args.contains(&"--e2e"));
    }

    #[test]
    fn test_build_args_with_node() {
        let params = TestContractParams {
            path: "./my_contract".to_string(),
            e2e: Some(true),
            node: Some("./node".to_string()),
        };
        let args = build_test_contract_args(&params);
        assert_eq!(
            args,
            vec![
                "test",
                "--path",
                "./my_contract",
                "--e2e",
                "--node",
                "./node"
            ]
        );
    }

    #[test]
    fn test_test_contract_success() {
        let executor = MockExecutor::success("All 5 tests passed!");
        let params = TestContractParams {
            path: "./my_contract".to_string(),
            e2e: None,
            node: None,
        };

        let result = test_contract(&executor, params).unwrap();
        assert!(!result.is_error.unwrap_or(true));
    }

    #[test]
    fn test_test_contract_failure() {
        let executor = MockExecutor::failure("Test failed: assertion error");
        let params = TestContractParams {
            path: "./my_contract".to_string(),
            e2e: None,
            node: None,
        };

        let result = test_contract(&executor, params).unwrap();
        assert!(result.is_error.unwrap_or(false));
    }
}
