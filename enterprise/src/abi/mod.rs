// Contract ABI (Application Binary Interface) Infrastructure
//
// Provides encoding/decoding for smart contract method calls and return values.
// Uses a JSON-based ABI format similar to Ethereum, adapted for WASM contracts.

pub mod types;
pub mod encoder;
pub mod decoder;

pub use types::{AbiType, AbiParam, AbiFunction, ContractAbi};
pub use encoder::AbiEncoder;
pub use decoder::{AbiDecoder, DecodedValue};

use crate::error::{EnterpriseError, Result};
use serde_json::Value as JsonValue;

/// Encode a contract method call with parameters
///
/// # Arguments
/// * `function` - The ABI function definition
/// * `params` - JSON object containing parameter values
///
/// # Returns
/// Encoded bytes ready for contract execution
pub fn encode_call(function: &AbiFunction, params: &JsonValue) -> Result<Vec<u8>> {
    let encoder = AbiEncoder::new();
    encoder.encode_function_call(function, params)
}

/// Decode a contract method return value
///
/// # Arguments
/// * `function` - The ABI function definition
/// * `data` - Raw bytes returned from contract
///
/// # Returns
/// Decoded JSON value
pub fn decode_return(function: &AbiFunction, data: &[u8]) -> Result<JsonValue> {
    let decoder = AbiDecoder::new();
    decoder.decode_function_return(function, data)
}

/// Load contract ABI from JSON
pub fn load_abi_from_json(json_str: &str) -> Result<ContractAbi> {
    serde_json::from_str(json_str).map_err(|e| {
        EnterpriseError::ValidationError(format!("Invalid ABI JSON: {}", e))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_simple_call() {
        let function = AbiFunction {
            name: "transfer".to_string(),
            inputs: vec![
                AbiParam {
                    name: "to".to_string(),
                    param_type: AbiType::Address,
                },
                AbiParam {
                    name: "amount".to_string(),
                    param_type: AbiType::Uint64,
                },
            ],
            outputs: vec![AbiParam {
                name: "success".to_string(),
                param_type: AbiType::Bool,
            }],
            state_mutability: "nonpayable".to_string(),
        };

        let params = serde_json::json!({
            "to": "0x1234567890abcdef1234567890abcdef12345678",
            "amount": 1000
        });

        let encoded = encode_call(&function, &params).unwrap();
        assert!(!encoded.is_empty());
    }
}
