// Type definitions for Enterprise Multipass (matching backend models exactly)

// ============================================================================
// IDENTITY & ATTESTATION
// ============================================================================

export interface IdentityProfile {
  identity_id: string
  root_pki_key_id: string
  legal_name: string
  email: string
  org_name?: string | null
  kyc_status: KycStatus
  aml_risk_score: number
  created_at: string
  updated_at: string
}

export type KycStatus = 'pending' | 'verified' | 'rejected' | 'revoked'

export interface IdentityAttestation {
  attestation_id: string
  identity_id: string
  attestation_type: AttestationType
  evidence_refs: string[]
  issuer: string
  status: AttestationStatus
  valid_from: string
  valid_to?: string | null
  chain_anchor_tx?: string | null
}

export type AttestationType = 'kyc' | 'address_proof' | 'income_proof' | 'asset_ownership' | 'social_graph' | 'professional_credential' | { custom: string }
export type AttestationStatus = 'valid' | 'expired' | 'revoked'

// ============================================================================
// WALLET
// ============================================================================

export interface WalletAccount {
  wallet_id: string
  identity_id: string
  boundless_addresses: BoundlessAddress[]
  labels: string[]
  created_at: string
}

export interface BoundlessAddress {
  address: string
  metadata?: string | null
  label?: string | null
}

export interface WalletBalance {
  wallet_id: string
  asset_type: AssetType
  amount: number
  locked_amount: number
  updated_at: string
}

export interface WalletTransaction {
  tx_id: string
  wallet_id: string
  chain_tx_hash: string
  asset_type: AssetType
  amount: number
  direction: TxDirection
  operation_type?: TransactionOperationType
  counterparty?: string | null
  application_context?: string | null
  metadata?: TransactionMetadata
  timestamp: string
  status: TxStatus
}

export type TransactionOperationType =
  | 'payment'
  | 'contract_deploy'
  | 'contract_call'
  | 'contract_sign'
  | 'storage_write'
  | 'document_anchor'
  | 'attestation_create'
  | 'attestation_revoke'
  | 'permission_grant'
  | 'permission_revoke'
  | 'app_connect'
  | 'asset_mint'
  | 'asset_burn'
  | 'market_order'
  | 'identity_update'
  | { custom: string }

export interface TransactionMetadata {
  contract_id?: string
  document_id?: string
  attestation_id?: string
  app_id?: string
  order_id?: string
  description?: string
  [key: string]: any
}

export type TxDirection = 'in' | 'out'
export type TxStatus = 'pending' | 'confirmed' | 'failed'

// ============================================================================
// AUTH
// ============================================================================

export interface MultipassCredential {
  credential_id: string
  identity_id: string
  password_hash: string
  webauthn_credentials: string[]
  nfc_card_id?: string | null
  pki_key_ids: string[]
  status: CredentialStatus
  created_at: string
  updated_at: string
}

export type CredentialStatus = 'active' | 'suspended' | 'compromised'

export interface MultipassSession {
  session_id: string
  identity_id: string
  device_fingerprint: string
  issued_at: string
  expires_at: string
  scopes: string[]
  token: string
}

// Auth API Requests/Responses
export interface LoginRequest {
  email: string
  password: string
}

export interface LoginResponse {
  session: MultipassSession
  token: string
}

// ============================================================================
// APPLICATIONS
// ============================================================================

export interface ApplicationModule {
  app_id: string
  name: string
  description: string
  category: AppCategory
  api_base_url: string
  required_scopes: string[]
  on_chain_contract_ref?: string | null
  enabled: boolean
  created_at: string
}

export type AppCategory = 'security' | 'invoicing' | 'ticketing' | 'healthcare' | 'supply_chain' | 'compliance' | 'finance' | 'marketplace' | { custom: string }

export interface ApplicationEvent {
  event_id: string
  app_id: string
  identity_id: string
  event_type: string
  timestamp: string
  metadata: any
}

// ============================================================================
// ASSETS & MARKET
// ============================================================================

export type AssetType = 'native' | 'utility_token' | 'equity_token' | 'carbon_credit' | 'nft' | 'subscription_pass' | { custom: string }

export interface AssetDefinition {
  asset_id: string
  asset_type: AssetType
  issuer_identity_id: string
  chain_contract_ref: string
  symbol: string
  name: string
  metadata: any
  created_at: string
}

export interface AssetPosition {
  position_id: string
  wallet_id: string
  asset_id: string
  quantity: number
  locked_quantity: number
  updated_at: string
}

export interface MarketOrder {
  order_id: string
  wallet_id: string
  asset_id: string
  order_type: OrderType
  quantity: number
  price: number
  status: OrderStatus
  created_at: string
  filled_at?: string | null
  settlement_tx?: string | null
}

export type OrderType = 'buy' | 'sell'
export type OrderStatus = 'open' | 'partially_filled' | 'filled' | 'cancelled'

// Market API responses (from AssetService)
export interface OrderBook {
  asset_id: string
  bids: OrderBookEntry[]
  asks: OrderBookEntry[]
  last_price?: number
}

export interface OrderBookEntry {
  price: number
  quantity: number
  wallet_id: string
}

export interface Trade {
  trade_id: string
  asset_id: string
  buyer_wallet_id: string
  seller_wallet_id: string
  quantity: number
  price: number
  timestamp: string
}

export interface Position {
  position_id: string
  wallet_id: string
  asset_id: string
  asset_name: string
  quantity: number
  locked_quantity: number
  average_cost: number
  current_value: number
}

export interface AssetBalance {
  wallet_id: string
  asset_id: string
  balance: number
  locked: number
}

// ============================================================================
// EVENTS & REPORTING
// ============================================================================

export interface Notification {
  notification_id: string
  identity_id: string
  notification_type: NotificationType
  source: string
  title: string
  message: string
  read: boolean
  created_at: string
}

export type NotificationType = 'security_alert' | 'payment_received' | 'payment_sent' | 'permission_change' | 'report_ready' | 'system_update' | 'application_event'

export interface ReportDefinition {
  report_def_id: string
  name: string
  description: string
  report_type: ReportType
  parameters_schema: any
  created_at: string
}

export type ReportType = 'financial' | 'asset' | 'application' | 'security' | 'compliance'

export interface ReportInstance {
  report_id: string
  report_def_id: string
  identity_id: string
  parameters: any
  generated_at: string
  data: any
}

export interface GeneratedReport {
  report_id: string
  report_type: ReportType
  name: string
  generated_at: string
  data: any
}

export type ExportFormat = 'json' | 'csv' | 'pdf'

// ============================================================================
// HARDWARE PASS
// ============================================================================

export interface HardwarePass {
  device_id: string
  identity_id: string
  public_key: string
  capabilities: HardwareCapability[]
  status: HardwareStatus
  issued_at: string
  last_used?: string | null
}

export type HardwareCapability = 'login_only' | 'sign_tx' | 'unlock_doors' | 'access_control'
export type HardwareStatus = 'active' | 'lost' | 'revoked'

// ============================================================================
// API RESPONSES
// ============================================================================

export interface ApiResponse<T> {
  data?: T
  error?: string
  message?: string
}

export interface PaginatedResponse<T> {
  data: T[]
  total: number
  limit: number
  offset: number
}

// ============================================================================
// DASHBOARD AGGREGATIONS (Frontend specific)
// ============================================================================

export interface DashboardStats {
  total_assets_value: number
  active_wallets: number
  pending_transactions: number
  kyc_status: KycStatus
  recent_activity: ActivityItem[]
}

export interface ActivityItem {
  id: string
  type: 'transaction' | 'order' | 'attestation' | 'notification'
  description: string
  timestamp: string
  status: string
}

// ============================================================================
// APPLICATION REGISTRY TYPES
// ============================================================================

export interface Application {
  app_id: string
  app_name: string
  developer: string
  category: ApplicationCategory
  description: string
  icon_url?: string | null
  homepage_url?: string | null
  requested_scopes: string[]
  risk_rating: RiskRating
  jurisdictions: string[]
  is_verified: boolean
  created_at: string
  updated_at: string
}

export type ApplicationCategory =
  | 'document_verification'
  | 'ticketing'
  | 'invoicing'
  | 'credentials'
  | 'finance'
  | 'healthcare'
  | 'real_estate'
  | 'supply_chain'
  | { custom: string }

export type RiskRating = 'low' | 'medium' | 'high'

export interface ConnectedApp {
  connection_id: string
  identity_id: string
  app_id: string
  app: Application
  granted_scopes: string[]
  status: ConnectionStatus
  connected_at: string
  last_activity?: string | null
}

export type ConnectionStatus = 'active' | 'revoked' | 'suspended'

export interface AppActivity {
  activity_id: string
  connection_id: string
  app_id: string
  activity_type: string
  description: string
  metadata?: Record<string, any>
  timestamp: string
}

export interface AppNotification {
  notification_id: string
  connection_id: string
  app_id: string
  title: string
  message: string
  level: NotificationLevel
  read: boolean
  created_at: string
}

export type NotificationLevel = 'info' | 'warning' | 'error' | 'success'

// ============================================================================
// SMART CONTRACT TYPES
// ============================================================================

export interface ContractTemplate {
  template_id: string
  template_name: string
  category: ContractCategory
  description: string
  version: string
  jurisdiction: string[]
  natural_language_terms: string
  code_hash: string
  parameters: ContractParameter[]
  is_verified: boolean
  created_by: string
  created_at: string
  updated_at: string
}

export type ContractCategory =
  | 'business'
  | 'real_estate'
  | 'family'
  | 'personal'
  | 'employment'
  | 'service_agreement'
  | { custom: string }

export interface ContractParameter {
  param_name: string
  param_type: string
  description: string
  required: boolean
  default_value?: any
}

export interface DeployedContract {
  contract_id: string
  template_id: string
  template: ContractTemplate
  deployer_id: string
  parties: ContractParty[]
  parameters: Record<string, any>
  status: ContractStatus
  chain_address?: string | null
  deployment_tx?: string | null
  natural_language_summary: string
  consent_records: ConsentRecord[]
  created_at: string
  updated_at: string
}

export interface ContractParty {
  identity_id: string
  role: string
  signed: boolean
  signature?: string | null
  signed_at?: string | null
}

export type ContractStatus =
  | 'draft'
  | 'pending_signatures'
  | 'active'
  | 'completed'
  | 'terminated'
  | 'disputed'

export interface ConsentRecord {
  consent_id: string
  identity_id: string
  contract_id: string
  consent_type: string
  granted: boolean
  timestamp: string
  signature: string
}

export interface ContractExecution {
  execution_id: string
  contract_id: string
  executed_by: string
  function_name: string
  parameters: Record<string, any>
  result?: any
  chain_tx?: string | null
  timestamp: string
}

// ============================================================================
// DOCUMENT STORAGE TYPES
// ============================================================================

export interface Document {
  document_id: string
  owner_id: string
  document_name: string
  document_type: DocumentType
  file_hash: string
  encrypted_content_uri: string
  size_bytes: number
  permissions: DocumentPermission[]
  is_immutable: boolean
  chain_anchor_tx?: string | null
  tags: string[]
  created_at: string
  updated_at: string
}

export type DocumentType =
  | 'contract'
  | 'invoice'
  | 'receipt'
  | 'certificate'
  | 'report'
  | 'compliance'
  | 'personal'
  | { custom: string }

export interface DocumentPermission {
  permission_id: string
  document_id: string
  identity_id: string
  permission_level: PermissionLevel
  granted_by: string
  granted_at: string
  expires_at?: string | null
}

export type PermissionLevel = 'read' | 'write' | 'admin'

export interface DocumentThread {
  thread_id: string
  document_id?: string | null
  participants: string[]
  topic: string
  status: ThreadStatus
  created_by: string
  created_at: string
  updated_at: string
}

export type ThreadStatus = 'active' | 'archived' | 'closed'

export interface SecureMessage {
  message_id: string
  thread_id: string
  sender_id: string
  encrypted_content: string
  signature: string
  attachments: string[]
  timestamp: string
  read_by: string[]
}

// ============================================================================
// INTERNAL MARKETS TYPES
// ============================================================================

export interface MarketListing {
  listing_id: string
  seller_id: string
  asset_id: string
  asset_type: AssetType
  quantity: number
  price_per_unit: number
  currency: string
  status: ListingStatus
  min_quantity?: number
  compliance_requirements: string[]
  jurisdictions: string[]
  created_at: string
  expires_at?: string | null
}

export type ListingStatus = 'active' | 'filled' | 'cancelled' | 'expired'

export interface MarketOrder {
  order_id: string
  buyer_id: string
  listing_id: string
  quantity: number
  total_price: number
  status: OrderStatus
  settlement_tx?: string | null
  created_at: string
  completed_at?: string | null
}

export type OrderStatus = 'pending' | 'approved' | 'settled' | 'rejected' | 'cancelled'

export interface AssetMetadata {
  asset_id: string
  asset_name: string
  asset_type: AssetType
  issuer_id: string
  total_supply: number
  compliance_level: ComplianceLevel
  sector?: MarketSector
  metadata: Record<string, any>
  created_at: string
}

export type ComplianceLevel = 'public' | 'accredited' | 'institutional' | 'regulated'

export type MarketSector =
  | 'carbon_credits'
  | 'real_estate'
  | 'equity'
  | 'commodities'
  | 'utilities'
  | 'healthcare'
  | 'ticketing'
  | { custom: string }

// ============================================================================
// IDENTITY-BOUND COMPUTE SESSIONS (IBC)
// ============================================================================

export interface IBCSession {
  session_id: string
  identity_id: string
  session_type: IBCSessionType
  started_at: string
  ended_at?: string | null
  status: IBCSessionStatus
  metadata: IBCSessionMetadata
  signature: string
  chain_anchor_tx?: string | null
}

export type IBCSessionType =
  | 'terminal'
  | 'api_calls'
  | 'git_operations'
  | 'build_logs'
  | 'debug_session'
  | 'deployment'
  | { custom: string }

export type IBCSessionStatus = 'active' | 'closed' | 'archived'

export interface IBCSessionMetadata {
  environment?: string
  project?: string
  commit_hash?: string
  branch?: string
  hostname?: string
  ip_address?: string
  [key: string]: any
}

export interface IBCEvent {
  event_id: string
  session_id: string
  timestamp: string
  event_type: string
  payload: any
  signature: string
}

export interface IBCPlayback {
  session_id: string
  events: IBCEvent[]
  timeline: IBCTimelineEntry[]
}

export interface IBCTimelineEntry {
  timestamp: string
  event_type: string
  description: string
  metadata?: any
}

// ============================================================================
// AI AGENT GOVERNANCE
// ============================================================================

export interface AIAgent {
  agent_id: string
  identity_id: string
  agent_name: string
  agent_type: AIAgentType
  capabilities: AIAgentCapability[]
  capability_tokens: CapabilityToken[]
  status: AIAgentStatus
  created_at: string
  last_active?: string | null
}

export type AIAgentType =
  | 'code_assistant'
  | 'data_analyst'
  | 'security_auditor'
  | 'compliance_monitor'
  | 'customer_support'
  | { custom: string }

export type AIAgentStatus = 'active' | 'suspended' | 'revoked'

export interface AIAgentCapability {
  capability: string
  scope: string[]
  granted_at: string
  expires_at?: string | null
}

export interface CapabilityToken {
  token_id: string
  agent_id: string
  capability: string
  scope: string[]
  issued_at: string
  expires_at?: string | null
  revoked: boolean
}

export interface AIAgentActivity {
  activity_id: string
  agent_id: string
  action_type: string
  resource_accessed?: string
  result: 'success' | 'failure' | 'denied'
  timestamp: string
  metadata?: any
}

// ============================================================================
// PORTABLE DEVELOPER ENVIRONMENTS (PDE)
// ============================================================================

export interface PDESnapshot {
  snapshot_id: string
  identity_id: string
  name: string
  description?: string
  environment_type: PDEType
  config: PDEConfig
  size_bytes: number
  created_at: string
  last_used?: string | null
  tags: string[]
}

export type PDEType =
  | 'node_js'
  | 'python'
  | 'rust'
  | 'docker'
  | 'kubernetes'
  | { custom: string }

export interface PDEConfig {
  base_image?: string
  packages: string[]
  environment_vars: Record<string, string>
  volumes?: string[]
  ports?: number[]
  startup_commands?: string[]
  [key: string]: any
}

export interface PDEDeployment {
  deployment_id: string
  snapshot_id: string
  identity_id: string
  status: PDEDeploymentStatus
  endpoint_url?: string
  deployed_at: string
  shutdown_at?: string | null
}

export type PDEDeploymentStatus = 'deploying' | 'running' | 'stopped' | 'failed'

// ============================================================================
// IDENTITY-ATTACHED SIGNING & PROVENANCE
// ============================================================================

export interface CodeSignature {
  signature_id: string
  identity_id: string
  artifact_hash: string
  artifact_type: ArtifactType
  signature_algorithm: string
  signature: string
  timestamp: string
  chain_anchor_tx?: string | null
  metadata?: CodeSignatureMetadata
}

export type ArtifactType =
  | 'source_code'
  | 'binary'
  | 'container_image'
  | 'package'
  | 'configuration'
  | 'documentation'
  | { custom: string }

export interface CodeSignatureMetadata {
  file_name?: string
  version?: string
  commit_hash?: string
  build_id?: string
  [key: string]: any
}

export interface ProvenanceChain {
  artifact_hash: string
  signatures: CodeSignature[]
  build_info?: BuildProvenance
  dependencies?: DependencyProvenance[]
}

export interface BuildProvenance {
  builder_identity_id: string
  build_timestamp: string
  source_repo?: string
  commit_hash?: string
  build_environment?: Record<string, string>
}

export interface DependencyProvenance {
  dependency_name: string
  version: string
  hash: string
  source?: string
}

// ============================================================================
// ENTERPRISE KNOWLEDGE VAULT
// ============================================================================

export interface KnowledgeNode {
  node_id: string
  identity_id: string
  node_type: KnowledgeNodeType
  title: string
  content: string
  tags: string[]
  relations: KnowledgeRelation[]
  access_level: 'private' | 'team' | 'organization' | 'public'
  created_at: string
  updated_at: string
}

export type KnowledgeNodeType =
  | 'document'
  | 'code_snippet'
  | 'api_reference'
  | 'architecture_diagram'
  | 'decision_record'
  | 'troubleshooting_guide'
  | { custom: string }

export interface KnowledgeRelation {
  related_node_id: string
  relation_type: string
  strength: number
}

export interface KnowledgeQuery {
  query: string
  filters?: KnowledgeFilter
  limit?: number
}

export interface KnowledgeFilter {
  node_types?: KnowledgeNodeType[]
  tags?: string[]
  access_levels?: string[]
  date_range?: { from: string; to: string }
}

export interface KnowledgeSearchResult {
  nodes: KnowledgeNode[]
  relevance_scores: Record<string, number>
  suggested_relations: KnowledgeRelation[]
}

// ============================================================================
// ZERO-TRUST COLLABORATION CAPSULES
// ============================================================================

export interface CollaborationCapsule {
  capsule_id: string
  name: string
  description?: string
  creator_id: string
  participants: CapsuleParticipant[]
  resources: CapsuleResource[]
  policies: CapsulePolicy[]
  status: CapsuleStatus
  created_at: string
  expires_at?: string | null
}

export interface CapsuleParticipant {
  identity_id: string
  role: string
  permissions: string[]
  joined_at: string
  last_active?: string | null
}

export interface CapsuleResource {
  resource_id: string
  resource_type: string
  resource_ref: string
  access_level: 'read' | 'write' | 'admin'
  added_at: string
}

export interface CapsulePolicy {
  policy_id: string
  policy_type: string
  rules: Record<string, any>
  enforced: boolean
}

export type CapsuleStatus = 'active' | 'suspended' | 'expired' | 'archived'

export interface CapsuleActivity {
  activity_id: string
  capsule_id: string
  identity_id: string
  action: string
  resource_id?: string
  timestamp: string
  result: 'allowed' | 'denied'
}

// ERROR TYPES
// ============================================================================

export interface EnterpriseError {
  code: string
  message: string
  details?: any
}
