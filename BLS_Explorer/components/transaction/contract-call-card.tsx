import { ContractCall } from '@/lib/types';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Code, Zap } from 'lucide-react';

interface ContractCallCardProps {
  contractCall: ContractCall;
}

export function ContractCallCard({ contractCall }: ContractCallCardProps) {
  return (
    <Card className="border-sky-blue bg-navy">
      <CardHeader className="border-b border-sky-blue/30">
        <div className="flex items-center space-x-3">
          <div className="flex h-10 w-10 items-center justify-center rounded-sm bg-sky-blue">
            <Zap className="h-5 w-5 text-navy" />
          </div>
          <div className="flex-1">
            <CardTitle className="text-sky-blue">Contract Execution</CardTitle>
            <p className="text-sm text-muted-foreground">Named Function Call</p>
          </div>
          <Badge variant="outline" className="border-sky-blue text-sky-blue">
            ABI Decoded
          </Badge>
        </div>
      </CardHeader>
      <CardContent className="space-y-4 pt-6">
        <div className="rounded-sm border border-sky-blue/30 bg-background/50 p-4">
          <div className="mb-2 flex items-center space-x-2">
            <Code className="h-4 w-4 text-sky-blue" />
            <p className="text-xs font-semibold uppercase tracking-wide text-sky-blue">
              Function Name
            </p>
          </div>
          <p className="font-mono text-lg font-bold text-gold">
            {contractCall.function_name}()
          </p>
        </div>

        <div className="space-y-2">
          <p className="text-xs font-semibold uppercase tracking-wide text-sky-blue">
            Parameters
          </p>
          <div className="rounded-sm border border-sky-blue/30 bg-background/50 p-4">
            <pre className="overflow-auto font-mono text-sm text-foreground">
              {JSON.stringify(contractCall.args, null, 2)}
            </pre>
          </div>
        </div>

        <div className="rounded-sm bg-sky-blue/10 p-4">
          <p className="text-sm text-muted-foreground">
            This transaction executes a smart contract function using Boundless BLS
            Named ABI encoding. The function name and arguments are decoded from the
            transaction data field.
          </p>
        </div>
      </CardContent>
    </Card>
  );
}
