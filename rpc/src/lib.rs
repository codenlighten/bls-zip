// Boundless BLS JSON-RPC API
//
// This module provides a JSON-RPC 2.0 API for interacting with the blockchain.
// Also includes HTTP REST bridge for Enterprise integration.

pub mod error;
pub mod http_bridge;
pub mod server;
pub mod types;

pub use error::RpcError;
pub use http_bridge::start_http_bridge;
pub use server::{RpcServer, RpcServerHandle};
pub use types::*;
