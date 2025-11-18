import Link from 'next/link';
import { AssetTransfer } from '@/lib/types';
import { Card, CardContent, CardHeader } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { ArrowRight, Package, DollarSign } from 'lucide-react';

interface AssetTransferTicketProps {
  transfer: AssetTransfer;
  txHash: string;
}

function truncateAddress(address: string): string {
  return `${address.substring(0, 12)}...${address.substring(address.length - 8)}`;
}

export function AssetTransferTicket({ transfer, txHash }: AssetTransferTicketProps) {
  return (
    <Card className="border-2 border-[#8b5cf6] bg-gradient-to-br from-[#8b5cf6]/5 to-transparent">
      <CardHeader className="border-b border-[#8b5cf6]/30 pb-4">
        <div className="flex items-center space-x-3">
          <div className="flex h-12 w-12 items-center justify-center rounded-sm bg-[#8b5cf6]">
            <Package className="h-6 w-6 text-navy" />
          </div>
          <div className="flex-1">
            <h3 className="text-lg font-bold text-[#8b5cf6]">Asset Transfer</h3>
            <p className="text-sm text-muted-foreground">
              {transfer.metadata?.asset_type || 'Digital Asset'}
            </p>
          </div>
          <Badge className="border-[#8b5cf6] bg-[#8b5cf6]/10 text-[#8b5cf6]">
            Metadata Transfer
          </Badge>
        </div>
      </CardHeader>
      <CardContent className="pt-6">
        <div className="space-y-6">
          <div className="grid grid-cols-[1fr_auto_1fr] items-center gap-6">
            <div className="rounded-sm border border-[#8b5cf6]/30 bg-navy p-4">
              <p className="mb-2 text-xs font-semibold uppercase tracking-wide text-[#8b5cf6]">
                From
              </p>
              <Link
                href={`/address/${transfer.from_address}`}
                className="block font-mono text-sm text-foreground hover:text-[#8b5cf6] hover:underline"
              >
                {truncateAddress(transfer.from_address)}
              </Link>
            </div>

            <div className="flex flex-col items-center">
              <ArrowRight className="h-6 w-6 text-[#8b5cf6]" />
            </div>

            <div className="rounded-sm border border-[#8b5cf6]/30 bg-navy p-4">
              <p className="mb-2 text-xs font-semibold uppercase tracking-wide text-[#8b5cf6]">
                To
              </p>
              <Link
                href={`/address/${transfer.to_address}`}
                className="block font-mono text-sm text-foreground hover:text-[#8b5cf6] hover:underline"
              >
                {truncateAddress(transfer.to_address)}
              </Link>
            </div>
          </div>

          <div className="space-y-4 rounded-sm border border-[#8b5cf6]/30 bg-navy p-4">
            <div className="flex items-start justify-between">
              <div className="flex-1">
                <p className="mb-2 text-xs font-semibold uppercase tracking-wide text-[#8b5cf6]">
                  Asset ID
                </p>
                <Link
                  href={`/asset/${transfer.asset_id}`}
                  className="font-mono text-sm text-foreground hover:text-[#8b5cf6] hover:underline"
                >
                  {transfer.asset_id}
                </Link>
              </div>
            </div>

            <div className="border-t border-border pt-4">
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <p className="mb-2 text-xs font-semibold uppercase tracking-wide text-[#8b5cf6]">
                    Quantity
                  </p>
                  <p className="text-2xl font-bold text-gold">
                    {transfer.quantity.toLocaleString()}
                  </p>
                </div>
                {transfer.price && (
                  <div>
                    <p className="mb-2 text-xs font-semibold uppercase tracking-wide text-[#8b5cf6]">
                      Price per Unit
                    </p>
                    <div className="flex items-center space-x-1">
                      <DollarSign className="h-5 w-5 text-gold" />
                      <p className="text-2xl font-bold text-gold">
                        {(transfer.price / 1000000000).toFixed(2)}
                      </p>
                      <span className="text-sm text-muted-foreground">BLS</span>
                    </div>
                  </div>
                )}
              </div>
            </div>

            {transfer.price && (
              <div className="border-t border-border pt-4">
                <p className="mb-2 text-xs font-semibold uppercase tracking-wide text-[#8b5cf6]">
                  Total Value
                </p>
                <p className="text-3xl font-bold text-gold">
                  {((transfer.quantity * transfer.price) / 1000000000).toLocaleString()} BLS
                </p>
              </div>
            )}
          </div>

          <div className="rounded-sm bg-[#8b5cf6]/10 p-4">
            <p className="text-sm text-muted-foreground">
              This is a metadata transfer with 0 native value. The asset transfer is encoded in
              the transaction data field and represents ownership change of off-chain or
              tokenized assets on the Boundless BLS blockchain.
            </p>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
