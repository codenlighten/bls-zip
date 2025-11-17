'use client'

import { useState } from 'react'
import { CheckBadgeIcon, ChartBarIcon, LockClosedIcon } from '@heroicons/react/24/outline'

interface VotingInterfaceProps {
  account: string
}

export default function VotingInterface({ account }: VotingInterfaceProps) {
  const [selectedOption, setSelectedOption] = useState<number | null>(null)
  const [hasVoted, setHasVoted] = useState(false)

  const poll = {
    question: 'Should the network increase the block gas limit?',
    options: [
      'Yes, increase to 15M',
      'Yes, increase to 20M',
      'No, keep at 10M',
      'Abstain',
    ],
    deadline: new Date(Date.now() + 86400000 * 3), // 3 days from now
    totalVotes: 1247,
  }

  const results = [
    { option: 'Yes, increase to 15M', votes: 523, percentage: 42 },
    { option: 'Yes, increase to 20M', votes: 312, percentage: 25 },
    { option: 'No, keep at 10M', votes: 374, percentage: 30 },
    { option: 'Abstain', votes: 38, percentage: 3 },
  ]

  const handleVote = () => {
    if (selectedOption === null) return
    setHasVoted(true)
  }

  return (
    <div className="max-w-4xl mx-auto space-y-6">
      {/* Poll Header */}
      <div className="bg-gradient-to-br from-purple-600/20 to-blue-600/20 border border-purple-500/30 rounded-xl p-6">
        <div className="flex items-start gap-4">
          <div className="w-12 h-12 bg-purple-600/30 rounded-lg flex items-center justify-center flex-shrink-0">
            <CheckBadgeIcon className="w-6 h-6 text-purple-400" />
          </div>
          <div className="flex-1">
            <h2 className="text-2xl font-bold text-white mb-2">{poll.question}</h2>
            <div className="flex items-center gap-4 text-purple-300 text-sm">
              <span>{poll.totalVotes} votes</span>
              <span>•</span>
              <span>Ends {poll.deadline.toLocaleDateString()}</span>
            </div>
          </div>
        </div>
      </div>

      {/* Privacy Notice */}
      <div className="bg-slate-800/50 backdrop-blur-sm border border-purple-500/20 rounded-xl p-4">
        <div className="flex items-center gap-3">
          <LockClosedIcon className="w-5 h-5 text-purple-400" />
          <p className="text-purple-300 text-sm">
            Your vote is encrypted using homomorphic encryption. Individual votes remain private while allowing verifiable tallying.
          </p>
        </div>
      </div>

      {!hasVoted ? (
        /* Voting Options */
        <div className="bg-slate-800/50 backdrop-blur-sm border border-purple-500/20 rounded-xl p-6">
          <h3 className="text-xl font-bold text-white mb-4">Cast Your Vote</h3>
          <div className="space-y-3 mb-6">
            {poll.options.map((option, index) => (
              <button
                key={index}
                onClick={() => setSelectedOption(index)}
                className={`w-full p-4 rounded-lg border-2 transition-all text-left ${
                  selectedOption === index
                    ? 'border-purple-500 bg-purple-600/20'
                    : 'border-purple-500/20 bg-slate-700/30 hover:border-purple-500/40'
                }`}
              >
                <div className="flex items-center gap-3">
                  <div className={`w-5 h-5 rounded-full border-2 flex items-center justify-center ${
                    selectedOption === index
                      ? 'border-purple-500 bg-purple-500'
                      : 'border-purple-500/50'
                  }`}>
                    {selectedOption === index && (
                      <div className="w-2 h-2 bg-white rounded-full" />
                    )}
                  </div>
                  <span className="text-white font-medium">{option}</span>
                </div>
              </button>
            ))}
          </div>
          <button
            onClick={handleVote}
            disabled={selectedOption === null}
            className="w-full px-6 py-3 bg-gradient-to-r from-purple-600 to-blue-600 text-white font-medium rounded-lg hover:from-purple-700 hover:to-blue-700 transition-all disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2"
          >
            <CheckBadgeIcon className="w-5 h-5" />
            Submit Encrypted Vote
          </button>
        </div>
      ) : (
        /* Results */
        <div className="bg-slate-800/50 backdrop-blur-sm border border-purple-500/20 rounded-xl p-6">
          <h3 className="text-xl font-bold text-white mb-6 flex items-center gap-2">
            <ChartBarIcon className="w-6 h-6 text-purple-400" />
            Current Results
          </h3>
          <div className="space-y-4">
            {results.map((result, index) => (
              <div key={index}>
                <div className="flex items-center justify-between mb-2">
                  <span className="text-white font-medium">{result.option}</span>
                  <span className="text-purple-300">{result.votes} votes ({result.percentage}%)</span>
                </div>
                <div className="w-full h-3 bg-slate-700/50 rounded-full overflow-hidden">
                  <div
                    className="h-full bg-gradient-to-r from-purple-600 to-blue-600 transition-all duration-500"
                    style={{ width: `${result.percentage}%` }}
                  />
                </div>
              </div>
            ))}
          </div>
          <div className="mt-6 p-4 bg-green-600/20 border border-green-500/30 rounded-lg">
            <p className="text-green-300 text-center">
              ✓ Your encrypted vote has been recorded and added to the tally
            </p>
          </div>
        </div>
      )}
    </div>
  )
}
