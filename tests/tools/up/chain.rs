use crate::common::{is_port_in_use, pop_executor, InkNode, DEFAULT_NODE_PORT, DEFAULT_NODE_URL};
use anyhow::Result;
use serial_test::serial;

#[test]
#[serial]
fn up_ink_node_launches_and_listens_on_default_port() -> Result<()> {
    let executor = pop_executor()?;

    let node = InkNode::launch(&executor)?;

    assert_eq!(node.url, DEFAULT_NODE_URL);
    assert!(is_port_in_use(DEFAULT_NODE_PORT));

    // Node cleaned up automatically when `node` drops
    Ok(())
}
