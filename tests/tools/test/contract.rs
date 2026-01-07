use crate::common::{is_error, is_success, text, Contract, TestContext};
use anyhow::Result;
use pop_mcp_server::tools::test::contract::{test_contract, TestContractParams};

#[test]
fn test_contract_unit_and_e2e_both_pass() -> Result<()> {
    let ctx = TestContext::new()?;
    let executor = ctx.executor()?;
    let contract = Contract::with_context(ctx, &executor, "test_contract_e2e")?;

    // Run unit tests
    let params = TestContractParams {
        path: contract.path.to_string_lossy().to_string(),
        e2e: false,
    };
    let result = test_contract(&executor, params)?;
    assert!(is_success(&result));
    assert!(text(&result)?.contains("Tests completed!"));

    // Run e2e tests
    let params_e2e = TestContractParams {
        path: contract.path.to_string_lossy().to_string(),
        e2e: true,
    };
    let result_e2e = test_contract(&executor, params_e2e)?;
    assert!(is_success(&result_e2e));
    assert!(text(&result_e2e)?.contains("Tests completed!"));
    Ok(())
}

#[test]
fn test_contract_nonexistent_path_fails() -> Result<()> {
    let ctx = TestContext::new()?;
    let executor = ctx.executor()?;
    let params = TestContractParams {
        path: "/nonexistent/path/to/contract".to_string(),
        e2e: false,
    };

    let result = test_contract(&executor, params)?;
    assert!(is_error(&result));
    assert!(text(&result)?.contains("Tests failed"));
    Ok(())
}
