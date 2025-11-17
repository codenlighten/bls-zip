'use client'

import { useState, useEffect } from 'react'
import { api } from '@/lib/api'
import type { IdentityProfile, WalletAccount, WalletBalance, WalletTransaction, AssetType, ConnectedApp, Application, AppActivity } from '@/types'
import {
  WalletIcon,
  ArrowUpIcon,
  ArrowDownIcon,
  PlusIcon,
  PaperAirplaneIcon,
  ArrowPathIcon,
  XMarkIcon,
  CheckCircleIcon,
  XCircleIcon,
  RectangleGroupIcon,
  DocumentCheckIcon,
  TicketIcon,
  DocumentTextIcon,
  IdentificationIcon,
  BellAlertIcon,
  LinkIcon,
  ArrowTopRightOnSquareIcon,
} from '@heroicons/react/24/outline'

export default function WalletPage() {
  const [identity, setIdentity] = useState<IdentityProfile | null>(null)
  const [wallets, setWallets] = useState<WalletAccount[]>([])
  const [allBalances, setAllBalances] = useState<WalletBalance[]>([])
  const [allTransactions, setAllTransactions] = useState<WalletTransaction[]>([])
  const [loading, setLoading] = useState(true)
  const [syncing, setSyncing] = useState(false)
  const [error, setError] = useState('')

  // Applications
  const [connectedApps, setConnectedApps] = useState<ConnectedApp[]>([])
  const [availableApps, setAvailableApps] = useState<Application[]>([])
  const [appActivities, setAppActivities] = useState<Map<string, AppActivity[]>>(new Map())
  const [selectedApp, setSelectedApp] = useState<ConnectedApp | null>(null)

  // Modals
  const [showCreateWalletModal, setShowCreateWalletModal] = useState(false)
  const [showTransferModal, setShowTransferModal] = useState(false)
  const [showConnectAppModal, setShowConnectAppModal] = useState(false)
  const [showAppDashboard, setShowAppDashboard] = useState(false)

  // Transfer form
  const [selectedWallet, setSelectedWallet] = useState<string>('')
  const [transferTo, setTransferTo] = useState('')
  const [transferAsset, setTransferAsset] = useState<AssetType>('native')
  const [transferAmount, setTransferAmount] = useState('')
  const [transferError, setTransferError] = useState('')
  const [transferring, setTransferring] = useState(false)

  // Create wallet form
  const [walletLabel, setWalletLabel] = useState('')
  const [creating, setCreating] = useState(false)

  useEffect(() => {
    loadWalletData()
  }, [])

  const loadWalletData = async () => {
    try {
      setLoading(true)
      setError('')

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
        setError('Failed to load wallets')
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

        // Fetch transactions from all wallets (limit 20 per wallet)
        const txPromises = walletsResponse.data.map(wallet =>
          api.getWalletTransactions(wallet.wallet_id, 20, 0)
        )
        const txResponses = await Promise.all(txPromises)
        const allTxs = txResponses
          .filter(res => res.data)
          .flatMap(res => res.data!)
          // Sort by timestamp descending
          .sort((a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime())
        setAllTransactions(allTxs)
      }

      // Fetch connected applications
      const appsResponse = await api.getConnectedApps(user.identity_id)
      if (appsResponse.data) {
        setConnectedApps(appsResponse.data)

        // Fetch recent activity for each app
        const activitiesMap = new Map<string, AppActivity[]>()
        for (const conn of appsResponse.data) {
          const activityResponse = await api.getAppActivity(conn.connection_id, 5, 0)
          if (activityResponse.data) {
            activitiesMap.set(conn.connection_id, activityResponse.data)
          }
        }
        setAppActivities(activitiesMap)
      }

      // Fetch available apps from registry
      const availableResponse = await api.getAvailableApps()
      if (availableResponse.data) {
        setAvailableApps(availableResponse.data)
      }

      setLoading(false)
    } catch (err) {
      console.error('Error loading wallet data:', err)
      setError('Failed to load wallet data')
      setLoading(false)
    }
  }

  const handleSyncBalances = async () => {
    if (wallets.length === 0) return

    setSyncing(true)
    try {
      // Sync all wallets with the blockchain
      const syncPromises = wallets.map(wallet =>
        api.syncWalletBalances(wallet.wallet_id)
      )
      await Promise.all(syncPromises)

      // Reload wallet data
      await loadWalletData()
    } catch (err) {
      console.error('Error syncing balances:', err)
      setError('Failed to sync with blockchain')
    }
    setSyncing(false)
  }

  const handleCreateWallet = async () => {
    if (!identity) return

    setCreating(true)
    try {
      const labels = walletLabel ? [walletLabel] : ['Main Wallet']
      const response = await api.createWallet(identity.identity_id, labels)

      if (response.error) {
        setError(response.error)
      } else {
        setShowCreateWalletModal(false)
        setWalletLabel('')
        // Reload wallets
        await loadWalletData()
      }
    } catch (err) {
      console.error('Error creating wallet:', err)
      setError('Failed to create wallet')
    }
    setCreating(false)
  }

  const handleTransfer = async () => {
    if (!selectedWallet || !transferTo || !transferAmount) {
      setTransferError('Please fill in all fields')
      return
    }

    setTransferring(true)
    setTransferError('')

    try {
      const amount = parseFloat(transferAmount)
      if (isNaN(amount) || amount <= 0) {
        setTransferError('Invalid amount')
        setTransferring(false)
        return
      }

      const response = await api.transfer(
        selectedWallet,
        transferTo,
        transferAsset,
        Math.floor(amount * 1e8) // Convert to satoshis
      )

      if (response.error) {
        setTransferError(response.error)
      } else {
        setShowTransferModal(false)
        setTransferTo('')
        setTransferAmount('')
        setTransferError('')
        // Reload transactions
        await loadWalletData()
      }
    } catch (err) {
      console.error('Error transferring:', err)
      setTransferError('Failed to send transaction')
    }
    setTransferring(false)
  }

  const handleConnectApp = async (app: Application) => {
    if (!identity) return

    try {
      const response = await api.connectApp({
        identity_id: identity.identity_id,
        app_id: app.app_id,
        granted_scopes: app.requested_scopes, // Grant all requested scopes
      })

      if (!response.error) {
        setShowConnectAppModal(false)
        await loadWalletData() // Reload to show new connection
      }
    } catch (err) {
      console.error('Error connecting app:', err)
    }
  }

  const handleRevokeApp = async (connection_id: string) => {
    try {
      await api.revokeAppConnection(connection_id)
      await loadWalletData() // Reload after revoking
    } catch (err) {
      console.error('Error revoking app:', err)
    }
  }

  const getAppIcon = (category: any) => {
    if (typeof category === 'string') {
      switch (category) {
        case 'document_verification':
          return DocumentCheckIcon
        case 'ticketing':
          return TicketIcon
        case 'invoicing':
          return DocumentTextIcon
        case 'credentials':
          return IdentificationIcon
        default:
          return RectangleGroupIcon
      }
    }
    return RectangleGroupIcon
  }

  const formatCategoryName = (category: any): string => {
    if (typeof category === 'string') {
      return category.replace(/_/g, ' ').split(' ').map(w => w.charAt(0).toUpperCase() + w.slice(1)).join(' ')
    }
    if (typeof category === 'object' && category.custom) {
      return category.custom
    }
    return 'Unknown'
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
      return `${(amount / 1e8).toFixed(8)} BLS`
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

  // Calculate total portfolio value (sum of all balances)
  const totalPortfolioValue = allBalances.reduce((sum, bal) => sum + bal.amount, 0)

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="flex flex-col items-center gap-4">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-primary-500"></div>
          <div className="text-slate-400">Loading wallet data from blockchain...</div>
        </div>
      </div>
    )
  }

  if (error && wallets.length === 0) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="text-center">
          <XCircleIcon className="w-12 h-12 text-red-400 mx-auto mb-4" />
          <div className="text-red-400 mb-4">{error}</div>
          <button onClick={loadWalletData} className="btn btn-primary">
            Retry
          </button>
        </div>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      {/* Page Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold text-white">Assets & Apps</h1>
          <p className="text-slate-400 mt-1">
            Multi-asset, application-aware wallet for your E² Multipass
          </p>
        </div>
        <div className="flex gap-3">
          <button
            onClick={handleSyncBalances}
            disabled={syncing || wallets.length === 0}
            className="btn btn-secondary flex items-center gap-2"
          >
            <ArrowPathIcon className={`w-5 h-5 ${syncing ? 'animate-spin' : ''}`} />
            {syncing ? 'Syncing...' : 'Sync with Blockchain'}
          </button>
          <button
            onClick={() => setShowCreateWalletModal(true)}
            className="btn btn-secondary flex items-center gap-2"
          >
            <PlusIcon className="w-5 h-5" />
            New Wallet
          </button>
          <button
            onClick={() => {
              if (wallets.length > 0) {
                setSelectedWallet(wallets[0].wallet_id)
                setShowTransferModal(true)
              }
            }}
            disabled={wallets.length === 0}
            className="btn btn-primary flex items-center gap-2"
          >
            <PaperAirplaneIcon className="w-5 h-5" />
            Send
          </button>
        </div>
      </div>

      {/* No Wallets State */}
      {wallets.length === 0 && (
        <div className="card text-center py-12">
          <WalletIcon className="w-16 h-16 mx-auto mb-4 text-slate-600" />
          <h3 className="text-xl font-semibold text-white mb-2">No Asset Wallets Yet</h3>
          <p className="text-slate-400 mb-6">
            Create your first wallet to manage multi-asset holdings and connect with applications
          </p>
          <button
            onClick={() => setShowCreateWalletModal(true)}
            className="btn btn-primary mx-auto"
          >
            <PlusIcon className="w-5 h-5 inline mr-2" />
            Create Wallet
          </button>
        </div>
      )}

      {/* Total Value Card */}
      {wallets.length > 0 && (
        <div className="card bg-gradient-to-br from-primary-600 via-primary-700 to-secondary-700 border-primary-500/30">
          <div className="text-primary-100 mb-2">Total Portfolio Value</div>
          <div className="text-4xl font-bold text-white mb-4">
            {totalPortfolioValue > 0 ? totalPortfolioValue.toLocaleString() : '0'} BLS
          </div>
          <div className="flex items-center gap-4 text-sm">
            <span className="text-primary-200">{wallets.length} Wallet{wallets.length !== 1 ? 's' : ''}</span>
            <span className="text-primary-200">•</span>
            <span className="text-primary-200">{allBalances.length} Asset{allBalances.length !== 1 ? 's' : ''}</span>
          </div>
        </div>
      )}

      {/* Wallets & Balances */}
      {wallets.length > 0 && (
        <div className="space-y-6">
          {wallets.map(wallet => {
            const walletBalances = allBalances.filter(b => b.wallet_id === wallet.wallet_id)
            const walletTotal = walletBalances.reduce((sum, b) => sum + b.amount, 0)

            return (
              <div key={wallet.wallet_id} className="card">
                <div className="flex items-center justify-between mb-4">
                  <div>
                    <h2 className="text-xl font-semibold text-white">
                      {wallet.labels.length > 0 ? wallet.labels[0] : 'Wallet'}
                    </h2>
                    <p className="text-sm text-slate-400 font-mono">
                      {wallet.boundless_addresses.length > 0
                        ? wallet.boundless_addresses[0].address.substring(0, 42) + '...'
                        : 'No address'}
                    </p>
                  </div>
                  <div className="text-right">
                    <div className="text-sm text-slate-400">Balance</div>
                    <div className="text-xl font-bold text-white">{walletTotal.toLocaleString()} BLS</div>
                  </div>
                </div>

                {walletBalances.length === 0 ? (
                  <div className="text-center py-8 text-slate-400">
                    No assets yet. Sync with blockchain to update balances.
                  </div>
                ) : (
                  <div className="space-y-3">
                    {walletBalances.map((balance) => (
                      <div
                        key={`${balance.wallet_id}-${formatAssetType(balance.asset_type)}`}
                        className="p-4 bg-slate-900/50 rounded-lg border border-slate-700"
                      >
                        <div className="flex items-center justify-between">
                          <div className="flex items-center gap-4">
                            <div className="w-12 h-12 bg-gradient-to-br from-primary-500 to-secondary-600 rounded-full flex items-center justify-center">
                              <span className="text-white font-bold">
                                {formatAssetType(balance.asset_type).charAt(0)}
                              </span>
                            </div>
                            <div>
                              <div className="font-semibold text-white">
                                {formatAssetType(balance.asset_type)}
                              </div>
                              <div className="text-sm text-slate-400">
                                Available: {formatAmount(balance.amount - balance.locked_amount, balance.asset_type)}
                              </div>
                              {balance.locked_amount > 0 && (
                                <div className="text-xs text-yellow-400">
                                  Locked: {formatAmount(balance.locked_amount, balance.asset_type)}
                                </div>
                              )}
                            </div>
                          </div>
                          <div className="text-right">
                            <div className="text-xl font-bold text-white">
                              {formatAmount(balance.amount, balance.asset_type)}
                            </div>
                            <div className="text-xs text-slate-500">
                              Updated: {formatTimestamp(balance.updated_at)}
                            </div>
                          </div>
                        </div>
                      </div>
                    ))}
                  </div>
                )}
              </div>
            )
          })}
        </div>
      )}

      {/* Recent Transactions */}
      {allTransactions.length > 0 && (
        <div className="card">
          <div className="flex items-center justify-between mb-6">
            <h2 className="text-xl font-semibold text-white">
              Transaction History
            </h2>
            <span className="text-sm text-slate-400">
              {allTransactions.length} transaction{allTransactions.length !== 1 ? 's' : ''}
            </span>
          </div>

          <div className="space-y-3">
            {allTransactions.map((tx) => (
              <div
                key={tx.tx_id}
                className="p-4 bg-slate-900/50 rounded-lg border border-slate-700"
              >
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-4">
                    <div
                      className={`w-10 h-10 rounded-full flex items-center justify-center ${
                        tx.direction === 'in'
                          ? 'bg-green-900/30 text-green-400'
                          : 'bg-red-900/30 text-red-400'
                      }`}
                    >
                      {tx.direction === 'in' ? (
                        <ArrowDownIcon className="w-5 h-5" />
                      ) : (
                        <ArrowUpIcon className="w-5 h-5" />
                      )}
                    </div>
                    <div>
                      <div className="font-medium text-white capitalize">
                        {tx.direction === 'in' ? 'Received' : 'Sent'} {formatAssetType(tx.asset_type)}
                      </div>
                      <div className="text-sm text-slate-400">
                        {formatTimestamp(tx.timestamp)}
                      </div>
                      <div className="text-xs text-slate-500 font-mono mt-1">
                        {tx.chain_tx_hash.substring(0, 32)}...
                      </div>
                    </div>
                  </div>
                  <div className="text-right">
                    <div className={`font-medium ${tx.direction === 'in' ? 'text-green-400' : 'text-white'}`}>
                      {tx.direction === 'in' ? '+' : '-'}{formatAmount(tx.amount, tx.asset_type)}
                    </div>
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
        </div>
      )}

      {/* Application Registry & Connected Apps */}
      <div className="card">
        <div className="flex items-center justify-between mb-6">
          <div>
            <h2 className="text-xl font-semibold text-white flex items-center gap-2">
              <RectangleGroupIcon className="w-6 h-6 text-primary-400" />
              Connected Applications
            </h2>
            <p className="text-sm text-slate-400 mt-1">
              Application-aware E² Multipass ecosystem
            </p>
          </div>
          <button
            onClick={() => setShowConnectAppModal(true)}
            className="btn btn-primary flex items-center gap-2"
          >
            <LinkIcon className="w-5 h-5" />
            Connect App
          </button>
        </div>

        {connectedApps.length === 0 ? (
          <div className="text-center py-12">
            <RectangleGroupIcon className="w-16 h-16 mx-auto mb-4 text-slate-600" />
            <h3 className="text-lg font-semibold text-white mb-2">No Connected Applications</h3>
            <p className="text-slate-400 mb-6">
              Connect applications from the E² ecosystem to enhance your Multipass experience
            </p>
            <button
              onClick={() => setShowConnectAppModal(true)}
              className="btn btn-secondary mx-auto"
            >
              Browse Applications
            </button>
          </div>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {connectedApps.map((conn) => {
              const Icon = getAppIcon(conn.app.category)
              const activities = appActivities.get(conn.connection_id) || []

              return (
                <div
                  key={conn.connection_id}
                  className="p-4 bg-slate-900/50 rounded-lg border border-slate-700 hover:border-primary-500/50 transition-colors cursor-pointer"
                  onClick={() => {
                    setSelectedApp(conn)
                    setShowAppDashboard(true)
                  }}
                >
                  <div className="flex items-start justify-between mb-3">
                    <div className="flex items-center gap-3">
                      <div className="w-12 h-12 bg-gradient-to-br from-primary-500 to-secondary-600 rounded-lg flex items-center justify-center">
                        <Icon className="w-6 h-6 text-white" />
                      </div>
                      <div>
                        <div className="font-semibold text-white">{conn.app.app_name}</div>
                        <div className="text-xs text-slate-400">{formatCategoryName(conn.app.category)}</div>
                      </div>
                    </div>
                    {conn.app.is_verified && (
                      <CheckCircleIcon className="w-5 h-5 text-green-400" title="Verified App" />
                    )}
                  </div>

                  <p className="text-sm text-slate-400 mb-3 line-clamp-2">
                    {conn.app.description}
                  </p>

                  {activities.length > 0 && (
                    <div className="space-y-2 mb-3">
                      <div className="text-xs font-medium text-slate-300">Recent Activity</div>
                      {activities.slice(0, 2).map((activity) => (
                        <div key={activity.activity_id} className="text-xs text-slate-500 flex items-start gap-2">
                          <span className="text-primary-400 shrink-0">•</span>
                          <span className="line-clamp-1">{activity.description}</span>
                        </div>
                      ))}
                    </div>
                  )}

                  <div className="flex items-center justify-between text-xs">
                    <span className={`badge ${
                      conn.app.risk_rating === 'low' ? 'badge-success' :
                      conn.app.risk_rating === 'medium' ? 'badge-warning' : 'badge-error'
                    }`}>
                      {conn.app.risk_rating.toUpperCase()} RISK
                    </span>
                    <span className="text-slate-500">
                      {conn.granted_scopes.length} scope{conn.granted_scopes.length !== 1 ? 's' : ''}
                    </span>
                  </div>

                  <button
                    onClick={(e) => {
                      e.stopPropagation()
                      setSelectedApp(conn)
                      setShowAppDashboard(true)
                    }}
                    className="w-full mt-3 text-sm text-primary-400 hover:text-primary-300 flex items-center justify-center gap-1"
                  >
                    View Dashboard
                    <ArrowTopRightOnSquareIcon className="w-4 h-4" />
                  </button>
                </div>
              )
            })}
          </div>
        )}
      </div>

      {/* Create Wallet Modal */}
      {showCreateWalletModal && (
        <div className="fixed inset-0 bg-slate-900/80 backdrop-blur-sm z-50 flex items-center justify-center p-4">
          <div className="card max-w-md w-full">
            <div className="flex items-center justify-between mb-6">
              <h3 className="text-xl font-bold text-white">Create New Wallet</h3>
              <button onClick={() => setShowCreateWalletModal(false)} className="text-slate-400 hover:text-white">
                <XMarkIcon className="w-6 h-6" />
              </button>
            </div>

            <div className="space-y-4">
              <div>
                <label className="label">Wallet Label (Optional)</label>
                <input
                  type="text"
                  value={walletLabel}
                  onChange={(e) => setWalletLabel(e.target.value)}
                  className="input"
                  placeholder="e.g., Personal, Business, Savings"
                />
              </div>

              <div className="bg-slate-900/50 p-4 rounded-lg border border-slate-700">
                <div className="text-sm text-slate-300 mb-2">
                  A new Boundless blockchain wallet will be created with:
                </div>
                <ul className="text-sm text-slate-400 space-y-1 list-disc list-inside">
                  <li>Post-quantum cryptographic keys (ML-DSA-44)</li>
                  <li>Unique Boundless address</li>
                  <li>Multi-asset support</li>
                </ul>
              </div>

              <div className="flex gap-3">
                <button
                  onClick={() => setShowCreateWalletModal(false)}
                  className="btn btn-secondary flex-1"
                >
                  Cancel
                </button>
                <button
                  onClick={handleCreateWallet}
                  disabled={creating}
                  className="btn btn-primary flex-1"
                >
                  {creating ? 'Creating...' : 'Create Wallet'}
                </button>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Transfer Modal */}
      {showTransferModal && (
        <div className="fixed inset-0 bg-slate-900/80 backdrop-blur-sm z-50 flex items-center justify-center p-4">
          <div className="card max-w-md w-full">
            <div className="flex items-center justify-between mb-6">
              <h3 className="text-xl font-bold text-white">Send Assets</h3>
              <button onClick={() => setShowTransferModal(false)} className="text-slate-400 hover:text-white">
                <XMarkIcon className="w-6 h-6" />
              </button>
            </div>

            <div className="space-y-4">
              {transferError && (
                <div className="bg-red-900/20 border border-red-700 rounded-lg p-3 text-red-300 text-sm">
                  {transferError}
                </div>
              )}

              <div>
                <label className="label">From Wallet</label>
                <select
                  value={selectedWallet}
                  onChange={(e) => setSelectedWallet(e.target.value)}
                  className="input"
                >
                  {wallets.map(w => (
                    <option key={w.wallet_id} value={w.wallet_id}>
                      {w.labels[0] || w.wallet_id.substring(0, 8)}
                    </option>
                  ))}
                </select>
              </div>

              <div>
                <label className="label">To Address</label>
                <input
                  type="text"
                  value={transferTo}
                  onChange={(e) => setTransferTo(e.target.value)}
                  className="input font-mono text-sm"
                  placeholder="Boundless address"
                />
              </div>

              <div>
                <label className="label">Asset</label>
                <select
                  value={transferAsset}
                  onChange={(e) => setTransferAsset(e.target.value as AssetType)}
                  className="input"
                >
                  <option value="native">BLS (Native)</option>
                  <option value="utility_token">Utility Token</option>
                  <option value="equity_token">Equity Token</option>
                </select>
              </div>

              <div>
                <label className="label">Amount</label>
                <input
                  type="number"
                  step="0.00000001"
                  value={transferAmount}
                  onChange={(e) => setTransferAmount(e.target.value)}
                  className="input"
                  placeholder="0.00"
                />
              </div>

              <div className="flex gap-3">
                <button
                  onClick={() => setShowTransferModal(false)}
                  className="btn btn-secondary flex-1"
                >
                  Cancel
                </button>
                <button
                  onClick={handleTransfer}
                  disabled={transferring}
                  className="btn btn-primary flex-1"
                >
                  {transferring ? 'Sending...' : 'Send Transaction'}
                </button>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Connect App Modal */}
      {showConnectAppModal && (
        <div className="fixed inset-0 bg-slate-900/80 backdrop-blur-sm z-50 flex items-center justify-center p-4">
          <div className="card max-w-3xl w-full max-h-[80vh] overflow-y-auto">
            <div className="flex items-center justify-between mb-6">
              <div>
                <h3 className="text-xl font-bold text-white">Connect Application</h3>
                <p className="text-sm text-slate-400 mt-1">Browse and connect apps from the E² ecosystem</p>
              </div>
              <button onClick={() => setShowConnectAppModal(false)} className="text-slate-400 hover:text-white">
                <XMarkIcon className="w-6 h-6" />
              </button>
            </div>

            {availableApps.length === 0 ? (
              <div className="text-center py-12 text-slate-400">
                No applications available in the registry
              </div>
            ) : (
              <div className="space-y-4">
                {availableApps.map((app) => {
                  const Icon = getAppIcon(app.category)
                  const isConnected = connectedApps.some(c => c.app_id === app.app_id)

                  return (
                    <div
                      key={app.app_id}
                      className="p-4 bg-slate-900/50 rounded-lg border border-slate-700"
                    >
                      <div className="flex items-start gap-4">
                        <div className="w-16 h-16 bg-gradient-to-br from-primary-500 to-secondary-600 rounded-lg flex items-center justify-center shrink-0">
                          <Icon className="w-8 h-8 text-white" />
                        </div>
                        <div className="flex-1">
                          <div className="flex items-start justify-between mb-2">
                            <div>
                              <div className="flex items-center gap-2">
                                <h4 className="font-semibold text-white">{app.app_name}</h4>
                                {app.is_verified && (
                                  <CheckCircleIcon className="w-5 h-5 text-green-400" title="Verified App" />
                                )}
                              </div>
                              <div className="text-sm text-slate-400">{app.developer} • {formatCategoryName(app.category)}</div>
                            </div>
                            <span className={`badge ${
                              app.risk_rating === 'low' ? 'badge-success' :
                              app.risk_rating === 'medium' ? 'badge-warning' : 'badge-error'
                            }`}>
                              {app.risk_rating.toUpperCase()} RISK
                            </span>
                          </div>

                          <p className="text-sm text-slate-300 mb-3">{app.description}</p>

                          <div className="flex flex-wrap gap-2 mb-3">
                            <div className="text-xs text-slate-400">Requested Scopes:</div>
                            {app.requested_scopes.map((scope, idx) => (
                              <span key={idx} className="badge bg-slate-800 text-slate-300 text-xs">
                                {scope}
                              </span>
                            ))}
                          </div>

                          {app.jurisdictions.length > 0 && (
                            <div className="text-xs text-slate-500 mb-3">
                              Jurisdictions: {app.jurisdictions.join(', ')}
                            </div>
                          )}

                          <div className="flex gap-3">
                            {app.homepage_url && (
                              <a
                                href={app.homepage_url}
                                target="_blank"
                                rel="noopener noreferrer"
                                className="text-sm text-primary-400 hover:text-primary-300 flex items-center gap-1"
                              >
                                Visit Website
                                <ArrowTopRightOnSquareIcon className="w-4 h-4" />
                              </a>
                            )}
                            {isConnected ? (
                              <span className="text-sm text-green-400 flex items-center gap-1">
                                <CheckCircleIcon className="w-4 h-4" />
                                Connected
                              </span>
                            ) : (
                              <button
                                onClick={() => handleConnectApp(app)}
                                className="btn btn-primary btn-sm"
                              >
                                Connect
                              </button>
                            )}
                          </div>
                        </div>
                      </div>
                    </div>
                  )
                })}
              </div>
            )}
          </div>
        </div>
      )}

      {/* App Dashboard Modal */}
      {showAppDashboard && selectedApp && (
        <div className="fixed inset-0 bg-slate-900/80 backdrop-blur-sm z-50 flex items-center justify-center p-4">
          <div className="card max-w-4xl w-full max-h-[85vh] overflow-y-auto">
            <div className="flex items-center justify-between mb-6">
              <div className="flex items-center gap-4">
                {(() => {
                  const Icon = getAppIcon(selectedApp.app.category)
                  return (
                    <div className="w-16 h-16 bg-gradient-to-br from-primary-500 to-secondary-600 rounded-lg flex items-center justify-center">
                      <Icon className="w-8 h-8 text-white" />
                    </div>
                  )
                })()}
                <div>
                  <h3 className="text-2xl font-bold text-white">{selectedApp.app.app_name}</h3>
                  <p className="text-sm text-slate-400">{selectedApp.app.developer} • {formatCategoryName(selectedApp.app.category)}</p>
                </div>
              </div>
              <button onClick={() => {
                setShowAppDashboard(false)
                setSelectedApp(null)
              }} className="text-slate-400 hover:text-white">
                <XMarkIcon className="w-6 h-6" />
              </button>
            </div>

            {/* App Details */}
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-6">
              <div className="card bg-slate-900/50">
                <div className="text-sm text-slate-400 mb-1">Status</div>
                <div className={`font-semibold ${
                  selectedApp.status === 'active' ? 'text-green-400' :
                  selectedApp.status === 'suspended' ? 'text-yellow-400' : 'text-red-400'
                }`}>
                  {selectedApp.status.toUpperCase()}
                </div>
              </div>
              <div className="card bg-slate-900/50">
                <div className="text-sm text-slate-400 mb-1">Connected</div>
                <div className="font-semibold text-white">
                  {new Date(selectedApp.connected_at).toLocaleDateString()}
                </div>
              </div>
              <div className="card bg-slate-900/50">
                <div className="text-sm text-slate-400 mb-1">Granted Scopes</div>
                <div className="font-semibold text-white">
                  {selectedApp.granted_scopes.length}
                </div>
              </div>
            </div>

            {/* Scopes */}
            <div className="mb-6">
              <h4 className="text-lg font-semibold text-white mb-3">Permissions</h4>
              <div className="flex flex-wrap gap-2">
                {selectedApp.granted_scopes.map((scope, idx) => (
                  <span key={idx} className="badge bg-primary-900/50 text-primary-300 border border-primary-700">
                    {scope}
                  </span>
                ))}
              </div>
            </div>

            {/* Activity Stream */}
            <div className="mb-6">
              <h4 className="text-lg font-semibold text-white mb-3">Activity Stream</h4>
              {(() => {
                const activities = appActivities.get(selectedApp.connection_id) || []
                return activities.length === 0 ? (
                  <div className="text-center py-8 text-slate-400">
                    No activity yet
                  </div>
                ) : (
                  <div className="space-y-3">
                    {activities.map((activity) => (
                      <div
                        key={activity.activity_id}
                        className="p-4 bg-slate-900/50 rounded-lg border border-slate-700"
                      >
                        <div className="flex items-start justify-between">
                          <div className="flex-1">
                            <div className="font-medium text-white mb-1">
                              {activity.activity_type.replace(/_/g, ' ').toUpperCase()}
                            </div>
                            <div className="text-sm text-slate-300 mb-2">
                              {activity.description}
                            </div>
                            <div className="text-xs text-slate-500">
                              {formatTimestamp(activity.timestamp)}
                            </div>
                          </div>
                        </div>
                      </div>
                    ))}
                  </div>
                )
              })()}
            </div>

            {/* Actions */}
            <div className="flex gap-3 pt-4 border-t border-slate-700">
              {selectedApp.app.homepage_url && (
                <a
                  href={selectedApp.app.homepage_url}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="btn btn-secondary flex items-center gap-2"
                >
                  Visit App
                  <ArrowTopRightOnSquareIcon className="w-5 h-5" />
                </a>
              )}
              <button
                onClick={() => {
                  if (confirm(`Revoke access for ${selectedApp.app.app_name}?`)) {
                    handleRevokeApp(selectedApp.connection_id)
                    setShowAppDashboard(false)
                    setSelectedApp(null)
                  }
                }}
                className="btn bg-red-900/30 text-red-300 border-red-700 hover:bg-red-900/50"
              >
                Revoke Access
              </button>
              <div className="flex-1" />
              <button
                onClick={() => {
                  setShowAppDashboard(false)
                  setSelectedApp(null)
                }}
                className="btn btn-primary"
              >
                Close
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
