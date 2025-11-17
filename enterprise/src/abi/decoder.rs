// ABI Decoder - Decodes contract return values from bytes

use crate::error::{EnterpriseError, Result};
use super::types::{AbiFunction, AbiType};
use serde_json::{json, Value as JsonValue};

pub type DecodedValue = JsonValue;

pub struct AbiDecoder;

impl AbiDecoder {
    pub fn new() -> Self {
        Self
    }

    /// Decode a function return value
    pub fn decode_function_return(&self, function: &AbiFunction, data: &[u8]) -> Result<JsonValue> {
        if function.outputs.is_empty() {
            return Ok(json!(null));
        }

        if function.outputs.len() == 1 {
            // Single return value
            let (value, _) = self.decode_value(&function.outputs[0].param_type, data, 0)?;
            return Ok(value);
        }

        // Multiple return values - return as object
        let mut result = serde_json::Map::new();
        let mut offset = 0;

        for output in &function.outputs {
            let (value, new_offset) = self.decode_value(&output.param_type, data, offset)?;
            result.insert(output.name.clone(), value);
            offset = new_offset;
        }

        Ok(JsonValue::Object(result))
    }

    /// Decode a single value, returning (value, next_offset)
    fn decode_value(&self, param_type: &AbiType, data: &[u8], offset: usize) -> Result<(JsonValue, usize)> {
        match param_type {
            AbiType::Bool => self.decode_bool(data, offset),
            AbiType::Uint8 => self.decode_uint8(data, offset),
            AbiType::Uint16 => self.decode_uint16(data, offset),
            AbiType::Uint32 => self.decode_uint32(data, offset),
            AbiType::Uint64 => self.decode_uint64(data, offset),
            AbiType::Uint128 => self.decode_uint128(data, offset),
            AbiType::Uint256 => self.decode_uint256(data, offset),
            AbiType::Int8 => self.decode_int8(data, offset),
            AbiType::Int16 => self.decode_int16(data, offset),
            AbiType::Int32 => self.decode_int32(data, offset),
            AbiType::Int64 => self.decode_int64(data, offset),
            AbiType::FixedBytes(n) => self.decode_fixed_bytes(data, offset, *n),
            AbiType::Bytes => self.decode_bytes(data, offset),
            AbiType::String => self.decode_string(data, offset),
            AbiType::Address => self.decode_address(data, offset),
            AbiType::Array(inner) => self.decode_array(inner, data, offset),
            AbiType::FixedArray(inner, size) => self.decode_fixed_array(inner, *size, data, offset),
            AbiType::Tuple(types) => self.decode_tuple(types, data, offset),
        }
    }

    fn decode_bool(&self, data: &[u8], offset: usize) -> Result<(JsonValue, usize)> {
        if offset >= data.len() {
            return Err(EnterpriseError::ValidationError("Insufficient data for bool".to_string()));
        }
        Ok((json!(data[offset] != 0), offset + 1))
    }

    fn decode_uint8(&self, data: &[u8], offset: usize) -> Result<(JsonValue, usize)> {
        if offset >= data.len() {
            return Err(EnterpriseError::ValidationError("Insufficient data for uint8".to_string()));
        }
        Ok((json!(data[offset]), offset + 1))
    }

    fn decode_uint16(&self, data: &[u8], offset: usize) -> Result<(JsonValue, usize)> {
        if offset + 2 > data.len() {
            return Err(EnterpriseError::ValidationError("Insufficient data for uint16".to_string()));
        }
        let value = u16::from_le_bytes([data[offset], data[offset + 1]]);
        Ok((json!(value), offset + 2))
    }

    fn decode_uint32(&self, data: &[u8], offset: usize) -> Result<(JsonValue, usize)> {
        if offset + 4 > data.len() {
            return Err(EnterpriseError::ValidationError("Insufficient data for uint32".to_string()));
        }
        let bytes: [u8; 4] = data[offset..offset + 4].try_into().unwrap();
        let value = u32::from_le_bytes(bytes);
        Ok((json!(value), offset + 4))
    }

    fn decode_uint64(&self, data: &[u8], offset: usize) -> Result<(JsonValue, usize)> {
        if offset + 8 > data.len() {
            return Err(EnterpriseError::ValidationError("Insufficient data for uint64".to_string()));
        }
        let bytes: [u8; 8] = data[offset..offset + 8].try_into().unwrap();
        let value = u64::from_le_bytes(bytes);
        Ok((json!(value), offset + 8))
    }

    fn decode_uint128(&self, data: &[u8], offset: usize) -> Result<(JsonValue, usize)> {
        if offset + 16 > data.len() {
            return Err(EnterpriseError::ValidationError("Insufficient data for uint128".to_string()));
        }
        let hex = format!("0x{}", hex::encode(&data[offset..offset + 16]));
        Ok((json!(hex), offset + 16))
    }

    fn decode_uint256(&self, data: &[u8], offset: usize) -> Result<(JsonValue, usize)> {
        if offset + 32 > data.len() {
            return Err(EnterpriseError::ValidationError("Insufficient data for uint256".to_string()));
        }
        let hex = format!("0x{}", hex::encode(&data[offset..offset + 32]));
        Ok((json!(hex), offset + 32))
    }

    fn decode_int8(&self, data: &[u8], offset: usize) -> Result<(JsonValue, usize)> {
        if offset >= data.len() {
            return Err(EnterpriseError::ValidationError("Insufficient data for int8".to_string()));
        }
        Ok((json!(data[offset] as i8), offset + 1))
    }

    fn decode_int16(&self, data: &[u8], offset: usize) -> Result<(JsonValue, usize)> {
        if offset + 2 > data.len() {
            return Err(EnterpriseError::ValidationError("Insufficient data for int16".to_string()));
        }
        let value = i16::from_le_bytes([data[offset], data[offset + 1]]);
        Ok((json!(value), offset + 2))
    }

    fn decode_int32(&self, data: &[u8], offset: usize) -> Result<(JsonValue, usize)> {
        if offset + 4 > data.len() {
            return Err(EnterpriseError::ValidationError("Insufficient data for int32".to_string()));
        }
        let bytes: [u8; 4] = data[offset..offset + 4].try_into().unwrap();
        let value = i32::from_le_bytes(bytes);
        Ok((json!(value), offset + 4))
    }

    fn decode_int64(&self, data: &[u8], offset: usize) -> Result<(JsonValue, usize)> {
        if offset + 8 > data.len() {
            return Err(EnterpriseError::ValidationError("Insufficient data for int64".to_string()));
        }
        let bytes: [u8; 8] = data[offset..offset + 8].try_into().unwrap();
        let value = i64::from_le_bytes(bytes);
        Ok((json!(value), offset + 8))
    }

    fn decode_fixed_bytes(&self, data: &[u8], offset: usize, size: usize) -> Result<(JsonValue, usize)> {
        if offset + size > data.len() {
            return Err(EnterpriseError::ValidationError("Insufficient data for fixed bytes".to_string()));
        }
        let hex = format!("0x{}", hex::encode(&data[offset..offset + size]));
        Ok((json!(hex), offset + size))
    }

    fn decode_bytes(&self, data: &[u8], offset: usize) -> Result<(JsonValue, usize)> {
        if offset + 4 > data.len() {
            return Err(EnterpriseError::ValidationError("Insufficient data for bytes length".to_string()));
        }
        let len_bytes: [u8; 4] = data[offset..offset + 4].try_into().unwrap();
        let len = u32::from_le_bytes(len_bytes) as usize;

        let data_start = offset + 4;
        if data_start + len > data.len() {
            return Err(EnterpriseError::ValidationError("Insufficient data for bytes content".to_string()));
        }

        let hex = format!("0x{}", hex::encode(&data[data_start..data_start + len]));
        Ok((json!(hex), data_start + len))
    }

    fn decode_string(&self, data: &[u8], offset: usize) -> Result<(JsonValue, usize)> {
        if offset + 4 > data.len() {
            return Err(EnterpriseError::ValidationError("Insufficient data for string length".to_string()));
        }
        let len_bytes: [u8; 4] = data[offset..offset + 4].try_into().unwrap();
        let len = u32::from_le_bytes(len_bytes) as usize;

        let data_start = offset + 4;
        if data_start + len > data.len() {
            return Err(EnterpriseError::ValidationError("Insufficient data for string content".to_string()));
        }

        let s = String::from_utf8(data[data_start..data_start + len].to_vec())
            .map_err(|_| EnterpriseError::ValidationError("Invalid UTF-8 in string".to_string()))?;

        Ok((json!(s), data_start + len))
    }

    fn decode_address(&self, data: &[u8], offset: usize) -> Result<(JsonValue, usize)> {
        self.decode_fixed_bytes(data, offset, 32)
    }

    fn decode_array(&self, inner_type: &AbiType, data: &[u8], offset: usize) -> Result<(JsonValue, usize)> {
        if offset + 4 > data.len() {
            return Err(EnterpriseError::ValidationError("Insufficient data for array length".to_string()));
        }
        let len_bytes: [u8; 4] = data[offset..offset + 4].try_into().unwrap();
        let len = u32::from_le_bytes(len_bytes) as usize;

        let mut arr = Vec::new();
        let mut current_offset = offset + 4;

        for _ in 0..len {
            let (value, new_offset) = self.decode_value(inner_type, data, current_offset)?;
            arr.push(value);
            current_offset = new_offset;
        }

        Ok((json!(arr), current_offset))
    }

    fn decode_fixed_array(&self, inner_type: &AbiType, size: usize, data: &[u8], offset: usize) -> Result<(JsonValue, usize)> {
        let mut arr = Vec::new();
        let mut current_offset = offset;

        for _ in 0..size {
            let (value, new_offset) = self.decode_value(inner_type, data, current_offset)?;
            arr.push(value);
            current_offset = new_offset;
        }

        Ok((json!(arr), current_offset))
    }

    fn decode_tuple(&self, types: &[AbiType], data: &[u8], offset: usize) -> Result<(JsonValue, usize)> {
        let mut arr = Vec::new();
        let mut current_offset = offset;

        for elem_type in types {
            let (value, new_offset) = self.decode_value(elem_type, data, current_offset)?;
            arr.push(value);
            current_offset = new_offset;
        }

        Ok((json!(arr), current_offset))
    }
}

impl Default for AbiDecoder {
    fn default() -> Self {
        Self::new()
    }
}
