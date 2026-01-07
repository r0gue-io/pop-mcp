//! Command execution for Pop CLI

#[cfg(feature = "pop-e2e")]
use std::ffi::OsString;
#[cfg(feature = "pop-e2e")]
use std::path::PathBuf;
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

/// Real implementation of Pop CLI command executor.
///
/// When the `pop-e2e` feature is enabled, the executor supports optional
/// working directory and environment overrides for test isolation.
#[derive(Debug, Clone, Default)]
pub struct PopExecutor {
    /// Override working directory for command execution (pop-e2e only).
    #[cfg(feature = "pop-e2e")]
    cwd: Option<PathBuf>,
    /// Environment variable overrides (pop-e2e only).
    #[cfg(feature = "pop-e2e")]
    env: Vec<(OsString, OsString)>,
}

impl PopExecutor {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create an executor with working directory and environment overrides.
    ///
    /// This is only available in `pop-e2e` builds for test isolation.
    #[cfg(feature = "pop-e2e")]
    pub fn with_overrides(
        cwd: Option<PathBuf>,
        env: impl IntoIterator<Item = (impl Into<OsString>, impl Into<OsString>)>,
    ) -> Self {
        Self {
            cwd,
            env: env.into_iter().map(|(k, v)| (k.into(), v.into())).collect(),
        }
    }

    fn execute_raw(&self, args: &[&str]) -> PopMcpResult<CommandOutput> {
        let mut cmd = Command::new("pop");
        cmd.args(args);

        // Apply overrides when pop-e2e feature is enabled
        #[cfg(feature = "pop-e2e")]
        {
            if let Some(ref cwd) = self.cwd {
                cmd.current_dir(cwd);
            }
            if !self.env.is_empty() {
                cmd.envs(self.env.iter().map(|(k, v)| (k, v)));
            }
        }

        let output = cmd.output().map_err(|e| {
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
