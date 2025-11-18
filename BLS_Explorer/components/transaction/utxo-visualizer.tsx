import Link from 'next/link';
import { Transaction } from '@/lib/types';
import { ArrowRight } from 'lucide-react';
import { Card, CardContent } from '@/components/ui/card';
import { cn } from '@/lib/utils';

interface UtxoVisualizerProps {
  transaction: Transaction;
}

function truncateAddress(address: string): string {
  return `${address.substring(0, 12)}...${address.substring(address.length - 8)}`;
}

function formatAmount(amount: number): string {
  return (amount / 1000000000).toFixed(6) + ' BLS';
}

function calculateTotalInput(transaction: Transaction): number {
  return transaction.outputs.reduce((sum, output) => sum + output.amount, 0) * 1.01;
}

function calculateTotalOutput(transaction: Transaction): number {
  return transaction.outputs.reduce((sum, output) => sum + output.amount, 0);
}

function calculateFee(transaction: Transaction): number {
  return calculateTotalInput(transaction) - calculateTotalOutput(transaction);
}

export function UtxoVisualizer({ transaction }: UtxoVisualizerProps) {
  const totalInput = calculateTotalInput(transaction);
  const totalOutput = calculateTotalOutput(transaction);
  const fee = calculateFee(transaction);

  return (
    <Card>
      <CardContent className="p-6">
        <div className="grid grid-cols-[1fr_auto_1fr] gap-6">
          <div className="space-y-3">
            <div className="mb-4">
              <h3 className="text-sm font-semibold text-muted-foreground">INPUTS</h3>
              <p className="text-xs text-muted-foreground">
                {transaction.inputs.length} input{transaction.inputs.length !== 1 ? 's' : ''}
              </p>
            </div>
            {transaction.inputs.map((input, index) => (
              <div
                key={index}
                className="rounded-sm border border-border bg-secondary/50 p-3 transition-all hover:border-primary"
              >
                <Link
                  href={`/tx/${input.prev_output_hash}`}
                  className="block font-mono text-xs text-primary hover:underline"
                >
                  {truncateAddress(input.prev_output_hash)}
                </Link>
                <p className="mt-1 text-xs text-muted-foreground">
                  Output #{input.output_index}
                </p>
              </div>
            ))}
          </div>

          <div className="flex items-center justify-center">
            <div className="flex flex-col items-center space-y-2">
              <ArrowRight className="h-6 w-6 text-gold" />
              <div className="text-center">
                <p className="text-xs font-medium text-muted-foreground">Fee</p>
                <p className="font-mono text-xs text-gold">{formatAmount(fee)}</p>
              </div>
            </div>
          </div>

          <div className="space-y-3">
            <div className="mb-4">
              <h3 className="text-sm font-semibold text-muted-foreground">OUTPUTS</h3>
              <p className="text-xs text-muted-foreground">
                {transaction.outputs.length} output{transaction.outputs.length !== 1 ? 's' : ''}
              </p>
            </div>
            {transaction.outputs.map((output, index) => (
              <div
                key={index}
                className={cn(
                  'rounded-sm border p-3 transition-all hover:border-gold',
                  output.is_spent
                    ? 'border-muted bg-muted/30 opacity-60'
                    : 'border-gold bg-gold/10'
                )}
              >
                <Link
                  href={`/address/${output.recipient_hash}`}
                  className={cn(
                    'block font-mono text-xs hover:underline',
                    output.is_spent ? 'text-muted-foreground' : 'text-gold'
                  )}
                >
                  {truncateAddress(output.recipient_hash)}
                </Link>
                <p className={cn(
                  'mt-1 font-mono text-sm font-semibold',
                  output.is_spent ? 'text-muted-foreground' : 'text-gold'
                )}>
                  {formatAmount(output.amount)}
                </p>
                {!output.is_spent && (
                  <p className="mt-1 text-xs text-gold">Unspent</p>
                )}
                {output.is_spent && (
                  <p className="mt-1 text-xs text-muted-foreground">Spent</p>
                )}
              </div>
            ))}
          </div>
        </div>

        <div className="mt-6 grid grid-cols-3 gap-4 border-t border-border pt-4">
          <div>
            <p className="text-xs text-muted-foreground">Total Input</p>
            <p className="font-mono text-sm font-semibold">
              {formatAmount(totalInput)}
            </p>
          </div>
          <div>
            <p className="text-xs text-muted-foreground">Total Output</p>
            <p className="font-mono text-sm font-semibold">
              {formatAmount(totalOutput)}
            </p>
          </div>
          <div>
            <p className="text-xs text-muted-foreground">Transaction Fee</p>
            <p className="font-mono text-sm font-semibold text-gold">
              {formatAmount(fee)}
            </p>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
