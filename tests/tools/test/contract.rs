use crate::common::{content_text, pop_executor, Contract};
use pop_mcp_server::tools::test::contract::{test_contract, TestContractParams};
use serial_test::serial;

#[test]
#[serial]
fn contract_success_and_e2e() {
    let executor = pop_executor();
    let contract =
        Contract::new(&executor, "test_contract_e2e").expect("Failed to create contract");

    let params = TestContractParams {
        path: contract.path.to_string_lossy().to_string(),
        e2e: false,
    };
    let result = test_contract(&executor, params).unwrap();
    assert_eq!(result.is_error, Some(false));
    assert!(content_text(&result).contains("Tests completed!"));

    let params_e2e = TestContractParams {
        path: contract.path.to_string_lossy().to_string(),
        e2e: true,
    };
    let result_e2e = test_contract(&executor, params_e2e).unwrap();
    assert_eq!(result_e2e.is_error, Some(false));
    assert!(content_text(&result_e2e).contains("Tests completed!"));
}

#[test]
fn contract_nonexistent_path() {
    let executor = pop_executor();
    let params = TestContractParams {
        path: "/nonexistent/path/to/contract".to_string(),
        e2e: false,
    };

    let result = test_contract(&executor, params).unwrap();
    assert!(result.is_error.unwrap());
    assert!(content_text(&result).contains("Tests failed"));
}
