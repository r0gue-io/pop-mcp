use crate::common::{is_error, is_success, text, Contract, InkNode, PrivateKeyGuard, TestEnv};
use anyhow::Result;
use pop_mcp_server::tools::call::contract::{call_contract, CallContractParams};
use pop_mcp_server::PopMcpError;

#[test]
fn call_contract_nonexistent_path_fails() -> Result<()> {
    let env = TestEnv::new()?;
    let params = CallContractParams {
        path: "/nonexistent/path/to/contract".to_string(),
        contract: "0x0000000000000000000000000000000000000000".to_string(),
        message: "get".to_string(),
        args: None,
        value: None,
        execute: None,
        url: None,
    };

    let result = call_contract(env.executor(), params)?;
    assert!(is_error(&result));
    assert!(text(&result)?.contains("Contract call failed"));
    Ok(())
}

#[test]
fn call_contract_get_and_flip_mutates_state() -> Result<()> {
    let _guard = PrivateKeyGuard::set();
    let env = TestEnv::new()?;
    let (url, _guard) = InkNode::ensure()?;
    let mut contract = Contract::create_build_or_use()?;
    contract.deploy(&url, "new", "false")?;
    let addr = contract.address().to_string();
    let path = contract.path.display().to_string();

    // Initial get - should return false
    let result = call_contract(
        env.executor(),
        CallContractParams {
            path: path.clone(),
            contract: addr.clone(),
            message: "get".to_string(),
            args: None,
            value: None,
            execute: None,
            url: Some(url.clone()),
        },
    )?;
    assert!(is_success(&result));
    assert!(text(&result)?.contains("false"));

    // Flip to mutate state
    let flip_result = call_contract(
        env.executor(),
        CallContractParams {
            path: path.clone(),
            contract: addr.clone(),
            message: "flip".to_string(),
            args: None,
            value: None,
            execute: Some(true),
            url: Some(url.clone()),
        },
    )?;
    assert!(is_success(&flip_result));

    // Get again - should return true
    let get_result = call_contract(
        env.executor(),
        CallContractParams {
            path,
            contract: addr,
            message: "get".to_string(),
            args: None,
            value: None,
            execute: None,
            url: Some(url),
        },
    )?;
    assert!(is_success(&get_result));
    assert!(text(&get_result)?.contains("true"));

    Ok(())
}

#[test]
fn call_contract_execute_requires_private_key() -> Result<()> {
    let _guard = PrivateKeyGuard::clear();

    let err = call_contract(
        TestEnv::new()?.executor(),
        CallContractParams {
            path: "dummy_contract".to_string(),
            contract: "0x1234".to_string(),
            message: "flip".to_string(),
            args: None,
            value: None,
            execute: Some(true),
            url: Some("ws://localhost:9944".to_string()),
        },
    )
    .unwrap_err();

    let PopMcpError::InvalidInput(message) = err else {
        panic!("expected InvalidInput error when PRIVATE_KEY is missing");
    };
    assert!(message.contains("PRIVATE_KEY"));
    Ok(())
}
