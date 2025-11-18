import { Card, CardContent } from '@/components/ui/card';

interface NetworkGradeRingProps {
  grade: 'A+' | 'A' | 'B' | 'C' | 'D';
  energyPerTxWh: number;
}

export function NetworkGradeRing({ grade, energyPerTxWh }: NetworkGradeRingProps) {
  const gradeColors = {
    'A+': { ring: 'stroke-emerald-500', bg: 'bg-emerald-500/10', text: 'text-emerald-500' },
    'A': { ring: 'stroke-emerald-400', bg: 'bg-emerald-400/10', text: 'text-emerald-400' },
    'B': { ring: 'stroke-blue-400', bg: 'bg-blue-400/10', text: 'text-blue-400' },
    'C': { ring: 'stroke-yellow-400', bg: 'bg-yellow-400/10', text: 'text-yellow-400' },
    'D': { ring: 'stroke-red-400', bg: 'bg-red-400/10', text: 'text-red-400' },
  };

  const colors = gradeColors[grade];
  const circumference = 2 * Math.PI * 90;
  const gradeValues = { 'A+': 95, 'A': 85, 'B': 70, 'C': 50, 'D': 30 };
  const percentage = gradeValues[grade];
  const offset = circumference - (percentage / 100) * circumference;

  return (
    <Card className="border-emerald-500/20 bg-card">
      <CardContent className="flex flex-col items-center justify-center p-8">
        <div className="relative">
          <svg width="200" height="200" className="transform -rotate-90">
            <circle
              cx="100"
              cy="100"
              r="90"
              className="stroke-secondary"
              strokeWidth="12"
              fill="none"
            />
            <circle
              cx="100"
              cy="100"
              r="90"
              className={colors.ring}
              strokeWidth="12"
              fill="none"
              strokeDasharray={circumference}
              strokeDashoffset={offset}
              strokeLinecap="round"
              style={{ transition: 'stroke-dashoffset 1s ease-in-out' }}
            />
          </svg>
          <div className="absolute inset-0 flex flex-col items-center justify-center">
            <div className={`text-5xl font-bold ${colors.text}`}>{grade}</div>
            <div className="text-xs text-muted-foreground mt-1">Network Grade</div>
          </div>
        </div>
        <div className="mt-6 text-center">
          <div className="text-2xl font-bold text-foreground">
            {energyPerTxWh.toFixed(2)} <span className="text-sm font-normal text-muted-foreground">Wh/Tx</span>
          </div>
          <div className="text-xs text-muted-foreground mt-1">Energy Intensity</div>
        </div>
        <div className={`mt-4 rounded-full ${colors.bg} px-4 py-2`}>
          <div className={`text-xs font-semibold ${colors.text}`}>
            {grade === 'A+' ? 'Exceptional Efficiency' : grade === 'A' ? 'High Efficiency' : grade === 'B' ? 'Good Efficiency' : 'Standard Efficiency'}
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
