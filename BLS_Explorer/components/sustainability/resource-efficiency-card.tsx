import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Server } from 'lucide-react';

interface ResourceEfficiencyCardProps {
  storage: number;
  network: number;
  compute: number;
}

export function ResourceEfficiencyCard({ storage, network, compute }: ResourceEfficiencyCardProps) {
  const maxValue = 100;
  const angleStep = 120;

  const points = [
    { label: 'Storage', value: storage, angle: -90 },
    { label: 'Network', value: network, angle: 30 },
    { label: 'Compute', value: compute, angle: 150 },
  ];

  const polarToCartesian = (angle: number, value: number, centerX: number, centerY: number, radius: number) => {
    const angleRad = (angle * Math.PI) / 180;
    const scaledRadius = (value / maxValue) * radius;
    return {
      x: centerX + scaledRadius * Math.cos(angleRad),
      y: centerY + scaledRadius * Math.sin(angleRad),
    };
  };

  const centerX = 150;
  const centerY = 150;
  const radius = 100;

  const dataPoints = points.map(p => polarToCartesian(p.angle, p.value, centerX, centerY, radius));
  const pathData = `M ${dataPoints[0].x} ${dataPoints[0].y} L ${dataPoints[1].x} ${dataPoints[1].y} L ${dataPoints[2].x} ${dataPoints[2].y} Z`;

  const gridLevels = [20, 40, 60, 80, 100];

  return (
    <Card className="border-emerald-500/30 bg-navy">
      <CardHeader>
        <div className="flex items-center space-x-3">
          <div className="flex h-10 w-10 items-center justify-center rounded-sm bg-emerald-500/10">
            <Server className="h-5 w-5 text-emerald-500" />
          </div>
          <div>
            <CardTitle className="text-emerald-500">Resource Efficiency</CardTitle>
            <p className="text-sm text-muted-foreground">System performance metrics</p>
          </div>
        </div>
      </CardHeader>
      <CardContent>
        <div className="flex items-center justify-center">
          <svg width="300" height="300" viewBox="0 0 300 300">
            {gridLevels.map((level, idx) => {
              const gridPoints = points.map(p =>
                polarToCartesian(p.angle, level, centerX, centerY, radius)
              );
              const gridPath = `M ${gridPoints[0].x} ${gridPoints[0].y} L ${gridPoints[1].x} ${gridPoints[1].y} L ${gridPoints[2].x} ${gridPoints[2].y} Z`;

              return (
                <path
                  key={idx}
                  d={gridPath}
                  fill="none"
                  stroke="currentColor"
                  strokeWidth="1"
                  className="text-border"
                  opacity={0.3}
                />
              );
            })}

            {points.map((point, idx) => {
              const endPoint = polarToCartesian(point.angle, 100, centerX, centerY, radius);
              return (
                <line
                  key={idx}
                  x1={centerX}
                  y1={centerY}
                  x2={endPoint.x}
                  y2={endPoint.y}
                  stroke="currentColor"
                  strokeWidth="1"
                  className="text-border"
                  opacity={0.3}
                />
              );
            })}

            <path
              d={pathData}
              fill="currentColor"
              fillOpacity={0.2}
              stroke="currentColor"
              strokeWidth="2"
              className="text-emerald-500"
            />

            {points.map((point, idx) => {
              const labelPoint = polarToCartesian(point.angle, 110, centerX, centerY, radius);
              return (
                <text
                  key={idx}
                  x={labelPoint.x}
                  y={labelPoint.y}
                  textAnchor="middle"
                  className="fill-current text-xs font-semibold text-foreground"
                >
                  {point.label}
                </text>
              );
            })}
          </svg>
        </div>

        <div className="mt-6 grid grid-cols-3 gap-4">
          <div className="text-center">
            <p className="text-2xl font-bold text-emerald-500">{storage.toFixed(1)}%</p>
            <p className="mt-1 text-xs text-muted-foreground">Storage</p>
          </div>
          <div className="text-center">
            <p className="text-2xl font-bold text-emerald-500">{network.toFixed(1)}%</p>
            <p className="mt-1 text-xs text-muted-foreground">Network</p>
          </div>
          <div className="text-center">
            <p className="text-2xl font-bold text-emerald-500">{compute.toFixed(1)}%</p>
            <p className="mt-1 text-xs text-muted-foreground">Compute</p>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
