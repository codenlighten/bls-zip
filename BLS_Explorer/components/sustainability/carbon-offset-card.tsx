import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Sprout, TreeDeciduous } from 'lucide-react';

interface CarbonOffsetCardProps {
  carbonFootprint: number;
}

export function CarbonOffsetCard({ carbonFootprint }: CarbonOffsetCardProps) {
  const treesEquivalent = Math.floor(carbonFootprint / 20);

  return (
    <Card className="border-emerald-500/30 bg-navy">
      <CardHeader>
        <div className="flex items-center space-x-3">
          <div className="flex h-10 w-10 items-center justify-center rounded-sm bg-emerald-500/10">
            <Sprout className="h-5 w-5 text-emerald-500" />
          </div>
          <div>
            <CardTitle className="text-emerald-500">Carbon Offset</CardTitle>
            <p className="text-sm text-muted-foreground">Environmental impact tracking</p>
          </div>
        </div>
      </CardHeader>
      <CardContent className="space-y-6">
        <div className="text-center">
          <p className="text-5xl font-bold text-emerald-500">
            {carbonFootprint.toFixed(1)}
          </p>
          <p className="mt-2 text-sm text-muted-foreground">kg CO₂ Total Footprint</p>
        </div>

        <div className="space-y-3">
          <div className="flex items-center justify-between rounded-sm border border-emerald-500/30 bg-emerald-500/5 p-4">
            <div className="flex items-center space-x-3">
              <TreeDeciduous className="h-6 w-6 text-emerald-500" />
              <div>
                <p className="text-sm font-semibold text-foreground">Trees Equivalent</p>
                <p className="text-xs text-muted-foreground">Annual CO₂ absorption</p>
              </div>
            </div>
            <p className="text-3xl font-bold text-emerald-500">{treesEquivalent}</p>
          </div>

          <div className="grid grid-cols-2 gap-3">
            <div className="rounded-sm border border-border bg-secondary p-3">
              <p className="text-xs text-muted-foreground">Daily Emissions</p>
              <p className="mt-1 text-lg font-bold text-emerald-500">
                {(carbonFootprint / 365).toFixed(2)} kg
              </p>
            </div>
            <div className="rounded-sm border border-border bg-secondary p-3">
              <p className="text-xs text-muted-foreground">Per Transaction</p>
              <p className="mt-1 text-lg font-bold text-emerald-500">
                0.02 g
              </p>
            </div>
          </div>
        </div>

        <div className="rounded-sm bg-emerald-500/10 p-4">
          <p className="text-sm text-emerald-500">
            <span className="font-semibold">99.9% lower carbon</span> than proof-of-work networks
          </p>
          <p className="mt-1 text-xs text-muted-foreground">
            Post-quantum security without the environmental cost
          </p>
        </div>
      </CardContent>
    </Card>
  );
}
