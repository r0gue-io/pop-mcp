use crate::common::{content_text, pop_executor};
use pop_mcp_server::tools::new::contract::{create_contract, CreateContractParams};
use serial_test::serial;
use tempfile::TempDir;

#[test]
#[serial]
fn create_standard_contract_succeeds() {
    let executor = pop_executor();
    let temp_dir = TempDir::new().unwrap();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let contract_name = "test_contract";
    let contract_path = temp_dir.path().join(contract_name);

    let params = CreateContractParams {
        name: contract_name.to_string(),
        template: "standard".to_string(),
    };

    let result = create_contract(&executor, params);
    std::env::set_current_dir(&original_dir).unwrap();

    assert!(result.is_ok());
    let result = result.unwrap();
    assert_eq!(result.is_error, Some(false));

    let text = content_text(&result);
    assert!(text.starts_with(&format!("Successfully created contract: {}", contract_name)));
    assert!(contract_path.exists());
    assert!(contract_path.join("Cargo.toml").exists());
    assert!(contract_path.join("lib.rs").exists());
}

#[test]
fn create_contract_invalid_name_fails_before_execution() {
    let executor = pop_executor();
    let params = CreateContractParams {
        name: "invalid-name".to_string(),
        template: "standard".to_string(),
    };
    let result = create_contract(&executor, params);
    assert!(result.is_err());
}

#[test]
fn create_contract_cli_failure() {
    let executor = pop_executor();
    let params = CreateContractParams {
        name: "test_contract".to_string(),
        template: "non_existing".to_string(),
    };
    let result = create_contract(&executor, params).unwrap();

    assert!(result.is_error.unwrap());
    let text = content_text(&result);
    assert!(text.starts_with("Failed to create contract:"));
}
