// Persistent storage layer using RocksDB
//
// This module provides persistent storage for blocks, transactions, and state.

pub mod db;
pub mod error;

pub use db::{Database, DatabaseConfig};
pub use error::StorageError;

// Column family names
pub const CF_BLOCKS: &str = "blocks";
pub const CF_TRANSACTIONS: &str = "transactions";
pub const CF_STATE: &str = "state";
pub const CF_META: &str = "meta";
