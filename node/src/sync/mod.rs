// Initial Block Download (IBD) Orchestrator
// Implements efficient blockchain synchronization with:
// - Headers-first sync
// - Parallel block downloads
// - Chainwork-based peer selection

pub mod ibd;
pub mod headers;
pub mod blocks;

pub use ibd::IbdOrchestrator;
pub use headers::HeadersSync;
pub use blocks::BlockDownloader;
