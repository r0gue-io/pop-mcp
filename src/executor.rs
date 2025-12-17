//! Command execution for Pop CLI

use std::process::Command;

use crate::error::{PopMcpError, PopMcpResult};

/// Output from command execution
#[derive(Debug, Clone)]
pub struct CommandOutput {
    pub stdout: String,
    pub stderr: String,
    pub success: bool,
}

impl CommandOutput {
    /// Get combined output, preferring stderr for Pop CLI
    pub fn combined(&self) -> String {
        let mut result = String::new();

        if !self.stderr.is_empty() {
            result.push_str(&self.stderr);
        }

        if !self.stdout.is_empty() {
            if !result.is_empty() {
                result.push_str("\n\n");
            }
            result.push_str(&self.stdout);
        }

        if result.is_empty() {
            "(Command succeeded but produced no output)".to_string()
        } else {
            result
        }
    }
}

/// Real implementation of Pop CLI command executor
#[derive(Debug, Clone, Default)]
pub struct PopExecutor;

impl PopExecutor {
    pub fn new() -> Self {
        Self
    }

    fn execute_raw(&self, args: &[&str]) -> PopMcpResult<CommandOutput> {
        let output = Command::new("pop").args(args).output().map_err(|e| {
            PopMcpError::CommandExecution(format!("Failed to execute pop command: {}", e))
        })?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        Ok(CommandOutput {
            stdout,
            stderr,
            success: output.status.success(),
        })
    }

    /// Execute a Pop CLI command with the given arguments
    pub fn execute(&self, args: &[&str]) -> PopMcpResult<String> {
        let output = self.execute_raw(args)?;

        if output.success {
            Ok(output.combined())
        } else {
            // For failed commands, return error with combined output
            let mut error = String::new();
            if !output.stderr.is_empty() {
                error.push_str(&output.stderr);
            }
            if !output.stdout.is_empty() {
                if !error.is_empty() {
                    error.push_str("\n\n");
                }
                error.push_str(&output.stdout);
            }
            Err(PopMcpError::CommandExecution(error))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_output_combines_streams() {
        let output = CommandOutput {
            stdout: "stdout content".to_string(),
            stderr: "stderr content".to_string(),
            success: true,
        };
        assert!(output.combined().contains("stderr content"));
        assert!(output.combined().contains("stdout content"));
    }

    #[test]
    fn command_output_empty() {
        let output = CommandOutput {
            stdout: String::new(),
            stderr: String::new(),
            success: true,
        };
        assert_eq!(
            output.combined(),
            "(Command succeeded but produced no output)"
        );
    }
}
