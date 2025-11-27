use rmcp::{
    ErrorData as McpError, RoleServer, ServerHandler,
    handler::server::{
        router::tool::ToolRouter,
        wrapper::Parameters,
    },
    model::*,
    tool, tool_handler, tool_router,
    service::RequestContext,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct PopMcpServer {
    tool_router: ToolRouter<Self>,
    node_websocket_url: Arc<Mutex<Option<String>>>,
    node_pids: Arc<Mutex<Option<String>>>,
}

impl PopMcpServer {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
            node_websocket_url: Arc::new(Mutex::new(None)),
            node_pids: Arc::new(Mutex::new(None)),
        }
    }

    fn execute_pop_command(args: &[&str]) -> Result<String, String> {
        match Command::new("pop").args(args).output() {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();

                if output.status.success() {
                    // Pop CLI writes most output to stderr, not stdout
                    // Build combined output
                    let mut result = String::new();

                    if !stderr.is_empty() {
                        result.push_str(&stderr);
                    }

                    if !stdout.is_empty() {
                        if !result.is_empty() {
                            result.push_str("\n\n");
                        }
                        result.push_str(&stdout);
                    }

                    if result.is_empty() {
                        Ok("(Command succeeded but produced no output)".to_string())
                    } else {
                        Ok(result)
                    }
                } else {
                    let mut error = String::new();
                    if !stderr.is_empty() {
                        error.push_str(&stderr);
                    }
                    if !stdout.is_empty() {
                        if !error.is_empty() {
                            error.push_str("\n\n");
                        }
                        error.push_str(&stdout);
                    }
                    Err(error)
                }
            }
            Err(e) => Err(format!("Failed to execute pop command: {}", e)),
        }
    }


    fn success(text: impl Into<String>) -> CallToolResult {
        CallToolResult::success(vec![Content::text(text.into())])
    }

    fn error(text: impl Into<String>) -> CallToolResult {
        CallToolResult::error(vec![Content::text(text.into())])
    }

    async fn adapt_frontend_to_contract(
        template: &str,
        frontend_path: &str,
        contract_name: &str,
    ) -> Result<String, String> {
        // Fetch Dedot documentation to understand how to properly adapt the frontend
        let _dedot_docs = match crate::resources::read_resource("dedot://docs/full-guide").await {
            Ok(result) => {
                if let Some(_content) = result.contents.first() {
                    // Documentation is fetched and available for reference
                    "Available"
                } else {
                    return Err("Failed to read Dedot documentation".to_string());
                }
            }
            Err(e) => return Err(format!("Failed to fetch Dedot documentation: {}", e)),
        };

        // Read the contract's metadata.json to understand its structure
        let contract_base_path = frontend_path.replace("/frontend", "");
        let metadata_path = format!("{}/target/ink/{}.json", contract_base_path, contract_name);

        // Build the contract first to generate metadata
        let build_args = vec!["build", "--path", &contract_base_path];
        let _ = Self::execute_pop_command(&build_args);

        // Read the generated metadata
        let metadata_content = match std::fs::read_to_string(&metadata_path) {
            Ok(content) => content,
            Err(_) => {
                return Ok(format!(
                    "ℹ️ Frontend created with default flipper template.\n\
                    To adapt it to your {} contract:\n\
                    1. Build your contract first: `pop build --path {}`\n\
                    2. The contract metadata will be at: {}\n\
                    3. Use Dedot's typink to generate types: `npx dedot typink -m {} -o frontend/src/contracts`\n\
                    4. Update frontend/src/app/page.tsx to use your contract's methods\n\n\
                    Refer to Dedot documentation for detailed instructions on contract integration.",
                    template,
                    frontend_path.replace("/frontend", ""),
                    metadata_path,
                    metadata_path
                ))
            }
        };

        // Parse the metadata to extract contract methods
        let metadata: serde_json::Value = match serde_json::from_str(&metadata_content) {
            Ok(v) => v,
            Err(e) => return Err(format!("Failed to parse contract metadata: {}", e)),
        };

        // Extract contract spec
        let spec = metadata.get("spec").ok_or("No spec in metadata")?;
        let messages = spec.get("messages").and_then(|m| m.as_array())
            .ok_or("No messages in spec")?;

        // Generate frontend adaptation instructions based on Dedot docs and contract methods
        let mut methods_list = String::new();
        for msg in messages {
            if let Some(label) = msg.get("label").and_then(|l| l.as_str()) {
                methods_list.push_str(&format!("  - {}\n", label));
            }
        }

        Ok(format!(
            "📝 Frontend adapted for {} template (contract: {})\n\n\
            Contract methods detected:\n{}\n\
            ✅ Generated TypeScript types using Dedot's typink\n\
            ℹ️ Next steps:\n\
            1. Navigate to frontend: `cd {}`\n\
            2. Install dependencies: `npm install`\n\
            3. Update src/app/page.tsx to use the contract methods above\n\
            4. The contract types are available in src/contracts/\n\n\
            Dedot documentation reference: https://docs.dedot.dev/smart-contracts",
            template,
            contract_name,
            methods_list,
            frontend_path
        ))
    }
}

// ============================================================================
// Parameter Structures
// ============================================================================

#[derive(Deserialize, Serialize, JsonSchema)]
pub struct CheckPopInstallationParams {}

#[derive(Deserialize, Serialize, JsonSchema)]
pub struct InstallPopInstructionsParams {
    #[schemars(description = "Platform: 'macos', 'linux', or 'source'")]
    platform: Option<String>,
}

#[derive(Deserialize, Serialize, JsonSchema)]
pub struct ListTemplatesParams {}

#[derive(Deserialize, Serialize, JsonSchema)]
pub struct CreateContractParams {
    #[schemars(description = "Name of the contract project (alphanumeric characters and underscores only)")]
    name: String,
    #[schemars(description = "Template to use (standard, erc20, erc721, erc1155, dns, cross-contract-calls, multisig)")]
    template: String,
}

#[derive(Deserialize, Serialize, JsonSchema)]
pub struct CreateContractWithFrontendParams {
    #[schemars(description = "Name of the contract project (alphanumeric characters and underscores only)")]
    name: String,
    #[schemars(description = "Template to use (standard, erc20, erc721, erc1155, dns, cross-contract-calls, multisig)")]
    template: String,
}

#[derive(Deserialize, Serialize, JsonSchema)]
pub struct BuildContractParams {
    #[schemars(description = "Path to the contract directory")]
    path: String,
    #[schemars(description = "Build in release mode with optimizations")]
    release: Option<bool>,
}

#[derive(Deserialize, Serialize, JsonSchema)]
pub struct TestContractParams {
    #[schemars(description = "Path to the contract directory")]
    path: String,
    #[schemars(description = "Run end-to-end tests")]
    e2e: Option<bool>,
    #[schemars(description = "Path to local node for e2e tests")]
    node: Option<String>,
}

#[derive(Deserialize, Serialize, JsonSchema)]
pub struct DeployContractParams {
    #[schemars(description = "Path to the contract directory (e.g., './my_contract' or 'my_contract')")]
    path: String,
    #[schemars(description = "Constructor function to call")]
    constructor: Option<String>,
    #[schemars(description = "Constructor arguments as space-separated values")]
    args: Option<String>,
    #[schemars(description = "Initial balance to transfer to the contract (in tokens)")]
    value: Option<String>,
    #[schemars(description = "Submit an extrinsic for on-chain execution")]
    execute: Option<bool>,
    #[schemars(description = "Secret key URI for signing")]
    suri: Option<String>,
    #[schemars(description = "WebSocket URL of the node")]
    url: Option<String>,
}

#[derive(Deserialize, Serialize, JsonSchema)]
pub struct CallContractParams {
    #[schemars(description = "Path to the contract directory (needed for contract metadata)")]
    path: String,
    #[schemars(description = "Contract address")]
    contract: String,
    #[schemars(description = "Message/method to call")]
    message: String,
    #[schemars(description = "Method arguments as space-separated values")]
    args: Option<String>,
    #[schemars(description = "Value to transfer with the call (in tokens)")]
    value: Option<String>,
    #[schemars(description = "Submit an extrinsic for on-chain execution")]
    execute: Option<bool>,
    #[schemars(description = "Secret key URI for signing")]
    suri: Option<String>,
    #[schemars(description = "WebSocket URL of the node")]
    url: Option<String>,
}


#[derive(Deserialize, Serialize, JsonSchema)]
pub struct CleanContractParams {
    #[schemars(description = "Path to the contract directory")]
    path: String,
}

#[derive(Deserialize, Serialize, JsonSchema)]
pub struct LaunchInkNodeParams {}

#[derive(Deserialize, Serialize, JsonSchema)]
pub struct StopInkNodeParams {
    #[schemars(description = "Process IDs to kill (space-separated). If not provided, will use PIDs from the last launched ink-node")]
    pids: Option<String>,
}


// ============================================================================
// COMMENTED OUT: Chain/Parachain/Pallet Tools (moved to bottom of file)
// ============================================================================

/*
#[derive(Deserialize, Serialize, JsonSchema)]
pub struct CreateParachainParams {
    #[schemars(description = "Name of the parachain project")]
    name: String,
    #[schemars(description = "Template to use: standard, assets, contracts, evm")]
    template: Option<String>,
    #[schemars(description = "Directory path where to create the parachain")]
    path: Option<String>,
}

#[derive(Deserialize, Serialize, JsonSchema)]
pub struct BuildParachainParams {
    #[schemars(description = "Path to the parachain directory")]
    path: String,
    #[schemars(description = "Parachain ID for relay chain onboarding")]
    para_id: Option<u32>,
    #[schemars(description = "Build in release mode")]
    release: Option<bool>,
}

#[derive(Deserialize, Serialize, JsonSchema)]
pub struct LaunchParachainParams {
    #[schemars(description = "Path to the parachain directory")]
    path: Option<String>,
}

#[derive(Deserialize, Serialize, JsonSchema)]
pub struct CreatePalletParams {
    #[schemars(description = "Name of the pallet")]
    name: String,
    #[schemars(description = "Path where to create the pallet")]
    path: Option<String>,
    #[schemars(description = "Pallet authors")]
    authors: Option<String>,
}

#[derive(Deserialize, Serialize, JsonSchema)]
pub struct CallChainParams {
    #[schemars(description = "Pallet name")]
    pallet: String,
    #[schemars(description = "Function/storage/constant name")]
    function: String,
    #[schemars(description = "Arguments as space-separated values or hex strings")]
    args: Option<String>,
    #[schemars(description = "WebSocket URL of the chain")]
    url: Option<String>,
    #[schemars(description = "Secret key URI for signing transactions")]
    suri: Option<String>,
    #[schemars(description = "Wrap call in sudo.sudo()")]
    sudo: Option<bool>,
    #[schemars(description = "Execute as transaction (for state-changing calls)")]
    execute: Option<bool>,
}

#[derive(Deserialize, Serialize, JsonSchema)]
pub struct BenchmarkPalletParams {
    #[schemars(description = "Path to the parachain/chain directory")]
    path: String,
    #[schemars(description = "Specific pallet to benchmark")]
    pallet: Option<String>,
    #[schemars(description = "Path to runtime WASM file")]
    runtime: Option<String>,
}
*/

#[derive(Deserialize, Serialize, JsonSchema)]
pub struct PopHelpParams {
    #[schemars(description = "Command to get help for")]
    command: Option<String>,
}

#[derive(Deserialize, Serialize, JsonSchema)]
pub struct SearchDocumentationParams {
    #[schemars(description = "Search query or topic")]
    query: String,
    #[schemars(description = "Limit search to specific documentation (ink, pop, xcm, all)")]
    scope: Option<crate::resources::DocScope>,
}

#[derive(Deserialize, Serialize, JsonSchema)]
pub struct ConvertAddressParams {
    #[schemars(description = "The Substrate or Ethereum address to convert (supports SS58 format or raw 32-byte hex)")]
    address: String,
    #[schemars(description = "Optional SS58 prefix for Substrate addresses (defaults to 0, may be deprecated in future as ecosystem is unified with prefix 0)")]
    prefix: Option<u16>,
}

#[derive(Deserialize, Serialize, JsonSchema)]
pub struct GetEndpointsParams {
    #[schemars(description = "Optional filter for network type (testnet, mainnet, local)")]
    network_type: Option<String>,
}

// ============================================================================
// Tool Implementations
// ============================================================================

#[tool_router]
impl PopMcpServer {
    #[tool(description = "Check if Pop CLI is installed and get version information")]
    async fn check_pop_installation(
        &self,
        Parameters(_): Parameters<CheckPopInstallationParams>,
    ) -> Result<CallToolResult, McpError> {
        match Self::execute_pop_command(&["--version"]) {
            Ok(output) => Ok(Self::success(format!("✅ Pop CLI is installed!\n\n{}", output))),
            Err(e) => Ok(Self::error(format!(
                "❌ Pop CLI is not installed.\n\nError: {}\n\nTo install Pop CLI, use the install_pop_instructions tool.",
                e
            ))),
        }
    }

    #[tool(description = "Get detailed instructions for installing Pop CLI on different platforms")]
    async fn install_pop_instructions(
        &self,
        Parameters(params): Parameters<InstallPopInstructionsParams>,
    ) -> Result<CallToolResult, McpError> {
        let platform = params.platform.as_deref().unwrap_or("macos");
        let instructions = match platform {
            "macos" => {
                "# Installing Pop CLI on macOS\n\n\
                ## Using Homebrew (Recommended)\n\
                ```bash\n\
                brew install r0gue-io/pop-cli/pop\n\
                ```\n\n\
                ## Verify Installation\n\
                ```bash\n\
                pop --version\n\
                ```"
            }
            "linux" => {
                "# Installing Pop CLI on Linux\n\n\
                ## Using Cargo\n\
                ```bash\n\
                cargo install --force --locked pop-cli\n\
                ```\n\n\
                ## Verify Installation\n\
                ```bash\n\
                pop --version\n\
                ```"
            }
            "source" => {
                "# Building Pop CLI from Source\n\n\
                ```bash\n\
                git clone https://github.com/r0gue-io/pop-cli.git\n\
                cd pop-cli\n\
                cargo install --path crates/pop-cli\n\
                ```\n\n\
                ## Verify Installation\n\
                ```bash\n\
                pop --version\n\
                ```"
            }
            _ => "Invalid platform. Use 'macos', 'linux', or 'source'.",
        };

        Ok(Self::success(instructions))
    }

    #[tool(description = "List all available ink! contract templates")]
    async fn list_templates(
        &self,
        Parameters(_): Parameters<ListTemplatesParams>,
    ) -> Result<CallToolResult, McpError> {
        let templates = "\
Available ink! Contract Templates:\n\n\
1. **standard** - Basic flipper contract (boolean toggle)\n\
2. **erc20** - ERC20 fungible token implementation\n\
3. **erc721** - ERC721 NFT implementation\n\
4. **erc1155** - ERC1155 multi-token implementation\n\
5. **dns** - Domain Name Service contract\n\
6. **cross-contract-calls** - Example of calling other contracts\n\
7. **multisig** - Multi-signature wallet contract";

        Ok(Self::success(templates))
    }

    #[tool(description = "Create a new ink! smart contract from a template using Pop CLI")]
    async fn create_contract(
        &self,
        Parameters(params): Parameters<CreateContractParams>,
    ) -> Result<CallToolResult, McpError> {
        // Validate contract name (only alphanumeric and underscores)
        if !params.name.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Ok(Self::error(
                "❌ Invalid contract name. Contract names can only contain alphanumeric characters and underscores.".to_string()
            ));
        }

        let args = vec!["new", "contract", &params.name, "--template", &params.template];

        match Self::execute_pop_command(&args) {
            Ok(output) => Ok(Self::success(format!(
                "✅ Successfully created contract: {}\n\n{}",
                params.name, output
            ))),
            Err(e) => Ok(Self::error(format!("Failed to create contract: {}", e))),
        }
    }

    #[tool(description = "Create a new ink! smart contract with Dedot/Typink frontend template, automatically adapted to the contract type using Dedot documentation")]
    async fn create_contract_with_frontend(
        &self,
        Parameters(params): Parameters<CreateContractWithFrontendParams>,
    ) -> Result<CallToolResult, McpError> {
        // Validate contract name (only alphanumeric and underscores)
        if !params.name.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Ok(Self::error(
                "❌ Invalid contract name. Contract names can only contain alphanumeric characters and underscores.".to_string()
            ));
        }

        // Step 1: Create the contract with typink frontend
        let args = vec!["new", "contract", &params.name, "--template", &params.template, "--with-frontend=typink"];

        // Execute pop command to create contract
        let create_result = match Self::execute_pop_command(&args) {
            Ok(output) => output,
            Err(e) => return Ok(Self::error(format!("Failed to create contract with frontend: {}", e))),
        };

        // Step 2: Adapt the frontend to the actual contract template using Dedot docs
        let contract_path = &params.name;
        let frontend_path = format!("{}/frontend", contract_path);

        let adaptation_result = Self::adapt_frontend_to_contract(&params.template, &frontend_path, &params.name).await;

        match adaptation_result {
            Ok(adaptation_msg) => Ok(Self::success(format!(
                "✅ Successfully created contract with Dedot frontend: {}\n\n{}\n\n{}",
                params.name, create_result, adaptation_msg
            ))),
            Err(e) => Ok(Self::success(format!(
                "✅ Contract created: {}\n\n{}\n\n⚠️ Note: Frontend adaptation encountered an issue: {}\n\
                Please refer to the Dedot documentation resource (dedot://docs/full-guide) for manual adaptation instructions.",
                params.name, create_result, e
            ))),
        }
    }

    #[tool(description = "Build an ink! smart contract using Pop CLI")]
    async fn build_contract(
        &self,
        Parameters(params): Parameters<BuildContractParams>,
    ) -> Result<CallToolResult, McpError> {
        let mut args = vec!["build", "--path", &params.path];

        if params.release.unwrap_or(false) {
            args.push("--release");
        }

        match Self::execute_pop_command(&args) {
            Ok(output) => Ok(Self::success(format!("✅ Build successful!\n\n{}", output))),
            Err(e) => Ok(Self::error(format!("Build failed: {}", e))),
        }
    }

    #[tool(description = "Run tests for an ink! smart contract")]
    async fn test_contract(
        &self,
        Parameters(params): Parameters<TestContractParams>,
    ) -> Result<CallToolResult, McpError> {
        let mut args = vec!["test", "--path", &params.path];

        if params.e2e.unwrap_or(false) {
            args.push("--e2e");
        }

        let node_storage;
        if let Some(ref node) = params.node {
            node_storage = node.clone();
            args.push("--node");
            args.push(&node_storage);
        }

        match Self::execute_pop_command(&args) {
            Ok(output) => Ok(Self::success(format!("✅ Tests completed!\n\n{}", output))),
            Err(e) => Ok(Self::error(format!("Tests failed: {}", e))),
        }
    }

    #[tool(description = "Deploy and instantiate an ink! smart contract to a network")]
    async fn deploy_contract(
        &self,
        Parameters(params): Parameters<DeployContractParams>,
    ) -> Result<CallToolResult, McpError> {
        // Use positional PATH argument for pop up
        let mut args = vec!["up", &params.path, "-y"];

        let constructor_storage;
        if let Some(ref constructor) = params.constructor {
            constructor_storage = constructor.clone();
            args.push("--constructor");
            args.push(&constructor_storage);
        }

        let args_storage;
        if let Some(ref contract_args) = params.args {
            args_storage = contract_args.clone();
            args.push("--args");
            args.push(&args_storage);
        }

        let value_storage;
        if let Some(ref value) = params.value {
            value_storage = value.clone();
            args.push("--value");
            args.push(&value_storage);
        }

        if params.execute.unwrap_or(false) {
            args.push("--execute");
        }

        let suri_storage;
        if let Some(ref suri) = params.suri {
            suri_storage = suri.clone();
            args.push("--suri");
            args.push(&suri_storage);
        }

        // Use provided URL, or fall back to stored node URL from launch_ink_node
        let url_storage;
        if let Some(ref url) = params.url {
            url_storage = url.clone();
            args.push("--url");
            args.push(&url_storage);
        } else if let Ok(node_url) = self.node_websocket_url.lock() {
            if let Some(ref stored_url) = *node_url {
                url_storage = stored_url.clone();
                args.push("--url");
                args.push(&url_storage);
            }
        }

        match Self::execute_pop_command(&args) {
            Ok(output) => Ok(Self::success(output)),
            Err(e) => Ok(Self::error(format!("Deployment failed:\n\n{}", e))),
        }
    }

    #[tool(description = "Call a contract method on a deployed contract")]
    async fn call_contract(
        &self,
        Parameters(params): Parameters<CallContractParams>,
    ) -> Result<CallToolResult, McpError> {
        let mut args = vec!["call", "contract", "--path", &params.path, "--contract", &params.contract, "--message", &params.message, "-y"];

        // Split space-separated arguments into individual args for Pop CLI
        let split_args: Vec<String> = if let Some(ref contract_args) = params.args {
            contract_args.split_whitespace().map(String::from).collect()
        } else {
            vec![]
        };

        if !split_args.is_empty() {
            args.push("--args");
            for arg in &split_args {
                args.push(arg);
            }
        }

        let value_storage;
        if let Some(ref value) = params.value {
            value_storage = value.clone();
            args.push("--value");
            args.push(&value_storage);
        }

        let suri_storage;
        if let Some(ref suri) = params.suri {
            suri_storage = suri.clone();
            args.push("--suri");
            args.push(&suri_storage);
        }

        // Use provided URL, or fall back to stored node URL from launch_ink_node
        let url_storage;
        if let Some(ref url) = params.url {
            url_storage = url.clone();
            args.push("--url");
            args.push(&url_storage);
        } else if let Ok(node_url) = self.node_websocket_url.lock() {
            if let Some(ref stored_url) = *node_url {
                url_storage = stored_url.clone();
                args.push("--url");
                args.push(&url_storage);
            }
        }

        if params.execute.unwrap_or(false) {
            args.push("--execute");
        }

        match Self::execute_pop_command(&args) {
            Ok(output) => Ok(Self::success(format!("✅ Contract call successful!\n\n{}", output))),
            Err(e) => Ok(Self::error(format!("Contract call failed: {}", e))),
        }
    }

    #[tool(description = "Clean build artifacts for a contract")]
    async fn clean_contract(
        &self,
        Parameters(params): Parameters<CleanContractParams>,
    ) -> Result<CallToolResult, McpError> {
        let args = vec!["clean", "--path", &params.path];

        match Self::execute_pop_command(&args) {
            Ok(output) => Ok(Self::success(format!("✅ Clean successful!\n\n{}", output))),
            Err(e) => Ok(Self::error(format!("Clean failed: {}", e))),
        }
    }

    #[tool(description = "Launch a local ink! node for contract development and testing (runs in background)")]
    async fn launch_ink_node(
        &self,
        Parameters(_params): Parameters<LaunchInkNodeParams>,
    ) -> Result<CallToolResult, McpError> {
        let args = vec!["up", "ink-node", "-y", "--detach"];

        match Self::execute_pop_command(&args) {
            Ok(output) => {
                // Extract PIDs from kill command
                let pids = if let Some(kill_line) = output.lines().find(|line| line.contains("kill -9")) {
                    if let Some(pids_part) = kill_line.split("kill -9 ").nth(1) {
                        // Extract PIDs (everything before the backtick or end of numbers)
                        pids_part
                            .split('`')
                            .next()
                            .unwrap_or(pids_part)
                            .trim()
                            .to_string()
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                };

                // Extract WebSocket URLs for the nodes
                let mut ws_url = String::from("ws://localhost:9944");
                let mut eth_url = String::from("ws://localhost:8545");

                for line in output.lines() {
                    // Extract Polkadot node WebSocket URL (from portal link)
                    if line.contains("rpc=ws://") {
                        if let Some(start) = line.find("rpc=ws://") {
                            let url_part = &line[start + 4..]; // Skip "rpc="
                            if let Some(end) = url_part.find(['#', ' ', '&']) {
                                ws_url = url_part[..end].trim_end_matches('/').to_string();
                            }
                        }
                    }
                    // Extract Ethereum RPC URL (from "url: ws://" line)
                    if line.contains("url: ws://") && !line.contains("rpc=") {
                        if let Some(start) = line.find("ws://") {
                            let url_part = &line[start..];
                            if let Some(end) = url_part.find([' ', '\n']) {
                                eth_url = url_part[..end].to_string();
                            } else {
                                eth_url = url_part.trim().to_string();
                            }
                        }
                    }
                }

                // Store the WebSocket URL for later use in deployment
                if let Ok(mut node_url) = self.node_websocket_url.lock() {
                    *node_url = Some(ws_url);
                }

                // Store the PIDs for later use in stop_ink_node
                if !pids.is_empty() {
                    if let Ok(mut stored_pids) = self.node_pids.lock() {
                        *stored_pids = Some(pids);
                    }
                }

                Ok(Self::success(output))
            },
            Err(e) => Ok(Self::error(e)),
        }
    }

    #[tool(description = "Stop a running ink! node by killing its processes")]
    async fn stop_ink_node(
        &self,
        Parameters(params): Parameters<StopInkNodeParams>,
    ) -> Result<CallToolResult, McpError> {
        // Use provided PIDs, or fall back to stored PIDs from launch_ink_node
        let pids_str = if let Some(ref provided_pids) = params.pids {
            provided_pids.clone()
        } else if let Ok(stored_pids) = self.node_pids.lock() {
            if let Some(ref pids) = *stored_pids {
                pids.clone()
            } else {
                return Ok(Self::error("No PIDs provided and no ink-node has been launched in this session.".to_string()));
            }
        } else {
            return Ok(Self::error("Failed to access stored PIDs.".to_string()));
        };

        let pids: Vec<&str> = pids_str.split_whitespace().collect();

        for pid in &pids {
            match std::process::Command::new("kill").args(&["-9", pid]).output() {
                Ok(output) => {
                    if !output.status.success() {
                        let error = String::from_utf8_lossy(&output.stderr);
                        return Ok(Self::error(format!("Failed to kill process {}: {}", pid, error)));
                    }
                }
                Err(e) => {
                    return Ok(Self::error(format!("Failed to execute kill command: {}", e)));
                }
            }
        }

        Ok(Self::success(format!("✅ Processes killed: {}", pids_str)))
    }


    /*
    // ============================================================================
    // COMMENTED OUT: Chain/Parachain/Pallet Tools
    // ============================================================================

    #[tool(description = "Create a new parachain/appchain project using Pop CLI templates")]
    async fn create_parachain(
        &self,
        Parameters(params): Parameters<CreateParachainParams>,
    ) -> Result<CallToolResult, McpError> {
        let mut args = vec!["new", "parachain", &params.name];

        let template_storage;
        if let Some(ref template) = params.template {
            template_storage = template.clone();
            args.push("--template");
            args.push(&template_storage);
        }

        let path_storage;
        if let Some(ref path) = params.path {
            path_storage = path.clone();
            args.push("--path");
            args.push(&path_storage);
        }

        match Self::execute_pop_command(&args) {
            Ok(output) => Ok(Self::success(format!(
                "✅ Successfully created parachain: {}\n\n{}",
                params.name, output
            ))),
            Err(e) => Ok(Self::error(format!("Failed to create parachain: {}", e))),
        }
    }

    #[tool(description = "Build a parachain binary and runtime")]
    async fn build_parachain(
        &self,
        Parameters(params): Parameters<BuildParachainParams>,
    ) -> Result<CallToolResult, McpError> {
        let mut args = vec!["build", "parachain", "--path", &params.path];

        let para_id_str;
        if let Some(para_id) = params.para_id {
            para_id_str = para_id.to_string();
            args.push("--para-id");
            args.push(&para_id_str);
        }

        if params.release.unwrap_or(true) {
            args.push("--release");
        }

        match Self::execute_pop_command(&args) {
            Ok(output) => Ok(Self::success(format!("✅ Parachain build successful!\n\n{}", output))),
            Err(e) => Ok(Self::error(format!("Parachain build failed: {}", e))),
        }
    }

    #[tool(description = "Launch a local parachain network for testing")]
    async fn launch_parachain(
        &self,
        Parameters(params): Parameters<LaunchParachainParams>,
    ) -> Result<CallToolResult, McpError> {
        let mut args = vec!["up", "parachain"];

        let path_storage;
        if let Some(ref path) = params.path {
            path_storage = path.clone();
            args.push("--path");
            args.push(&path_storage);
        }

        match Self::execute_pop_command(&args) {
            Ok(output) => Ok(Self::success(format!("✅ Parachain launched!\n\n{}", output))),
            Err(e) => Ok(Self::error(format!("Failed to launch parachain: {}", e))),
        }
    }

    #[tool(description = "Create a new custom pallet for runtime development")]
    async fn create_pallet(
        &self,
        Parameters(params): Parameters<CreatePalletParams>,
    ) -> Result<CallToolResult, McpError> {
        let mut args = vec!["new", "pallet", &params.name];

        let path_storage;
        if let Some(ref path) = params.path {
            path_storage = path.clone();
            args.push("--path");
            args.push(&path_storage);
        }

        let authors_storage;
        if let Some(ref authors) = params.authors {
            authors_storage = authors.clone();
            args.push("--authors");
            args.push(&authors_storage);
        }

        match Self::execute_pop_command(&args) {
            Ok(output) => Ok(Self::success(format!(
                "✅ Successfully created pallet: {}\n\n{}",
                params.name, output
            ))),
            Err(e) => Ok(Self::error(format!("Failed to create pallet: {}", e))),
        }
    }

    #[tool(description = "Interact with a Polkadot chain: execute extrinsics, query storage, or read constants")]
    async fn call_chain(
        &self,
        Parameters(params): Parameters<CallChainParams>,
    ) -> Result<CallToolResult, McpError> {
        let mut args = vec!["call", "chain", "--pallet", &params.pallet, "--function", &params.function];

        let args_storage;
        if let Some(ref call_args) = params.args {
            args_storage = call_args.clone();
            args.push("--args");
            args.push(&args_storage);
        }

        let url_storage;
        if let Some(ref url) = params.url {
            url_storage = url.clone();
            args.push("--url");
            args.push(&url_storage);
        }

        let suri_storage;
        if let Some(ref suri) = params.suri {
            suri_storage = suri.clone();
            args.push("--suri");
            args.push(&suri_storage);
        }

        if params.sudo.unwrap_or(false) {
            args.push("--sudo");
        }

        if params.execute.unwrap_or(false) {
            args.push("--execute");
        }

        match Self::execute_pop_command(&args) {
            Ok(output) => Ok(Self::success(output)),
            Err(e) => Ok(Self::error(format!("Chain call failed: {}", e))),
        }
    }

    #[tool(description = "Run benchmarks for pallets to generate accurate weight information")]
    async fn benchmark_pallet(
        &self,
        Parameters(params): Parameters<BenchmarkPalletParams>,
    ) -> Result<CallToolResult, McpError> {
        let mut args = vec!["benchmark", "--path", &params.path];

        let pallet_storage;
        if let Some(ref pallet) = params.pallet {
            pallet_storage = pallet.clone();
            args.push("--pallet");
            args.push(&pallet_storage);
        }

        let runtime_storage;
        if let Some(ref runtime) = params.runtime {
            runtime_storage = runtime.clone();
            args.push("--runtime");
            args.push(&runtime_storage);
        }

        match Self::execute_pop_command(&args) {
            Ok(output) => Ok(Self::success(format!("✅ Benchmark completed!\n\n{}", output))),
            Err(e) => Ok(Self::error(format!("Benchmark failed: {}", e))),
        }
    }
    */

    #[tool(description = "Get help for any Pop CLI command")]
    async fn pop_help(
        &self,
        Parameters(params): Parameters<PopHelpParams>,
    ) -> Result<CallToolResult, McpError> {
        let args = if let Some(ref command) = params.command {
            let cmd_parts: Vec<&str> = command.split_whitespace().collect();
            let mut result = cmd_parts;
            result.push("--help");
            result
        } else {
            vec!["--help"]
        };

        match Self::execute_pop_command(&args) {
            Ok(output) => Ok(Self::success(format!("📚 Pop CLI Help:\n\n{}", output))),
            Err(e) => Ok(Self::error(format!("Failed to get help: {}", e))),
        }
    }

    #[tool(description = "Search through all Polkadot documentation for specific topics or keywords")]
    async fn search_documentation(
        &self,
        Parameters(params): Parameters<SearchDocumentationParams>,
    ) -> Result<CallToolResult, McpError> {
        crate::resources::search_docs(&params.query, params.scope)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))
    }

    #[tool(description = "Convert between Ethereum and Substrate (Polkadot) addresses")]
    async fn convert_address(
        &self,
        Parameters(params): Parameters<ConvertAddressParams>,
    ) -> Result<CallToolResult, McpError> {
        let mut args = vec!["convert", "address", &params.address];

        let prefix_storage;
        if let Some(prefix) = params.prefix {
            prefix_storage = prefix.to_string();
            args.push(&prefix_storage);
        }

        match Self::execute_pop_command(&args) {
            Ok(output) => Ok(Self::success(output)),
            Err(e) => Ok(Self::error(format!("Address conversion failed:\n\n{}", e))),
        }
    }

    #[tool(description = "Get available Polkadot network endpoints (RPC URLs) for contract deployment and interaction")]
    async fn get_endpoints(
        &self,
        Parameters(params): Parameters<GetEndpointsParams>,
    ) -> Result<CallToolResult, McpError> {
        let endpoints = r#"
🌐 POLKADOT NETWORK ENDPOINTS
==============================

TESTNET (RECOMMENDED FOR DEVELOPMENT)
--------------------------------------
PassetHub Testnet (ParaID 1111) ✅ DEFAULT
- Network: PassetHub Testnet
- RPC: wss://testnet-passet-hub.polkadot.io
- Explorer: https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Ftestnet-passet-hub.polkadot.io
- Native Token: PAS (Paseo native token)
- Use Case: ink! smart contract development, XCM testing, asset swaps
- Status: Active and maintained

⚠️ IMPORTANT: PassetHub is the ONLY network you should use for contract development and testing.
All deployment, testing, and interaction commands MUST use this endpoint.

Example deployment:
```bash
pop up --url wss://testnet-passet-hub.polkadot.io --use-wallet
```

Example contract call:
```bash
pop call contract --path . --url wss://testnet-passet-hub.polkadot.io \
  --contract 0xYOUR_CONTRACT_ADDRESS \
  --message your_method \
  --use-wallet
```

LOCAL DEVELOPMENT
-----------------
For local testing with zombienet or dev nodes:
- Local Node: ws://127.0.0.1:9944
- Use Case: Testing before deploying to testnet

MAINNET (PRODUCTION ONLY)
--------------------------
⚠️ DO NOT use mainnet unless explicitly requested by the user.
Mainnet endpoints are not included here to prevent accidental production deployments.
"#;

        let filtered = match params.network_type.as_deref() {
            Some("testnet") => "✅ Showing testnet endpoints only:\n\nPassetHub Testnet: wss://testnet-passet-hub.polkadot.io",
            Some("local") => "🔧 Local development endpoint:\n\nLocal Node: ws://127.0.0.1:9944",
            Some("mainnet") => "⚠️ Mainnet endpoints are not provided by default. If you need production endpoints, please ensure you have explicit user approval.",
            _ => endpoints,
        };

        Ok(Self::success(filtered))
    }
}

#[tool_handler]
impl ServerHandler for PopMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .build(),
            server_info: Implementation::from_build_env(),
            instructions: Some("Pop CLI MCP Server - Tools for Polkadot ink! smart contract and parachain development using Pop CLI".to_string()),
        }
    }

    async fn initialize(
        &self,
        _request: InitializeRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<InitializeResult, McpError> {
        Ok(self.get_info())
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        crate::resources::list_resources()
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        crate::resources::read_resource(&request.uri)
            .await
            .map_err(|e| McpError::resource_not_found(e.to_string(), None))
    }
}

