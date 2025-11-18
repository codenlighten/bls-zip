import { Card, CardContent } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Shield } from 'lucide-react';

interface NetworkGradeCardProps {
  grade: 'A+' | 'A' | 'B' | 'C' | 'D';
  score: number;
}

function getGradeColor(grade: string): { bg: string; text: string; ring: string } {
  switch (grade) {
    case 'A+':
    case 'A':
      return { bg: 'bg-emerald-500', text: 'text-emerald-500', ring: 'stroke-emerald-500' };
    case 'B':
      return { bg: 'bg-blue-500', text: 'text-blue-500', ring: 'stroke-blue-500' };
    case 'C':
      return { bg: 'bg-yellow-500', text: 'text-yellow-500', ring: 'stroke-yellow-500' };
    default:
      return { bg: 'bg-red-500', text: 'text-red-500', ring: 'stroke-red-500' };
  }
}

export function NetworkGradeCard({ grade, score }: NetworkGradeCardProps) {
  const colors = getGradeColor(grade);
  const circumference = 2 * Math.PI * 90;
  const offset = circumference - (score / 100) * circumference;

  return (
    <Card className="border-emerald-500/50 bg-navy">
      <CardContent className="flex flex-col items-center justify-center p-8">
        <div className="relative">
          <svg className="h-64 w-64 -rotate-90 transform">
            <circle
              cx="128"
              cy="128"
              r="90"
              stroke="currentColor"
              strokeWidth="16"
              fill="none"
              className="text-secondary"
            />
            <circle
              cx="128"
              cy="128"
              r="90"
              stroke="currentColor"
              strokeWidth="16"
              fill="none"
              className={colors.ring}
              strokeDasharray={circumference}
              strokeDashoffset={offset}
              strokeLinecap="round"
            />
          </svg>
          <div className="absolute inset-0 flex flex-col items-center justify-center">
            <p className={`text-8xl font-bold ${colors.text}`}>{grade}</p>
            <p className="mt-2 text-2xl font-semibold text-muted-foreground">{score.toFixed(1)}/100</p>
          </div>
        </div>

        <div className="mt-6 text-center">
          <h3 className="text-2xl font-bold text-emerald-500">Network Grade</h3>
          <p className="mt-2 text-sm text-muted-foreground">Post-Quantum Efficiency Rating</p>
        </div>

        <div className="mt-6">
          <Badge className="border-emerald-500 bg-emerald-500/10 px-4 py-2 text-emerald-500">
            <Shield className="mr-2 h-4 w-4" />
            Verified by Boundless Enterprise
          </Badge>
        </div>
      </CardContent>
    </Card>
  );
}
