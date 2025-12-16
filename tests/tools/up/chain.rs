use crate::common::{is_port_in_use, pop_executor, InkNode};
use anyhow::Result;
use serial_test::serial;

#[test]
#[serial]
fn up_ink_node_launches_node() -> Result<()> {
    let executor = pop_executor()?;

    let node = InkNode::launch(&executor)?;

    assert_eq!(node.url, "ws://localhost:9944");
    assert!(
        is_port_in_use(9944),
        "Port 9944 should be in use after launch"
    );

    // Node cleaned up automatically when `node` drops
    Ok(())
}
