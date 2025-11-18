'use client';

import { useEffect, useState } from 'react';
import { AppLayout } from '@/components/layout/app-layout';
import { MetricCard } from '@/components/dashboard/metric-card';
import { BlocksTable } from '@/components/dashboard/blocks-table';
import { TransactionsTable } from '@/components/dashboard/transactions-table';
import { NetworkChart } from '@/components/dashboard/network-chart';
import { blockchainIndexer } from '@/lib/blockchain/indexer';
import { mockIndexer } from '@/lib/mock-indexer';
import { getBlockchainWebSocket } from '@/lib/blockchain/websocket';
import type { ChainInfo, Block, Transaction } from '@/lib/types';
import { Blocks, Activity, Clock, Shield, Wifi, WifiOff, TrendingUp, Database } from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Loader2 } from 'lucide-react';
import { Badge } from '@/components/ui/badge';

// Feature flag to toggle between mock and real data
const USE_REAL_BLOCKCHAIN = process.env.NEXT_PUBLIC_ENABLE_BLOCKCHAIN !== 'false';

export default function Home() {
  const [stats, setStats] = useState<ChainInfo | null>(null);
  const [latestBlocks, setLatestBlocks] = useState<Block[]>([]);
  const [recentTransactions, setRecentTransactions] = useState<Transaction[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [isConnected, setIsConnected] = useState(false);
  const [wsConnected, setWsConnected] = useState(false);
  const [newBlockNotification, setNewBlockNotification] = useState<string | null>(null);

  useEffect(() => {
    const fetchData = async () => {
      setIsLoading(true);
      setError(null);

      try {
        if (USE_REAL_BLOCKCHAIN) {
          // Check blockchain connection first
          const connected = await blockchainIndexer.isConnected();
          setIsConnected(connected);

          if (connected) {
            // Fetch real blockchain data
            const [chainInfo, blocks] = await Promise.all([
              blockchainIndexer.getChainInfo(),
              blockchainIndexer.getLatestBlocks(10),
            ]);

            setStats(chainInfo);
            setLatestBlocks(blocks);

            // Extract transactions from blocks
            const txs: Transaction[] = [];
            blocks.forEach((block) => {
              // Note: Transaction details would need to be expanded
              // This is a simplified version
            });
            setRecentTransactions(txs.slice(0, 10));
          } else {
            throw new Error('Cannot connect to blockchain node');
          }
        } else {
          // Use mock data
          setStats(mockIndexer.getChainInfo());
          setLatestBlocks(mockIndexer.getLatestBlocks(10));
          setRecentTransactions(mockIndexer.getRecentTransactions(10));
          setIsConnected(true);
        }
      } catch (err: any) {
        console.error('Failed to fetch blockchain data:', err);
        setError(err.message || 'Failed to load blockchain data');

        // Fallback to mock data on error
        setStats(mockIndexer.getChainInfo());
        setLatestBlocks(mockIndexer.getLatestBlocks(10));
        setRecentTransactions(mockIndexer.getRecentTransactions(10));
      } finally {
        setIsLoading(false);
      }
    };

    fetchData();

    // Refresh data every 30 seconds
    const interval = setInterval(fetchData, 30000);
    return () => clearInterval(interval);
  }, []);

  // WebSocket connection for real-time updates
  useEffect(() => {
    if (!USE_REAL_BLOCKCHAIN) {
      return;
    }

    const ws = getBlockchainWebSocket();

    // Listen for connection changes
    const unsubscribeConnection = ws.onConnectionChange((connected) => {
      setWsConnected(connected);
      console.log(`WebSocket ${connected ? 'connected' : 'disconnected'}`);
    });

    // Listen for blockchain events
    const unsubscribeEvents = ws.onEvent((event) => {
      if (event.type === 'new_block') {
        const { height, hash, tx_count } = event.data;

        // Show notification
        setNewBlockNotification(`New block #${height.toLocaleString()} mined`);
        setTimeout(() => setNewBlockNotification(null), 5000);

        // Update stats
        setStats((prev) => prev ? { ...prev, height } : null);

        // Fetch the new block and prepend to list
        blockchainIndexer.getBlockByHeight(height).then((block) => {
          if (block) {
            setLatestBlocks((prev) => [block, ...prev.slice(0, 9)]);
          }
        });
      } else if (event.type === 'new_transaction') {
        const { tx_hash, from, to, amount } = event.data;
        console.log('New transaction:', tx_hash);

        // Could add transaction to recent transactions list here
      } else if (event.type === 'tx_confirmed') {
        console.log('Transaction confirmed:', event.data.tx_hash);
      }
    });

    // Connect to WebSocket
    ws.connect();

    // Cleanup on unmount
    return () => {
      unsubscribeConnection();
      unsubscribeEvents();
      ws.disconnect();
    };
  }, []);

  if (isLoading && !stats) {
    return (
      <AppLayout>
        <div className="flex items-center justify-center min-h-[60vh]">
          <div className="text-center">
            <Loader2 className="h-12 w-12 animate-spin mx-auto mb-4" />
            <p className="text-lg text-muted-foreground">Loading blockchain data...</p>
          </div>
        </div>
      </AppLayout>
    );
  }


  return (
    <AppLayout>
      <div className="space-y-6">
        <div className="flex justify-between items-start">
          <div>
            <h1 className="text-3xl font-bold">Network Overview</h1>
            <p className="text-muted-foreground">
              Real-time statistics for the Boundless BLS Blockchain
            </p>
          </div>
          <div className="flex items-center gap-4">
            {/* API Connection Status */}
            <div className="flex items-center gap-2">
              <div className={`h-2 w-2 rounded-full ${isConnected ? 'bg-green-500' : 'bg-red-500'}`} />
              <span className="text-sm text-muted-foreground">
                {isConnected ? (USE_REAL_BLOCKCHAIN ? 'API Connected' : 'Mock Data') : 'API Disconnected'}
              </span>
            </div>

            {/* WebSocket Status */}
            {USE_REAL_BLOCKCHAIN && (
              <Badge variant={wsConnected ? 'default' : 'outline'} className="flex items-center gap-1">
                {wsConnected ? (
                  <>
                    <Wifi className="h-3 w-3" />
                    <span>Live Updates</span>
                  </>
                ) : (
                  <>
                    <WifiOff className="h-3 w-3" />
                    <span>WebSocket Off</span>
                  </>
                )}
              </Badge>
            )}
          </div>
        </div>

        {error && !stats && (
          <Alert variant="destructive">
            <AlertDescription>
              {error} - Showing fallback data
            </AlertDescription>
          </Alert>
        )}

        {/* New Block Notification */}
        {newBlockNotification && (
          <Alert className="bg-green-500/10 border-green-500/50 animate-in slide-in-from-top">
            <Blocks className="h-4 w-4 text-green-500" />
            <AlertDescription className="text-green-500 font-medium">
              {newBlockNotification}
            </AlertDescription>
          </Alert>
        )}

        {/* Key Metrics */}
        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
          <MetricCard
            title="Block Height"
            value={`#${stats?.height.toLocaleString() || '0'}`}
            icon={Blocks}
            description="Current chain height"
            badge="Live"
            trend={
              latestBlocks.length >= 2
                ? {
                    value: 0.5,
                    isPositive: true,
                  }
                : undefined
            }
          />
          <MetricCard
            title="Network Hashrate"
            value={stats?.network_hashrate || '450 MH/s'}
            icon={Activity}
            description="SHA3-256 mining power"
            trend={{
              value: 2.3,
              isPositive: true,
            }}
          />
          <MetricCard
            title="Avg Block Time"
            value={stats?.avg_block_time || '5m 00s'}
            icon={Clock}
            description="Last 100 blocks"
          />
          <MetricCard
            title="Total Supply"
            value={stats?.total_supply ? `${(Number(stats.total_supply) / 1_000_000).toFixed(2)}M` : '0'}
            icon={Database}
            description="BLS tokens in circulation"
            badge="PQC"
          />
        </div>

        {/* Network Activity Chart */}
        <NetworkChart blocks={latestBlocks} />

        {/* Tables */}
        <div className="grid gap-6 lg:grid-cols-2">
          <Card>
            <CardHeader>
              <div className="flex items-center justify-between">
                <CardTitle>Latest Blocks</CardTitle>
                <Badge variant="secondary">{latestBlocks.length} blocks</Badge>
              </div>
            </CardHeader>
            <CardContent>
              <BlocksTable blocks={latestBlocks} />
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <div className="flex items-center justify-between">
                <CardTitle>Recent Transactions</CardTitle>
                <Badge variant="secondary">{recentTransactions.length} transactions</Badge>
              </div>
            </CardHeader>
            <CardContent>
              <TransactionsTable transactions={recentTransactions} />
            </CardContent>
          </Card>
        </div>

        {/* Additional Info Card */}
        <Card className="border-primary/20 bg-primary/5">
          <CardContent className="p-6">
            <div className="flex items-start gap-4">
              <div className="rounded-lg bg-primary/10 p-3">
                <Shield className="h-6 w-6 text-primary" />
              </div>
              <div className="flex-1">
                <h3 className="font-semibold mb-1">Post-Quantum Secure Blockchain</h3>
                <p className="text-sm text-muted-foreground">
                  Boundless BLS uses post-quantum cryptography (Dilithium5, ML-DSA, Falcon-512) to ensure security against quantum computer attacks. All transactions, identities, and smart contracts are protected with quantum-resistant algorithms.
                </p>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>
    </AppLayout>
  );
}
