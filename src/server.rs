//! MCP Server implementation for Pop CLI

use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::*,
    service::RequestContext,
    tool, tool_handler, tool_router, ErrorData as McpError, RoleServer, ServerHandler,
};
use std::sync::{Arc, Mutex};

use crate::executor::PopExecutor;
use crate::resources::SearchDocumentationParams;
use crate::tools::{self, *};

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
        tools::check_pop_installation(&self.executor, CheckPopInstallationParams {})
            .map_err(|e| McpError::internal_error(e.to_string(), None))
    }

    #[tool(description = "Get detailed instructions for installing Pop CLI on different platforms")]
    async fn install_pop_instructions(
        &self,
        Parameters(params): Parameters<InstallPopInstructionsParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::install_pop_instructions(params)
            .map_err(|e| McpError::internal_error(e.to_string(), None))
    }

    #[tool(description = "List all available ink! contract templates")]
    async fn list_templates(
        &self,
        Parameters(_): Parameters<ListTemplatesParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::list_templates(ListTemplatesParams {})
            .map_err(|e| McpError::internal_error(e.to_string(), None))
    }

    #[tool(description = "Create a new ink! smart contract from a template using Pop CLI")]
    async fn create_contract(
        &self,
        Parameters(params): Parameters<CreateContractParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::create_contract(&self.executor, params)
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
        tools::build_contract(&self.executor, params)
            .map_err(|e| McpError::internal_error(e.to_string(), None))
    }

    #[tool(description = "Run tests for an ink! smart contract")]
    async fn test_contract(
        &self,
        Parameters(params): Parameters<TestContractParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::test_contract(&self.executor, params)
            .map_err(|e| McpError::internal_error(e.to_string(), None))
    }

    #[tool(description = "Deploy and instantiate an ink! smart contract to a network")]
    async fn deploy_contract(
        &self,
        Parameters(params): Parameters<DeployContractParams>,
    ) -> Result<CallToolResult, McpError> {
        let stored_url = self.get_stored_url();
        tools::deploy_contract(&self.executor, params, stored_url.as_deref())
            .map_err(|e| McpError::internal_error(e.to_string(), None))
    }

    #[tool(description = "Call a contract method on a deployed contract")]
    async fn call_contract(
        &self,
        Parameters(params): Parameters<CallContractParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::call_contract(&self.executor, params)
            .map_err(|e| McpError::internal_error(e.to_string(), None))
    }

    #[tool(
        description = "Launch a local ink! node for contract development and testing (runs in background)"
    )]
    async fn up_ink_node(
        &self,
        Parameters(params): Parameters<UpInkNodeParams>,
    ) -> Result<CallToolResult, McpError> {
        let result = tools::up_ink_node(&self.executor, params)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        // Store the WebSocket URL for later use (result contains the URL on success)
        if result.is_error != Some(true) {
            if let Some(url) = tools::common::extract_text(&result) {
                if let Ok(mut stored_url) = self.node_websocket_url.lock() {
                    *stored_url = Some(url);
                }
            }
        }

        Ok(result)
    }

    #[tool(description = "Stop running local ink! nodes by PID")]
    async fn clean_nodes(
        &self,
        Parameters(params): Parameters<CleanNodesParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::clean_nodes(&self.executor, params)
            .map_err(|e| McpError::internal_error(e.to_string(), None))
    }

    #[tool(description = "Get help for any Pop CLI command")]
    async fn pop_help(
        &self,
        Parameters(params): Parameters<PopHelpParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::pop_help(&self.executor, params)
            .map_err(|e| McpError::internal_error(e.to_string(), None))
    }

    #[tool(description = "Convert between Ethereum and Substrate (Polkadot) addresses")]
    async fn convert_address(
        &self,
        Parameters(params): Parameters<ConvertAddressParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::convert_address(&self.executor, params)
            .map_err(|e| McpError::internal_error(e.to_string(), None))
    }

    #[tool(
        description = "Search through all Polkadot documentation for specific topics or keywords"
    )]
    async fn search_documentation(
        &self,
        Parameters(params): Parameters<SearchDocumentationParams>,
    ) -> Result<CallToolResult, McpError> {
        crate::resources::search_documentation(params).await
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
                "Pop CLI MCP Server - Tools for Polkadot ink! smart contract and parachain development using Pop CLI"
                    .to_string(),
            ),
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

#[cfg(test)]
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
        assert!(info.instructions.is_some());
        let instructions = info.instructions.unwrap();
        assert!(instructions.contains("Pop CLI MCP Server"));
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
        {
            let mut guard = server.node_websocket_url.lock().unwrap();
            *guard = Some("ws://localhost:9944".to_string());
        }

        // Verify retrieval
        let url = server.get_stored_url();
        assert_eq!(url, Some("ws://localhost:9944".to_string()));
    }

    #[test]
    fn all_tool_schemas_are_claude_code_compatible() {
        // Claude Code's MCP client has specific schema requirements:
        // 1. Top-level must have "type": "object" (not {"const": null} from Parameters<()>)
        // 2. No {"const": null} anywhere in schema (generated by Option<Enum> types)
        // Use empty structs for no-param tools, and Option<String> instead of Option<Enum>.
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
            let schema_value = serde_json::to_value(&tool.input_schema).unwrap();

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
        }
    }
}
