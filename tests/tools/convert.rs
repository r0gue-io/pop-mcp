use crate::common::{content_text, pop_executor};
use pop_mcp_server::tools::convert::{convert_address, ConvertAddressParams};

#[test]
fn convert_ethereum_to_substrate() {
    let executor = pop_executor();
    let params = ConvertAddressParams {
        address: "0x742d35Cc6634C0532925a3b844Bc454e4438f44e".to_string(),
    };

    let result = convert_address(&executor, params).unwrap();
    assert_eq!(result.is_error, Some(false));
    assert!(!content_text(&result).is_empty());
}

#[test]
fn convert_substrate_to_ethereum() {
    let executor = pop_executor();
    let params = ConvertAddressParams {
        address: "13dKz82CEiU7fKfhfQ5aLpdbXHApLfJH5Z6y2RTZpRwKiNhX".to_string(),
    };

    let result = convert_address(&executor, params).unwrap();
    assert_eq!(result.is_error, Some(false));
    assert!(content_text(&result)
        .to_lowercase()
        .contains("0x742d35cc6634c0532925a3b844bc454e4438f44e"));
}

#[test]
fn convert_invalid_address() {
    let executor = pop_executor();
    let params = ConvertAddressParams {
        address: "not_a_valid_address".to_string(),
    };

    let result = convert_address(&executor, params).unwrap();
    assert!(result.is_error.unwrap_or(false));
    assert!(content_text(&result).contains("Address conversion failed"));
}
