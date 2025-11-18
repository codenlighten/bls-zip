'use client';

import { useEffect, useState } from 'react';
import { Line, LineChart, ResponsiveContainer, Tooltip, XAxis, YAxis, CartesianGrid } from 'recharts';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { TrendingUp, TrendingDown } from 'lucide-react';

interface ChartDataPoint {
  height: number;
  timestamp: string;
  txCount: number;
}

interface NetworkChartProps {
  blocks: any[];
}

export function NetworkChart({ blocks }: NetworkChartProps) {
  const [chartData, setChartData] = useState<ChartDataPoint[]>([]);
  const [trend, setTrend] = useState<'up' | 'down' | 'stable'>('stable');

  useEffect(() => {
    if (blocks && blocks.length > 0) {
      // Convert blocks to chart data
      const data = blocks
        .slice(0, 20)
        .reverse()
        .map((block) => ({
          height: block.height,
          timestamp: new Date(block.timestamp).toLocaleTimeString('en-US', {
            hour: '2-digit',
            minute: '2-digit',
          }),
          txCount: block.tx_count || 0,
        }));

      setChartData(data);

      // Calculate trend
      if (data.length >= 2) {
        const recent = data.slice(-5).reduce((sum, d) => sum + d.txCount, 0) / 5;
        const older = data.slice(0, 5).reduce((sum, d) => sum + d.txCount, 0) / 5;

        if (recent > older * 1.1) {
          setTrend('up');
        } else if (recent < older * 0.9) {
          setTrend('down');
        } else {
          setTrend('stable');
        }
      }
    }
  }, [blocks]);

  const avgTxPerBlock = chartData.length > 0
    ? (chartData.reduce((sum, d) => sum + d.txCount, 0) / chartData.length).toFixed(1)
    : '0';

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <div>
            <CardTitle>Network Activity</CardTitle>
            <CardDescription>Transactions per block over last 20 blocks</CardDescription>
          </div>
          <div className="flex items-center gap-2">
            {trend === 'up' && (
              <div className="flex items-center gap-1 text-green-500">
                <TrendingUp className="h-4 w-4" />
                <span className="text-sm font-medium">Increasing</span>
              </div>
            )}
            {trend === 'down' && (
              <div className="flex items-center gap-1 text-red-500">
                <TrendingDown className="h-4 w-4" />
                <span className="text-sm font-medium">Decreasing</span>
              </div>
            )}
            {trend === 'stable' && (
              <div className="flex items-center gap-1 text-muted-foreground">
                <span className="text-sm font-medium">Stable</span>
              </div>
            )}
          </div>
        </div>
      </CardHeader>
      <CardContent>
        <div className="mb-4">
          <div className="text-2xl font-bold">{avgTxPerBlock}</div>
          <p className="text-xs text-muted-foreground">Average transactions per block</p>
        </div>

        {chartData.length > 0 ? (
          <ResponsiveContainer width="100%" height={200}>
            <LineChart data={chartData}>
              <CartesianGrid strokeDasharray="3 3" className="stroke-muted" />
              <XAxis
                dataKey="timestamp"
                fontSize={12}
                tickLine={false}
                axisLine={false}
                className="text-muted-foreground"
              />
              <YAxis
                fontSize={12}
                tickLine={false}
                axisLine={false}
                className="text-muted-foreground"
              />
              <Tooltip
                contentStyle={{
                  backgroundColor: 'hsl(var(--popover))',
                  border: '1px solid hsl(var(--border))',
                  borderRadius: '6px',
                }}
                labelStyle={{ color: 'hsl(var(--popover-foreground))' }}
              />
              <Line
                type="monotone"
                dataKey="txCount"
                stroke="hsl(var(--primary))"
                strokeWidth={2}
                dot={{ fill: 'hsl(var(--primary))', r: 4 }}
                activeDot={{ r: 6 }}
              />
            </LineChart>
          </ResponsiveContainer>
        ) : (
          <div className="h-[200px] flex items-center justify-center text-muted-foreground">
            <p>No data available</p>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
