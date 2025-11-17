// Blockchain state management with UTXO tracking
use crate::{
    asset::AssetRegistry,
    contract::{ContractInfo, ContractState},
    proof::ProofStorage,
    tx_index::TransactionIndex,
    Block, Transaction, TxOutput,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unique identifier for a transaction output (UTXO)
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct OutPoint {
    /// Transaction hash containing the output
    pub tx_hash: [u8; 32],
    /// Output index within the transaction
    pub index: u32,
}

impl OutPoint {
    pub fn new(tx_hash: [u8; 32], index: u32) -> Self {
        Self { tx_hash, index }
    }
}

/// Blockchain state with UTXO tracking and account management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainState {
    /// UTXO set: OutPoint -> TxOutput
    utxo_set: HashMap<OutPoint, TxOutput>,

    /// Account nonces for replay protection (pubkey_hash -> nonce)
    account_nonces: HashMap<[u8; 32], u64>,

    /// Current block height
    block_height: u64,

    /// Best block hash
    best_block_hash: [u8; 32],

    /// Total circulating supply
    total_supply: u64,

    /// Coinbase reward per block
    block_reward: u64,

    /// Consumed UTXOs by block height (for rollback support)
    /// Maps block_height -> (OutPoint -> TxOutput)
    consumed_utxos: HashMap<u64, HashMap<OutPoint, TxOutput>>,

    /// Proof anchoring storage (for identity attestations)
    proof_storage: ProofStorage,

    /// Multi-asset registry (for tokens beyond native BLS)
    asset_registry: AssetRegistry,

    /// Transaction index (for history queries and E2 integration)
    tx_index: TransactionIndex,

    /// Smart contract registry (immutable contract information)
    /// Maps contract_address -> ContractInfo
    contract_registry: HashMap<[u8; 32], ContractInfo>,

    /// Smart contract states (mutable contract storage)
    /// Maps contract_address -> ContractState
    contract_states: HashMap<[u8; 32], ContractState>,
}

impl BlockchainState {
    /// Create a new empty blockchain state
    pub fn new() -> Self {
        Self {
            utxo_set: HashMap::new(),
            account_nonces: HashMap::new(),
            block_height: 0,
            best_block_hash: [0u8; 32],
            total_supply: 0,
            block_reward: 5_000_000_000, // 50 BLS coins
            consumed_utxos: HashMap::new(),
            proof_storage: ProofStorage::new(),
            asset_registry: AssetRegistry::new(),
            tx_index: TransactionIndex::new(),
            contract_registry: HashMap::new(),
            contract_states: HashMap::new(),
        }
    }

    /// Initialize genesis block state
    pub fn with_genesis(genesis_block: &Block) -> Result<Self, StateError> {
        let mut state = Self::new();
        state.apply_block(genesis_block)?;
        Ok(state)
    }

    /// Get current block height
    pub fn height(&self) -> u64 {
        self.block_height
    }

    /// Get best block hash
    pub fn best_block_hash(&self) -> [u8; 32] {
        self.best_block_hash
    }

    /// Get total supply
    pub fn total_supply(&self) -> u64 {
        self.total_supply
    }

    /// Get proof storage (for identity attestation anchoring)
    pub fn proof_storage(&self) -> &ProofStorage {
        &self.proof_storage
    }

    /// Get mutable proof storage
    pub fn proof_storage_mut(&mut self) -> &mut ProofStorage {
        &mut self.proof_storage
    }

    /// Get asset registry (for multi-asset support)
    pub fn asset_registry(&self) -> &AssetRegistry {
        &self.asset_registry
    }

    /// Get mutable asset registry
    pub fn asset_registry_mut(&mut self) -> &mut AssetRegistry {
        &mut self.asset_registry
    }

    /// Get transaction index (for history queries)
    pub fn tx_index(&self) -> &TransactionIndex {
        &self.tx_index
    }

    /// Get mutable transaction index
    pub fn tx_index_mut(&mut self) -> &mut TransactionIndex {
        &mut self.tx_index
    }

    /// Get contract by address
    pub fn get_contract(&self, address: &[u8; 32]) -> Option<&ContractInfo> {
        self.contract_registry.get(address)
    }

    /// Get contract state by address
    pub fn get_contract_state(&self, address: &[u8; 32]) -> Option<&ContractState> {
        self.contract_states.get(address)
    }

    /// Get mutable contract state by address
    pub fn get_contract_state_mut(&mut self, address: &[u8; 32]) -> Option<&mut ContractState> {
        self.contract_states.get_mut(address)
    }

    /// Check if a contract exists
    pub fn has_contract(&self, address: &[u8; 32]) -> bool {
        self.contract_registry.contains_key(address)
    }

    /// Apply a block to the state
    pub fn apply_block(&mut self, block: &Block) -> Result<(), StateError> {
        // Validate block height (allow genesis block at height 0)
        let expected_height = if self.block_height == 0 && block.header.height == 1 {
            1 // Genesis block
        } else {
            self.block_height + 1
        };

        if block.header.height != expected_height {
            return Err(StateError::InvalidBlockHeight {
                expected: expected_height,
                got: block.header.height,
            });
        }

        // Validate previous block hash (skip for genesis)
        if block.header.height > 1 && block.header.previous_hash != self.best_block_hash {
            return Err(StateError::InvalidPreviousHash);
        }

        // Process each transaction
        for (index, tx) in block.transactions.iter().enumerate() {
            // Calculate fee (returned from apply_transaction)
            let fee = if index == 0 {
                // First transaction is coinbase (no inputs required)
                self.apply_coinbase(tx, block.header.height)?;
                0 // Coinbase has no fee
            } else {
                // Apply regular transaction and get fee
                self.apply_transaction(tx, block.header.height)?
            };

            // Index transaction for history queries
            self.tx_index.index_transaction(
                tx,
                block.header.height,
                block.header.hash(),
                block.header.timestamp,
                fee,
            );
        }

        // Update block height and best hash
        self.block_height = block.header.height;
        self.best_block_hash = block.header.hash();

        Ok(())
    }

    /// Apply a coinbase transaction
    fn apply_coinbase(&mut self, tx: &Transaction, DIFFICULTY_ADJUSTMENT_INTERVAL: u64) -> Result<(), StateError> {
        // Coinbase should have no inputs
        if !tx.inputs.is_empty() {
            return Err(StateError::InvalidCoinbase(
                "Coinbase has inputs".to_string(),
            ));
        }

        // Coinbase should have exactly one output
        if tx.outputs.len() != 1 {
            return Err(StateError::InvalidCoinbase(
                "Coinbase must have exactly one output".to_string(),
            ));
        }

        let output = &tx.outputs[0];

        // Verify coinbase amount doesn't exceed block reward
        if output.amount > self.block_reward {
            return Err(StateError::InvalidCoinbase(format!(
                "Coinbase amount {} exceeds block reward {}",
                output.amount, self.block_reward
            )));
        }

        // Add coinbase output to UTXO set
        let tx_hash = tx.hash();
        let outpoint = OutPoint::new(tx_hash, 0);
        self.utxo_set.insert(outpoint, output.clone());

        // Update total supply
        self.total_supply = self
            .total_supply
            .checked_add(output.amount)
            .ok_or(StateError::ArithmeticOverflow)?;

        Ok(())
    }

    /// Apply a regular transaction to the state
    fn apply_transaction(&mut self, tx: &Transaction, block_height: u64) -> Result<u64, StateError> {
        use crate::tx_types::{TransactionBuilder, TransactionType};

        let tx_hash = tx.hash();

        // Check transaction type and handle accordingly
        let tx_type = TransactionBuilder::get_transaction_type(tx);

        match tx_type {
            TransactionType::ProofAnchor => {
                return self.apply_proof_anchor_transaction(tx, block_height);
            }
            TransactionType::AssetTransfer => {
                return self.apply_asset_transfer_transaction(tx, block_height);
            }
            TransactionType::AssetRegister => {
                return self.apply_asset_register_transaction(tx, block_height);
            }
            TransactionType::ContractDeployment => {
                return self.apply_contract_deployment_transaction(tx, block_height);
            }
            TransactionType::ContractCall => {
                return self.apply_contract_call_transaction(tx, block_height);
            }
            TransactionType::Standard => {
                // Continue with standard UTXO processing
            }
        }

        // SECURITY: Validate transaction fee before processing
        self.validate_transaction_fee(tx)?;

        // Initialize consumed UTXOs tracking for current block if not exists
        let block_consumed = self
            .consumed_utxos
            .entry(block_height)
            .or_insert_with(HashMap::new);

        // Verify inputs exist and consume them
        let mut input_sum = 0u64;
        for input in &tx.inputs {
            let outpoint = OutPoint::new(input.previous_output_hash, input.output_index);

            // Remove UTXO from set (and track it for rollback)
            let output = self
                .utxo_set
                .remove(&outpoint)
                .ok_or_else(|| StateError::UTXONotFound(outpoint.clone()))?;

            // Track consumed UTXO for potential rollback
            block_consumed.insert(outpoint, output.clone());

            input_sum = input_sum
                .checked_add(output.amount)
                .ok_or(StateError::ArithmeticOverflow)?;

            // Verify and update nonce if present
            if let Some(nonce) = input.nonce {
                let pubkey_hash = &output.recipient_pubkey_hash;
                let current_nonce = self.account_nonces.get(pubkey_hash).copied().unwrap_or(0);

                if nonce != current_nonce {
                    return Err(StateError::InvalidNonce {
                        expected: current_nonce,
                        got: nonce,
                    });
                }

                // Increment nonce
                self.account_nonces.insert(*pubkey_hash, current_nonce + 1);
            }
        }

        // Add new outputs to UTXO set
        let mut output_sum = 0u64;
        for (index, output) in tx.outputs.iter().enumerate() {
            let outpoint = OutPoint::new(tx_hash, index as u32);

            // Prevent duplicate UTXOs
            if self.utxo_set.contains_key(&outpoint) {
                return Err(StateError::DuplicateUTXO(outpoint));
            }

            self.utxo_set.insert(outpoint, output.clone());

            output_sum = output_sum
                .checked_add(output.amount)
                .ok_or(StateError::ArithmeticOverflow)?;
        }

        // Verify inputs >= outputs (fee is the difference)
        if input_sum < output_sum {
            return Err(StateError::InsufficientInputs {
                inputs: input_sum,
                outputs: output_sum,
            });
        }

        // Calculate and return fee (input - output)
        let fee = input_sum - output_sum;
        Ok(fee)
    }

    /// Apply a proof anchoring transaction
    fn apply_proof_anchor_transaction(
        &mut self,
        tx: &Transaction,
        block_height: u64,
    ) -> Result<u64, StateError> {
        use crate::tx_types::TransactionBuilder;

        // Extract proof data
        let proof_data = TransactionBuilder::extract_proof_data(tx)
            .map_err(|e| StateError::InvalidTransaction(e))?;

        // Create proof anchor
        let proof_anchor = crate::proof::ProofAnchor::new(
            proof_data.identity_id,
            proof_data.proof_type,
            proof_data.proof_hash,
            block_height,
            tx.timestamp,
            proof_data.metadata,
        );

        // Anchor proof in storage
        self.proof_storage.anchor_proof(proof_anchor).map_err(|e| {
            StateError::InvalidTransaction(format!("Proof anchoring failed: {}", e))
        })?;

        // Proof anchoring transactions don't consume/create UTXOs
        // Fixed fee for proof anchoring
        Ok(100)
    }

    /// Apply an asset transfer transaction
    fn apply_asset_transfer_transaction(
        &mut self,
        tx: &Transaction,
        DIFFICULTY_ADJUSTMENT_INTERVAL: u64,
    ) -> Result<u64, StateError> {
        use crate::tx_types::TransactionBuilder;

        // Extract transfer data
        let transfer_data = TransactionBuilder::extract_asset_transfer(tx)
            .map_err(|e| StateError::InvalidTransaction(e))?;

        // Get sender address from input public key
        let sender = if !tx.inputs.is_empty() {
            // Hash the public key to get the sender address
            use sha3::{Digest, Sha3_256};
            let public_key = &tx.inputs[0].public_key;

            if public_key.is_empty() {
                return Err(StateError::InvalidTransaction(
                    "No public key in input".to_string(),
                ));
            }

            // Create sender address by hashing the public key
            let mut hasher = Sha3_256::new();
            hasher.update(public_key);
            hasher.finalize().into()
        } else {
            return Err(StateError::InvalidTransaction(
                "No inputs in asset transfer".to_string(),
            ));
        };

        // Perform asset transfer
        self.asset_registry
            .transfer(
                &sender,
                &transfer_data.recipient,
                &transfer_data.asset_id,
                transfer_data.amount,
            )
            .map_err(|e| StateError::InvalidTransaction(format!("Asset transfer failed: {}", e)))?;

        // Fixed fee for asset transfer
        Ok(100)
    }

    /// Apply an asset registration transaction
    fn apply_asset_register_transaction(
        &mut self,
        tx: &Transaction,
        block_height: u64,
    ) -> Result<u64, StateError> {
        use crate::tx_types::TransactionBuilder;

        // Extract registration data
        let register_data = TransactionBuilder::extract_asset_register(tx)
            .map_err(|e| StateError::InvalidTransaction(e))?;

        // Get issuer address from input public key
        let issuer = if !tx.inputs.is_empty() {
            // Hash the public key to get the issuer address
            use sha3::{Digest, Sha3_256};
            let public_key = &tx.inputs[0].public_key;

            if public_key.is_empty() {
                return Err(StateError::InvalidTransaction(
                    "No public key in input".to_string(),
                ));
            }

            // Create issuer address by hashing the public key
            let mut hasher = Sha3_256::new();
            hasher.update(public_key);
            hasher.finalize().into()
        } else {
            return Err(StateError::InvalidTransaction(
                "No inputs in asset registration".to_string(),
            ));
        };

        // Parse asset type
        let asset_type = crate::asset::AssetType::from_str(&register_data.asset_type);

        // Create asset definition
        let asset_def = crate::asset::AssetDefinition::new(
            asset_type,
            register_data.name,
            register_data.symbol,
            register_data.decimals,
            register_data.total_supply,
            issuer,
            block_height,
            register_data.metadata,
            register_data.transferable,
            register_data.burnable,
            register_data.mintable,
        );

        // Register asset
        self.asset_registry.register_asset(asset_def).map_err(|e| {
            StateError::InvalidTransaction(format!("Asset registration failed: {}", e))
        })?;

        // Fixed fee for asset registration
        Ok(100)
    }

    /// Apply a contract deployment transaction
    fn apply_contract_deployment_transaction(
        &mut self,
        tx: &Transaction,
        block_height: u64,
    ) -> Result<u64, StateError> {
        use crate::tx_types::TransactionBuilder;
        use sha3::{Digest, Sha3_256};

        // Extract deployment data
        let deployment_data = TransactionBuilder::extract_contract_deployment(tx)
            .map_err(|e| StateError::InvalidTransaction(e))?;

        // Derive contract address from transaction hash (SHA3-256)
        let tx_hash = tx.hash();
        let mut hasher = Sha3_256::new();
        hasher.update(&tx_hash);
        let contract_address: [u8; 32] = hasher.finalize().into();

        // Check if contract already exists
        if self.has_contract(&contract_address) {
            return Err(StateError::InvalidTransaction(format!(
                "Contract already exists at address: {}",
                hex::encode(contract_address)
            )));
        }

        // Extract WASM bytecode from transaction output
        // According to architecture: Contract deployment creates a special UTXO with WASM in script field
        let wasm_bytecode = if !tx.outputs.is_empty() {
            tx.outputs[0]
                .script
                .clone()
                .ok_or_else(|| {
                    StateError::InvalidTransaction(
                        "Contract deployment output missing WASM bytecode in script field".to_string(),
                    )
                })?
        } else {
            return Err(StateError::InvalidTransaction(
                "Contract deployment has no outputs".to_string(),
            ));
        };

        // Verify output has CONTRACT_DEPLOYMENT_MARKER
        if tx.outputs[0].recipient_pubkey_hash != crate::contract::CONTRACT_DEPLOYMENT_MARKER {
            return Err(StateError::InvalidTransaction(
                "Contract deployment output missing CONTRACT_DEPLOYMENT_MARKER".to_string(),
            ));
        }

        // Create contract info
        let contract_info = ContractInfo::new(
            contract_address,
            wasm_bytecode,
            deployment_data.deployer,
            block_height,
            tx_hash,
        );

        // Validate WASM bytecode
        contract_info.validate_bytecode().map_err(|e| {
            StateError::InvalidTransaction(format!("WASM validation failed: {}", e))
        })?;

        // Create initial contract state
        let mut contract_state = ContractState::new(contract_address);
        contract_state.last_modified = block_height;

        // If initial state is provided, apply it
        if !deployment_data.initial_state.is_empty() {
            // Deserialize initial state as Vec<StateChange>
            let initial_changes: Vec<crate::contract::StateChange> =
                bincode::deserialize(&deployment_data.initial_state).map_err(|e| {
                    StateError::InvalidTransaction(format!("Invalid initial state: {}", e))
                })?;

            contract_state.apply_changes(&initial_changes).map_err(|e| {
                StateError::InvalidTransaction(format!("Failed to apply initial state: {}", e))
            })?;
        }

        // Register contract
        self.contract_registry
            .insert(contract_address, contract_info);
        self.contract_states.insert(contract_address, contract_state);

        // Fixed fee for contract deployment
        Ok(1000) // Higher fee for deployment (10x standard)
    }

    /// Apply a contract call transaction
    /// Note: This currently processes the transaction structure but doesn't execute WASM
    /// WASM execution will be integrated in Phase 3
    fn apply_contract_call_transaction(
        &mut self,
        tx: &Transaction,
        block_height: u64,
    ) -> Result<u64, StateError> {
        use crate::tx_types::TransactionBuilder;

        // Extract call data
        let call_data = TransactionBuilder::extract_contract_call(tx)
            .map_err(|e| StateError::InvalidTransaction(e))?;

        // Verify contract exists
        if !self.has_contract(&call_data.contract_address) {
            return Err(StateError::InvalidTransaction(format!(
                "Contract not found at address: {}",
                hex::encode(call_data.contract_address)
            )));
        }

        // Update last modified time
        if let Some(state) = self.get_contract_state_mut(&call_data.contract_address) {
            state.last_modified = block_height;
        }

        // TODO (Phase 3): Execute WASM contract with wasm-runtime
        // For now, we just validate the transaction structure and update the last_modified time
        // The actual execution will:
        // 1. Load contract WASM bytecode from contract_registry
        // 2. Call wasm_runtime::execute() with function_name and args
        // 3. Apply returned StateChanges to contract_state
        // 4. Validate storage quotas

        // Fixed fee for contract call
        Ok(500) // Higher than standard, lower than deployment
    }

    /// Rollback a block from the state (for reorgs)
    pub fn rollback_block(&mut self, block: &Block) -> Result<(), StateError> {
        // Validate we're rolling back the correct block
        if block.header.height != self.block_height {
            return Err(StateError::InvalidBlockHeight {
                expected: self.block_height,
                got: block.header.height,
            });
        }

        // Remove outputs created by this block
        for (tx_index, tx) in block.transactions.iter().enumerate() {
            let tx_hash = tx.hash();
            for (index, output) in tx.outputs.iter().enumerate() {
                let outpoint = OutPoint::new(tx_hash, index as u32);
                self.utxo_set.remove(&outpoint);

                // Subtract from total supply (coinbase only)
                if tx_index == 0 {
                    self.total_supply = self.total_supply.saturating_sub(output.amount);
                }
            }
        }

        // Restore inputs consumed by this block
        if let Some(consumed) = self.consumed_utxos.remove(&block.header.height) {
            for (outpoint, output) in consumed {
                // Restore UTXO to the set
                self.utxo_set.insert(outpoint, output);
            }
        }

        // Rollback nonces for accounts that had transactions in this block
        // Note: This is a simplified approach - we decrement nonces for any account
        // that had a transaction in this block. This assumes nonces are sequential.
        for (tx_index, tx) in block.transactions.iter().enumerate() {
            // Skip coinbase
            if tx_index == 0 {
                continue;
            }

            for input in &tx.inputs {
                if input.nonce.is_some() {
                    // Get the pubkey hash from the original UTXO (now restored)
                    let outpoint = OutPoint::new(input.previous_output_hash, input.output_index);
                    if let Some(output) = self.utxo_set.get(&outpoint) {
                        let pubkey_hash = &output.recipient_pubkey_hash;
                        let current_nonce =
                            self.account_nonces.get(pubkey_hash).copied().unwrap_or(0);

                        // Decrement nonce (but don't go below 0)
                        if current_nonce > 0 {
                            self.account_nonces.insert(*pubkey_hash, current_nonce - 1);
                        }
                    }
                }
            }
        }

        // Update state pointers
        if self.block_height > 0 {
            self.block_height -= 1;
        }

        // Update best block hash to previous block's hash
        self.best_block_hash = block.header.previous_hash;

        Ok(())
    }

    /// Get account balance (sum of all UTXOs for this pubkey hash)
    pub fn get_balance(&self, pubkey_hash: &[u8; 32]) -> u64 {
        self.utxo_set
            .values()
            .filter(|output| &output.recipient_pubkey_hash == pubkey_hash)
            .map(|output| output.amount)
            .sum()
    }

    /// Get account nonce
    pub fn get_nonce(&self, pubkey_hash: &[u8; 32]) -> u64 {
        self.account_nonces.get(pubkey_hash).copied().unwrap_or(0)
    }

    /// Check if a UTXO exists
    pub fn has_utxo(&self, outpoint: &OutPoint) -> bool {
        self.utxo_set.contains_key(outpoint)
    }

    /// Get a UTXO
    pub fn get_utxo(&self, outpoint: &OutPoint) -> Option<&TxOutput> {
        self.utxo_set.get(outpoint)
    }

    /// Get all UTXOs for an account
    pub fn get_utxos(&self, pubkey_hash: &[u8; 32]) -> Vec<(OutPoint, TxOutput)> {
        self.utxo_set
            .iter()
            .filter(|(_, output)| &output.recipient_pubkey_hash == pubkey_hash)
            .map(|(outpoint, output)| (outpoint.clone(), output.clone()))
            .collect()
    }

    /// Get total number of UTXOs
    pub fn utxo_count(&self) -> usize {
        self.utxo_set.len()
    }

    /// Calculate transaction fee (inputs - outputs)
    /// Requires blockchain state access to look up input amounts
    pub fn calculate_transaction_fee(&self, tx: &Transaction) -> Result<u64, StateError> {
        // Coinbase transactions have no fee
        if tx.is_coinbase() {
            return Ok(0);
        }

        let mut input_sum = 0u64;
        for input in &tx.inputs {
            let outpoint = OutPoint::new(input.previous_output_hash, input.output_index);
            let output = self
                .get_utxo(&outpoint)
                .ok_or_else(|| StateError::UTXONotFound(outpoint))?;
            input_sum = input_sum
                .checked_add(output.amount)
                .ok_or(StateError::ArithmeticOverflow)?;
        }

        let output_sum: u64 = tx.outputs.iter().map(|o| o.amount).sum();

        // Fee is the difference between inputs and outputs
        Ok(input_sum.saturating_sub(output_sum))
    }

    /// Validate transaction fee meets minimum requirements
    /// Minimum fee is calculated as: tx_size_bytes * MIN_FEE_PER_BYTE
    pub fn validate_transaction_fee(&self, tx: &Transaction) -> Result<(), StateError> {
        const MIN_FEE_PER_BYTE: u64 = 100; // satoshis per byte

        // Skip fee validation for coinbase transactions
        if tx.is_coinbase() {
            return Ok(());
        }

        let fee = self.calculate_transaction_fee(tx)?;
        let tx_size = tx.size_bytes() as u64;
        let min_fee = tx_size.saturating_mul(MIN_FEE_PER_BYTE);

        if fee < min_fee {
            return Err(StateError::InsufficientFee {
                required: min_fee,
                provided: fee,
            });
        }

        Ok(())
    }
}

impl Default for BlockchainState {
    fn default() -> Self {
        Self::new()
    }
}

/// State management errors
#[derive(Debug, thiserror::Error)]
pub enum StateError {
    #[error("UTXO not found: tx_hash={}, index={}", hex::encode(.0.tx_hash), .0.index)]
    UTXONotFound(OutPoint),

    #[error("Invalid block height: expected {expected}, got {got}")]
    InvalidBlockHeight { expected: u64, got: u64 },

    #[error("Invalid previous block hash")]
    InvalidPreviousHash,

    #[error("Invalid nonce: expected {expected}, got {got}")]
    InvalidNonce { expected: u64, got: u64 },

    #[error("Insufficient inputs: inputs={inputs}, outputs={outputs}")]
    InsufficientInputs { inputs: u64, outputs: u64 },

    #[error("Insufficient fee: required={required}, provided={provided}")]
    InsufficientFee { required: u64, provided: u64 },

    #[error("Arithmetic overflow")]
    ArithmeticOverflow,

    #[error("Invalid coinbase: {0}")]
    InvalidCoinbase(String),

    #[error("Duplicate UTXO: tx_hash={}, index={}", hex::encode(.0.tx_hash), .0.index)]
    DuplicateUTXO(OutPoint),

    #[error("Invalid transaction: {0}")]
    InvalidTransaction(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Block, BlockHeader, Transaction, TxOutput};

    #[test]
    fn test_state_creation() {
        let state = BlockchainState::new();
        assert_eq!(state.height(), 0);
        assert_eq!(state.total_supply(), 0);
        assert_eq!(state.utxo_count(), 0);
    }

    #[test]
    fn test_apply_coinbase() {
        let mut state = BlockchainState::new();

        let coinbase_output = TxOutput {
            amount: 5_000_000_000,
            recipient_pubkey_hash: [1u8; 32],
            script: None,
        };

        let coinbase_tx = Transaction::new(1, vec![], vec![coinbase_output], 1234567890, None);

        let block = Block::new(
            BlockHeader::new(1, [0u8; 32], [0u8; 32], 1234567890, 0x1f0fffff, 0, 1),
            vec![coinbase_tx],
        );

        assert!(state.apply_block(&block).is_ok());
        assert_eq!(state.height(), 1);
        assert_eq!(state.total_supply(), 5_000_000_000);
        assert_eq!(state.get_balance(&[1u8; 32]), 5_000_000_000);
    }

    #[test]
    fn test_utxo_tracking() {
        let mut state = BlockchainState::new();

        // Create genesis block
        let coinbase_tx = Transaction::new(
            1,
            vec![],
            vec![TxOutput {
                amount: 5_000_000_000,
                recipient_pubkey_hash: [1u8; 32],
                script: None,
            }],
            1234567890,
            None,
        );

        let genesis = Block::new(
            BlockHeader::new(1, [0u8; 32], [0u8; 32], 1234567890, 0x1f0fffff, 0, 1),
            vec![coinbase_tx.clone()],
        );

        state.apply_block(&genesis).unwrap();

        // Verify UTXO exists
        let outpoint = OutPoint::new(coinbase_tx.hash(), 0);
        assert!(state.has_utxo(&outpoint));
        assert_eq!(state.utxo_count(), 1);
    }

    #[test]
    fn test_insufficient_inputs() {
        let mut state = BlockchainState::new();

        // Create a transaction with outputs > inputs (should fail)
        let tx = Transaction::new(
            1,
            vec![], // No inputs
            vec![TxOutput {
                amount: 1000,
                recipient_pubkey_hash: [2u8; 32],
                script: None,
            }],
            1234567890,
            None,
        );

        // This should fail because it's not a coinbase but has no inputs
        let result = state.apply_transaction(&tx, 1);
        assert!(result.is_err());
    }
}
