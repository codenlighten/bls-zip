'use client';

import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { Chrome as Home, Blocks, Search, FileText, Shield, Activity, Leaf } from 'lucide-react';
import { cn } from '@/lib/utils';

const navigation = [
  { name: 'Dashboard', href: '/', icon: Home },
  { name: 'Blocks', href: '/blocks', icon: Blocks },
  { name: 'Transactions', href: '/transactions', icon: FileText },
  { name: 'Identities', href: '/identities', icon: Shield },
  { name: 'Network Stats', href: '/stats', icon: Activity },
  { name: 'Sustainability', href: '/sustainability', icon: Leaf },
];

export function Sidebar() {
  const pathname = usePathname();

  return (
    <div className="flex h-full w-64 flex-col border-r border-border bg-card">
      <div className="flex h-16 items-center border-b border-border px-6">
        <div className="flex items-center space-x-2">
          <div className="flex h-8 w-8 items-center justify-center rounded-sm bg-primary">
            <span className="text-sm font-bold text-primary-foreground">BLS</span>
          </div>
          <div className="flex flex-col">
            <span className="text-sm font-semibold leading-none">Boundless</span>
            <span className="text-xs text-muted-foreground">Explorer</span>
          </div>
        </div>
      </div>

      <nav className="flex-1 space-y-1 px-3 py-4">
        {navigation.map((item) => {
          const isActive = pathname === item.href;
          return (
            <Link
              key={item.name}
              href={item.href}
              className={cn(
                'flex items-center space-x-3 rounded-sm px-3 py-2 text-sm font-medium transition-colors',
                isActive
                  ? 'bg-primary text-primary-foreground'
                  : 'text-muted-foreground hover:bg-secondary hover:text-foreground'
              )}
            >
              <item.icon className="h-4 w-4" />
              <span>{item.name}</span>
            </Link>
          );
        })}
      </nav>

      <div className="border-t border-border p-4">
        <div className="rounded-sm bg-secondary p-3">
          <div className="flex items-center space-x-2">
            <Shield className="h-4 w-4 text-primary" />
            <span className="text-xs font-medium">Post-Quantum Secure</span>
          </div>
          <p className="mt-1 text-xs text-muted-foreground">
            ML-DSA & Falcon-512 signatures
          </p>
        </div>
      </div>
    </div>
  );
}
