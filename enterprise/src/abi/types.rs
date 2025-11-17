// ABI Type System
//
// Defines types for contract method parameters and return values.
// Based on common smart contract type systems (Ethereum ABI, WASM types).

use serde::{Deserialize, Serialize};

/// ABI type for contract parameters and return values
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AbiType {
    /// Boolean (true/false)
    Bool,

    /// Unsigned 8-bit integer (0-255)
    Uint8,

    /// Unsigned 16-bit integer
    Uint16,

    /// Unsigned 32-bit integer
    Uint32,

    /// Unsigned 64-bit integer
    Uint64,

    /// Unsigned 128-bit integer (as hex string)
    Uint128,

    /// Unsigned 256-bit integer (as hex string)
    Uint256,

    /// Signed 8-bit integer
    Int8,

    /// Signed 16-bit integer
    Int16,

    /// Signed 32-bit integer
    Int32,

    /// Signed 64-bit integer
    Int64,

    /// Fixed-size bytes (e.g., hash, signature)
    FixedBytes(usize),

    /// Variable-size bytes
    Bytes,

    /// UTF-8 string
    String,

    /// 32-byte address (hex string)
    Address,

    /// Array of specific type
    Array(Box<AbiType>),

    /// Fixed-size array
    FixedArray(Box<AbiType>, usize),

    /// Tuple of types
    Tuple(Vec<AbiType>),
}

impl AbiType {
    /// Check if type is dynamic (variable size)
    pub fn is_dynamic(&self) -> bool {
        match self {
            AbiType::Bytes | AbiType::String | AbiType::Array(_) => true,
            AbiType::Tuple(types) => types.iter().any(|t| t.is_dynamic()),
            _ => false,
        }
    }

    /// Get the size in bytes for fixed-size types
    pub fn fixed_size(&self) -> Option<usize> {
        match self {
            AbiType::Bool | AbiType::Uint8 | AbiType::Int8 => Some(1),
            AbiType::Uint16 | AbiType::Int16 => Some(2),
            AbiType::Uint32 | AbiType::Int32 => Some(4),
            AbiType::Uint64 | AbiType::Int64 => Some(8),
            AbiType::Uint128 => Some(16),
            AbiType::Uint256 => Some(32),
            AbiType::Address => Some(32),
            AbiType::FixedBytes(n) => Some(*n),
            AbiType::FixedArray(inner, count) => {
                inner.fixed_size().map(|size| size * count)
            }
            _ => None,
        }
    }
}

/// Parameter definition in ABI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbiParam {
    /// Parameter name
    pub name: String,

    /// Parameter type
    #[serde(rename = "type")]
    pub param_type: AbiType,
}

/// Function definition in ABI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbiFunction {
    /// Function name
    pub name: String,

    /// Input parameters
    pub inputs: Vec<AbiParam>,

    /// Output parameters (return values)
    pub outputs: Vec<AbiParam>,

    /// State mutability (view, pure, nonpayable, payable)
    #[serde(rename = "stateMutability")]
    pub state_mutability: String,
}

impl AbiFunction {
    /// Check if function modifies state
    pub fn is_read_only(&self) -> bool {
        matches!(self.state_mutability.as_str(), "view" | "pure")
    }

    /// Get function selector (first 4 bytes of function signature hash)
    pub fn selector(&self) -> [u8; 4] {
        use sha3::{Digest, Sha3_256};

        let signature = self.signature();
        let hash = Sha3_256::digest(signature.as_bytes());
        let mut selector = [0u8; 4];
        selector.copy_from_slice(&hash[..4]);
        selector
    }

    /// Get function signature string (e.g., "transfer(address,uint256)")
    pub fn signature(&self) -> String {
        let param_types: Vec<String> = self.inputs.iter()
            .map(|p| format_type(&p.param_type))
            .collect();
        format!("{}({})", self.name, param_types.join(","))
    }
}

/// Format ABI type as string for signature
fn format_type(t: &AbiType) -> String {
    match t {
        AbiType::Bool => "bool".to_string(),
        AbiType::Uint8 => "uint8".to_string(),
        AbiType::Uint16 => "uint16".to_string(),
        AbiType::Uint32 => "uint32".to_string(),
        AbiType::Uint64 => "uint64".to_string(),
        AbiType::Uint128 => "uint128".to_string(),
        AbiType::Uint256 => "uint256".to_string(),
        AbiType::Int8 => "int8".to_string(),
        AbiType::Int16 => "int16".to_string(),
        AbiType::Int32 => "int32".to_string(),
        AbiType::Int64 => "int64".to_string(),
        AbiType::FixedBytes(n) => format!("bytes{}", n),
        AbiType::Bytes => "bytes".to_string(),
        AbiType::String => "string".to_string(),
        AbiType::Address => "address".to_string(),
        AbiType::Array(inner) => format!("{}[]", format_type(inner)),
        AbiType::FixedArray(inner, size) => format!("{}[{}]", format_type(inner), size),
        AbiType::Tuple(types) => {
            let type_strs: Vec<String> = types.iter().map(format_type).collect();
            format!("({})", type_strs.join(","))
        }
    }
}

/// Complete contract ABI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractAbi {
    /// Contract name
    pub name: String,

    /// Contract functions
    pub functions: Vec<AbiFunction>,

    /// Contract version
    #[serde(default = "default_version")]
    pub version: String,
}

fn default_version() -> String {
    "1.0.0".to_string()
}

impl ContractAbi {
    /// Find function by name
    pub fn get_function(&self, name: &str) -> Option<&AbiFunction> {
        self.functions.iter().find(|f| f.name == name)
    }

    /// List all function names
    pub fn function_names(&self) -> Vec<&str> {
        self.functions.iter().map(|f| f.name.as_str()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_abi_type_sizes() {
        assert_eq!(AbiType::Bool.fixed_size(), Some(1));
        assert_eq!(AbiType::Uint64.fixed_size(), Some(8));
        assert_eq!(AbiType::Uint256.fixed_size(), Some(32));
        assert_eq!(AbiType::Address.fixed_size(), Some(32));
        assert_eq!(AbiType::FixedBytes(32).fixed_size(), Some(32));
        assert_eq!(AbiType::Bytes.fixed_size(), None);
        assert_eq!(AbiType::String.fixed_size(), None);
    }

    #[test]
    fn test_abi_type_dynamic() {
        assert!(!AbiType::Bool.is_dynamic());
        assert!(!AbiType::Uint256.is_dynamic());
        assert!(AbiType::Bytes.is_dynamic());
        assert!(AbiType::String.is_dynamic());
        assert!(AbiType::Array(Box::new(AbiType::Uint256)).is_dynamic());
    }

    #[test]
    fn test_function_signature() {
        let function = AbiFunction {
            name: "transfer".to_string(),
            inputs: vec![
                AbiParam {
                    name: "to".to_string(),
                    param_type: AbiType::Address,
                },
                AbiParam {
                    name: "amount".to_string(),
                    param_type: AbiType::Uint256,
                },
            ],
            outputs: vec![AbiParam {
                name: "success".to_string(),
                param_type: AbiType::Bool,
            }],
            state_mutability: "nonpayable".to_string(),
        };

        assert_eq!(function.signature(), "transfer(address,uint256)");
        assert!(!function.is_read_only());
    }

    #[test]
    fn test_view_function() {
        let function = AbiFunction {
            name: "balanceOf".to_string(),
            inputs: vec![AbiParam {
                name: "account".to_string(),
                param_type: AbiType::Address,
            }],
            outputs: vec![AbiParam {
                name: "balance".to_string(),
                param_type: AbiType::Uint256,
            }],
            state_mutability: "view".to_string(),
        };

        assert!(function.is_read_only());
    }
}
