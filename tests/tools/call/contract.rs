use crate::common::{is_error, is_success, text, Contract, InkNode, TestEnv, DEFAULT_SURI};
use anyhow::Result;
use pop_mcp_server::tools::call::contract::{call_contract, CallContractParams};

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
        suri: None,
        url: None,
    };

    let result = call_contract(env.executor(), params)?;
    assert!(is_error(&result));
    assert!(text(&result)?.contains("Contract call failed"));
    Ok(())
}

#[test]
fn call_contract_get_and_flip_mutates_state() -> Result<()> {
    let env = TestEnv::new()?;
    let mut contract = Contract::create_build_or_use()?;
    contract.deploy("new", "false")?;

    let url = InkNode::start_or_get_url()?.to_string();
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
            suri: Some(DEFAULT_SURI.to_string()),
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
            suri: Some(DEFAULT_SURI.to_string()),
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
            suri: Some(DEFAULT_SURI.to_string()),
            url: Some(url),
        },
    )?;
    assert!(is_success(&get_result));
    assert!(text(&get_result)?.contains("true"));
    Ok(())
}
