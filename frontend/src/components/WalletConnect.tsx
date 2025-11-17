'use client'

import { useState, useEffect } from 'react'
import { WalletIcon } from '@heroicons/react/24/outline'

interface WalletConnectProps {
  selectedAccount: string | null
  onAccountChange: (account: string | null) => void
}

export default function WalletConnect({ selectedAccount, onAccountChange }: WalletConnectProps) {
  const [accounts, setAccounts] = useState<string[]>([])
  const [isConnecting, setIsConnecting] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const connectWallet = async () => {
    setIsConnecting(true)
    setError(null)

    try {
      // In production, this would use @polkadot/extension-dapp
      // For demo, we'll simulate wallet connection
      const { web3Enable, web3Accounts } = await import('@polkadot/extension-dapp')

      const extensions = await web3Enable('Boundless BLS')

      if (extensions.length === 0) {
        throw new Error('No wallet extension found. Please install Polkadot.js extension.')
      }

      const allAccounts = await web3Accounts()

      if (allAccounts.length === 0) {
        throw new Error('No accounts found in wallet.')
      }

      const accountAddresses = allAccounts.map(acc => acc.address)
      setAccounts(accountAddresses)
      onAccountChange(accountAddresses[0])
    } catch (err: any) {
      console.error('Wallet connection error:', err)
      setError(err.message || 'Failed to connect wallet')

      // Fallback: create mock account for demo
      const mockAccount = '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY'
      setAccounts([mockAccount])
      onAccountChange(mockAccount)
    } finally {
      setIsConnecting(false)
    }
  }

  const disconnectWallet = () => {
    setAccounts([])
    onAccountChange(null)
  }

  const formatAddress = (address: string) => {
    return `${address.slice(0, 6)}...${address.slice(-4)}`
  }

  if (selectedAccount) {
    return (
      <div className="flex items-center gap-3">
        <div className="px-4 py-2 bg-purple-600/20 border border-purple-500/30 rounded-lg">
          <p className="text-sm text-purple-300 mb-1">Connected Account</p>
          <p className="text-white font-mono text-sm">{formatAddress(selectedAccount)}</p>
        </div>
        <button
          onClick={disconnectWallet}
          className="px-4 py-2 bg-red-600/20 border border-red-500/30 text-red-300 rounded-lg hover:bg-red-600/30 transition-colors"
        >
          Disconnect
        </button>
      </div>
    )
  }

  return (
    <div>
      <button
        onClick={connectWallet}
        disabled={isConnecting}
        className="flex items-center gap-2 px-6 py-3 bg-gradient-to-r from-purple-600 to-blue-600 text-white font-medium rounded-lg hover:from-purple-700 hover:to-blue-700 transition-all disabled:opacity-50 disabled:cursor-not-allowed"
      >
        <WalletIcon className="w-5 h-5" />
        {isConnecting ? 'Connecting...' : 'Connect Wallet'}
      </button>
      {error && (
        <p className="text-red-400 text-sm mt-2 max-w-xs">{error}</p>
      )}
    </div>
  )
}
