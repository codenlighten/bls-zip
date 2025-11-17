'use client'

import { useState, useEffect } from 'react'
import { api } from '@/lib/api'
import type { IdentityProfile, WalletAccount, WalletBalance, WalletTransaction } from '@/types'
import {
  CreditCardIcon,
  ShieldCheckIcon,
  ChartBarIcon,
  BanknotesIcon,
  ArrowTrendingUpIcon,
  ArrowTrendingDownIcon,
  CheckCircleIcon,
  ClockIcon,
  XCircleIcon,
} from '@heroicons/react/24/outline'

export default function DashboardPage() {
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState('')
  const [identity, setIdentity] = useState<IdentityProfile | null>(null)
  const [wallets, setWallets] = useState<WalletAccount[]>([])
  const [allBalances, setAllBalances] = useState<WalletBalance[]>([])
  const [recentTransactions, setRecentTransactions] = useState<WalletTransaction[]>([])

  useEffect(() => {
    loadDashboardData()
  }, [])

  const loadDashboardData = async () => {
    try {
      // Load user identity from localStorage
      const userStr = localStorage.getItem('user_identity')
      if (!userStr) {
        setError('No user session found')
        setLoading(false)
        return
      }

      const user: IdentityProfile = JSON.parse(userStr)
      setIdentity(user)

      // Fetch all wallets for this identity
      const walletsResponse = await api.getIdentityWallets(user.identity_id)
      if (walletsResponse.error) {
        console.error('Error loading wallets:', walletsResponse.error)
        setWallets([])
      } else if (walletsResponse.data) {
        setWallets(walletsResponse.data)

        // Fetch balances for all wallets
        const balancesPromises = walletsResponse.data.map(wallet =>
          api.getWalletBalances(wallet.wallet_id)
        )
        const balancesResponses = await Promise.all(balancesPromises)
        const allBals = balancesResponses
          .filter(res => res.data)
          .flatMap(res => res.data!)
        setAllBalances(allBals)

        // Fetch recent transactions from all wallets (limit 5 per wallet)
        const txPromises = walletsResponse.data.map(wallet =>
          api.getWalletTransactions(wallet.wallet_id, 5, 0)
        )
        const txResponses = await Promise.all(txPromises)
        const allTxs = txResponses
          .filter(res => res.data)
          .flatMap(res => res.data!)
          // Sort by timestamp descending
          .sort((a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime())
          // Take top 5
          .slice(0, 5)
        setRecentTransactions(allTxs)
      }

      setLoading(false)
    } catch (err) {
      console.error('Error loading dashboard:', err)
      setError('Failed to load dashboard data')
      setLoading(false)
    }
  }

  // Calculate stats
  const totalAssetsValue = allBalances.reduce((sum, bal) => sum + bal.amount, 0)
  const activeWallets = wallets.length
  const pendingTransactions = recentTransactions.filter(tx => tx.status === 'pending').length
  const kycStatus = identity?.kyc_status || 'pending'

  const getKycStatusBadge = () => {
    const statusConfig = {
      verified: { color: 'text-green-400', icon: CheckCircleIcon, text: 'Verified' },
      pending: { color: 'text-yellow-400', icon: ClockIcon, text: 'Pending' },
      rejected: { color: 'text-red-400', icon: XCircleIcon, text: 'Rejected' },
      revoked: { color: 'text-red-400', icon: XCircleIcon, text: 'Revoked' },
    }
    return statusConfig[kycStatus] || statusConfig.pending
  }

  const formatAssetType = (assetType: any): string => {
    if (typeof assetType === 'string') {
      return assetType.replace(/_/g, ' ').toUpperCase()
    }
    if (typeof assetType === 'object' && assetType.custom) {
      return assetType.custom
    }
    return 'UNKNOWN'
  }

  const formatAmount = (amount: number, assetType: any): string => {
    const type = formatAssetType(assetType)
    if (type === 'NATIVE') {
      return `${(amount / 1e8).toFixed(4)} BLS`
    }
    return `${amount.toLocaleString()} ${type}`
  }

  const formatTimestamp = (timestamp: string): string => {
    const date = new Date(timestamp)
    return date.toLocaleString('en-US', {
      month: 'short',
      day: 'numeric',
      hour: 'numeric',
      minute: '2-digit',
      hour12: true
    })
  }

  const formatOperationType = (operation_type?: any): string => {
    if (!operation_type) return 'Transfer'

    if (typeof operation_type === 'string') {
      return operation_type
        .replace(/_/g, ' ')
        .split(' ')
        .map(w => w.charAt(0).toUpperCase() + w.slice(1))
        .join(' ')
    }
    if (typeof operation_type === 'object' && operation_type.custom) {
      return operation_type.custom
    }
    return 'Transaction'
  }

  const statCards = [
    {
      name: 'Total Assets',
      value: totalAssetsValue > 0 ? totalAssetsValue.toLocaleString() : '0',
      change: '+12.5%',
      changeType: 'positive',
      icon: BanknotesIcon,
    },
    {
      name: 'Active Wallets',
      value: activeWallets.toString(),
      change: activeWallets > 0 ? `+${activeWallets}` : '0',
      changeType: 'positive',
      icon: CreditCardIcon,
    },
    {
      name: 'Pending TX',
      value: pendingTransactions.toString(),
      change: pendingTransactions > 0 ? `${pendingTransactions}` : 'None',
      changeType: pendingTransactions > 0 ? 'negative' : 'positive',
      icon: ChartBarIcon,
    },
    {
      name: 'KYC Status',
      value: getKycStatusBadge().text,
      change: getKycStatusBadge().text,
      changeType: kycStatus === 'verified' ? 'positive' : 'negative',
      icon: ShieldCheckIcon,
    },
  ]

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="flex flex-col items-center gap-4">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-primary-500"></div>
          <div className="text-slate-400">Loading dashboard...</div>
        </div>
      </div>
    )
  }

  if (error) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="text-center">
          <XCircleIcon className="w-12 h-12 text-red-400 mx-auto mb-4" />
          <div className="text-red-400">{error}</div>
        </div>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      {/* Page Header */}
      <div>
        <h1 className="text-3xl font-bold text-white">Dashboard</h1>
        <p className="text-slate-400 mt-1">
          Welcome back, {identity?.legal_name}! Here's an overview of your enterprise account.
        </p>
      </div>

      {/* Stats Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        {statCards.map((stat) => (
          <div key={stat.name} className="card">
            <div className="flex items-center justify-between mb-4">
              <div className="p-2 bg-primary-900/30 rounded-lg">
                <stat.icon className="w-6 h-6 text-primary-400" />
              </div>
              <div
                className={`flex items-center gap-1 text-sm ${
                  stat.changeType === 'positive'
                    ? 'text-green-400'
                    : 'text-yellow-400'
                }`}
              >
                {stat.changeType === 'positive' ? (
                  <ArrowTrendingUpIcon className="w-4 h-4" />
                ) : (
                  <ArrowTrendingDownIcon className="w-4 h-4" />
                )}
                {stat.change}
              </div>
            </div>
            <div className="text-2xl font-bold text-white">{stat.value}</div>
            <div className="text-sm text-slate-400 mt-1">{stat.name}</div>
          </div>
        ))}
      </div>

      {/* Recent Transactions */}
      <div className="card">
        <div className="flex items-center justify-between mb-6">
          <h2 className="text-xl font-semibold text-white">
            Recent Transactions
          </h2>
          <a
            href="/wallet"
            className="text-primary-400 hover:text-primary-300 text-sm font-medium"
          >
            View all →
          </a>
        </div>

        {recentTransactions.length === 0 ? (
          <div className="text-center py-8 text-slate-400">
            No transactions yet. Start by creating a wallet and making your first transaction.
          </div>
        ) : (
          <div className="space-y-3">
            {recentTransactions.map((tx) => (
              <div
                key={tx.tx_id}
                className="flex items-center justify-between p-4 bg-slate-900/50 rounded-lg border border-slate-700"
              >
                <div className="flex items-center gap-4">
                  <div
                    className={`w-10 h-10 rounded-full flex items-center justify-center ${
                      tx.direction === 'in'
                        ? 'bg-green-900/30 text-green-400'
                        : 'bg-red-900/30 text-red-400'
                    }`}
                  >
                    {tx.direction === 'in' ? '↓' : '↑'}
                  </div>
                  <div>
                    <div className="font-medium text-white">
                      {formatOperationType(tx.operation_type)}
                    </div>
                    <div className="text-sm text-slate-400">
                      {tx.direction === 'in' ? 'Received' : 'Sent'} • {formatTimestamp(tx.timestamp)}
                    </div>
                    {tx.metadata?.description && (
                      <div className="text-xs text-slate-500 mt-1">{tx.metadata.description}</div>
                    )}
                  </div>
                </div>
                <div className="text-right">
                  <div className={`font-medium ${tx.direction === 'in' ? 'text-green-400' : 'text-white'}`}>
                    {tx.direction === 'in' ? '+' : '-'}{formatAmount(tx.amount, tx.asset_type)}
                  </div>
                  <div className="text-sm">
                    <span
                      className={`badge ${
                        tx.status === 'confirmed'
                          ? 'badge-success'
                          : tx.status === 'pending'
                          ? 'badge-warning'
                          : 'badge-error'
                      }`}
                    >
                      {tx.status}
                    </span>
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Quick Actions */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        <a
          href="/wallet"
          className="card hover:border-primary-500 transition-colors cursor-pointer"
        >
          <CreditCardIcon className="w-8 h-8 text-primary-400 mb-3" />
          <h3 className="font-semibold text-white mb-1">Manage Wallets</h3>
          <p className="text-sm text-slate-400">
            View balances and transaction history
          </p>
        </a>

        <a
          href="/trading"
          className="card hover:border-primary-500 transition-colors cursor-pointer"
        >
          <ChartBarIcon className="w-8 h-8 text-primary-400 mb-3" />
          <h3 className="font-semibold text-white mb-1">Trade Assets</h3>
          <p className="text-sm text-slate-400">
            Access the internal marketplace
          </p>
        </a>

        <a
          href="/identity"
          className="card hover:border-primary-500 transition-colors cursor-pointer"
        >
          <ShieldCheckIcon className="w-8 h-8 text-primary-400 mb-3" />
          <h3 className="font-semibold text-white mb-1">Identity & KYC</h3>
          <p className="text-sm text-slate-400">
            Manage identity and attestations
          </p>
        </a>
      </div>
    </div>
  )
}
