import Link from 'next/link';
import { ProofAnchor, ProofType } from '@/lib/types';
import { Badge } from '@/components/ui/badge';

interface TrustChainTimelineProps {
  anchors: ProofAnchor[];
}

function formatTimestamp(timestamp: number): string {
  return new Date(timestamp).toLocaleString();
}

function formatProofType(type: ProofType): string {
  return type.replace(/_/g, ' ');
}

function getProofIcon(type: ProofType): string {
  switch (type) {
    case 'KYC_LEVEL_1':
      return '📧';
    case 'KYC_LEVEL_2':
      return '🆔';
    case 'KYC_LEVEL_3':
      return '🏦';
    case 'ACCREDITED':
      return '💼';
    default:
      return '✓';
  }
}

function getProofDescription(type: ProofType): string {
  switch (type) {
    case 'KYC_LEVEL_1':
      return 'Email Verified';
    case 'KYC_LEVEL_2':
      return 'Document Verified';
    case 'KYC_LEVEL_3':
      return 'Bank Verified';
    case 'ACCREDITED':
      return 'Investor Status';
    default:
      return 'Verified';
  }
}

export function TrustChainTimeline({ anchors }: TrustChainTimelineProps) {
  return (
    <div className="relative space-y-6">
      <div className="absolute left-8 top-0 h-full w-0.5 bg-gold" />
      {anchors.map((anchor, index) => (
        <div key={anchor.tx_hash} className="relative flex space-x-4">
          <div className="relative flex h-16 w-16 shrink-0 items-center justify-center rounded-sm border-2 border-gold bg-navy">
            <span className="text-3xl">{getProofIcon(anchor.proof_type)}</span>
          </div>
          <div className="flex-1 pb-6">
            <div className="flex items-start justify-between">
              <div>
                <div className="mb-2 flex items-center space-x-3">
                  <Badge className="border-gold bg-gold/10 text-gold">
                    {formatProofType(anchor.proof_type)}
                  </Badge>
                  <Badge
                    variant="outline"
                    className="border-success text-success"
                  >
                    {anchor.status}
                  </Badge>
                </div>
                <p className="text-lg font-semibold text-foreground">
                  {getProofDescription(anchor.proof_type)}
                </p>
                <p className="mt-1 text-sm text-muted-foreground">
                  {formatTimestamp(anchor.timestamp)}
                </p>
              </div>
            </div>
            <div className="mt-4 rounded-sm border border-border bg-navy p-4">
              <div className="space-y-3 text-sm">
                <div className="flex justify-between">
                  <span className="text-muted-foreground">Transaction</span>
                  <Link
                    href={`/tx/${anchor.tx_hash}`}
                    className="font-mono text-xs text-primary hover:underline"
                  >
                    {anchor.tx_hash.substring(0, 16)}...
                  </Link>
                </div>
                <div className="flex justify-between">
                  <span className="text-muted-foreground">Block Height</span>
                  <Link
                    href={`/block/${anchor.block_height}`}
                    className="font-mono text-xs text-primary hover:underline"
                  >
                    #{anchor.block_height}
                  </Link>
                </div>
                <div className="flex justify-between">
                  <span className="text-muted-foreground">Proof Hash</span>
                  <span className="font-mono text-xs text-muted-foreground">
                    {anchor.proof_hash.substring(0, 16)}...
                  </span>
                </div>
              </div>
            </div>
          </div>
        </div>
      ))}
    </div>
  );
}
