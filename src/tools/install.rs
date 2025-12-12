//! Installation tools for Pop CLI

use rmcp::model::CallToolResult;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::PopMcpResult;
use crate::executor::CommandExecutor;

use super::helpers::{error_result, success_result};

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct CheckPopInstallationParams {}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct InstallPopInstructionsParams {
    #[schemars(description = "Platform: 'macos', 'linux', or 'source'")]
    pub platform: Option<String>,
}

pub fn check_pop_installation<E: CommandExecutor>(
    executor: &E,
    _params: CheckPopInstallationParams,
) -> PopMcpResult<CallToolResult> {
    match executor.execute(&["--version"]) {
        Ok(output) => Ok(success_result(format!("Pop CLI is installed!\n\n{}", output))),
        Err(e) => Ok(error_result(format!(
            "Pop CLI is not installed.\n\nError: {}\n\nTo install Pop CLI, use the install_pop_instructions tool.",
            e
        ))),
    }
}

pub fn install_pop_instructions(
    params: InstallPopInstructionsParams,
) -> PopMcpResult<CallToolResult> {
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

    Ok(success_result(instructions))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::test_utils::MockExecutor;

    #[test]
    fn test_check_pop_installation_installed() {
        let executor = MockExecutor::success("pop 0.5.0");
        let result = check_pop_installation(&executor, CheckPopInstallationParams {}).unwrap();
        assert!(!result.is_error.unwrap_or(true));
    }

    #[test]
    fn test_check_pop_installation_not_installed() {
        let executor = MockExecutor::failure("command not found");
        let result = check_pop_installation(&executor, CheckPopInstallationParams {}).unwrap();
        assert!(result.is_error.unwrap_or(false));
    }

    #[test]
    fn test_install_pop_instructions_default() {
        let params = InstallPopInstructionsParams { platform: None };
        let result = install_pop_instructions(params).unwrap();
        assert!(!result.is_error.unwrap_or(false));
    }
}
