//! Command execution for Pop CLI

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::{PopMcpError, PopMcpResult};

/// Output from command execution.
#[derive(Debug, Clone)]
struct CommandOutput {
    /// Standard output from the command.
    stdout: String,
    /// Standard error from the command.
    stderr: String,
    /// Whether the command exited successfully.
    success: bool,
}

impl CommandOutput {
    /// Get combined output, preferring stderr for Pop CLI
    fn combined(&self) -> String {
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
            "(Command succeeded but produced no output)".to_owned()
        } else {
            result
        }
    }
}

/// Real implementation of Pop CLI command executor.
///
/// When the `pop-e2e` feature is enabled, the executor supports optional
/// working directory override for test isolation.
#[derive(Debug, Clone, Default)]
pub struct PopExecutor {
    #[cfg(feature = "pop-e2e")]
    cwd: Option<PathBuf>,
}

impl PopExecutor {
    /// Create a new executor with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create an executor with a working directory override.
    #[cfg(feature = "pop-e2e")]
    pub fn with_cwd(cwd: PathBuf) -> Self {
        Self { cwd: Some(cwd) }
    }

    fn execute_raw(&self, args: &[&str]) -> PopMcpResult<CommandOutput> {
        let mut cmd = Command::new(resolve_pop_binary());
        cmd.args(args);

        #[cfg(feature = "pop-e2e")]
        if let Some(ref cwd) = self.cwd {
            cmd.current_dir(cwd);
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

fn resolve_pop_binary() -> PathBuf {
    if let Ok(path) = std::env::var("POP_CLI_PATH") {
        let candidate = PathBuf::from(path);
        if candidate.exists() {
            return candidate;
        }
    }

    if let Some(path) = find_in_path("pop") {
        return path;
    }

    let mut candidates = Vec::new();
    if let Ok(home) = std::env::var("HOME") {
        candidates.push(PathBuf::from(home).join(".cargo/bin/pop"));
    }
    candidates.push(PathBuf::from("/opt/homebrew/bin/pop"));
    candidates.push(PathBuf::from("/usr/local/bin/pop"));

    candidates
        .into_iter()
        .find(|p| p.exists())
        .unwrap_or_else(|| PathBuf::from("pop"))
}

fn find_in_path(bin: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    for entry in std::env::split_paths(&path) {
        let candidate = entry.join(bin);
        if is_executable(&candidate) {
            return Some(candidate);
        }
    }
    None
}

fn is_executable(path: &Path) -> bool {
    if !path.exists() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = path.metadata() {
            return metadata.permissions().mode() & 0o111 != 0;
        }
        false
    }
    #[cfg(not(unix))]
    {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn command_output_combines_streams() {
        let output = CommandOutput {
            stdout: "stdout content".to_owned(),
            stderr: "stderr content".to_owned(),
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

    #[test]
    fn resolve_pop_binary_prefers_pop_cli_path() {
        let temp = match tempdir() {
            Ok(dir) => dir,
            Err(_) => {
                assert!(false);
                return;
            }
        };
        let pop_path = temp.path().join("pop");
        if std::fs::write(&pop_path, "echo pop").is_err() {
            assert!(false);
            return;
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = match std::fs::metadata(&pop_path) {
                Ok(m) => m.permissions(),
                Err(_) => {
                    assert!(false);
                    return;
                }
            };
            perms.set_mode(0o755);
            if std::fs::set_permissions(&pop_path, perms).is_err() {
                assert!(false);
                return;
            }
        }

        let prev = std::env::var_os("POP_CLI_PATH");
        std::env::set_var("POP_CLI_PATH", &pop_path);
        let resolved = resolve_pop_binary();
        if let Some(value) = prev {
            std::env::set_var("POP_CLI_PATH", value);
        } else {
            std::env::remove_var("POP_CLI_PATH");
        }

        assert_eq!(resolved, pop_path);
    }

    #[test]
    fn resolve_pop_binary_uses_path_search() {
        let temp = match tempdir() {
            Ok(dir) => dir,
            Err(_) => {
                assert!(false);
                return;
            }
        };
        let pop_path = temp.path().join("pop");
        if std::fs::write(&pop_path, "echo pop").is_err() {
            assert!(false);
            return;
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = match std::fs::metadata(&pop_path) {
                Ok(m) => m.permissions(),
                Err(_) => {
                    assert!(false);
                    return;
                }
            };
            perms.set_mode(0o755);
            if std::fs::set_permissions(&pop_path, perms).is_err() {
                assert!(false);
                return;
            }
        }

        let prev_pop_cli_path = std::env::var_os("POP_CLI_PATH");
        let prev_path = std::env::var_os("PATH");
        std::env::remove_var("POP_CLI_PATH");
        std::env::set_var("PATH", temp.path());

        let resolved = resolve_pop_binary();

        if let Some(value) = prev_pop_cli_path {
            std::env::set_var("POP_CLI_PATH", value);
        } else {
            std::env::remove_var("POP_CLI_PATH");
        }
        if let Some(value) = prev_path {
            std::env::set_var("PATH", value);
        } else {
            std::env::remove_var("PATH");
        }

        assert_eq!(resolved, pop_path);
    }
}
