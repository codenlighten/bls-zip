'use client';

import { use, useEffect, useState } from 'react';
import { AppLayout } from '@/components/layout/app-layout';
import { UtxoVisualizer } from '@/components/transaction/utxo-visualizer';
import { SignatureBadge } from '@/components/transaction/signature-badge';
import { ProofCard } from '@/components/transaction/proof-card';
import { AssetTransferTicket } from '@/components/transaction/asset-transfer-ticket';
import { ContractCallCard } from '@/components/transaction/contract-call-card';
import { blockchainIndexer } from '@/lib/blockchain/indexer';
import { mockIndexer } from '@/lib/mock-indexer';
import { decodeAssetTransfer, decodeContractCall } from '@/lib/abi-decoder';
import type { Transaction } from '@/lib/types';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Separator } from '@/components/ui/separator';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Alert, AlertDescription } from '@/components/ui/alert';
import Link from 'next/link';
import { ArrowLeft, Loader2 } from 'lucide-react';
import { Button } from '@/components/ui/button';

interface PageProps {
  params: Promise<{ hash: string }>;
}

const USE_REAL_BLOCKCHAIN = process.env.NEXT_PUBLIC_ENABLE_BLOCKCHAIN !== 'false';

function formatTimestamp(timestamp: number): string {
  return new Date(timestamp).toLocaleString();
}

function getScriptTypeBadge(scriptType: string) {
  const variants: Record<string, 'default' | 'secondary' | 'outline'> = {
    p2pkh: 'secondary',
    proof_anchor: 'default',
    contract_deploy: 'outline',
  };

  return <Badge variant={variants[scriptType] || 'secondary'}>{scriptType.replace('_', ' ')}</Badge>;
}

export default function TransactionPage({ params }: PageProps) {
  const { hash } = use(params);
  const [transaction, setTransaction] = useState<Transaction | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchTransaction = async () => {
      setIsLoading(true);
      setError(null);

      try {
        let fetchedTx: Transaction | null = null;

        if (USE_REAL_BLOCKCHAIN) {
          // Try to fetch from real blockchain
          const connected = await blockchainIndexer.isConnected();

          if (connected) {
            fetchedTx = await blockchainIndexer.getTransaction(hash);
          } else {
            throw new Error('Cannot connect to blockchain node');
          }
        } else {
          // Use mock data
          fetchedTx = mockIndexer.getTransaction(hash) || null;
        }

        if (!fetchedTx) {
          throw new Error('Transaction not found');
        }

        setTransaction(fetchedTx);
      } catch (err: any) {
        console.error('Failed to fetch transaction:', err);
        setError(err.message || 'Failed to load transaction');

        // Fallback to mock data
        const mockTx = mockIndexer.getTransaction(hash) || null;
        if (mockTx) {
          setTransaction(mockTx);
        }
      } finally {
        setIsLoading(false);
      }
    };

    fetchTransaction();
  }, [hash]);

  if (isLoading) {
    return (
      <AppLayout>
        <div className="flex items-center justify-center min-h-[60vh]">
          <div className="text-center">
            <Loader2 className="h-12 w-12 animate-spin mx-auto mb-4" />
            <p className="text-lg text-muted-foreground">Loading transaction...</p>
          </div>
        </div>
      </AppLayout>
    );
  }

  if (!transaction) {
    return (
      <AppLayout>
        <div className="flex flex-col items-center justify-center py-20">
          <h1 className="text-2xl font-bold">Transaction Not Found</h1>
          <p className="mt-2 text-muted-foreground">
            The transaction with hash {hash} could not be found.
          </p>
          {error && (
            <Alert variant="destructive" className="mt-4 max-w-md">
              <AlertDescription>{error}</AlertDescription>
            </Alert>
          )}
          <Link href="/">
            <Button className="mt-4">Return to Dashboard</Button>
          </Link>
        </div>
      </AppLayout>
    );
  }

  const isProofAnchor = !!transaction.proof_data;
  const assetTransfer = transaction.data ? decodeAssetTransfer(transaction.data) : null;
  const contractCall = transaction.data && !assetTransfer ? decodeContractCall(transaction.data) : null;
  const showUtxoVisualizer = !assetTransfer;

  return (
    <AppLayout>
      <div className="space-y-6">
        <div className="flex items-center space-x-4">
          <Link href="/">
            <Button variant="ghost" size="sm">
              <ArrowLeft className="mr-2 h-4 w-4" />
              Back
            </Button>
          </Link>
          <div>
            <h1 className="text-3xl font-bold">Transaction Details</h1>
            <p className="text-sm text-muted-foreground">
              Hash: <span className="font-mono">{hash}</span>
            </p>
          </div>
        </div>

        {assetTransfer ? (
          <AssetTransferTicket transfer={assetTransfer} txHash={hash} />
        ) : isProofAnchor && transaction.proof_data ? (
          <ProofCard
            identityId={transaction.proof_data.identity_id}
            proofType={transaction.proof_data.proof_type}
            proofHash={transaction.proof_data.proof_hash}
          />
        ) : (
          <Card>
            <CardHeader>
              <CardTitle>Overview</CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="grid gap-4 md:grid-cols-2">
                <div>
                  <p className="text-sm text-muted-foreground">Transaction Hash</p>
                  <p className="font-mono text-sm">{transaction.tx_hash}</p>
                </div>
                <div>
                  <p className="text-sm text-muted-foreground">Script Type</p>
                  <div className="mt-1">{getScriptTypeBadge(transaction.outputs[0]?.script_type || 'p2pkh')}</div>
                </div>
                <div>
                  <p className="text-sm text-muted-foreground">Version</p>
                  <p className="font-mono text-sm">{transaction.version}</p>
                </div>
                <div>
                  <p className="text-sm text-muted-foreground">Timestamp</p>
                  <p className="text-sm">{formatTimestamp(transaction.timestamp)}</p>
                </div>
              </div>
            </CardContent>
          </Card>
        )}

        {showUtxoVisualizer && (
          <div>
            <h2 className="mb-3 text-xl font-bold">UTXO Flow</h2>
            <UtxoVisualizer transaction={transaction} />
          </div>
        )}

        <Card>
          <CardHeader>
            <CardTitle>Post-Quantum Signatures</CardTitle>
          </CardHeader>
          <CardContent className="space-y-3">
            {transaction.inputs.map((input, index) => (
              <div key={index}>
                <p className="mb-2 text-xs text-muted-foreground">
                  Input #{index} Signature
                </p>
                <SignatureBadge
                  signatureType={input.signature_type}
                  sizeBytes={input.signature_size_bytes}
                />
              </div>
            ))}
          </CardContent>
        </Card>

        {contractCall && (
          <ContractCallCard contractCall={contractCall} />
        )}

        {transaction.contract_data && !contractCall && (
          <Card>
            <CardHeader>
              <CardTitle>Contract Execution</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="space-y-3">
                <div>
                  <p className="text-sm text-muted-foreground">Function</p>
                  <p className="font-mono text-sm font-semibold">{transaction.contract_data.function}</p>
                </div>
                <Separator />
                <div>
                  <p className="text-sm text-muted-foreground">Arguments</p>
                  <pre className="mt-2 rounded-sm bg-secondary p-3 text-xs">
                    {JSON.stringify(transaction.contract_data.args, null, 2)}
                  </pre>
                </div>
              </div>
            </CardContent>
          </Card>
        )}

        <Tabs defaultValue="details">
          <TabsList>
            <TabsTrigger value="details">Technical Details</TabsTrigger>
            <TabsTrigger value="raw">Raw Data</TabsTrigger>
          </TabsList>
          <TabsContent value="details" className="mt-4">
            <Card>
              <CardContent className="pt-6">
                <div className="space-y-3">
                  <div className="flex justify-between">
                    <span className="text-sm text-muted-foreground">Version</span>
                    <span className="font-mono text-sm">{transaction.version}</span>
                  </div>
                  <Separator />
                  <div className="flex justify-between">
                    <span className="text-sm text-muted-foreground">Input Count</span>
                    <span className="font-mono text-sm">{transaction.inputs.length}</span>
                  </div>
                  <Separator />
                  <div className="flex justify-between">
                    <span className="text-sm text-muted-foreground">Output Count</span>
                    <span className="font-mono text-sm">{transaction.outputs.length}</span>
                  </div>
                  <Separator />
                  <div className="flex justify-between">
                    <span className="text-sm text-muted-foreground">Timestamp</span>
                    <span className="text-sm">{formatTimestamp(transaction.timestamp)}</span>
                  </div>
                  {transaction.data && (
                    <>
                      <Separator />
                      <div className="flex justify-between">
                        <span className="text-sm text-muted-foreground">Data Field Present</span>
                        <Badge variant="outline">Yes</Badge>
                      </div>
                    </>
                  )}
                </div>
              </CardContent>
            </Card>
          </TabsContent>
          <TabsContent value="raw" className="mt-4">
            <Card>
              <CardContent className="pt-6">
                <pre className="overflow-auto rounded-sm bg-secondary p-4 text-xs">
                  {JSON.stringify(transaction, null, 2)}
                </pre>
              </CardContent>
            </Card>
          </TabsContent>
        </Tabs>
      </div>
    </AppLayout>
  );
}
