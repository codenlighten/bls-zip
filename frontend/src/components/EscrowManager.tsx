'use client'

import { useState } from 'react'
import { ShieldCheckIcon, ClockIcon, UserGroupIcon } from '@heroicons/react/24/outline'

interface EscrowManagerProps {
  account: string
}

export default function EscrowManager({ account }: EscrowManagerProps) {
  const [showCreateForm, setShowCreateForm] = useState(false)

  const escrows = [
    {
      id: 1,
      beneficiary: '5Df...8Kp',
      amount: '500.00 BLS',
      condition: 'Website delivery completed',
      status: 'Active',
      releaseDeadline: new Date(Date.now() + 86400000 * 7),
      arbiter: '5Hm...3Lq',
    },
    {
      id: 2,
      beneficiary: '5Gr...tQY',
      amount: '1,250.00 BLS',
      condition: 'Smart contract audit report',
      status: 'Released',
      releaseDeadline: new Date(Date.now() - 86400000 * 2),
      arbiter: null,
    },
  ]

  return (
    <div className="max-w-6xl mx-auto space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <h2 className="text-2xl font-bold text-white flex items-center gap-3">
          <ShieldCheckIcon className="w-8 h-8 text-purple-400" />
          Escrow Agreements
        </h2>
        <button
          onClick={() => setShowCreateForm(!showCreateForm)}
          className="px-6 py-3 bg-gradient-to-r from-purple-600 to-blue-600 text-white font-medium rounded-lg hover:from-purple-700 hover:to-blue-700 transition-all"
        >
          {showCreateForm ? 'Cancel' : 'Create Escrow'}
        </button>
      </div>

      {/* Create Form */}
      {showCreateForm && (
        <div className="bg-slate-800/50 backdrop-blur-sm border border-purple-500/20 rounded-xl p-6">
          <h3 className="text-xl font-bold text-white mb-6">Create New Escrow</h3>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div>
              <label className="block text-purple-300 text-sm mb-2">Beneficiary Address</label>
              <input
                type="text"
                placeholder="5GrwvaEF5zXb..."
                className="w-full px-4 py-3 bg-slate-700/50 border border-purple-500/30 rounded-lg text-white placeholder-purple-400/50 focus:outline-none focus:border-purple-500"
              />
            </div>
            <div>
              <label className="block text-purple-300 text-sm mb-2">Amount (BLS)</label>
              <input
                type="number"
                placeholder="0.00"
                className="w-full px-4 py-3 bg-slate-700/50 border border-purple-500/30 rounded-lg text-white placeholder-purple-400/50 focus:outline-none focus:border-purple-500"
              />
            </div>
            <div>
              <label className="block text-purple-300 text-sm mb-2">Arbiter (Optional)</label>
              <input
                type="text"
                placeholder="5Hm..."
                className="w-full px-4 py-3 bg-slate-700/50 border border-purple-500/30 rounded-lg text-white placeholder-purple-400/50 focus:outline-none focus:border-purple-500"
              />
            </div>
            <div>
              <label className="block text-purple-300 text-sm mb-2">Release Deadline (days)</label>
              <input
                type="number"
                placeholder="7"
                className="w-full px-4 py-3 bg-slate-700/50 border border-purple-500/30 rounded-lg text-white placeholder-purple-400/50 focus:outline-none focus:border-purple-500"
              />
            </div>
            <div className="md:col-span-2">
              <label className="block text-purple-300 text-sm mb-2">Condition Description</label>
              <textarea
                placeholder="Describe the release condition..."
                rows={3}
                className="w-full px-4 py-3 bg-slate-700/50 border border-purple-500/30 rounded-lg text-white placeholder-purple-400/50 focus:outline-none focus:border-purple-500"
              />
            </div>
          </div>
          <button className="w-full mt-4 px-6 py-3 bg-gradient-to-r from-purple-600 to-blue-600 text-white font-medium rounded-lg hover:from-purple-700 hover:to-blue-700 transition-all">
            Create Escrow
          </button>
        </div>
      )}

      {/* Escrow List */}
      <div className="space-y-4">
        {escrows.map(escrow => (
          <div
            key={escrow.id}
            className="bg-slate-800/50 backdrop-blur-sm border border-purple-500/20 rounded-xl p-6"
          >
            <div className="flex items-start justify-between mb-4">
              <div className="flex items-start gap-4">
                <div className="w-12 h-12 bg-purple-600/20 rounded-lg flex items-center justify-center flex-shrink-0">
                  <ShieldCheckIcon className="w-6 h-6 text-purple-400" />
                </div>
                <div>
                  <p className="text-white font-bold text-lg mb-1">{escrow.amount}</p>
                  <p className="text-purple-300 text-sm mb-2">{escrow.condition}</p>
                  <div className="flex items-center gap-4 text-sm">
                    <span className="text-purple-400">
                      To: <span className="font-mono">{escrow.beneficiary}</span>
                    </span>
                    {escrow.arbiter && (
                      <>
                        <span className="text-purple-500">•</span>
                        <span className="text-purple-400 flex items-center gap-1">
                          <UserGroupIcon className="w-4 h-4" />
                          Arbiter: <span className="font-mono">{escrow.arbiter}</span>
                        </span>
                      </>
                    )}
                  </div>
                </div>
              </div>
              <span className={`px-3 py-1 rounded-full text-sm font-medium ${
                escrow.status === 'Active'
                  ? 'bg-green-600/20 text-green-300 border border-green-500/30'
                  : 'bg-blue-600/20 text-blue-300 border border-blue-500/30'
              }`}>
                {escrow.status}
              </span>
            </div>

            <div className="flex items-center gap-2 text-purple-300 text-sm mb-4">
              <ClockIcon className="w-4 h-4" />
              {escrow.status === 'Active' ? (
                <span>Deadline: {escrow.releaseDeadline.toLocaleDateString()}</span>
              ) : (
                <span>Released on {escrow.releaseDeadline.toLocaleDateString()}</span>
              )}
            </div>

            {escrow.status === 'Active' && (
              <div className="flex gap-3">
                <button className="px-4 py-2 bg-green-600/20 border border-green-500/30 text-green-300 rounded-lg hover:bg-green-600/30 transition-colors">
                  Release Funds
                </button>
                <button className="px-4 py-2 bg-red-600/20 border border-red-500/30 text-red-300 rounded-lg hover:bg-red-600/30 transition-colors">
                  Refund
                </button>
                {escrow.arbiter && (
                  <button className="px-4 py-2 bg-orange-600/20 border border-orange-500/30 text-orange-300 rounded-lg hover:bg-orange-600/30 transition-colors">
                    Raise Dispute
                  </button>
                )}
              </div>
            )}
          </div>
        ))}
      </div>
    </div>
  )
}
