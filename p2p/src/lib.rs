// Boundless BLS P2P Networking
//
// This module provides peer-to-peer networking using libp2p for block and
// transaction propagation across the network.

pub mod network;
pub mod peer;
pub mod protocol;
pub mod reputation;
pub mod service;

pub use network::{NetworkConfig, NetworkEvent, NetworkNode};
pub use peer::PeerInfo;
pub use protocol::{BoundlessProtocol, Message, MessageType};
pub use reputation::{PeerReputation, ReputationConfig, ReputationManager, Violation};
pub use service::{NetworkCommand, NetworkHandle};
