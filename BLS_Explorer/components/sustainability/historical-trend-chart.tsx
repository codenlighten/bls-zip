import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { SustainabilityMetrics } from '@/lib/types';
import { TrendingDown } from 'lucide-react';

interface HistoricalTrendChartProps {
  metrics: SustainabilityMetrics[];
}

export function HistoricalTrendChart({ metrics }: HistoricalTrendChartProps) {
  const values = metrics.map(m => m.energy_per_tx_wh);
  const maxValue = Math.max(...values);
  const minValue = Math.min(...values);
  const range = maxValue - minValue;

  const chartWidth = 800;
  const chartHeight = 200;
  const padding = 40;

  const pointWidth = (chartWidth - 2 * padding) / (metrics.length - 1);

  const points = metrics.map((metric, index) => {
    const x = padding + index * pointWidth;
    const normalizedValue = range > 0 ? (metric.energy_per_tx_wh - minValue) / range : 0.5;
    const y = chartHeight - padding - normalizedValue * (chartHeight - 2 * padding);
    return { x, y, value: metric.energy_per_tx_wh };
  });

  const pathData = points.map((point, index) =>
    index === 0 ? `M ${point.x} ${point.y}` : `L ${point.x} ${point.y}`
  ).join(' ');

  const areaPath = `${pathData} L ${points[points.length - 1].x} ${chartHeight - padding} L ${padding} ${chartHeight - padding} Z`;

  const improvement = ((values[0] - values[values.length - 1]) / values[0] * 100).toFixed(1);

  return (
    <Card className="border-emerald-500/30 bg-navy">
      <CardHeader>
        <div className="flex items-center justify-between">
          <div className="flex items-center space-x-3">
            <div className="flex h-10 w-10 items-center justify-center rounded-sm bg-emerald-500/10">
              <TrendingDown className="h-5 w-5 text-emerald-500" />
            </div>
            <div>
              <CardTitle className="text-emerald-500">Energy Optimization Trend</CardTitle>
              <p className="text-sm text-muted-foreground">Last 30 days performance</p>
            </div>
          </div>
          <div className="text-right">
            <p className="text-2xl font-bold text-emerald-500">↓ {improvement}%</p>
            <p className="text-xs text-muted-foreground">Efficiency Gain</p>
          </div>
        </div>
      </CardHeader>
      <CardContent>
        <div className="overflow-x-auto">
          <svg width={chartWidth} height={chartHeight} className="mx-auto">
            <defs>
              <linearGradient id="energyGradient" x1="0%" y1="0%" x2="0%" y2="100%">
                <stop offset="0%" stopColor="rgb(16, 185, 129)" stopOpacity="0.3" />
                <stop offset="100%" stopColor="rgb(16, 185, 129)" stopOpacity="0.05" />
              </linearGradient>
            </defs>

            <line
              x1={padding}
              y1={chartHeight - padding}
              x2={chartWidth - padding}
              y2={chartHeight - padding}
              stroke="currentColor"
              strokeWidth="1"
              className="text-border"
            />
            <line
              x1={padding}
              y1={padding}
              x2={padding}
              y2={chartHeight - padding}
              stroke="currentColor"
              strokeWidth="1"
              className="text-border"
            />

            {[0, 25, 50, 75, 100].map((percent, index) => {
              const y = chartHeight - padding - (percent / 100) * (chartHeight - 2 * padding);
              const value = minValue + (percent / 100) * range;
              return (
                <g key={index}>
                  <line
                    x1={padding}
                    y1={y}
                    x2={chartWidth - padding}
                    y2={y}
                    stroke="currentColor"
                    strokeWidth="1"
                    strokeDasharray="4 4"
                    className="text-border"
                    opacity={0.3}
                  />
                  <text
                    x={padding - 10}
                    y={y + 4}
                    textAnchor="end"
                    className="fill-current text-xs text-muted-foreground"
                  >
                    {value.toFixed(2)}
                  </text>
                </g>
              );
            })}

            <path
              d={areaPath}
              fill="url(#energyGradient)"
            />

            <path
              d={pathData}
              fill="none"
              stroke="rgb(16, 185, 129)"
              strokeWidth="3"
              strokeLinecap="round"
              strokeLinejoin="round"
            />

            {points.map((point, index) => (
              <circle
                key={index}
                cx={point.x}
                cy={point.y}
                r="4"
                fill="rgb(16, 185, 129)"
                stroke="rgb(15, 23, 42)"
                strokeWidth="2"
              />
            ))}

            <text
              x={chartWidth / 2}
              y={chartHeight - 10}
              textAnchor="middle"
              className="fill-current text-xs text-muted-foreground"
            >
              Days Ago
            </text>
          </svg>
        </div>

        <div className="mt-4 rounded-sm bg-emerald-500/10 p-4">
          <p className="text-sm text-emerald-500">
            <span className="font-semibold">Continuous optimization</span> driven by post-quantum efficiency algorithms
          </p>
        </div>
      </CardContent>
    </Card>
  );
}
