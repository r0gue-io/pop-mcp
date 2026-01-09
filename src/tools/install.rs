//! Installation tools for Pop CLI

use rmcp::model::CallToolResult;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::PopMcpResult;
use crate::executor::PopExecutor;

use super::common::{error_result, success_result};

/// Parameters for the check_pop_installation tool.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[allow(clippy::empty_structs_with_brackets)]
pub struct CheckPopInstallationParams {}

/// Parameters for the install_pop_instructions tool.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct InstallPopInstructionsParams {
    /// Target platform for installation instructions.
    #[schemars(description = "Platform: 'macos', 'linux', or 'source'")]
    pub platform: Option<String>,
}

/// Check if Pop CLI is installed and return version information.
pub fn check_pop_installation(
    executor: &PopExecutor,
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

/// Get installation instructions for Pop CLI.
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
#[allow(clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn install_pop_instructions_default() {
        let params = InstallPopInstructionsParams { platform: None };
        let Ok(result) = install_pop_instructions(params) else {
            panic!("Expected Ok result");
        };
        assert!(!result.is_error.unwrap_or(false));
    }
}
