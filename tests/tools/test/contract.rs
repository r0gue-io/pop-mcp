use crate::common::{is_error, is_success, text, Contract, TestEnv};
use anyhow::Result;
use pop_mcp_server::tools::test::contract::{test_contract, TestContractParams};

#[test]
fn test_contract_unit_and_e2e_both_pass() -> Result<()> {
    let env = TestEnv::new()?;
    let contract = Contract::create_build_or_use()?;

    // Run unit tests
    let result = test_contract(
        env.executor(),
        TestContractParams {
            path: contract.path.display().to_string(),
            e2e: false,
        },
    )?;
    assert!(is_success(&result));
    assert!(text(&result)?.contains("Tests completed!"));

    // Run e2e tests
    let result_e2e = test_contract(
        env.executor(),
        TestContractParams {
            path: contract.path.display().to_string(),
            e2e: true,
        },
    )?;
    assert!(is_success(&result_e2e));
    assert!(text(&result_e2e)?.contains("Tests completed!"));
    Ok(())
}

#[test]
fn test_contract_nonexistent_path_fails() -> Result<()> {
    let env = TestEnv::new()?;
    let params = TestContractParams {
        path: "/nonexistent/path/to/contract".to_string(),
        e2e: false,
    };

    let result = test_contract(env.executor(), params)?;
    assert!(is_error(&result));
    assert!(text(&result)?.contains("Tests failed"));
    Ok(())
}
