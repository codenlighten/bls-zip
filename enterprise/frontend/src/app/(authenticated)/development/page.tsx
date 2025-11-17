'use client'

import { useState, useEffect } from 'react'
import { api } from '@/lib/api'
import type { IdentityProfile, IBCSession, PDESnapshot, CodeSignature } from '@/types'
import {
  CommandLineIcon,
  CubeIcon,
  ShieldCheckIcon,
  PlusIcon,
  PlayIcon,
  StopIcon,
  XCircleIcon,
  ClockIcon,
  CheckCircleIcon,
  DocumentTextIcon,
  TrashIcon,
  EyeIcon,
  CodeBracketIcon,
  ServerIcon,
} from '@heroicons/react/24/outline'

type Tab = 'ibc' | 'pde' | 'signing'

export default function DevelopmentPage() {
  const [activeTab, setActiveTab] = useState<Tab>('ibc')
  const [identity, setIdentity] = useState<IdentityProfile | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState('')

  // IBC Session state
  const [ibcSessions, setIbcSessions] = useState<IBCSession[]>([])
  const [showCreateSession, setShowCreateSession] = useState(false)
  const [sessionType, setSessionType] = useState('terminal')
  const [sessionMetadata, setSessionMetadata] = useState<Record<string, string>>({})

  // PDE state
  const [pdeSnapshots, setPdeSnapshots] = useState<PDESnapshot[]>([])
  const [showCreatePDE, setShowCreatePDE] = useState(false)
  const [pdeName, setPdeName] = useState('')
  const [pdeType, setPdeType] = useState('node_js')
  const [pdePackages, setPdePackages] = useState<string[]>([])

  // Code Signing state
  const [signatures, setSignatures] = useState<CodeSignature[]>([])
  const [showSignArtifact, setShowSignArtifact] = useState(false)
  const [artifactHash, setArtifactHash] = useState('')
  const [artifactType, setArtifactType] = useState('source_code')

  useEffect(() => {
    loadDevelopmentData()
  }, [activeTab])

  const loadDevelopmentData = async () => {
    try {
      const userStr = localStorage.getItem('user_identity')
      if (!userStr) {
        setError('No user session found')
        setLoading(false)
        return
      }

      const user: IdentityProfile = JSON.parse(userStr)
      setIdentity(user)

      // Load data based on active tab
      if (activeTab === 'ibc') {
        const response = await api.getIBCSessions(user.identity_id)
        if (response.data) {
          setIbcSessions(response.data)
        }
      } else if (activeTab === 'pde') {
        const response = await api.getPDESnapshots(user.identity_id)
        if (response.data) {
          setPdeSnapshots(response.data)
        }
      } else if (activeTab === 'signing') {
        const response = await api.getMySignatures(user.identity_id)
        if (response.data) {
          setSignatures(response.data)
        }
      }

      setLoading(false)
    } catch (err) {
      console.error('Error loading development data:', err)
      setError('Failed to load development data')
      setLoading(false)
    }
  }

  const handleCreateIBCSession = async () => {
    if (!identity) return

    try {
      const response = await api.createIBCSession({
        identity_id: identity.identity_id,
        session_type: sessionType,
        metadata: sessionMetadata,
      })

      if (response.error) {
        alert(`Failed to create session: ${response.error}`)
      } else if (response.data) {
        setIbcSessions([response.data, ...ibcSessions])
        setShowCreateSession(false)
        setSessionMetadata({})
      }
    } catch (err) {
      console.error('Error creating IBC session:', err)
      alert('Failed to create session')
    }
  }

  const handleCloseSession = async (sessionId: string) => {
    try {
      const response = await api.closeIBCSession(sessionId)
      if (response.error) {
        alert(`Failed to close session: ${response.error}`)
      } else {
        loadDevelopmentData()
      }
    } catch (err) {
      console.error('Error closing session:', err)
      alert('Failed to close session')
    }
  }

  const handleCreatePDE = async () => {
    if (!identity || !pdeName.trim()) return

    try {
      const response = await api.createPDESnapshot({
        identity_id: identity.identity_id,
        name: pdeName.trim(),
        environment_type: pdeType,
        config: {
          packages: pdePackages,
          environment_vars: {},
        },
      })

      if (response.error) {
        alert(`Failed to create PDE snapshot: ${response.error}`)
      } else if (response.data) {
        setPdeSnapshots([response.data, ...pdeSnapshots])
        setShowCreatePDE(false)
        setPdeName('')
        setPdePackages([])
      }
    } catch (err) {
      console.error('Error creating PDE:', err)
      alert('Failed to create PDE snapshot')
    }
  }

  const handleDeployPDE = async (snapshotId: string) => {
    try {
      const response = await api.deployPDESnapshot(snapshotId)
      if (response.error) {
        alert(`Failed to deploy PDE: ${response.error}`)
      } else {
        alert('PDE deployment started successfully')
      }
    } catch (err) {
      console.error('Error deploying PDE:', err)
      alert('Failed to deploy PDE')
    }
  }

  const handleSignArtifact = async () => {
    if (!identity || !artifactHash.trim()) return

    try {
      const response = await api.signArtifact({
        identity_id: identity.identity_id,
        artifact_hash: artifactHash.trim(),
        artifact_type: artifactType,
      })

      if (response.error) {
        alert(`Failed to sign artifact: ${response.error}`)
      } else if (response.data) {
        setSignatures([response.data, ...signatures])
        setShowSignArtifact(false)
        setArtifactHash('')
      }
    } catch (err) {
      console.error('Error signing artifact:', err)
      alert('Failed to sign artifact')
    }
  }

  const tabs = [
    { id: 'ibc' as Tab, name: 'IBC Sessions', icon: CommandLineIcon },
    { id: 'pde' as Tab, name: 'PDE Snapshots', icon: CubeIcon },
    { id: 'signing' as Tab, name: 'Code Signing', icon: ShieldCheckIcon },
  ]

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
          <div className="text-slate-400">Loading development tools...</div>
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
        <h1 className="text-3xl font-bold text-white">Development Tools</h1>
        <p className="text-slate-400 mt-1">
          Identity-bound compute sessions, portable environments, and code provenance
        </p>
      </div>

      {/* Tabs */}
      <div className="border-b border-slate-700">
        <div className="flex gap-4">
          {tabs.map((tab) => (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id)}
              className={`flex items-center gap-2 px-4 py-3 border-b-2 transition-colors ${
                activeTab === tab.id
                  ? 'border-primary-500 text-white'
                  : 'border-transparent text-slate-400 hover:text-white'
              }`}
            >
              <tab.icon className="w-5 h-5" />
              <span className="font-medium">{tab.name}</span>
            </button>
          ))}
        </div>
      </div>

      {/* IBC Sessions Tab */}
      {activeTab === 'ibc' && (
        <div className="space-y-6">
          <div className="card">
            <div className="flex items-center justify-between mb-6">
              <div>
                <h2 className="text-xl font-semibold text-white">Identity-Bound Compute Sessions</h2>
                <p className="text-sm text-slate-400 mt-1">
                  Cryptographically signed session recordings for debugging and compliance
                </p>
              </div>
              <button
                onClick={() => setShowCreateSession(true)}
                className="btn btn-primary flex items-center gap-2"
              >
                <PlusIcon className="w-5 h-5" />
                Start Session
              </button>
            </div>

            {ibcSessions.length === 0 ? (
              <div className="text-center py-12 text-slate-400">
                <CommandLineIcon className="w-16 h-16 mx-auto mb-4 opacity-50" />
                <p>No IBC sessions recorded</p>
                <p className="text-sm mt-2">
                  Start recording terminal sessions, API calls, or build logs with cryptographic proof
                </p>
              </div>
            ) : (
              <div className="space-y-3">
                {ibcSessions.map((session) => (
                  <div
                    key={session.session_id}
                    className="p-4 bg-slate-900/50 rounded-lg border border-slate-700"
                  >
                    <div className="flex items-start justify-between">
                      <div className="flex-1">
                        <div className="flex items-center gap-3 mb-2">
                          <span className="badge bg-primary-900/50 text-primary-300 border border-primary-700">
                            {typeof session.session_type === 'string'
                              ? session.session_type.replace(/_/g, ' ').toUpperCase()
                              : 'CUSTOM'}
                          </span>
                          <span className={`badge ${
                            session.status === 'active'
                              ? 'badge-success'
                              : session.status === 'closed'
                              ? 'badge-warning'
                              : 'badge-error'
                          }`}>
                            {session.status.toUpperCase()}
                          </span>
                          {session.chain_anchor_tx && (
                            <span className="badge bg-secondary-900/50 text-secondary-300 border border-secondary-700">
                              On-Chain
                            </span>
                          )}
                        </div>
                        <div className="text-white font-medium mb-1">
                          Session #{session.session_id.substring(0, 8)}
                        </div>
                        <div className="text-sm text-slate-400">
                          Started: {formatTimestamp(session.started_at)}
                          {session.ended_at && (
                            <span className="ml-4">Ended: {formatTimestamp(session.ended_at)}</span>
                          )}
                        </div>
                        {session.metadata.project && (
                          <div className="text-xs text-slate-500 mt-2">
                            Project: {session.metadata.project}
                          </div>
                        )}
                      </div>
                      <div className="flex gap-2">
                        <button className="btn btn-sm btn-secondary flex items-center gap-1">
                          <EyeIcon className="w-4 h-4" />
                          View
                        </button>
                        {session.status === 'active' && (
                          <button
                            onClick={() => handleCloseSession(session.session_id)}
                            className="btn btn-sm bg-red-900/30 text-red-400 hover:bg-red-900/50 border border-red-700"
                          >
                            <StopIcon className="w-4 h-4" />
                          </button>
                        )}
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>
      )}

      {/* PDE Snapshots Tab */}
      {activeTab === 'pde' && (
        <div className="space-y-6">
          <div className="card">
            <div className="flex items-center justify-between mb-6">
              <div>
                <h2 className="text-xl font-semibold text-white">Portable Developer Environments</h2>
                <p className="text-sm text-slate-400 mt-1">
                  Identity-bound environment snapshots that can be deployed anywhere
                </p>
              </div>
              <button
                onClick={() => setShowCreatePDE(true)}
                className="btn btn-primary flex items-center gap-2"
              >
                <PlusIcon className="w-5 h-5" />
                Create Snapshot
              </button>
            </div>

            {pdeSnapshots.length === 0 ? (
              <div className="text-center py-12 text-slate-400">
                <CubeIcon className="w-16 h-16 mx-auto mb-4 opacity-50" />
                <p>No PDE snapshots created</p>
                <p className="text-sm mt-2">
                  Capture your development environment and deploy it to any infrastructure
                </p>
              </div>
            ) : (
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                {pdeSnapshots.map((snapshot) => (
                  <div
                    key={snapshot.snapshot_id}
                    className="p-4 bg-slate-900/50 rounded-lg border border-slate-700"
                  >
                    <div className="flex items-start gap-3 mb-3">
                      <div className="p-2 bg-primary-900/30 rounded-lg">
                        <ServerIcon className="w-6 h-6 text-primary-400" />
                      </div>
                      <div className="flex-1">
                        <div className="font-medium text-white">{snapshot.name}</div>
                        <div className="text-xs text-slate-400">
                          {typeof snapshot.environment_type === 'string'
                            ? snapshot.environment_type.replace(/_/g, ' ').toUpperCase()
                            : 'CUSTOM'}
                        </div>
                      </div>
                    </div>
                    {snapshot.description && (
                      <div className="text-sm text-slate-400 mb-3">{snapshot.description}</div>
                    )}
                    <div className="flex flex-wrap gap-2 mb-3">
                      {snapshot.tags.map((tag) => (
                        <span
                          key={tag}
                          className="px-2 py-1 bg-slate-800 text-slate-300 text-xs rounded"
                        >
                          {tag}
                        </span>
                      ))}
                    </div>
                    <div className="text-xs text-slate-500 mb-3">
                      Size: {(snapshot.size_bytes / 1024 / 1024).toFixed(2)} MB • Created {formatTimestamp(snapshot.created_at)}
                    </div>
                    <div className="flex gap-2">
                      <button
                        onClick={() => handleDeployPDE(snapshot.snapshot_id)}
                        className="btn btn-sm btn-primary flex-1 flex items-center justify-center gap-1"
                      >
                        <PlayIcon className="w-4 h-4" />
                        Deploy
                      </button>
                      <button className="btn btn-sm bg-red-900/30 text-red-400 hover:bg-red-900/50 border border-red-700">
                        <TrashIcon className="w-4 h-4" />
                      </button>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>
      )}

      {/* Code Signing Tab */}
      {activeTab === 'signing' && (
        <div className="space-y-6">
          <div className="card">
            <div className="flex items-center justify-between mb-6">
              <div>
                <h2 className="text-xl font-semibold text-white">Code Signing & Provenance</h2>
                <p className="text-sm text-slate-400 mt-1">
                  Cryptographically sign artifacts with PQC algorithms and track provenance chains
                </p>
              </div>
              <button
                onClick={() => setShowSignArtifact(true)}
                className="btn btn-primary flex items-center gap-2"
              >
                <PlusIcon className="w-5 h-5" />
                Sign Artifact
              </button>
            </div>

            {signatures.length === 0 ? (
              <div className="text-center py-12 text-slate-400">
                <ShieldCheckIcon className="w-16 h-16 mx-auto mb-4 opacity-50" />
                <p>No signed artifacts</p>
                <p className="text-sm mt-2">
                  Sign code, binaries, or containers with your identity for tamper-proof provenance
                </p>
              </div>
            ) : (
              <div className="space-y-3">
                {signatures.map((sig) => (
                  <div
                    key={sig.signature_id}
                    className="p-4 bg-slate-900/50 rounded-lg border border-slate-700"
                  >
                    <div className="flex items-start justify-between">
                      <div className="flex-1">
                        <div className="flex items-center gap-3 mb-2">
                          <span className="badge bg-primary-900/50 text-primary-300 border border-primary-700">
                            {typeof sig.artifact_type === 'string'
                              ? sig.artifact_type.replace(/_/g, ' ').toUpperCase()
                              : 'CUSTOM'}
                          </span>
                          {sig.chain_anchor_tx && (
                            <span className="badge bg-secondary-900/50 text-secondary-300 border border-secondary-700">
                              On-Chain
                            </span>
                          )}
                        </div>
                        <div className="text-white font-medium mb-1">
                          {sig.metadata?.file_name || `Artifact ${sig.artifact_hash.substring(0, 8)}`}
                        </div>
                        <div className="text-sm text-slate-400 mb-2">
                          Algorithm: {sig.signature_algorithm}
                        </div>
                        <div className="text-xs text-slate-500 font-mono break-all">
                          Hash: {sig.artifact_hash.substring(0, 64)}...
                        </div>
                        <div className="text-xs text-slate-400 mt-2">
                          Signed: {formatTimestamp(sig.timestamp)}
                        </div>
                      </div>
                      <button className="btn btn-sm btn-secondary flex items-center gap-1">
                        <CheckCircleIcon className="w-4 h-4" />
                        Verify
                      </button>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>
      )}

      {/* Create IBC Session Modal */}
      {showCreateSession && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="card max-w-lg w-full mx-4">
            <div className="flex items-center justify-between mb-6">
              <h2 className="text-xl font-semibold text-white">Start IBC Session</h2>
              <button
                onClick={() => setShowCreateSession(false)}
                className="text-slate-400 hover:text-white"
              >
                <XCircleIcon className="w-6 h-6" />
              </button>
            </div>

            <div className="space-y-4">
              <div>
                <label className="label">Session Type</label>
                <select
                  value={sessionType}
                  onChange={(e) => setSessionType(e.target.value)}
                  className="input w-full"
                >
                  <option value="terminal">Terminal Session</option>
                  <option value="api_calls">API Calls</option>
                  <option value="git_operations">Git Operations</option>
                  <option value="build_logs">Build Logs</option>
                  <option value="debug_session">Debug Session</option>
                  <option value="deployment">Deployment</option>
                </select>
              </div>

              <div className="bg-primary-900/20 border border-primary-700 rounded-lg p-4">
                <div className="text-sm">
                  <div className="text-primary-300 font-medium mb-1">Cryptographic Recording</div>
                  <div className="text-slate-400 text-xs">
                    All events will be cryptographically signed and timestamped. Sessions can be anchored
                    on-chain for immutable proof of execution.
                  </div>
                </div>
              </div>

              <div className="flex gap-3">
                <button
                  onClick={() => setShowCreateSession(false)}
                  className="btn btn-secondary flex-1"
                >
                  Cancel
                </button>
                <button
                  onClick={handleCreateIBCSession}
                  className="btn btn-primary flex-1"
                >
                  Start Recording
                </button>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Create PDE Modal */}
      {showCreatePDE && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="card max-w-lg w-full mx-4">
            <div className="flex items-center justify-between mb-6">
              <h2 className="text-xl font-semibold text-white">Create PDE Snapshot</h2>
              <button
                onClick={() => setShowCreatePDE(false)}
                className="text-slate-400 hover:text-white"
              >
                <XCircleIcon className="w-6 h-6" />
              </button>
            </div>

            <div className="space-y-4">
              <div>
                <label className="label">Snapshot Name</label>
                <input
                  type="text"
                  value={pdeName}
                  onChange={(e) => setPdeName(e.target.value)}
                  placeholder="e.g., My Node.js Environment"
                  className="input w-full"
                />
              </div>

              <div>
                <label className="label">Environment Type</label>
                <select
                  value={pdeType}
                  onChange={(e) => setPdeType(e.target.value)}
                  className="input w-full"
                >
                  <option value="node_js">Node.js</option>
                  <option value="python">Python</option>
                  <option value="rust">Rust</option>
                  <option value="docker">Docker</option>
                  <option value="kubernetes">Kubernetes</option>
                </select>
              </div>

              <div className="bg-secondary-900/20 border border-secondary-700 rounded-lg p-4">
                <div className="text-sm">
                  <div className="text-secondary-300 font-medium mb-1">Identity-Bound</div>
                  <div className="text-slate-400 text-xs">
                    This environment snapshot is bound to your identity and can be deployed
                    securely on any infrastructure while maintaining cryptographic proof of origin.
                  </div>
                </div>
              </div>

              <div className="flex gap-3">
                <button
                  onClick={() => setShowCreatePDE(false)}
                  className="btn btn-secondary flex-1"
                >
                  Cancel
                </button>
                <button
                  onClick={handleCreatePDE}
                  disabled={!pdeName.trim()}
                  className="btn btn-primary flex-1"
                >
                  Create Snapshot
                </button>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Sign Artifact Modal */}
      {showSignArtifact && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="card max-w-lg w-full mx-4">
            <div className="flex items-center justify-between mb-6">
              <h2 className="text-xl font-semibold text-white">Sign Artifact</h2>
              <button
                onClick={() => setShowSignArtifact(false)}
                className="text-slate-400 hover:text-white"
              >
                <XCircleIcon className="w-6 h-6" />
              </button>
            </div>

            <div className="space-y-4">
              <div>
                <label className="label">Artifact Hash</label>
                <input
                  type="text"
                  value={artifactHash}
                  onChange={(e) => setArtifactHash(e.target.value)}
                  placeholder="SHA-256 hash of artifact"
                  className="input w-full font-mono text-sm"
                />
              </div>

              <div>
                <label className="label">Artifact Type</label>
                <select
                  value={artifactType}
                  onChange={(e) => setArtifactType(e.target.value)}
                  className="input w-full"
                >
                  <option value="source_code">Source Code</option>
                  <option value="binary">Binary</option>
                  <option value="container_image">Container Image</option>
                  <option value="package">Package</option>
                  <option value="configuration">Configuration</option>
                  <option value="documentation">Documentation</option>
                </select>
              </div>

              <div className="bg-primary-900/20 border border-primary-700 rounded-lg p-4">
                <div className="text-sm">
                  <div className="text-primary-300 font-medium mb-1">PQC Signature</div>
                  <div className="text-slate-400 text-xs">
                    Artifacts are signed using post-quantum cryptography (ML-DSA-44) with your
                    identity for future-proof provenance tracking.
                  </div>
                </div>
              </div>

              <div className="flex gap-3">
                <button
                  onClick={() => setShowSignArtifact(false)}
                  className="btn btn-secondary flex-1"
                >
                  Cancel
                </button>
                <button
                  onClick={handleSignArtifact}
                  disabled={!artifactHash.trim()}
                  className="btn btn-primary flex-1"
                >
                  Sign Artifact
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
