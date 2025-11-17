// ABI Encoding Tests - Verifying function name encoding
//
// Tests that the ABI encoder correctly encodes function calls using function names
// instead of 4-byte selectors, matching the WASM runtime expectations.

use boundless_enterprise::abi::encoder::AbiEncoder;
use boundless_enterprise::abi::types::{AbiFunction, AbiParam, AbiType};
use serde_json::json;

/// Helper to create a simple test function
fn create_test_function(name: &str) -> AbiFunction {
    AbiFunction {
        name: name.to_string(),
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
    }
}

#[test]
fn test_function_name_encoding_format() {
    let encoder = AbiEncoder::new();
    let function = create_test_function("transfer");

    let params = json!({
        "to": "0x0000000000000000000000000000000000000000000000000000000000000001",
        "amount": 1000
    });

    let encoded = encoder
        .encode_function_call(&function, &params)
        .expect("Should encode");

    // Format should be: [2-byte name_len][function_name UTF-8][params]

    // First 2 bytes should be name length
    assert!(encoded.len() >= 2);
    let name_len = u16::from_le_bytes([encoded[0], encoded[1]]) as usize;
    assert_eq!(name_len, "transfer".len());

    // Next bytes should be the function name
    let name_end = 2 + name_len;
    assert!(encoded.len() >= name_end);
    let name_bytes = &encoded[2..name_end];
    assert_eq!(std::str::from_utf8(name_bytes).unwrap(), "transfer");

    // Remaining bytes should be the encoded parameters
    let params_start = name_end;
    assert!(encoded.len() > params_start); // Should have parameter data
}

#[test]
fn test_different_function_names() {
    let encoder = AbiEncoder::new();

    let test_cases = vec![
        ("transfer", 8),
        ("approve", 7),
        ("mint", 4),
        ("balanceOf", 9),
        ("totalSupply", 11),
    ];

    for (func_name, expected_len) in test_cases {
        let function = create_test_function(func_name);
        let params = json!({
            "to": "0x0000000000000000000000000000000000000000000000000000000000000001",
            "amount": 100
        });

        let encoded = encoder
            .encode_function_call(&function, &params)
            .expect("Should encode");

        // Verify name length
        let name_len = u16::from_le_bytes([encoded[0], encoded[1]]) as usize;
        assert_eq!(name_len, expected_len);

        // Verify name matches
        let name_bytes = &encoded[2..2 + name_len];
        assert_eq!(std::str::from_utf8(name_bytes).unwrap(), func_name);
    }
}

#[test]
fn test_no_selector_in_encoding() {
    let encoder = AbiEncoder::new();
    let function = create_test_function("transfer");

    let params = json!({
        "to": "0x0000000000000000000000000000000000000000000000000000000000000001",
        "amount": 1000
    });

    let encoded = encoder
        .encode_function_call(&function, &params)
        .expect("Should encode");

    // OLD FORMAT (Ethereum-style):
    // [4-byte selector][params]
    //
    // NEW FORMAT (Boundless WASM):
    // [2-byte name_len][name UTF-8][params]

    // Verify first 2 bytes are a reasonable name length (not a selector)
    let name_len = u16::from_le_bytes([encoded[0], encoded[1]]) as usize;
    assert!(name_len > 0 && name_len < 256); // Reasonable name length

    // Verify bytes 2+ are UTF-8 text (not binary selector)
    let name_bytes = &encoded[2..2 + name_len];
    assert!(std::str::from_utf8(name_bytes).is_ok());

    // Verify the function name is actually in the encoding
    let name_str = std::str::from_utf8(name_bytes).unwrap();
    assert_eq!(name_str, "transfer");
}

#[test]
fn test_encoding_with_various_parameter_types() {
    let encoder = AbiEncoder::new();

    // Function with multiple parameter types
    let function = AbiFunction {
        name: "complexFunction".to_string(),
        inputs: vec![
            AbiParam {
                name: "flag".to_string(),
                param_type: AbiType::Bool,
            },
            AbiParam {
                name: "count".to_string(),
                param_type: AbiType::Uint32,
            },
            AbiParam {
                name: "addr".to_string(),
                param_type: AbiType::Address,
            },
            AbiParam {
                name: "message".to_string(),
                param_type: AbiType::String,
            },
        ],
        outputs: vec![],
        state_mutability: "nonpayable".to_string(),
    };

    let params = json!({
        "flag": true,
        "count": 42,
        "addr": "0x0000000000000000000000000000000000000000000000000000000000000001",
        "message": "Hello WASM"
    });

    let encoded = encoder
        .encode_function_call(&function, &params)
        .expect("Should encode");

    // Verify function name encoding
    let name_len = u16::from_le_bytes([encoded[0], encoded[1]]) as usize;
    assert_eq!(name_len, "complexFunction".len());

    let name_bytes = &encoded[2..2 + name_len];
    assert_eq!(
        std::str::from_utf8(name_bytes).unwrap(),
        "complexFunction"
    );

    // Verify there's parameter data after the name
    assert!(encoded.len() > 2 + name_len);
}

#[test]
fn test_encoding_deterministic() {
    let encoder = AbiEncoder::new();
    let function = create_test_function("transfer");

    let params = json!({
        "to": "0x0000000000000000000000000000000000000000000000000000000000000001",
        "amount": 1000
    });

    // Encode multiple times
    let encoded1 = encoder
        .encode_function_call(&function, &params)
        .expect("Should encode");
    let encoded2 = encoder
        .encode_function_call(&function, &params)
        .expect("Should encode");
    let encoded3 = encoder
        .encode_function_call(&function, &params)
        .expect("Should encode");

    // All encodings should be identical
    assert_eq!(encoded1, encoded2);
    assert_eq!(encoded2, encoded3);
}

#[test]
fn test_zero_parameter_function() {
    let encoder = AbiEncoder::new();

    let function = AbiFunction {
        name: "getTotalSupply".to_string(),
        inputs: vec![],
        outputs: vec![AbiParam {
            name: "total".to_string(),
            param_type: AbiType::Uint64,
        }],
        state_mutability: "view".to_string(),
    };

    let params = json!({});

    let encoded = encoder
        .encode_function_call(&function, &params)
        .expect("Should encode");

    // Should still have function name encoding
    let name_len = u16::from_le_bytes([encoded[0], encoded[1]]) as usize;
    assert_eq!(name_len, "getTotalSupply".len());

    let name_bytes = &encoded[2..2 + name_len];
    assert_eq!(
        std::str::from_utf8(name_bytes).unwrap(),
        "getTotalSupply"
    );

    // With no parameters, encoding should be just the name
    assert_eq!(encoded.len(), 2 + name_len);
}

#[test]
fn test_long_function_name() {
    let encoder = AbiEncoder::new();

    let long_name = "veryLongFunctionNameThatIsStillValid";
    let function = AbiFunction {
        name: long_name.to_string(),
        inputs: vec![],
        outputs: vec![],
        state_mutability: "nonpayable".to_string(),
    };

    let params = json!({});

    let encoded = encoder
        .encode_function_call(&function, &params)
        .expect("Should encode");

    // Verify name length
    let name_len = u16::from_le_bytes([encoded[0], encoded[1]]) as usize;
    assert_eq!(name_len, long_name.len());

    // Verify full name is encoded
    let name_bytes = &encoded[2..2 + name_len];
    assert_eq!(std::str::from_utf8(name_bytes).unwrap(), long_name);
}

#[test]
fn test_matches_wasm_call_data_format() {
    // This test verifies that ABI encoder output matches the format expected
    // by ContractCallData::encode_for_wasm() in boundless-core
    //
    // Format: [2-byte name_len][function_name UTF-8][args]

    let encoder = AbiEncoder::new();
    let function = create_test_function("mint");

    let params = json!({
        "to": "0x0000000000000000000000000000000000000000000000000000000000000099",
        "amount": 5000
    });

    let encoded = encoder
        .encode_function_call(&function, &params)
        .expect("Should encode");

    // Parse as WASM format
    let name_len = u16::from_le_bytes([encoded[0], encoded[1]]) as usize;
    let function_name = std::str::from_utf8(&encoded[2..2 + name_len])
        .expect("Should be valid UTF-8");
    let args = &encoded[2 + name_len..];

    // Verify it matches expected WASM call data format
    assert_eq!(function_name, "mint");
    assert!(args.len() > 0); // Should have parameter bytes

    // Reconstruct to verify format
    let mut reconstructed = Vec::new();
    reconstructed.extend_from_slice(&(name_len as u16).to_le_bytes());
    reconstructed.extend_from_slice(function_name.as_bytes());
    reconstructed.extend_from_slice(args);

    assert_eq!(encoded, reconstructed);
}
