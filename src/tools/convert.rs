//! Address conversion tool (pop convert address)

use rmcp::model::CallToolResult;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::PopMcpResult;
use crate::executor::PopExecutor;

use super::common::{error_result, success_result};

/// Parameters for the convert_address tool.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ConvertAddressParams {
    /// The address to convert (SS58 or Ethereum format).
    #[schemars(
        description = "The Substrate or Ethereum address to convert (supports SS58 format or raw 32-byte hex)"
    )]
    pub address: String,
}

impl ConvertAddressParams {
    /// Validate the address parameter
    pub fn validate(&self) -> Result<(), String> {
        if self.address.is_empty() {
            return Err("Address cannot be empty".to_owned());
        }
        Ok(())
    }
}

/// Build command arguments for convert_address
pub fn build_convert_address_args(params: &ConvertAddressParams) -> [&str; 3] {
    ["convert", "address", params.address.as_str()]
}

/// Execute convert_address tool
pub fn convert_address(
    executor: &PopExecutor,
    params: ConvertAddressParams,
) -> PopMcpResult<CallToolResult> {
    // Validate parameters
    params
        .validate()
        .map_err(crate::error::PopMcpError::InvalidInput)?;

    let args = build_convert_address_args(&params);

    match executor.execute(&args) {
        Ok(output) => Ok(success_result(output)),
        Err(e) => Ok(error_result(format!("Address conversion failed:\n\n{}", e))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_empty_address_fails_before_execution() {
        let params = ConvertAddressParams {
            address: String::new(),
        };
        assert!(params.validate().is_err());
    }
}
