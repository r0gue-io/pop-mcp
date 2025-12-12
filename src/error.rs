//! Custom error types for Pop MCP Server

use std::fmt;

/// Result type alias for Pop MCP operations
pub type PopMcpResult<T> = Result<T, PopMcpError>;

/// Custom error type for Pop MCP Server
#[derive(Debug)]
pub enum PopMcpError {
    /// Error from Pop CLI command execution
    CommandExecution(String),
    /// Invalid input parameters
    InvalidInput(String),
    /// Resource not found
    ResourceNotFound(String),
    /// Network/fetch error
    NetworkError(String),
    /// Internal server error
    Internal(String),
}

impl fmt::Display for PopMcpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PopMcpError::CommandExecution(msg) => write!(f, "Command execution error: {}", msg),
            PopMcpError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            PopMcpError::ResourceNotFound(msg) => write!(f, "Resource not found: {}", msg),
            PopMcpError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            PopMcpError::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for PopMcpError {}

impl From<anyhow::Error> for PopMcpError {
    fn from(err: anyhow::Error) -> Self {
        PopMcpError::Internal(err.to_string())
    }
}

impl From<reqwest::Error> for PopMcpError {
    fn from(err: reqwest::Error) -> Self {
        PopMcpError::NetworkError(err.to_string())
    }
}
