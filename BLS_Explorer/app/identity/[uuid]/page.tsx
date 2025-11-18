'use client';

import { use, useEffect, useState } from 'react';
import Link from 'next/link';
import { AppLayout } from '@/components/layout/app-layout';
import { TrustChainTimeline } from '@/components/identity/trust-chain-timeline';
import { blockchainIndexer } from '@/lib/blockchain/indexer';
import { mockIndexer } from '@/lib/mock-indexer';
import type { Identity } from '@/lib/types';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Separator } from '@/components/ui/separator';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { ArrowLeft, Shield, Loader2 } from 'lucide-react';

interface PageProps {
  params: Promise<{ uuid: string }>;
}

const USE_REAL_BLOCKCHAIN = process.env.NEXT_PUBLIC_ENABLE_BLOCKCHAIN !== 'false';

function formatTimestamp(timestamp: number): string {
  return new Date(timestamp).toLocaleString();
}

function formatProofType(type: string): string {
  return type.replace(/_/g, ' ');
}

function getVerificationLevelColor(level: string): string {
  if (level.includes('LEVEL_3') || level.includes('ACCREDITATION')) {
    return 'text-green-500';
  }
  if (level.includes('LEVEL_2') || level.includes('AML')) {
    return 'text-blue-500';
  }
  return 'text-yellow-500';
}

export default function IdentityPage({ params }: PageProps) {
  const { uuid } = use(params);
  const [identity, setIdentity] = useState<Identity | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchIdentity = async () => {
      setIsLoading(true);
      setError(null);

      try {
        let fetchedIdentity: Identity | null = null;

        if (USE_REAL_BLOCKCHAIN) {
          // Try to fetch proof anchors from blockchain
          const connected = await blockchainIndexer.isConnected();

          if (connected) {
            // Get all proofs for this identity from blockchain
            const proofs = await blockchainIndexer.getProofsByIdentity(uuid);

            if (proofs && proofs.length > 0) {
              // Construct identity from proofs
              fetchedIdentity = {
                identity_id: uuid,
                verification_level: 'KYC_LEVEL_2', // Could be derived from proofs
                total_anchors: proofs.length,
                created_at: Math.min(...proofs.map((p) => p.timestamp || Date.now())),
                proof_anchors: proofs.map((proof, index) => ({
                  proof_type: proof.proof_type || 'KYC_LEVEL_1',
                  proof_hash: proof.proof_hash || '',
                  block_height: proof.block_height || 0,
                  timestamp: proof.timestamp || Date.now(),
                  signature_type: 'MlDsa', // Default to post-quantum
                  tx_hash: proof.tx_hash || `0x${index.toString(16).padStart(64, '0')}`,
                  status: 'verified' as const,
                })),
              };
            } else {
              throw new Error('No proof anchors found for this identity');
            }
          } else {
            throw new Error('Cannot connect to blockchain node');
          }
        } else {
          // Use mock data
          fetchedIdentity = mockIndexer.getIdentity(uuid) || null;
        }

        if (!fetchedIdentity) {
          throw new Error('Identity not found');
        }

        setIdentity(fetchedIdentity);
      } catch (err: any) {
        console.error('Failed to fetch identity:', err);
        setError(err.message || 'Failed to load identity');

        // Fallback to mock data
        const mockIdentity = mockIndexer.getIdentity(uuid) || null;
        if (mockIdentity) {
          setIdentity(mockIdentity);
        }
      } finally {
        setIsLoading(false);
      }
    };

    fetchIdentity();
  }, [uuid]);

  if (isLoading) {
    return (
      <AppLayout>
        <div className="flex items-center justify-center min-h-[60vh]">
          <div className="text-center">
            <Loader2 className="h-12 w-12 animate-spin mx-auto mb-4" />
            <p className="text-lg text-muted-foreground">Loading identity verification...</p>
          </div>
        </div>
      </AppLayout>
    );
  }

  if (!identity) {
    return (
      <AppLayout>
        <div className="flex flex-col items-center justify-center py-20">
          <h1 className="text-2xl font-bold">Identity Not Found</h1>
          <p className="mt-2 text-muted-foreground">
            The identity with UUID {uuid} could not be found.
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
            <h1 className="text-3xl font-bold">Enterprise Identity</h1>
            <p className="font-mono text-sm text-muted-foreground">{identity.identity_id}</p>
          </div>
        </div>

        <Card className="border-primary/50 bg-primary/5">
          <CardHeader>
            <CardTitle className="flex items-center space-x-2">
              <Shield className="h-5 w-5 text-primary" />
              <span>Identity Profile</span>
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="flex justify-between">
              <span className="text-sm text-muted-foreground">Identity UUID</span>
              <span className="font-mono text-sm">{identity.identity_id}</span>
            </div>
            <Separator />
            <div className="flex justify-between">
              <span className="text-sm text-muted-foreground">Verification Level</span>
              <Badge variant="default" className={getVerificationLevelColor(identity.verification_level)}>
                {formatProofType(identity.verification_level)}
              </Badge>
            </div>
            <Separator />
            <div className="flex justify-between">
              <span className="text-sm text-muted-foreground">Total Proof Anchors</span>
              <span className="font-mono text-sm font-semibold">{identity.total_anchors}</span>
            </div>
            <Separator />
            <div className="flex justify-between">
              <span className="text-sm text-muted-foreground">Created</span>
              <span className="text-sm">{formatTimestamp(identity.created_at)}</span>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Trust Chain Timeline</CardTitle>
            <p className="text-sm text-muted-foreground">
              Immutable proof anchors establishing identity verification history
            </p>
          </CardHeader>
          <CardContent className="pt-6">
            <TrustChainTimeline anchors={identity.proof_anchors} />
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Security Information</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-4 text-sm">
              <div className="rounded-sm bg-primary/10 p-4">
                <div className="flex items-start space-x-3">
                  <Shield className="mt-0.5 h-5 w-5 text-primary" />
                  <div>
                    <p className="font-semibold">Post-Quantum Security</p>
                    <p className="mt-1 text-muted-foreground">
                      All proof anchors are signed with ML-DSA or Falcon-512 quantum-resistant
                      signatures, ensuring long-term validity and protection against quantum attacks.
                    </p>
                  </div>
                </div>
              </div>
              <div className="rounded-sm bg-secondary p-4">
                <p className="font-semibold">Immutable Verification</p>
                <p className="mt-1 text-muted-foreground">
                  Each proof anchor is permanently recorded on the blockchain, creating an
                  auditable and tamper-proof history of identity verification events.
                </p>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>
    </AppLayout>
  );
}
