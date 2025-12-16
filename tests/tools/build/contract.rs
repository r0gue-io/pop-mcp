use crate::common::{content_text, pop_executor, Contract};
use pop_mcp_server::tools::build::contract::{build_contract, BuildContractParams};
use serial_test::serial;

#[test]
fn build_nonexistent_path_fails() {
    let executor = pop_executor();
    let params = BuildContractParams {
        path: "/nonexistent/path/to/contract".to_string(),
        release: None,
    };

    let result = build_contract(&executor, params).unwrap();
    assert!(result.is_error.unwrap_or(false));
    assert!(content_text(&result).contains("Build failed"));
}

#[test]
#[serial]
fn build_contract_success_creates_artifacts() {
    let executor = pop_executor();
    let contract = Contract::new(&executor, "build_test").expect("Failed to create contract");

    let build_params = BuildContractParams {
        path: contract.path.to_string_lossy().to_string(),
        release: None,
    };

    let result = build_contract(&executor, build_params).unwrap();
    assert_eq!(result.is_error, Some(false));
    assert_eq!(content_text(&result), "Build successful!");

    // Verify build artifacts exist
    assert!(contract.path.join("target/ink").exists());
}
