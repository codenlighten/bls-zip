// Enterprise Multipass API Client
// Matches backend API endpoints exactly

import type {
  // Auth
  LoginRequest,
  LoginResponse,
  MultipassSession,
  // Identity
  IdentityProfile,
  IdentityAttestation,
  KycStatus,
  AttestationType,
  // Wallet
  WalletAccount,
  WalletBalance,
  WalletTransaction,
  AssetType,
  // Assets & Market
  AssetDefinition,
  AssetPosition,
  AssetBalance,
  MarketOrder,
  OrderBook,
  OrderType,
  Trade,
  Position,
  // Events & Reporting
  Notification,
  NotificationType,
  ReportDefinition,
  ReportInstance,
  GeneratedReport,
  ReportType,
  ExportFormat,
  // Applications
  ApplicationModule,
  Application,
  ConnectedApp,
  AppActivity,
  AppNotification,
  ApplicationCategory,
  ConnectionStatus,
  NotificationLevel,
  // Smart Contracts
  ContractTemplate,
  DeployedContract,
  ContractParty,
  ContractExecution,
  ContractCategory,
  ContractStatus,
  // Documents & Messaging
  Document,
  DocumentType,
  DocumentPermission,
  PermissionLevel,
  DocumentThread,
  SecureMessage,
  ThreadStatus,
  // Internal Markets
  MarketListing,
  MarketOrder,
  AssetMetadata,
  ListingStatus,
  OrderStatus,
  ComplianceLevel,
  MarketSector,
  // Hardware
  HardwarePass,
  // Common
  ApiResponse,
} from '@/types'

const API_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080'

class ApiClient {
  private token: string | null = null

  constructor() {
    if (typeof window !== 'undefined') {
      this.token = localStorage.getItem('auth_token')
    }
  }

  setToken(token: string) {
    this.token = token
    if (typeof window !== 'undefined') {
      localStorage.setItem('auth_token', token)
    }
  }

  clearToken() {
    this.token = null
    if (typeof window !== 'undefined') {
      localStorage.removeItem('auth_token')
      localStorage.removeItem('user_identity')
    }
  }

  private async request<T>(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<ApiResponse<T>> {
    const headers: HeadersInit = {
      'Content-Type': 'application/json',
      ...options.headers,
    }

    if (this.token) {
      headers['Authorization'] = `Bearer ${this.token}`
    }

    try {
      const response = await fetch(`${API_URL}${endpoint}`, {
        ...options,
        headers,
      })

      if (!response.ok) {
        const error = await response.text()
        return { error: error || response.statusText }
      }

      // Handle empty responses
      const text = await response.text()
      const data = text ? JSON.parse(text) : null

      return { data }
    } catch (error) {
      return { error: error instanceof Error ? error.message : 'Network error' }
    }
  }

  // ============================================================================
  // AUTH API
  // ============================================================================

  async register(identity_id: string, password: string): Promise<ApiResponse<{ credential_id: string }>> {
    return this.request('/api/auth/register', {
      method: 'POST',
      body: JSON.stringify({ identity_id, password }),
    })
  }

  async login(credentials: LoginRequest): Promise<ApiResponse<LoginResponse>> {
    const result = await this.request<LoginResponse>('/api/auth/login', {
      method: 'POST',
      body: JSON.stringify(credentials),
    })

    if (result.data) {
      this.setToken(result.data.token)
    }

    return result
  }

  async logout(session_id?: string): Promise<ApiResponse<void>> {
    if (session_id) {
      await this.request('/api/auth/logout', {
        method: 'POST',
        body: JSON.stringify({ session_id }),
      })
    }
    this.clearToken()
    return { data: undefined }
  }

  async refreshSession(session_id: string): Promise<ApiResponse<string>> {
    return this.request(`/api/auth/refresh/${session_id}`, {
      method: 'POST',
    })
  }

  async verifyToken(token: string): Promise<ApiResponse<{ identity_id: string; valid: boolean }>> {
    return this.request('/api/auth/verify', {
      method: 'POST',
      body: JSON.stringify({ token }),
    })
  }

  async getSession(session_id: string): Promise<ApiResponse<MultipassSession>> {
    return this.request(`/api/auth/session/${session_id}`)
  }

  // ============================================================================
  // IDENTITY API
  // ============================================================================

  async createIdentity(data: {
    legal_name: string
    email: string
    org_name?: string
  }): Promise<ApiResponse<{ identity: IdentityProfile; public_key: string }>> {
    return this.request('/api/identity/create', {
      method: 'POST',
      body: JSON.stringify(data),
    })
  }

  async getIdentity(id: string): Promise<ApiResponse<IdentityProfile>> {
    return this.request(`/api/identity/${id}`)
  }

  async getIdentityByEmail(email: string): Promise<ApiResponse<IdentityProfile>> {
    return this.request(`/api/identity/email/${email}`)
  }

  async listIdentities(limit: number = 100, offset: number = 0): Promise<ApiResponse<IdentityProfile[]>> {
    return this.request(`/api/identity/list?limit=${limit}&offset=${offset}`)
  }

  async updateKycStatus(
    id: string,
    kyc_status: KycStatus,
    aml_risk_score: number
  ): Promise<ApiResponse<void>> {
    return this.request(`/api/identity/${id}/kyc-status`, {
      method: 'PUT',
      body: JSON.stringify({ kyc_status, aml_risk_score }),
    })
  }

  async createAttestation(
    identity_id: string,
    data: {
      attestation_type: AttestationType
      evidence_refs: string[]
      issuer: string
      valid_to?: string
      anchor_to_chain: boolean
    }
  ): Promise<ApiResponse<IdentityAttestation>> {
    return this.request(`/api/identity/${identity_id}/attestations`, {
      method: 'POST',
      body: JSON.stringify(data),
    })
  }

  async getAttestations(identity_id: string): Promise<ApiResponse<IdentityAttestation[]>> {
    return this.request(`/api/identity/${identity_id}/attestations`)
  }

  async lookupIdentityByEmail(email: string): Promise<ApiResponse<{ identity_id: string }>> {
    return this.request(`/api/identity/lookup?email=${encodeURIComponent(email)}`)
  }

  async revokeAttestation(attestation_id: string): Promise<ApiResponse<void>> {
    return this.request(`/api/identity/attestations/${attestation_id}`, {
      method: 'DELETE',
    })
  }

  // ============================================================================
  // WALLET API
  // ============================================================================

  async createWallet(identity_id: string, labels: string[]): Promise<ApiResponse<WalletAccount>> {
    return this.request('/api/wallet/create', {
      method: 'POST',
      body: JSON.stringify({ identity_id, labels }),
    })
  }

  async getWallet(wallet_id: string): Promise<ApiResponse<WalletAccount>> {
    return this.request(`/api/wallet/${wallet_id}`)
  }

  async getWalletBalances(wallet_id: string): Promise<ApiResponse<WalletBalance[]>> {
    return this.request(`/api/wallet/${wallet_id}/balances`)
  }

  async getWalletTransactions(
    wallet_id: string,
    limit: number = 100,
    offset: number = 0
  ): Promise<ApiResponse<WalletTransaction[]>> {
    return this.request(`/api/wallet/${wallet_id}/transactions?limit=${limit}&offset=${offset}`)
  }

  async transfer(
    wallet_id: string,
    to_address: string,
    asset_type: AssetType,
    amount: number
  ): Promise<ApiResponse<WalletTransaction>> {
    return this.request(`/api/wallet/${wallet_id}/transfer`, {
      method: 'POST',
      body: JSON.stringify({ to_address, asset_type, amount }),
    })
  }

  async syncWalletBalances(wallet_id: string): Promise<ApiResponse<void>> {
    return this.request(`/api/wallet/${wallet_id}/sync`, {
      method: 'POST',
    })
  }

  async getIdentityWallets(identity_id: string): Promise<ApiResponse<WalletAccount[]>> {
    return this.request(`/api/wallet/identity/${identity_id}`)
  }

  // ============================================================================
  // ASSET API
  // ============================================================================

  async defineAsset(data: {
    name: string
    symbol: string
    asset_type: AssetType
    total_supply: number
    metadata: any
  }): Promise<ApiResponse<{ asset: AssetDefinition }>> {
    return this.request('/api/assets/define', {
      method: 'POST',
      body: JSON.stringify(data),
    })
  }

  async listAssets(limit: number = 20, offset: number = 0): Promise<ApiResponse<AssetDefinition[]>> {
    return this.request(`/api/assets/list?limit=${limit}&offset=${offset}`)
  }

  async getAsset(asset_id: string): Promise<ApiResponse<AssetDefinition>> {
    return this.request(`/api/assets/${asset_id}`)
  }

  async issueAsset(
    asset_id: string,
    to_wallet: string,
    amount: number
  ): Promise<ApiResponse<void>> {
    return this.request(`/api/assets/${asset_id}/issue`, {
      method: 'POST',
      body: JSON.stringify({ to_wallet, amount }),
    })
  }

  async transferAsset(
    asset_id: string,
    from_wallet: string,
    to_wallet: string,
    amount: number
  ): Promise<ApiResponse<void>> {
    return this.request(`/api/assets/${asset_id}/transfer`, {
      method: 'POST',
      body: JSON.stringify({ from_wallet, to_wallet, amount }),
    })
  }

  async getAssetBalance(asset_id: string, wallet_id: string): Promise<ApiResponse<AssetBalance>> {
    return this.request(`/api/assets/${asset_id}/balance/${wallet_id}`)
  }

  // ============================================================================
  // MARKET API
  // ============================================================================

  async createOrder(data: {
    wallet_id: string
    asset_id: string
    order_type: OrderType
    quantity: number
    price: number
  }): Promise<ApiResponse<{ order: MarketOrder }>> {
    return this.request('/api/market/orders', {
      method: 'POST',
      body: JSON.stringify(data),
    })
  }

  async getOrder(order_id: string): Promise<ApiResponse<MarketOrder>> {
    return this.request(`/api/market/orders/${order_id}`)
  }

  async cancelOrder(order_id: string): Promise<ApiResponse<void>> {
    return this.request(`/api/market/orders/${order_id}/cancel`, {
      method: 'PUT',
    })
  }

  async getWalletOrders(
    wallet_id: string,
    limit: number = 20,
    offset: number = 0
  ): Promise<ApiResponse<MarketOrder[]>> {
    return this.request(`/api/market/wallet/${wallet_id}/orders?limit=${limit}&offset=${offset}`)
  }

  async getOrderBook(asset_id: string): Promise<ApiResponse<OrderBook>> {
    return this.request(`/api/market/orderbook/${asset_id}`)
  }

  async getPositions(wallet_id: string): Promise<ApiResponse<Position[]>> {
    return this.request(`/api/market/positions/${wallet_id}`)
  }

  async getTrades(
    asset_id: string,
    limit: number = 20,
    offset: number = 0
  ): Promise<ApiResponse<Trade[]>> {
    return this.request(`/api/market/trades/${asset_id}?limit=${limit}&offset=${offset}`)
  }

  // ============================================================================
  // NOTIFICATIONS API
  // ============================================================================

  async createNotification(data: {
    identity_id: string
    notification_type: NotificationType
    title: string
    message: string
    metadata: any
  }): Promise<ApiResponse<{ notification: Notification }>> {
    return this.request('/api/notifications', {
      method: 'POST',
      body: JSON.stringify(data),
    })
  }

  async getNotification(notification_id: string): Promise<ApiResponse<Notification>> {
    return this.request(`/api/notifications/${notification_id}`)
  }

  async markNotificationAsRead(notification_id: string): Promise<ApiResponse<void>> {
    return this.request(`/api/notifications/${notification_id}/read`, {
      method: 'PUT',
    })
  }

  async deleteNotification(notification_id: string): Promise<ApiResponse<void>> {
    return this.request(`/api/notifications/${notification_id}`, {
      method: 'DELETE',
    })
  }

  async getIdentityNotifications(
    identity_id: string,
    limit: number = 20,
    offset: number = 0
  ): Promise<ApiResponse<Notification[]>> {
    return this.request(`/api/notifications/identity/${identity_id}?limit=${limit}&offset=${offset}`)
  }

  async getUnreadCount(identity_id: string): Promise<ApiResponse<{ count: number }>> {
    return this.request(`/api/notifications/identity/${identity_id}/unread`)
  }

  async markAllAsRead(identity_id: string): Promise<ApiResponse<{ marked_count: number }>> {
    return this.request(`/api/notifications/identity/${identity_id}/mark_all_read`, {
      method: 'PUT',
    })
  }

  // ============================================================================
  // REPORTS API
  // ============================================================================

  async createReportDefinition(data: {
    name: string
    description: string
    report_type: ReportType
    sql_template: string
    parameters: string[]
  }): Promise<ApiResponse<{ definition: ReportDefinition }>> {
    return this.request('/api/reports/definitions', {
      method: 'POST',
      body: JSON.stringify(data),
    })
  }

  async listReportDefinitions(): Promise<ApiResponse<ReportDefinition[]>> {
    return this.request('/api/reports/definitions')
  }

  async getReportDefinition(report_id: string): Promise<ApiResponse<ReportDefinition>> {
    return this.request(`/api/reports/definitions/${report_id}`)
  }

  async deleteReportDefinition(report_id: string): Promise<ApiResponse<void>> {
    return this.request(`/api/reports/definitions/${report_id}`, {
      method: 'DELETE',
    })
  }

  async generateReport(data: {
    report_id: string
    identity_id: string
    parameters: any
    format: ExportFormat
  }): Promise<ApiResponse<{ report: GeneratedReport }>> {
    return this.request('/api/reports/generate', {
      method: 'POST',
      body: JSON.stringify(data),
    })
  }

  async getGeneratedReport(generated_report_id: string): Promise<ApiResponse<GeneratedReport>> {
    return this.request(`/api/reports/${generated_report_id}`)
  }

  async listGeneratedReports(
    identity_id: string,
    limit: number = 20,
    offset: number = 0
  ): Promise<ApiResponse<GeneratedReport[]>> {
    return this.request(`/api/reports/identity/${identity_id}?limit=${limit}&offset=${offset}`)
  }

  // ============================================================================
  // APPLICATIONS API
  // ============================================================================

  async listApplications(limit: number = 20, offset: number = 0): Promise<ApiResponse<ApplicationModule[]>> {
    return this.request(`/api/applications?limit=${limit}&offset=${offset}`)
  }

  async getApplication(app_id: string): Promise<ApiResponse<ApplicationModule>> {
    return this.request(`/api/applications/${app_id}`)
  }

  // ============================================================================
  // HARDWARE API
  // ============================================================================

  async listHardwareDevices(identity_id: string): Promise<ApiResponse<HardwarePass[]>> {
    return this.request(`/api/hardware/identity/${identity_id}`)
  }

  async registerHardwareDevice(data: {
    identity_id: string
    public_key: string
    capabilities: string[]
  }): Promise<ApiResponse<HardwarePass>> {
    return this.request('/api/hardware/register', {
      method: 'POST',
      body: JSON.stringify(data),
    })
  }

  async revokeHardwareDevice(device_id: string): Promise<ApiResponse<void>> {
    return this.request(`/api/hardware/${device_id}/revoke`, {
      method: 'PUT',
    })
  }

  // ============================================================================
  // APPLICATION REGISTRY API
  // ============================================================================

  async getAvailableApps(): Promise<ApiResponse<Application[]>> {
    return this.request('/api/apps/registry')
  }

  async getConnectedApps(identity_id: string): Promise<ApiResponse<ConnectedApp[]>> {
    return this.request(`/api/apps/identity/${identity_id}/connections`)
  }

  async connectApp(data: {
    identity_id: string
    app_id: string
    granted_scopes: string[]
  }): Promise<ApiResponse<{ connection: ConnectedApp }>> {
    return this.request('/api/apps/connect', {
      method: 'POST',
      body: JSON.stringify(data),
    })
  }

  async revokeAppConnection(connection_id: string): Promise<ApiResponse<void>> {
    return this.request(`/api/apps/connections/${connection_id}/revoke`, {
      method: 'PUT',
    })
  }

  async getAppActivity(connection_id: string, limit = 20, offset = 0): Promise<ApiResponse<AppActivity[]>> {
    return this.request(`/api/apps/connections/${connection_id}/activity?limit=${limit}&offset=${offset}`)
  }

  async getAppNotifications(connection_id: string): Promise<ApiResponse<AppNotification[]>> {
    return this.request(`/api/apps/connections/${connection_id}/notifications`)
  }

  async markNotificationRead(notification_id: string): Promise<ApiResponse<void>> {
    return this.request(`/api/apps/notifications/${notification_id}/read`, {
      method: 'PUT',
    })
  }

  // ============================================================================
  // SMART CONTRACTS API
  // ============================================================================

  async getContractTemplates(category?: string): Promise<ApiResponse<ContractTemplate[]>> {
    const params = category ? `?category=${category}` : ''
    return this.request(`/api/contracts/templates${params}`)
  }

  async getDeployedContracts(identity_id: string): Promise<ApiResponse<DeployedContract[]>> {
    return this.request(`/api/contracts/identity/${identity_id}`)
  }

  async deployContract(data: {
    template_id: string
    deployer_id: string
    parties: ContractParty[]
    parameters: Record<string, any>
  }): Promise<ApiResponse<{ contract: DeployedContract }>> {
    return this.request('/api/contracts/deploy', {
      method: 'POST',
      body: JSON.stringify(data),
    })
  }

  async signContract(contract_id: string, identity_id: string): Promise<ApiResponse<void>> {
    return this.request(`/api/contracts/${contract_id}/sign`, {
      method: 'POST',
      body: JSON.stringify({ identity_id }),
    })
  }

  async executeContract(contract_id: string, data: {
    executed_by: string
    function_name: string
    parameters: Record<string, any>
  }): Promise<ApiResponse<{ execution: ContractExecution }>> {
    return this.request(`/api/contracts/${contract_id}/execute`, {
      method: 'POST',
      body: JSON.stringify(data),
    })
  }

  async terminateContract(contract_id: string, terminated_by: string): Promise<ApiResponse<void>> {
    return this.request(`/api/contracts/${contract_id}/terminate`, {
      method: 'PUT',
      body: JSON.stringify({ terminated_by }),
    })
  }

  // ============================================================================
  // DOCUMENT STORAGE API
  // ============================================================================

  async getDocuments(owner_id: string): Promise<ApiResponse<Document[]>> {
    return this.request(`/api/documents/owner/${owner_id}`)
  }

  async uploadDocument(data: {
    owner_id: string
    document_name: string
    document_type: DocumentType
    encrypted_content: string
    tags?: string[]
    is_immutable?: boolean
  }): Promise<ApiResponse<{ document: Document }>> {
    return this.request('/api/documents/upload', {
      method: 'POST',
      body: JSON.stringify(data),
    })
  }

  async grantDocumentPermission(document_id: string, data: {
    identity_id: string
    permission_level: PermissionLevel
    granted_by: string
    expires_at?: string
  }): Promise<ApiResponse<{ permission: DocumentPermission }>> {
    return this.request(`/api/documents/${document_id}/permissions`, {
      method: 'POST',
      body: JSON.stringify(data),
    })
  }

  async revokeDocumentPermission(permission_id: string): Promise<ApiResponse<void>> {
    return this.request(`/api/documents/permissions/${permission_id}`, {
      method: 'DELETE',
    })
  }

  // ============================================================================
  // MESSAGING API
  // ============================================================================

  async getThreads(identity_id: string): Promise<ApiResponse<DocumentThread[]>> {
    return this.request(`/api/messaging/identity/${identity_id}/threads`)
  }

  async createThread(data: {
    created_by: string
    participants: string[]
    topic: string
    document_id?: string
  }): Promise<ApiResponse<{ thread: DocumentThread }>> {
    return this.request('/api/messaging/threads', {
      method: 'POST',
      body: JSON.stringify(data),
    })
  }

  async getMessages(thread_id: string): Promise<ApiResponse<SecureMessage[]>> {
    return this.request(`/api/messaging/threads/${thread_id}/messages`)
  }

  async sendMessage(thread_id: string, data: {
    sender_id: string
    content: string
    attachments?: string[]
  }): Promise<ApiResponse<{ message: SecureMessage }>> {
    return this.request(`/api/messaging/threads/${thread_id}/messages`, {
      method: 'POST',
      body: JSON.stringify(data),
    })
  }

  async markMessageRead(message_id: string, reader_id: string): Promise<ApiResponse<void>> {
    return this.request(`/api/messaging/messages/${message_id}/read`, {
      method: 'PUT',
      body: JSON.stringify({ reader_id }),
    })
  }

  // ============================================================================
  // INTERNAL MARKETS API
  // ============================================================================

  async getMarketListings(sector?: string): Promise<ApiResponse<MarketListing[]>> {
    const params = sector ? `?sector=${sector}` : ''
    return this.request(`/api/markets/listings${params}`)
  }

  async createListing(data: {
    seller_id: string
    asset_id: string
    asset_type: AssetType
    quantity: number
    price_per_unit: number
    currency: string
    compliance_requirements?: string[]
    jurisdictions?: string[]
    expires_at?: string
  }): Promise<ApiResponse<{ listing: MarketListing }>> {
    return this.request('/api/markets/listings', {
      method: 'POST',
      body: JSON.stringify(data),
    })
  }

  async createMarketOrder(data: {
    buyer_id: string
    listing_id: string
    quantity: number
  }): Promise<ApiResponse<{ order: MarketOrder }>> {
    return this.request('/api/markets/orders', {
      method: 'POST',
      body: JSON.stringify(data),
    })
  }

  async getMyOrders(identity_id: string): Promise<ApiResponse<MarketOrder[]>> {
    return this.request(`/api/markets/orders/identity/${identity_id}`)
  }

  async getAssetMetadata(asset_id: string): Promise<ApiResponse<AssetMetadata>> {
    return this.request(`/api/markets/assets/${asset_id}`)
  }

  async cancelListing(listing_id: string): Promise<ApiResponse<void>> {
    return this.request(`/api/markets/listings/${listing_id}/cancel`, {
      method: 'PUT',
    })
  }

  // ============================================================================
  // IDENTITY-BOUND COMPUTE SESSIONS (IBC) API
  // ============================================================================

  async createIBCSession(data: {
    identity_id: string
    session_type: string
    metadata: Record<string, any>
  }): Promise<ApiResponse<any>> {
    return this.request('/api/ibc/sessions', {
      method: 'POST',
      body: JSON.stringify(data),
    })
  }

  async getIBCSessions(identity_id: string): Promise<ApiResponse<any[]>> {
    return this.request(`/api/ibc/sessions/identity/${identity_id}`)
  }

  async getIBCSession(session_id: string): Promise<ApiResponse<any>> {
    return this.request(`/api/ibc/sessions/${session_id}`)
  }

  async closeIBCSession(session_id: string): Promise<ApiResponse<void>> {
    return this.request(`/api/ibc/sessions/${session_id}/close`, {
      method: 'PUT',
    })
  }

  async addIBCEvent(session_id: string, data: {
    event_type: string
    payload: any
  }): Promise<ApiResponse<any>> {
    return this.request(`/api/ibc/sessions/${session_id}/events`, {
      method: 'POST',
      body: JSON.stringify(data),
    })
  }

  async getIBCPlayback(session_id: string): Promise<ApiResponse<any>> {
    return this.request(`/api/ibc/sessions/${session_id}/playback`)
  }

  // ============================================================================
  // AI AGENT GOVERNANCE API
  // ============================================================================

  async registerAIAgent(data: {
    identity_id: string
    agent_name: string
    agent_type: string
    capabilities: any[]
  }): Promise<ApiResponse<any>> {
    return this.request('/api/ai/agents', {
      method: 'POST',
      body: JSON.stringify(data),
    })
  }

  async getAIAgents(identity_id: string): Promise<ApiResponse<any[]>> {
    return this.request(`/api/ai/agents/identity/${identity_id}`)
  }

  async getAIAgent(agent_id: string): Promise<ApiResponse<any>> {
    return this.request(`/api/ai/agents/${agent_id}`)
  }

  async updateAgentCapabilities(agent_id: string, data: {
    capabilities: any[]
  }): Promise<ApiResponse<any>> {
    return this.request(`/api/ai/agents/${agent_id}/capabilities`, {
      method: 'PUT',
      body: JSON.stringify(data),
    })
  }

  async issueCapabilityToken(agent_id: string, data: {
    capability: string
    scope: string[]
    expires_at?: string
  }): Promise<ApiResponse<any>> {
    return this.request(`/api/ai/agents/${agent_id}/tokens`, {
      method: 'POST',
      body: JSON.stringify(data),
    })
  }

  async revokeCapabilityToken(token_id: string): Promise<ApiResponse<void>> {
    return this.request(`/api/ai/tokens/${token_id}/revoke`, {
      method: 'PUT',
    })
  }

  async getAgentActivity(agent_id: string, limit?: number): Promise<ApiResponse<any[]>> {
    const query = limit ? `?limit=${limit}` : ''
    return this.request(`/api/ai/agents/${agent_id}/activity${query}`)
  }

  async suspendAgent(agent_id: string): Promise<ApiResponse<void>> {
    return this.request(`/api/ai/agents/${agent_id}/suspend`, {
      method: 'PUT',
    })
  }

  // ============================================================================
  // PORTABLE DEVELOPER ENVIRONMENTS (PDE) API
  // ============================================================================

  async createPDESnapshot(data: {
    identity_id: string
    name: string
    description?: string
    environment_type: string
    config: Record<string, any>
    tags?: string[]
  }): Promise<ApiResponse<any>> {
    return this.request('/api/pde/snapshots', {
      method: 'POST',
      body: JSON.stringify(data),
    })
  }

  async getPDESnapshots(identity_id: string): Promise<ApiResponse<any[]>> {
    return this.request(`/api/pde/snapshots/identity/${identity_id}`)
  }

  async getPDESnapshot(snapshot_id: string): Promise<ApiResponse<any>> {
    return this.request(`/api/pde/snapshots/${snapshot_id}`)
  }

  async deployPDESnapshot(snapshot_id: string): Promise<ApiResponse<any>> {
    return this.request(`/api/pde/snapshots/${snapshot_id}/deploy`, {
      method: 'POST',
    })
  }

  async stopPDEDeployment(deployment_id: string): Promise<ApiResponse<void>> {
    return this.request(`/api/pde/deployments/${deployment_id}/stop`, {
      method: 'PUT',
    })
  }

  async getMyPDEDeployments(identity_id: string): Promise<ApiResponse<any[]>> {
    return this.request(`/api/pde/deployments/identity/${identity_id}`)
  }

  async deletePDESnapshot(snapshot_id: string): Promise<ApiResponse<void>> {
    return this.request(`/api/pde/snapshots/${snapshot_id}`, {
      method: 'DELETE',
    })
  }

  // ============================================================================
  // CODE SIGNING & PROVENANCE API
  // ============================================================================

  async signArtifact(data: {
    identity_id: string
    artifact_hash: string
    artifact_type: string
    metadata?: Record<string, any>
  }): Promise<ApiResponse<any>> {
    return this.request('/api/provenance/sign', {
      method: 'POST',
      body: JSON.stringify(data),
    })
  }

  async verifySignature(signature_id: string): Promise<ApiResponse<any>> {
    return this.request(`/api/provenance/signatures/${signature_id}/verify`)
  }

  async getProvenanceChain(artifact_hash: string): Promise<ApiResponse<any>> {
    return this.request(`/api/provenance/chain/${artifact_hash}`)
  }

  async getMySignatures(identity_id: string): Promise<ApiResponse<any[]>> {
    return this.request(`/api/provenance/signatures/identity/${identity_id}`)
  }

  async addBuildProvenance(artifact_hash: string, data: {
    builder_identity_id: string
    source_repo?: string
    commit_hash?: string
    build_environment?: Record<string, string>
  }): Promise<ApiResponse<any>> {
    return this.request(`/api/provenance/artifacts/${artifact_hash}/build`, {
      method: 'POST',
      body: JSON.stringify(data),
    })
  }

  // ============================================================================
  // KNOWLEDGE VAULT API
  // ============================================================================

  async createKnowledgeNode(data: {
    identity_id: string
    node_type: string
    title: string
    content: string
    tags?: string[]
    access_level?: string
  }): Promise<ApiResponse<any>> {
    return this.request('/api/knowledge/nodes', {
      method: 'POST',
      body: JSON.stringify(data),
    })
  }

  async getKnowledgeNodes(identity_id: string): Promise<ApiResponse<any[]>> {
    return this.request(`/api/knowledge/nodes/identity/${identity_id}`)
  }

  async getKnowledgeNode(node_id: string): Promise<ApiResponse<any>> {
    return this.request(`/api/knowledge/nodes/${node_id}`)
  }

  async updateKnowledgeNode(node_id: string, data: {
    title?: string
    content?: string
    tags?: string[]
  }): Promise<ApiResponse<any>> {
    return this.request(`/api/knowledge/nodes/${node_id}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    })
  }

  async searchKnowledge(query: {
    query: string
    filters?: Record<string, any>
    limit?: number
  }): Promise<ApiResponse<any>> {
    return this.request('/api/knowledge/search', {
      method: 'POST',
      body: JSON.stringify(query),
    })
  }

  async addKnowledgeRelation(node_id: string, data: {
    related_node_id: string
    relation_type: string
    strength: number
  }): Promise<ApiResponse<void>> {
    return this.request(`/api/knowledge/nodes/${node_id}/relations`, {
      method: 'POST',
      body: JSON.stringify(data),
    })
  }

  async deleteKnowledgeNode(node_id: string): Promise<ApiResponse<void>> {
    return this.request(`/api/knowledge/nodes/${node_id}`, {
      method: 'DELETE',
    })
  }

  // ============================================================================
  // COLLABORATION CAPSULES API
  // ============================================================================

  async createCapsule(data: {
    creator_id: string
    name: string
    description?: string
    participants: any[]
    policies?: any[]
    expires_at?: string
  }): Promise<ApiResponse<any>> {
    return this.request('/api/capsules', {
      method: 'POST',
      body: JSON.stringify(data),
    })
  }

  async getMyCapsules(identity_id: string): Promise<ApiResponse<any[]>> {
    return this.request(`/api/capsules/identity/${identity_id}`)
  }

  async getCapsule(capsule_id: string): Promise<ApiResponse<any>> {
    return this.request(`/api/capsules/${capsule_id}`)
  }

  async addCapsuleParticipant(capsule_id: string, data: {
    identity_id: string
    role: string
    permissions: string[]
  }): Promise<ApiResponse<void>> {
    return this.request(`/api/capsules/${capsule_id}/participants`, {
      method: 'POST',
      body: JSON.stringify(data),
    })
  }

  async removeCapsuleParticipant(capsule_id: string, identity_id: string): Promise<ApiResponse<void>> {
    return this.request(`/api/capsules/${capsule_id}/participants/${identity_id}`, {
      method: 'DELETE',
    })
  }

  async addCapsuleResource(capsule_id: string, data: {
    resource_type: string
    resource_ref: string
    access_level: string
  }): Promise<ApiResponse<void>> {
    return this.request(`/api/capsules/${capsule_id}/resources`, {
      method: 'POST',
      body: JSON.stringify(data),
    })
  }

  async getCapsuleActivity(capsule_id: string, limit?: number): Promise<ApiResponse<any[]>> {
    const query = limit ? `?limit=${limit}` : ''
    return this.request(`/api/capsules/${capsule_id}/activity${query}`)
  }

  async suspendCapsule(capsule_id: string): Promise<ApiResponse<void>> {
    return this.request(`/api/capsules/${capsule_id}/suspend`, {
      method: 'PUT',
    })
  }

  async archiveCapsule(capsule_id: string): Promise<ApiResponse<void>> {
    return this.request(`/api/capsules/${capsule_id}/archive`, {
      method: 'PUT',
    })
  }
}

export const api = new ApiClient()
