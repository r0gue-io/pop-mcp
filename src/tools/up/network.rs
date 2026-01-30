//! Network management (pop up network)

use rmcp::model::{CallToolResult, Content};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use crate::error::{PopMcpError, PopMcpResult};
use crate::executor::PopExecutor;
use crate::tools::common::error_result;

/// Parameters for the up_network tool.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[schemars(extend("properties" = {}))]
pub struct UpNetworkParams {
    /// Path to the Zombienet config file.
    #[schemars(description = "Path to the Zombienet network config file")]
    pub path: String,
    /// Whether the output should be verbose (default: false).
    #[schemars(description = "Whether the output should be verbose (default: false)")]
    pub verbose: Option<bool>,
    /// Automatically source all necessary binaries required without prompting (default: true).
    #[schemars(
        description = "Automatically source all necessary binaries required without prompting (default: true)"
    )]
    pub skip_confirm: Option<bool>,
    /// Remove the network state directory on teardown (default: false).
    #[schemars(description = "Remove the network state directory on teardown (default: false)")]
    pub rm: Option<bool>,
}

impl UpNetworkParams {
    /// Validate parameters.
    fn validate(&self) -> Result<(), String> {
        if self.path.trim().is_empty() {
            return Err("Path cannot be empty".to_owned());
        }
        Ok(())
    }
}

/// Build command arguments for up_network.
fn build_up_network_args(params: &UpNetworkParams) -> Vec<&str> {
    let mut args = vec!["up", "network", params.path.as_str()];

    if params.verbose.unwrap_or(false) {
        args.push("-v");
    }

    if params.skip_confirm.unwrap_or(true) {
        args.push("-y");
    }

    if params.rm.unwrap_or(false) {
        args.push("--rm");
    }

    args
}

/// Spawn readers for stdout/stderr to capture output lines.
fn spawn_reader<R: std::io::Read + Send + 'static>(
    reader: R,
    sender: mpsc::Sender<String>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let buffered = BufReader::new(reader);
        for line in buffered.lines().map_while(Result::ok) {
            let _ = sender.send(line);
        }
    })
}

/// Execute up_network tool (pop up network).
///
/// Returns the initial network output and the Pop CLI PID once the network is launched.
pub fn up_network(executor: &PopExecutor, params: UpNetworkParams) -> PopMcpResult<CallToolResult> {
    params.validate().map_err(PopMcpError::InvalidInput)?;

    let args = build_up_network_args(&params);
    let mut child = executor.spawn(&args)?;

    let pid = child.id();
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| PopMcpError::CommandExecution("Failed to capture pop stdout".to_owned()))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| PopMcpError::CommandExecution("Failed to capture pop stderr".to_owned()))?;

    let (sender, receiver) = mpsc::channel::<String>();
    let _stdout_handle = spawn_reader(stdout, sender.clone());
    let _stderr_handle = spawn_reader(stderr, sender);

    let ready_marker = "Network launched successfully - Ctrl+C to terminate";
    let start = Instant::now();
    let timeout = Duration::from_secs(180);
    let mut output_lines = Vec::new();

    loop {
        if start.elapsed() > timeout {
            let _ = child.kill();
            return Ok(error_result(
                "Timed out waiting for network to launch. Ensure required binaries are available.",
            ));
        }

        match receiver.recv_timeout(Duration::from_millis(200)) {
            Ok(line) => {
                if line.contains(ready_marker) {
                    output_lines.push(line);
                    break;
                }
                output_lines.push(line);
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }

        if let Some(status) = child.try_wait().map_err(|e| {
            PopMcpError::CommandExecution(format!("Failed to check pop process status: {}", e))
        })? {
            let output = output_lines.join("\n");
            if status.success() {
                return Ok(error_result(format!(
                    "Pop CLI exited before the network was ready.\n\n{}",
                    output
                )));
            }
            return Ok(error_result(format!(
                "Pop CLI exited with an error before the network was ready.\n\n{}",
                output
            )));
        }
    }

    let output = output_lines.join("\n");
    let mut content = vec![Content::text(output)];
    content.push(Content::text(format!("pid: {}", pid)));

    Ok(CallToolResult::success(content))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_rejects_empty_path() {
        let params = UpNetworkParams {
            path: "  ".to_owned(),
            verbose: None,
            skip_confirm: None,
            rm: None,
        };
        assert!(params.validate().is_err());
    }

    #[test]
    fn validate_accepts_path() {
        let params = UpNetworkParams {
            path: "./network.toml".to_owned(),
            verbose: None,
            skip_confirm: None,
            rm: None,
        };
        assert!(params.validate().is_ok());
    }

    #[test]
    fn build_args_includes_defaults() {
        let params = UpNetworkParams {
            path: "./network.toml".to_owned(),
            verbose: None,
            skip_confirm: None,
            rm: None,
        };
        let args = build_up_network_args(&params);
        assert_eq!(args, vec!["up", "network", "./network.toml", "-y"]);
    }

    #[test]
    fn build_args_respects_flags() {
        let params = UpNetworkParams {
            path: "./network.toml".to_owned(),
            verbose: Some(true),
            skip_confirm: Some(true),
            rm: Some(true),
        };
        let args = build_up_network_args(&params);
        assert_eq!(
            args,
            vec!["up", "network", "./network.toml", "-v", "-y", "--rm"]
        );
    }

    #[test]
    fn build_args_allows_skip_confirm_false() {
        let params = UpNetworkParams {
            path: "./network.toml".to_owned(),
            verbose: None,
            skip_confirm: Some(false),
            rm: None,
        };
        let args = build_up_network_args(&params);
        assert_eq!(args, vec!["up", "network", "./network.toml"]);
    }
}
