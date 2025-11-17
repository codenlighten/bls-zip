// Boundless BLS Core - Block, Transaction, and Account Data Structures
//
// This module provides the fundamental data structures for the Boundless BLS blockchain,
// including support for Post-Quantum Cryptography (PQC) signatures.

pub mod account;
pub mod asset;
pub mod block;
pub mod contract;
pub mod error;
pub mod merkle;
pub mod proof;
pub mod state;
pub mod transaction;
pub mod tx_index;
pub mod tx_types;

pub use account::Account;
pub use asset::{AssetBalance, AssetDefinition, AssetRegistry, AssetType};
pub use block::{Block, BlockHeader};
pub use contract::{ContractInfo, ContractState, StateChange, CONTRACT_DEPLOYMENT_MARKER};
pub use error::CoreError;
pub use merkle::MerkleTree;
pub use proof::{ProofAnchor, ProofStorage, ProofType};
pub use state::{BlockchainState, OutPoint, StateError};
pub use transaction::{Signature, Transaction, TxInput, TxOutput};
pub use tx_index::{TransactionIndex, TransactionRecord, TransactionStatus};
pub use tx_types::{
    AssetRegisterData, AssetTransferData, ContractCallData, ContractDeploymentData,
    ProofAnchorData, TransactionBuilder, TransactionType,
};

/// Version number for protocol changes
pub const PROTOCOL_VERSION: u32 = 1;

/// Target block time in seconds (5 minutes)
pub const TARGET_BLOCK_TIME_SECS: u64 = 300;

/// Difficulty adjustment interval in blocks (~3.5 days)
pub const DIFFICULTY_ADJUSTMENT_INTERVAL: u64 = 1008;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(PROTOCOL_VERSION, 1);
        assert_eq!(TARGET_BLOCK_TIME_SECS, 300);
        assert_eq!(DIFFICULTY_ADJUSTMENT_INTERVAL, 1008);
    }
}
