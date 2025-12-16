use crate::common::{is_error, is_success, pop_executor, text, Contract};
use anyhow::Result;
use pop_mcp_server::tools::test::contract::{test_contract, TestContractParams};
use serial_test::serial;

#[test]
#[serial]
fn contract_success_and_e2e() -> Result<()> {
    let executor = pop_executor()?;
    let contract = Contract::new(&executor, "test_contract_e2e")?;

    let params = TestContractParams {
        path: contract.path.to_string_lossy().to_string(),
        e2e: false,
    };
    let result = test_contract(&executor, params)?;
    assert!(is_success(&result));
    assert!(text(&result)?.contains("Tests completed!"));

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
fn contract_nonexistent_path() -> Result<()> {
    let executor = pop_executor()?;
    let params = TestContractParams {
        path: "/nonexistent/path/to/contract".to_string(),
        e2e: false,
    };

    let result = test_contract(&executor, params)?;
    assert!(is_error(&result));
    assert!(text(&result)?.contains("Tests failed"));
    Ok(())
}
