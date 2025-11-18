import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Zap } from 'lucide-react';

interface AnnualizedDrawCardProps {
  dailyEnergyKWh: number;
}

export function AnnualizedDrawCard({ dailyEnergyKWh }: AnnualizedDrawCardProps) {
  const annualGWh = (dailyEnergyKWh * 365) / 1_000_000;

  return (
    <Card className="border-emerald-500/20 bg-card">
      <CardHeader className="pb-3">
        <CardTitle className="flex items-center gap-2 text-sm font-medium text-muted-foreground">
          <Zap className="h-4 w-4 text-emerald-500" />
          Annualized Draw
        </CardTitle>
      </CardHeader>
      <CardContent>
        <div className="text-3xl font-bold text-foreground">
          {annualGWh.toFixed(2)}
          <span className="ml-2 text-lg font-normal text-muted-foreground">GWh/yr</span>
        </div>
        <div className="mt-2 text-xs text-muted-foreground">projected annual consumption</div>
        <div className="mt-4 rounded-sm bg-secondary/50 p-3">
          <div className="flex items-center justify-between">
            <span className="text-xs text-muted-foreground">Daily Average</span>
            <span className="text-xs font-semibold text-emerald-400">
              {(dailyEnergyKWh / 1000).toFixed(2)} MWh
            </span>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
