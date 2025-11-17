'use client'

import { useState, useEffect } from 'react'
import { api } from '@/lib/api'
import type { IdentityProfile, AIAgent, CapabilityToken, AIAgentActivity } from '@/types'
import {
  CpuChipIcon,
  PlusIcon,
  XCircleIcon,
  CheckCircleIcon,
  ShieldExclamationIcon,
  KeyIcon,
  ClockIcon,
  StopIcon,
  EyeIcon,
  SparklesIcon,
  CodeBracketIcon,
  ChartBarIcon,
  ShieldCheckIcon,
  UserIcon,
} from '@heroicons/react/24/outline'

export default function AIAgentsPage() {
  const [identity, setIdentity] = useState<IdentityProfile | null>(null)
  const [agents, setAgents] = useState<AIAgent[]>([])
  const [selectedAgent, setSelectedAgent] = useState<AIAgent | null>(null)
  const [agentActivity, setAgentActivity] = useState<AIAgentActivity[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState('')

  const [showRegisterAgent, setShowRegisterAgent] = useState(false)
  const [agentName, setAgentName] = useState('')
  const [agentType, setAgentType] = useState('code_assistant')
  const [selectedCapabilities, setSelectedCapabilities] = useState<string[]>([])

  const [showIssueToken, setShowIssueToken] = useState(false)
  const [tokenCapability, setTokenCapability] = useState('')
  const [tokenScope, setTokenScope] = useState<string[]>([])

  useEffect(() => {
    loadAgentsData()
  }, [])

  useEffect(() => {
    if (selectedAgent) {
      loadAgentActivity(selectedAgent.agent_id)
    }
  }, [selectedAgent])

  const loadAgentsData = async () => {
    try {
      const userStr = localStorage.getItem('user_identity')
      if (!userStr) {
        setError('No user session found')
        setLoading(false)
        return
      }

      const user: IdentityProfile = JSON.parse(userStr)
      setIdentity(user)

      const response = await api.getAIAgents(user.identity_id)
      if (response.error) {
        console.error('Error loading agents:', response.error)
      } else if (response.data) {
        setAgents(response.data)
        if (response.data.length > 0 && !selectedAgent) {
          setSelectedAgent(response.data[0])
        }
      }

      setLoading(false)
    } catch (err) {
      console.error('Error loading AI agents:', err)
      setError('Failed to load AI agents')
      setLoading(false)
    }
  }

  const loadAgentActivity = async (agentId: string) => {
    try {
      const response = await api.getAgentActivity(agentId, 20)
      if (response.data) {
        setAgentActivity(response.data)
      }
    } catch (err) {
      console.error('Error loading agent activity:', err)
    }
  }

  const handleRegisterAgent = async () => {
    if (!identity || !agentName.trim()) return

    try {
      const response = await api.registerAIAgent({
        identity_id: identity.identity_id,
        agent_name: agentName.trim(),
        agent_type: agentType,
        capabilities: selectedCapabilities.map(cap => ({
          capability: cap,
          scope: ['*'],
          granted_at: new Date().toISOString(),
        })),
      })

      if (response.error) {
        alert(`Failed to register agent: ${response.error}`)
      } else if (response.data) {
        setAgents([...agents, response.data])
        setShowRegisterAgent(false)
        setAgentName('')
        setSelectedCapabilities([])
      }
    } catch (err) {
      console.error('Error registering agent:', err)
      alert('Failed to register agent')
    }
  }

  const handleIssueToken = async () => {
    if (!selectedAgent || !tokenCapability.trim()) return

    try {
      const response = await api.issueCapabilityToken(selectedAgent.agent_id, {
        capability: tokenCapability.trim(),
        scope: tokenScope,
      })

      if (response.error) {
        alert(`Failed to issue token: ${response.error}`)
      } else {
        setShowIssueToken(false)
        setTokenCapability('')
        setTokenScope([])
        loadAgentsData()
      }
    } catch (err) {
      console.error('Error issuing token:', err)
      alert('Failed to issue token')
    }
  }

  const handleSuspendAgent = async (agentId: string) => {
    if (!confirm('Are you sure you want to suspend this AI agent? This will revoke all its capabilities.')) {
      return
    }

    try {
      const response = await api.suspendAgent(agentId)
      if (response.error) {
        alert(`Failed to suspend agent: ${response.error}`)
      } else {
        loadAgentsData()
      }
    } catch (err) {
      console.error('Error suspending agent:', err)
      alert('Failed to suspend agent')
    }
  }

  const toggleCapability = (capability: string) => {
    if (selectedCapabilities.includes(capability)) {
      setSelectedCapabilities(selectedCapabilities.filter(c => c !== capability))
    } else {
      setSelectedCapabilities([...selectedCapabilities, capability])
    }
  }

  const getAgentTypeIcon = (type: any) => {
    const typeStr = typeof type === 'string' ? type : 'custom'
    switch (typeStr) {
      case 'code_assistant': return CodeBracketIcon
      case 'data_analyst': return ChartBarIcon
      case 'security_auditor': return ShieldCheckIcon
      case 'compliance_monitor': return ShieldExclamationIcon
      case 'customer_support': return UserIcon
      default: return CpuChipIcon
    }
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

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="flex flex-col items-center gap-4">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-primary-500"></div>
          <div className="text-slate-400">Loading AI agents...</div>
        </div>
      </div>
    )
  }

  if (error || !identity) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="text-center">
          <XCircleIcon className="w-12 h-12 text-red-400 mx-auto mb-4" />
          <div className="text-red-400">{error || 'No identity found'}</div>
        </div>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      {/* Page Header */}
      <div>
        <h1 className="text-3xl font-bold text-white">AI Agent Governance</h1>
        <p className="text-slate-400 mt-1">
          Register, monitor, and control AI agents with scoped capability tokens
        </p>
      </div>

      {/* Main Content */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* Agents List */}
        <div className="lg:col-span-1">
          <div className="card">
            <div className="flex items-center justify-between mb-4">
              <h2 className="text-lg font-semibold text-white">Registered Agents</h2>
              <button
                onClick={() => setShowRegisterAgent(true)}
                className="btn btn-sm btn-primary flex items-center gap-1"
              >
                <PlusIcon className="w-4 h-4" />
                Register
              </button>
            </div>

            {agents.length === 0 ? (
              <div className="text-center py-8 text-slate-400">
                <CpuChipIcon className="w-12 h-12 mx-auto mb-3 opacity-50" />
                <p className="text-sm">No agents registered</p>
              </div>
            ) : (
              <div className="space-y-2">
                {agents.map((agent) => {
                  const Icon = getAgentTypeIcon(agent.agent_type)
                  const isSelected = selectedAgent?.agent_id === agent.agent_id
                  return (
                    <button
                      key={agent.agent_id}
                      onClick={() => setSelectedAgent(agent)}
                      className={`w-full p-3 rounded-lg border text-left transition-colors ${
                        isSelected
                          ? 'bg-primary-900/30 border-primary-600'
                          : 'bg-slate-900/50 border-slate-700 hover:border-slate-600'
                      }`}
                    >
                      <div className="flex items-center gap-3 mb-2">
                        <div className={`p-2 rounded-lg ${
                          agent.status === 'active'
                            ? 'bg-primary-900/30'
                            : 'bg-red-900/30'
                        }`}>
                          <Icon className={`w-5 h-5 ${
                            agent.status === 'active'
                              ? 'text-primary-400'
                              : 'text-red-400'
                          }`} />
                        </div>
                        <div className="flex-1 min-w-0">
                          <div className="font-medium text-white truncate">{agent.agent_name}</div>
                          <div className="text-xs text-slate-400">
                            {typeof agent.agent_type === 'string'
                              ? agent.agent_type.replace(/_/g, ' ')
                              : 'custom'}
                          </div>
                        </div>
                      </div>
                      <span className={`badge text-xs ${
                        agent.status === 'active'
                          ? 'badge-success'
                          : agent.status === 'suspended'
                          ? 'badge-warning'
                          : 'badge-error'
                      }`}>
                        {agent.status.toUpperCase()}
                      </span>
                    </button>
                  )
                })}
              </div>
            )}
          </div>
        </div>

        {/* Agent Details */}
        <div className="lg:col-span-2 space-y-6">
          {selectedAgent ? (
            <>
              {/* Agent Info */}
              <div className="card">
                <div className="flex items-start justify-between mb-6">
                  <div className="flex items-center gap-4">
                    <div className={`p-3 rounded-lg ${
                      selectedAgent.status === 'active'
                        ? 'bg-primary-900/30'
                        : 'bg-red-900/30'
                    }`}>
                      {(() => {
                        const Icon = getAgentTypeIcon(selectedAgent.agent_type)
                        return <Icon className={`w-8 h-8 ${
                          selectedAgent.status === 'active'
                            ? 'text-primary-400'
                            : 'text-red-400'
                        }`} />
                      })()}
                    </div>
                    <div>
                      <h2 className="text-xl font-bold text-white">{selectedAgent.agent_name}</h2>
                      <p className="text-slate-400">
                        {typeof selectedAgent.agent_type === 'string'
                          ? selectedAgent.agent_type.replace(/_/g, ' ').toUpperCase()
                          : 'CUSTOM'}
                      </p>
                    </div>
                  </div>
                  {selectedAgent.status === 'active' && (
                    <button
                      onClick={() => handleSuspendAgent(selectedAgent.agent_id)}
                      className="btn btn-sm bg-red-900/30 text-red-400 hover:bg-red-900/50 border border-red-700"
                    >
                      <StopIcon className="w-4 h-4" />
                    </button>
                  )}
                </div>

                <div className="grid grid-cols-2 gap-4 mb-6">
                  <div>
                    <label className="label">Created</label>
                    <div className="text-white">{formatTimestamp(selectedAgent.created_at)}</div>
                  </div>
                  <div>
                    <label className="label">Last Active</label>
                    <div className="text-white">
                      {selectedAgent.last_active
                        ? formatTimestamp(selectedAgent.last_active)
                        : 'Never'}
                    </div>
                  </div>
                </div>

                <div className="mb-6">
                  <label className="label">Agent ID</label>
                  <div className="text-white font-mono text-sm break-all">
                    {selectedAgent.agent_id}
                  </div>
                </div>
              </div>

              {/* Capabilities & Tokens */}
              <div className="card">
                <div className="flex items-center justify-between mb-6">
                  <h3 className="text-lg font-semibold text-white">Capability Tokens</h3>
                  {selectedAgent.status === 'active' && (
                    <button
                      onClick={() => setShowIssueToken(true)}
                      className="btn btn-sm btn-primary flex items-center gap-1"
                    >
                      <KeyIcon className="w-4 h-4" />
                      Issue Token
                    </button>
                  )}
                </div>

                {selectedAgent.capability_tokens.length === 0 ? (
                  <div className="text-center py-8 text-slate-400">
                    <KeyIcon className="w-12 h-12 mx-auto mb-3 opacity-50" />
                    <p className="text-sm">No capability tokens issued</p>
                  </div>
                ) : (
                  <div className="space-y-3">
                    {selectedAgent.capability_tokens.map((token) => (
                      <div
                        key={token.token_id}
                        className={`p-4 rounded-lg border ${
                          token.revoked
                            ? 'bg-red-900/10 border-red-700'
                            : 'bg-slate-900/50 border-slate-700'
                        }`}
                      >
                        <div className="flex items-start justify-between">
                          <div className="flex-1">
                            <div className="flex items-center gap-2 mb-2">
                              <span className="badge bg-primary-900/50 text-primary-300 border border-primary-700">
                                {token.capability}
                              </span>
                              {token.revoked ? (
                                <span className="badge badge-error">REVOKED</span>
                              ) : token.expires_at && new Date(token.expires_at) < new Date() ? (
                                <span className="badge badge-warning">EXPIRED</span>
                              ) : (
                                <span className="badge badge-success">ACTIVE</span>
                              )}
                            </div>
                            <div className="text-sm text-slate-400 mb-1">
                              Scope: {token.scope.join(', ')}
                            </div>
                            <div className="text-xs text-slate-500">
                              Issued: {formatTimestamp(token.issued_at)}
                              {token.expires_at && (
                                <span className="ml-4">Expires: {formatTimestamp(token.expires_at)}</span>
                              )}
                            </div>
                          </div>
                          {!token.revoked && (
                            <button
                              onClick={async () => {
                                try {
                                  await api.revokeCapabilityToken(token.token_id)
                                  loadAgentsData()
                                } catch (err) {
                                  alert('Failed to revoke token')
                                }
                              }}
                              className="btn btn-sm bg-red-900/30 text-red-400 hover:bg-red-900/50 border border-red-700"
                            >
                              Revoke
                            </button>
                          )}
                        </div>
                      </div>
                    ))}
                  </div>
                )}
              </div>

              {/* Activity Log */}
              <div className="card">
                <h3 className="text-lg font-semibold text-white mb-6">Activity Log</h3>
                {agentActivity.length === 0 ? (
                  <div className="text-center py-8 text-slate-400">
                    <ClockIcon className="w-12 h-12 mx-auto mb-3 opacity-50" />
                    <p className="text-sm">No activity recorded</p>
                  </div>
                ) : (
                  <div className="space-y-2">
                    {agentActivity.map((activity) => (
                      <div
                        key={activity.activity_id}
                        className="p-3 bg-slate-900/50 rounded-lg border border-slate-700"
                      >
                        <div className="flex items-start justify-between">
                          <div className="flex-1">
                            <div className="flex items-center gap-2 mb-1">
                              <span className="text-white font-medium">{activity.action_type}</span>
                              <span className={`badge text-xs ${
                                activity.result === 'success'
                                  ? 'badge-success'
                                  : activity.result === 'failure'
                                  ? 'badge-error'
                                  : 'badge-warning'
                              }`}>
                                {activity.result.toUpperCase()}
                              </span>
                            </div>
                            {activity.resource_accessed && (
                              <div className="text-sm text-slate-400 mb-1">
                                Resource: {activity.resource_accessed}
                              </div>
                            )}
                            <div className="text-xs text-slate-500">
                              {formatTimestamp(activity.timestamp)}
                            </div>
                          </div>
                        </div>
                      </div>
                    ))}
                  </div>
                )}
              </div>
            </>
          ) : (
            <div className="card">
              <div className="text-center py-12 text-slate-400">
                <SparklesIcon className="w-16 h-16 mx-auto mb-4 opacity-50" />
                <p>Select an agent to view details</p>
              </div>
            </div>
          )}
        </div>
      </div>

      {/* Register Agent Modal */}
      {showRegisterAgent && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="card max-w-lg w-full mx-4">
            <div className="flex items-center justify-between mb-6">
              <h2 className="text-xl font-semibold text-white">Register AI Agent</h2>
              <button
                onClick={() => setShowRegisterAgent(false)}
                className="text-slate-400 hover:text-white"
              >
                <XCircleIcon className="w-6 h-6" />
              </button>
            </div>

            <div className="space-y-4">
              <div>
                <label className="label">Agent Name</label>
                <input
                  type="text"
                  value={agentName}
                  onChange={(e) => setAgentName(e.target.value)}
                  placeholder="e.g., My Code Assistant"
                  className="input w-full"
                />
              </div>

              <div>
                <label className="label">Agent Type</label>
                <select
                  value={agentType}
                  onChange={(e) => setAgentType(e.target.value)}
                  className="input w-full"
                >
                  <option value="code_assistant">Code Assistant</option>
                  <option value="data_analyst">Data Analyst</option>
                  <option value="security_auditor">Security Auditor</option>
                  <option value="compliance_monitor">Compliance Monitor</option>
                  <option value="customer_support">Customer Support</option>
                </select>
              </div>

              <div>
                <label className="label">Initial Capabilities</label>
                <div className="space-y-2">
                  {['read_code', 'write_code', 'execute_commands', 'access_data', 'send_messages'].map((cap) => (
                    <label
                      key={cap}
                      className="flex items-center gap-3 p-3 bg-slate-900/50 rounded-lg border border-slate-700 cursor-pointer hover:border-primary-600"
                    >
                      <input
                        type="checkbox"
                        checked={selectedCapabilities.includes(cap)}
                        onChange={() => toggleCapability(cap)}
                        className="form-checkbox h-4 w-4 text-primary-500 rounded border-slate-600"
                      />
                      <span className="text-white">{cap.replace(/_/g, ' ').toUpperCase()}</span>
                    </label>
                  ))}
                </div>
              </div>

              <div className="bg-primary-900/20 border border-primary-700 rounded-lg p-4">
                <div className="text-sm">
                  <div className="text-primary-300 font-medium mb-1">Identity-Bound Agent</div>
                  <div className="text-slate-400 text-xs">
                    AI agents are cryptographically bound to your identity. All actions are logged
                    and auditable with capability-based access control.
                  </div>
                </div>
              </div>

              <div className="flex gap-3">
                <button
                  onClick={() => setShowRegisterAgent(false)}
                  className="btn btn-secondary flex-1"
                >
                  Cancel
                </button>
                <button
                  onClick={handleRegisterAgent}
                  disabled={!agentName.trim()}
                  className="btn btn-primary flex-1"
                >
                  Register Agent
                </button>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Issue Token Modal */}
      {showIssueToken && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="card max-w-lg w-full mx-4">
            <div className="flex items-center justify-between mb-6">
              <h2 className="text-xl font-semibold text-white">Issue Capability Token</h2>
              <button
                onClick={() => setShowIssueToken(false)}
                className="text-slate-400 hover:text-white"
              >
                <XCircleIcon className="w-6 h-6" />
              </button>
            </div>

            <div className="space-y-4">
              <div>
                <label className="label">Capability</label>
                <input
                  type="text"
                  value={tokenCapability}
                  onChange={(e) => setTokenCapability(e.target.value)}
                  placeholder="e.g., read_documents"
                  className="input w-full"
                />
              </div>

              <div>
                <label className="label">Scope (one per line)</label>
                <textarea
                  value={tokenScope.join('\n')}
                  onChange={(e) => setTokenScope(e.target.value.split('\n').filter(s => s.trim()))}
                  placeholder="e.g., /documents/*&#10;/api/data/*"
                  className="input w-full h-24"
                />
              </div>

              <div className="bg-secondary-900/20 border border-secondary-700 rounded-lg p-4">
                <div className="text-sm">
                  <div className="text-secondary-300 font-medium mb-1">Scoped Access</div>
                  <div className="text-slate-400 text-xs">
                    Capability tokens grant fine-grained permissions. Tokens can be revoked at any time
                    and all agent actions are cryptographically logged.
                  </div>
                </div>
              </div>

              <div className="flex gap-3">
                <button
                  onClick={() => setShowIssueToken(false)}
                  className="btn btn-secondary flex-1"
                >
                  Cancel
                </button>
                <button
                  onClick={handleIssueToken}
                  disabled={!tokenCapability.trim()}
                  className="btn btn-primary flex-1"
                >
                  Issue Token
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
