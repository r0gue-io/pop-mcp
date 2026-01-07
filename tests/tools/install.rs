use crate::common::{is_success, TestEnv};
use anyhow::Result;
use pop_mcp_server::tools::install::{check_pop_installation, CheckPopInstallationParams};

#[test]
fn check_pop_installation_succeeds() -> Result<()> {
    let env = TestEnv::new()?;
    let result = check_pop_installation(env.executor(), CheckPopInstallationParams {})?;
    assert!(is_success(&result));
    Ok(())
}
