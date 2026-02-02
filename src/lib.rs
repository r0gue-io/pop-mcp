//! Pop MCP Server - MCP tools for Polkadot ink! smart contract development
//!
//! This library provides MCP (Model Context Protocol) tools for interacting
//! with Pop CLI, enabling AI assistants to help with smart contract development.
pub mod error;
pub mod executor;
pub mod resources;
pub mod server;
pub mod tools;

pub use error::{PopMcpError, PopMcpResult};
pub use executor::PopExecutor;
pub use server::PopMcpServer;

/// SURI from PRIVATE_KEY env var.
pub fn read_private_key_suri() -> Option<String> {
    std::env::var("PRIVATE_KEY").ok()
}
