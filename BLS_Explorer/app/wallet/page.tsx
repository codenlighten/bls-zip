'use client';

import { useEffect, useState } from 'react';
import Link from 'next/link';
import { AppLayout } from '@/components/layout/app-layout';
import { useRequireAuth } from '@/lib/e2/auth-context';
import { e2Client, E2Wallet, E2WalletBalance, E2Transaction } from '@/lib/e2/client';
import { SendTransactionDialog } from '@/components/wallet/send-transaction-dialog';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Separator } from '@/components/ui/separator';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Wallet, Copy, RefreshCw, Plus, Send, Loader2, ArrowUpRight, ArrowDownRight, Clock } from 'lucide-react';
import { Alert, AlertDescription } from '@/components/ui/alert';

export default function WalletPage() {
  const { user, wallets, refreshWallets } = useRequireAuth();
  const [balances, setBalances] = useState<Map<string, E2WalletBalance>>(new Map());
  const [transactions, setTransactions] = useState<Map<string, E2Transaction[]>>(new Map());
  const [isRefreshing, setIsRefreshing] = useState(false);
  const [error, setError] = useState('');
  const [sendDialogOpen, setSendDialogOpen] = useState(false);
  const [selectedWallet, setSelectedWallet] = useState<E2Wallet | null>(null);
  const [copied, setCopied] = useState<string | null>(null);

  useEffect(() => {
    if (wallets.length > 0) {
      fetchBalances();
    }
  }, [wallets]);

  const fetchBalances = async () => {
    setIsRefreshing(true);
    setError('');

    try {
      const newBalances = new Map<string, E2WalletBalance>();

      for (const wallet of wallets) {
        try {
          const balance = await e2Client.getWalletBalance(wallet.wallet_id);
          newBalances.set(wallet.wallet_id, balance);
        } catch (err) {
          console.error(`Failed to get balance for wallet ${wallet.wallet_id}:`, err);
        }
      }

      setBalances(newBalances);
    } catch (err: any) {
      setError(err.message || 'Failed to fetch balances');
    } finally {
      setIsRefreshing(false);
    }
  };

  const fetchTransactions = async (walletId: string) => {
    try {
      const txs = await e2Client.getWalletTransactions(walletId);
      setTransactions((prev) => new Map(prev).set(walletId, txs));
    } catch (err) {
      console.error(`Failed to get transactions for wallet ${walletId}:`, err);
    }
  };

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text);
    setCopied(text);
    setTimeout(() => setCopied(null), 2000);
  };

  const handleSendClick = (wallet: E2Wallet) => {
    setSelectedWallet(wallet);
    setSendDialogOpen(true);
  };

  const handleSendSuccess = () => {
    // Refresh balances and transactions after successful send
    fetchBalances();
    if (selectedWallet) {
      fetchTransactions(selectedWallet.wallet_id);
    }
  };

  const handleRefresh = async () => {
    await Promise.all([refreshWallets(), fetchBalances()]);
  };

  if (!user) {
    return (
      <AppLayout>
        <div className="flex items-center justify-center min-h-[60vh]">
          <Loader2 className="h-12 w-12 animate-spin" />
        </div>
      </AppLayout>
    );
  }

  return (
    <AppLayout>
      <div className="space-y-6">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-3xl font-bold">My Wallets</h1>
            <p className="text-muted-foreground">
              Manage your quantum-secure BLS wallets
            </p>
          </div>
          <div className="flex gap-2">
            <Button onClick={handleRefresh} variant="outline" disabled={isRefreshing}>
              <RefreshCw className={`mr-2 h-4 w-4 ${isRefreshing ? 'animate-spin' : ''}`} />
              Refresh
            </Button>
            <Button>
              <Plus className="mr-2 h-4 w-4" />
              Create Wallet
            </Button>
          </div>
        </div>

        {error && (
          <Alert variant="destructive">
            <AlertDescription>{error}</AlertDescription>
          </Alert>
        )}

        {wallets.length === 0 ? (
          <Card>
            <CardContent className="py-12 text-center">
              <Wallet className="h-12 w-12 mx-auto mb-4 text-muted-foreground" />
              <h3 className="text-lg font-semibold mb-2">No Wallets Found</h3>
              <p className="text-muted-foreground mb-4">
                Create your first quantum-secure wallet to get started
              </p>
              <Button>
                <Plus className="mr-2 h-4 w-4" />
                Create Wallet
              </Button>
            </CardContent>
          </Card>
        ) : (
          <div className="grid gap-6 md:grid-cols-2">
            {wallets.map((wallet) => {
              const balance = balances.get(wallet.wallet_id);

              return (
                <Card key={wallet.wallet_id} className="overflow-hidden">
                  <CardHeader className="bg-primary/5">
                    <div className="flex items-start justify-between">
                      <div>
                        <CardTitle className="flex items-center gap-2">
                          <Wallet className="h-5 w-5" />
                          {wallet.name}
                        </CardTitle>
                        <CardDescription className="mt-1">
                          {wallet.asset_type} Wallet
                        </CardDescription>
                      </div>
                      <Badge variant="outline">Active</Badge>
                    </div>
                  </CardHeader>

                  <CardContent className="pt-6 space-y-4">
                    {/* Balance */}
                    <div>
                      <p className="text-sm text-muted-foreground mb-1">Balance</p>
                      <div className="flex items-baseline gap-2">
                        <p className="text-3xl font-bold">
                          {balance ? balance.balance.toFixed(4) : wallet.balance.toFixed(4)}
                        </p>
                        <span className="text-lg text-muted-foreground">BLS</span>
                      </div>
                      {balance && balance.pending_balance > 0 && (
                        <p className="text-xs text-muted-foreground mt-1">
                          +{balance.pending_balance.toFixed(4)} BLS pending
                        </p>
                      )}
                    </div>

                    <Separator />

                    {/* Address */}
                    <div>
                      <p className="text-sm text-muted-foreground mb-2">Wallet Address</p>
                      <div className="flex items-center gap-2">
                        <code className="flex-1 text-xs bg-secondary px-3 py-2 rounded">
                          {wallet.address}
                        </code>
                        <Button
                          size="sm"
                          variant="ghost"
                          onClick={() => copyToClipboard(wallet.address)}
                        >
                          {copied === wallet.address ? (
                            <span className="text-xs text-green-500">Copied!</span>
                          ) : (
                            <Copy className="h-4 w-4" />
                          )}
                        </Button>
                      </div>
                    </div>

                    <Separator />

                    {/* Actions */}
                    <div className="flex gap-2">
                      <Button
                        className="flex-1"
                        size="sm"
                        onClick={() => handleSendClick(wallet)}
                      >
                        <Send className="mr-2 h-4 w-4" />
                        Send
                      </Button>
                      <Button
                        className="flex-1"
                        size="sm"
                        variant="outline"
                        onClick={() => fetchTransactions(wallet.wallet_id)}
                      >
                        View Transactions
                      </Button>
                    </div>

                    {/* Transaction History */}
                    {transactions.get(wallet.wallet_id) && (
                      <>
                        <Separator />
                        <div>
                          <p className="text-sm font-medium mb-2">Recent Transactions</p>
                          <div className="space-y-2">
                            {transactions.get(wallet.wallet_id)!.slice(0, 3).map((tx) => (
                              <div key={tx.tx_id} className="flex items-center gap-2 p-2 bg-secondary rounded text-xs">
                                {tx.amount > 0 ? (
                                  <ArrowDownRight className="h-4 w-4 text-green-500" />
                                ) : (
                                  <ArrowUpRight className="h-4 w-4 text-red-500" />
                                )}
                                <div className="flex-1">
                                  <p className="font-medium">{Math.abs(tx.amount).toFixed(4)} BLS</p>
                                  <p className="text-muted-foreground">
                                    {new Date(tx.created_at).toLocaleDateString()}
                                  </p>
                                </div>
                                <Badge variant={tx.status === 'confirmed' ? 'default' : 'outline'}>
                                  {tx.status}
                                </Badge>
                              </div>
                            ))}
                          </div>
                        </div>
                      </>
                    )}

                    {/* Post-Quantum Badge */}
                    <div className="pt-2">
                      <div className="inline-flex items-center gap-2 px-3 py-1.5 bg-primary/10 rounded-full text-xs">
                        <div className="h-2 w-2 bg-green-500 rounded-full" />
                        <span>Post-Quantum Secure (Dilithium5)</span>
                      </div>
                    </div>
                  </CardContent>
                </Card>
              );
            })}
          </div>
        )}

        {/* Info Card */}
        <Card>
          <CardHeader>
            <CardTitle className="text-lg">About Your Wallets</CardTitle>
          </CardHeader>
          <CardContent className="space-y-3 text-sm">
            <p>
              Your wallets are protected by <strong>Dilithium5</strong>, a post-quantum
              cryptographic algorithm that is secure against attacks from quantum computers.
            </p>
            <p>
              All private keys are encrypted and stored securely in the E² Multipass system.
              You can use these wallets to send and receive BLS tokens on the Boundless blockchain.
            </p>
          </CardContent>
        </Card>
      </div>

      {/* Send Transaction Dialog */}
      {selectedWallet && (
        <SendTransactionDialog
          wallet={selectedWallet}
          open={sendDialogOpen}
          onOpenChange={setSendDialogOpen}
          onSuccess={handleSendSuccess}
        />
      )}
    </AppLayout>
  );
}
