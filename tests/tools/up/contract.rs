use crate::common::{ink_node_url, is_error, is_success, text, Contract, TestContext, DEFAULT_SURI};
use anyhow::Result;
use pop_mcp_server::tools::up::contract::{deploy_contract, DeployContractParams};

#[test]
fn deploy_contract_nonexistent_path_fails() -> Result<()> {
    let ctx = TestContext::new()?;
    let executor = ctx.executor()?;
    let params = DeployContractParams {
        path: "/nonexistent/path/to/contract".to_string(),
        constructor: None,
        args: None,
        value: None,
        execute: None,
        suri: None,
        url: None,
    };

    let result = deploy_contract(&executor, params, None)?;
    assert!(is_error(&result));
    assert!(text(&result)?.contains("Deployment failed"));
    Ok(())
}

#[test]
fn deploy_contract_succeeds_and_returns_address() -> Result<()> {
    let ctx = TestContext::new()?;
    let executor = ctx.executor()?;
    let contract = Contract::with_context(ctx, &executor, "deploy_test")?;
    contract.build(&executor)?;
    let node_url = ink_node_url(&executor)?;

    let result = deploy_contract(
        &executor,
        DeployContractParams {
            path: contract.path.to_string_lossy().to_string(),
            constructor: Some("new".to_string()),
            args: Some("false".to_string()),
            value: None,
            execute: Some(true),
            suri: Some(DEFAULT_SURI.to_string()),
            url: Some(node_url),
        },
        None,
    )?;

    assert!(is_success(&result));
    let output = text(&result)?;
    assert!(output.contains("0x") || output.contains("5"));
    Ok(())
}
