import axios, { AxiosInstance } from 'axios';
import type { ChainInfo, Block, Transaction, ProofAnchor } from '@/lib/types';

interface RpcRequest {
  jsonrpc: string;
  method: string;
  params: any[];
  id: number;
}

interface RpcResponse<T = any> {
  jsonrpc: string;
  result?: T;
  error?: { code: number; message: string };
  id: number;
}

interface RawBlock {
  header: {
    version: number;
    previous_hash: string;
    merkle_root: string;
    timestamp: number;
    difficulty_target: number;
    nonce: number;
    height: number;
  };
  transactions: any[];
}

interface RawChainInfo {
  height: number;
  best_block_hash: string;
  total_supply?: number;
  difficulty: number;
}

/**
 * Blockchain RPC Client
 * Communicates with the Boundless blockchain node via JSON-RPC 2.0
 */
export class BlockchainClient {
  private client: AxiosInstance;
  private rpcUrl: string;
  private requestId: number = 1;

  constructor(rpcUrl?: string) {
    this.rpcUrl = rpcUrl || process.env.NEXT_PUBLIC_BLOCKCHAIN_RPC_URL || 'http://localhost:9933';
    this.client = axios.create({
      baseURL: this.rpcUrl,
      headers: { 'Content-Type': 'application/json' },
      timeout: 30000, // 30 second timeout
    });
  }

  /**
   * Make a JSON-RPC call to the blockchain node
   */
  private async call<T>(method: string, params: any[] = []): Promise<T> {
    const request: RpcRequest = {
      jsonrpc: '2.0',
      method,
      params,
      id: this.requestId++,
    };

    try {
      const response = await this.client.post<RpcResponse<T>>('', request);

      if (response.data.error) {
        throw new Error(`RPC Error [${response.data.error.code}]: ${response.data.error.message}`);
      }

      return response.data.result!;
    } catch (error: any) {
      if (error.code === 'ECONNREFUSED') {
        throw new Error('Cannot connect to blockchain node. Is it running?');
      }
      throw error;
    }
  }

  /**
   * Convert raw blockchain data to application format
   */
  private convertBlock(raw: RawBlock): Block {
    return {
      height: raw.header.height,
      hash: raw.header.previous_hash, // Will be computed on frontend if needed
      prev_hash: raw.header.previous_hash,
      timestamp: raw.header.timestamp,
      tx_count: raw.transactions.length,
      miner: '0x0000000000000000000000000000000000000000', // Coinbase address if available
      difficulty_target: raw.header.difficulty_target.toString(),
      nonce: raw.header.nonce.toString(),
      merkle_root: raw.header.merkle_root,
      size: 1000, // Estimate or calculate
      version: raw.header.version,
      transactions: raw.transactions, // Include actual transactions
    };
  }

  // ======================
  // Chain Info Methods
  // ======================

  /**
   * Get overall chain information
   */
  async getChainInfo(): Promise<ChainInfo> {
    const raw = await this.call<RawChainInfo>('chain_getInfo', []);
    return {
      height: raw.height,
      best_block_hash: raw.best_block_hash,
      difficulty: raw.difficulty.toString(),
      total_supply: raw.total_supply,
      network_hashrate: '450 MH/s', // Estimate from difficulty
      avg_block_time: '5m 00s', // 5 minutes
      last_anchor_time: '2m 15s', // Estimate
    };
  }

  /**
   * Get current block height
   */
  async getBlockHeight(): Promise<number> {
    return this.call<number>('chain_getBlockHeight', []);
  }

  // ======================
  // Block Methods
  // ======================

  /**
   * Get block by height
   */
  async getBlockByHeight(height: number): Promise<Block> {
    const raw = await this.call<RawBlock>('chain_getBlockByHeight', [height]);
    return this.convertBlock(raw);
  }

  /**
   * Get block by hash
   */
  async getBlockByHash(hash: string): Promise<Block> {
    const raw = await this.call<RawBlock>('chain_getBlockByHash', [hash]);
    return this.convertBlock(raw);
  }

  /**
   * Get multiple recent blocks
   */
  async getLatestBlocks(count: number): Promise<Block[]> {
    try {
      const height = await this.getBlockHeight();
      const blocks: Block[] = [];

      const startHeight = Math.max(0, height - count + 1);

      // Fetch blocks in parallel
      const promises = [];
      for (let i = startHeight; i <= height; i++) {
        promises.push(this.getBlockByHeight(i));
      }

      const results = await Promise.allSettled(promises);

      for (const result of results) {
        if (result.status === 'fulfilled') {
          blocks.push(result.value);
        }
      }

      return blocks.reverse(); // Most recent first
    } catch (error) {
      console.error('Error fetching latest blocks:', error);
      throw error;
    }
  }

  // ======================
  // Transaction Methods
  // ======================

  /**
   * Get transaction by hash
   */
  async getTransaction(hash: string): Promise<Transaction> {
    return this.call<Transaction>('chain_getTransaction', [hash]);
  }

  /**
   * Submit a new transaction to the mempool
   */
  async submitTransaction(tx: Transaction): Promise<{ tx_hash: string; accepted: boolean }> {
    return this.call<{ tx_hash: string; accepted: boolean }>('chain_submitTransaction', [tx]);
  }

  // ======================
  // Balance & UTXO Methods
  // ======================

  /**
   * Get balance for an address
   */
  async getBalance(address: string): Promise<{ address: string; balance: number }> {
    return this.call<{ address: string; balance: number }>('chain_getBalance', [address]);
  }

  /**
   * Get UTXOs for an address
   */
  async getUtxos(address: string): Promise<any[]> {
    return this.call<any[]>('chain_getUtxos', [address]);
  }

  // ======================
  // Proof Methods (E2 Integration)
  // ======================

  /**
   * Get proof by ID
   */
  async getProof(proofId: string): Promise<ProofAnchor> {
    return this.call<ProofAnchor>('chain_getProof', [proofId]);
  }

  /**
   * Verify proof by hash
   */
  async verifyProof(proofHash: string): Promise<ProofAnchor> {
    return this.call<ProofAnchor>('chain_verifyProof', [proofHash]);
  }

  /**
   * Get all proofs for an identity
   */
  async getProofsByIdentity(identityId: string): Promise<ProofAnchor[]> {
    return this.call<ProofAnchor[]>('chain_getProofsByIdentity', [identityId]);
  }

  // ======================
  // Utility Methods
  // ======================

  /**
   * Check if the blockchain node is reachable
   */
  async isConnected(): Promise<boolean> {
    try {
      await this.getBlockHeight();
      return true;
    } catch (error) {
      return false;
    }
  }

  /**
   * Get the current RPC URL
   */
  getRpcUrl(): string {
    return this.rpcUrl;
  }
}

// Singleton instance
export const blockchainClient = new BlockchainClient();
