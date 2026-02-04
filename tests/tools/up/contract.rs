use crate::common::{is_error, is_success, text, Contract, InkNode, PrivateKeyGuard, TestEnv};
use anyhow::Result;
use pop_mcp_server::tools::up::contract::{deploy_contract, DeployContractParams};
use pop_mcp_server::PopMcpError;

#[test]
fn deploy_contract_nonexistent_path_fails() -> Result<()> {
    let env = TestEnv::new()?;
    let params = DeployContractParams {
        path: "/nonexistent/path/to/contract".to_string(),
        constructor: None,
        args: None,
        value: None,
        execute: None,
        url: None,
    };

    let result = deploy_contract(env.executor(), params, None)?;
    assert!(is_error(&result));
    assert!(text(&result)?.contains("Deployment failed"));
    Ok(())
}

#[test]
fn deploy_contract_succeeds_and_returns_address() -> Result<()> {
    let _guard = PrivateKeyGuard::set();
    let env = TestEnv::new()?;
    let (url, _guard) = InkNode::ensure()?;
    let contract = Contract::create_build_or_use()?;

    let result = deploy_contract(
        env.executor(),
        DeployContractParams {
            path: contract.path.display().to_string(),
            constructor: Some("new".to_string()),
            args: Some("false".to_string()),
            value: None,
            execute: Some(true),
            url: Some(url.clone()),
        },
        None,
    )?;

    assert!(is_success(&result));
    let output = text(&result)?;
    assert!(output.contains("0x") || output.contains("5"));
    Ok(())
}

#[test]
fn deploy_contract_execute_requires_private_key() -> Result<()> {
    let _guard = PrivateKeyGuard::clear();

    let err = deploy_contract(
        TestEnv::new()?.executor(),
        DeployContractParams {
            path: "dummy_contract".to_string(),
            constructor: Some("new".to_string()),
            args: Some("false".to_string()),
            value: None,
            execute: Some(true),
            url: Some("ws://localhost:9944".to_string()),
        },
        None,
    )
    .unwrap_err();

    let PopMcpError::InvalidInput(message) = err else {
        panic!("expected InvalidInput error when PRIVATE_KEY is missing");
    };
    assert!(message.contains("PRIVATE_KEY"));
    Ok(())
}
