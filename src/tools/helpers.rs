//! Helper functions for tool implementations

#[cfg(test)]
use crate::executor::{CommandExecutor, PopExecutor};
#[cfg(test)]
use crate::tools::new::contract::{create_contract, CreateContractParams};
use rmcp::model::{CallToolResult, Content, RawContent};
#[cfg(test)]
use std::path::PathBuf;
#[cfg(test)]
use tempfile::TempDir;

/// Create a success result with the given text
pub fn success_result(text: impl Into<String>) -> CallToolResult {
    CallToolResult::success(vec![Content::text(text.into())])
}

/// Create an error result with the given text
pub fn error_result(text: impl Into<String>) -> CallToolResult {
    CallToolResult::error(vec![Content::text(text.into())])
}

/// Extract text content from a CallToolResult
pub fn extract_text(result: &CallToolResult) -> Option<String> {
    result.content.first().and_then(|c| match &c.raw {
        RawContent::Text(t) => Some(t.text.clone()),
        _ => None,
    })
}

#[cfg(test)]
pub fn content_text(result: &CallToolResult) -> String {
    extract_text(result).expect("CallToolResult must contain text content")
}

#[cfg(test)]
pub fn pop_available(executor: &PopExecutor) -> bool {
    executor.execute(&["--version"]).is_ok()
        || std::process::Command::new("pop")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
}

/// Create a standard contract in a temp directory and return its path.
///
/// The returned guard keeps the temporary directory alive so the contract
/// artifacts persist for the duration of the test. It temporarily changes
/// the process working directory while creating the contract; prefer
/// `serial` tests or single-threaded runs when using it.
#[cfg(test)]
pub struct Contract {
    pub temp_dir: TempDir,
    pub path: PathBuf,
}

#[cfg(test)]
pub fn create_standard_contract(executor: &PopExecutor, name: &str) -> Contract {
    let temp_dir = TempDir::new().expect("create temp dir");
    let original_dir = std::env::current_dir().expect("get cwd");
    std::env::set_current_dir(temp_dir.path()).expect("enter temp dir");

    let params = CreateContractParams {
        name: name.to_string(),
        template: "standard".to_string(),
    };
    let result = create_contract(executor, params).expect("create contract");
    assert!(!content_text(&result).is_empty());

    std::env::set_current_dir(original_dir).expect("restore cwd");

    Contract {
        path: temp_dir.path().join(name),
        temp_dir,
    }
}

#[cfg(test)]
pub mod test_utils {
    use std::process::Command;

    /// Check if a port is in use using lsof
    pub fn is_port_in_use(port: u16) -> bool {
        Command::new("lsof")
            .args(["-i", &format!(":{}", port)])
            .output()
            .map(|o| !o.stdout.is_empty())
            .unwrap_or(false)
    }
}
