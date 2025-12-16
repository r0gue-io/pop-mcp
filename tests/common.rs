use pop_mcp_server::executor::{CommandExecutor, PopExecutor};
use pop_mcp_server::tools::build::contract::{build_contract, BuildContractParams};
use pop_mcp_server::tools::clean::{clean_nodes, CleanNodesParams};
pub use pop_mcp_server::tools::common::content_text;
use pop_mcp_server::tools::common::extract_text;
use pop_mcp_server::tools::install::{check_pop_installation, CheckPopInstallationParams};
use pop_mcp_server::tools::new::contract::{create_contract, CreateContractParams};
use pop_mcp_server::tools::up::chain::{up_ink_node, UpInkNodeParams};
use pop_mcp_server::tools::up::contract::{deploy_contract, DeployContractParams};
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

/// Create a PopExecutor for testing.
/// Panics if Pop CLI is not available.
pub fn pop_executor() -> PopExecutor {
    let executor = PopExecutor::new();
    assert!(
        executor.execute(&["--version"]).is_ok(),
        "Pop CLI is not available. Please install it first."
    );
    executor
}

/// A guard that manages an ink-node's lifecycle.
/// Automatically cleans up the node when dropped.
pub struct InkNode<'a> {
    executor: &'a PopExecutor,
    pub url: String,
}

impl<'a> InkNode<'a> {
    /// Launch an ink-node and return a guard that cleans it up on drop.
    pub fn launch(executor: &'a PopExecutor) -> Result<Self, String> {
        let result = up_ink_node(executor, UpInkNodeParams {})
            .map_err(|e| format!("Failed to launch ink-node: {}", e))?;

        if result.is_error == Some(true) {
            return Err("Failed to launch ink-node".to_string());
        }

        let url = extract_text(&result).ok_or("Failed to extract URL from ink-node output")?;

        // Verify the node is actually running
        if !is_port_in_use(9944) {
            return Err("Port 9944 not in use after launching ink-node".to_string());
        }

        Ok(Self { executor, url })
    }

    /// Get the node URL
    pub fn url(&self) -> &str {
        &self.url
    }
}

impl Drop for InkNode<'_> {
    fn drop(&mut self) {
        let _ = clean_nodes(self.executor, CleanNodesParams {});
    }
}

/// Check if a port is in use using lsof
pub fn is_port_in_use(port: u16) -> bool {
    Command::new("lsof")
        .args(["-i", &format!(":{}", port)])
        .output()
        .map(|o| !o.stdout.is_empty())
        .unwrap_or(false)
}

/// A contract in a temp directory with optional build and deployment state.
///
/// The returned guard keeps the temporary directory alive so the contract
/// artifacts persist for the duration of the test. It temporarily changes
/// the process working directory while creating the contract; prefer
/// `serial` tests or single-threaded runs when using it.
pub struct Contract<'a> {
    pub temp_dir: TempDir,
    pub path: PathBuf,
    pub address: Option<String>,
    ink_node: Option<InkNode<'a>>,
}

impl<'a> Contract<'a> {
    /// Create a new contract from the standard template.
    pub fn new(executor: &PopExecutor, name: &str) -> Result<Self, String> {
        let temp_dir = TempDir::new().map_err(|e| format!("Failed to create temp dir: {}", e))?;
        let original_dir =
            std::env::current_dir().map_err(|e| format!("Failed to get cwd: {}", e))?;
        let _cwd_guard = CwdRestoreGuard::new(&original_dir);
        std::env::set_current_dir(temp_dir.path())
            .map_err(|e| format!("Failed to enter temp dir: {}", e))?;

        let create_params = CreateContractParams {
            name: name.to_string(),
            template: "standard".to_string(),
        };
        let create_result = create_contract(executor, create_params)
            .map_err(|e| format!("Failed to create contract: {}", e))?;
        if create_result.is_error == Some(true) {
            return Err(format!(
                "Contract creation failed: {}",
                content_text(&create_result)
            ));
        }

        let contract_path = temp_dir.path().join(name);

        Ok(Contract {
            temp_dir,
            path: contract_path,
            address: None,
            ink_node: None,
        })
    }

    /// Build the contract.
    pub fn build(&self, executor: &PopExecutor) -> Result<(), String> {
        let build_params = BuildContractParams {
            path: self.path.to_string_lossy().to_string(),
            release: None,
        };
        let build_result = build_contract(executor, build_params)
            .map_err(|e| format!("Failed to build contract: {}", e))?;
        if build_result.is_error == Some(true) {
            return Err(format!(
                "Contract build failed: {}",
                content_text(&build_result)
            ));
        }
        Ok(())
    }

    /// Launch an ink node and deploy the contract.
    /// Sets the contract address on success.
    pub fn deploy(
        &mut self,
        executor: &'a PopExecutor,
        constructor: Option<&str>,
        args: Option<&str>,
    ) -> Result<(), String> {
        // Launch ink node
        let ink_node = InkNode::launch(executor)?;
        let url = ink_node.url().to_string();

        // Deploy the contract
        let deploy_params = DeployContractParams {
            path: self.path.to_string_lossy().to_string(),
            constructor: constructor.map(String::from),
            args: args.map(String::from),
            value: None,
            execute: Some(true),
            suri: Some("//Alice".to_string()),
            url: Some(url),
        };
        let deploy_result = deploy_contract(executor, deploy_params, None)
            .map_err(|e| format!("Failed to deploy contract: {}", e))?;
        if deploy_result.is_error == Some(true) {
            return Err(format!(
                "Contract deployment failed: {}",
                content_text(&deploy_result)
            ));
        }

        // Parse contract address from output
        let output = content_text(&deploy_result);
        let address = parse_contract_address(&output)
            .ok_or_else(|| format!("Failed to parse contract address from output: {}", output))?;

        self.address = Some(address);
        self.ink_node = Some(ink_node);
        Ok(())
    }

    /// Get the node URL if deployed.
    pub fn node_url(&self) -> Option<&str> {
        self.ink_node.as_ref().map(|n| n.url())
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
        if word.starts_with('5') && word.len() >= 47 && word.len() <= 48 {
            if word.chars().all(|c| c.is_alphanumeric()) {
                return Some(word.to_string());
            }
        }
    }

    None
}

/// Restores the working directory on drop to avoid leaving the process in a deleted temp dir.
struct CwdRestoreGuard {
    original_dir: PathBuf,
}

impl CwdRestoreGuard {
    fn new(original_dir: &PathBuf) -> Self {
        Self {
            original_dir: original_dir.clone(),
        }
    }
}

impl Drop for CwdRestoreGuard {
    fn drop(&mut self) {
        let _ = std::env::set_current_dir(&self.original_dir);
    }
}

#[test]
fn pop_is_available() {
    let executor = pop_executor();
    let result = check_pop_installation(&executor, CheckPopInstallationParams {}).unwrap();
    assert_eq!(result.is_error, Some(false));
}
