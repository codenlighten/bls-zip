// Mining implementation for finding valid blocks
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::error::ConsensusError;
use crate::pow::ProofOfWork;
use boundless_core::{Block, BlockHeader};

/// Result of a mining operation
#[derive(Debug, Clone)]
pub struct MiningResult {
    /// The successfully mined block
    pub block: Block,

    /// Number of hashes computed
    pub hashes_computed: u64,

    /// Time taken to mine
    pub duration: Duration,

    /// Hash rate achieved (hashes per second)
    pub hash_rate: f64,
}

/// Miner for finding valid Proof-of-Work blocks
pub struct Miner {
    /// Number of threads to use for mining
    threads: usize,

    /// Stop signal
    should_stop: Arc<AtomicBool>,

    /// Hashes computed counter
    hashes_computed: Arc<AtomicU64>,
}

impl Miner {
    /// Create a new miner
    pub fn new(threads: usize) -> Self {
        Self {
            threads: threads.max(1),
            should_stop: Arc::new(AtomicBool::new(false)),
            hashes_computed: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Mine a block (find valid nonce)
    ///
    /// # Arguments
    /// * `mut block` - The block to mine (header will be modified with valid nonce)
    ///
    /// # Returns
    /// MiningResult with the mined block and statistics
    pub fn mine(&self, block: Block) -> Result<MiningResult, ConsensusError> {
        let start_time = Instant::now();
        self.should_stop.store(false, Ordering::Relaxed);
        self.hashes_computed.store(0, Ordering::Relaxed);

        println!(
            "Mining block with difficulty target 0x{:08x}...",
            block.header.difficulty_target
        );

        let target = BlockHeader::compact_to_target(block.header.difficulty_target);

        // HIGH PRIORITY FIX: Shared atomic timestamp for coordinated nonce exhaustion
        // Prevents duplicate work when threads exhaust nonce space
        let shared_timestamp = Arc::new(AtomicU64::new(block.header.timestamp));

        // Multi-threaded mining with work distribution
        let thread_count = self.threads;
        let stop_signal = self.should_stop.clone();
        let hashes_counter = self.hashes_computed.clone();

        // Channel for communicating found blocks
        let (tx, rx) = std::sync::mpsc::channel();

        // Spawn worker threads
        let handles: Vec<_> = (0..thread_count)
            .map(|thread_id| {
                let mut block_clone = block.clone();
                let target = target;
                let tx = tx.clone();
                let stop = stop_signal.clone();
                let counter = hashes_counter.clone();
                let start = start_time;
                let timestamp_ref = shared_timestamp.clone();

                std::thread::spawn(move || {
                    // Each thread uses a different starting nonce and increments by thread_count
                    // Thread 0: 0, thread_count, 2*thread_count, ...
                    // Thread 1: 1, thread_count+1, 2*thread_count+1, ...
                    // This ensures no overlap and good load distribution
                    let mut nonce: u64 = thread_id as u64;

                    loop {
                        if stop.load(Ordering::Relaxed) {
                            return;
                        }

                        // HIGH PRIORITY FIX: Read shared timestamp for coordinated mining
                        block_clone.header.timestamp = timestamp_ref.load(Ordering::Relaxed);
                        block_clone.header.nonce = nonce;
                        let hash = block_clone.header.hash();
                        counter.fetch_add(1, Ordering::Relaxed);

                        let hash_value = primitive_types::U256::from_big_endian(&hash);

                        if hash_value < target {
                            // Found valid block!
                            stop.store(true, Ordering::Relaxed);
                            let _ = tx.send((block_clone, nonce, hash));
                            return;
                        }

                        // Increment nonce by thread count to avoid overlap
                        let prev_nonce = nonce;
                        nonce = nonce.wrapping_add(thread_count as u64);

                        // HIGH PRIORITY FIX: Detect wraparound and atomically increment shared timestamp
                        // Only one thread will successfully increment when wrapping occurs
                        if nonce < prev_nonce || nonce < thread_count as u64 {
                            let old_timestamp = timestamp_ref.fetch_add(1, Ordering::SeqCst);
                            if thread_id == 0 {
                                println!(
                                    "\n⏰ Nonce space exhausted, incrementing timestamp: {} → {}",
                                    old_timestamp,
                                    old_timestamp + 1
                                );
                            }
                        }

                        // Progress update every 100k hashes (only from thread 0)
                        if thread_id == 0
                            && nonce % (100_000 * thread_count as u64) < thread_count as u64
                        {
                            let elapsed = start.elapsed().as_secs_f64();
                            let total_hashes = counter.load(Ordering::Relaxed);
                            let current_rate = total_hashes as f64 / elapsed;
                            print!(
                                "\rHashes: {}, Rate: {:.2} H/s, Time: {:.1}s",
                                total_hashes, current_rate, elapsed
                            );
                        }
                    }
                })
            })
            .collect();

        // Drop the original sender so the channel closes when all threads finish
        drop(tx);

        // Wait for result from any thread
        match rx.recv() {
            Ok((found_block, nonce, hash)) => {
                // Stop all threads
                self.should_stop.store(true, Ordering::Relaxed);

                // Wait for all threads to finish
                for handle in handles {
                    let _ = handle.join();
                }

                let duration = start_time.elapsed();
                let hashes = self.hashes_computed.load(Ordering::Relaxed);
                let hash_rate = hashes as f64 / duration.as_secs_f64();

                println!(
                    "\nBlock mined! Nonce: {}, Hash: {}, Hashes: {}, Hash rate: {:.2} H/s",
                    nonce,
                    hex::encode(hash),
                    hashes,
                    hash_rate
                );

                Ok(MiningResult {
                    block: found_block,
                    hashes_computed: hashes,
                    duration,
                    hash_rate,
                })
            }
            Err(_) => {
                // Channel closed without finding a solution (should_stop was set)
                for handle in handles {
                    let _ = handle.join();
                }
                Err(ConsensusError::MiningStopped)
            }
        }
    }

    /// Stop the current mining operation
    pub fn stop(&self) {
        self.should_stop.store(true, Ordering::Relaxed);
    }

    /// Get the number of hashes computed in the current/last mining operation
    pub fn hashes_computed(&self) -> u64 {
        self.hashes_computed.load(Ordering::Relaxed)
    }

    /// Verify that a block meets the PoW requirements
    pub fn verify_block(&self, block: &Block) -> Result<(), ConsensusError> {
        ProofOfWork::validate_block(block)
    }
}

impl Default for Miner {
    fn default() -> Self {
        Self::new(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use boundless_core::{Transaction, TxOutput};

    #[test]
    fn test_miner_creation() {
        let miner = Miner::new(4);
        assert_eq!(miner.threads, 4);
    }

    #[test]
    #[ignore] // Slow test, run with --ignored
    fn test_mining_easy_difficulty() {
        let miner = Miner::new(1);

        // Create a block with very easy difficulty
        let header = BlockHeader::new(
            1, [0u8; 32], [0u8; 32], 1234567890, 0x1f0fffff, // Very easy
            0, 1, // height
        );

        let block = Block::new(
            header,
            vec![Transaction::new(
                1,
                vec![],
                vec![TxOutput {
                    amount: 5000000000, // 50 BLS coins
                    recipient_pubkey_hash: [0u8; 32],
                    script: None,
                }],
                1234567890,
                None,
            )],
        );

        let result = miner.mine(block);
        assert!(result.is_ok());

        let mining_result = result.unwrap();
        assert!(mining_result.hashes_computed > 0);
        assert!(mining_result.hash_rate > 0.0);
    }
}
