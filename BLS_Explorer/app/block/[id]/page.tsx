'use client';

import { use, useEffect, useState } from 'react';
import Link from 'next/link';
import { AppLayout } from '@/components/layout/app-layout';
import { blockchainIndexer } from '@/lib/blockchain/indexer';
import { mockIndexer } from '@/lib/mock-indexer';
import type { Block } from '@/lib/types';
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Separator } from '@/components/ui/separator';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { CopyButton } from '@/components/ui/copy-button';
import { ArrowLeft, ArrowRight, Loader2, Clock, Hash, Box, User, FileText } from 'lucide-react';
import { timeAgo, formatTimestamp, formatBytes, truncateHash } from '@/lib/utils/time';

interface PageProps {
  params: Promise<{ id: string }>;
}

const USE_REAL_BLOCKCHAIN = process.env.NEXT_PUBLIC_ENABLE_BLOCKCHAIN !== 'false';

function getScriptTypeBadge(scriptType: string) {
  const variants: Record<string, 'default' | 'secondary' | 'outline'> = {
    p2pkh: 'secondary',
    proof_anchor: 'default',
    contract_deploy: 'outline',
  };

  return <Badge variant={variants[scriptType] || 'secondary'}>{scriptType.replace('_', ' ')}</Badge>;
}

function calculateFee(tx: any): number {
  const totalOutput = tx.outputs.reduce((sum: number, o: any) => sum + o.amount, 0);
  return totalOutput * 0.01;
}

export default function BlockPage({ params }: PageProps) {
  const { id } = use(params);
  const [block, setBlock] = useState<Block | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchBlock = async () => {
      setIsLoading(true);
      setError(null);

      try {
        const blockHeight = Number(id);

        if (isNaN(blockHeight)) {
          throw new Error('Invalid block height');
        }

        let fetchedBlock: Block | null = null;

        if (USE_REAL_BLOCKCHAIN) {
          // Try to fetch from real blockchain
          const connected = await blockchainIndexer.isConnected();

          if (connected) {
            fetchedBlock = await blockchainIndexer.getBlockByHeight(blockHeight);
          } else {
            throw new Error('Cannot connect to blockchain node');
          }
        } else {
          // Use mock data
          fetchedBlock = mockIndexer.getBlockByHeight(blockHeight) || null;
        }

        if (!fetchedBlock) {
          throw new Error('Block not found');
        }

        setBlock(fetchedBlock);
      } catch (err: any) {
        console.error('Failed to fetch block:', err);
        setError(err.message || 'Failed to load block');

        // Fallback to mock data
        const mockBlock = mockIndexer.getBlockByHeight(Number(id)) || null;
        if (mockBlock) {
          setBlock(mockBlock);
        }
      } finally {
        setIsLoading(false);
      }
    };

    fetchBlock();
  }, [id]);

  if (isLoading) {
    return (
      <AppLayout>
        <div className="flex items-center justify-center min-h-[60vh]">
          <div className="text-center">
            <Loader2 className="h-12 w-12 animate-spin mx-auto mb-4" />
            <p className="text-lg text-muted-foreground">Loading block #{id}...</p>
          </div>
        </div>
      </AppLayout>
    );
  }

  if (!block) {
    return (
      <AppLayout>
        <div className="flex flex-col items-center justify-center py-20">
          <h1 className="text-2xl font-bold">Block Not Found</h1>
          <p className="mt-2 text-muted-foreground">
            The block with ID {id} could not be found.
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

  const transactions = block?.transactions || [];

  return (
    <AppLayout>
      <div className="space-y-6">
        {/* Header with Navigation */}
        <div className="flex items-center justify-between">
          <div className="flex items-center space-x-4">
            <Link href="/">
              <Button variant="ghost" size="sm">
                <ArrowLeft className="mr-2 h-4 w-4" />
                Dashboard
              </Button>
            </Link>
            <div>
              <h1 className="text-3xl font-bold">Block #{block.height.toLocaleString()}</h1>
              <div className="flex items-center gap-2 mt-1">
                <p className="font-mono text-sm text-muted-foreground">{truncateHash(block.hash, 12, 12)}</p>
                <CopyButton text={block.hash} />
              </div>
            </div>
          </div>

          {/* Previous/Next Navigation */}
          <div className="flex items-center gap-2">
            {block.height > 0 && (
              <Link href={`/block/${block.height - 1}`}>
                <Button variant="outline" size="sm">
                  <ArrowLeft className="mr-2 h-4 w-4" />
                  Previous
                </Button>
              </Link>
            )}
            <Link href={`/block/${block.height + 1}`}>
              <Button variant="outline" size="sm">
                Next
                <ArrowRight className="ml-2 h-4 w-4" />
              </Button>
            </Link>
          </div>
        </div>

        {/* Quick Stats */}
        <div className="grid gap-4 md:grid-cols-4">
          <Card>
            <CardContent className="p-4">
              <div className="flex items-center gap-3">
                <div className="rounded-lg bg-primary/10 p-2">
                  <Clock className="h-5 w-5 text-primary" />
                </div>
                <div>
                  <p className="text-xs text-muted-foreground">Timestamp</p>
                  <p className="text-sm font-medium">{timeAgo(block.timestamp)}</p>
                </div>
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardContent className="p-4">
              <div className="flex items-center gap-3">
                <div className="rounded-lg bg-primary/10 p-2">
                  <FileText className="h-5 w-5 text-primary" />
                </div>
                <div>
                  <p className="text-xs text-muted-foreground">Transactions</p>
                  <p className="text-sm font-medium">{block.tx_count}</p>
                </div>
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardContent className="p-4">
              <div className="flex items-center gap-3">
                <div className="rounded-lg bg-primary/10 p-2">
                  <Box className="h-5 w-5 text-primary" />
                </div>
                <div>
                  <p className="text-xs text-muted-foreground">Size</p>
                  <p className="text-sm font-medium">{formatBytes(block.size || 0)}</p>
                </div>
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardContent className="p-4">
              <div className="flex items-center gap-3">
                <div className="rounded-lg bg-primary/10 p-2">
                  <Hash className="h-5 w-5 text-primary" />
                </div>
                <div>
                  <p className="text-xs text-muted-foreground">Nonce</p>
                  <p className="text-sm font-medium font-mono">{block.nonce}</p>
                </div>
              </div>
            </CardContent>
          </Card>
        </div>

        {/* Block Information */}
        <Card>
          <CardHeader>
            <CardTitle>Block Information</CardTitle>
            <CardDescription>Detailed information about this block</CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="grid gap-4 md:grid-cols-2">
              <div className="space-y-3">
                <div>
                  <p className="text-sm text-muted-foreground mb-1">Block Height</p>
                  <p className="font-mono text-sm font-semibold">#{block.height.toLocaleString()}</p>
                </div>

                <div>
                  <p className="text-sm text-muted-foreground mb-1">Block Hash</p>
                  <div className="flex items-center gap-2">
                    <code className="text-xs bg-secondary px-2 py-1 rounded flex-1 overflow-hidden">
                      {block.hash}
                    </code>
                    <CopyButton text={block.hash} />
                  </div>
                </div>

                <div>
                  <p className="text-sm text-muted-foreground mb-1">Parent Hash</p>
                  {block.height > 0 ? (
                    <div className="flex items-center gap-2">
                      <Link
                        href={`/block/${block.height - 1}`}
                        className="font-mono text-xs text-primary hover:underline bg-secondary px-2 py-1 rounded flex-1"
                      >
                        {block.prev_hash}
                      </Link>
                      <CopyButton text={block.prev_hash} />
                    </div>
                  ) : (
                    <code className="text-xs bg-secondary px-2 py-1 rounded block">{block.prev_hash}</code>
                  )}
                </div>

                <div>
                  <p className="text-sm text-muted-foreground mb-1">Merkle Root</p>
                  <div className="flex items-center gap-2">
                    <code className="text-xs bg-secondary px-2 py-1 rounded flex-1 overflow-hidden">
                      {block.merkle_root}
                    </code>
                    <CopyButton text={block.merkle_root} />
                  </div>
                </div>
              </div>

              <div className="space-y-3">
                <div>
                  <p className="text-sm text-muted-foreground mb-1">Timestamp</p>
                  <div className="space-y-1">
                    <p className="text-sm">{formatTimestamp(block.timestamp)}</p>
                    <p className="text-xs text-muted-foreground">{timeAgo(block.timestamp)}</p>
                  </div>
                </div>

                <div>
                  <p className="text-sm text-muted-foreground mb-1">Miner Address</p>
                  <div className="flex items-center gap-2">
                    <code className="text-xs bg-secondary px-2 py-1 rounded flex-1 overflow-hidden">
                      {block.miner}
                    </code>
                    <CopyButton text={block.miner} />
                  </div>
                </div>

                <div>
                  <p className="text-sm text-muted-foreground mb-1">Difficulty Target</p>
                  <p className="font-mono text-sm">{block.difficulty_target}</p>
                </div>

                <div>
                  <p className="text-sm text-muted-foreground mb-1">Version</p>
                  <Badge variant="secondary">v{block.version}</Badge>
                </div>
              </div>
            </div>
          </CardContent>
        </Card>

        <Tabs defaultValue="transactions">
          <TabsList>
            <TabsTrigger value="transactions">
              Transactions ({transactions.length})
            </TabsTrigger>
            <TabsTrigger value="technical">Technical Details</TabsTrigger>
          </TabsList>
          <TabsContent value="transactions" className="mt-4">
            <Card>
              <CardContent className="pt-6">
                <div className="rounded-sm border border-border">
                  <Table>
                    <TableHeader>
                      <TableRow>
                        <TableHead>Hash</TableHead>
                        <TableHead>Type</TableHead>
                        <TableHead>Inputs</TableHead>
                        <TableHead>Outputs</TableHead>
                        <TableHead>Fee</TableHead>
                      </TableRow>
                    </TableHeader>
                    <TableBody>
                      {transactions.map((tx) => (
                        <TableRow key={tx.tx_hash}>
                          <TableCell>
                            <Link
                              href={`/tx/${tx.tx_hash}`}
                              className="font-mono text-xs text-primary hover:underline"
                            >
                              {truncateHash(tx.tx_hash)}
                            </Link>
                          </TableCell>
                          <TableCell>{getScriptTypeBadge(tx.outputs[0]?.script_type || 'p2pkh')}</TableCell>
                          <TableCell>
                            <Badge variant="outline">{tx.inputs.length}</Badge>
                          </TableCell>
                          <TableCell>
                            <Badge variant="outline">{tx.outputs.length}</Badge>
                          </TableCell>
                          <TableCell className="font-mono text-xs">
                            {(calculateFee(tx) / 1000000000).toFixed(6)} BLS
                          </TableCell>
                        </TableRow>
                      ))}
                    </TableBody>
                  </Table>
                </div>
              </CardContent>
            </Card>
          </TabsContent>
          <TabsContent value="technical" className="mt-4">
            <Card>
              <CardContent className="pt-6">
                <div className="space-y-3">
                  <div className="flex justify-between">
                    <span className="text-sm text-muted-foreground">Version</span>
                    <span className="font-mono text-sm">{block.version}</span>
                  </div>
                  <Separator />
                  <div className="flex justify-between">
                    <span className="text-sm text-muted-foreground">Difficulty Target</span>
                    <span className="font-mono text-xs">{block.difficulty_target}</span>
                  </div>
                  <Separator />
                  <div className="flex justify-between">
                    <span className="text-sm text-muted-foreground">Nonce</span>
                    <span className="font-mono text-sm">{block.nonce}</span>
                  </div>
                  <Separator />
                  <div className="flex justify-between">
                    <span className="text-sm text-muted-foreground">Size</span>
                    <span className="font-mono text-sm">{formatBytes(block.size)}</span>
                  </div>
                  <Separator />
                  <div className="flex justify-between">
                    <span className="text-sm text-muted-foreground">Transaction Count</span>
                    <span className="font-mono text-sm">{block.tx_count}</span>
                  </div>
                </div>
              </CardContent>
            </Card>
          </TabsContent>
        </Tabs>

        <div className="flex justify-between">
          {block.height > 0 && (
            <Link href={`/block/${block.height - 1}`}>
              <Button variant="outline">
                <ArrowLeft className="mr-2 h-4 w-4" />
                Previous Block
              </Button>
            </Link>
          )}
          <div className="flex-1" />
          <Link href={`/block/${block.height + 1}`}>
            <Button variant="outline">
              Next Block
              <ArrowLeft className="ml-2 h-4 w-4 rotate-180" />
            </Button>
          </Link>
        </div>
      </div>
    </AppLayout>
  );
}
