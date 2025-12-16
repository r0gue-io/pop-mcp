use crate::common::{is_error, is_success, pop_executor, text};
use anyhow::Result;
use pop_mcp_server::tools::new::contract::{create_contract, CreateContractParams};
use serial_test::serial;
use tempfile::TempDir;

#[test]
#[serial]
fn create_standard_contract_succeeds() -> Result<()> {
    let executor = pop_executor()?;
    let temp_dir = TempDir::new()?;
    let original_dir = std::env::current_dir()?;
    std::env::set_current_dir(temp_dir.path())?;

    let contract_name = "test_contract";
    let contract_path = temp_dir.path().join(contract_name);

    let params = CreateContractParams {
        name: contract_name.to_string(),
        template: "standard".to_string(),
    };

    let result = create_contract(&executor, params);
    std::env::set_current_dir(&original_dir)?;

    let result = result?;
    assert!(is_success(&result));

    let text = text(&result)?;
    assert!(text.starts_with(&format!("Successfully created contract: {}", contract_name)));
    assert!(contract_path.exists());
    assert!(contract_path.join("Cargo.toml").exists());
    assert!(contract_path.join("lib.rs").exists());
    Ok(())
}

#[test]
fn create_contract_invalid_name_fails_before_execution() -> Result<()> {
    let executor = pop_executor()?;
    let params = CreateContractParams {
        name: "invalid-name".to_string(),
        template: "standard".to_string(),
    };
    let result = create_contract(&executor, params);
    assert!(result.is_err());
    Ok(())
}

#[test]
fn create_contract_cli_failure() -> Result<()> {
    let executor = pop_executor()?;
    let params = CreateContractParams {
        name: "test_contract".to_string(),
        template: "non_existing".to_string(),
    };
    let result = create_contract(&executor, params)?;

    assert!(is_error(&result));
    let text = text(&result)?;
    assert!(text.starts_with("Failed to create contract:"));
    Ok(())
}
