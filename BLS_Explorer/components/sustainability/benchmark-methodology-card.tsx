import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Scale } from 'lucide-react';

export function BenchmarkMethodologyCard() {
  return (
    <Card className="border-emerald-500/20 bg-card">
      <CardHeader>
        <CardTitle className="flex items-center gap-2 text-lg">
          <Scale className="h-5 w-5 text-emerald-500" />
          Benchmark Methodology
        </CardTitle>
      </CardHeader>
      <CardContent>
        <p className="text-sm text-muted-foreground leading-relaxed">
          Estimates based on <span className="font-semibold text-emerald-400">Antminer S19 efficiency</span> (110 TH/s @ 3.25kW)
          and standard grid intensity (0.5 kgCO₂/kWh). This industry-standard benchmark provides a conservative
          estimate of network environmental impact.
        </p>
        <div className="mt-4 grid grid-cols-3 gap-4 rounded-sm bg-secondary/50 p-3">
          <div>
            <div className="text-xs text-muted-foreground">Hashrate</div>
            <div className="text-sm font-semibold text-emerald-400">110 TH/s</div>
          </div>
          <div>
            <div className="text-xs text-muted-foreground">Power Draw</div>
            <div className="text-sm font-semibold text-emerald-400">3.25 kW</div>
          </div>
          <div>
            <div className="text-xs text-muted-foreground">Grid Factor</div>
            <div className="text-sm font-semibold text-emerald-400">0.5 kg/kWh</div>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
