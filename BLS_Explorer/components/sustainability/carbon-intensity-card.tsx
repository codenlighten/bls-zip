import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Leaf } from 'lucide-react';

interface CarbonIntensityCardProps {
  carbonPerTxG: number;
}

export function CarbonIntensityCard({ carbonPerTxG }: CarbonIntensityCardProps) {
  return (
    <Card className="border-emerald-500/20 bg-card">
      <CardHeader className="pb-3">
        <CardTitle className="flex items-center gap-2 text-sm font-medium text-muted-foreground">
          <Leaf className="h-4 w-4 text-emerald-500" />
          Carbon Intensity
        </CardTitle>
      </CardHeader>
      <CardContent>
        <div className="text-3xl font-bold text-foreground">
          {carbonPerTxG.toFixed(2)}
          <span className="ml-2 text-lg font-normal text-muted-foreground">g CO₂</span>
        </div>
        <div className="mt-2 text-xs text-muted-foreground">per transaction</div>
        <div className="mt-4 flex items-center gap-2">
          <div className="h-2 flex-1 rounded-full bg-secondary">
            <div
              className="h-full rounded-full bg-gradient-to-r from-emerald-500 to-teal-500"
              style={{ width: `${Math.min((carbonPerTxG / 500) * 100, 100)}%` }}
            />
          </div>
          <div className="text-xs font-semibold text-emerald-500">
            {((1 - carbonPerTxG / 500) * 100).toFixed(0)}% cleaner
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
