import type {
  Block,
  Transaction,
  ChainInfo,
  Identity,
  ProofAnchor,
  SignatureType,
  ScriptType,
  ProofType,
  SustainabilityMetrics,
} from './types';

function generateHash(prefix: string = ''): string {
  const chars = '0123456789abcdef';
  let hash = prefix;
  for (let i = 0; i < (64 - prefix.length); i++) {
    hash += chars[Math.floor(Math.random() * chars.length)];
  }
  return hash;
}

function generateUUID(): string {
  return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, (c) => {
    const r = Math.random() * 16 | 0;
    const v = c === 'x' ? r : (r & 0x3 | 0x8);
    return v.toString(16);
  });
}

function getRandomElement<T>(arr: T[]): T {
  return arr[Math.floor(Math.random() * arr.length)];
}

const SIGNATURE_TYPES: SignatureType[] = ['Classical', 'MlDsa', 'Falcon', 'Hybrid'];
const SCRIPT_TYPES: ScriptType[] = ['p2pkh', 'contract_deploy', 'proof_anchor'];
const PROOF_TYPES: ProofType[] = ['KYC_LEVEL_1', 'KYC_LEVEL_2', 'KYC_LEVEL_3', 'ACCREDITED'];

const MOCK_ADDRESSES = Array.from({ length: 100 }, () => generateHash(''));
const MOCK_IDENTITIES = Array.from({ length: 20 }, () => generateUUID());

class MockIndexer {
  private currentHeight: number = 12450;
  private blocks: Map<number, Block> = new Map();
  private blocksByHash: Map<string, Block> = new Map();
  private transactions: Map<string, Transaction> = new Map();
  private identities: Map<string, Identity> = new Map();

  constructor() {
    this.initializeMockData();
  }

  private initializeMockData() {
    const now = Date.now();

    for (let i = 0; i < 50; i++) {
      const height = this.currentHeight - i;
      const block = this.generateBlock(height, now - (i * 5 * 60 * 1000));
      this.blocks.set(height, block);
      this.blocksByHash.set(block.hash, block);

      const txCount = Math.floor(Math.random() * 10) + 1;
      const txs: Transaction[] = [];
      for (let j = 0; j < txCount; j++) {
        const tx = this.generateTransaction(now - (i * 5 * 60 * 1000) - (j * 1000));
        this.transactions.set(tx.tx_hash, tx);
        txs.push(tx);
      }
      block.transactions = txs;
    }

    MOCK_IDENTITIES.forEach(id => {
      this.identities.set(id, this.generateIdentity(id));
    });
  }

  private generateBlock(height: number, timestamp: number): Block {
    return {
      height,
      hash: generateHash(''),
      prev_hash: height > 0 ? generateHash('') : '0000000000000000000000000000000000000000000000000000000000000000',
      timestamp,
      tx_count: Math.floor(Math.random() * 10) + 1,
      miner: getRandomElement(MOCK_ADDRESSES),
      merkle_root: generateHash(''),
      state_root: generateHash(''),
      difficulty_target: '0x' + '0'.repeat(8) + Math.floor(Math.random() * 1000000).toString(16).padStart(8, '0'),
      nonce: '0x' + Math.floor(Math.random() * 0xFFFFFFFF).toString(16).padStart(8, '0'),
      size: Math.floor(Math.random() * 500000) + 100000,
      version: 1,
    };
  }

  private generateTransaction(timestamp: number): Transaction {
    const rand = Math.random();
    const hasProofData = rand > 0.8;
    const hasAssetTransfer = !hasProofData && rand > 0.6;
    const hasContractCall = !hasProofData && !hasAssetTransfer && rand > 0.7;

    const inputCount = hasAssetTransfer ? 1 : Math.floor(Math.random() * 3) + 1;
    const outputCount = hasAssetTransfer ? 1 : Math.floor(Math.random() * 3) + 1;

    const inputs = Array.from({ length: inputCount }, (_, i) => {
      const sigType = getRandomElement(SIGNATURE_TYPES);
      let sigSize = 256;
      if (sigType === 'MlDsa') sigSize = 2420;
      if (sigType === 'Falcon') sigSize = 1280;
      if (sigType === 'Hybrid') sigSize = 3700;

      return {
        prev_output_hash: generateHash(''),
        output_index: i,
        signature_type: sigType,
        signature_size_bytes: sigSize,
        public_key: generateHash(''),
      };
    });

    const scriptType: ScriptType = hasProofData ? 'proof_anchor' : hasContractCall ? 'contract_deploy' : 'p2pkh';

    const outputs = Array.from({ length: outputCount }, (_, i) => ({
      amount: hasAssetTransfer ? 0 : Math.floor(Math.random() * 1000) * 1000000000,
      recipient_hash: getRandomElement(MOCK_ADDRESSES),
      script_type: scriptType,
      is_spent: Math.random() > 0.5,
    }));

    const tx: Transaction = {
      tx_hash: generateHash(''),
      version: 1,
      timestamp,
      inputs,
      outputs,
    };

    if (hasProofData) {
      tx.proof_data = {
        identity_id: getRandomElement(MOCK_IDENTITIES),
        proof_type: getRandomElement(PROOF_TYPES),
        proof_hash: generateHash(''),
      };
    }

    if (hasAssetTransfer) {
      const assetData = {
        type: 'asset_transfer',
        from_address: getRandomElement(MOCK_ADDRESSES),
        to_address: getRandomElement(MOCK_ADDRESSES),
        asset_id: generateUUID(),
        quantity: Math.floor(Math.random() * 10000) + 100,
        price: Math.random() > 0.5 ? Math.floor(Math.random() * 100000) + 1000 : undefined,
        metadata: {
          asset_type: getRandomElement(['Equity Token', 'Carbon Credit', 'Real Estate Share', 'Commodity Token']),
        },
      };
      const jsonString = JSON.stringify(assetData);
      const encoder = new TextEncoder();
      const bytes = encoder.encode(jsonString);
      tx.data = '0x' + Array.from(bytes).map(b => b.toString(16).padStart(2, '0')).join('');
    }

    if (hasContractCall) {
      const functionName = getRandomElement(['transfer', 'mint', 'burn', 'stake', 'approve', 'delegate']);
      const args = {
        to: getRandomElement(MOCK_ADDRESSES).substring(0, 40),
        amount: Math.floor(Math.random() * 100000),
        timestamp: Date.now(),
      };

      const encoder = new TextEncoder();
      const nameBytes = encoder.encode(functionName);
      const nameLen = nameBytes.length;
      const argsString = JSON.stringify(args);
      const argsBytes = encoder.encode(argsString);

      const result = new Uint8Array(2 + nameBytes.length + argsBytes.length);
      result[0] = (nameLen >> 8) & 0xFF;
      result[1] = nameLen & 0xFF;
      result.set(nameBytes, 2);
      result.set(argsBytes, 2 + nameBytes.length);

      tx.data = '0x' + Array.from(result).map(b => b.toString(16).padStart(2, '0')).join('');

      tx.contract_data = {
        function: functionName,
        args,
      };
    }

    return tx;
  }

  private generateIdentity(identityId: string): Identity {
    const anchorCount = Math.floor(Math.random() * 5) + 1;
    const anchors: ProofAnchor[] = Array.from({ length: anchorCount }, (_, i) => ({
      tx_hash: generateHash(''),
      block_height: this.currentHeight - Math.floor(Math.random() * 1000),
      timestamp: Date.now() - (i * 30 * 24 * 60 * 60 * 1000),
      proof_type: getRandomElement(PROOF_TYPES),
      proof_hash: generateHash(''),
      status: 'verified' as const,
    }));

    return {
      identity_id: identityId,
      created_at: Date.now() - (anchorCount * 30 * 24 * 60 * 60 * 1000),
      proof_anchors: anchors.sort((a, b) => b.timestamp - a.timestamp),
      total_anchors: anchorCount,
      verification_level: getRandomElement(PROOF_TYPES),
    };
  }

  getChainInfo(): ChainInfo {
    return {
      height: this.currentHeight,
      best_block_hash: this.blocks.get(this.currentHeight)?.hash || generateHash(''),
      difficulty: '0x00000000ffffffff',
      network_hashrate: '450 MH/s',
      avg_block_time: '5m 00s',
      last_anchor_time: '2m 15s',
    };
  }

  getLatestBlocks(count: number = 10): Block[] {
    const blocks: Block[] = [];
    for (let i = 0; i < count; i++) {
      const block = this.blocks.get(this.currentHeight - i);
      if (block) blocks.push(block);
    }
    return blocks;
  }

  getRecentTransactions(count: number = 10): Transaction[] {
    return Array.from(this.transactions.values())
      .sort((a, b) => b.timestamp - a.timestamp)
      .slice(0, count);
  }

  getBlockByHeight(height: number): Block | undefined {
    return this.blocks.get(height);
  }

  getBlockByHash(hash: string): Block | undefined {
    return this.blocksByHash.get(hash);
  }

  getTransaction(hash: string): Transaction | undefined {
    return this.transactions.get(hash);
  }

  getIdentity(identityId: string): Identity | undefined {
    return this.identities.get(identityId);
  }

  search(query: string): { type: 'block' | 'transaction' | 'identity' | 'address'; result: any } | null {
    if (query.length === 64 && /^[0-9a-f]+$/i.test(query)) {
      const tx = this.getTransaction(query);
      if (tx) return { type: 'transaction', result: tx };

      const block = this.getBlockByHash(query);
      if (block) return { type: 'block', result: block };
    }

    if (/^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i.test(query)) {
      const identity = this.getIdentity(query);
      if (identity) return { type: 'identity', result: identity };
    }

    if (!isNaN(Number(query))) {
      const block = this.getBlockByHeight(Number(query));
      if (block) return { type: 'block', result: block };
    }

    return null;
  }

  getNetworkHashrate(): number {
    return 450000 + Math.random() * 50000;
  }

  getTxCount24h(): number {
    return 150000 + Math.floor(Math.random() * 50000);
  }

  getSustainabilityMetrics(): SustainabilityMetrics {
    const networkHashrate = this.getNetworkHashrate();
    const txCount24h = this.getTxCount24h();

    return {
      id: generateUUID(),
      network_hashrate_th: networkHashrate,
      tx_count_24h: txCount24h,
      power_consumption_kwh: 1250.5,
      energy_per_tx_wh: 0.05,
      carbon_footprint_kg: 450.2,
      carbon_per_tx_g: 0.02,
      storage_efficiency: 92.5,
      network_efficiency: 88.3,
      compute_efficiency: 94.7,
      overall_score: 91.2,
      grade: 'A+',
      timestamp: Date.now(),
    };
  }

  getHistoricalSustainabilityMetrics(days: number = 30): SustainabilityMetrics[] {
    const metrics: SustainabilityMetrics[] = [];
    const now = Date.now();

    for (let i = 0; i < days; i++) {
      const dayOffset = days - i - 1;
      const baseHashrate = 450000 + (dayOffset * 500);
      const networkHashrate = baseHashrate + Math.random() * 30000;
      const baseTxCount = 150000 + (dayOffset * 800);
      const txCount24h = baseTxCount + Math.floor(Math.random() * 30000);

      metrics.push({
        id: generateUUID(),
        network_hashrate_th: networkHashrate,
        tx_count_24h: txCount24h,
        power_consumption_kwh: 1200 + Math.random() * 100,
        energy_per_tx_wh: Math.max(0.04, 0.08 - (dayOffset * 0.001) + (Math.random() * 0.01 - 0.005)),
        carbon_footprint_kg: 430 + Math.random() * 40,
        carbon_per_tx_g: Math.max(0.015, 0.035 - (dayOffset * 0.0005) + (Math.random() * 0.005 - 0.0025)),
        storage_efficiency: 90 + Math.random() * 5,
        network_efficiency: 85 + Math.random() * 8,
        compute_efficiency: 92 + Math.random() * 5,
        overall_score: 88 + Math.random() * 6,
        grade: 'A+',
        timestamp: now - (dayOffset * 24 * 60 * 60 * 1000),
      });
    }

    return metrics;
  }
}

export const mockIndexer = new MockIndexer();
