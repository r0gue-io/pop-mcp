use crate::common::{is_error, is_success, pop_executor, text};
use anyhow::Result;
use pop_mcp_server::tools::convert::{convert_address, ConvertAddressParams};

#[test]
fn convert_ethereum_to_substrate() -> Result<()> {
    let executor = pop_executor()?;
    let params = ConvertAddressParams {
        address: "0x742d35Cc6634C0532925a3b844Bc454e4438f44e".to_string(),
    };

    let result = convert_address(&executor, params)?;
    assert!(is_success(&result));
    assert!(!text(&result)?.is_empty());
    Ok(())
}

#[test]
fn convert_substrate_to_ethereum() -> Result<()> {
    let executor = pop_executor()?;
    let params = ConvertAddressParams {
        address: "13dKz82CEiU7fKfhfQ5aLpdbXHApLfJH5Z6y2RTZpRwKiNhX".to_string(),
    };

    let result = convert_address(&executor, params)?;
    assert!(is_success(&result));
    assert!(text(&result)?
        .to_lowercase()
        .contains("0x742d35cc6634c0532925a3b844bc454e4438f44e"));
    Ok(())
}

#[test]
fn convert_invalid_address() -> Result<()> {
    let executor = pop_executor()?;
    let params = ConvertAddressParams {
        address: "not_a_valid_address".to_string(),
    };

    let result = convert_address(&executor, params)?;
    assert!(is_error(&result));
    assert!(text(&result)?.contains("Address conversion failed"));
    Ok(())
}
