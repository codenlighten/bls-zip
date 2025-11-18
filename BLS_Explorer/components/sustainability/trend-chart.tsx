import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import type { SustainabilityMetrics } from '@/lib/types';

interface TrendChartProps {
  metrics: SustainabilityMetrics[];
}

export function TrendChart({ metrics }: TrendChartProps) {
  if (metrics.length === 0) return null;

  const maxCarbon = Math.max(...metrics.map(m => m.carbon_per_tx_g));
  const minCarbon = Math.min(...metrics.map(m => m.carbon_per_tx_g));
  const range = maxCarbon - minCarbon;
  const padding = range * 0.1;

  const width = 1000;
  const height = 300;
  const chartHeight = height - 60;
  const chartWidth = width - 80;

  const points = metrics.map((m, i) => {
    const x = 60 + (i / (metrics.length - 1)) * chartWidth;
    const normalizedValue = (m.carbon_per_tx_g - minCarbon + padding) / (range + padding * 2);
    const y = height - 40 - normalizedValue * chartHeight;
    return { x, y, value: m.carbon_per_tx_g, date: new Date(m.timestamp) };
  });

  const pathData = points.map((p, i) => `${i === 0 ? 'M' : 'L'} ${p.x} ${p.y}`).join(' ');

  const areaData = `${pathData} L ${points[points.length - 1].x} ${height - 40} L 60 ${height - 40} Z`;

  const improvement = ((maxCarbon - minCarbon) / maxCarbon * 100).toFixed(1);

  return (
    <Card className="border-emerald-500/20 bg-card">
      <CardHeader>
        <div className="flex items-center justify-between">
          <CardTitle className="text-lg">Historical Carbon Intensity Trend</CardTitle>
          <div className="flex items-center gap-2">
            <div className="text-sm text-emerald-500 font-semibold">↓ {improvement}%</div>
            <div className="text-xs text-muted-foreground">30-day improvement</div>
          </div>
        </div>
      </CardHeader>
      <CardContent>
        <svg viewBox={`0 0 ${width} ${height}`} className="w-full">
          <defs>
            <linearGradient id="carbonGradient" x1="0" x2="0" y1="0" y2="1">
              <stop offset="0%" stopColor="#10b981" stopOpacity="0.3" />
              <stop offset="100%" stopColor="#10b981" stopOpacity="0.05" />
            </linearGradient>
          </defs>

          <line x1="60" y1={height - 40} x2={width - 20} y2={height - 40} stroke="currentColor" strokeOpacity="0.1" strokeWidth="1" />

          <path d={areaData} fill="url(#carbonGradient)" />

          <path d={pathData} stroke="#10b981" strokeWidth="2" fill="none" />

          {points.map((p, i) => (
            <g key={i}>
              <circle cx={p.x} cy={p.y} r="3" fill="#10b981" />
              {i === 0 && (
                <text x={p.x} y={height - 15} fill="currentColor" fontSize="12" textAnchor="middle" opacity="0.6">
                  Day 1
                </text>
              )}
              {i === points.length - 1 && (
                <>
                  <text x={p.x} y={height - 15} fill="currentColor" fontSize="12" textAnchor="middle" opacity="0.6">
                    Today
                  </text>
                  <text x={p.x + 10} y={p.y - 10} fill="#10b981" fontSize="14" fontWeight="600">
                    {p.value.toFixed(2)}g
                  </text>
                </>
              )}
            </g>
          ))}

          <text x="20" y="20" fill="currentColor" fontSize="12" opacity="0.6">
            {maxCarbon.toFixed(2)}g
          </text>
          <text x="20" y={height - 25} fill="currentColor" fontSize="12" opacity="0.6">
            {minCarbon.toFixed(2)}g
          </text>
        </svg>
        <div className="mt-4 text-center text-xs text-muted-foreground">
          Carbon emissions per transaction showing continuous efficiency improvements over the past 30 days
        </div>
      </CardContent>
    </Card>
  );
}
