// Smart Contract State Management
//
// Provides structures for storing and managing smart contract state in a UTXO blockchain

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Contract deployment marker address
/// Special recipient_pubkey_hash value that marks a transaction output as a contract deployment
pub const CONTRACT_DEPLOYMENT_MARKER: [u8; 32] = [0xFF; 32];

/// Contract registry entry
/// Stores immutable information about a deployed contract
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractInfo {
    /// Contract address (derived from deployment transaction hash)
    pub contract_address: [u8; 32],

    /// WASM bytecode (stored once at deployment)
    pub wasm_bytecode: Vec<u8>,

    /// Original deployer address
    pub deployer: [u8; 32],

    /// Block height at which contract was deployed
    pub deployed_at_height: u64,

    /// Deployment transaction hash
    pub deployed_at_tx: [u8; 32],
}

impl ContractInfo {
    /// Create new contract info
    pub fn new(
        contract_address: [u8; 32],
        wasm_bytecode: Vec<u8>,
        deployer: [u8; 32],
        deployed_at_height: u64,
        deployed_at_tx: [u8; 32],
    ) -> Self {
        Self {
            contract_address,
            wasm_bytecode,
            deployer,
            deployed_at_height,
            deployed_at_tx,
        }
    }

    /// Validate WASM bytecode
    pub fn validate_bytecode(&self) -> Result<(), String> {
        // Maximum WASM bytecode size: 1 MB
        const MAX_WASM_SIZE: usize = 1024 * 1024;

        if self.wasm_bytecode.is_empty() {
            return Err("WASM bytecode is empty".to_string());
        }

        if self.wasm_bytecode.len() > MAX_WASM_SIZE {
            return Err(format!(
                "WASM bytecode exceeds maximum size: {} > {}",
                self.wasm_bytecode.len(),
                MAX_WASM_SIZE
            ));
        }

        // Check for WASM magic number (0x00 0x61 0x73 0x6D)
        if self.wasm_bytecode.len() >= 4 {
            let magic = &self.wasm_bytecode[0..4];
            if magic != &[0x00, 0x61, 0x73, 0x6D] {
                return Err("Invalid WASM magic number".to_string());
            }
        } else {
            return Err("WASM bytecode too short".to_string());
        }

        Ok(())
    }
}

/// Storage change for a contract
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateChange {
    /// Storage key (SHA3-256 hash)
    pub key: [u8; 32],

    /// New value (None = deletion)
    pub value: Option<Vec<u8>>,
}

impl StateChange {
    /// Create a new state change
    pub fn new(key: [u8; 32], value: Option<Vec<u8>>) -> Self {
        Self { key, value }
    }

    /// Create a state update
    pub fn update(key: [u8; 32], value: Vec<u8>) -> Self {
        Self {
            key,
            value: Some(value),
        }
    }

    /// Create a state deletion
    pub fn delete(key: [u8; 32]) -> Self {
        Self { key, value: None }
    }
}

/// Contract state storage
/// Manages the mutable state of a deployed contract
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractState {
    /// Contract address
    pub address: [u8; 32],

    /// Key-value storage (key = SHA3-256 hash, value = arbitrary bytes)
    pub storage: HashMap<[u8; 32], Vec<u8>>,

    /// Storage quota (max number of storage slots)
    pub storage_quota: u64,

    /// Storage used (current number of slots)
    pub storage_used: u64,

    /// Last modified block height
    pub last_modified: u64,
}

impl ContractState {
    /// Create new empty contract state
    pub fn new(address: [u8; 32]) -> Self {
        Self {
            address,
            storage: HashMap::new(),
            storage_quota: 10000, // Default: 10,000 storage slots
            storage_used: 0,
            last_modified: 0,
        }
    }

    /// Create contract state with custom quota
    pub fn with_quota(address: [u8; 32], quota: u64) -> Self {
        Self {
            address,
            storage: HashMap::new(),
            storage_quota: quota,
            storage_used: 0,
            last_modified: 0,
        }
    }

    /// Get storage value
    pub fn get(&self, key: &[u8; 32]) -> Option<&Vec<u8>> {
        self.storage.get(key)
    }

    /// Set storage value
    pub fn set(&mut self, key: [u8; 32], value: Vec<u8>) -> Result<(), String> {
        // Maximum value size: 1 KB
        const MAX_VALUE_SIZE: usize = 1024;

        if value.len() > MAX_VALUE_SIZE {
            return Err(format!(
                "Storage value exceeds maximum size: {} > {}",
                value.len(),
                MAX_VALUE_SIZE
            ));
        }

        // Check if this is a new slot
        let is_new_slot = !self.storage.contains_key(&key);

        if is_new_slot {
            // Check storage quota
            if self.storage_used >= self.storage_quota {
                return Err(format!(
                    "Storage quota exceeded: {} >= {}",
                    self.storage_used, self.storage_quota
                ));
            }

            self.storage_used += 1;
        }

        self.storage.insert(key, value);
        Ok(())
    }

    /// Remove storage value
    pub fn remove(&mut self, key: &[u8; 32]) -> Option<Vec<u8>> {
        if let Some(value) = self.storage.remove(key) {
            self.storage_used = self.storage_used.saturating_sub(1);
            Some(value)
        } else {
            None
        }
    }

    /// Apply state changes
    pub fn apply_changes(&mut self, changes: &[StateChange]) -> Result<(), String> {
        for change in changes {
            match &change.value {
                Some(value) => {
                    self.set(change.key, value.clone())?;
                }
                None => {
                    self.remove(&change.key);
                }
            }
        }
        Ok(())
    }

    /// Get storage usage percentage
    pub fn usage_percentage(&self) -> f64 {
        if self.storage_quota == 0 {
            0.0
        } else {
            (self.storage_used as f64 / self.storage_quota as f64) * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contract_info_validate_wasm() {
        // Valid WASM (with magic number)
        let mut wasm = vec![0x00, 0x61, 0x73, 0x6D]; // WASM magic
        wasm.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]); // Version 1

        let contract = ContractInfo::new([1u8; 32], wasm, [2u8; 32], 100, [3u8; 32]);

        assert!(contract.validate_bytecode().is_ok());

        // Invalid WASM (wrong magic)
        let invalid_wasm = vec![0xFF, 0xFF, 0xFF, 0xFF];
        let invalid_contract =
            ContractInfo::new([1u8; 32], invalid_wasm, [2u8; 32], 100, [3u8; 32]);

        assert!(invalid_contract.validate_bytecode().is_err());
    }

    #[test]
    fn test_contract_state_storage() {
        let mut state = ContractState::new([1u8; 32]);

        // Set value
        let key1 = [1u8; 32];
        let value1 = vec![1, 2, 3];
        assert!(state.set(key1, value1.clone()).is_ok());
        assert_eq!(state.storage_used, 1);
        assert_eq!(state.get(&key1), Some(&value1));

        // Update value
        let value2 = vec![4, 5, 6];
        assert!(state.set(key1, value2.clone()).is_ok());
        assert_eq!(state.storage_used, 1); // Still 1 slot
        assert_eq!(state.get(&key1), Some(&value2));

        // Remove value
        assert_eq!(state.remove(&key1), Some(value2));
        assert_eq!(state.storage_used, 0);
        assert_eq!(state.get(&key1), None);
    }

    #[test]
    fn test_contract_state_quota() {
        let mut state = ContractState::with_quota([1u8; 32], 2);

        // Add first value
        assert!(state.set([1u8; 32], vec![1]).is_ok());
        assert_eq!(state.storage_used, 1);

        // Add second value
        assert!(state.set([2u8; 32], vec![2]).is_ok());
        assert_eq!(state.storage_used, 2);

        // Try to exceed quota
        let result = state.set([3u8; 32], vec![3]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Storage quota exceeded"));
    }

    #[test]
    fn test_state_changes() {
        let mut state = ContractState::new([1u8; 32]);

        let changes = vec![
            StateChange::update([1u8; 32], vec![1, 2, 3]),
            StateChange::update([2u8; 32], vec![4, 5, 6]),
            StateChange::delete([1u8; 32]),
        ];

        assert!(state.apply_changes(&changes).is_ok());
        assert_eq!(state.get(&[1u8; 32]), None); // Deleted
        assert_eq!(state.get(&[2u8; 32]), Some(&vec![4, 5, 6]));
        assert_eq!(state.storage_used, 1);
    }

    #[test]
    fn test_usage_percentage() {
        let mut state = ContractState::with_quota([1u8; 32], 10);

        assert_eq!(state.usage_percentage(), 0.0);

        state.set([1u8; 32], vec![1]).unwrap();
        assert_eq!(state.usage_percentage(), 10.0);

        state.set([2u8; 32], vec![2]).unwrap();
        state.set([3u8; 32], vec![3]).unwrap();
        state.set([4u8; 32], vec![4]).unwrap();
        state.set([5u8; 32], vec![5]).unwrap();
        assert_eq!(state.usage_percentage(), 50.0);
    }
}
