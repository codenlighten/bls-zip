'use client'

import { useEffect, useState } from 'react'
import {
  CubeIcon,
  ClockIcon,
  ServerIcon,
  ChartBarIcon,
  ShieldCheckIcon,
  BoltIcon
} from '@heroicons/react/24/outline'

interface DashboardProps {
  account: string
}

interface BlockchainStats {
  blockHeight: number
  blockTime: number
  difficulty: string
  hashRate: string
  peerCount: number
  accountBalance: string
}

export default function Dashboard({ account }: DashboardProps) {
  const [stats, setStats] = useState<BlockchainStats>({
    blockHeight: 125847,
    blockTime: 4.8,
    difficulty: '0x1d00ffff',
    hashRate: '2.4 MH/s',
    peerCount: 42,
    accountBalance: '1,234.56',
  })

  const [recentBlocks, setRecentBlocks] = useState([
    { height: 125847, hash: '0xa3f5...9d2c', txCount: 15, timestamp: Date.now() - 240000 },
    { height: 125846, hash: '0x7b2e...4a1f', txCount: 23, timestamp: Date.now() - 540000 },
    { height: 125845, hash: '0x5c9d...8e3b', txCount: 18, timestamp: Date.now() - 840000 },
  ])

  // Simulate real-time updates
  useEffect(() => {
    const interval = setInterval(() => {
      setStats(prev => ({
        ...prev,
        blockHeight: prev.blockHeight + 1,
      }))
    }, 5000)

    return () => clearInterval(interval)
  }, [])

  return (
    <div className="space-y-6">
      {/* Stats Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        <StatCard
          icon={<CubeIcon className="w-8 h-8" />}
          label="Block Height"
          value={stats.blockHeight.toLocaleString()}
          bgColor="from-purple-600/20 to-blue-600/20"
          borderColor="purple-500/30"
        />
        <StatCard
          icon={<ClockIcon className="w-8 h-8" />}
          label="Average Block Time"
          value={`${stats.blockTime}s`}
          bgColor="from-blue-600/20 to-cyan-600/20"
          borderColor="blue-500/30"
        />
        <StatCard
          icon={<BoltIcon className="w-8 h-8" />}
          label="Network Hash Rate"
          value={stats.hashRate}
          bgColor="from-green-600/20 to-emerald-600/20"
          borderColor="green-500/30"
        />
        <StatCard
          icon={<ServerIcon className="w-8 h-8" />}
          label="Connected Peers"
          value={stats.peerCount.toString()}
          bgColor="from-orange-600/20 to-red-600/20"
          borderColor="orange-500/30"
        />
        <StatCard
          icon={<ShieldCheckIcon className="w-8 h-8" />}
          label="Difficulty"
          value={stats.difficulty}
          bgColor="from-pink-600/20 to-purple-600/20"
          borderColor="pink-500/30"
        />
        <StatCard
          icon={<ChartBarIcon className="w-8 h-8" />}
          label="Your Balance"
          value={`${stats.accountBalance} BLS`}
          bgColor="from-indigo-600/20 to-purple-600/20"
          borderColor="indigo-500/30"
        />
      </div>

      {/* Recent Blocks */}
      <div className="bg-slate-800/50 backdrop-blur-sm border border-purple-500/20 rounded-xl p-6">
        <h3 className="text-xl font-bold text-white mb-4 flex items-center gap-2">
          <CubeIcon className="w-6 h-6 text-purple-400" />
          Recent Blocks
        </h3>
        <div className="space-y-3">
          {recentBlocks.map(block => (
            <div
              key={block.height}
              className="flex items-center justify-between p-4 bg-slate-700/30 border border-purple-500/10 rounded-lg hover:border-purple-500/30 transition-colors"
            >
              <div className="flex items-center gap-4">
                <div className="w-12 h-12 bg-purple-600/20 rounded-lg flex items-center justify-center">
                  <CubeIcon className="w-6 h-6 text-purple-400" />
                </div>
                <div>
                  <p className="text-white font-medium">Block #{block.height}</p>
                  <p className="text-purple-300 text-sm font-mono">{block.hash}</p>
                </div>
              </div>
              <div className="text-right">
                <p className="text-white">{block.txCount} txns</p>
                <p className="text-purple-300 text-sm">
                  {Math.floor((Date.now() - block.timestamp) / 60000)}m ago
                </p>
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* PQC Security Notice */}
      <div className="bg-gradient-to-r from-purple-600/20 to-blue-600/20 border border-purple-500/30 rounded-xl p-6">
        <div className="flex items-start gap-4">
          <ShieldCheckIcon className="w-8 h-8 text-purple-400 flex-shrink-0" />
          <div>
            <h4 className="text-lg font-bold text-white mb-2">
              Post-Quantum Security Enabled
            </h4>
            <p className="text-purple-200 mb-3">
              This blockchain uses NIST-standardized post-quantum cryptographic algorithms:
            </p>
            <ul className="space-y-1 text-purple-300 text-sm">
              <li>• ML-KEM-768 (FIPS 203) for key encapsulation</li>
              <li>• ML-DSA-44 (FIPS 204) for digital signatures</li>
              <li>• SHA-3/SHAKE256 for proof-of-work hashing</li>
              <li>• Hybrid classical+PQC schemes for gradual transition</li>
            </ul>
          </div>
        </div>
      </div>
    </div>
  )
}

function StatCard({
  icon,
  label,
  value,
  bgColor,
  borderColor
}: {
  icon: React.ReactNode
  label: string
  value: string
  bgColor: string
  borderColor: string
}) {
  return (
    <div className={`bg-gradient-to-br ${bgColor} border border-${borderColor} rounded-xl p-6`}>
      <div className="flex items-center gap-4">
        <div className="text-purple-400">
          {icon}
        </div>
        <div>
          <p className="text-purple-300 text-sm mb-1">{label}</p>
          <p className="text-white text-2xl font-bold">{value}</p>
        </div>
      </div>
    </div>
  )
}
