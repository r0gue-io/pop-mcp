//! Integration tests for pop-mcp-server.
#![cfg(feature = "pop-e2e")]
#![allow(
    missing_docs,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::str_to_string,
    clippy::use_self,
    clippy::redundant_closure_for_method_calls
)]

mod common {
    //! Common test utilities and fixtures for pop-mcp-server integration tests.

    use anyhow::{anyhow, Context, Result};
    use pop_mcp_server::executor::PopExecutor;
    // use pop_mcp_server::tools::build::chain::{build_chain, BuildChainParams};
    use pop_mcp_server::tools::build::contract::{build_contract, BuildContractParams};
    use pop_mcp_server::tools::common::extract_text;
    // use pop_mcp_server::tools::new::chain::{create_chain, CreateChainParams};
    use pop_mcp_server::tools::new::contract::{create_contract, CreateContractParams};
    use pop_mcp_server::tools::up::chain::{up_ink_node, UpInkNodeParams};
    use pop_mcp_server::tools::up::contract::{deploy_contract, DeployContractParams};
    use rmcp::model::{CallToolResult, RawContent};
    use std::net::{TcpStream, ToSocketAddrs};
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Mutex, OnceLock,
    };
    use std::time::{Duration, Instant};
    use tempfile::TempDir;

    /// Default signer URI for test transactions.
    pub(crate) const DEFAULT_SURI: &str = "//Alice";
    static PRIVATE_KEY_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    pub(crate) struct PrivateKeyGuard {
        _lock: std::sync::MutexGuard<'static, ()>,
        previous: Option<String>,
    }

    impl PrivateKeyGuard {
        pub(crate) fn set() -> Self {
            let lock = PRIVATE_KEY_LOCK
                .get_or_init(|| Mutex::new(()))
                .lock()
                .expect("Failed to lock PRIVATE_KEY guard");
            let previous = std::env::var("PRIVATE_KEY").ok();
            std::env::set_var("PRIVATE_KEY", DEFAULT_SURI);
            Self {
                _lock: lock,
                previous,
            }
        }

        pub(crate) fn clear() -> Self {
            let lock = PRIVATE_KEY_LOCK
                .get_or_init(|| Mutex::new(()))
                .lock()
                .expect("Failed to lock PRIVATE_KEY guard");
            let previous = std::env::var("PRIVATE_KEY").ok();
            std::env::remove_var("PRIVATE_KEY");
            Self {
                _lock: lock,
                previous,
            }
        }
    }

    impl Drop for PrivateKeyGuard {
        fn drop(&mut self) {
            if let Some(previous) = self.previous.take() {
                std::env::set_var("PRIVATE_KEY", previous);
            } else {
                std::env::remove_var("PRIVATE_KEY");
            }
        }
    }

    pub(crate) fn is_error(result: &CallToolResult) -> bool {
        result.is_error == Some(true)
    }

    pub(crate) fn is_success(result: &CallToolResult) -> bool {
        !is_error(result)
    }

    pub(crate) fn text(result: &CallToolResult) -> Result<String> {
        extract_text(result).ok_or_else(|| anyhow!("CallToolResult missing text content"))
    }

    pub(crate) fn texts(result: &CallToolResult) -> Vec<String> {
        result
            .content
            .iter()
            .filter_map(|c| match &c.raw {
                RawContent::Text(t) => Some(t.text.clone()),
                _ => None,
            })
            .collect()
    }

    pub(crate) struct TestEnv {
        tempdir: TempDir,
        executor: PopExecutor,
    }

    impl TestEnv {
        /// Create a temporary workdir and verify Pop CLI is available.
        pub(crate) fn new() -> Result<Self> {
            let tempdir = TempDir::new().context("Failed to create temp dir")?;
            let executor = PopExecutor::with_cwd(tempdir.path().to_path_buf());
            executor
                .execute(&["--version"])
                .map_err(|e| anyhow!("Pop CLI not available: {e}"))?;
            Ok(Self { tempdir, executor })
        }

        pub(crate) fn executor(&self) -> &PopExecutor {
            &self.executor
        }

        pub(crate) fn workdir(&self) -> &Path {
            self.tempdir.path()
        }
    }

    pub(crate) struct InkNode;

    impl InkNode {
        /// Port for shared test node (different from default 9944 to avoid conflicts).
        pub(crate) const PORT: u16 = 9945;
        /// ETH RPC port for shared test node.
        pub(crate) const ETH_PORT: u16 = 8546;

        /// Start the shared ink-node if needed and return its URL.
        /// Uses a Mutex to allow restart if node was killed.
        fn start_or_get_url() -> Result<String> {
            let mut node = NODE.lock().unwrap();

            // Check if we have a node and it's still alive
            if let Some(ref info) = *node {
                if is_port_open("127.0.0.1", Self::PORT) {
                    info.users.fetch_add(1, Ordering::SeqCst);
                    return Ok(info.url.clone());
                }
                // Node is dead, will restart below
            }

            // Start (or restart) the node
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

            if url.is_empty() || !url.starts_with("ws://") {
                return Err(anyhow!("Invalid URL from ink-node: '{}'", url));
            }

            let output = texts(&result).join("\n");
            let pids = parse_pids(&output).context("Failed to parse ink-node PIDs")?;

            // Wait for node to be ready
            wait_for_port("127.0.0.1", Self::PORT, Duration::from_secs(30))
                .context("ink-node not listening on expected port")?;

            // Create new node info with counter=1 (this caller)
            let info = NodeInfo {
                url: url.clone(),
                users: AtomicUsize::new(1),
                pids,
            };

            *node = Some(info);
            Ok(url)
        }

        pub(crate) fn ensure() -> Result<(String, SharedNodeGuard)> {
            let url = Self::start_or_get_url()?;
            let guard = SharedNodeGuard::new();
            Ok((url, guard))
        }
    }

    struct NodeInfo {
        url: String,
        users: AtomicUsize,
        pids: Vec<u32>,
    }

    /// Global node mutex - used by both start_or_get_url and cleanup
    static NODE: Mutex<Option<NodeInfo>> = Mutex::new(None);

    const SHARED_CONTRACT_NAME: &str = "shared_contract";
    static SHARED_CONTRACT_DIR: OnceLock<TempDir> = OnceLock::new();
    static SHARED_CONTRACT_INIT: Mutex<()> = Mutex::new(());

    // const SHARED_CHAIN_NAME: &str = "shared_chain";
    // static SHARED_CHAIN_DIR: OnceLock<TempDir> = OnceLock::new();
    // static SHARED_CHAIN_INIT: Mutex<()> = Mutex::new(());

    fn shared_contract_path() -> Result<PathBuf> {
        // Check for override path first
        if let Ok(path) = std::env::var("POP_E2E_SHARED_CONTRACT_PATH") {
            let path = PathBuf::from(path);
            if !path.join("Cargo.toml").exists() {
                return Err(anyhow!(
                    "Shared contract override missing Cargo.toml: {}",
                    path.display()
                ));
            }
            if !path.join("lib.rs").exists() {
                return Err(anyhow!(
                    "Shared contract override missing lib.rs: {}",
                    path.display()
                ));
            }
            return Ok(path);
        }

        // Fast path: already initialized
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
                with_frontend: None,
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

        // Verify creation artifacts
        if !path.join("Cargo.toml").exists() {
            return Err(anyhow!(
                "Shared contract creation failed: missing Cargo.toml"
            ));
        }
        if !path.join("lib.rs").exists() {
            return Err(anyhow!("Shared contract creation failed: missing lib.rs"));
        }

        // Determine release mode from env var
        let release = match std::env::var("POP_E2E_SHARED_CONTRACT_RELEASE") {
            Ok(val) if val == "0" => Some(false),
            Ok(val) if val == "1" => Some(true),
            _ => None,
        };

        // Build unless skipped
        if std::env::var("POP_E2E_SHARED_CONTRACT_SKIP_BUILD")
            .ok()
            .as_deref()
            != Some("1")
        {
            let build_result = build_contract(
                &executor,
                BuildContractParams {
                    path: path.display().to_string(),
                    release,
                },
            )
            .context("Failed to build shared contract")?;

            if is_error(&build_result) {
                return Err(anyhow!(
                    "Shared contract build failed: {}",
                    text(&build_result)?
                ));
            }

            // Verify build artifacts
            if !path.join("target/ink").exists() {
                return Err(anyhow!(
                    "Shared contract build failed: missing build artifacts in target/ink"
                ));
            }
        }

        SHARED_CONTRACT_DIR.set(tempdir).ok();
        Ok(path)
    }

    pub(crate) struct Contract {
        pub(crate) path: PathBuf,
        address: Option<String>,
    }

    impl Contract {
        /// Create, build, or reuse the shared contract for testing.
        pub(crate) fn create_build_or_use() -> Result<Self> {
            Ok(Contract {
                path: shared_contract_path()?,
                address: None,
            })
        }

        /// Deploy to shared ink-node.
        pub(crate) fn deploy(&mut self, url: &str, constructor: &str, args: &str) -> Result<()> {
            let executor = PopExecutor::new();

            let result = deploy_contract(
                &executor,
                DeployContractParams {
                    path: self.path.to_string_lossy().to_string(),
                    constructor: Some(constructor.to_string()),
                    args: Some(args.to_string()),
                    value: None,
                    execute: Some(true),
                    url: Some(url.to_string()),
                },
                None,
            )
            .context("Failed to deploy contract")?;

            if is_error(&result) {
                return Err(anyhow!("Deployment failed: {}", text(&result)?));
            }

            let output = text(&result)?;
            self.address = Some(parse_contract_address(&output).ok_or_else(|| {
                anyhow!("Failed to parse contract address from output: {}", output)
            })?);

            Ok(())
        }

        /// Get deployed address.
        pub(crate) fn address(&self) -> &str {
            self.address.as_ref().expect("Contract not deployed")
        }
    }

    // fn shared_chain_path() -> Result<PathBuf> {
    //     // Check for override path first
    //     if let Ok(path) = std::env::var("POP_E2E_SHARED_CHAIN_PATH") {
    //         let path = PathBuf::from(path);
    //         if !path.join("Cargo.toml").exists() {
    //             return Err(anyhow!(
    //                 "Shared chain override missing Cargo.toml: {}",
    //                 path.display()
    //             ));
    //         }
    //         if !path.join("runtime").exists() {
    //             return Err(anyhow!(
    //                 "Shared chain override missing runtime directory: {}",
    //                 path.display()
    //             ));
    //         }
    //         return Ok(path);
    //     }

    //     // Fast path: already initialized
    //     if let Some(dir) = SHARED_CHAIN_DIR.get() {
    //         return Ok(dir.path().join(SHARED_CHAIN_NAME));
    //     }

    //     // Double-checked locking for initialization
    //     let _guard = SHARED_CHAIN_INIT.lock().unwrap();
    //     if let Some(dir) = SHARED_CHAIN_DIR.get() {
    //         return Ok(dir.path().join(SHARED_CHAIN_NAME));
    //     }

    //     // Create temp dir and chain project
    //     let tempdir = TempDir::new().context("Failed to create temp dir")?;
    //     let executor = PopExecutor::with_cwd(tempdir.path().to_path_buf());

    //     let create_result = create_chain(
    //         &executor,
    //         CreateChainParams {
    //             name: SHARED_CHAIN_NAME.to_string(),
    //             provider: "pop".to_string(),
    //             template: "r0gue-io/base-parachain".to_string(),
    //             symbol: None,
    //             decimals: None,
    //         },
    //     )
    //     .context("Failed to create shared chain")?;

    //     if is_error(&create_result) {
    //         return Err(anyhow!(
    //             "Shared chain creation failed: {}",
    //             text(&create_result)?
    //         ));
    //     }

    //     let path = tempdir.path().join(SHARED_CHAIN_NAME);

    //     // Verify creation artifacts
    //     if !path.join("Cargo.toml").exists() {
    //         return Err(anyhow!("Shared chain creation failed: missing Cargo.toml"));
    //     }
    //     if !path.join("runtime").exists() {
    //         return Err(anyhow!(
    //             "Shared chain creation failed: missing runtime directory"
    //         ));
    //     }

    //     // Determine release mode from env var
    //     let release = match std::env::var("POP_E2E_SHARED_CHAIN_RELEASE") {
    //         Ok(val) if val == "0" => Some(false),
    //         Ok(val) if val == "1" => Some(true),
    //         _ => None,
    //     };

    //     // Build unless skipped
    //     if std::env::var("POP_E2E_SHARED_CHAIN_SKIP_BUILD")
    //         .ok()
    //         .as_deref()
    //         != Some("1")
    //     {
    //         let build_result = build_chain(
    //             &executor,
    //             BuildChainParams {
    //                 path: path.display().to_string(),
    //                 release,
    //             },
    //         )
    //         .context("Failed to build shared chain")?;

    //         if is_error(&build_result) {
    //             return Err(anyhow!(
    //                 "Shared chain build failed: {}",
    //                 text(&build_result)?
    //             ));
    //         }

    //         // Verify build artifacts
    //         let release_enabled = release.unwrap_or(true);
    //         let build_dir = if release_enabled {
    //             "target/release"
    //         } else {
    //             "target/debug"
    //         };
    //         if !path.join(build_dir).exists() {
    //             return Err(anyhow!(
    //                 "Shared chain build failed: missing build artifacts in {}",
    //                 build_dir
    //             ));
    //         }
    //     }

    //     SHARED_CHAIN_DIR.set(tempdir).ok();
    //     Ok(path)
    // }

    // pub(crate) struct Chain {
    //     pub(crate) path: PathBuf,
    // }

    // impl Chain {
    //     /// Create, build, or reuse the shared chain project for testing.
    //     pub(crate) fn create_build_or_use() -> Result<Self> {
    //         Ok(Chain {
    //             path: shared_chain_path()?,
    //         })
    //     }
    // }

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

    pub(crate) fn is_port_in_use(port: u16) -> bool {
        is_port_open("127.0.0.1", port)
    }

    pub(crate) fn wait_for_port_closed(port: u16, timeout: Duration) -> Result<()> {
        if wait_until(timeout, || !is_port_open("127.0.0.1", port)) {
            return Ok(());
        }
        Err(anyhow!("Timed out waiting for port {port} to close"))
    }

    fn is_port_open(host: &str, port: u16) -> bool {
        match (host, port).to_socket_addrs() {
            Ok(mut addrs) => addrs
                .any(|addr| TcpStream::connect_timeout(&addr, Duration::from_millis(200)).is_ok()),
            Err(_) => false,
        }
    }

    pub(crate) fn ws_port_from_url(url: &str) -> Result<u16> {
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

    pub(crate) fn parse_pids(output: &str) -> Result<Vec<u32>> {
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

    pub(crate) struct SharedNodeGuard {
        _private: (), // prevent construction outside this module
    }

    impl SharedNodeGuard {
        fn new() -> Self {
            Self { _private: () }
        }
    }

    impl Drop for SharedNodeGuard {
        fn drop(&mut self) {
            let mut node = NODE.lock().unwrap();
            if let Some(ref info) = *node {
                let prev = info.users.fetch_sub(1, Ordering::SeqCst);
                if prev == 1 {
                    // Last user - kill the node
                    for pid in &info.pids {
                        let _ = Command::new("kill").arg("-9").arg(pid.to_string()).status();
                    }
                    // Clear node info so next ensure() restarts
                    *node = None;
                }
            }
        }
    }
}

mod tools;
