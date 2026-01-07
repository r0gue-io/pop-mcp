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
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};
use tempfile::TempDir;

/// Default signer URI for test transactions
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

fn wait_for_port(host: &str, port: u16, timeout: Duration) -> Result<()> {
    let start = Instant::now();
    while start.elapsed() < timeout {
        if is_port_open(host, port) {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    Err(anyhow!("Timed out waiting for port {host}:{port}"))
}

pub fn wait_for_port_closed(port: u16, timeout: Duration) -> Result<()> {
    let start = Instant::now();
    while start.elapsed() < timeout {
        if !is_port_in_use(port) {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    Err(anyhow!("Timed out waiting for port {port} to close"))
}

static NODE_URL: OnceLock<String> = OnceLock::new();
static NODE_INIT: Mutex<()> = Mutex::new(());

/// Port used by the shared test node (different from default 9944 to avoid conflicts).
pub const SHARED_NODE_INK_PORT: u16 = 9945;
/// ETH RPC port for shared test node (different from default 8545).
pub const SHARED_NODE_ETH_PORT: u16 = 8546;

/// Start a local ink-node once per test process and return its WebSocket URL.
///
/// Uses non-default ports (9945/8546) to avoid conflicts with node lifecycle tests.
/// Thread-safe: uses OnceLock + Mutex to ensure only one node is started even
/// when called from multiple threads simultaneously.
pub fn ink_node_url(executor: &PopExecutor) -> Result<String> {
    // Fast path: already initialized (lock-free read)
    if let Some(url) = NODE_URL.get() {
        return Ok(url.clone());
    }

    // Slow path: acquire lock to ensure only one thread initializes
    let _guard = NODE_INIT.lock().unwrap();

    // Double-check after acquiring lock (another thread may have initialized)
    if let Some(url) = NODE_URL.get() {
        return Ok(url.clone());
    }

    // We're the first: start the node
    let result = up_ink_node(
        executor,
        UpInkNodeParams {
            ink_node_port: Some(SHARED_NODE_INK_PORT),
            eth_rpc_port: Some(SHARED_NODE_ETH_PORT),
        },
    )
    .map_err(|e| anyhow!(e.to_string()))?;
    if is_error(&result) {
        let msg = text(&result).unwrap_or_else(|_| "Failed to launch ink-node".to_string());
        return Err(anyhow!(msg));
    }
    let url = extract_text(&result)
        .ok_or_else(|| anyhow!("Failed to extract URL from ink-node output"))?;

    let (host, port) = parse_ws_host_port(&url).context("Failed to parse ink-node URL")?;
    wait_for_port(&host, port, Duration::from_secs(30))
        .context("ink-node not listening on expected port")?;

    NODE_URL.set(url.clone()).ok();
    Ok(url)
}

/// Parse the host and port from a `ws://` URL.
fn parse_ws_host_port(url: &str) -> Result<(String, u16)> {
    let url = url
        .trim()
        .strip_prefix("ws://")
        .ok_or_else(|| anyhow!("Expected ws:// URL, got {url}"))?;
    let host_port = url.split('/').next().unwrap_or(url);
    let (host, port) = host_port
        .rsplit_once(':')
        .ok_or_else(|| anyhow!("Missing port in URL: {host_port}"))?;
    let port = port
        .parse::<u16>()
        .context("Invalid port in ink-node URL")?;
    Ok((host.to_string(), port))
}

/// Extract the port from a `ws://` URL.
#[allow(dead_code)]
pub fn ws_port_from_url(url: &str) -> Result<u16> {
    parse_ws_host_port(url).map(|(_, port)| port)
}

/// Check if a local port is accepting TCP connections.
pub fn is_port_in_use(port: u16) -> bool {
    is_port_open("127.0.0.1", port)
}

fn is_port_open(host: &str, port: u16) -> bool {
    let addrs = (host, port).to_socket_addrs();
    match addrs {
        Ok(mut addrs) => addrs.any(|addr| {
            TcpStream::connect_timeout(&addr, Duration::from_millis(200)).is_ok()
        }),
        Err(_) => false,
    }
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

/// Test context providing an isolated working directory.
///
/// Provides a temporary workdir for contract operations without mutating
/// global cwd. The real HOME/XDG environment is preserved so tests can
/// access cached binaries (e.g., ink-node).
pub struct TestContext {
    /// Temporary directory (kept alive for test duration).
    #[allow(dead_code)]
    temp_dir: TempDir,
    /// Working directory for contract operations.
    pub workdir: PathBuf,
}

impl TestContext {
    /// Create a new test context with an isolated working directory.
    pub fn new() -> Result<Self> {
        let temp_dir = TempDir::new().context("Failed to create temp dir")?;
        let workdir = temp_dir.path().to_path_buf();

        Ok(Self { temp_dir, workdir })
    }

    /// Create a PopExecutor configured to use this context's workdir.
    pub fn executor(&self) -> Result<PopExecutor> {
        let no_env: Vec<(String, String)> = vec![];
        let executor = PopExecutor::with_overrides(Some(self.workdir.clone()), no_env);
        executor
            .execute(&["--version"])
            .map_err(|e| anyhow!("Pop CLI is not available: {e}"))?;
        Ok(executor)
    }
}

/// A contract in a temp directory with optional build and deployment state.
///
/// Use `Contract::with_context()` for isolated test execution without global
/// cwd mutation.
pub struct Contract {
    /// Test context providing isolated temp directories.
    #[allow(dead_code)]
    context: TestContext,
    pub path: PathBuf,
    pub address: Option<String>,
    node_url: Option<String>,
}

impl Contract {
    /// Create a new contract using a TestContext for isolation.
    ///
    /// The executor should be obtained from `ctx.executor()`.
    pub fn with_context(ctx: TestContext, executor: &PopExecutor, name: &str) -> Result<Self> {
        let create_params = CreateContractParams {
            name: name.to_string(),
            template: "standard".to_string(),
        };
        let create_result =
            create_contract(executor, create_params).context("Failed to create contract")?;
        if is_error(&create_result) {
            let msg = text(&create_result)?;
            return Err(anyhow!("Contract creation failed: {}", msg));
        }

        let contract_path = ctx.workdir.join(name);

        Ok(Contract {
            context: ctx,
            path: contract_path,
            address: None,
            node_url: None,
        })
    }

    /// Build the contract.
    pub fn build(&self, executor: &PopExecutor) -> Result<()> {
        let build_params = BuildContractParams {
            path: self.path.to_string_lossy().to_string(),
            release: None,
        };
        let build_result =
            build_contract(executor, build_params).context("Failed to build contract")?;
        if is_error(&build_result) {
            let msg = text(&build_result)?;
            return Err(anyhow!("Contract build failed: {}", msg));
        }
        Ok(())
    }

    /// Launch an ink node and deploy the contract.
    /// Sets the contract address on success.
    pub fn deploy(
        &mut self,
        executor: &PopExecutor,
        constructor: Option<&str>,
        args: Option<&str>,
    ) -> Result<()> {
        let url = ink_node_url(executor).context("Failed to get shared ink-node URL")?;

        // Deploy the contract
        let deploy_params = DeployContractParams {
            path: self.path.to_string_lossy().to_string(),
            constructor: constructor.map(String::from),
            args: args.map(String::from),
            value: None,
            execute: Some(true),
            suri: Some(DEFAULT_SURI.to_string()),
            url: Some(url.clone()),
        };
        let deploy_result =
            deploy_contract(executor, deploy_params, None).context("Failed to deploy contract")?;
        if is_error(&deploy_result) {
            let msg = text(&deploy_result)?;
            return Err(anyhow!("Contract deployment failed: {}", msg));
        }

        // Parse contract address from output
        let output = text(&deploy_result)?;
        let address = parse_contract_address(&output)
            .ok_or_else(|| anyhow!("Failed to parse contract address from output: {}", output))?;

        self.address = Some(address);
        self.node_url = Some(url);
        Ok(())
    }

    /// Get the node URL if deployed.
    pub fn node_url(&self) -> Option<&str> {
        self.node_url.as_deref()
    }
}

/// Parse contract address from deployment output.
/// Supports both Ethereum-style hex addresses (0x...) and SS58 addresses (5...).
fn parse_contract_address(output: &str) -> Option<String> {
    // First, try to find Ethereum-style address in quotes: "0x..."
    // The output format is: The contract address is "0x..."
    if let Some(start) = output.find("\"0x") {
        let addr_start = start + 1; // Skip the opening quote
        if let Some(end) = output[addr_start..].find('"') {
            let address = &output[addr_start..addr_start + end];
            // Verify it's a valid Ethereum address (0x + 40 hex chars)
            if address.len() == 42 && address.starts_with("0x") {
                return Some(address.to_string());
            }
        }
    }

    // Fallback: Look for SS58 addresses (start with 5 and are 47-48 characters)
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
