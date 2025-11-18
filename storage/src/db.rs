// RocksDB database wrapper
use rocksdb::{ColumnFamilyDescriptor, Options, WriteBatch, DB};
use std::path::Path;
use std::sync::Arc;
use tracing::{debug, info};

use crate::{StorageError, CF_BLOCKS, CF_META, CF_STATE, CF_TRANSACTIONS};
use boundless_core::{Block, BlockchainState, Transaction};

/// Database configuration
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    /// Path to database directory
    pub path: String,

    /// Cache size in MB
    pub cache_size_mb: usize,

    /// Enable compression
    pub enable_compression: bool,

    /// Max open files
    pub max_open_files: i32,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            path: "./data/db".to_string(),
            cache_size_mb: 128,
            enable_compression: true,
            max_open_files: 1000,
        }
    }
}

/// Database wrapper for blockchain storage
pub struct Database {
    db: Arc<DB>,
    config: DatabaseConfig,
}

impl Database {
    /// Open or create database
    pub fn open(config: DatabaseConfig) -> Result<Self, StorageError> {
        info!("📂 Opening database at {}", config.path);

        let path = Path::new(&config.path);

        // Create options
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);
        opts.set_max_open_files(config.max_open_files);

        // Set cache
        if config.cache_size_mb > 0 {
            let cache_size = config.cache_size_mb * 1024 * 1024;
            opts.set_write_buffer_size(cache_size / 4);
        }

        // Enable compression
        if config.enable_compression {
            opts.set_compression_type(rocksdb::DBCompressionType::Lz4);
        }

        // Define column families
        let cfs = vec![
            ColumnFamilyDescriptor::new(CF_BLOCKS, Options::default()),
            ColumnFamilyDescriptor::new(CF_TRANSACTIONS, Options::default()),
            ColumnFamilyDescriptor::new(CF_STATE, Options::default()),
            ColumnFamilyDescriptor::new(CF_META, Options::default()),
        ];

        // Open database
        let db = DB::open_cf_descriptors(&opts, path, cfs)?;

        info!("✅ Database opened successfully");

        Ok(Self {
            db: Arc::new(db),
            config,
        })
    }

    /// Store a block
    pub fn store_block(&self, block: &Block) -> Result<(), StorageError> {
        let cf = self
            .db
            .cf_handle(CF_BLOCKS)
            .ok_or_else(|| StorageError::DatabaseError("CF_BLOCKS not found".to_string()))?;

        // Key: block height (8 bytes, big-endian)
        let key = block.header.height.to_be_bytes();

        // Value: serialized block
        let value = bincode::serialize(block)?;

        self.db.put_cf(&cf, key, value)?;

        // Also index by hash
        let hash_key = block.header.hash();
        self.db.put_cf(&cf, hash_key, key)?;

        debug!("💾 Stored block #{}", block.header.height);

        Ok(())
    }

    /// Get block by height
    pub fn get_block_by_height(&self, height: u64) -> Result<Option<Block>, StorageError> {
        let cf = self
            .db
            .cf_handle(CF_BLOCKS)
            .ok_or_else(|| StorageError::DatabaseError("CF_BLOCKS not found".to_string()))?;

        let key = height.to_be_bytes();

        match self.db.get_cf(&cf, key)? {
            Some(bytes) => {
                let block = bincode::deserialize(&bytes)?;
                Ok(Some(block))
            }
            None => Ok(None),
        }
    }

    /// Get block by hash
    pub fn get_block_by_hash(&self, hash: &[u8; 32]) -> Result<Option<Block>, StorageError> {
        let cf = self
            .db
            .cf_handle(CF_BLOCKS)
            .ok_or_else(|| StorageError::DatabaseError("CF_BLOCKS not found".to_string()))?;

        // Hash maps to height
        match self.db.get_cf(&cf, hash)? {
            Some(height_bytes) => {
                if height_bytes.len() == 8 {
                    let height_array: [u8; 8] = height_bytes
                        .try_into()
                        .map_err(|_| StorageError::DatabaseError("Invalid height bytes".to_string()))?;
                    let height = u64::from_be_bytes(height_array);
                    self.get_block_by_height(height)
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }

    /// Store a transaction
    pub fn store_transaction(
        &self,
        tx: &Transaction,
        block_height: u64,
    ) -> Result<(), StorageError> {
        let cf = self
            .db
            .cf_handle(CF_TRANSACTIONS)
            .ok_or_else(|| StorageError::DatabaseError("CF_TRANSACTIONS not found".to_string()))?;

        let key = tx.hash();
        let value = bincode::serialize(&(tx, block_height))?;

        self.db.put_cf(&cf, key, value)?;

        Ok(())
    }

    /// Get transaction by hash
    pub fn get_transaction(
        &self,
        hash: &[u8; 32],
    ) -> Result<Option<(Transaction, u64)>, StorageError> {
        let cf = self
            .db
            .cf_handle(CF_TRANSACTIONS)
            .ok_or_else(|| StorageError::DatabaseError("CF_TRANSACTIONS not found".to_string()))?;

        match self.db.get_cf(&cf, hash)? {
            Some(bytes) => {
                let data = bincode::deserialize(&bytes)?;
                Ok(Some(data))
            }
            None => Ok(None),
        }
    }

    /// Store blockchain state
    pub fn store_state(&self, state: &BlockchainState) -> Result<(), StorageError> {
        let cf = self
            .db
            .cf_handle(CF_STATE)
            .ok_or_else(|| StorageError::DatabaseError("CF_STATE not found".to_string()))?;

        let key = b"current_state";
        let value = bincode::serialize(state)?;

        self.db.put_cf(&cf, key, value)?;

        debug!("💾 Stored blockchain state at height {}", state.height());

        Ok(())
    }

    /// Load blockchain state
    pub fn load_state(&self) -> Result<Option<BlockchainState>, StorageError> {
        let cf = self
            .db
            .cf_handle(CF_STATE)
            .ok_or_else(|| StorageError::DatabaseError("CF_STATE not found".to_string()))?;

        let key = b"current_state";

        match self.db.get_cf(&cf, key)? {
            Some(bytes) => {
                let state = bincode::deserialize(&bytes)?;
                Ok(Some(state))
            }
            None => Ok(None),
        }
    }

    /// Store metadata
    pub fn store_meta(&self, key: &str, value: &[u8]) -> Result<(), StorageError> {
        let cf = self
            .db
            .cf_handle(CF_META)
            .ok_or_else(|| StorageError::DatabaseError("CF_META not found".to_string()))?;

        self.db.put_cf(&cf, key.as_bytes(), value)?;

        Ok(())
    }

    /// Get metadata
    pub fn get_meta(&self, key: &str) -> Result<Option<Vec<u8>>, StorageError> {
        let cf = self
            .db
            .cf_handle(CF_META)
            .ok_or_else(|| StorageError::DatabaseError("CF_META not found".to_string()))?;

        Ok(self.db.get_cf(&cf, key.as_bytes())?)
    }

    /// Batch write
    pub fn batch_write<F>(&self, f: F) -> Result<(), StorageError>
    where
        F: FnOnce(&mut WriteBatch) -> Result<(), StorageError>,
    {
        let mut batch = WriteBatch::default();
        f(&mut batch)?;
        self.db.write(batch)?;
        Ok(())
    }

    /// Get database statistics
    pub fn stats(&self) -> String {
        self.db
            .property_value("rocksdb.stats")
            .unwrap_or_else(|_| Some("N/A".to_string()))
            .unwrap_or_else(|| "N/A".to_string())
    }

    /// Compact database
    pub fn compact(&self) {
        info!("🗜️  Compacting database...");
        self.db.compact_range::<&[u8], &[u8]>(None, None);
        info!("✅ Compaction complete");
    }
}

impl Drop for Database {
    fn drop(&mut self) {
        info!("💾 Closing database");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use boundless_core::{BlockHeader, TxOutput};
    use tempfile::tempdir;

    #[test]
    fn test_database_open() {
        let temp_dir = tempdir().unwrap();
        let config = DatabaseConfig {
            path: temp_dir.path().to_str().unwrap().to_string(),
            ..Default::default()
        };

        let db = Database::open(config);
        assert!(db.is_ok());
    }

    #[test]
    fn test_store_and_retrieve_block() {
        let temp_dir = tempdir().unwrap();
        let config = DatabaseConfig {
            path: temp_dir.path().to_str().unwrap().to_string(),
            ..Default::default()
        };

        let db = Database::open(config).unwrap();

        // Create test block
        let header = BlockHeader::new(1, [0u8; 32], [0u8; 32], [0u8; 32], 123456, 0x1f0fffff, 0, 1); // Added height
        let tx = Transaction::new(
            1,
            vec![],
            vec![TxOutput {
                amount: 1000,
                recipient_pubkey_hash: [1u8; 32],
                script: None,
            }],
            123456,
            None,
        );
        let block = Block::new(header, vec![tx]);

        // Store block
        assert!(db.store_block(&block).is_ok());

        // Retrieve by height
        let retrieved = db.get_block_by_height(1).unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().header.height, 1);

        // Retrieve by hash
        let hash = block.header.hash();
        let retrieved_by_hash = db.get_block_by_hash(&hash).unwrap();
        assert!(retrieved_by_hash.is_some());
    }
}
