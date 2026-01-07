use crate::common::{is_error, is_success, text, TestContext};
use anyhow::Result;
use pop_mcp_server::tools::new::contract::{create_contract, CreateContractParams};

#[test]
fn create_contract_standard_template_creates_files() -> Result<()> {
    let ctx = TestContext::new()?;
    let executor = ctx.executor()?;

    let contract_name = "test_contract";
    let contract_path = ctx.workdir.join(contract_name);

    let params = CreateContractParams {
        name: contract_name.to_string(),
        template: "standard".to_string(),
    };

    let result = create_contract(&executor, params)?;
    assert!(is_success(&result));

    let output = text(&result)?;
    assert!(output.starts_with(&format!("Successfully created contract: {}", contract_name)));
    assert!(contract_path.exists());
    assert!(contract_path.join("Cargo.toml").exists());
    assert!(contract_path.join("lib.rs").exists());
    Ok(())
}

#[test]
fn create_contract_invalid_name_with_hyphen_fails_validation() -> Result<()> {
    let ctx = TestContext::new()?;
    let executor = ctx.executor()?;
    let params = CreateContractParams {
        name: "invalid-name".to_string(),
        template: "standard".to_string(),
    };
    let result = create_contract(&executor, params);
    assert!(result.is_err());
    Ok(())
}

#[test]
fn create_contract_nonexistent_template_fails() -> Result<()> {
    let ctx = TestContext::new()?;
    let executor = ctx.executor()?;
    let params = CreateContractParams {
        name: "test_contract".to_string(),
        template: "non_existing".to_string(),
    };
    let result = create_contract(&executor, params)?;
    assert!(is_error(&result));
    assert!(text(&result)?.starts_with("Failed to create contract:"));
    Ok(())
}
