// Blockchain management and state transitions
use crate::config::NodeConfig;
use anyhow::Result;
use boundless_consensus::DifficultyAdjustment;
use boundless_core::{Block, BlockHeader, BlockchainState, OutPoint, Transaction, TxOutput};
use boundless_storage::Database;
use sha3::{Digest, Sha3_256};
use std::collections::HashMap;
use std::path::PathBuf;

pub struct Blockchain {
    /// Current blockchain state
    state: BlockchainState,

    /// Configuration
    config: NodeConfig,

    /// Data directory
    data_dir: PathBuf,

    /// Persistent storage
    storage: Option<Database>,

    /// Block cache (height -> block)
    block_cache: HashMap<u64, Block>,

    /// Pending transactions for next block
    pending_txs: Vec<Transaction>,

    /// HIGH PRIORITY FIX: Fork tracking
    /// Maps block hash -> block for blocks not on main chain
    fork_blocks: HashMap<[u8; 32], Block>,

    /// HIGH PRIORITY FIX: Orphan blocks
    /// Blocks whose parent is not yet known (hash -> block)
    orphan_blocks: HashMap<[u8; 32], Block>,

    /// HIGH PRIORITY FIX: Checkpoints
    /// Maps block height -> block hash for immutable checkpoints
    /// Prevents reorganizations past these points
    checkpoints: HashMap<u64, [u8; 32]>,

    /// HIGH PRIORITY FIX: Checkpoint interval
    /// Number of blocks between auto-checkpoints (0 = disabled)
    checkpoint_interval: u64,
}

impl Blockchain {
    /// Create a new blockchain or load existing one
    pub fn new(data_dir: PathBuf, config: NodeConfig) -> Result<Self> {
        // Initialize storage
        let storage_config = boundless_storage::DatabaseConfig {
            path: data_dir.join("db").to_str().unwrap().to_string(),
            cache_size_mb: config.storage.cache_size_mb,
            enable_compression: true,
            max_open_files: 1000,
        };

        let storage = Database::open(storage_config)?;

        // Try to load existing state
        let (state, genesis_hash) = if let Some(saved_state) = storage.load_state()? {
            tracing::info!("📖 Loaded existing blockchain state");
            // Load genesis to get its hash for checkpoint
            let genesis = storage.get_block_by_height(0)?.ok_or_else(|| {
                anyhow::anyhow!("Genesis block not found in storage")
            })?;
            (saved_state, genesis.header.hash())
        } else {
            tracing::info!("🆕 Creating genesis block");
            let genesis = Self::create_genesis_block()?;
            let genesis_hash = genesis.header.hash();
            storage.store_block(&genesis)?;
            let state = BlockchainState::with_genesis(&genesis)?;
            (state, genesis_hash)
        };

        // HIGH PRIORITY FIX: Initialize checkpoint system with genesis
        let mut checkpoints = HashMap::new();
        checkpoints.insert(0, genesis_hash); // Genesis is always a checkpoint
        tracing::info!("🔒 Genesis checkpoint set: {}", hex::encode(&genesis_hash[..8]));

        let checkpoint_interval = config.operational.checkpoint_interval;
        if checkpoint_interval > 0 {
            tracing::info!("✅ Auto-checkpointing enabled (every {} blocks)", checkpoint_interval);
        } else {
            tracing::warn!("⚠️  Auto-checkpointing disabled (checkpoint_interval = 0)");
        }

        Ok(Self {
            state,
            config,
            data_dir,
            storage: Some(storage),
            block_cache: HashMap::new(),
            pending_txs: Vec::new(),
            fork_blocks: HashMap::new(),
            orphan_blocks: HashMap::new(),
            checkpoints,
            checkpoint_interval,
        })
    }

    /// Create the genesis block
    fn create_genesis_block() -> Result<Block> {
        // Coinbase transaction for genesis
        let coinbase_tx = Transaction::new(
            1,
            vec![], // No inputs for coinbase
            vec![TxOutput {
                amount: 5_000_000_000,            // 50 BLS
                recipient_pubkey_hash: [0u8; 32], // Genesis address
                script: None,
            }],
            0, // Genesis timestamp
            None,
        );

        let genesis_header = BlockHeader::new(
            1,                  // version
            [0u8; 32],          // previous hash (none for genesis)
            coinbase_tx.hash(), // merkle root (single transaction)
            0,                  // timestamp
            0x1f0fffff,         // difficulty (very easy)
            0,                  // nonce
            1,                  // height (genesis is height 1)
        );

        Ok(Block::new(genesis_header, vec![coinbase_tx]))
    }

    /// Get current blockchain height
    pub fn height(&self) -> u64 {
        self.state.height()
    }

    /// Get best block hash
    pub fn best_block_hash(&self) -> [u8; 32] {
        self.state.best_block_hash()
    }

    /// Get total supply
    pub fn total_supply(&self) -> u64 {
        self.state.total_supply()
    }

    /// Get account balance
    pub fn get_balance(&self, address: &[u8; 32]) -> u64 {
        self.state.get_balance(address)
    }

    /// Get account nonce
    pub fn get_nonce(&self, address: &[u8; 32]) -> u64 {
        self.state.get_nonce(address)
    }

    /// Check if UTXO exists
    pub fn has_utxo(&self, outpoint: &OutPoint) -> bool {
        self.state.has_utxo(outpoint)
    }

    /// Get a reference to the blockchain state
    pub fn state(&self) -> &BlockchainState {
        &self.state
    }

    /// Get the number of fork blocks being tracked
    pub fn fork_blocks_count(&self) -> usize {
        self.fork_blocks.len()
    }

    /// Get the number of orphan blocks being tracked
    pub fn orphan_blocks_count(&self) -> usize {
        self.orphan_blocks.len()
    }

    /// Create the next block with given transactions
    pub async fn create_next_block(
        &self,
        coinbase_address: [u8; 32],
        mut transactions: Vec<Transaction>,
    ) -> Result<Block> {
        let current_height = self.state.height();
        let next_height = current_height + 1;

        // Calculate difficulty for next block
        let difficulty = if DifficultyAdjustment::should_adjust(next_height) {
            // Adjustment interval reached - recalculate difficulty
            // Get the previous block's difficulty (current difficulty)
            let current_difficulty = if let Some(prev_block) = self.get_block(current_height) {
                prev_block.header.difficulty_target
            } else {
                0x1f0fffff // Fallback to genesis difficulty
            };

            // Calculate epoch timing
            let adjustment_start_height =
                next_height - boundless_consensus::DIFFICULTY_ADJUSTMENT_INTERVAL;

            // Get blocks from epoch start and end
            let epoch_start_block = self.get_block(adjustment_start_height);
            let epoch_end_block = self.get_block(current_height);

            if let (Some(start_block), Some(end_block)) = (epoch_start_block, epoch_end_block) {
                let actual_time = end_block.header.timestamp - start_block.header.timestamp;
                let expected_time = DifficultyAdjustment::expected_epoch_time();

                let new_difficulty = DifficultyAdjustment::adjust_difficulty(
                    current_difficulty,
                    actual_time,
                    expected_time,
                );

                tracing::info!(
                    "⚖️  Difficulty adjustment at height {}:\n   Actual epoch time: {}s (expected {}s)\n   Old difficulty: 0x{:08x}\n   New difficulty: 0x{:08x}",
                    next_height,
                    actual_time,
                    expected_time,
                    current_difficulty,
                    new_difficulty
                );

                new_difficulty
            } else {
                // If we can't retrieve blocks, keep current difficulty
                tracing::warn!("⚠️  Could not retrieve epoch blocks for difficulty adjustment, keeping current difficulty");
                current_difficulty
            }
        } else {
            // No adjustment needed - use previous block's difficulty
            if let Some(prev_block) = self.get_block(current_height) {
                prev_block.header.difficulty_target
            } else {
                0x1f0fffff // Fallback to genesis difficulty
            }
        };

        // Create coinbase transaction
        let coinbase_tx = Transaction::new(
            1,
            vec![],
            vec![TxOutput {
                amount: 5_000_000_000, // 50 BLS block reward
                recipient_pubkey_hash: coinbase_address,
                script: None,
            }],
            chrono::Utc::now().timestamp() as u64,
            None,
        );

        // Add coinbase as first transaction
        transactions.insert(0, coinbase_tx);

        // Calculate merkle root
        let tx_hashes: Vec<Vec<u8>> = transactions.iter().map(|tx| tx.hash().to_vec()).collect();
        let merkle_root = if !tx_hashes.is_empty() {
            boundless_core::MerkleTree::new(tx_hashes).root()
        } else {
            [0u8; 32]
        };

        // Create block header
        let header = BlockHeader::new(
            1, // version
            self.state.best_block_hash(),
            merkle_root,
            chrono::Utc::now().timestamp() as u64,
            difficulty,
            0,           // Nonce will be set by miner
            next_height, // height
        );

        Ok(Block::new(header, transactions))
    }

    /// Apply a mined block to the blockchain
    pub async fn apply_block(&mut self, block: &Block) -> Result<()> {
        self.state.apply_block(block)?;

        // Persist block and state to storage
        if let Some(storage) = &self.storage {
            storage.store_block(block)?;
            storage.store_state(&self.state)?;

            // Store transactions
            for tx in &block.transactions {
                storage.store_transaction(tx, block.header.height)?;
            }
        }

        // Add to cache
        self.block_cache.insert(block.header.height, block.clone());

        // HIGH PRIORITY FIX: Auto-checkpoint at intervals
        self.auto_checkpoint()?;

        Ok(())
    }

    /// Get block from cache or storage
    pub fn get_block(&self, height: u64) -> Option<Block> {
        // Check cache first
        if let Some(block) = self.block_cache.get(&height) {
            return Some(block.clone());
        }

        // Check storage
        if let Some(storage) = &self.storage {
            storage.get_block_by_height(height).ok().flatten()
        } else {
            None
        }
    }

    /// Get block by hash
    pub fn get_block_by_hash(&self, hash: &[u8; 32]) -> Option<Block> {
        if let Some(storage) = &self.storage {
            storage.get_block_by_hash(hash).ok().flatten()
        } else {
            None
        }
    }

    /// Add transaction to pending list
    pub fn add_pending_transaction(&mut self, tx: Transaction) -> Result<()> {
        // Validate transaction
        self.validate_transaction(&tx)?;
        self.pending_txs.push(tx);
        Ok(())
    }

    /// Get pending transactions
    pub fn pending_transactions(&self) -> &[Transaction] {
        &self.pending_txs
    }

    /// Clear pending transactions
    pub fn clear_pending(&mut self) {
        self.pending_txs.clear();
    }

    /// Validate a transaction against current state
    pub fn validate_transaction(&self, tx: &Transaction) -> Result<bool> {
        // Basic structure validation
        tx.validate()?;

        // Verify all inputs exist in UTXO set and validate signatures
        for (input_index, input) in tx.inputs.iter().enumerate() {
            // FIX N2: Use correct field name (previous_output_hash, not previous_tx_hash)
            let outpoint = OutPoint::new(input.previous_output_hash, input.output_index);

            if !self.state.has_utxo(&outpoint) {
                anyhow::bail!("Input UTXO not found: {:?}", outpoint);
            }

            // FIX C3: Implement signature verification
            if let Some(utxo) = self.state.get_utxo(&outpoint) {
                // Verify the signature is valid
                let signature_valid = tx
                    .verify_input_signature(input_index, &input.public_key)
                    .map_err(|e| anyhow::anyhow!("Signature verification failed: {}", e))?;

                if !signature_valid {
                    anyhow::bail!("Invalid signature for input {}", input_index);
                }

                // Verify the public key matches the UTXO recipient
                let mut hasher = Sha3_256::new();
                hasher.update(&input.public_key);
                let pubkey_hash: [u8; 32] = hasher.finalize().into();

                if pubkey_hash != utxo.recipient_pubkey_hash {
                    anyhow::bail!(
                        "Public key does not match UTXO recipient for input {}. Expected: {:?}, Got: {:?}",
                        input_index,
                        utxo.recipient_pubkey_hash,
                        pubkey_hash
                    );
                }
            }
        }

        Ok(true)
    }

    /// Get the current difficulty target from the latest block
    pub fn current_difficulty(&self) -> u32 {
        if let Some(block) = self.get_block(self.height()) {
            block.header.difficulty_target
        } else {
            0x1f0fffff // Default genesis difficulty
        }
    }

    /// HIGH PRIORITY FIX: Add block with reorganization handling
    /// This is the primary method for accepting blocks from P2P network
    ///
    /// Handles three scenarios:
    /// 1. Block extends current chain (simple case)
    /// 2. Block creates/extends a fork (track and possibly reorganize)
    /// 3. Block is orphaned (parent unknown - store for later)
    pub fn add_block(&mut self, block: Block) -> Result<()> {
        let block_hash = block.header.hash();
        let block_height = block.header.height;
        let prev_hash = block.header.previous_hash;

        tracing::debug!(
            "Processing block #{} (hash: {})",
            block_height,
            hex::encode(&block_hash[..8])
        );

        // Validate block structure and PoW
        self.validate_block(&block)?;

        // Check if we already have this block
        if self.get_block_by_hash(&block_hash).is_some() {
            tracing::debug!("Block already known, ignoring");
            return Ok(());
        }

        // CASE 1: Block extends current best chain
        if block.header.height == self.state.height() + 1
            && block.header.previous_hash == self.state.best_block_hash()
        {
            tracing::info!(
                "✅ Block #{} extends main chain",
                block.header.height
            );
            return self.apply_block_to_chain(block);
        }

        // CASE 2: Block's parent is unknown - it's an orphan
        if self.get_block_by_hash(&prev_hash).is_none() {
            tracing::warn!(
                "📦 Orphan block #{} (parent {} unknown)",
                block_height,
                hex::encode(&prev_hash[..8])
            );
            self.orphan_blocks.insert(block_hash, block);
            return Ok(());
        }

        // CASE 3: Block creates or extends a fork
        tracing::info!(
            "🔀 Fork block #{} (parent: {})",
            block_height,
            hex::encode(&prev_hash[..8])
        );

        // Store the fork block
        self.fork_blocks.insert(block_hash, block.clone());

        // Check if this fork is now longer than main chain
        self.check_reorganization(&block)?;

        // Check if any orphans can now be connected
        self.process_orphans(&block_hash)?;

        Ok(())
    }

    /// Apply a block directly to the main chain
    fn apply_block_to_chain(&mut self, block: Block) -> Result<()> {
        // Apply to state
        self.state.apply_block(&block)?;

        // Persist to storage
        if let Some(storage) = &self.storage {
            storage.store_block(&block)?;
            storage.store_state(&self.state)?;

            for tx in &block.transactions {
                storage.store_transaction(tx, block.header.height)?;
            }
        }

        // Cache the block
        self.block_cache.insert(block.header.height, block);

        // HIGH PRIORITY FIX: Auto-checkpoint at intervals
        self.auto_checkpoint()?;

        Ok(())
    }

    /// Validate block structure and proof of work
    fn validate_block(&self, block: &Block) -> Result<()> {
        // HIGH PRIORITY FIX: Validate against checkpoints first
        self.validate_checkpoint(block)?;

        // Verify PoW
        let block_hash = block.header.hash();
        let target = BlockHeader::compact_to_target(block.header.difficulty_target);

        if block_hash > target {
            anyhow::bail!(
                "Invalid proof of work: block hash {} exceeds target {}",
                hex::encode(&block_hash[..8]),
                hex::encode(&target[..8])
            );
        }

        // Validate transactions
        for tx in &block.transactions {
            tx.validate()?;
        }

        // Additional validation can be added here
        // - Merkle root verification
        // - Timestamp checks
        // - Coinbase validation

        Ok(())
    }

    /// Check if a fork chain is longer and perform reorganization if needed
    fn check_reorganization(&mut self, fork_tip: &Block) -> Result<()> {
        // Calculate chain work from fork tip back to common ancestor
        let fork_work = self.calculate_chain_work_from_fork(fork_tip)?;
        let main_work = self.calculate_main_chain_work();

        if fork_work > main_work {
            tracing::warn!(
                "⚠️  Fork has more work ({} vs {}), reorganizing...",
                fork_work,
                main_work
            );
            self.reorganize_to_fork(fork_tip)?;
        } else {
            tracing::debug!(
                "Fork has less work ({} vs {}), staying on main chain",
                fork_work,
                main_work
            );
        }

        Ok(())
    }

    /// Calculate cumulative work for main chain
    fn calculate_main_chain_work(&self) -> u64 {
        // Calculate cumulative work by summing difficulty of all blocks
        let mut total_work = 0u64;
        let current_height = self.state.height();

        // Iterate through all blocks and sum their difficulty
        for height in 0..=current_height {
            if let Ok(block) = self.get_block_by_height(height) {
                // Work = difficulty target (simplified)
                // In a full implementation: work = 2^256 / (target + 1)
                total_work = total_work.saturating_add(block.header.difficulty_target as u64);
            }
        }

        total_work
    }

    /// Calculate cumulative work from a fork block back to common ancestor
    fn calculate_chain_work_from_fork(&self, fork_tip: &Block) -> Result<u64> {
        let mut work = 0u64;
        let mut current_hash = fork_tip.header.hash();
        let mut visited = 0;

        // Walk back through fork blocks until we hit main chain
        loop {
            visited += 1;
            if visited > 1000 {
                // Prevent infinite loops from malicious data
                anyhow::bail!("Fork chain too deep (>1000 blocks)");
            }

            // Check if this block is on main chain
            if let Some(main_block) = self.get_block_by_hash(&current_hash) {
                // Hit main chain - count blocks from here
                work += main_block.header.height;
                break;
            }

            // Check fork blocks
            if let Some(fork_block) = self.fork_blocks.get(&current_hash) {
                work += 1; // Count this fork block
                current_hash = fork_block.header.previous_hash;
            } else {
                // Shouldn't happen - fork chain is incomplete
                anyhow::bail!("Incomplete fork chain");
            }
        }

        Ok(work)
    }

    /// Perform chain reorganization to switch to a fork
    fn reorganize_to_fork(&mut self, fork_tip: &Block) -> Result<()> {
        // Find common ancestor
        let (blocks_to_undo, blocks_to_apply) = self.find_reorganization_path(fork_tip)?;

        // HIGH PRIORITY FIX: Checkpoint validation
        // Prevent reorganizations that would undo checkpointed blocks
        for block in &blocks_to_undo {
            if let Some(checkpoint_hash) = self.checkpoints.get(&block.header.height) {
                let block_hash = block.header.hash();
                if *checkpoint_hash == block_hash {
                    tracing::error!(
                        "❌ Cannot reorganize past checkpoint at height {} (hash: {})",
                        block.header.height,
                        hex::encode(&checkpoint_hash[..8])
                    );
                    anyhow::bail!(
                        "Reorganization rejected: would undo checkpoint at height {}",
                        block.header.height
                    );
                }
            }
        }

        tracing::warn!(
            "🔄 Reorganizing: undoing {} blocks, applying {} blocks",
            blocks_to_undo.len(),
            blocks_to_apply.len()
        );

        // Step 1: Undo blocks from current chain back to common ancestor
        for block in blocks_to_undo.iter().rev() {
            tracing::info!("↩️  Undoing block #{}", block.header.height);
            self.undo_block(block)?;
        }

        // Step 2: Apply blocks from fork forward from common ancestor
        for block in &blocks_to_apply {
            tracing::info!("✅ Applying fork block #{}", block.header.height);
            self.apply_block_to_chain(block.clone())?;
        }

        // Step 3: Move old main chain blocks to fork_blocks
        for block in blocks_to_undo {
            let hash = block.header.hash();
            self.fork_blocks.insert(hash, block);
        }

        // Step 4: Remove newly-main-chain blocks from fork_blocks
        for block in &blocks_to_apply {
            let hash = block.header.hash();
            self.fork_blocks.remove(&hash);
        }

        tracing::warn!("✅ Reorganization complete");

        Ok(())
    }

    /// Find the path for reorganization
    /// Returns (blocks to undo from main chain, blocks to apply from fork)
    fn find_reorganization_path(
        &self,
        fork_tip: &Block,
    ) -> Result<(Vec<Block>, Vec<Block>)> {
        let mut fork_chain = vec![fork_tip.clone()];
        let mut current_hash = fork_tip.header.previous_hash;

        // Walk back through fork until we hit main chain
        loop {
            if let Some(main_block) = self.get_block_by_hash(&current_hash) {
                // Found common ancestor on main chain
                let common_ancestor_height = main_block.header.height;

                // Collect main chain blocks to undo
                let mut blocks_to_undo = Vec::new();
                for height in (common_ancestor_height + 1)..=self.state.height() {
                    if let Some(block) = self.get_block(height) {
                        blocks_to_undo.push(block);
                    }
                }

                // Fork chain is already collected (in reverse order)
                fork_chain.reverse();

                return Ok((blocks_to_undo, fork_chain));
            }

            // Continue walking back through fork
            if let Some(fork_block) = self.fork_blocks.get(&current_hash) {
                fork_chain.push(fork_block.clone());
                current_hash = fork_block.header.previous_hash;
            } else {
                anyhow::bail!("Cannot find common ancestor");
            }
        }
    }

    /// Undo a block from the chain (for reorganization)
    fn undo_block(&mut self, block: &Block) -> Result<()> {
        tracing::info!(
            "↩️  Rolling back block #{} (hash: {})",
            block.header.height,
            hex::encode(&block.header.hash()[..8])
        );

        // Use the state's rollback_block method to properly undo:
        // 1. Restore spent UTXOs from consumed_utxos tracking
        // 2. Remove created UTXOs
        // 3. Revert balance changes
        // 4. Remove transaction index entries
        // 5. Update block height and best hash
        self.state
            .rollback_block(block)
            .map_err(|e| anyhow::anyhow!("Failed to rollback block: {}", e))?;

        // Remove block from cache
        self.block_cache.remove(&block.header.height);

        // Remove from storage if present
        if let Some(storage) = &self.storage {
            // Note: Storage removal is optional - we can keep historical data
            // For now, we'll keep the block data but mark the state as rolled back
            storage.store_state(&self.state)?;
        }

        tracing::debug!("✅ Successfully rolled back block #{}", block.header.height);
        Ok(())
    }

    /// Process orphan blocks that might now have their parent
    fn process_orphans(&mut self, new_block_hash: &[u8; 32]) -> Result<()> {
        let mut orphans_to_process = Vec::new();

        // Find orphans whose parent is the new block
        for (orphan_hash, orphan_block) in &self.orphan_blocks {
            if orphan_block.header.previous_hash == *new_block_hash {
                orphans_to_process.push((*orphan_hash, orphan_block.clone()));
            }
        }

        // Process each orphan
        for (orphan_hash, orphan_block) in orphans_to_process {
            tracing::info!(
                "📦 Processing orphan block #{} (parent found)",
                orphan_block.header.height
            );

            self.orphan_blocks.remove(&orphan_hash);

            // Recursively add the orphan (it might extend the chain or create a fork)
            self.add_block(orphan_block)?;
        }

        Ok(())
    }

    /// Get best block hash (for compatibility)
    pub fn get_best_block_hash(&self) -> [u8; 32] {
        self.best_block_hash()
    }

    /// HIGH PRIORITY FIX: Add a checkpoint at a specific height
    /// Checkpoints prevent reorganizations past that point
    /// Returns Ok if checkpoint was added, Err if block at that height doesn't exist
    pub fn add_checkpoint(&mut self, height: u64) -> Result<()> {
        // Get the block at this height
        let block = self.get_block(height).ok_or_else(|| {
            anyhow::anyhow!("Cannot add checkpoint: block at height {} not found", height)
        })?;

        let block_hash = block.header.hash();

        // Check if there's already a checkpoint at this height
        if let Some(existing_hash) = self.checkpoints.get(&height) {
            if *existing_hash != block_hash {
                anyhow::bail!(
                    "Checkpoint conflict at height {}: existing {} vs new {}",
                    height,
                    hex::encode(&existing_hash[..8]),
                    hex::encode(&block_hash[..8])
                );
            }
            // Same checkpoint already exists
            return Ok(());
        }

        self.checkpoints.insert(height, block_hash);
        tracing::info!(
            "🔒 Checkpoint added at height {} (hash: {})",
            height,
            hex::encode(&block_hash[..8])
        );

        Ok(())
    }

    /// HIGH PRIORITY FIX: Automatically create checkpoints at intervals
    /// This should be called after applying a new block to main chain
    /// Creates a checkpoint every N blocks (configurable via checkpoint_interval)
    pub fn auto_checkpoint(&mut self) -> Result<()> {
        // Skip if auto-checkpointing is disabled
        if self.checkpoint_interval == 0 {
            return Ok(());
        }

        let current_height = self.state.height();

        // Check if current height is a checkpoint interval
        if current_height > 0 && current_height % self.checkpoint_interval == 0 {
            // Only add if not already checkpointed
            if !self.checkpoints.contains_key(&current_height) {
                self.add_checkpoint(current_height)?;
                tracing::info!(
                    "✅ Auto-checkpoint created at height {} (every {} blocks)",
                    current_height,
                    self.checkpoint_interval
                );
            }
        }

        Ok(())
    }

    /// HIGH PRIORITY FIX: Validate a block against checkpoints
    /// Ensures that blocks at checkpoint heights match the checkpoint hash
    pub fn validate_checkpoint(&self, block: &Block) -> Result<()> {
        if let Some(checkpoint_hash) = self.checkpoints.get(&block.header.height) {
            let block_hash = block.header.hash();
            if *checkpoint_hash != block_hash {
                anyhow::bail!(
                    "Block #{} fails checkpoint validation: expected hash {}, got {}",
                    block.header.height,
                    hex::encode(&checkpoint_hash[..8]),
                    hex::encode(&block_hash[..8])
                );
            }
        }
        Ok(())
    }

    /// Get all checkpoints (for debugging/monitoring)
    pub fn get_checkpoints(&self) -> &HashMap<u64, [u8; 32]> {
        &self.checkpoints
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::RwLock;
    use tempfile::TempDir;

    async fn create_test_blockchain() -> (Blockchain, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let config = NodeConfig::default();
        let blockchain = Blockchain::new(temp_dir.path().to_path_buf(), config).unwrap();
        (blockchain, temp_dir)
    }

    // SECURITY TESTS: Optimistic Mining Pattern

    #[tokio::test]
    async fn test_create_next_block_with_read_lock() {
        // Test that create_next_block() works with just a read lock (not write)
        let (blockchain, _temp) = create_test_blockchain().await;
        let blockchain = Arc::new(RwLock::new(blockchain));

        let coinbase = [1u8; 32];
        let transactions = vec![];

        // Acquire READ lock and create block
        {
            let chain = blockchain.read().await;
            let result = chain.create_next_block(coinbase, transactions).await;

            // Should succeed with just read access
            assert!(result.is_ok());
            let block = result.unwrap();
            assert_eq!(block.header.height, 1); // Genesis is 0, next is 1
        }
    }

    #[tokio::test]
    async fn test_concurrent_reads_during_block_creation() {
        // Test that multiple readers can access blockchain while create_next_block is running
        let (blockchain, _temp) = create_test_blockchain().await;
        let blockchain = Arc::new(RwLock::new(blockchain));

        let coinbase = [1u8; 32];

        // Spawn multiple concurrent read tasks
        let mut handles = vec![];

        for i in 0..5 {
            let bc_clone = blockchain.clone();
            let handle = tokio::spawn(async move {
                let chain = bc_clone.read().await;

                // Each task can read blockchain state concurrently
                let height = chain.height();
                let hash = chain.best_block_hash();
                let supply = chain.total_supply();

                // Create block while holding read lock (simulates mining preparation)
                let result = chain.create_next_block([i as u8; 32], vec![]).await;

                assert!(result.is_ok());
                (height, hash, supply)
            });
            handles.push(handle);
        }

        // All tasks should complete successfully (no deadlocks)
        let results = futures::future::join_all(handles).await;

        for result in results {
            assert!(result.is_ok());
            let (height, _hash, supply) = result.unwrap();
            assert_eq!(height, 0); // Genesis
            assert_eq!(supply, 5_000_000_000); // Genesis supply
        }
    }

    #[tokio::test]
    async fn test_optimistic_mining_state_check() {
        // Test the optimistic concurrency check: verify state before applying block
        let (blockchain, _temp) = create_test_blockchain().await;
        let blockchain = Arc::new(RwLock::new(blockchain));

        let coinbase = [1u8; 32];
        let transactions = vec![];

        // Step 1: Capture blockchain state (simulates mining start)
        let (expected_height, expected_hash, candidate_block) = {
            let chain = blockchain.read().await;
            let height = chain.height();
            let hash = chain.best_block_hash();
            let block = chain.create_next_block(coinbase, transactions.clone()).await.unwrap();
            (height, hash, block)
        };

        // Step 2: Verify state hasn't changed (optimistic check)
        {
            let chain = blockchain.read().await;
            let current_height = chain.height();
            let current_hash = chain.best_block_hash();

            // State should be unchanged
            assert_eq!(current_height, expected_height);
            assert_eq!(current_hash, expected_hash);
        }

        // Step 3: Apply the block (with write lock)
        {
            let mut chain = blockchain.write().await;

            // Re-verify state before applying
            let current_height = chain.height();
            let current_hash = chain.best_block_hash();

            assert_eq!(current_height, expected_height);
            assert_eq!(current_hash, expected_hash);

            // Safe to apply
            let result = chain.apply_block(&candidate_block).await;
            assert!(result.is_ok());
        }

        // Verify block was applied
        {
            let chain = blockchain.read().await;
            assert_eq!(chain.height(), 1); // Height increased
        }
    }

    #[tokio::test]
    async fn test_optimistic_mining_state_changed() {
        // Test that we can detect when blockchain state changes during mining
        let (blockchain, _temp) = create_test_blockchain().await;
        let blockchain = Arc::new(RwLock::new(blockchain));

        let coinbase1 = [1u8; 32];
        let coinbase2 = [2u8; 32];

        // Miner 1: Capture state and create candidate block
        let (expected_height, expected_hash, candidate_block1) = {
            let chain = blockchain.read().await;
            let height = chain.height();
            let hash = chain.best_block_hash();
            let block = chain.create_next_block(coinbase1, vec![]).await.unwrap();
            (height, hash, block)
        };

        // Meanwhile, Miner 2 mines and applies a block (simulates another miner winning)
        {
            let mut chain = blockchain.write().await;
            let block2 = chain.create_next_block(coinbase2, vec![]).await.unwrap();
            chain.apply_block(&block2).await.unwrap();
        }

        // Miner 1 tries to apply their block
        {
            let mut chain = blockchain.write().await;
            let current_height = chain.height();
            let current_hash = chain.best_block_hash();

            // OPTIMISTIC CHECK: State has changed!
            let state_changed = current_height != expected_height || current_hash != expected_hash;

            assert!(state_changed, "Blockchain state should have changed");
            assert_eq!(current_height, 1); // Miner 2's block applied
            assert_ne!(current_height, expected_height); // Height changed from 0 to 1

            // In real mining loop, we would discard candidate_block1 and restart
            // For this test, we verify the state change was detected
        }
    }

    #[tokio::test]
    async fn test_optimistic_mining_hash_changed() {
        // Test detection of hash change even if height stays same
        let (blockchain, _temp) = create_test_blockchain().await;
        let blockchain = Arc::new(RwLock::new(blockchain));

        // Capture initial state
        let (expected_height, expected_hash) = {
            let chain = blockchain.read().await;
            (chain.height(), chain.best_block_hash())
        };

        // Apply a block (changes hash)
        {
            let mut chain = blockchain.write().await;
            let block = chain.create_next_block([1u8; 32], vec![]).await.unwrap();
            chain.apply_block(&block).await.unwrap();
        }

        // Check state
        {
            let chain = blockchain.read().await;
            let current_height = chain.height();
            let current_hash = chain.best_block_hash();

            // Height changed
            assert_ne!(current_height, expected_height);
            // Hash also changed
            assert_ne!(current_hash, expected_hash);
        }
    }

    #[tokio::test]
    async fn test_apply_block_requires_write_lock() {
        // Test that apply_block requires mutable access (write lock)
        let (blockchain, _temp) = create_test_blockchain().await;
        let blockchain = Arc::new(RwLock::new(blockchain));

        let block = {
            let chain = blockchain.read().await;
            chain.create_next_block([1u8; 32], vec![]).await.unwrap()
        };

        // This should compile and work
        {
            let mut chain = blockchain.write().await; // WRITE lock required
            let result = chain.apply_block(&block).await;
            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_height_and_hash_consistency() {
        // Test that height() and best_block_hash() remain consistent
        let (blockchain, _temp) = create_test_blockchain().await;
        let blockchain = Arc::new(RwLock::new(blockchain));

        // Read initial values
        let (height1, hash1) = {
            let chain = blockchain.read().await;
            (chain.height(), chain.best_block_hash())
        };

        // Read again - should be same
        let (height2, hash2) = {
            let chain = blockchain.read().await;
            (chain.height(), chain.best_block_hash())
        };

        assert_eq!(height1, height2);
        assert_eq!(hash1, hash2);

        // Apply a block
        {
            let mut chain = blockchain.write().await;
            let block = chain.create_next_block([1u8; 32], vec![]).await.unwrap();
            chain.apply_block(&block).await.unwrap();
        }

        // Read new values - should be different
        let (height3, hash3) = {
            let chain = blockchain.read().await;
            (chain.height(), chain.best_block_hash())
        };

        assert_ne!(height1, height3);
        assert_ne!(hash1, hash3);
        assert_eq!(height3, height1 + 1);
    }
}
