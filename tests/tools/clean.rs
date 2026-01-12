use crate::common::{
    is_port_in_use, is_success, parse_pids, text, texts, wait_for_port_closed, ws_port_from_url,
    InkNode, TestEnv,
};
use anyhow::Result;
use pop_mcp_server::tools::clean::{clean_nodes, CleanNodesParams};
use pop_mcp_server::tools::up::chain::{up_ink_node, UpInkNodeParams};
use std::time::Duration;

/// Ports for lifecycle test (different from shared node on 9945/8546).
const TEST_INK_PORT: u16 = 9946;
const TEST_ETH_PORT: u16 = 8547;

#[test]
fn up_ink_node_and_clean_nodes_lifecycle() -> Result<()> {
    // Ensure shared node is ready first to avoid concurrent Pop CLI node bootstrapping.
    // Pop CLI cannot handle multiple simultaneous `pop up ink-node` invocations.
    InkNode::start_or_get_url()?;

    let env = TestEnv::new()?;

    // Launch ink-node on separate ports
    let result = up_ink_node(
        env.executor(),
        UpInkNodeParams {
            ink_node_port: Some(TEST_INK_PORT),
            eth_rpc_port: Some(TEST_ETH_PORT),
        },
    )?;
    if !is_success(&result) {
        panic!("up_ink_node failed: {}", text(&result)?);
    }

    // Verify URL and node is listening
    let url = text(&result)?;
    assert!(url.starts_with("ws://"));
    let port = ws_port_from_url(&url)?;
    assert!(is_port_in_use(port));

    // Parse PIDs
    let pid_output = texts(&result).join("\n");
    let pids = parse_pids(&pid_output)?;
    assert!(!pids.is_empty());

    // Clean up
    let result = clean_nodes(env.executor(), CleanNodesParams { pids })?;
    assert!(is_success(&result));
    wait_for_port_closed(port, Duration::from_secs(30))?;

    Ok(())
}
