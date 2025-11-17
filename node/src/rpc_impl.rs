// RPC trait implementation for Blockchain
use crate::blockchain::Blockchain;
use boundless_core::{Block, Transaction};
use boundless_rpc::server::BlockchainRpc;

impl BlockchainRpc for Blockchain {
    fn height(&self) -> u64 {
        self.state().height()
    }

    fn best_block_hash(&self) -> [u8; 32] {
        self.state().best_block_hash()
    }

    fn total_supply(&self) -> u64 {
        self.state().total_supply()
    }

    fn get_balance(&self, address: &[u8; 32]) -> u64 {
        self.state().get_balance(address)
    }

    fn get_nonce(&self, address: &[u8; 32]) -> u64 {
        self.state().get_nonce(address)
    }

    fn get_block_by_height(&self, height: u64) -> Option<Block> {
        self.get_block(height)
    }

    fn get_block_by_hash(&self, hash: &[u8; 32]) -> Option<Block> {
        self.get_block_by_hash(hash)
    }

    fn submit_transaction(&self, tx: Transaction) -> Result<[u8; 32], String> {
        // Basic validation
        tx.validate()
            .map_err(|e| format!("Validation failed: {}", e))?;

        // Return transaction hash
        Ok(tx.hash())
    }

    fn current_difficulty(&self) -> u32 {
        self.current_difficulty()
    }

    fn get_transaction(&self, tx_hash: &[u8; 32]) -> Option<boundless_core::TransactionRecord> {
        self.state().tx_index().get_transaction(tx_hash).cloned()
    }

    fn get_address_transactions(
        &self,
        address: &[u8; 32],
        limit: usize,
        offset: usize,
    ) -> Vec<boundless_core::TransactionRecord> {
        self.state()
            .tx_index()
            .get_address_transactions(address, limit, offset)
            .into_iter()
            .cloned()
            .collect()
    }

    fn get_address_tx_count(&self, address: &[u8; 32]) -> usize {
        self.state().tx_index().get_address_tx_count(address)
    }

    fn get_proof_by_id(&self, proof_id: &[u8; 32]) -> Option<boundless_core::ProofAnchor> {
        self.state().proof_storage().get_proof(proof_id).cloned()
    }

    fn verify_proof_by_hash(
        &self,
        proof_hash: &[u8; 32],
    ) -> Option<boundless_core::ProofAnchor> {
        self.state().proof_storage().verify_proof(proof_hash).cloned()
    }

    fn get_utxos(&self, address: &[u8; 32]) -> Vec<boundless_rpc::types::UtxoData> {
        // Get UTXOs from the blockchain state
        let utxos = self.state().get_utxos(address);

        // Convert to RPC format
        utxos
            .into_iter()
            .map(|(outpoint, output)| {
                // Get block height from transaction index
                let block_height = self
                    .state()
                    .tx_index()
                    .get_transaction(&outpoint.tx_hash)
                    .map(|record| record.block_height)
                    .unwrap_or(0); // Default to 0 if not found in index

                boundless_rpc::types::UtxoData {
                    tx_hash: hex::encode(outpoint.tx_hash),
                    output_index: outpoint.index,
                    amount: output.amount,
                    block_height,
                    script: output.script.map(hex::encode),
                }
            })
            .collect()
    }
}
