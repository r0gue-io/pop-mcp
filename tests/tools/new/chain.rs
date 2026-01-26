use crate::common::{is_error, is_success, text, TestEnv};
use anyhow::Result;
use pop_mcp_server::tools::new::chain::{create_chain, CreateChainParams};

#[test]
fn create_chain_pop_standard_template_creates_files() -> Result<()> {
    let env = TestEnv::new()?;

    let chain_name = "test_chain";
    let chain_path = env.workdir().join(chain_name);

    let params = CreateChainParams {
        name: chain_name.to_string(),
        provider: "pop".to_string(),
        template: "r0gue-io/base-parachain".to_string(),
        symbol: Some("TEST".to_string()),
        decimals: Some(18),
    };

    let result = create_chain(env.executor(), params)?;
    assert!(is_success(&result));

    let output = text(&result)?;
    assert!(output.contains("Successfully created chain project:"));
    assert!(chain_path.exists());
    assert!(chain_path.join("Cargo.toml").exists());
    Ok(())
}

#[test]
fn create_chain_nonexistent_template_fails() -> Result<()> {
    let env = TestEnv::new()?;
    let params = CreateChainParams {
        name: "test_chain".to_string(),
        provider: "pop".to_string(),
        template: "r0gue-io/nonexistent-template".to_string(),
        symbol: None,
        decimals: None,
    };
    let result = create_chain(env.executor(), params)?;
    assert!(is_error(&result));
    assert!(text(&result)?.contains("Failed to create chain"));
    Ok(())
}
