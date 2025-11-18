import type { Block, Transaction, ChainInfo, Identity } from './types';

const API_BASE_URL = process.env.NEXT_PUBLIC_API_BASE_URL || 'http://localhost:3001';

class ApiClient {
  private baseUrl: string;

  constructor(baseUrl: string = API_BASE_URL) {
    this.baseUrl = baseUrl;
  }

  private async fetch<T>(endpoint: string): Promise<T> {
    const response = await fetch(`${this.baseUrl}${endpoint}`);
    if (!response.ok) {
      throw new Error(`API Error: ${response.statusText}`);
    }
    return response.json();
  }

  async getChainInfo(): Promise<ChainInfo> {
    return this.fetch<ChainInfo>('/api/v1/chain/info');
  }

  async getBlockByHeight(height: number): Promise<Block> {
    return this.fetch<Block>(`/api/v1/block/height/${height}`);
  }

  async getBlockByHash(hash: string): Promise<Block> {
    return this.fetch<Block>(`/api/v1/block/hash/${hash}`);
  }

  async getTransaction(hash: string): Promise<Transaction> {
    return this.fetch<Transaction>(`/api/v1/transaction/${hash}`);
  }

  async getProof(id: string): Promise<Identity> {
    return this.fetch<Identity>(`/api/v1/proof/${id}`);
  }

  async search(query: string): Promise<{ type: 'block' | 'transaction' | 'identity'; data: any }> {
    return this.fetch(`/api/v1/search?q=${encodeURIComponent(query)}`);
  }
}

export const apiClient = new ApiClient();
