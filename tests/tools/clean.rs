use crate::common::{is_port_in_use, pop_executor};
use pop_mcp_server::tools::clean::{clean_nodes, CleanNodesParams};
use pop_mcp_server::tools::up::chain::{up_ink_node, UpInkNodeParams};
use serial_test::serial;

#[test]
#[serial]
fn clean_nodes_stops_running_node() {
    let executor = pop_executor();

    // Use raw up_ink_node since we're specifically testing clean_nodes
    let result = up_ink_node(&executor, UpInkNodeParams {}).unwrap();
    assert_eq!(result.is_error, Some(false));
    assert!(
        is_port_in_use(9944),
        "Port 9944 should be in use after launch"
    );

    // Test clean_nodes functionality
    let result = clean_nodes(&executor, CleanNodesParams {}).unwrap();
    assert_eq!(result.is_error, Some(false));

    assert!(
        !is_port_in_use(9944),
        "Port 9944 should be free after clean"
    );
}
