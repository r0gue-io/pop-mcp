use crate::common::{is_success, pop_executor};
use anyhow::Result;
use pop_mcp_server::tools::install::{check_pop_installation, CheckPopInstallationParams};

#[test]
fn check_pop_installation_succeeds() -> Result<()> {
    let executor = pop_executor()?;
    let result = check_pop_installation(&executor, CheckPopInstallationParams {})?;
    assert!(is_success(&result));
    Ok(())
}
