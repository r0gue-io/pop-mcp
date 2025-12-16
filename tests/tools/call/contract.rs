use crate::common::{content_text, pop_executor, Contract};
use pop_mcp_server::tools::call::contract::{call_contract, CallContractParams};
use serial_test::serial;

#[test]
fn call_contract_failure() {
    let executor = pop_executor();

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

    let result = call_contract(&executor, params, None).unwrap();
    assert!(result.is_error.unwrap());
    assert!(content_text(&result).contains("Contract call failed"));
}

#[test]
#[serial]
fn call_contract_success() {
    let executor = pop_executor();

    let mut contract = Contract::new(&executor, "call_test").expect("Failed to create contract");
    contract.build(&executor).expect("Failed to build contract");
    contract
        .deploy(&executor, Some("new"), Some("false"))
        .expect("Failed to deploy contract");

    let params = CallContractParams {
        path: contract.path.to_string_lossy().to_string(),
        contract: contract.address.clone().unwrap(),
        message: "get".to_string(),
        args: None,
        value: None,
        execute: None,
        suri: Some("//Alice".to_string()),
        url: Some(contract.node_url().unwrap().to_string()),
    };

    let result = call_contract(&executor, params, None).unwrap();
    assert_eq!(result.is_error, Some(false));
    assert!(content_text(&result).contains("false"));
}
