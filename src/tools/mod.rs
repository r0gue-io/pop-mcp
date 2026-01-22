//! Tool implementations for Pop MCP Server

use rmcp::model::CallToolResult;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::PopMcpResult;
use crate::executor::PopExecutor;

pub mod build;
pub mod call;
pub mod clean;
pub mod common;
pub mod convert;
pub mod install;
pub mod new;
pub mod test;
pub mod up;

pub use build::contract::{build_contract, BuildContractParams};
pub use call::contract::{call_contract, CallContractParams};
pub use clean::{clean_nodes, CleanNodesParams};
pub use convert::{convert_address, ConvertAddressParams};
pub use install::{
    check_pop_installation, install_pop_instructions, CheckPopInstallationParams,
    InstallPopInstructionsParams,
};
pub use new::chain::{create_chain, CreateChainParams};
pub use new::contract::{create_contract, CreateContractParams};
pub use test::contract::{test_contract, TestContractParams};
pub use up::chain::{up_ink_node, UpInkNodeParams};
pub use up::contract::{deploy_contract, DeployContractParams};

pub(crate) use new::contract::{list_templates, ListTemplatesParams};

/// Parameters for the pop_help tool.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[schemars(extend("properties" = {}))]
pub struct PopHelpParams {
    /// Command to get help for.
    #[schemars(description = "Command to get help for")]
    pub command: Option<String>,
}

/// Get help for Pop CLI commands.
pub(crate) fn pop_help(
    executor: &PopExecutor,
    params: PopHelpParams,
) -> PopMcpResult<CallToolResult> {
    let args = if let Some(ref command) = params.command {
        let mut cmd_parts: Vec<&str> = command.split_whitespace().collect();
        cmd_parts.push("--help");
        cmd_parts
    } else {
        vec!["--help"]
    };

    match executor.execute(&args) {
        Ok(output) => Ok(common::success_result(format!(
            "Pop CLI Help:\n\n{}",
            output
        ))),
        Err(e) => Ok(common::error_result(format!("Failed to get help: {}", e))),
    }
}
