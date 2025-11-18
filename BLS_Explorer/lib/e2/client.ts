import axios, { AxiosInstance } from 'axios';

// ==================
// Request/Response Types
// ==================

export interface E2LoginRequest {
  email: string;
  password: string;
}

export interface E2SignupRequest {
  email: string;
  password: string;
  first_name: string;
  last_name: string;
}

export interface E2AuthResponse {
  token: string;
  identity_id: string;
  email: string;
}

export interface E2User {
  identity_id: string;
  email: string;
  first_name: string;
  last_name: string;
  kyc_level: number;
  created_at: string;
}

export interface E2Wallet {
  wallet_id: string;
  identity_id: string;
  name: string;
  asset_type: string;
  address: string;
  balance: number;
  created_at: string;
}

export interface E2WalletBalance {
  wallet_id: string;
  balance: number;
  pending_balance: number;
  last_updated: string;
}

export interface E2Identity {
  identity_id: string;
  email: string;
  first_name: string;
  last_name: string;
  kyc_level: number;
  verification_status: string;
  created_at: string;
  updated_at: string;
}

export interface E2Attestation {
  attestation_id: string;
  identity_id: string;
  attestation_type: string;
  data: Record<string, any>;
  status: string;
  created_at: string;
}

export interface E2Transaction {
  tx_id: string;
  wallet_id: string;
  tx_hash: string;
  amount: number;
  fee: number;
  status: string;
  created_at: string;
}

/**
 * E2 Multipass API Client
 * Handles authentication, identity management, wallets, and blockchain interaction
 */
export class E2Client {
  private client: AxiosInstance;
  private token: string | null = null;
  private baseUrl: string;

  constructor(baseUrl?: string) {
    this.baseUrl = baseUrl || process.env.NEXT_PUBLIC_E2_API_URL || 'http://localhost:8080';

    this.client = axios.create({
      baseURL: this.baseUrl,
      headers: {
        'Content-Type': 'application/json',
      },
      timeout: 30000, // 30 second timeout
    });

    // Load token from localStorage if available (client-side only)
    if (typeof window !== 'undefined') {
      this.token = localStorage.getItem('e2_token');
      if (this.token) {
        this.setAuthToken(this.token);
      }
    }

    // Response interceptor for handling auth errors
    this.client.interceptors.response.use(
      (response) => response,
      (error) => {
        if (error.response?.status === 401) {
          // Token expired or invalid
          this.clearAuthToken();
        }
        return Promise.reject(error);
      }
    );
  }

  // ==================
  // Token Management
  // ==================

  private setAuthToken(token: string) {
    this.token = token;
    this.client.defaults.headers.common['Authorization'] = `Bearer ${token}`;

    if (typeof window !== 'undefined') {
      localStorage.setItem('e2_token', token);
    }
  }

  private clearAuthToken() {
    this.token = null;
    delete this.client.defaults.headers.common['Authorization'];

    if (typeof window !== 'undefined') {
      localStorage.removeItem('e2_token');
    }
  }

  // ==================
  // Authentication
  // ==================

  /**
   * Login with email and password
   */
  async login(email: string, password: string): Promise<E2AuthResponse> {
    try {
      const response = await this.client.post<E2AuthResponse>('/api/auth/login', {
        email,
        password,
      });

      this.setAuthToken(response.data.token);
      return response.data;
    } catch (error: any) {
      console.error('Login failed:', error.response?.data || error.message);
      throw new Error(error.response?.data?.error || 'Login failed');
    }
  }

  /**
   * Sign up new user with automatic wallet creation
   */
  async signup(data: E2SignupRequest): Promise<E2AuthResponse> {
    try {
      const response = await this.client.post<E2AuthResponse>('/api/signup', data);

      this.setAuthToken(response.data.token);
      return response.data;
    } catch (error: any) {
      console.error('Signup failed:', error.response?.data || error.message);
      throw new Error(error.response?.data?.error || 'Signup failed');
    }
  }

  /**
   * Logout current user
   */
  async logout() {
    this.clearAuthToken();
  }

  /**
   * Get current authenticated user
   */
  async getCurrentUser(): Promise<E2User> {
    try {
      const response = await this.client.get<E2User>('/api/identity/me');
      return response.data;
    } catch (error: any) {
      console.error('Failed to get current user:', error.response?.data || error.message);
      throw new Error(error.response?.data?.error || 'Failed to get user');
    }
  }

  /**
   * Check if user is authenticated
   */
  isAuthenticated(): boolean {
    return this.token !== null;
  }

  // ==================
  // Identity Management
  // ==================

  /**
   * Get full identity profile
   */
  async getIdentity(): Promise<E2Identity> {
    try {
      const response = await this.client.get<E2Identity>('/api/identity/profile');
      return response.data;
    } catch (error: any) {
      console.error('Failed to get identity:', error.response?.data || error.message);
      throw new Error(error.response?.data?.error || 'Failed to get identity');
    }
  }

  /**
   * Submit KYC attestation
   */
  async submitAttestation(type: string, data: any): Promise<E2Attestation> {
    try {
      const response = await this.client.post<E2Attestation>('/api/identity/attestations', {
        attestation_type: type,
        data,
      });
      return response.data;
    } catch (error: any) {
      console.error('Failed to submit attestation:', error.response?.data || error.message);
      throw new Error(error.response?.data?.error || 'Failed to submit attestation');
    }
  }

  /**
   * Get all attestations for current identity
   */
  async getAttestations(): Promise<E2Attestation[]> {
    try {
      const response = await this.client.get<E2Attestation[]>('/api/identity/attestations');
      return response.data;
    } catch (error: any) {
      console.error('Failed to get attestations:', error.response?.data || error.message);
      throw new Error(error.response?.data?.error || 'Failed to get attestations');
    }
  }

  // ==================
  // Wallet Management
  // ==================

  /**
   * Get all wallets for current user
   * Note: Requires getting current user first to obtain identity_id
   */
  async getWallets(): Promise<E2Wallet[]> {
    try {
      // First get current user to obtain identity_id
      const user = await this.getCurrentUser();
      // Then get wallets for that identity
      const response = await this.client.get<E2Wallet[]>(`/api/wallet/identity/${user.identity_id}`);
      return response.data;
    } catch (error: any) {
      console.error('Failed to get wallets:', error.response?.data || error.message);
      throw new Error(error.response?.data?.error || 'Failed to get wallets');
    }
  }

  /**
   * Create new wallet
   */
  async createWallet(name: string, assetType: string = 'BLS'): Promise<E2Wallet> {
    try {
      // Get current user to obtain identity_id
      const user = await this.getCurrentUser();
      const response = await this.client.post<E2Wallet>('/api/wallet/create', {
        identity_id: user.identity_id,
        labels: [name, assetType], // Server expects labels array
      });
      return response.data;
    } catch (error: any) {
      console.error('Failed to create wallet:', error.response?.data || error.message);
      throw new Error(error.response?.data?.error || 'Failed to create wallet');
    }
  }

  /**
   * Get wallet balance from blockchain
   * Note: Server returns array of balances for different assets
   */
  async getWalletBalance(walletId: string): Promise<E2WalletBalance> {
    try {
      const response = await this.client.get<E2WalletBalance[]>(`/api/wallet/${walletId}/balances`);
      // Return first balance or create empty one
      if (response.data && response.data.length > 0) {
        return response.data[0];
      }
      // Return default balance if none found
      return {
        wallet_id: walletId,
        balance: 0,
        pending_balance: 0,
        last_updated: new Date().toISOString(),
      };
    } catch (error: any) {
      console.error('Failed to get wallet balance:', error.response?.data || error.message);
      throw new Error(error.response?.data?.error || 'Failed to get balance');
    }
  }

  /**
   * Get wallet transactions
   */
  async getWalletTransactions(walletId: string): Promise<E2Transaction[]> {
    try {
      const response = await this.client.get<E2Transaction[]>(`/api/wallet/${walletId}/transactions`);
      return response.data;
    } catch (error: any) {
      console.error('Failed to get transactions:', error.response?.data || error.message);
      throw new Error(error.response?.data?.error || 'Failed to get transactions');
    }
  }

  /**
   * Send transaction from wallet
   */
  async sendTransaction(
    walletId: string,
    toAddress: string,
    amount: number
  ): Promise<E2Transaction> {
    try {
      const response = await this.client.post<E2Transaction>(`/api/wallet/${walletId}/transfer`, {
        to_address: toAddress,
        asset_type: 'BLS', // Default to BLS asset type
        amount,
      });
      return response.data;
    } catch (error: any) {
      console.error('Failed to send transaction:', error.response?.data || error.message);
      throw new Error(error.response?.data?.error || 'Failed to send transaction');
    }
  }

  // ==================
  // Utility Methods
  // ==================

  /**
   * Get current auth token
   */
  getToken(): string | null {
    return this.token;
  }

  /**
   * Get API base URL
   */
  getBaseUrl(): string {
    return this.baseUrl;
  }

  /**
   * Check if E2 API is reachable
   */
  async checkHealth(): Promise<boolean> {
    try {
      const response = await this.client.get('/health');
      return response.status === 200;
    } catch (error) {
      return false;
    }
  }
}

// Singleton instance
export const e2Client = new E2Client();
