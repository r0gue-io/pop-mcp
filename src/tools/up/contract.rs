//! Contract deployment (pop up <contract>)

use rmcp::model::CallToolResult;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::PopMcpResult;
use crate::executor::CommandExecutor;
use crate::tools::helpers::{error_result, success_result};

// Parameters

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct DeployContractParams {
    #[schemars(
        description = "Path to the contract directory (e.g., './my_contract' or 'my_contract')"
    )]
    pub path: String,
    #[schemars(description = "Constructor function to call")]
    pub constructor: Option<String>,
    #[schemars(description = "Constructor arguments as space-separated values")]
    pub args: Option<String>,
    #[schemars(description = "Initial balance to transfer to the contract (in tokens)")]
    pub value: Option<String>,
    #[schemars(description = "Submit an extrinsic for on-chain execution")]
    pub execute: Option<bool>,
    #[schemars(description = "Secret key URI for signing")]
    pub suri: Option<String>,
    #[schemars(description = "WebSocket URL of the node")]
    pub url: Option<String>,
}

/// Build command arguments for deploy_contract
pub fn build_deploy_contract_args<'a>(
    params: &'a DeployContractParams,
    stored_url: Option<&'a str>,
) -> Vec<&'a str> {
    let mut args = vec!["up", params.path.as_str(), "-y"];

    if let Some(ref constructor) = params.constructor {
        args.push("--constructor");
        args.push(constructor.as_str());
    }

    if let Some(ref contract_args) = params.args {
        args.push("--args");
        args.push(contract_args.as_str());
    }

    if let Some(ref value) = params.value {
        args.push("--value");
        args.push(value.as_str());
    }

    if params.execute.unwrap_or(false) {
        args.push("--execute");
    }

    if let Some(ref suri) = params.suri {
        args.push("--suri");
        args.push(suri.as_str());
    }

    // Use provided URL or fall back to stored URL
    if let Some(ref url) = params.url {
        args.push("--url");
        args.push(url.as_str());
    } else if let Some(url) = stored_url {
        args.push("--url");
        args.push(url);
    }

    args
}

/// Execute deploy_contract tool
pub fn deploy_contract<E: CommandExecutor>(
    executor: &E,
    params: DeployContractParams,
    stored_url: Option<&str>,
) -> PopMcpResult<CallToolResult> {
    let args = build_deploy_contract_args(&params, stored_url);

    match executor.execute(&args) {
        Ok(output) => Ok(success_result(output)),
        Err(e) => Ok(error_result(format!("Deployment failed:\n\n{}", e))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::PopExecutor;
    use rmcp::model::RawContent;

    fn content_text(result: &rmcp::model::CallToolResult) -> String {
        result
            .content
            .last()
            .and_then(|c| match &c.raw {
                RawContent::Text(t) => Some(t.text.clone()),
                _ => None,
            })
            .unwrap_or_default()
    }

    #[test]
    fn test_build_args_minimal() {
        let params = DeployContractParams {
            path: "./my_contract".to_string(),
            constructor: None,
            args: None,
            value: None,
            execute: None,
            suri: None,
            url: None,
        };
        let args = build_deploy_contract_args(&params, None);
        assert_eq!(args, vec!["up", "./my_contract", "-y"]);
    }

    #[test]
    fn test_build_args_full() {
        let params = DeployContractParams {
            path: "./my_contract".to_string(),
            constructor: Some("new".to_string()),
            args: Some("100".to_string()),
            value: Some("1000".to_string()),
            execute: Some(true),
            suri: Some("//Alice".to_string()),
            url: Some("ws://localhost:9944".to_string()),
        };
        let args = build_deploy_contract_args(&params, None);

        assert!(args.contains(&"--constructor"));
        assert!(args.contains(&"new"));
        assert!(args.contains(&"--args"));
        assert!(args.contains(&"100"));
        assert!(args.contains(&"--value"));
        assert!(args.contains(&"1000"));
        assert!(args.contains(&"--execute"));
        assert!(args.contains(&"--suri"));
        assert!(args.contains(&"//Alice"));
        assert!(args.contains(&"--url"));
        assert!(args.contains(&"ws://localhost:9944"));
    }

    #[test]
    fn test_build_args_url_fallback() {
        let params = DeployContractParams {
            path: "./my_contract".to_string(),
            constructor: None,
            args: None,
            value: None,
            execute: None,
            suri: None,
            url: None,
        };
        // Stored URL is used when no explicit URL
        let args = build_deploy_contract_args(&params, Some("ws://stored:9944"));
        assert!(args.contains(&"ws://stored:9944"));

        // Explicit URL overrides stored
        let params_with_url = DeployContractParams {
            url: Some("ws://explicit:9944".to_string()),
            ..params
        };
        let args = build_deploy_contract_args(&params_with_url, Some("ws://stored:9944"));
        assert!(args.contains(&"ws://explicit:9944"));
        assert!(!args.contains(&"ws://stored:9944"));
    }

    #[test]
    fn test_deploy_nonexistent_path() {
        let executor = PopExecutor::new();
        let params = DeployContractParams {
            path: "/nonexistent/path/to/contract".to_string(),
            constructor: None,
            args: None,
            value: None,
            execute: None,
            suri: None,
            url: None,
        };

        let result = deploy_contract(&executor, params, None).unwrap();
        assert!(result.is_error.unwrap_or(false));

        let text = content_text(&result);
        assert!(text.contains("Deployment failed"));
    }
}
