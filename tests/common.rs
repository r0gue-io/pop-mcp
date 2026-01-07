#![cfg(feature = "pop-e2e")]

use anyhow::{anyhow, Context, Result};
use pop_mcp_server::executor::PopExecutor;
use pop_mcp_server::tools::build::contract::{build_contract, BuildContractParams};
use pop_mcp_server::tools::common::extract_text;
use pop_mcp_server::tools::new::contract::{create_contract, CreateContractParams};
use pop_mcp_server::tools::up::chain::{up_ink_node, UpInkNodeParams};
use pop_mcp_server::tools::up::contract::{deploy_contract, DeployContractParams};
use rmcp::model::{CallToolResult, RawContent};
use std::net::{TcpStream, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};
use tempfile::TempDir;

/// Default signer URI for test transactions.
pub const DEFAULT_SURI: &str = "//Alice";

pub fn is_error(result: &CallToolResult) -> bool {
    result.is_error == Some(true)
}

pub fn is_success(result: &CallToolResult) -> bool {
    !is_error(result)
}

pub fn text(result: &CallToolResult) -> Result<String> {
    extract_text(result).ok_or_else(|| anyhow!("CallToolResult missing text content"))
}

pub fn texts(result: &CallToolResult) -> Vec<String> {
    result
        .content
        .iter()
        .filter_map(|c| match &c.raw {
            RawContent::Text(t) => Some(t.text.clone()),
            _ => None,
        })
        .collect()
}

pub struct TestEnv {
    tempdir: TempDir,
    executor: PopExecutor,
}

impl TestEnv {
    /// Create a temporary workdir and verify Pop CLI is available.
    pub fn new() -> Result<Self> {
        let tempdir = TempDir::new().context("Failed to create temp dir")?;
        let executor = PopExecutor::with_cwd(tempdir.path().to_path_buf());
        executor
            .execute(&["--version"])
            .map_err(|e| anyhow!("Pop CLI not available: {e}"))?;
        Ok(Self { tempdir, executor })
    }

    pub fn executor(&self) -> &PopExecutor {
        &self.executor
    }

    pub fn workdir(&self) -> &Path {
        self.tempdir.path()
    }
}

pub struct InkNode;

impl InkNode {
    /// Port for shared test node (different from default 9944 to avoid conflicts).
    pub const PORT: u16 = 9945;
    /// ETH RPC port for shared test node.
    pub const ETH_PORT: u16 = 8546;

    /// Start the shared ink-node if needed and return its URL.
    pub fn start_or_get_url() -> Result<&'static str> {
        static URL: OnceLock<String> = OnceLock::new();
        static INIT: Mutex<()> = Mutex::new(());

        if let Some(url) = URL.get() {
            return Ok(url.as_str());
        }

        let _guard = INIT.lock().unwrap();
        if let Some(url) = URL.get() {
            return Ok(url.as_str());
        }

        // Start the node
        let executor = PopExecutor::new();
        let result = up_ink_node(
            &executor,
            UpInkNodeParams {
                ink_node_port: Some(Self::PORT),
                eth_rpc_port: Some(Self::ETH_PORT),
            },
        )
        .map_err(|e| anyhow!(e.to_string()))?;

        if is_error(&result) {
            let msg = text(&result).unwrap_or_else(|_| "Failed to launch ink-node".to_string());
            return Err(anyhow!(msg));
        }

        let url = extract_text(&result)
            .ok_or_else(|| anyhow!("Failed to extract URL from ink-node output"))?;

        // Wait for node to be ready
        wait_for_port("127.0.0.1", Self::PORT, Duration::from_secs(30))
            .context("ink-node not listening on expected port")?;

        URL.set(url).ok();
        Ok(URL.get().unwrap().as_str())
    }
}

const SHARED_CONTRACT_NAME: &str = "shared_contract";
static SHARED_CONTRACT_DIR: OnceLock<TempDir> = OnceLock::new();
static SHARED_CONTRACT_INIT: Mutex<()> = Mutex::new(());

fn shared_contract_path() -> Result<PathBuf> {
    if let Some(dir) = SHARED_CONTRACT_DIR.get() {
        return Ok(dir.path().join(SHARED_CONTRACT_NAME));
    }

    let _guard = SHARED_CONTRACT_INIT.lock().unwrap();
    if let Some(dir) = SHARED_CONTRACT_DIR.get() {
        return Ok(dir.path().join(SHARED_CONTRACT_NAME));
    }

    // Create and build shared contract
    let tempdir = TempDir::new().context("Failed to create temp dir")?;
    let executor = PopExecutor::with_cwd(tempdir.path().to_path_buf());

    let create_result = create_contract(
        &executor,
        CreateContractParams {
            name: SHARED_CONTRACT_NAME.to_string(),
            template: "standard".to_string(),
        },
    )
    .context("Failed to create shared contract")?;

    if is_error(&create_result) {
        return Err(anyhow!(
            "Shared contract creation failed: {}",
            text(&create_result)?
        ));
    }

    let path = tempdir.path().join(SHARED_CONTRACT_NAME);

    let build_result = build_contract(
        &executor,
        BuildContractParams {
            path: path.display().to_string(),
            release: None,
        },
    )
    .context("Failed to build shared contract")?;

    if is_error(&build_result) {
        return Err(anyhow!(
            "Shared contract build failed: {}",
            text(&build_result)?
        ));
    }

    SHARED_CONTRACT_DIR.set(tempdir).ok();
    Ok(path)
}

pub struct Contract {
    pub path: PathBuf,
    address: Option<String>,
}

impl Contract {
    /// Create, build, or reuse the shared contract for testing.
    pub fn create_build_or_use() -> Result<Self> {
        Ok(Contract {
            path: shared_contract_path()?,
            address: None,
        })
    }

    /// Deploy to shared ink-node.
    pub fn deploy(&mut self, constructor: &str, args: &str) -> Result<()> {
        let url = InkNode::start_or_get_url()?;
        let executor = PopExecutor::new();

        let result = deploy_contract(
            &executor,
            DeployContractParams {
                path: self.path.to_string_lossy().to_string(),
                constructor: Some(constructor.to_string()),
                args: Some(args.to_string()),
                value: None,
                execute: Some(true),
                suri: Some(DEFAULT_SURI.to_string()),
                url: Some(url.to_string()),
            },
            None,
        )
        .context("Failed to deploy contract")?;

        if is_error(&result) {
            return Err(anyhow!("Deployment failed: {}", text(&result)?));
        }

        let output = text(&result)?;
        self.address =
            Some(parse_contract_address(&output).ok_or_else(|| {
                anyhow!("Failed to parse contract address from output: {}", output)
            })?);

        Ok(())
    }

    /// Get deployed address.
    pub fn address(&self) -> &str {
        self.address.as_ref().expect("Contract not deployed")
    }
}

/// Parse contract address from deployment output.
fn parse_contract_address(output: &str) -> Option<String> {
    // Ethereum-style: "0x..."
    if let Some(start) = output.find("\"0x") {
        let addr_start = start + 1;
        if let Some(end) = output[addr_start..].find('\"') {
            let address = &output[addr_start..addr_start + end];
            if address.len() == 42 && address.starts_with("0x") {
                return Some(address.to_string());
            }
        }
    }

    // SS58: starts with 5, 47-48 chars
    for word in output.split_whitespace() {
        if word.starts_with('5')
            && word.len() >= 47
            && word.len() <= 48
            && word.chars().all(|c| c.is_alphanumeric())
        {
            return Some(word.to_string());
        }
    }

    None
}

fn wait_until(timeout: Duration, mut condition: impl FnMut() -> bool) -> bool {
    let start = Instant::now();
    while start.elapsed() < timeout {
        if condition() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    false
}

fn wait_for_port(host: &str, port: u16, timeout: Duration) -> Result<()> {
    if wait_until(timeout, || is_port_open(host, port)) {
        return Ok(());
    }
    Err(anyhow!("Timed out waiting for port {host}:{port}"))
}

pub fn wait_for_port_closed(port: u16, timeout: Duration) -> Result<()> {
    if wait_until(timeout, || !is_port_open("127.0.0.1", port)) {
        return Ok(());
    }
    Err(anyhow!("Timed out waiting for port {port} to close"))
}

pub fn is_port_in_use(port: u16) -> bool {
    is_port_open("127.0.0.1", port)
}

fn is_port_open(host: &str, port: u16) -> bool {
    match (host, port).to_socket_addrs() {
        Ok(mut addrs) => {
            addrs.any(|addr| TcpStream::connect_timeout(&addr, Duration::from_millis(200)).is_ok())
        }
        Err(_) => false,
    }
}

pub fn ws_port_from_url(url: &str) -> Result<u16> {
    let url = url
        .trim()
        .strip_prefix("ws://")
        .ok_or_else(|| anyhow!("Expected ws:// URL"))?;
    let host_port = url.split('/').next().unwrap_or(url);
    let port = host_port
        .rsplit_once(':')
        .ok_or_else(|| anyhow!("Missing port in URL"))?
        .1;
    port.parse().context("Invalid port")
}

pub fn parse_pids(output: &str) -> Result<Vec<u32>> {
    fn parse_pid_list(input: &str) -> Vec<u32> {
        input
            .split_whitespace()
            .filter_map(|token| {
                let token = token.trim_matches(|c: char| !c.is_ascii_digit());
                if token.is_empty() {
                    None
                } else {
                    token.parse::<u32>().ok()
                }
            })
            .collect()
    }

    for line in output.lines() {
        let trimmed = line.trim().trim_start_matches('│').trim();
        if let Some(rest) = trimmed.strip_prefix("pids:") {
            let pids = parse_pid_list(rest);
            if !pids.is_empty() {
                return Ok(pids);
            }
        }
        if let Some(start) = trimmed.find("kill -9") {
            let rest = &trimmed[start + "kill -9".len()..];
            let pids = parse_pid_list(rest);
            if !pids.is_empty() {
                return Ok(pids);
            }
        }
    }

    Err(anyhow!("No pids found in output"))
}
