use crate::common::{is_error, is_success, pop_executor, text, Contract};
use anyhow::Result;
use pop_mcp_server::tools::build::contract::{build_contract, BuildContractParams};
use serial_test::serial;

#[test]
fn build_contract_nonexistent_path_fails() -> Result<()> {
    let executor = pop_executor()?;
    let params = BuildContractParams {
        path: "/nonexistent/path/to/contract".to_string(),
        release: None,
    };

    let result = build_contract(&executor, params)?;
    assert!(is_error(&result));
    assert!(text(&result)?.contains("Build failed"));
    Ok(())
}

#[test]
#[serial]
fn build_contract_creates_ink_artifacts() -> Result<()> {
    let executor = pop_executor()?;
    let contract = Contract::new(&executor, "build_test")?;

    let build_params = BuildContractParams {
        path: contract.path.to_string_lossy().to_string(),
        release: None,
    };

    let result = build_contract(&executor, build_params)?;
    assert!(is_success(&result));

    let output = text(&result)?;
    assert!(output.contains("Build successful"));

    // Verify build artifacts exist
    assert!(contract.path.join("target/ink").exists());
    Ok(())
}
