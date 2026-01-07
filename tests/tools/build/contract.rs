use crate::common::{is_error, is_success, text, TestEnv};
use anyhow::Result;
use pop_mcp_server::tools::build::contract::{build_contract, BuildContractParams};
use pop_mcp_server::tools::new::contract::{create_contract, CreateContractParams};

#[test]
fn build_contract_nonexistent_path_fails() -> Result<()> {
    let env = TestEnv::new()?;
    let params = BuildContractParams {
        path: "/nonexistent/path/to/contract".to_string(),
        release: None,
    };

    let result = build_contract(env.executor(), params)?;
    assert!(is_error(&result));
    assert!(text(&result)?.contains("Build failed"));
    Ok(())
}

#[test]
fn build_contract_creates_ink_artifacts() -> Result<()> {
    let env = TestEnv::new()?;

    // Create a fresh contract
    create_contract(
        env.executor(),
        CreateContractParams {
            name: "build_test".to_string(),
            template: "standard".to_string(),
        },
    )?;

    let contract_path = env.workdir().join("build_test");

    // Build it
    let result = build_contract(
        env.executor(),
        BuildContractParams {
            path: contract_path.display().to_string(),
            release: None,
        },
    )?;

    assert!(is_success(&result));
    assert!(text(&result)?.contains("Build successful"));
    assert!(contract_path.join("target/ink").exists());
    Ok(())
}
