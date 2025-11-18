'use client';

/**
 * WebSocket client for real-time blockchain updates
 * Connects to blockchain node WebSocket endpoint for live events
 */

export type BlockchainEvent =
  | { type: 'new_block'; data: NewBlockEvent }
  | { type: 'new_transaction'; data: NewTransactionEvent }
  | { type: 'tx_confirmed'; data: TransactionConfirmedEvent }
  | { type: 'chain_reorg'; data: ChainReorgEvent };

export interface NewBlockEvent {
  height: number;
  hash: string;
  timestamp: string;
  tx_count: number;
  miner: string;
}

export interface NewTransactionEvent {
  tx_hash: string;
  from: string;
  to: string;
  amount: number;
  fee: number;
  timestamp: string;
}

export interface TransactionConfirmedEvent {
  tx_hash: string;
  block_height: number;
  block_hash: string;
  confirmations: number;
}

export interface ChainReorgEvent {
  old_height: number;
  new_height: number;
  old_hash: string;
  new_hash: string;
}

type EventCallback = (event: BlockchainEvent) => void;
type ConnectionCallback = (connected: boolean) => void;

export class BlockchainWebSocket {
  private ws: WebSocket | null = null;
  private wsUrl: string;
  private eventCallbacks: Set<EventCallback> = new Set();
  private connectionCallbacks: Set<ConnectionCallback> = new Set();
  private reconnectAttempts = 0;
  private maxReconnectAttempts = 5;
  private reconnectDelay = 1000; // Start with 1 second
  private isIntentionallyClosed = false;

  constructor(wsUrl?: string) {
    // WebSocket typically uses same host as RPC but with /ws endpoint
    const rpcUrl = process.env.NEXT_PUBLIC_BLOCKCHAIN_RPC_URL || 'http://localhost:9933';
    const wsProtocol = rpcUrl.startsWith('https') ? 'wss' : 'ws';
    const host = rpcUrl.replace(/^https?:\/\//, '').replace(/:\d+$/, '');
    const port = rpcUrl.match(/:(\d+)/)?.[1] || '9933';

    this.wsUrl = wsUrl || `${wsProtocol}://${host}:${port}/ws`;
  }

  /**
   * Connect to the WebSocket endpoint
   */
  connect(): void {
    if (this.ws?.readyState === WebSocket.OPEN) {
      console.log('WebSocket already connected');
      return;
    }

    this.isIntentionallyClosed = false;

    try {
      console.log(`Connecting to WebSocket: ${this.wsUrl}`);
      this.ws = new WebSocket(this.wsUrl);

      this.ws.onopen = () => {
        console.log('WebSocket connected');
        this.reconnectAttempts = 0;
        this.reconnectDelay = 1000;
        this.notifyConnectionChange(true);

        // Subscribe to all events
        this.subscribe('new_blocks');
        this.subscribe('new_transactions');
        this.subscribe('tx_confirmations');
      };

      this.ws.onmessage = (event) => {
        try {
          const message = JSON.parse(event.data);
          this.handleMessage(message);
        } catch (err) {
          console.error('Failed to parse WebSocket message:', err);
        }
      };

      this.ws.onerror = (error) => {
        console.error('WebSocket error:', error);
      };

      this.ws.onclose = () => {
        console.log('WebSocket disconnected');
        this.notifyConnectionChange(false);

        if (!this.isIntentionallyClosed) {
          this.attemptReconnect();
        }
      };
    } catch (err) {
      console.error('Failed to create WebSocket connection:', err);
      this.attemptReconnect();
    }
  }

  /**
   * Disconnect from WebSocket
   */
  disconnect(): void {
    this.isIntentionallyClosed = true;
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
  }

  /**
   * Subscribe to a specific event type
   */
  private subscribe(eventType: string): void {
    if (this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify({
        jsonrpc: '2.0',
        method: 'subscribe',
        params: [eventType],
        id: Date.now(),
      }));
    }
  }

  /**
   * Handle incoming WebSocket message
   */
  private handleMessage(message: any): void {
    // Handle subscription confirmations
    if (message.result && message.id) {
      console.log('Subscription confirmed:', message.result);
      return;
    }

    // Handle event notifications
    if (message.method === 'subscription' && message.params) {
      const { subscription, result } = message.params;

      // Parse the event based on subscription type
      let event: BlockchainEvent | null = null;

      if (result.type === 'new_block') {
        event = {
          type: 'new_block',
          data: {
            height: result.height,
            hash: result.hash,
            timestamp: result.timestamp,
            tx_count: result.tx_count || 0,
            miner: result.miner || 'unknown',
          },
        };
      } else if (result.type === 'new_transaction') {
        event = {
          type: 'new_transaction',
          data: {
            tx_hash: result.tx_hash,
            from: result.from,
            to: result.to,
            amount: result.amount,
            fee: result.fee,
            timestamp: result.timestamp,
          },
        };
      } else if (result.type === 'tx_confirmed') {
        event = {
          type: 'tx_confirmed',
          data: {
            tx_hash: result.tx_hash,
            block_height: result.block_height,
            block_hash: result.block_hash,
            confirmations: result.confirmations,
          },
        };
      } else if (result.type === 'chain_reorg') {
        event = {
          type: 'chain_reorg',
          data: {
            old_height: result.old_height,
            new_height: result.new_height,
            old_hash: result.old_hash,
            new_hash: result.new_hash,
          },
        };
      }

      if (event) {
        this.notifyEventCallbacks(event);
      }
    }
  }

  /**
   * Attempt to reconnect after disconnect
   */
  private attemptReconnect(): void {
    if (this.reconnectAttempts >= this.maxReconnectAttempts) {
      console.error('Max reconnection attempts reached');
      return;
    }

    this.reconnectAttempts++;
    const delay = this.reconnectDelay * Math.pow(2, this.reconnectAttempts - 1); // Exponential backoff

    console.log(`Attempting to reconnect in ${delay}ms (attempt ${this.reconnectAttempts}/${this.maxReconnectAttempts})`);

    setTimeout(() => {
      if (!this.isIntentionallyClosed) {
        this.connect();
      }
    }, delay);
  }

  /**
   * Register a callback for blockchain events
   */
  onEvent(callback: EventCallback): () => void {
    this.eventCallbacks.add(callback);

    // Return unsubscribe function
    return () => {
      this.eventCallbacks.delete(callback);
    };
  }

  /**
   * Register a callback for connection status changes
   */
  onConnectionChange(callback: ConnectionCallback): () => void {
    this.connectionCallbacks.add(callback);

    // Return unsubscribe function
    return () => {
      this.connectionCallbacks.delete(callback);
    };
  }

  /**
   * Notify all event callbacks
   */
  private notifyEventCallbacks(event: BlockchainEvent): void {
    this.eventCallbacks.forEach((callback) => {
      try {
        callback(event);
      } catch (err) {
        console.error('Error in event callback:', err);
      }
    });
  }

  /**
   * Notify all connection callbacks
   */
  private notifyConnectionChange(connected: boolean): void {
    this.connectionCallbacks.forEach((callback) => {
      try {
        callback(connected);
      } catch (err) {
        console.error('Error in connection callback:', err);
      }
    });
  }

  /**
   * Get current connection state
   */
  isConnected(): boolean {
    return this.ws?.readyState === WebSocket.OPEN;
  }
}

// Singleton instance
let blockchainWS: BlockchainWebSocket | null = null;

/**
 * Get or create WebSocket client instance
 */
export function getBlockchainWebSocket(): BlockchainWebSocket {
  if (typeof window === 'undefined') {
    // Return mock for SSR
    return {
      connect: () => {},
      disconnect: () => {},
      onEvent: () => () => {},
      onConnectionChange: () => () => {},
      isConnected: () => false,
    } as any;
  }

  if (!blockchainWS) {
    blockchainWS = new BlockchainWebSocket();
  }

  return blockchainWS;
}
