'use client';

import { useState } from 'react';
import { SignatureType } from '@/lib/types';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { CheckCircle2, Copy, ChevronDown, ChevronUp, Shield } from 'lucide-react';
import { useToast } from '@/hooks/use-toast';

interface SignatureBadgeProps {
  signatureType: SignatureType;
  sizeBytes: number;
  signatureData?: string;
}

export function SignatureBadge({ signatureType, sizeBytes, signatureData }: SignatureBadgeProps) {
  const [copied, setCopied] = useState(false);
  const [isExpanded, setIsExpanded] = useState(sizeBytes <= 1000);
  const { toast } = useToast();

  const isPostQuantum = signatureType === 'MlDsa' || signatureType === 'Hybrid';

  const handleCopy = () => {
    if (signatureData) {
      navigator.clipboard.writeText(signatureData);
      setCopied(true);
      toast({
        title: 'Copied to clipboard',
        description: 'Signature data copied successfully',
      });
      setTimeout(() => setCopied(false), 2000);
    }
  };

  return (
    <div className="rounded-sm border border-sky-blue bg-navy p-3">
      <div className="flex items-center justify-between">
        <div className="flex flex-1 items-center space-x-3">
          <Badge variant="outline" className="border-sky-blue font-mono text-sky-blue">
            {signatureType}
          </Badge>
          <div className="flex items-center space-x-2 text-sm">
            <span className="text-muted-foreground">Size:</span>
            <span className="font-mono font-medium text-foreground">
              {sizeBytes.toLocaleString()} bytes
            </span>
          </div>
          {isPostQuantum && (
            <div className="flex items-center space-x-2">
              <Shield className="h-4 w-4 text-success" />
              <span className="text-sm font-medium text-success">Post-Quantum Secure</span>
            </div>
          )}
          <div className="flex items-center space-x-2">
            <CheckCircle2 className="h-4 w-4 text-success" />
            <span className="text-sm font-medium text-success">Valid</span>
          </div>
        </div>
        <div className="flex items-center space-x-2">
          {sizeBytes > 1000 && (
            <Button
              size="sm"
              variant="ghost"
              onClick={() => setIsExpanded(!isExpanded)}
              className="h-8"
            >
              {isExpanded ? (
                <ChevronUp className="h-4 w-4" />
              ) : (
                <ChevronDown className="h-4 w-4" />
              )}
            </Button>
          )}
          {signatureData && (
            <Button
              size="sm"
              variant="ghost"
              onClick={handleCopy}
              className="h-8"
            >
              {copied ? (
                <CheckCircle2 className="h-4 w-4" />
              ) : (
                <Copy className="h-4 w-4" />
              )}
            </Button>
          )}
        </div>
      </div>
      {!isExpanded && sizeBytes > 1000 && (
        <div className="mt-2 text-xs text-muted-foreground">
          Signature data collapsed due to size. Click to expand.
        </div>
      )}
    </div>
  );
}
