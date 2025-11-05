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

#[derive(Clone)]
pub struct PopMcpServer {
    tool_router: ToolRouter<Self>,
}

impl PopMcpServer {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    fn execute_pop_command(args: &[&str]) -> Result<String, String> {
        match Command::new("pop").args(args).output() {
            Ok(output) => {
                if output.status.success() {
                    Ok(String::from_utf8_lossy(&output.stdout).to_string())
                } else {
                    Err(String::from_utf8_lossy(&output.stderr).to_string())
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
    #[schemars(description = "Name of the contract project")]
    name: String,
    #[schemars(description = "Template to use (standard, erc20, erc721, erc1155, dns, cross-contract-calls, multisig)")]
    template: String,
    #[schemars(description = "Directory path where to create the contract")]
    path: Option<String>,
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
    #[schemars(description = "Path to the contract directory or .contract bundle")]
    path: String,
    #[schemars(description = "Constructor function to call")]
    constructor: Option<String>,
    #[schemars(description = "Constructor arguments as space-separated values")]
    args: Option<String>,
    #[schemars(description = "Secret key URI for signing")]
    suri: Option<String>,
    #[schemars(description = "WebSocket URL of the node")]
    url: Option<String>,
    #[schemars(description = "Only upload code without instantiating")]
    upload_only: Option<bool>,
    #[schemars(description = "Perform a dry run without submitting")]
    dry_run: Option<bool>,
}

#[derive(Deserialize, Serialize, JsonSchema)]
pub struct CallContractParams {
    #[schemars(description = "Contract address")]
    contract: String,
    #[schemars(description = "Message/method to call")]
    message: String,
    #[schemars(description = "Method arguments as space-separated values")]
    args: Option<String>,
    #[schemars(description = "Secret key URI for signing")]
    suri: Option<String>,
    #[schemars(description = "WebSocket URL of the node")]
    url: Option<String>,
    #[schemars(description = "Perform a dry run without submitting")]
    dry_run: Option<bool>,
}

#[derive(Deserialize, Serialize, JsonSchema)]
pub struct GetContractInfoParams {
    #[schemars(description = "Path to the contract directory or .contract file")]
    path: String,
}

#[derive(Deserialize, Serialize, JsonSchema)]
pub struct CleanContractParams {
    #[schemars(description = "Path to the contract directory")]
    path: String,
}

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
    #[schemars(description = "The Substrate or Ethereum address to convert")]
    address: String,
    #[schemars(description = "The SS58 prefix for Substrate addresses (defaults to 0 for Polkadot)")]
    prefix: Option<u16>,
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
            Err(e) => Ok(Self::success(format!(
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
        let mut args = vec!["new", "contract", &params.name, "--template", &params.template];

        let path_storage;
        if let Some(ref path) = params.path {
            path_storage = path.clone();
            args.push("--path");
            args.push(&path_storage);
        }

        match Self::execute_pop_command(&args) {
            Ok(output) => Ok(Self::success(format!(
                "✅ Successfully created contract: {}\n\n{}",
                params.name, output
            ))),
            Err(e) => Ok(Self::error(format!("Failed to create contract: {}", e))),
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
        let mut args = vec!["up", "--path", &params.path];

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

        let suri_storage;
        if let Some(ref suri) = params.suri {
            suri_storage = suri.clone();
            args.push("--suri");
            args.push(&suri_storage);
        }

        let url_storage;
        if let Some(ref url) = params.url {
            url_storage = url.clone();
            args.push("--url");
            args.push(&url_storage);
        }

        if params.upload_only.unwrap_or(false) {
            args.push("--upload-only");
        }

        if params.dry_run.unwrap_or(false) {
            args.push("--dry-run");
        }

        match Self::execute_pop_command(&args) {
            Ok(output) => Ok(Self::success(format!("✅ Deployment successful!\n\n{}", output))),
            Err(e) => Ok(Self::error(format!("Deployment failed: {}", e))),
        }
    }

    #[tool(description = "Call a contract method on a deployed contract")]
    async fn call_contract(
        &self,
        Parameters(params): Parameters<CallContractParams>,
    ) -> Result<CallToolResult, McpError> {
        let mut args = vec!["call", "--contract", &params.contract, "--message", &params.message];

        let args_storage;
        if let Some(ref contract_args) = params.args {
            args_storage = contract_args.clone();
            args.push("--args");
            args.push(&args_storage);
        }

        let suri_storage;
        if let Some(ref suri) = params.suri {
            suri_storage = suri.clone();
            args.push("--suri");
            args.push(&suri_storage);
        }

        let url_storage;
        if let Some(ref url) = params.url {
            url_storage = url.clone();
            args.push("--url");
            args.push(&url_storage);
        }

        if params.dry_run.unwrap_or(false) {
            args.push("--dry-run");
        }

        match Self::execute_pop_command(&args) {
            Ok(output) => Ok(Self::success(format!("✅ Contract call successful!\n\n{}", output))),
            Err(e) => Ok(Self::error(format!("Contract call failed: {}", e))),
        }
    }

    #[tool(description = "Get information about a built contract (metadata, size, etc.)")]
    async fn get_contract_info(
        &self,
        Parameters(params): Parameters<GetContractInfoParams>,
    ) -> Result<CallToolResult, McpError> {
        let args = vec!["info", "--path", &params.path];

        match Self::execute_pop_command(&args) {
            Ok(output) => Ok(Self::success(output)),
            Err(e) => Ok(Self::error(format!("Failed to get contract info: {}", e))),
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
            Ok(output) => Ok(Self::success(output)),
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

    #[tool(description = "Convert between Ethereum and Substrate (Polkadot) addresses. If pop convert fails, use the runtime API fallback.")]
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
            Ok(output) => Ok(Self::success(format!("✅ Address conversion:\n\n{}", output))),
            Err(e) => {
                // Fallback: provide instructions for using runtime API
                let fallback_script = format!(
r#"❌ Pop convert failed: {}

📝 Fallback method using Runtime API:

If you need to convert a Substrate address to Ethereum format, use this Node.js script:

```javascript
const {{ ApiPromise, WsProvider }} = require('@polkadot/api');
const {{ decodeAddress }} = require('@polkadot/util-crypto');
const {{ u8aToHex }} = require('@polkadot/util');

async function getEthAddress() {{
  const provider = new WsProvider('wss://testnet-passet-hub.polkadot.io');
  const api = await ApiPromise.create({{ provider }});

  const accountId = '{}';
  const publicKey = decodeAddress(accountId);
  const publicKeyHex = u8aToHex(publicKey);

  const result = await api.call.reviveApi.address(publicKeyHex);
  console.log('Ethereum address:', result.toHex());

  await api.disconnect();
}}

getEthAddress().catch(console.error);
```

To run this:
1. Install dependencies: npm install @polkadot/api @polkadot/util @polkadot/util-crypto
2. Save the script to a file (e.g., convert.js)
3. Run: node convert.js

Note: Conversion from Ethereum to Substrate addresses is not reliably possible.
"#, e, params.address);

                Ok(Self::success(fallback_script))
            }
        }
    }
}

#[tool_handler]
impl ServerHandler for PopMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
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
}
