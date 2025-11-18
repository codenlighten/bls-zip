'use client';

import { useState, useEffect } from 'react';
import Link from 'next/link';
import { Search, User, Wallet, LogOut, Settings, ChevronDown } from 'lucide-react';
import { Input } from '@/components/ui/input';
import { Button } from '@/components/ui/button';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { useRouter } from 'next/navigation';
import { useAuth } from '@/lib/e2/auth-context';
import { getBlockchainWebSocket } from '@/lib/blockchain/websocket';

export function Header() {
  const [searchQuery, setSearchQuery] = useState('');
  const router = useRouter();
  const { user, wallets, isAuthenticated, logout, refreshWallets } = useAuth();

  const handleSearch = (e: React.FormEvent) => {
    e.preventDefault();
    if (!searchQuery.trim()) return;

    const query = searchQuery.trim();

    if (/^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i.test(query)) {
      const lowerQuery = query.toLowerCase();
      if (lowerQuery.startsWith('a') || lowerQuery.startsWith('c')) {
        router.push(`/asset/${query}`);
      } else {
        router.push(`/identity/${query}`);
      }
    } else if (!isNaN(Number(query))) {
      router.push(`/block/${query}`);
    } else if (query.length === 64 && /^[0-9a-f]+$/i.test(query)) {
      router.push(`/tx/${query}`);
    } else {
      router.push(`/search?q=${encodeURIComponent(query)}`);
    }

    setSearchQuery('');
  };

  const handleLogout = async () => {
    await logout();
    router.push('/login');
  };

  const primaryWallet = wallets[0]; // First wallet as primary

  // Listen for transaction confirmations to refresh balance
  useEffect(() => {
    if (!isAuthenticated || wallets.length === 0) {
      return;
    }

    const ws = getBlockchainWebSocket();

    const unsubscribe = ws.onEvent((event) => {
      if (event.type === 'tx_confirmed') {
        // Transaction confirmed - refresh wallet balances
        console.log('Transaction confirmed, refreshing wallet balances');
        refreshWallets();
      } else if (event.type === 'new_transaction') {
        // New transaction detected - check if it involves our wallets
        const walletAddresses = wallets.map((w) => w.address);
        if (
          walletAddresses.includes(event.data.from) ||
          walletAddresses.includes(event.data.to)
        ) {
          console.log('Transaction involving our wallet detected');
          refreshWallets();
        }
      }
    });

    return () => {
      unsubscribe();
    };
  }, [isAuthenticated, wallets, refreshWallets]);

  return (
    <header className="sticky top-0 z-10 flex h-16 items-center border-b border-border bg-card px-6">
      <form onSubmit={handleSearch} className="flex-1">
        <div className="relative max-w-md">
          <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
          <Input
            type="text"
            placeholder="Search by Block, Tx Hash, Identity/Asset UUID, or Address..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="pl-10"
          />
        </div>
      </form>

      {/* Auth Section */}
      <div className="ml-4 flex items-center gap-4">
        {isAuthenticated && user ? (
          <>
            {/* Wallet Info */}
            {primaryWallet && (
              <div className="hidden md:flex items-center gap-2 px-3 py-1.5 bg-primary/10 rounded-md">
                <Wallet className="h-4 w-4 text-primary" />
                <div className="flex flex-col">
                  <span className="text-xs font-medium">
                    {primaryWallet.balance.toFixed(4)} BLS
                  </span>
                  <span className="text-[10px] text-muted-foreground">
                    {primaryWallet.address.substring(0, 8)}...
                  </span>
                </div>
              </div>
            )}

            {/* User Menu */}
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <Button variant="ghost" className="flex items-center gap-2">
                  <div className="h-8 w-8 rounded-full bg-primary/20 flex items-center justify-center">
                    <User className="h-4 w-4 text-primary" />
                  </div>
                  <span className="hidden md:inline text-sm">
                    {user.first_name} {user.last_name}
                  </span>
                  <ChevronDown className="h-4 w-4" />
                </Button>
              </DropdownMenuTrigger>

              <DropdownMenuContent align="end" className="w-56">
                <DropdownMenuLabel>
                  <div className="flex flex-col space-y-1">
                    <p className="text-sm font-medium">
                      {user.first_name} {user.last_name}
                    </p>
                    <p className="text-xs text-muted-foreground">{user.email}</p>
                  </div>
                </DropdownMenuLabel>

                <DropdownMenuSeparator />

                <DropdownMenuItem asChild>
                  <Link href="/wallet" className="cursor-pointer">
                    <Wallet className="mr-2 h-4 w-4" />
                    My Wallets
                  </Link>
                </DropdownMenuItem>

                <DropdownMenuItem asChild>
                  <Link href={`/identity/${user.identity_id}`} className="cursor-pointer">
                    <User className="mr-2 h-4 w-4" />
                    My Identity
                  </Link>
                </DropdownMenuItem>

                <DropdownMenuItem asChild>
                  <Link href="/settings" className="cursor-pointer">
                    <Settings className="mr-2 h-4 w-4" />
                    Settings
                  </Link>
                </DropdownMenuItem>

                <DropdownMenuSeparator />

                <DropdownMenuItem onClick={handleLogout} className="cursor-pointer text-red-500">
                  <LogOut className="mr-2 h-4 w-4" />
                  Logout
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
          </>
        ) : (
          <>
            {/* Not authenticated */}
            <Link href="/login">
              <Button variant="ghost" size="sm">
                Sign In
              </Button>
            </Link>
            <Link href="/signup">
              <Button size="sm">
                Sign Up
              </Button>
            </Link>
          </>
        )}
      </div>
    </header>
  );
}
