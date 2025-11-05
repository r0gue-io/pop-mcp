mod tools;
mod resources;

use anyhow::Result;
use rmcp::{transport::stdio, ServiceExt};
use tools::PopMcpServer;

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
