'use client'

import { useState, useEffect } from 'react'
import WalletConnect from '@/components/WalletConnect'
import Dashboard from '@/components/Dashboard'
import TokenManager from '@/components/TokenManager'
import VotingInterface from '@/components/VotingInterface'
import EscrowManager from '@/components/EscrowManager'
import { ChartBarIcon, CurrencyDollarIcon, CheckBadgeIcon, ShieldCheckIcon } from '@heroicons/react/24/outline'

export default function Home() {
  const [selectedAccount, setSelectedAccount] = useState<string | null>(null)
  const [activeTab, setActiveTab] = useState<'dashboard' | 'token' | 'voting' | 'escrow'>('dashboard')

  return (
    <main className="min-h-screen bg-gradient-to-br from-slate-900 via-purple-900 to-slate-900">
      {/* Header */}
      <header className="border-b border-purple-500/20 bg-slate-900/50 backdrop-blur-sm">
        <div className="container mx-auto px-4 py-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <div className="w-10 h-10 bg-gradient-to-br from-purple-500 to-blue-500 rounded-lg flex items-center justify-center">
                <ShieldCheckIcon className="w-6 h-6 text-white" />
              </div>
              <div>
                <h1 className="text-2xl font-bold text-white">Boundless BLS</h1>
                <p className="text-sm text-purple-300">Post-Quantum Blockchain</p>
              </div>
            </div>
            <WalletConnect
              selectedAccount={selectedAccount}
              onAccountChange={setSelectedAccount}
            />
          </div>
        </div>
      </header>

      {/* Navigation Tabs */}
      <div className="border-b border-purple-500/20 bg-slate-900/30 backdrop-blur-sm">
        <div className="container mx-auto px-4">
          <nav className="flex gap-1">
            <TabButton
              active={activeTab === 'dashboard'}
              onClick={() => setActiveTab('dashboard')}
              icon={<ChartBarIcon className="w-5 h-5" />}
              label="Dashboard"
            />
            <TabButton
              active={activeTab === 'token'}
              onClick={() => setActiveTab('token')}
              icon={<CurrencyDollarIcon className="w-5 h-5" />}
              label="Tokens"
            />
            <TabButton
              active={activeTab === 'voting'}
              onClick={() => setActiveTab('voting')}
              icon={<CheckBadgeIcon className="w-5 h-5" />}
              label="Voting"
            />
            <TabButton
              active={activeTab === 'escrow'}
              onClick={() => setActiveTab('escrow')}
              icon={<ShieldCheckIcon className="w-5 h-5" />}
              label="Escrow"
            />
          </nav>
        </div>
      </div>

      {/* Content */}
      <div className="container mx-auto px-4 py-8">
        {!selectedAccount ? (
          <div className="text-center py-20">
            <ShieldCheckIcon className="w-20 h-20 text-purple-400 mx-auto mb-4" />
            <h2 className="text-3xl font-bold text-white mb-2">Welcome to Boundless BLS</h2>
            <p className="text-purple-200 mb-6">
              The first post-quantum blockchain platform with privacy-preserving smart contracts
            </p>
            <p className="text-purple-300">
              Connect your wallet to get started
            </p>
          </div>
        ) : (
          <>
            {activeTab === 'dashboard' && <Dashboard account={selectedAccount} />}
            {activeTab === 'token' && <TokenManager account={selectedAccount} />}
            {activeTab === 'voting' && <VotingInterface account={selectedAccount} />}
            {activeTab === 'escrow' && <EscrowManager account={selectedAccount} />}
          </>
        )}
      </div>

      {/* Footer */}
      <footer className="border-t border-purple-500/20 bg-slate-900/50 backdrop-blur-sm mt-20">
        <div className="container mx-auto px-4 py-6">
          <div className="flex flex-col md:flex-row justify-between items-center gap-4">
            <div className="text-purple-300 text-sm">
              <p>Powered by ML-KEM-768, ML-DSA-44, and SHA-3 PoW</p>
            </div>
            <div className="flex gap-6 text-sm text-purple-300">
              <a href="#" className="hover:text-purple-100 transition-colors">Documentation</a>
              <a href="#" className="hover:text-purple-100 transition-colors">GitHub</a>
              <a href="#" className="hover:text-purple-100 transition-colors">Support</a>
            </div>
          </div>
        </div>
      </footer>
    </main>
  )
}

function TabButton({
  active,
  onClick,
  icon,
  label
}: {
  active: boolean
  onClick: () => void
  icon: React.ReactNode
  label: string
}) {
  return (
    <button
      onClick={onClick}
      className={`flex items-center gap-2 px-6 py-3 font-medium transition-colors ${
        active
          ? 'text-white bg-purple-600/50 border-b-2 border-purple-400'
          : 'text-purple-300 hover:text-white hover:bg-purple-600/20'
      }`}
    >
      {icon}
      {label}
    </button>
  )
}
