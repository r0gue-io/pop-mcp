use crate::common::{content_text, pop_executor, Contract, InkNode};
use pop_mcp_server::tools::up::contract::{deploy_contract, DeployContractParams};
use serial_test::serial;

#[test]
fn deploy_nonexistent_path() {
    let executor = pop_executor();
    let params = DeployContractParams {
        path: "/nonexistent/path/to/contract".to_string(),
        constructor: None,
        args: None,
        value: None,
        execute: None,
        suri: None,
        url: None,
    };

    let result = deploy_contract(&executor, params, None).unwrap();
    assert!(result.is_error.unwrap());
    assert!(content_text(&result).contains("Deployment failed"));
}

#[test]
#[serial]
fn deploy_contract_success() {
    let executor = pop_executor();
    let contract = Contract::new(&executor, "deploy_test").expect("Failed to create contract");
    contract.build(&executor).expect("Failed to build contract");
    let node = InkNode::launch(&executor).expect("Failed to launch node");

    let result = deploy_contract(
        &executor,
        DeployContractParams {
            path: contract.path.to_string_lossy().to_string(),
            constructor: Some("new".to_string()),
            args: Some("false".to_string()),
            value: None,
            execute: Some(true),
            suri: Some("//Alice".to_string()),
            url: Some(node.url.clone()),
        },
        None,
    )
    .unwrap();

    assert_eq!(result.is_error, Some(false));
    assert!(content_text(&result).contains("0x")); // Has contract address
}
