import Link from 'next/link';
import { ProofType } from '@/lib/types';
import { Card, CardContent, CardHeader } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Shield, CheckCircle2, Award } from 'lucide-react';

interface ProofCardProps {
  identityId: string;
  proofType: ProofType;
  proofHash: string;
}

function formatProofType(type: ProofType): string {
  return type.replace(/_/g, ' ');
}

function getProofIcon(type: ProofType) {
  if (type === 'ACCREDITED') return Award;
  if (type === 'KYC_LEVEL_3') return Shield;
  return CheckCircle2;
}

export function ProofCard({ identityId, proofType, proofHash }: ProofCardProps) {
  const Icon = getProofIcon(proofType);

  return (
    <Card className="border-2 border-gold bg-gradient-to-br from-gold/5 to-transparent">
      <CardHeader className="border-b border-gold/30 pb-4">
        <div className="flex items-center space-x-3">
          <div className="flex h-12 w-12 items-center justify-center rounded-sm bg-gold">
            <Icon className="h-6 w-6 text-navy" />
          </div>
          <div className="flex-1">
            <h3 className="text-lg font-bold text-gold">Identity Attestation</h3>
            <p className="text-sm text-muted-foreground">Proof Anchor Transaction</p>
          </div>
          <Badge className="border-gold bg-gold/10 text-gold">
            Verified
          </Badge>
        </div>
      </CardHeader>
      <CardContent className="pt-6">
        <div className="space-y-6">
          <div className="rounded-sm border border-gold/30 bg-navy p-4">
            <div className="space-y-4">
              <div>
                <p className="mb-2 text-xs font-semibold uppercase tracking-wide text-gold">
                  Identity UUID
                </p>
                <Link
                  href={`/identity/${identityId}`}
                  className="block font-mono text-sm text-foreground hover:text-gold hover:underline"
                >
                  {identityId}
                </Link>
              </div>

              <div className="border-t border-border pt-4">
                <p className="mb-2 text-xs font-semibold uppercase tracking-wide text-gold">
                  Verification Level
                </p>
                <Badge variant="outline" className="border-gold text-gold">
                  {formatProofType(proofType)}
                </Badge>
              </div>

              <div className="border-t border-border pt-4">
                <p className="mb-2 text-xs font-semibold uppercase tracking-wide text-gold">
                  Proof Hash
                </p>
                <p className="font-mono text-xs text-muted-foreground">
                  {proofHash.substring(0, 32)}...
                </p>
              </div>
            </div>
          </div>

          <div className="flex items-start space-x-3 rounded-sm bg-success/10 p-4">
            <CheckCircle2 className="mt-0.5 h-5 w-5 shrink-0 text-success" />
            <div className="text-sm">
              <p className="font-semibold text-success">Anchored & Immutable</p>
              <p className="mt-1 text-muted-foreground">
                This transaction permanently anchors an enterprise identity verification to the
                blockchain, providing immutable proof of compliance and credential verification
                secured by post-quantum cryptography.
              </p>
            </div>
          </div>

          <Link href={`/identity/${identityId}`} className="block">
            <Button className="w-full bg-gold text-navy hover:bg-gold/90">
              <Shield className="mr-2 h-4 w-4" />
              View Trust Chain
            </Button>
          </Link>
        </div>
      </CardContent>
    </Card>
  );
}
