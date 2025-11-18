import Link from 'next/link';
import { Transaction } from '@/lib/types';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';
import { Badge } from '@/components/ui/badge';

interface TransactionsTableProps {
  transactions: Transaction[];
}

function truncateHash(hash: string): string {
  return `${hash.substring(0, 10)}...${hash.substring(hash.length - 10)}`;
}

function getScriptTypeBadge(scriptType: string) {
  const variants: Record<string, 'default' | 'secondary' | 'outline'> = {
    p2pkh: 'secondary',
    proof_anchor: 'default',
    contract_deploy: 'outline',
  };

  return <Badge variant={variants[scriptType] || 'secondary'}>{scriptType.replace('_', ' ')}</Badge>;
}

function formatAmount(amount: number): string {
  return (amount / 1000000000).toFixed(2) + ' BLS';
}

function calculateTotalOutput(tx: Transaction): number {
  return tx.outputs.reduce((sum, output) => sum + output.amount, 0);
}

export function TransactionsTable({ transactions }: TransactionsTableProps) {
  return (
    <div className="rounded-sm border border-border">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Hash</TableHead>
            <TableHead>Type</TableHead>
            <TableHead>Value</TableHead>
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
              <TableCell className="font-mono text-sm">
                {formatAmount(calculateTotalOutput(tx))}
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </div>
  );
}
