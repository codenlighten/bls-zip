import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Zap } from 'lucide-react';

interface EnergyIntensityCardProps {
  energyPerTx: number;
}

export function EnergyIntensityCard({ energyPerTx }: EnergyIntensityCardProps) {
  const comparisons = [
    { name: 'Boundless BLS', value: energyPerTx, color: 'bg-emerald-500' },
    { name: 'Ethereum', value: 62.5, color: 'bg-blue-500' },
    { name: 'Bitcoin', value: 707, color: 'bg-yellow-500' },
  ];

  const maxValue = Math.max(...comparisons.map(c => c.value));

  return (
    <Card className="border-emerald-500/30 bg-navy">
      <CardHeader>
        <div className="flex items-center space-x-3">
          <div className="flex h-10 w-10 items-center justify-center rounded-sm bg-emerald-500/10">
            <Zap className="h-5 w-5 text-emerald-500" />
          </div>
          <div>
            <CardTitle className="text-emerald-500">Energy Intensity</CardTitle>
            <p className="text-sm text-muted-foreground">Watt-hours per transaction</p>
          </div>
        </div>
      </CardHeader>
      <CardContent className="space-y-6">
        {comparisons.map((comparison, index) => (
          <div key={index} className="space-y-2">
            <div className="flex items-center justify-between text-sm">
              <span className={index === 0 ? 'font-semibold text-emerald-500' : 'text-foreground'}>
                {comparison.name}
              </span>
              <span className={index === 0 ? 'font-bold text-emerald-500' : 'text-muted-foreground'}>
                {comparison.value.toFixed(2)} Wh
              </span>
            </div>
            <div className="h-3 w-full overflow-hidden rounded-full bg-secondary">
              <div
                className={`h-full ${comparison.color} transition-all duration-1000`}
                style={{ width: `${(comparison.value / maxValue) * 100}%` }}
              />
            </div>
          </div>
        ))}

        <div className="mt-6 rounded-sm bg-emerald-500/10 p-4">
          <p className="text-sm font-semibold text-emerald-500">
            {((comparisons[1].value / energyPerTx)).toFixed(0)}x more efficient than Ethereum
          </p>
          <p className="mt-1 text-xs text-muted-foreground">
            {((comparisons[2].value / energyPerTx)).toFixed(0)}x more efficient than Bitcoin
          </p>
        </div>
      </CardContent>
    </Card>
  );
}
