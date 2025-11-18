import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Factory } from 'lucide-react';

interface OffsetEquivalentCardProps {
  dailyCarbonKg: number;
}

export function OffsetEquivalentCard({ dailyCarbonKg }: OffsetEquivalentCardProps) {
  const treesRequired = Math.ceil(dailyCarbonKg / 20);

  return (
    <Card className="border-emerald-500/20 bg-card">
      <CardHeader className="pb-3">
        <CardTitle className="flex items-center gap-2 text-sm font-medium text-muted-foreground">
          <Factory className="h-4 w-4 text-emerald-500" />
          Offset Equivalent
        </CardTitle>
      </CardHeader>
      <CardContent>
        <div className="text-3xl font-bold text-foreground">
          {treesRequired.toLocaleString()}
          <span className="ml-2 text-lg font-normal text-muted-foreground">Trees</span>
        </div>
        <div className="mt-2 text-xs text-muted-foreground">required for carbon neutrality</div>
        <div className="mt-4 rounded-sm bg-secondary/50 p-3">
          <div className="flex items-center justify-between">
            <span className="text-xs text-muted-foreground">Daily Footprint</span>
            <span className="text-xs font-semibold text-emerald-400">
              {dailyCarbonKg.toFixed(2)} kg CO₂
            </span>
          </div>
          <div className="mt-2 text-xs text-muted-foreground">
            Based on 20 kg CO₂ absorption per tree annually
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
