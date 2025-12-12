//! Tool implementations for Pop MCP Server

use rmcp::model::CallToolResult;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::PopMcpResult;
use crate::executor::CommandExecutor;

pub mod build;
pub mod call;
pub mod convert;
pub mod helpers;
pub mod install;
pub mod new;
pub mod test;
pub mod up;

pub use build::contract::{build_build_contract_args, build_contract, BuildContractParams};
pub use call::contract::{build_call_contract_args, call_contract, CallContractParams};
pub use convert::{convert_address, ConvertAddressParams};
pub use install::{
    check_pop_installation, install_pop_instructions, CheckPopInstallationParams,
    InstallPopInstructionsParams,
};
pub use new::contract::{build_create_contract_args, create_contract, CreateContractParams};
pub use new::list_templates;
pub use test::contract::{build_test_contract_args, test_contract, TestContractParams};
pub use up::chain::{launch_ink_node, parse_launch_output, stop_nodes, LaunchNodeResult};
pub use up::contract::{build_deploy_contract_args, deploy_contract, DeployContractParams};

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct PopHelpParams {
    #[schemars(description = "Command to get help for")]
    pub command: Option<String>,
}

pub fn pop_help<E: CommandExecutor>(
    executor: &E,
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
        Ok(output) => Ok(helpers::success_result(format!(
            "Pop CLI Help:\n\n{}",
            output
        ))),
        Err(e) => Ok(helpers::error_result(format!("Failed to get help: {}", e))),
    }
}
