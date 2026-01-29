use crate::common::{is_error, is_success, text, InkNode, TestEnv, DEFAULT_SURI};
use anyhow::Result;
use pop_mcp_server::tools::call::chain::{call_chain, CallChainParams};

#[test]
fn call_chain_metadata_lists_pallets() -> Result<()> {
    let env = TestEnv::new()?;
    let (url, _guard) = InkNode::ensure()?;

    let result = call_chain(
        env.executor(),
        CallChainParams {
            url: url.clone(),
            pallet: None,
            function: None,
            args: None,
            suri: None,
            sudo: None,
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
            url: url.clone(),
            pallet: Some("System".to_string()),
            function: None,
            args: None,
            suri: None,
            sudo: None,
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
            url: url.clone(),
            pallet: Some("NonExistentPallet".to_string()),
            function: None,
            args: None,
            suri: None,
            sudo: None,
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
            url: url.clone(),
            pallet: Some("Balances".to_string()),
            function: Some("ExistentialDeposit".to_string()),
            args: None,
            suri: None,
            sudo: None,
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
            url: url.clone(),
            pallet: Some("System".to_string()),
            function: Some("Account".to_string()),
            args: Some(vec![
                "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY".to_string()
            ]),
            suri: None,
            sudo: None,
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
    let env = TestEnv::new()?;
    let (url, _guard) = InkNode::ensure()?;

    // Execute a remark transaction (no state change, just emits event)
    let result = call_chain(
        env.executor(),
        CallChainParams {
            url: url.clone(),
            pallet: Some("System".to_string()),
            function: Some("remark".to_string()),
            args: Some(vec!["0x1234".to_string()]),
            suri: Some(DEFAULT_SURI.to_string()),
            sudo: None,
            metadata: None,
        },
    )?;

    assert!(is_success(&result));
    let output = text(&result)?;
    // Successful transaction returns extrinsic hash
    assert!(output.contains("Extrinsic") || output.contains("hash") || output.contains("0x"));
    Ok(())
}
