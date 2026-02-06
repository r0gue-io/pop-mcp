use crate::common::{is_error, text};
use anyhow::{anyhow, Result};
use pop_mcp_server::executor::PopExecutor;
use pop_mcp_server::tools::clean::{clean_network, CleanNetworkParams};
use pop_mcp_server::tools::up::network::{up_network, UpNetworkParams};
use std::net::TcpListener;
use tempfile::TempDir;

fn allocate_port() -> Result<u16> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    Ok(listener.local_addr()?.port())
}

#[test]
fn up_network_launches() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let relay_ws = allocate_port()?;
    let relay_rpc = allocate_port()?;
    let relay_ws_bob = allocate_port()?;
    let relay_rpc_bob = allocate_port()?;
    let collator_ws = allocate_port()?;
    let collator_rpc = allocate_port()?;
    let config_path = temp_dir.path().join("network.toml");
    let config = format!(
        r#"[relaychain]
chain = "paseo-local"

[[relaychain.nodes]]
name = "alice"
validator = true
ws_port = {relay_ws}
rpc_port = {relay_rpc}

[[relaychain.nodes]]
name = "bob"
validator = true
ws_port = {relay_ws_bob}
rpc_port = {relay_rpc_bob}

[[parachains]]
id = 1000
chain = "asset-hub-paseo-local"

[[parachains.collators]]
name = "asset-hub"
ws_port = {collator_ws}
rpc_port = {collator_rpc}
args = ["-lxcm=trace,lsystem::events=trace,lruntime=trace"]
"#,
    );
    std::fs::write(&config_path, config)?;

    let executor = PopExecutor::new();
    let result = up_network(
        &executor,
        UpNetworkParams {
            path: Some(config_path.to_string_lossy().to_string()),
            chain: None,
            verbose: Some(true),
        },
    )?;

    if is_error(&result) {
        return Err(anyhow!("up_network failed: {}", text(&result)?));
    }

    let output = text(&result)?;
    assert!(output.contains("zombie.json"));

    let _ = clean_network(
        &executor,
        CleanNetworkParams {
            path: None,
            all: Some(true),
            keep_state: Some(false),
        },
    );

    Ok(())
}
