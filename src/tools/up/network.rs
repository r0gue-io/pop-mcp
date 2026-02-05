//! Network management (pop up network)

use crate::error::{PopMcpError, PopMcpResult};
use crate::executor::PopExecutor;
use crate::tools::common::error_result;
use rmcp::model::{CallToolResult, Content};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

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
    let mut args = vec!["up", "network", params.path.as_str(), "-y"];

    if params.verbose.unwrap_or(false) {
        args.push("--verbose");
    }

    args
}

fn read_ws_endpoints(zombie_json: &str) -> Result<(String, String), PopMcpError> {
    let contents = fs::read_to_string(zombie_json).map_err(|e| {
        PopMcpError::CommandExecution(format!(
            "Failed to read zombie.json at {}: {}",
            zombie_json, e
        ))
    })?;
    let data: serde_json::Value = serde_json::from_str(&contents).map_err(|e| {
        PopMcpError::CommandExecution(format!(
            "Failed to parse zombie.json at {}: {}",
            zombie_json, e
        ))
    })?;

    let relay_ws = data
        .get("relay")
        .and_then(|v| v.get("nodes"))
        .and_then(|v| v.as_array())
        .and_then(|nodes| {
            nodes
                .iter()
                .find_map(|n| n.get("ws_uri").and_then(|w| w.as_str()))
        })
        .map(ToOwned::to_owned)
        .ok_or_else(|| {
            PopMcpError::CommandExecution("Missing relay ws_uri in zombie.json".to_owned())
        })?;

    let parachains = data.get("parachains");
    let chain_ws = match parachains {
        Some(value) if value.is_array() => value
            .as_array()
            .and_then(|chains| {
                chains.iter().find_map(|chain| {
                    chain
                        .get("collators")
                        .and_then(|c| c.as_array())
                        .and_then(|collators| {
                            collators
                                .iter()
                                .find_map(|n| n.get("ws_uri").and_then(|w| w.as_str()))
                        })
                })
            })
            .map(ToOwned::to_owned),
        Some(value) if value.is_object() => value
            .as_object()
            .and_then(|map| {
                map.values().find_map(|chains| {
                    chains.as_array().and_then(|chain_list| {
                        chain_list.iter().find_map(|chain| {
                            chain.get("collators").and_then(|c| c.as_array()).and_then(
                                |collators| {
                                    collators
                                        .iter()
                                        .find_map(|n| n.get("ws_uri").and_then(|w| w.as_str()))
                                },
                            )
                        })
                    })
                })
            })
            .map(ToOwned::to_owned),
        _ => None,
    }
    .ok_or_else(|| {
        PopMcpError::CommandExecution("Missing chain ws_uri in zombie.json".to_owned())
    })?;

    Ok((relay_ws, chain_ws))
}

fn read_ws_endpoints_with_retry(zombie_json: &str) -> Result<(String, String), PopMcpError> {
    let timeout = Duration::from_secs(60);
    let start = Instant::now();
    loop {
        match read_ws_endpoints(zombie_json) {
            Ok(result) => return Ok(result),
            Err(err) => {
                if start.elapsed() >= timeout {
                    return Err(PopMcpError::CommandExecution(format!(
                        "Timed out waiting for zombie.json to be readable/valid: {}",
                        err
                    )));
                }
            }
        }
        thread::sleep(Duration::from_millis(200));
    }
}

fn find_zombie_json() -> Option<String> {
    let temp_dir = std::env::temp_dir();
    let entries = fs::read_dir(&temp_dir).ok()?;
    let mut newest: Option<(std::time::SystemTime, String)> = None;
    for entry in entries.flatten() {
        let file_type = entry.file_type().ok()?;
        if !file_type.is_dir() {
            continue;
        }
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if !name.starts_with("zombie-") {
            continue;
        }
        let zombie_json = entry.path().join("zombie.json");
        if zombie_json.is_file() {
            let modified = entry
                .metadata()
                .and_then(|m| m.modified())
                .unwrap_or_else(|_| std::time::SystemTime::UNIX_EPOCH);
            let path = zombie_json.to_string_lossy().to_string();
            match newest {
                None => newest = Some((modified, path)),
                Some((current, _)) if modified > current => newest = Some((modified, path)),
                Some(_) => {}
            }
        }
    }
    newest.map(|(_, path)| path)
}

fn execute_up_network(args: &[&str]) -> Result<(String, u32, String), PopMcpError> {
    let mut cmd = Command::new(crate::executor::resolve_pop_command());
    cmd.args(args);

    let temp_name = format!(
        "pop-mcp-up-network-{}.log",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_else(|_| Duration::from_nanos(0))
            .as_nanos()
    );
    let temp_path = std::env::temp_dir().join(temp_name);
    let file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&temp_path)
        .map_err(|e| PopMcpError::CommandExecution(format!("Failed to open log file: {}", e)))?;
    let file_clone = file
        .try_clone()
        .map_err(|e| PopMcpError::CommandExecution(format!("Failed to clone log file: {}", e)))?;

    cmd.stdout(Stdio::from(file_clone));
    cmd.stderr(Stdio::from(file));

    let mut child = cmd.spawn().map_err(|e| {
        PopMcpError::CommandExecution(format!("Failed to execute pop command: {}", e))
    })?;
    let pid = child.id();

    let timeout = Duration::from_secs(300);
    let start = Instant::now();
    let mut output;
    loop {
        output = fs::read_to_string(&temp_path).unwrap_or_default();
        if output.contains("Could not launch local network") {
            let _ = child.kill();
            let _ = child.wait();
            return Err(PopMcpError::CommandExecution(output));
        }
        if let Some(zombie_json) = find_zombie_json() {
            std::mem::forget(child);
            return Ok((output, pid, zombie_json));
        }
        if start.elapsed() >= timeout {
            let _ = child.kill();
            let _ = child.wait();
            return Err(PopMcpError::CommandExecution(
                "Timed out waiting for network output".to_owned(),
            ));
        }
        thread::sleep(Duration::from_millis(200));
    }
}

/// Execute up_network tool (pop up network).
///
/// Returns the output plus parsed zombie.json path for cleanup.
pub fn up_network(
    _executor: &PopExecutor,
    params: UpNetworkParams,
) -> PopMcpResult<CallToolResult> {
    params.validate().map_err(PopMcpError::InvalidInput)?;

    let args = build_up_network_args(&params);
    match execute_up_network(&args) {
        Ok((output, pop_pid, zombie_json)) => {
            let (relay_ws, chain_ws) = match read_ws_endpoints_with_retry(&zombie_json) {
                Ok(endpoints) => endpoints,
                Err(err) => return Ok(error_result(err.to_string())),
            };
            let mut content = vec![Content::text(output.clone())];
            if let Some(base_dir) = Path::new(&zombie_json)
                .parent()
                .map(|p| p.to_string_lossy().to_string())
            {
                content.push(Content::text(format!("base_dir: {}", base_dir)));
            }
            content.push(Content::text(format!("zombie_json: {}", zombie_json)));
            content.push(Content::text(format!("relay_ws: {}", relay_ws)));
            content.push(Content::text(format!("chain_ws: {}", chain_ws)));
            content.push(Content::text(format!("pop_pid: {}", pop_pid)));
            Ok(CallToolResult::success(content))
        }
        Err(e) => Ok(error_result(e.to_string())),
    }
}

#[cfg(test)]
#[allow(clippy::panic)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn validate_rejects_empty_path() {
        let params = UpNetworkParams {
            path: "  ".to_owned(),
            verbose: None,
        };
        assert!(params.validate().is_err());
    }

    #[test]
    fn validate_accepts_path() {
        let params = UpNetworkParams {
            path: "./network.toml".to_owned(),
            verbose: None,
        };
        assert!(params.validate().is_ok());
    }

    #[test]
    fn build_args_includes_defaults() {
        let params = UpNetworkParams {
            path: "./network.toml".to_owned(),
            verbose: None,
        };
        let args = build_up_network_args(&params);
        assert_eq!(args, vec!["up", "network", "./network.toml", "-y"]);
    }

    #[test]
    fn build_args_respects_verbose() {
        let params = UpNetworkParams {
            path: "./network.toml".to_owned(),
            verbose: Some(true),
        };
        let args = build_up_network_args(&params);
        assert_eq!(
            args,
            vec!["up", "network", "./network.toml", "-y", "--verbose"]
        );
    }

    #[test]
    fn read_ws_endpoints_parses_zombie_json() {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| Duration::from_nanos(0))
            .as_nanos();
        let path = std::env::temp_dir().join(format!("zombie-test-{}.json", nanos));
        let json = r#"
{
  "relay": {
    "nodes": [
      { "ws_uri": "ws://127.0.0.1:1111" }
    ]
  },
  "parachains": {
    "1000": [
      {
        "collators": [
          { "ws_uri": "ws://127.0.0.1:2222" }
        ]
      }
    ]
  }
}
"#;
        if let Err(err) = fs::write(&path, json) {
            panic!("write zombie json: {}", err);
        }
        let path_str = path.to_string_lossy();
        let (relay_ws, chain_ws) = match read_ws_endpoints(&path_str) {
            Ok(endpoints) => endpoints,
            Err(err) => panic!("parse endpoints: {}", err),
        };
        assert_eq!(relay_ws, "ws://127.0.0.1:1111");
        assert_eq!(chain_ws, "ws://127.0.0.1:2222");
    }
}
