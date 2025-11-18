'use client';

import { AppLayout } from '@/components/layout/app-layout';
import { NetworkGradeRing } from '@/components/sustainability/network-grade-ring';
import { BenchmarkMethodologyCard } from '@/components/sustainability/benchmark-methodology-card';
import { CarbonIntensityCard } from '@/components/sustainability/carbon-intensity-card';
import { AnnualizedDrawCard } from '@/components/sustainability/annualized-draw-card';
import { OffsetEquivalentCard } from '@/components/sustainability/offset-equivalent-card';
import { TrendChart } from '@/components/sustainability/trend-chart';
import { mockIndexer } from '@/lib/mock-indexer';
import { calculateNetworkImpact, calculateNetworkGrade } from '@/lib/sustainability-calc';

export default function SustainabilityPage() {
  const metrics = mockIndexer.getSustainabilityMetrics();
  const historicalData = mockIndexer.getHistoricalSustainabilityMetrics(30);

  const impact = calculateNetworkImpact(metrics.network_hashrate_th, metrics.tx_count_24h);
  const grade = calculateNetworkGrade(impact.energyPerTxWh);

  return (
    <AppLayout>
      <div className="space-y-8">
        <div>
          <h1 className="text-4xl font-bold text-foreground">Sustainability Dashboard</h1>
          <p className="mt-2 text-muted-foreground">
            Enterprise ESG metrics based on Antminer S19 benchmark methodology
          </p>
        </div>

        <div className="grid gap-8 lg:grid-cols-5">
          <div className="lg:col-span-2">
            <NetworkGradeRing grade={grade} energyPerTxWh={impact.energyPerTxWh} />
          </div>

          <div className="lg:col-span-3">
            <BenchmarkMethodologyCard />
          </div>
        </div>

        <div className="grid gap-6 md:grid-cols-3">
          <CarbonIntensityCard carbonPerTxG={impact.carbonPerTxG} />
          <AnnualizedDrawCard dailyEnergyKWh={impact.dailyEnergyKWh} />
          <OffsetEquivalentCard dailyCarbonKg={impact.dailyCarbonKg} />
        </div>

        <TrendChart metrics={historicalData} />
      </div>
    </AppLayout>
  );
}
