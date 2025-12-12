//! Pop MCP Server binary entry point

use anyhow::Result;
use pop_mcp_server::PopMcpServer;
use rmcp::{transport::stdio, ServiceExt};

#[tokio::main]
async fn main() -> Result<()> {
    // Create MCP server with Pop CLI tools
    let server = PopMcpServer::new();

    // Serve over stdio
    let service = server.serve(stdio()).await?;

    // Wait for shutdown
    service.waiting().await?;

    Ok(())
}
