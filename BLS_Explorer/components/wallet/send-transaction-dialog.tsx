'use client';

import { useState, useEffect } from 'react';
import { e2Client, E2Wallet } from '@/lib/e2/client';
import { getBlockchainWebSocket } from '@/lib/blockchain/websocket';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Input } from '@/components/ui/input';
import { Button } from '@/components/ui/button';
import { Label } from '@/components/ui/label';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Loader2, Send, AlertTriangle, CheckCircle2 } from 'lucide-react';
import { Badge } from '@/components/ui/badge';

interface SendTransactionDialogProps {
  wallet: E2Wallet;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSuccess?: () => void;
}

export function SendTransactionDialog({
  wallet,
  open,
  onOpenChange,
  onSuccess,
}: SendTransactionDialogProps) {
  const [toAddress, setToAddress] = useState('');
  const [amount, setAmount] = useState('');
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [error, setError] = useState('');
  const [success, setSuccess] = useState(false);
  const [txHash, setTxHash] = useState('');
  const [confirmations, setConfirmations] = useState(0);
  const [isConfirmed, setIsConfirmed] = useState(false);

  // Listen for transaction confirmations via WebSocket
  useEffect(() => {
    if (!txHash || !success) {
      return;
    }

    const ws = getBlockchainWebSocket();

    const unsubscribe = ws.onEvent((event) => {
      if (event.type === 'tx_confirmed' && event.data.tx_hash === txHash) {
        const { confirmations: confs } = event.data;
        setConfirmations(confs);

        if (confs >= 1 && !isConfirmed) {
          setIsConfirmed(true);
        }
      }
    });

    return () => {
      unsubscribe();
    };
  }, [txHash, success, isConfirmed]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError('');
    setSuccess(false);
    setTxHash('');

    // Validation
    if (!toAddress.trim()) {
      setError('Please enter a recipient address');
      return;
    }

    const amountNum = parseFloat(amount);
    if (isNaN(amountNum) || amountNum <= 0) {
      setError('Please enter a valid amount');
      return;
    }

    if (amountNum > wallet.balance) {
      setError('Insufficient balance');
      return;
    }

    // Address validation (basic)
    if (toAddress.length < 32) {
      setError('Invalid address format');
      return;
    }

    setIsSubmitting(true);

    try {
      const transaction = await e2Client.sendTransaction(
        wallet.wallet_id,
        toAddress,
        amountNum
      );

      setTxHash(transaction.tx_hash);
      setSuccess(true);

      // Reset form
      setTimeout(() => {
        setToAddress('');
        setAmount('');
        setSuccess(false);
        setTxHash('');
        onOpenChange(false);
        onSuccess?.();
      }, 3000);
    } catch (err: any) {
      setError(err.message || 'Failed to send transaction');
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleClose = () => {
    if (!isSubmitting) {
      setToAddress('');
      setAmount('');
      setError('');
      setSuccess(false);
      setTxHash('');
      setConfirmations(0);
      setIsConfirmed(false);
      onOpenChange(false);
    }
  };

  const estimatedFee = 0.0001; // 0.0001 BLS fee
  const totalAmount = parseFloat(amount) + estimatedFee;

  return (
    <Dialog open={open} onOpenChange={handleClose}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Send BLS Tokens</DialogTitle>
          <DialogDescription>
            Send tokens from {wallet.name} to another address
          </DialogDescription>
        </DialogHeader>

        <form onSubmit={handleSubmit} className="space-y-4">
          {success ? (
            <Alert className="bg-green-500/10 border-green-500/50">
              <CheckCircle2 className="h-4 w-4 text-green-500" />
              <AlertDescription className="space-y-2">
                <div className="flex items-center justify-between">
                  <p className="font-semibold text-green-500">Transaction Sent!</p>
                  {isConfirmed && (
                    <Badge variant="default" className="bg-green-500">
                      Confirmed
                    </Badge>
                  )}
                </div>
                <p className="text-xs font-mono break-all">
                  Tx Hash: {txHash}
                </p>
                {confirmations > 0 ? (
                  <p className="text-xs text-green-500">
                    {confirmations} confirmation{confirmations > 1 ? 's' : ''} received
                  </p>
                ) : (
                  <p className="text-xs text-muted-foreground">
                    Waiting for confirmation...
                  </p>
                )}
                <p className="text-xs text-muted-foreground">
                  Closing in 3 seconds...
                </p>
              </AlertDescription>
            </Alert>
          ) : (
            <>
              {error && (
                <Alert variant="destructive">
                  <AlertTriangle className="h-4 w-4" />
                  <AlertDescription>{error}</AlertDescription>
                </Alert>
              )}

              {/* From Wallet Info */}
              <div className="p-3 bg-secondary rounded-lg">
                <p className="text-xs text-muted-foreground mb-1">From</p>
                <p className="text-sm font-medium">{wallet.name}</p>
                <p className="text-xs text-muted-foreground">
                  Balance: {wallet.balance.toFixed(4)} BLS
                </p>
              </div>

              {/* Recipient Address */}
              <div className="space-y-2">
                <Label htmlFor="toAddress">Recipient Address</Label>
                <Input
                  id="toAddress"
                  type="text"
                  placeholder="Enter BLS address (0x...)"
                  value={toAddress}
                  onChange={(e) => setToAddress(e.target.value)}
                  disabled={isSubmitting}
                  required
                  className="font-mono text-xs"
                />
              </div>

              {/* Amount */}
              <div className="space-y-2">
                <Label htmlFor="amount">Amount (BLS)</Label>
                <div className="relative">
                  <Input
                    id="amount"
                    type="number"
                    step="0.0001"
                    min="0.0001"
                    max={wallet.balance}
                    placeholder="0.0000"
                    value={amount}
                    onChange={(e) => setAmount(e.target.value)}
                    disabled={isSubmitting}
                    required
                    className="pr-16"
                  />
                  <Button
                    type="button"
                    variant="ghost"
                    size="sm"
                    className="absolute right-1 top-1 h-7 text-xs"
                    onClick={() => setAmount((wallet.balance - estimatedFee).toString())}
                    disabled={isSubmitting}
                  >
                    Max
                  </Button>
                </div>
                {amount && !isNaN(parseFloat(amount)) && (
                  <p className="text-xs text-muted-foreground">
                    ≈ ${(parseFloat(amount) * 10).toFixed(2)} USD
                  </p>
                )}
              </div>

              {/* Transaction Summary */}
              {amount && !isNaN(parseFloat(amount)) && (
                <div className="p-3 bg-secondary rounded-lg space-y-2 text-sm">
                  <div className="flex justify-between">
                    <span className="text-muted-foreground">Amount</span>
                    <span className="font-medium">{parseFloat(amount).toFixed(4)} BLS</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-muted-foreground">Network Fee</span>
                    <span className="font-medium">{estimatedFee.toFixed(4)} BLS</span>
                  </div>
                  <div className="h-px bg-border" />
                  <div className="flex justify-between font-semibold">
                    <span>Total</span>
                    <span>{totalAmount.toFixed(4)} BLS</span>
                  </div>
                </div>
              )}

              {/* Warning */}
              <Alert>
                <AlertTriangle className="h-4 w-4" />
                <AlertDescription className="text-xs">
                  Transactions on the Boundless blockchain are irreversible. Please verify
                  the recipient address carefully before sending.
                </AlertDescription>
              </Alert>
            </>
          )}

          <DialogFooter>
            {!success && (
              <>
                <Button
                  type="button"
                  variant="outline"
                  onClick={handleClose}
                  disabled={isSubmitting}
                >
                  Cancel
                </Button>
                <Button type="submit" disabled={isSubmitting || !amount || !toAddress}>
                  {isSubmitting ? (
                    <>
                      <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                      Sending...
                    </>
                  ) : (
                    <>
                      <Send className="mr-2 h-4 w-4" />
                      Send Transaction
                    </>
                  )}
                </Button>
              </>
            )}
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
