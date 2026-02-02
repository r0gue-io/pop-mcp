//! Contract creation (pop new contract)

use rmcp::model::CallToolResult;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::process::Command;

use crate::error::PopMcpResult;
use crate::executor::PopExecutor;
use crate::tools::common::{error_result, success_result};

/// Parameters for the list_templates tool.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[schemars(extend("properties" = {}))]
#[allow(clippy::empty_structs_with_brackets)]
pub(crate) struct ListTemplatesParams {}

/// List available ink! contract templates.
pub(crate) fn list_templates(_params: ListTemplatesParams) -> PopMcpResult<CallToolResult> {
    let templates = "\
Available ink! Contract Templates:\n\n\
1. **standard** - Basic flipper contract (boolean toggle)\n\
2. **erc20** - ERC20 fungible token implementation\n\
3. **erc721** - ERC721 NFT implementation\n\
4. **erc1155** - ERC1155 multi-token implementation\n\
5. **dns** - Domain Name Service contract\n\
6. **cross-contract-calls** - Example of calling other contracts\n\
7. **multisig** - Multi-signature wallet contract";

    Ok(success_result(templates))
}

/// Parameters for the create_contract tool.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[schemars(extend("properties" = {}))]
pub struct CreateContractParams {
    /// Name of the contract project.
    #[schemars(
        description = "Name of the contract project (alphanumeric characters and underscores only)"
    )]
    pub name: String,
    /// Template to use for the contract.
    #[schemars(
        description = "Template to use (standard, erc20, erc721, erc1155, dns, cross-contract-calls, multisig)"
    )]
    pub template: String,
    /// Whether to scaffold a frontend using the typink template.
    #[schemars(description = "Scaffold a typink frontend alongside the contract")]
    pub with_frontend: Option<bool>,
}

impl CreateContractParams {
    /// Validate the contract name
    fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Contract name cannot be empty".to_owned());
        }
        if !self.name.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(
                "Contract names can only contain alphanumeric characters and underscores"
                    .to_owned(),
            );
        }
        Ok(())
    }
}

/// Build command arguments for create_contract
fn build_create_contract_args(params: &CreateContractParams) -> Vec<&str> {
    let mut args = vec![
        "new",
        "contract",
        params.name.as_str(),
        "--template",
        params.template.as_str(),
    ];
    if params.with_frontend == Some(true) {
        args.push("--with-frontend=typink");
        args.push("--package-manager");
        args.push("npm");
    }
    args
}

/// Execute create_contract tool
pub fn create_contract(
    executor: &PopExecutor,
    params: CreateContractParams,
) -> PopMcpResult<CallToolResult> {
    // Validate parameters
    params
        .validate()
        .map_err(crate::error::PopMcpError::InvalidInput)?;

    if params.with_frontend == Some(true) {
        if let Err(message) = validate_frontend_requirements() {
            return Ok(error_result(message));
        }
    }

    let args = build_create_contract_args(&params);

    match executor.execute(&args) {
        Ok(_) => {
            let message = if params.with_frontend == Some(true) {
                format!(
                    "Successfully created contract with typink frontend: {}",
                    params.name
                )
            } else {
                format!("Successfully created contract: {}", params.name)
            };
            Ok(success_result(message))
        }
        Err(e) => Ok(error_result(format!("Failed to create contract: {}", e))),
    }
}

fn validate_frontend_requirements() -> Result<(), String> {
    let node_major = node_major_version()?;
    if node_major < 20 {
        return Err(format!(
            "with_frontend requires Node.js v20+ (detected v{}). Install Node.js v20+ and try again.",
            node_major
        ));
    }
    if !has_npm() {
        return Err("with_frontend requires npm available on PATH.".to_owned());
    }
    Ok(())
}

fn node_major_version() -> Result<u32, String> {
    let output = Command::new("node")
        .arg("--version")
        .output()
        .map_err(|_| {
            "with_frontend requires Node.js v20+ installed. Install Node.js v20+ and try again."
                .to_owned()
        })?;

    if !output.status.success() {
        return Err(
            "with_frontend requires Node.js v20+ installed. Install Node.js v20+ and try again."
                .to_owned(),
        );
    }

    let version = String::from_utf8(output.stdout)
        .map_err(|_| "Failed to parse Node.js version output.".to_owned())?;
    let version = version.trim();
    let version = version.strip_prefix('v').unwrap_or(version);
    let major = version
        .split('.')
        .next()
        .ok_or_else(|| "Failed to parse Node.js version output.".to_owned())?;
    major
        .parse::<u32>()
        .map_err(|_| "Failed to parse Node.js major version.".to_owned())
}

fn has_npm() -> bool {
    Command::new("npm")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

// Frontend creation temporarily disabled.
/*
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct CreateContractWithFrontendParams {
    #[schemars(
        description = "Name of the contract project (alphanumeric characters and underscores only)"
    )]
    pub name: String,
    #[schemars(
        description = "Template to use (standard, erc20, erc721, erc1155, dns, cross-contract-calls, multisig)"
    )]
    pub template: String,
}

impl CreateContractWithFrontendParams {
    /// Validate the contract name
    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Contract name cannot be empty".to_string());
        }
        if !self.name.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(
                "Contract names can only contain alphanumeric characters and underscores"
                    .to_string(),
            );
        }
        Ok(())
    }
}


/// Build command arguments for create_contract_with_frontend
pub fn build_create_contract_with_frontend_args(
    params: &CreateContractWithFrontendParams,
) -> Vec<String> {
    vec![
        "new".to_string(),
        "contract".to_string(),
        params.name.clone(),
        "--template".to_string(),
        params.template.clone(),
        "--with-frontend=typink".to_string(),
    ]
}

/// Execute create_contract_with_frontend tool
pub async fn create_contract_with_frontend<E: CommandExecutor>(
    executor: &E,
    params: CreateContractWithFrontendParams,
) -> PopMcpResult<CallToolResult> {
    // Validate parameters
    params
        .validate()
        .map_err(|e| crate::error::PopMcpError::InvalidInput(e))?;

    let args = build_create_contract_with_frontend_args(&params);
    let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

    let create_result = match executor.execute(&args_refs) {
        Ok(output) => output,
        Err(e) => {
            return Ok(error_result(format!(
                "Failed to create contract with frontend: {}",
                e
            )))
        }
    };

    // Adapt frontend to contract template
    let contract_path = &params.name;
    let frontend_path = format!("{}/frontend", contract_path);

    let adaptation_result =
        adapt_frontend_to_contract(executor, &params.template, &frontend_path, &params.name).await;

    match adaptation_result {
        Ok(adaptation_msg) => Ok(success_result(format!(
            "Successfully created contract with Dedot frontend: {}\n\n{}\n\n{}",
            params.name, create_result, adaptation_msg
        ))),
        Err(e) => Ok(success_result(format!(
            "Contract created: {}\n\n{}\n\nNote: Frontend adaptation encountered an issue: {}\n\
            Please refer to the Dedot documentation resource (dedot://docs/full-guide) for manual adaptation instructions.",
            params.name, create_result, e
        ))),
    }
}

/// Adapt frontend to contract template
async fn adapt_frontend_to_contract<E: CommandExecutor>(
    executor: &E,
    template: &str,
    frontend_path: &str,
    contract_name: &str,
) -> Result<String, String> {
    // Fetch Dedot documentation
    let _dedot_docs = match crate::resources::read_resource("dedot://docs/full-guide").await {
        Ok(result) => {
            if result.contents.first().is_some() {
                "Available"
            } else {
                return Err("Failed to read Dedot documentation".to_string());
            }
        }
        Err(e) => return Err(format!("Failed to fetch Dedot documentation: {}", e)),
    };

    // Build the contract first to generate metadata
    let contract_base_path = frontend_path.replace("/frontend", "");
    let metadata_path = format!("{}/target/ink/{}.json", contract_base_path, contract_name);

    let build_args = vec!["build", "--path", &contract_base_path];
    let _ = executor.execute(&build_args);

    // Read the generated metadata
    let metadata_content = match std::fs::read_to_string(&metadata_path) {
        Ok(content) => content,
        Err(_) => {
            return Ok(format!(
                "Frontend created with default flipper template.\n\
                To adapt it to your {} contract:\n\
                1. Build your contract first: `pop build --path {}`\n\
                2. The contract metadata will be at: {}\n\
                3. Use Dedot's typink to generate types: `npx dedot typink -m {} -o frontend/src/contracts`\n\
                4. Update frontend/src/app/page.tsx to use your contract's methods\n\n\
                Refer to Dedot documentation for detailed instructions on contract integration.",
                template,
                frontend_path.replace("/frontend", ""),
                metadata_path,
                metadata_path
            ))
        }
    };

    // Parse metadata to extract contract methods
    let metadata: serde_json::Value = match serde_json::from_str(&metadata_content) {
        Ok(v) => v,
        Err(e) => return Err(format!("Failed to parse contract metadata: {}", e)),
    };

    let spec = metadata.get("spec").ok_or("No spec in metadata")?;
    let messages = spec
        .get("messages")
        .and_then(|m| m.as_array())
        .ok_or("No messages in spec")?;

    let mut methods_list = String::new();
    for msg in messages {
        if let Some(label) = msg.get("label").and_then(|l| l.as_str()) {
            methods_list.push_str(&format!("  - {}\n", label));
        }
    }

    Ok(format!(
        "Frontend adapted for {} template (contract: {})\n\n\
        Contract methods detected:\n{}\n\
        Generated TypeScript types using Dedot's typink\n\
        Next steps:\n\
        1. Navigate to frontend: `cd {}`\n\
        2. Install dependencies: `npm install`\n\
        3. Update src/app/page.tsx to use the contract methods above\n\
        4. The contract types are available in src/contracts/\n\n\
        Dedot documentation reference: https://docs.dedot.dev/smart-contracts",
        template, contract_name, methods_list, frontend_path
    ))
}
*/

#[cfg(test)]
#[allow(clippy::panic)]
mod tests {
    use super::*;
    use crate::tools::common::content_text;

    #[test]
    fn validate_allows_valid_names() {
        for name in ["mytoken123", "my_token_v2"] {
            let params = CreateContractParams {
                name: (*name).to_owned(),
                template: "standard".to_owned(),
                with_frontend: None,
            };
            assert!(params.validate().is_ok());
        }
    }

    #[test]
    fn validate_rejects_invalid_names() {
        for name in [
            "", "my-token", "my token", "my@token", "my#token", "my.token",
        ] {
            let params = CreateContractParams {
                name: (*name).to_owned(),
                template: "standard".to_owned(),
                with_frontend: None,
            };
            assert!(params.validate().is_err());
        }
    }

    #[test]
    fn build_args_include_template() {
        let params = CreateContractParams {
            name: "my_contract".to_owned(),
            template: "erc20".to_owned(),
            with_frontend: None,
        };
        let args = build_create_contract_args(&params);
        assert_eq!(
            args,
            vec!["new", "contract", "my_contract", "--template", "erc20"]
        );
    }

    #[test]
    fn build_args_include_frontend_when_true() {
        let params = CreateContractParams {
            name: "my_contract".to_owned(),
            template: "standard".to_owned(),
            with_frontend: Some(true),
        };
        let args = build_create_contract_args(&params);
        assert_eq!(
            args,
            vec![
                "new",
                "contract",
                "my_contract",
                "--template",
                "standard",
                "--with-frontend=typink",
                "--package-manager",
                "npm"
            ]
        );
    }

    #[test]
    fn build_args_exclude_frontend_when_false() {
        let params = CreateContractParams {
            name: "my_contract".to_owned(),
            template: "standard".to_owned(),
            with_frontend: Some(false),
        };
        let args = build_create_contract_args(&params);
        assert_eq!(
            args,
            vec!["new", "contract", "my_contract", "--template", "standard"]
        );
    }

    #[test]
    fn list_templates_includes_known_entries() {
        let Ok(result) = list_templates(ListTemplatesParams {}) else {
            panic!("Expected Ok result");
        };
        assert!(!result.is_error.unwrap_or(true));
        let text = content_text(&result);
        for expected in [
            "standard",
            "erc20",
            "erc721",
            "erc1155",
            "dns",
            "cross-contract-calls",
            "multisig",
        ] {
            assert!(text.contains(expected));
        }
    }
}
