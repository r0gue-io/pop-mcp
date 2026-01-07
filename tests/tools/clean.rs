use crate::common::{
    is_port_in_use, is_success, parse_pids, text, texts, wait_for_port_closed, ws_port_from_url,
    TestContext,
};
use anyhow::Result;
use pop_mcp_server::tools::clean::{clean_nodes, CleanNodesParams};
use pop_mcp_server::tools::up::chain::{up_ink_node, UpInkNodeParams};
use serial_test::serial;

/// Ports used by this test (different from shared node on 9945/8546 and defaults 9944/8545).
const TEST_INK_PORT: u16 = 9946;
const TEST_ETH_PORT: u16 = 8547;

/// Test node lifecycle: launch ink-node, verify it's running, then clean it up.
///
/// This test covers both `up_ink_node` and `clean_nodes` functionality:
/// - Verifies `up_ink_node` launches successfully and returns URL + PIDs
/// - Verifies the node is actually listening on the expected port
/// - Verifies `clean_nodes` terminates the node processes
/// - Verifies the port is released after cleanup
///
/// Uses explicit ports (9944/8545) which don't conflict with the shared test node
/// that runs on different ports (9945/8546).
#[test]
#[serial]
fn up_ink_node_and_clean_nodes_lifecycle() -> Result<()> {
    let ctx = TestContext::new()?;
    let executor = ctx.executor()?;

    // Launch ink-node with explicit ports
    let result = up_ink_node(
        &executor,
        UpInkNodeParams {
            ink_node_port: Some(TEST_INK_PORT),
            eth_rpc_port: Some(TEST_ETH_PORT),
        },
    )?;
    if !is_success(&result) {
        let err = text(&result).unwrap_or_else(|_| "unknown error".to_string());
        panic!("up_ink_node failed: {}", err);
    }

    // Verify URL is returned and node is listening
    let url = text(&result)?;
    assert!(url.starts_with("ws://"));
    let port = ws_port_from_url(&url)?;
    assert!(is_port_in_use(port));

    // Parse PIDs from output
    let pid_output = texts(&result).join("\n");
    let pids = parse_pids(&pid_output)?;
    assert!(!pids.is_empty());

    // Clean up the node
    let result = clean_nodes(&executor, CleanNodesParams { pids })?;
    assert!(is_success(&result));

    // Verify port is released
    wait_for_port_closed(port, std::time::Duration::from_secs(10))?;
    assert!(!is_port_in_use(port));

    Ok(())
}
