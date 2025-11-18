import { blockchainClient } from './client';
import type { ChainInfo, Block, Transaction, ProofAnchor } from '@/lib/types';

/**
 * Blockchain Indexer
 * Provides a high-level API for accessing blockchain data
 * Replaces the mock-indexer.ts with real blockchain queries
 */
export class BlockchainIndexer {
  private client = blockchainClient;

  /**
   * Get overall chain information and statistics
   */
  async getChainInfo(): Promise<ChainInfo> {
    try {
      return await this.client.getChainInfo();
    } catch (error) {
      console.error('Failed to fetch chain info:', error);
      throw new Error('Unable to fetch blockchain information');
    }
  }

  /**
   * Get the latest N blocks
   */
  async getLatestBlocks(count: number): Promise<Block[]> {
    try {
      return await this.client.getLatestBlocks(count);
    } catch (error) {
      console.error(`Failed to fetch latest ${count} blocks:`, error);
      return []; // Return empty array on error to prevent UI crash
    }
  }

  /**
   * Get block by height
   */
  async getBlockByHeight(height: number): Promise<Block | null> {
    try {
      return await this.client.getBlockByHeight(height);
    } catch (error) {
      console.error(`Block not found at height ${height}:`, error);
      return null;
    }
  }

  /**
   * Get block by hash
   */
  async getBlockByHash(hash: string): Promise<Block | null> {
    try {
      return await this.client.getBlockByHash(hash);
    } catch (error) {
      console.error(`Block not found with hash ${hash}:`, error);
      return null;
    }
  }

  /**
   * Get transaction by hash
   */
  async getTransaction(hash: string): Promise<Transaction | null> {
    try {
      return await this.client.getTransaction(hash);
    } catch (error) {
      console.error(`Transaction not found ${hash}:`, error);
      return null;
    }
  }

  /**
   * Get balance for an address
   */
  async getBalance(address: string): Promise<number> {
    try {
      const result = await this.client.getBalance(address);
      return result.balance;
    } catch (error) {
      console.error(`Failed to get balance for ${address}:`, error);
      return 0;
    }
  }

  /**
   * Get UTXOs for an address
   */
  async getUtxos(address: string): Promise<any[]> {
    try {
      return await this.client.getUtxos(address);
    } catch (error) {
      console.error(`Failed to get UTXOs for ${address}:`, error);
      return [];
    }
  }

  /**
   * Get proof by ID (for E2 identity verification)
   */
  async getProof(proofId: string): Promise<ProofAnchor | null> {
    try {
      return await this.client.getProof(proofId);
    } catch (error) {
      console.error(`Proof not found ${proofId}:`, error);
      return null;
    }
  }

  /**
   * Verify proof by hash
   */
  async verifyProof(proofHash: string): Promise<ProofAnchor | null> {
    try {
      return await this.client.verifyProof(proofHash);
    } catch (error) {
      console.error(`Failed to verify proof ${proofHash}:`, error);
      return null;
    }
  }

  /**
   * Get all proofs for an identity
   */
  async getProofsByIdentity(identityId: string): Promise<ProofAnchor[]> {
    try {
      return await this.client.getProofsByIdentity(identityId);
    } catch (error) {
      console.error(`Failed to get proofs for identity ${identityId}:`, error);
      return [];
    }
  }

  /**
   * Submit a transaction to the blockchain
   */
  async submitTransaction(tx: Transaction): Promise<{ tx_hash: string; accepted: boolean }> {
    try {
      return await this.client.submitTransaction(tx);
    } catch (error) {
      console.error('Failed to submit transaction:', error);
      throw new Error('Transaction submission failed');
    }
  }

  /**
   * Check if connected to blockchain node
   */
  async isConnected(): Promise<boolean> {
    return await this.client.isConnected();
  }
}

// Singleton instance
export const blockchainIndexer = new BlockchainIndexer();

// Export for backward compatibility with existing code
export default blockchainIndexer;
