use crate::common::{is_error, is_success, pop_executor, text, Contract, DEFAULT_SURI};
use anyhow::{anyhow, Result};
use pop_mcp_server::tools::call::contract::{call_contract, CallContractParams};
use serial_test::serial;

#[test]
fn call_contract_nonexistent_path_fails() -> Result<()> {
    let executor = pop_executor()?;

    // Call with a non-existent contract path - this will definitely fail
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

    let result = call_contract(&executor, params)?;
    assert!(is_error(&result));
    assert!(text(&result)?.contains("Contract call failed"));
    Ok(())
}

#[test]
#[serial]
fn call_contract_get_and_flip_mutates_state() -> Result<()> {
    let executor = pop_executor()?;

    let mut contract = Contract::new(&executor, "call_test")?;
    contract.build(&executor)?;
    contract.deploy(&executor, Some("new"), Some("false"))?;
    let addr = contract
        .address
        .as_ref()
        .ok_or_else(|| anyhow!("Missing contract address after deploy"))?
        .to_string();
    let node_url = contract
        .node_url()
        .ok_or_else(|| anyhow!("Missing node URL after deploy"))?
        .to_string();

    // Initial get - should return false (initial state)
    let params = CallContractParams {
        path: contract.path.to_string_lossy().to_string(),
        contract: addr.clone(),
        message: "get".to_string(),
        args: None,
        value: None,
        execute: None,
        suri: Some(DEFAULT_SURI.to_string()),
        url: Some(node_url.clone()),
    };

    let result = call_contract(&executor, params)?;
    assert!(is_success(&result));
    assert!(text(&result)?.contains("false"));

    // Call flip to mutate state
    let flip_params = CallContractParams {
        path: contract.path.to_string_lossy().to_string(),
        contract: addr.clone(),
        message: "flip".to_string(),
        args: None,
        value: None,
        execute: Some(true),
        suri: Some(DEFAULT_SURI.to_string()),
        url: Some(node_url.clone()),
    };
    let flip_result = call_contract(&executor, flip_params)?;
    assert!(is_success(&flip_result));
    assert!(!text(&flip_result)?.is_empty());

    // Call get again - should now return true
    let get_params = CallContractParams {
        path: contract.path.to_string_lossy().to_string(),
        contract: addr,
        message: "get".to_string(),
        args: None,
        value: None,
        execute: None,
        suri: Some(DEFAULT_SURI.to_string()),
        url: Some(node_url),
    };
    let get_result = call_contract(&executor, get_params)?;
    assert!(is_success(&get_result));
    assert!(text(&get_result)?.contains("true"));
    Ok(())
}
