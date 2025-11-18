export type SignatureType = 'Classical' | 'MlDsa' | 'Falcon' | 'Hybrid';

export type ScriptType = 'p2pkh' | 'contract_deploy' | 'proof_anchor';

export type ProofType = 'KYC_LEVEL_1' | 'KYC_LEVEL_2' | 'KYC_LEVEL_3' | 'ACCREDITED';

export interface UtxoInput {
  prev_output_hash: string;
  output_index: number;
  signature_type: SignatureType;
  signature_size_bytes: number;
  public_key: string;
}

export interface UtxoOutput {
  amount: number;
  recipient_hash: string;
  script_type: ScriptType;
  is_spent?: boolean;
}

export interface Transaction {
  tx_hash: string;
  version: number;
  timestamp: number;
  inputs: UtxoInput[];
  outputs: UtxoOutput[];
  data?: string;
  proof_data?: {
    identity_id: string;
    proof_type: ProofType;
    proof_hash: string;
  };
  contract_data?: {
    function: string;
    args: Record<string, any>;
  };
}

export interface AssetTransfer {
  type: 'asset_transfer';
  from_address: string;
  to_address: string;
  asset_id: string;
  quantity: number;
  price?: number;
  metadata?: Record<string, any>;
}

export interface ContractCall {
  function_name: string;
  args: Record<string, any>;
}

export interface Block {
  height: number;
  hash: string;
  prev_hash: string;
  timestamp: number;
  tx_count: number;
  miner: string;
  merkle_root: string;
  difficulty_target: string;
  nonce: string;
  size: number;
  version: number;
  transactions?: Transaction[];
}

export interface ChainInfo {
  height: number;
  best_block_hash: string;
  difficulty: string;
  total_supply?: number;
  network_hashrate?: string;
  avg_block_time?: string;
  last_anchor_time?: string;
}

export interface Identity {
  identity_id: string;
  created_at: number;
  proof_anchors: ProofAnchor[];
  total_anchors: number;
  verification_level: ProofType;
}

export interface ProofAnchor {
  tx_hash: string;
  block_height: number;
  timestamp: number;
  proof_type: ProofType;
  proof_hash: string;
  status: 'verified' | 'pending' | 'failed';
}

export interface SustainabilityMetrics {
  id: string;
  network_hashrate_th: number;
  tx_count_24h: number;
  power_consumption_kwh: number;
  energy_per_tx_wh: number;
  carbon_footprint_kg: number;
  carbon_per_tx_g: number;
  storage_efficiency: number;
  network_efficiency: number;
  compute_efficiency: number;
  overall_score: number;
  grade: 'A+' | 'A' | 'B' | 'C' | 'D';
  timestamp: number;
}
