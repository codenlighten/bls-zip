// ABI Encoder - Encodes contract method calls into bytes
//
// Uses a simple packed encoding format for WASM contract compatibility.
// For a production system, consider using more sophisticated encoding like SCALE or Borsh.

use crate::error::{EnterpriseError, Result};
use super::types::{AbiFunction, AbiParam, AbiType};
use serde_json::Value as JsonValue;

pub struct AbiEncoder;

impl AbiEncoder {
    pub fn new() -> Self {
        Self
    }

    /// Encode a function call with parameters
    pub fn encode_function_call(&self, function: &AbiFunction, params: &JsonValue) -> Result<Vec<u8>> {
        let mut encoded = Vec::new();

        // Add function name with length prefix (WASM-compatible format)
        // Format: [2-byte name_len][function_name UTF-8]
        // This matches the ContractCallData::encode_for_wasm() format
        let function_name = function.name.as_bytes();
        let name_len = function_name.len() as u16;
        encoded.extend_from_slice(&name_len.to_le_bytes());  // 2-byte length
        encoded.extend_from_slice(function_name);             // UTF-8 name

        // Encode each parameter
        for param in &function.inputs {
            let value = params.get(&param.name)
                .ok_or_else(|| EnterpriseError::ValidationError(
                    format!("Missing parameter: {}", param.name)
                ))?;

            let param_bytes = self.encode_value(&param.param_type, value)?;
            encoded.extend_from_slice(&param_bytes);
        }

        Ok(encoded)
    }

    /// Encode a single value based on its type
    fn encode_value(&self, param_type: &AbiType, value: &JsonValue) -> Result<Vec<u8>> {
        match param_type {
            AbiType::Bool => self.encode_bool(value),
            AbiType::Uint8 => self.encode_uint8(value),
            AbiType::Uint16 => self.encode_uint16(value),
            AbiType::Uint32 => self.encode_uint32(value),
            AbiType::Uint64 => self.encode_uint64(value),
            AbiType::Uint128 => self.encode_uint128(value),
            AbiType::Uint256 => self.encode_uint256(value),
            AbiType::Int8 => self.encode_int8(value),
            AbiType::Int16 => self.encode_int16(value),
            AbiType::Int32 => self.encode_int32(value),
            AbiType::Int64 => self.encode_int64(value),
            AbiType::FixedBytes(n) => self.encode_fixed_bytes(value, *n),
            AbiType::Bytes => self.encode_bytes(value),
            AbiType::String => self.encode_string(value),
            AbiType::Address => self.encode_address(value),
            AbiType::Array(inner) => self.encode_array(inner, value),
            AbiType::FixedArray(inner, size) => self.encode_fixed_array(inner, *size, value),
            AbiType::Tuple(types) => self.encode_tuple(types, value),
        }
    }

    fn encode_bool(&self, value: &JsonValue) -> Result<Vec<u8>> {
        let b = value.as_bool()
            .ok_or_else(|| EnterpriseError::ValidationError("Expected boolean".to_string()))?;
        Ok(vec![if b { 1 } else { 0 }])
    }

    fn encode_uint8(&self, value: &JsonValue) -> Result<Vec<u8>> {
        let n = value.as_u64()
            .ok_or_else(|| EnterpriseError::ValidationError("Expected unsigned integer".to_string()))?;
        if n > u8::MAX as u64 {
            return Err(EnterpriseError::ValidationError("Value exceeds uint8 range".to_string()));
        }
        Ok(vec![n as u8])
    }

    fn encode_uint16(&self, value: &JsonValue) -> Result<Vec<u8>> {
        let n = value.as_u64()
            .ok_or_else(|| EnterpriseError::ValidationError("Expected unsigned integer".to_string()))?;
        if n > u16::MAX as u64 {
            return Err(EnterpriseError::ValidationError("Value exceeds uint16 range".to_string()));
        }
        Ok((n as u16).to_le_bytes().to_vec())
    }

    fn encode_uint32(&self, value: &JsonValue) -> Result<Vec<u8>> {
        let n = value.as_u64()
            .ok_or_else(|| EnterpriseError::ValidationError("Expected unsigned integer".to_string()))?;
        if n > u32::MAX as u64 {
            return Err(EnterpriseError::ValidationError("Value exceeds uint32 range".to_string()));
        }
        Ok((n as u32).to_le_bytes().to_vec())
    }

    fn encode_uint64(&self, value: &JsonValue) -> Result<Vec<u8>> {
        let n = value.as_u64()
            .ok_or_else(|| EnterpriseError::ValidationError("Expected unsigned integer".to_string()))?;
        Ok(n.to_le_bytes().to_vec())
    }

    fn encode_uint128(&self, value: &JsonValue) -> Result<Vec<u8>> {
        let s = value.as_str()
            .ok_or_else(|| EnterpriseError::ValidationError("Expected hex string for uint128".to_string()))?;
        let bytes = hex::decode(s.trim_start_matches("0x"))
            .map_err(|_| EnterpriseError::ValidationError("Invalid hex string".to_string()))?;
        if bytes.len() > 16 {
            return Err(EnterpriseError::ValidationError("Value exceeds uint128 range".to_string()));
        }
        let mut padded = vec![0u8; 16];
        padded[..bytes.len()].copy_from_slice(&bytes);
        Ok(padded)
    }

    fn encode_uint256(&self, value: &JsonValue) -> Result<Vec<u8>> {
        let s = value.as_str()
            .ok_or_else(|| EnterpriseError::ValidationError("Expected hex string for uint256".to_string()))?;
        let bytes = hex::decode(s.trim_start_matches("0x"))
            .map_err(|_| EnterpriseError::ValidationError("Invalid hex string".to_string()))?;
        if bytes.len() > 32 {
            return Err(EnterpriseError::ValidationError("Value exceeds uint256 range".to_string()));
        }
        let mut padded = vec![0u8; 32];
        padded[..bytes.len()].copy_from_slice(&bytes);
        Ok(padded)
    }

    fn encode_int8(&self, value: &JsonValue) -> Result<Vec<u8>> {
        let n = value.as_i64()
            .ok_or_else(|| EnterpriseError::ValidationError("Expected signed integer".to_string()))?;
        if n < i8::MIN as i64 || n > i8::MAX as i64 {
            return Err(EnterpriseError::ValidationError("Value exceeds int8 range".to_string()));
        }
        Ok(vec![(n as i8) as u8])
    }

    fn encode_int16(&self, value: &JsonValue) -> Result<Vec<u8>> {
        let n = value.as_i64()
            .ok_or_else(|| EnterpriseError::ValidationError("Expected signed integer".to_string()))?;
        if n < i16::MIN as i64 || n > i16::MAX as i64 {
            return Err(EnterpriseError::ValidationError("Value exceeds int16 range".to_string()));
        }
        Ok((n as i16).to_le_bytes().to_vec())
    }

    fn encode_int32(&self, value: &JsonValue) -> Result<Vec<u8>> {
        let n = value.as_i64()
            .ok_or_else(|| EnterpriseError::ValidationError("Expected signed integer".to_string()))?;
        if n < i32::MIN as i64 || n > i32::MAX as i64 {
            return Err(EnterpriseError::ValidationError("Value exceeds int32 range".to_string()));
        }
        Ok((n as i32).to_le_bytes().to_vec())
    }

    fn encode_int64(&self, value: &JsonValue) -> Result<Vec<u8>> {
        let n = value.as_i64()
            .ok_or_else(|| EnterpriseError::ValidationError("Expected signed integer".to_string()))?;
        Ok(n.to_le_bytes().to_vec())
    }

    fn encode_fixed_bytes(&self, value: &JsonValue, size: usize) -> Result<Vec<u8>> {
        let s = value.as_str()
            .ok_or_else(|| EnterpriseError::ValidationError("Expected hex string".to_string()))?;
        let bytes = hex::decode(s.trim_start_matches("0x"))
            .map_err(|_| EnterpriseError::ValidationError("Invalid hex string".to_string()))?;
        if bytes.len() != size {
            return Err(EnterpriseError::ValidationError(
                format!("Expected {} bytes, got {}", size, bytes.len())
            ));
        }
        Ok(bytes)
    }

    fn encode_bytes(&self, value: &JsonValue) -> Result<Vec<u8>> {
        let s = value.as_str()
            .ok_or_else(|| EnterpriseError::ValidationError("Expected hex string".to_string()))?;
        let bytes = hex::decode(s.trim_start_matches("0x"))
            .map_err(|_| EnterpriseError::ValidationError("Invalid hex string".to_string()))?;

        // Length prefix (4 bytes)
        let mut encoded = (bytes.len() as u32).to_le_bytes().to_vec();
        encoded.extend_from_slice(&bytes);
        Ok(encoded)
    }

    fn encode_string(&self, value: &JsonValue) -> Result<Vec<u8>> {
        let s = value.as_str()
            .ok_or_else(|| EnterpriseError::ValidationError("Expected string".to_string()))?;
        let bytes = s.as_bytes();

        // Length prefix (4 bytes)
        let mut encoded = (bytes.len() as u32).to_le_bytes().to_vec();
        encoded.extend_from_slice(bytes);
        Ok(encoded)
    }

    fn encode_address(&self, value: &JsonValue) -> Result<Vec<u8>> {
        self.encode_fixed_bytes(value, 32)
    }

    fn encode_array(&self, inner_type: &AbiType, value: &JsonValue) -> Result<Vec<u8>> {
        let arr = value.as_array()
            .ok_or_else(|| EnterpriseError::ValidationError("Expected array".to_string()))?;

        // Length prefix (4 bytes)
        let mut encoded = (arr.len() as u32).to_le_bytes().to_vec();

        // Encode each element
        for elem in arr {
            let elem_bytes = self.encode_value(inner_type, elem)?;
            encoded.extend_from_slice(&elem_bytes);
        }

        Ok(encoded)
    }

    fn encode_fixed_array(&self, inner_type: &AbiType, size: usize, value: &JsonValue) -> Result<Vec<u8>> {
        let arr = value.as_array()
            .ok_or_else(|| EnterpriseError::ValidationError("Expected array".to_string()))?;

        if arr.len() != size {
            return Err(EnterpriseError::ValidationError(
                format!("Expected array of size {}, got {}", size, arr.len())
            ));
        }

        let mut encoded = Vec::new();
        for elem in arr {
            let elem_bytes = self.encode_value(inner_type, elem)?;
            encoded.extend_from_slice(&elem_bytes);
        }

        Ok(encoded)
    }

    fn encode_tuple(&self, types: &[AbiType], value: &JsonValue) -> Result<Vec<u8>> {
        let arr = value.as_array()
            .ok_or_else(|| EnterpriseError::ValidationError("Expected array for tuple".to_string()))?;

        if arr.len() != types.len() {
            return Err(EnterpriseError::ValidationError(
                format!("Expected tuple of {} elements, got {}", types.len(), arr.len())
            ));
        }

        let mut encoded = Vec::new();
        for (elem_type, elem_value) in types.iter().zip(arr.iter()) {
            let elem_bytes = self.encode_value(elem_type, elem_value)?;
            encoded.extend_from_slice(&elem_bytes);
        }

        Ok(encoded)
    }
}

impl Default for AbiEncoder {
    fn default() -> Self {
        Self::new()
    }
}
