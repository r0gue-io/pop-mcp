use crate::common::{is_error, is_success, text, TestEnv};
use anyhow::Result;
use pop_mcp_server::tools::new::contract::{create_contract, CreateContractParams};
use std::process::Command;

#[test]
fn create_contract_standard_template_creates_files() -> Result<()> {
    let env = TestEnv::new()?;

    let contract_name = "test_contract";
    let contract_path = env.workdir().join(contract_name);

    let params = CreateContractParams {
        name: contract_name.to_string(),
        template: "standard".to_string(),
        with_frontend: None,
    };

    let result = create_contract(env.executor(), params)?;
    assert!(is_success(&result));

    let output = text(&result)?;
    assert!(output.starts_with(&format!("Successfully created contract: {}", contract_name)));
    assert!(contract_path.exists());
    assert!(contract_path.join("Cargo.toml").exists());
    assert!(contract_path.join("lib.rs").exists());
    Ok(())
}

#[test]
fn create_contract_invalid_name_with_hyphen_fails_validation() -> Result<()> {
    let env = TestEnv::new()?;
    let params = CreateContractParams {
        name: "invalid-name".to_string(),
        template: "standard".to_string(),
        with_frontend: None,
    };
    let result = create_contract(env.executor(), params);
    assert!(result.is_err());
    Ok(())
}

#[test]
fn create_contract_nonexistent_template_fails() -> Result<()> {
    let env = TestEnv::new()?;
    let params = CreateContractParams {
        name: "test_contract".to_string(),
        template: "non_existing".to_string(),
        with_frontend: None,
    };
    let result = create_contract(env.executor(), params)?;
    assert!(is_error(&result));
    assert!(text(&result)?.starts_with("Failed to create contract:"));
    Ok(())
}

#[test]
fn create_contract_with_frontend_creates_frontend_dir() -> Result<()> {
    let env = TestEnv::new()?;

    let contract_name = "frontend_test";
    let contract_path = env.workdir().join(contract_name);

    let params = CreateContractParams {
        name: contract_name.to_string(),
        template: "standard".to_string(),
        with_frontend: Some(true),
    };

    let result = create_contract(env.executor(), params)?;
    if !frontend_requirements_met() {
        assert!(is_error(&result));
        let message = text(&result)?;
        assert!(
            message.contains("with_frontend requires Node.js v20+")
                || message.contains("with_frontend requires a package manager")
        );
        return Ok(());
    }

    assert!(is_success(&result));
    assert!(contract_path.join("frontend").exists());
    Ok(())
}

fn frontend_requirements_met() -> bool {
    node_major_version().is_some_and(|major| major >= 20) && has_supported_package_manager()
}

fn node_major_version() -> Option<u32> {
    let output = Command::new("node").arg("--version").output().ok()?;
    if !output.status.success() {
        return None;
    }
    let version = String::from_utf8(output.stdout).ok()?;
    let version = version.trim();
    let version = version.strip_prefix('v').unwrap_or(version);
    version.split('.').next()?.parse::<u32>().ok()
}

fn has_supported_package_manager() -> bool {
    ["pnpm", "bun", "yarn", "npm"].iter().any(|bin| {
        Command::new(bin)
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    })
}
