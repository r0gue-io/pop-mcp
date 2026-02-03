//! Network management (pop up network)

use crate::error::{PopMcpError, PopMcpResult};
use crate::executor::PopExecutor;
use crate::tools::common::error_result;
use rmcp::model::{CallToolResult, Content};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fs;
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
    let mut args = vec!["up", "network", params.path.as_str(), "--detach", "-y"];

    if params.verbose.unwrap_or(false) {
        args.push("--verbose");
    }

    args
}

/// Parse the output to extract zombie.json path.
fn parse_zombie_json(output: &str) -> Option<String> {
    for line in output.lines() {
        let trimmed = line.trim().trim_start_matches('│').trim();
        if let Some(index) = trimmed.find("zombie.json:") {
            let rest = &trimmed[index + "zombie.json:".len()..];
            let path = rest.trim();
            if !path.is_empty() {
                return Some(path.to_owned());
            }
        }
    }
    None
}

/// Parse the output to extract base dir path.
fn parse_base_dir(output: &str) -> Option<String> {
    for line in output.lines() {
        let trimmed = line.trim().trim_start_matches('│').trim();
        if let Some(index) = trimmed.find("base dir:") {
            let rest = &trimmed[index + "base dir:".len()..];
            let path = rest.trim();
            if !path.is_empty() {
                return Some(path.to_owned());
            }
        }
    }
    None
}

fn resolve_zombie_json(output: &str) -> Option<String> {
    parse_zombie_json(output)
        .or_else(|| parse_base_dir(output).map(|base_dir| format!("{}/zombie.json", base_dir)))
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
                    return Err(err);
                }
            }
        }
        thread::sleep(Duration::from_millis(200));
    }
}

/// Execute up_network tool (pop up network).
///
/// Returns the output plus parsed zombie.json path for cleanup.
pub fn up_network(executor: &PopExecutor, params: UpNetworkParams) -> PopMcpResult<CallToolResult> {
    params.validate().map_err(PopMcpError::InvalidInput)?;

    let args = build_up_network_args(&params);
    match executor.execute(&args) {
        Ok(output) => {
            let zombie_json = match resolve_zombie_json(&output) {
                Some(path) => path,
                None => return Ok(error_result("Failed to parse zombie.json path from output")),
            };
            let (relay_ws, chain_ws) = match read_ws_endpoints_with_retry(&zombie_json) {
                Ok(endpoints) => endpoints,
                Err(err) => return Ok(error_result(err.to_string())),
            };
            let mut content = vec![Content::text(output.clone())];
            if let Some(base_dir) = parse_base_dir(&output) {
                content.push(Content::text(format!("base_dir: {}", base_dir)));
            }
            content.push(Content::text(format!("zombie_json: {}", zombie_json)));
            content.push(Content::text(format!("relay_ws: {}", relay_ws)));
            content.push(Content::text(format!("chain_ws: {}", chain_ws)));
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
        assert_eq!(
            args,
            vec!["up", "network", "./network.toml", "--detach", "-y"]
        );
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
            vec![
                "up",
                "network",
                "./network.toml",
                "--detach",
                "-y",
                "--verbose"
            ]
        );
    }

    #[test]
    fn parse_zombie_json_extracts_path() {
        let output = r#"
│  base dir: /tmp/zombie-abc
│  zombie.json: /tmp/zombie-abc/zombie.json
"#;
        let path = parse_zombie_json(output);
        assert_eq!(path, Some("/tmp/zombie-abc/zombie.json".to_owned()));
    }

    #[test]
    fn parse_base_dir_extracts_path() {
        let output = r#"
│  base dir: /tmp/zombie-xyz
│  zombie.json: /tmp/zombie-xyz/zombie.json
"#;
        let path = parse_base_dir(output);
        assert_eq!(path, Some("/tmp/zombie-xyz".to_owned()));
    }

    #[test]
    fn resolve_zombie_json_prefers_explicit_path() {
        let output = r#"
│  base dir: /tmp/zombie-aaa
│  zombie.json: /tmp/zombie-aaa/zombie.json
"#;
        let path = resolve_zombie_json(output);
        assert_eq!(path, Some("/tmp/zombie-aaa/zombie.json".to_owned()));
    }

    #[test]
    fn resolve_zombie_json_from_base_dir_when_missing() {
        let output = r#"
│  base dir: /tmp/zombie-bbb
"#;
        let path = resolve_zombie_json(output);
        assert_eq!(path, Some("/tmp/zombie-bbb/zombie.json".to_owned()));
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
