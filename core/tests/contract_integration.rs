// Integration tests for smart contract functionality
//
// Tests contract deployment, execution, and state management

use boundless_core::*;
use sha3::{Digest, Sha3_256};

/// Create a minimal valid WASM module for testing
/// This is a tiny valid WASM binary with a simple exported function
fn create_test_wasm() -> Vec<u8> {
    // Minimal WASM module:
    // (module
    //   (func (export "test") (result i32)
    //     i32.const 42
    //   )
    // )
    vec![
        0x00, 0x61, 0x73, 0x6D, // WASM magic number
        0x01, 0x00, 0x00, 0x00, // Version 1
        // Type section
        0x01, 0x05, 0x01, 0x60, 0x00, 0x01, 0x7F,
        // Function section
        0x03, 0x02, 0x01, 0x00,
        // Export section
        0x07, 0x08, 0x01, 0x04, 0x74, 0x65, 0x73, 0x74, 0x00, 0x00,
        // Code section
        0x0A, 0x06, 0x01, 0x04, 0x00, 0x41, 0x2A, 0x0B,
    ]
}

/// Create a test deployer keypair
fn create_test_deployer() -> [u8; 32] {
    let mut deployer = [0u8; 32];
    deployer[0] = 0xDE;
    deployer[1] = 0xAD;
    deployer[2] = 0xBE;
    deployer[3] = 0xEF;
    deployer
}

#[test]
fn test_contract_deployment_data_encoding() {
    let deployer = create_test_deployer();
    let initial_state = vec![1, 2, 3, 4];
    let metadata = b"Test Contract v1.0".to_vec();

    let deployment = ContractDeploymentData {
        deployer,
        initial_state: initial_state.clone(),
        metadata: metadata.clone(),
    };

    // Test encoding
    let encoded = deployment.encode();
    assert!(!encoded.is_empty());

    // Test decoding
    let decoded = ContractDeploymentData::decode(&encoded).expect("Should decode");
    assert_eq!(decoded.deployer, deployer);
    assert_eq!(decoded.initial_state, initial_state);
    assert_eq!(decoded.metadata, metadata);
}

#[test]
fn test_contract_deployment_validation() {
    let deployer = create_test_deployer();

    // Test valid deployment
    let valid = ContractDeploymentData {
        deployer,
        initial_state: vec![0; 100],
        metadata: vec![0; 100],
    };
    assert!(valid.validate().is_ok());

    // Test oversized initial state
    let oversized_state = ContractDeploymentData {
        deployer,
        initial_state: vec![0; 5000], // Max is 4096
        metadata: vec![0; 100],
    };
    assert!(oversized_state.validate().is_err());

    // Test oversized metadata
    let oversized_metadata = ContractDeploymentData {
        deployer,
        initial_state: vec![0; 100],
        metadata: vec![0; 3000], // Max is 2048
    };
    assert!(oversized_metadata.validate().is_err());
}

#[test]
fn test_contract_call_data_encoding() {
    let contract_address = [0x42; 32];
    let caller = create_test_deployer();
    let function_name = "transfer".to_string();
    let args = vec![1, 2, 3, 4, 5];

    let call_data = ContractCallData {
        contract_address,
        function_name: function_name.clone(),
        args: args.clone(),
        caller,
    };

    // Test encoding
    let encoded = call_data.encode();
    assert!(!encoded.is_empty());

    // Test decoding
    let decoded = ContractCallData::decode(&encoded).expect("Should decode");
    assert_eq!(decoded.contract_address, contract_address);
    assert_eq!(decoded.function_name, function_name);
    assert_eq!(decoded.args, args);
    assert_eq!(decoded.caller, caller);
}

#[test]
fn test_contract_call_wasm_encoding() {
    let contract_address = [0x42; 32];
    let caller = create_test_deployer();
    let function_name = "transfer".to_string();
    let args = vec![0xAA, 0xBB, 0xCC];

    let call_data = ContractCallData {
        contract_address,
        function_name: function_name.clone(),
        args: args.clone(),
        caller,
    };

    // Test WASM encoding format: [2-byte name_len][name][args]
    let wasm_encoded = call_data.encode_for_wasm();

    // Verify format
    assert!(wasm_encoded.len() >= 2);

    // First 2 bytes should be name length
    let name_len = u16::from_le_bytes([wasm_encoded[0], wasm_encoded[1]]) as usize;
    assert_eq!(name_len, function_name.len());

    // Next bytes should be function name
    let name_bytes = &wasm_encoded[2..2 + name_len];
    assert_eq!(name_bytes, function_name.as_bytes());

    // Remaining bytes should be args
    let args_bytes = &wasm_encoded[2 + name_len..];
    assert_eq!(args_bytes, args.as_slice());

    // Test decoding
    let decoded = ContractCallData::decode_from_wasm(&wasm_encoded).expect("Should decode");
    assert_eq!(decoded.0, function_name);
    assert_eq!(decoded.1, args);
}

#[test]
fn test_contract_call_validation() {
    let contract_address = [0x42; 32];
    let caller = create_test_deployer();

    // Test valid call
    let valid = ContractCallData {
        contract_address,
        function_name: "transfer".to_string(),
        args: vec![0; 100],
        caller,
    };
    assert!(valid.validate().is_ok());

    // Test empty function name
    let empty_name = ContractCallData {
        contract_address,
        function_name: "".to_string(),
        args: vec![0; 100],
        caller,
    };
    assert!(empty_name.validate().is_err());

    // Test oversized function name
    let long_name = ContractCallData {
        contract_address,
        function_name: "a".repeat(300), // Max is 256
        args: vec![0; 100],
        caller,
    };
    assert!(long_name.validate().is_err());

    // Test oversized args
    let large_args = ContractCallData {
        contract_address,
        function_name: "transfer".to_string(),
        args: vec![0; 10000], // Max is 8192
        caller,
    };
    assert!(large_args.validate().is_err());
}

#[test]
fn test_contract_info_creation() {
    let contract_address = [0x42; 32];
    let deployer = create_test_deployer();
    let wasm_bytecode = create_test_wasm();
    let deployed_at_height = 100;
    let deployed_at_tx = [0x99; 32];

    let contract_info = ContractInfo {
        contract_address,
        wasm_bytecode: wasm_bytecode.clone(),
        deployer,
        deployed_at_height,
        deployed_at_tx,
    };

    // Verify fields
    assert_eq!(contract_info.contract_address, contract_address);
    assert_eq!(contract_info.wasm_bytecode, wasm_bytecode);
    assert_eq!(contract_info.deployer, deployer);
    assert_eq!(contract_info.deployed_at_height, deployed_at_height);
    assert_eq!(contract_info.deployed_at_tx, deployed_at_tx);

    // Test WASM validation
    assert!(contract_info.validate_wasm().is_ok());
}

#[test]
fn test_contract_info_wasm_validation() {
    let contract_address = [0x42; 32];
    let deployer = create_test_deployer();
    let deployed_at_tx = [0x99; 32];

    // Test valid WASM
    let valid_contract = ContractInfo {
        contract_address,
        wasm_bytecode: create_test_wasm(),
        deployer,
        deployed_at_height: 100,
        deployed_at_tx,
    };
    assert!(valid_contract.validate_wasm().is_ok());

    // Test invalid magic number
    let invalid_magic = ContractInfo {
        contract_address,
        wasm_bytecode: vec![0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00],
        deployer,
        deployed_at_height: 100,
        deployed_at_tx,
    };
    assert!(invalid_magic.validate_wasm().is_err());

    // Test too small
    let too_small = ContractInfo {
        contract_address,
        wasm_bytecode: vec![0x00, 0x61],
        deployer,
        deployed_at_height: 100,
        deployed_at_tx,
    };
    assert!(too_small.validate_wasm().is_err());

    // Test too large
    let too_large = ContractInfo {
        contract_address,
        wasm_bytecode: vec![0; 2_000_000], // Max is 1MB
        deployer,
        deployed_at_height: 100,
        deployed_at_tx,
    };
    assert!(too_large.validate_wasm().is_err());
}

#[test]
fn test_contract_state_operations() {
    let address = [0x42; 32];
    let mut state = ContractState::new(address, 10_000);

    // Test initial state
    assert_eq!(state.address, address);
    assert_eq!(state.storage_used, 0);
    assert_eq!(state.storage_quota, 10_000);

    // Test setting value
    let key = [0xAA; 32];
    let value = vec![1, 2, 3, 4];
    assert!(state.set(key, value.clone()).is_ok());
    assert_eq!(state.storage_used, 1);

    // Test getting value
    let retrieved = state.get(&key);
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap(), &value);

    // Test updating value
    let new_value = vec![5, 6, 7, 8, 9];
    assert!(state.set(key, new_value.clone()).is_ok());
    assert_eq!(state.get(&key).unwrap(), &new_value);

    // Test removing value
    assert!(state.remove(&key).is_ok());
    assert!(state.get(&key).is_none());
    assert_eq!(state.storage_used, 0);
}

#[test]
fn test_contract_state_quota_enforcement() {
    let address = [0x42; 32];
    let mut state = ContractState::new(address, 10);

    // Fill up quota (each slot counts as 1 used)
    for i in 0..10 {
        let mut key = [0u8; 32];
        key[0] = i as u8;
        assert!(state.set(key, vec![i as u8]).is_ok());
    }
    assert_eq!(state.storage_used, 10);

    // Try to exceed quota
    let overflow_key = [0xFF; 32];
    assert!(state.set(overflow_key, vec![42]).is_err());
}

#[test]
fn test_contract_state_apply_changes() {
    let address = [0x42; 32];
    let mut state = ContractState::new(address, 10_000);

    // Create state changes
    let key1 = [0x01; 32];
    let key2 = [0x02; 32];
    let changes = vec![
        StateChange {
            key: key1,
            value: Some(vec![1, 2, 3]),
        },
        StateChange {
            key: key2,
            value: Some(vec![4, 5, 6]),
        },
    ];

    // Apply changes
    state.apply_changes(&changes).expect("Should apply changes");

    // Verify changes applied
    assert_eq!(state.get(&key1).unwrap(), &vec![1, 2, 3]);
    assert_eq!(state.get(&key2).unwrap(), &vec![4, 5, 6]);
    assert_eq!(state.storage_used, 2);

    // Apply deletion
    let delete_changes = vec![StateChange {
        key: key1,
        value: None,
    }];
    state.apply_changes(&delete_changes).expect("Should apply deletion");
    assert!(state.get(&key1).is_none());
    assert_eq!(state.storage_used, 1);
}

#[test]
fn test_contract_state_usage_percentage() {
    let address = [0x42; 32];
    let mut state = ContractState::new(address, 100);

    // Empty state
    assert_eq!(state.usage_percentage(), 0.0);

    // 50% usage
    for i in 0..50 {
        let mut key = [0u8; 32];
        key[0] = i as u8;
        state.set(key, vec![i as u8]).unwrap();
    }
    assert_eq!(state.usage_percentage(), 50.0);

    // 100% usage
    for i in 50..100 {
        let mut key = [0u8; 32];
        key[0] = i as u8;
        state.set(key, vec![i as u8]).unwrap();
    }
    assert_eq!(state.usage_percentage(), 100.0);
}

#[test]
fn test_contract_address_derivation() {
    // Contract address should be SHA3-256(deployment_tx_hash)
    let tx_hash = [0x12; 32];

    let mut hasher = Sha3_256::new();
    hasher.update(&tx_hash);
    let expected_address: [u8; 32] = hasher.finalize().into();

    // This is the expected pattern - service layer should use this
    assert_eq!(expected_address.len(), 32);

    // Verify it's deterministic
    let mut hasher2 = Sha3_256::new();
    hasher2.update(&tx_hash);
    let address2: [u8; 32] = hasher2.finalize().into();
    assert_eq!(expected_address, address2);
}

#[test]
fn test_contract_deployment_marker() {
    // Verify the deployment marker constant
    assert_eq!(CONTRACT_DEPLOYMENT_MARKER, [0xFF; 32]);

    // This marker is used in transaction outputs to identify contract deployments
    let marker = CONTRACT_DEPLOYMENT_MARKER;
    assert!(marker.iter().all(|&b| b == 0xFF));
}

#[test]
fn test_transaction_type_ids() {
    // Verify transaction type IDs are correct
    assert_eq!(TransactionType::ContractDeployment.to_u8(), 4);
    assert_eq!(TransactionType::ContractCall.to_u8(), 5);

    // Verify round-trip conversion
    assert_eq!(
        TransactionType::from_u8(4),
        Some(TransactionType::ContractDeployment)
    );
    assert_eq!(
        TransactionType::from_u8(5),
        Some(TransactionType::ContractCall)
    );
}
