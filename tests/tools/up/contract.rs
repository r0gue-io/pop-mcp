use crate::common::{is_error, is_success, pop_executor, text, Contract, InkNode, DEFAULT_SURI};
use anyhow::Result;
use pop_mcp_server::tools::up::contract::{deploy_contract, DeployContractParams};
use serial_test::serial;

#[test]
fn deploy_contract_nonexistent_path_fails() -> Result<()> {
    let executor = pop_executor()?;
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
#[serial]
fn deploy_contract_succeeds_and_returns_address() -> Result<()> {
    let executor = pop_executor()?;
    let contract = Contract::new(&executor, "deploy_test")?;
    contract.build(&executor)?;
    let node = InkNode::launch(&executor)?;

    let result = deploy_contract(
        &executor,
        DeployContractParams {
            path: contract.path.to_string_lossy().to_string(),
            constructor: Some("new".to_string()),
            args: Some("false".to_string()),
            value: None,
            execute: Some(true),
            suri: Some(DEFAULT_SURI.to_string()),
            url: Some(node.url.clone()),
        },
        None,
    )?;

    assert!(is_success(&result));
    let output = text(&result)?;
    assert!(output.contains("0x") || output.contains("5"));
    Ok(())
}
