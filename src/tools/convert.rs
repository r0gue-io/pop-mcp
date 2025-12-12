//! Address conversion tool (pop convert address)

use rmcp::model::CallToolResult;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::PopMcpResult;
use crate::executor::CommandExecutor;

use super::helpers::{error_result, success_result};

// Parameters

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ConvertAddressParams {
    #[schemars(
        description = "The Substrate or Ethereum address to convert (supports SS58 format or raw 32-byte hex)"
    )]
    pub address: String,
}

impl ConvertAddressParams {
    /// Validate the address parameter
    pub fn validate(&self) -> Result<(), String> {
        if self.address.is_empty() {
            return Err("Address cannot be empty".to_string());
        }
        Ok(())
    }
}

/// Build command arguments for convert_address
pub fn build_convert_address_args(params: &ConvertAddressParams) -> [&str; 3] {
    ["convert", "address", params.address.as_str()]
}

/// Execute convert_address tool
pub fn convert_address<E: CommandExecutor>(
    executor: &E,
    params: ConvertAddressParams,
) -> PopMcpResult<CallToolResult> {
    // Validate parameters
    params
        .validate()
        .map_err(|e| crate::error::PopMcpError::InvalidInput(e))?;

    let args = build_convert_address_args(&params);

    match executor.execute(&args) {
        Ok(output) => Ok(success_result(output)),
        Err(e) => Ok(error_result(format!("Address conversion failed:\n\n{}", e))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::test_utils::MockExecutor;
    use crate::executor::PopExecutor;
    use rmcp::model::RawContent;

    fn pop_available(executor: &PopExecutor) -> bool {
        executor.execute(&["--version"]).is_ok()
    }

    fn content_text(result: &rmcp::model::CallToolResult) -> String {
        result
            .content
            .last()
            .and_then(|c| match &c.raw {
                RawContent::Text(t) => Some(t.text.clone()),
                _ => None,
            })
            .unwrap_or_default()
    }

    #[test]
    fn test_convert_empty_address_fails_before_execution() {
        let executor = MockExecutor::success("Should not reach here");
        let params = ConvertAddressParams {
            address: "".to_string(),
        };
        assert!(convert_address(&executor, params).is_err());
    }

    #[test]
    fn test_convert_ethereum_to_substrate() {
        let executor = PopExecutor::new();
        if !pop_available(&executor) {
            return;
        }

        let params = ConvertAddressParams {
            address: "0x742d35Cc6634C0532925a3b844Bc454e4438f44e".to_string(),
        };

        let result = convert_address(&executor, params).unwrap();
        assert!(!result.is_error.unwrap_or(true));
        assert!(!content_text(&result).is_empty());
    }

    #[test]
    fn test_convert_substrate_to_ethereum() {
        let executor = PopExecutor::new();
        if !pop_available(&executor) {
            return;
        }

        // Substrate address derived from 0x742d35Cc6634C0532925a3b844Bc454e4438f44e
        let params = ConvertAddressParams {
            address: "13dKz82CEiU7fKfhfQ5aLpdbXHApLfJH5Z6y2RTZpRwKiNhX".to_string(),
        };

        let result = convert_address(&executor, params).unwrap();
        assert!(!result.is_error.unwrap_or(true));
        assert!(content_text(&result)
            .to_lowercase()
            .contains("0x742d35cc6634c0532925a3b844bc454e4438f44e"));
    }

    #[test]
    fn test_convert_invalid_address() {
        let executor = PopExecutor::new();
        if !pop_available(&executor) {
            return;
        }

        let params = ConvertAddressParams {
            address: "not_a_valid_address".to_string(),
        };

        let result = convert_address(&executor, params).unwrap();
        assert!(result.is_error.unwrap_or(false));
        assert!(content_text(&result).contains("Address conversion failed"));
    }
}
