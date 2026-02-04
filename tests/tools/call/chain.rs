use crate::common::{is_error, is_success, text, InkNode, PrivateKeyGuard, TestEnv};
use anyhow::Result;
use pop_mcp_server::tools::call::chain::{call_chain, CallChainParams};
use pop_mcp_server::PopMcpError;

#[test]
fn call_chain_metadata_lists_pallets() -> Result<()> {
    let env = TestEnv::new()?;
    let (url, _guard) = InkNode::ensure()?;

    let result = call_chain(
        env.executor(),
        CallChainParams {
            url,
            pallet: None,
            function: None,
            args: None,
            sudo: None,
            execute: None,
            metadata: Some(true),
        },
    )?;

    assert!(is_success(&result));
    let output = text(&result)?;
    // ink-node has System and Balances pallets
    assert!(output.contains("System"));
    assert!(output.contains("Balances"));
    Ok(())
}

#[test]
fn call_chain_metadata_inspects_pallet() -> Result<()> {
    let env = TestEnv::new()?;
    let (url, _guard) = InkNode::ensure()?;

    let result = call_chain(
        env.executor(),
        CallChainParams {
            url,
            pallet: Some("System".to_string()),
            function: None,
            args: None,
            sudo: None,
            execute: None,
            metadata: Some(true),
        },
    )?;

    assert!(is_success(&result));
    let output = text(&result)?;
    // System pallet has remark extrinsic and Account storage
    assert!(output.contains("remark"));
    assert!(output.contains("Account"));
    Ok(())
}

#[test]
fn call_chain_metadata_invalid_pallet_fails() -> Result<()> {
    let env = TestEnv::new()?;
    let (url, _guard) = InkNode::ensure()?;

    let result = call_chain(
        env.executor(),
        CallChainParams {
            url,
            pallet: Some("NonExistentPallet".to_string()),
            function: None,
            args: None,
            sudo: None,
            execute: None,
            metadata: Some(true),
        },
    )?;

    assert!(is_error(&result));
    let output = text(&result)?;
    // Error format: "Failed to find the pallet: NonExistentPallet"
    assert!(output.contains("Failed to find the pallet"));
    Ok(())
}

#[test]
fn call_chain_reads_constant() -> Result<()> {
    let env = TestEnv::new()?;
    let (url, _guard) = InkNode::ensure()?;

    let result = call_chain(
        env.executor(),
        CallChainParams {
            url,
            pallet: Some("Balances".to_string()),
            function: Some("ExistentialDeposit".to_string()),
            args: None,
            sudo: None,
            execute: None,
            metadata: None,
        },
    )?;

    assert!(is_success(&result));
    // ExistentialDeposit returns a numeric value
    let output = text(&result)?;
    assert!(output.chars().any(|c| c.is_ascii_digit()));
    Ok(())
}

#[test]
fn call_chain_queries_storage() -> Result<()> {
    let env = TestEnv::new()?;
    let (url, _guard) = InkNode::ensure()?;

    // Query Alice's account (dev account that should exist)
    let result = call_chain(
        env.executor(),
        CallChainParams {
            url,
            pallet: Some("System".to_string()),
            function: Some("Account".to_string()),
            args: Some(vec![
                "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY".to_string()
            ]),
            sudo: None,
            execute: None,
            metadata: None,
        },
    )?;

    assert!(is_success(&result));
    let output = text(&result)?;
    // Account info contains nonce and data fields
    assert!(output.contains("nonce") || output.contains("data") || output.contains("free"));
    Ok(())
}

#[test]
fn call_chain_executes_transaction() -> Result<()> {
    let _guard = PrivateKeyGuard::set();
    let env = TestEnv::new()?;
    let (url, _guard) = InkNode::ensure()?;

    // Execute a remark transaction (no state change, just emits event)
    let result = call_chain(
        env.executor(),
        CallChainParams {
            url,
            pallet: Some("System".to_string()),
            function: Some("remark".to_string()),
            args: Some(vec!["0x1234".to_string()]),
            sudo: None,
            execute: Some(true),
            metadata: None,
        },
    )?;

    assert!(is_success(&result));
    let output = text(&result)?;
    // Successful transaction returns extrinsic hash
    assert!(output.contains("Extrinsic") || output.contains("hash") || output.contains("0x"));
    Ok(())
}

#[test]
fn call_chain_transaction_uses_env_suri() -> Result<()> {
    let _guard = PrivateKeyGuard::set();
    let env = TestEnv::new()?;
    let (url, _guard) = InkNode::ensure()?;

    let result = call_chain(
        env.executor(),
        CallChainParams {
            url,
            pallet: Some("System".to_string()),
            function: Some("remark".to_string()),
            args: Some(vec!["0x5678".to_string()]),
            sudo: None,
            execute: Some(true),
            metadata: None,
        },
    )?;

    assert!(is_success(&result));
    let output = text(&result)?;
    assert!(output.contains("Extrinsic") || output.contains("hash") || output.contains("0x"));
    Ok(())
}

#[test]
fn call_chain_execute_requires_private_key() -> Result<()> {
    let _guard = PrivateKeyGuard::clear();

    let err = call_chain(
        TestEnv::new()?.executor(),
        CallChainParams {
            url: "ws://localhost:9944".to_string(),
            pallet: Some("System".to_string()),
            function: Some("remark".to_string()),
            args: Some(vec!["0x9999".to_string()]),
            sudo: None,
            execute: Some(true),
            metadata: None,
        },
    )
    .unwrap_err();

    let PopMcpError::InvalidInput(message) = err else {
        panic!("expected InvalidInput error when PRIVATE_KEY is missing");
    };
    assert!(message.contains("PRIVATE_KEY"));
    Ok(())
}
