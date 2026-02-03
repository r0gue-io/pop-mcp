//! MCP Server implementation for Pop CLI

use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::*,
    service::{RequestContext, RoleServer},
    tool, tool_handler, tool_router, ErrorData as McpError, ServerHandler,
};
use std::sync::{Arc, Mutex};

use crate::executor::PopExecutor;
use crate::resources;
use crate::tools::{common, *};

/// Pop MCP Server - provides tools for Polkadot ink! smart contract development
#[derive(Clone)]
pub struct PopMcpServer {
    tool_router: ToolRouter<Self>,
    executor: PopExecutor,
    node_websocket_url: Arc<Mutex<Option<String>>>,
}

impl PopMcpServer {
    /// Create a new PopMcpServer
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
            executor: PopExecutor::new(),
            node_websocket_url: Arc::new(Mutex::new(None)),
        }
    }

    /// Get the stored node websocket URL
    fn get_stored_url(&self) -> Option<String> {
        self.node_websocket_url
            .lock()
            .ok()
            .and_then(|guard| guard.clone())
    }
}

impl Default for PopMcpServer {
    fn default() -> Self {
        Self::new()
    }
}

#[tool_router]
impl PopMcpServer {
    #[tool(description = "Check if Pop CLI is installed and get version information")]
    async fn check_pop_installation(
        &self,
        Parameters(_): Parameters<CheckPopInstallationParams>,
    ) -> Result<CallToolResult, McpError> {
        check_pop_installation(&self.executor, CheckPopInstallationParams {})
            .map_err(|e| McpError::internal_error(e.to_string(), None))
    }

    #[tool(description = "Get detailed instructions for installing Pop CLI on different platforms")]
    async fn install_pop_instructions(
        &self,
        Parameters(params): Parameters<InstallPopInstructionsParams>,
    ) -> Result<CallToolResult, McpError> {
        install_pop_instructions(params).map_err(|e| McpError::internal_error(e.to_string(), None))
    }

    #[tool(description = "List all available ink! contract templates")]
    async fn list_templates(
        &self,
        Parameters(_): Parameters<ListTemplatesParams>,
    ) -> Result<CallToolResult, McpError> {
        list_templates(ListTemplatesParams {})
            .map_err(|e| McpError::internal_error(e.to_string(), None))
    }

    #[tool(description = "Create a new ink! smart contract from a template using Pop CLI")]
    async fn create_contract(
        &self,
        Parameters(params): Parameters<CreateContractParams>,
    ) -> Result<CallToolResult, McpError> {
        create_contract(&self.executor, params)
            .map_err(|e| McpError::internal_error(e.to_string(), None))
    }

    #[tool(
        description = "Create a new Polkadot Chain project from a template scaffold. Providers: pop, openzeppelin, parity. Templates: r0gue-io/base-parachain, r0gue-io/assets-parachain, r0gue-io/contracts-parachain (pop), openzeppelin/generic-template, openzeppelin/evm-template (openzeppelin), paritytech/polkadot-sdk-parachain-template (parity)"
    )]
    async fn create_chain(
        &self,
        Parameters(params): Parameters<CreateChainParams>,
    ) -> Result<CallToolResult, McpError> {
        create_chain(&self.executor, params)
            .map_err(|e| McpError::internal_error(e.to_string(), None))
    }

    // Frontend-assisted contract creation temporarily disabled.
    /*
    #[tool(
        description = "Create a new ink! smart contract with Dedot/Typink frontend template, automatically adapted to the contract type using Dedot documentation"
    )]
    async fn create_contract_with_frontend(
        &self,
        Parameters(params): Parameters<CreateContractWithFrontendParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::create_contract_with_frontend(&self.executor, params)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))
    }
    */

    #[tool(description = "Build an ink! smart contract using Pop CLI")]
    async fn build_contract(
        &self,
        Parameters(params): Parameters<BuildContractParams>,
    ) -> Result<CallToolResult, McpError> {
        build_contract(&self.executor, params)
            .map_err(|e| McpError::internal_error(e.to_string(), None))
    }

    #[tool(description = "Build a chain project using Pop CLI")]
    async fn build_chain(
        &self,
        Parameters(params): Parameters<BuildChainParams>,
    ) -> Result<CallToolResult, McpError> {
        build_chain(&self.executor, params)
            .map_err(|e| McpError::internal_error(e.to_string(), None))
    }

    #[tool(description = "Run tests for an ink! smart contract")]
    async fn test_contract(
        &self,
        Parameters(params): Parameters<TestContractParams>,
    ) -> Result<CallToolResult, McpError> {
        test_contract(&self.executor, params)
            .map_err(|e| McpError::internal_error(e.to_string(), None))
    }

    #[tool(description = "Run tests for a chain project")]
    async fn test_chain(
        &self,
        Parameters(params): Parameters<TestChainParams>,
    ) -> Result<CallToolResult, McpError> {
        test_chain(&self.executor, params)
            .map_err(|e| McpError::internal_error(e.to_string(), None))
    }

    #[tool(description = "Deploy and instantiate an ink! smart contract to a network")]
    async fn deploy_contract(
        &self,
        Parameters(params): Parameters<DeployContractParams>,
    ) -> Result<CallToolResult, McpError> {
        let stored_url = self.get_stored_url();
        deploy_contract(&self.executor, params, stored_url.as_deref())
            .map_err(|e| McpError::internal_error(e.to_string(), None))
    }

    #[tool(description = "Call a contract method on a deployed contract")]
    async fn call_contract(
        &self,
        Parameters(params): Parameters<CallContractParams>,
    ) -> Result<CallToolResult, McpError> {
        call_contract(&self.executor, params)
            .map_err(|e| McpError::internal_error(e.to_string(), None))
    }

    #[tool(
        description = "Interact with a chain runtime: execute transactions, query storage, or read constants. Use metadata=true to discover pallets/extrinsics/storage/constants."
    )]
    async fn call_chain(
        &self,
        Parameters(params): Parameters<CallChainParams>,
    ) -> Result<CallToolResult, McpError> {
        call_chain(&self.executor, params)
            .map_err(|e| McpError::internal_error(e.to_string(), None))
    }

    #[tool(
        description = "Launch a local ink! node for contract development and testing (runs in background)"
    )]
    async fn up_ink_node(
        &self,
        Parameters(params): Parameters<UpInkNodeParams>,
    ) -> Result<CallToolResult, McpError> {
        let result = up_ink_node(&self.executor, params)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        // Store the WebSocket URL for later use (result contains the URL on success)
        if result.is_error != Some(true) {
            if let Some(url) = common::extract_text(&result) {
                if let Ok(mut stored_url) = self.node_websocket_url.lock() {
                    *stored_url = Some(url);
                }
            }
        }

        Ok(result)
    }

    #[tool(description = "Launch a local network using a zombienet spec")]
    async fn up_network(
        &self,
        Parameters(params): Parameters<UpNetworkParams>,
    ) -> Result<CallToolResult, McpError> {
        up_network(&self.executor, params)
            .map_err(|e| McpError::internal_error(e.to_string(), None))
    }

    #[tool(description = "Stop running local ink! nodes by PID")]
    async fn clean_nodes(
        &self,
        Parameters(params): Parameters<CleanNodesParams>,
    ) -> Result<CallToolResult, McpError> {
        clean_nodes(&self.executor, params)
            .map_err(|e| McpError::internal_error(e.to_string(), None))
    }

    #[tool(description = "Stop a running network by zombie.json path or base dir")]
    async fn clean_network(
        &self,
        Parameters(params): Parameters<CleanNetworkParams>,
    ) -> Result<CallToolResult, McpError> {
        clean_network(&self.executor, params)
            .map_err(|e| McpError::internal_error(e.to_string(), None))
    }

    #[tool(description = "Get help for any Pop CLI command")]
    async fn pop_help(
        &self,
        Parameters(params): Parameters<PopHelpParams>,
    ) -> Result<CallToolResult, McpError> {
        pop_help(&self.executor, params).map_err(|e| McpError::internal_error(e.to_string(), None))
    }

    #[tool(description = "Convert between Ethereum and Substrate (Polkadot) addresses")]
    async fn convert_address(
        &self,
        Parameters(params): Parameters<ConvertAddressParams>,
    ) -> Result<CallToolResult, McpError> {
        convert_address(&self.executor, params)
            .map_err(|e| McpError::internal_error(e.to_string(), None))
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
            instructions: Some(
                "Pop CLI MCP Server - Tools for Polkadot ink! smart contract and chain development using Pop CLI"
                    .to_owned(),
            ),
        }
    }

    fn list_resources(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<ListResourcesResult, McpError>> + Send + '_ {
        std::future::ready(Ok(ListResourcesResult {
            resources: resources::list_resources(),
            next_cursor: None,
        }))
    }

    fn read_resource(
        &self,
        request: ReadResourceRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<ReadResourceResult, McpError>> + Send + '_ {
        std::future::ready(match resources::read_resource(&request.uri) {
            Some(contents) => Ok(ReadResourceResult {
                contents: vec![contents],
            }),
            None => Err(McpError::resource_not_found(
                format!("Resource not found: {}", request.uri),
                None,
            )),
        })
    }
}

#[cfg(test)]
#[allow(clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn server_info_includes_capabilities() {
        let server = PopMcpServer::new();
        let info = server.get_info();

        // Verify protocol version
        assert_eq!(info.protocol_version, ProtocolVersion::V_2024_11_05);

        // Verify capabilities
        assert!(info.capabilities.tools.is_some());
        assert!(info.capabilities.resources.is_some());

        // Verify instructions
        assert!(info
            .instructions
            .as_ref()
            .is_some_and(|i| i.contains("Pop CLI MCP Server")));
    }

    #[test]
    fn server_has_executor() {
        let server = PopMcpServer::new();
        // Server should have executor for running pop CLI commands
        // The #[tool_router] macro ensures tools are registered
        // Just verify the server can be created without panic
        let _ = &server.executor;
    }

    #[test]
    fn url_storage_round_trips() {
        let server = PopMcpServer::new();

        // Initially empty
        assert!(server.get_stored_url().is_none());

        // Store a URL
        if let Ok(mut guard) = server.node_websocket_url.lock() {
            *guard = Some("ws://localhost:9944".to_owned());
        }

        // Verify retrieval
        let url = server.get_stored_url();
        assert_eq!(url, Some("ws://localhost:9944".to_owned()));
    }

    #[test]
    fn all_tool_schemas_are_claude_code_compatible() {
        // Claude Code's MCP client has specific schema requirements:
        // 1. Top-level must have "type": "object" (not {"const": null} from Parameters<()>)
        // 2. No {"const": null} anywhere in schema (generated by Option<Enum> types)
        // 3. "properties" must be an empty object {} (via #[schemars(extend("properties" = {}))])
        //
        // Use empty structs for no-param tools, and Option<String> instead of Option<Enum>.
        // Add #[schemars(extend("properties" = {}))] to ALL param structs.
        let server = PopMcpServer::new();
        let tools = server.tool_router.list_all();

        fn contains_const_null(value: &serde_json::Value) -> bool {
            match value {
                serde_json::Value::Object(map) => {
                    if map.get("const") == Some(&serde_json::Value::Null) {
                        return true;
                    }
                    map.values().any(contains_const_null)
                }
                serde_json::Value::Array(arr) => arr.iter().any(contains_const_null),
                _ => false,
            }
        }

        for tool in tools {
            let Ok(schema_value) = serde_json::to_value(&tool.input_schema) else {
                panic!("Tool '{}' schema failed to serialize", tool.name);
            };

            // Check top-level type
            let schema_type = schema_value.get("type").and_then(|v| v.as_str());
            assert_eq!(
                schema_type,
                Some("object"),
                "Tool '{}' schema must have top-level \"type\": \"object\"",
                tool.name
            );

            // Check for nested {"const": null}
            assert!(
                !contains_const_null(&schema_value),
                "Tool '{}' schema contains {{\"const\": null}} which breaks Claude Code. \
                 Use Option<String> instead of Option<Enum>.",
                tool.name
            );

            // Check that properties is an empty object {} (required for Claude Code compatibility)
            // This is enforced by adding #[schemars(extend("properties" = {}))] to param structs.
            let properties = schema_value.get("properties");
            assert!(
                properties == Some(&serde_json::json!({})),
                "Tool '{}' schema must have \"properties\": {{}}. \
                 Add #[schemars(extend(\"properties\" = {{}}))] to the params struct.",
                tool.name
            );
        }
    }
}
