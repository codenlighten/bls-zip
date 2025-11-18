import Link from 'next/link';
import { Block } from '@/lib/types';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';
import { Badge } from '@/components/ui/badge';

interface BlocksTableProps {
  blocks: Block[];
}

function formatTimestamp(timestamp: number): string {
  const seconds = Math.floor((Date.now() - timestamp) / 1000);

  if (seconds < 60) return `${seconds}s ago`;
  if (seconds < 3600) return `${Math.floor(seconds / 60)}m ago`;
  if (seconds < 86400) return `${Math.floor(seconds / 3600)}h ago`;
  return `${Math.floor(seconds / 86400)}d ago`;
}

function truncateHash(hash: string): string {
  return `${hash.substring(0, 8)}...${hash.substring(hash.length - 8)}`;
}

export function BlocksTable({ blocks }: BlocksTableProps) {
  return (
    <div className="rounded-sm border border-border">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Height</TableHead>
            <TableHead>Miner</TableHead>
            <TableHead>Transactions</TableHead>
            <TableHead>Time</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {blocks.map((block) => (
            <TableRow key={block.height}>
              <TableCell>
                <Link
                  href={`/block/${block.height}`}
                  className="font-mono text-sm font-medium text-primary hover:underline"
                >
                  #{block.height}
                </Link>
              </TableCell>
              <TableCell>
                <span className="font-mono text-xs text-muted-foreground">
                  {truncateHash(block.miner)}
                </span>
              </TableCell>
              <TableCell>
                <Badge variant="secondary">{block.tx_count}</Badge>
              </TableCell>
              <TableCell className="text-sm text-muted-foreground">
                {formatTimestamp(block.timestamp)}
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </div>
  );
}
