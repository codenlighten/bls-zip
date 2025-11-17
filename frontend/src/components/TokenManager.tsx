'use client'

import { useState } from 'react'
import { CurrencyDollarIcon, PaperAirplaneIcon, ArrowPathIcon } from '@heroicons/react/24/outline'

interface TokenManagerProps {
  account: string
}

export default function TokenManager({ account }: TokenManagerProps) {
  const [balance, setBalance] = useState('1,234.56')
  const [recipient, setRecipient] = useState('')
  const [amount, setAmount] = useState('')
  const [isTransferring, setIsTransferring] = useState(false)

  const handleTransfer = async () => {
    if (!recipient || !amount) return

    setIsTransferring(true)
    // Simulate transfer
    await new Promise(resolve => setTimeout(resolve, 2000))
    setIsTransferring(false)
    setRecipient('')
    setAmount('')
  }

  return (
    <div className="max-w-4xl mx-auto space-y-6">
      {/* Balance Card */}
      <div className="bg-gradient-to-br from-purple-600/20 to-blue-600/20 border border-purple-500/30 rounded-xl p-8">
        <div className="flex items-center justify-between">
          <div>
            <p className="text-purple-300 text-sm mb-2">Your BLS Balance</p>
            <p className="text-white text-4xl font-bold">{balance} BLS</p>
            <p className="text-purple-300 text-sm mt-2">≈ $2,469.12 USD</p>
          </div>
          <div className="w-20 h-20 bg-purple-600/30 rounded-full flex items-center justify-center">
            <CurrencyDollarIcon className="w-10 h-10 text-purple-400" />
          </div>
        </div>
      </div>

      {/* Transfer Form */}
      <div className="bg-slate-800/50 backdrop-blur-sm border border-purple-500/20 rounded-xl p-6">
        <h3 className="text-xl font-bold text-white mb-6 flex items-center gap-2">
          <PaperAirplaneIcon className="w-6 h-6 text-purple-400" />
          Send BLS Tokens
        </h3>
        <div className="space-y-4">
          <div>
            <label className="block text-purple-300 text-sm mb-2">
              Recipient Address
            </label>
            <input
              type="text"
              value={recipient}
              onChange={(e) => setRecipient(e.target.value)}
              placeholder="5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
              className="w-full px-4 py-3 bg-slate-700/50 border border-purple-500/30 rounded-lg text-white placeholder-purple-400/50 focus:outline-none focus:border-purple-500"
            />
          </div>
          <div>
            <label className="block text-purple-300 text-sm mb-2">
              Amount (BLS)
            </label>
            <input
              type="number"
              value={amount}
              onChange={(e) => setAmount(e.target.value)}
              placeholder="0.00"
              className="w-full px-4 py-3 bg-slate-700/50 border border-purple-500/30 rounded-lg text-white placeholder-purple-400/50 focus:outline-none focus:border-purple-500"
            />
          </div>
          <button
            onClick={handleTransfer}
            disabled={isTransferring || !recipient || !amount}
            className="w-full px-6 py-3 bg-gradient-to-r from-purple-600 to-blue-600 text-white font-medium rounded-lg hover:from-purple-700 hover:to-blue-700 transition-all disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2"
          >
            {isTransferring ? (
              <>
                <ArrowPathIcon className="w-5 h-5 animate-spin" />
                Processing...
              </>
            ) : (
              <>
                <PaperAirplaneIcon className="w-5 h-5" />
                Send Tokens
              </>
            )}
          </button>
        </div>
      </div>

      {/* Transaction History */}
      <div className="bg-slate-800/50 backdrop-blur-sm border border-purple-500/20 rounded-xl p-6">
        <h3 className="text-xl font-bold text-white mb-4">Recent Transactions</h3>
        <div className="space-y-3">
          {[
            { type: 'Received', amount: '+500.00', from: '5Gr...tQY', time: '2h ago' },
            { type: 'Sent', amount: '-250.00', to: '5Df...8Kp', time: '5h ago' },
            { type: 'Received', amount: '+1000.00', from: '5Hm...3Lq', time: '1d ago' },
          ].map((tx, i) => (
            <div
              key={i}
              className="flex items-center justify-between p-4 bg-slate-700/30 border border-purple-500/10 rounded-lg"
            >
              <div>
                <p className="text-white font-medium">{tx.type}</p>
                <p className="text-purple-300 text-sm">
                  {tx.type === 'Sent' ? `To: ${tx.to}` : `From: ${tx.from}`}
                </p>
              </div>
              <div className="text-right">
                <p className={`font-bold ${tx.amount.startsWith('+') ? 'text-green-400' : 'text-red-400'}`}>
                  {tx.amount} BLS
                </p>
                <p className="text-purple-300 text-sm">{tx.time}</p>
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  )
}
