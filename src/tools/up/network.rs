//! Network management (pop up network)

use crate::error::{PopMcpError, PopMcpResult};
use crate::executor::PopExecutor;
use crate::tools::common::{error_result, success_result};
use rmcp::model::CallToolResult;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Known relay chains that can be spawned directly via `pop up <chain>`.
const KNOWN_CHAINS: &[&str] = &["paseo", "kusama", "polkadot", "westend"];

/// Parameters for the up_network tool.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[schemars(extend("properties" = {}))]
pub struct UpNetworkParams {
    /// Path to the Zombienet config file. Mutually exclusive with `chain`.
    #[schemars(description = "Path to the Zombienet network config file")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// Known relay chain to spawn (paseo, kusama, polkadot, westend). Mutually exclusive with `path`.
    #[schemars(
        description = "Known relay chain to spawn: paseo, kusama, polkadot, westend (case-insensitive)"
    )]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain: Option<String>,
    /// Whether the output should be verbose (default: false).
    #[schemars(description = "Whether the output should be verbose (default: false)")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verbose: Option<bool>,
}

impl UpNetworkParams {
    /// Validate parameters.
    ///
    /// - Exactly one of `path` or `chain` must be provided.
    /// - If `chain` is provided, it must be one of the known chains (case-insensitive).
    fn validate(&self) -> Result<(), String> {
        match (&self.path, &self.chain) {
            (Some(p), None) => {
                if p.trim().is_empty() {
                    return Err("Path cannot be empty".to_owned());
                }
                Ok(())
            }
            (None, Some(c)) => {
                let normalized = c.to_lowercase();
                if !KNOWN_CHAINS.contains(&normalized.as_str()) {
                    return Err(format!(
                        "Unknown chain '{}'. Must be one of: {}",
                        c,
                        KNOWN_CHAINS.join(", ")
                    ));
                }
                Ok(())
            }
            (Some(_), Some(_)) => {
                Err("Cannot specify both 'path' and 'chain'. Use one or the other.".to_owned())
            }
            (None, None) => Err("Must specify either 'path' or 'chain'.".to_owned()),
        }
    }

    /// Get the normalized chain name (lowercase) if provided.
    fn normalized_chain(&self) -> Option<String> {
        self.chain.as_ref().map(|c| c.to_lowercase())
    }
}

/// Command mode determined by parameters.
enum UpMode<'a> {
    /// Run `pop up network <path>`.
    Network(&'a str),
    /// Run `pop up <chain>`.
    Chain(&'a str),
}

/// Build command arguments for up_network.
fn build_up_network_args<'a>(
    params: &'a UpNetworkParams,
    chain_normalized: &'a Option<String>,
) -> Vec<&'a str> {
    let mode = if let Some(ref path) = params.path {
        UpMode::Network(path.as_str())
    } else if let Some(ref chain) = chain_normalized {
        UpMode::Chain(chain.as_str())
    } else {
        unreachable!("validate() ensures one of path or chain is set")
    };

    let mut args = match mode {
        UpMode::Network(path) => vec!["up", "network", path, "-y", "--detach"],
        UpMode::Chain(chain) => vec!["up", chain, "-y", "--detach"],
    };

    if params.verbose.unwrap_or(false) {
        args.push("--verbose");
    }

    args
}

/// Execute up_network tool (pop up network / pop up <chain>).
///
/// Returns the Pop CLI output directly, which includes the zombie.json path
/// and network status.
pub fn up_network(executor: &PopExecutor, params: UpNetworkParams) -> PopMcpResult<CallToolResult> {
    params.validate().map_err(PopMcpError::InvalidInput)?;

    let chain_normalized = params.normalized_chain();
    let args = build_up_network_args(&params, &chain_normalized);
    match executor.execute(&args) {
        Ok(output) => Ok(success_result(output)),
        Err(e) => Ok(error_result(e.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validation tests

    #[test]
    fn validate_rejects_empty_path() {
        let params = UpNetworkParams {
            path: Some("  ".to_owned()),
            chain: None,
            verbose: None,
        };
        assert!(params.validate().is_err());
    }

    #[test]
    fn validate_accepts_path() {
        let params = UpNetworkParams {
            path: Some("./network.toml".to_owned()),
            chain: None,
            verbose: None,
        };
        assert!(params.validate().is_ok());
    }

    #[test]
    fn validate_accepts_known_chain() {
        for chain in KNOWN_CHAINS {
            let params = UpNetworkParams {
                path: None,
                chain: Some(chain.to_string()),
                verbose: None,
            };
            assert!(params.validate().is_ok(), "should accept chain '{}'", chain);
        }
    }

    #[test]
    fn validate_accepts_chain_case_insensitive() {
        let params = UpNetworkParams {
            path: None,
            chain: Some("PASEO".to_owned()),
            verbose: None,
        };
        assert!(params.validate().is_ok());

        let params = UpNetworkParams {
            path: None,
            chain: Some("Kusama".to_owned()),
            verbose: None,
        };
        assert!(params.validate().is_ok());
    }

    #[test]
    fn validate_rejects_unknown_chain() {
        let params = UpNetworkParams {
            path: None,
            chain: Some("unknown".to_owned()),
            verbose: None,
        };
        assert!(params.validate().is_err());
    }

    #[test]
    fn validate_rejects_both_path_and_chain() {
        let params = UpNetworkParams {
            path: Some("./network.toml".to_owned()),
            chain: Some("paseo".to_owned()),
            verbose: None,
        };
        assert!(params.validate().is_err());
    }

    #[test]
    fn validate_rejects_neither_path_nor_chain() {
        let params = UpNetworkParams {
            path: None,
            chain: None,
            verbose: None,
        };
        assert!(params.validate().is_err());
    }

    // Build args tests

    #[test]
    fn build_args_for_path() {
        let params = UpNetworkParams {
            path: Some("./network.toml".to_owned()),
            chain: None,
            verbose: None,
        };
        let chain_normalized = params.normalized_chain();
        let args = build_up_network_args(&params, &chain_normalized);
        assert_eq!(
            args,
            vec!["up", "network", "./network.toml", "-y", "--detach"]
        );
    }

    #[test]
    fn build_args_for_chain() {
        let params = UpNetworkParams {
            path: None,
            chain: Some("PASEO".to_owned()),
            verbose: None,
        };
        let chain_normalized = params.normalized_chain();
        let args = build_up_network_args(&params, &chain_normalized);
        assert_eq!(args, vec!["up", "paseo", "-y", "--detach"]);
    }

    #[test]
    fn build_args_respects_verbose_for_path() {
        let params = UpNetworkParams {
            path: Some("./network.toml".to_owned()),
            chain: None,
            verbose: Some(true),
        };
        let chain_normalized = params.normalized_chain();
        let args = build_up_network_args(&params, &chain_normalized);
        assert_eq!(
            args,
            vec![
                "up",
                "network",
                "./network.toml",
                "-y",
                "--detach",
                "--verbose"
            ]
        );
    }

    #[test]
    fn build_args_respects_verbose_for_chain() {
        let params = UpNetworkParams {
            path: None,
            chain: Some("kusama".to_owned()),
            verbose: Some(true),
        };
        let chain_normalized = params.normalized_chain();
        let args = build_up_network_args(&params, &chain_normalized);
        assert_eq!(args, vec!["up", "kusama", "-y", "--detach", "--verbose"]);
    }
}
