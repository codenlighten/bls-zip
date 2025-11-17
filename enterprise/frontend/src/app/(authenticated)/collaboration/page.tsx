'use client'

import { useState, useEffect } from 'react'
import { api } from '@/lib/api'
import type { IdentityProfile, CollaborationCapsule, CapsuleActivity } from '@/types'
import {
  UserGroupIcon,
  PlusIcon,
  XCircleIcon,
  ShieldCheckIcon,
  LockClosedIcon,
  DocumentIcon,
  ClockIcon,
  CheckCircleIcon,
  XMarkIcon,
  UserPlusIcon,
  FolderPlusIcon,
  EyeIcon,
  ArchiveBoxIcon,
  ExclamationTriangleIcon,
} from '@heroicons/react/24/outline'

export default function CollaborationPage() {
  const [identity, setIdentity] = useState<IdentityProfile | null>(null)
  const [capsules, setCapsules] = useState<CollaborationCapsule[]>([])
  const [selectedCapsule, setSelectedCapsule] = useState<CollaborationCapsule | null>(null)
  const [capsuleActivity, setCapsuleActivity] = useState<CapsuleActivity[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState('')

  const [showCreateCapsule, setShowCreateCapsule] = useState(false)
  const [capsuleName, setCapsuleName] = useState('')
  const [capsuleDescription, setCapsuleDescription] = useState('')

  const [showAddParticipant, setShowAddParticipant] = useState(false)
  const [participantEmail, setParticipantEmail] = useState('')
  const [participantRole, setParticipantRole] = useState('member')
  const [participantPermissions, setParticipantPermissions] = useState<string[]>([])

  const [showAddResource, setShowAddResource] = useState(false)
  const [resourceType, setResourceType] = useState('document')
  const [resourceRef, setResourceRef] = useState('')
  const [resourceAccess, setResourceAccess] = useState<'read' | 'write' | 'admin'>('read')

  useEffect(() => {
    loadCollaborationData()
  }, [])

  useEffect(() => {
    if (selectedCapsule) {
      loadCapsuleActivity(selectedCapsule.capsule_id)
    }
  }, [selectedCapsule])

  const loadCollaborationData = async () => {
    try {
      const userStr = localStorage.getItem('user_identity')
      if (!userStr) {
        setError('No user session found')
        setLoading(false)
        return
      }

      const user: IdentityProfile = JSON.parse(userStr)
      setIdentity(user)

      const response = await api.getMyCapsules(user.identity_id)
      if (response.error) {
        console.error('Error loading capsules:', response.error)
      } else if (response.data) {
        setCapsules(response.data)
        if (response.data.length > 0 && !selectedCapsule) {
          setSelectedCapsule(response.data[0])
        }
      }

      setLoading(false)
    } catch (err) {
      console.error('Error loading collaboration capsules:', err)
      setError('Failed to load collaboration capsules')
      setLoading(false)
    }
  }

  const loadCapsuleActivity = async (capsuleId: string) => {
    try {
      const response = await api.getCapsuleActivity(capsuleId, 20)
      if (response.data) {
        setCapsuleActivity(response.data)
      }
    } catch (err) {
      console.error('Error loading capsule activity:', err)
    }
  }

  const handleCreateCapsule = async () => {
    if (!identity || !capsuleName.trim()) return

    try {
      const response = await api.createCapsule({
        creator_id: identity.identity_id,
        name: capsuleName.trim(),
        description: capsuleDescription.trim() || undefined,
        participants: [{
          identity_id: identity.identity_id,
          role: 'admin',
          permissions: ['*'],
        }],
      })

      if (response.error) {
        alert(`Failed to create capsule: ${response.error}`)
      } else if (response.data) {
        setCapsules([response.data, ...capsules])
        setShowCreateCapsule(false)
        setCapsuleName('')
        setCapsuleDescription('')
      }
    } catch (err) {
      console.error('Error creating capsule:', err)
      alert('Failed to create collaboration capsule')
    }
  }

  const handleAddParticipant = async () => {
    if (!selectedCapsule || !participantEmail.trim()) return

    try {
      // Lookup identity_id by email from the backend
      const lookupResponse = await api.lookupIdentityByEmail(participantEmail.trim())

      if (lookupResponse.error) {
        alert(`Failed to find identity: ${lookupResponse.error}. User must be registered in E² Multipass.`)
        return
      }

      if (!lookupResponse.data?.identity_id) {
        alert('Identity not found. The user must first register with E² Multipass.')
        return
      }

      // Add participant to capsule with their verified identity_id
      const response = await api.addCapsuleParticipant(selectedCapsule.capsule_id, {
        identity_id: lookupResponse.data.identity_id,
        role: participantRole,
        permissions: participantPermissions,
      })

      if (response.error) {
        alert(`Failed to add participant: ${response.error}`)
      } else {
        setShowAddParticipant(false)
        setParticipantEmail('')
        setParticipantPermissions([])
        loadCollaborationData()
      }
    } catch (err) {
      console.error('Error adding participant:', err)
      alert('Failed to add participant')
    }
  }

  const handleAddResource = async () => {
    if (!selectedCapsule || !resourceRef.trim()) return

    try {
      const response = await api.addCapsuleResource(selectedCapsule.capsule_id, {
        resource_type: resourceType,
        resource_ref: resourceRef.trim(),
        access_level: resourceAccess,
      })

      if (response.error) {
        alert(`Failed to add resource: ${response.error}`)
      } else {
        setShowAddResource(false)
        setResourceRef('')
        loadCollaborationData()
      }
    } catch (err) {
      console.error('Error adding resource:', err)
      alert('Failed to add resource')
    }
  }

  const handleSuspendCapsule = async (capsuleId: string) => {
    if (!confirm('Are you sure you want to suspend this capsule? All access will be revoked.')) {
      return
    }

    try {
      const response = await api.suspendCapsule(capsuleId)
      if (response.error) {
        alert(`Failed to suspend capsule: ${response.error}`)
      } else {
        loadCollaborationData()
      }
    } catch (err) {
      console.error('Error suspending capsule:', err)
      alert('Failed to suspend capsule')
    }
  }

  const handleArchiveCapsule = async (capsuleId: string) => {
    if (!confirm('Are you sure you want to archive this capsule?')) {
      return
    }

    try {
      const response = await api.archiveCapsule(capsuleId)
      if (response.error) {
        alert(`Failed to archive capsule: ${response.error}`)
      } else {
        loadCollaborationData()
      }
    } catch (err) {
      console.error('Error archiving capsule:', err)
      alert('Failed to archive capsule')
    }
  }

  const togglePermission = (permission: string) => {
    if (participantPermissions.includes(permission)) {
      setParticipantPermissions(participantPermissions.filter(p => p !== permission))
    } else {
      setParticipantPermissions([...participantPermissions, permission])
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
          <div className="text-slate-400">Loading collaboration capsules...</div>
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
        <h1 className="text-3xl font-bold text-white">Collaboration Capsules</h1>
        <p className="text-slate-400 mt-1">
          Zero-trust ephemeral collaboration spaces for cross-organization projects
        </p>
      </div>

      {/* Main Content */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* Capsules List */}
        <div className="lg:col-span-1">
          <div className="card">
            <div className="flex items-center justify-between mb-4">
              <h2 className="text-lg font-semibold text-white">My Capsules</h2>
              <button
                onClick={() => setShowCreateCapsule(true)}
                className="btn btn-sm btn-primary flex items-center gap-1"
              >
                <PlusIcon className="w-4 h-4" />
                Create
              </button>
            </div>

            {capsules.length === 0 ? (
              <div className="text-center py-8 text-slate-400">
                <UserGroupIcon className="w-12 h-12 mx-auto mb-3 opacity-50" />
                <p className="text-sm">No capsules yet</p>
              </div>
            ) : (
              <div className="space-y-2">
                {capsules.map((capsule) => {
                  const isSelected = selectedCapsule?.capsule_id === capsule.capsule_id
                  return (
                    <button
                      key={capsule.capsule_id}
                      onClick={() => setSelectedCapsule(capsule)}
                      className={`w-full p-3 rounded-lg border text-left transition-colors ${
                        isSelected
                          ? 'bg-primary-900/30 border-primary-600'
                          : 'bg-slate-900/50 border-slate-700 hover:border-slate-600'
                      }`}
                    >
                      <div className="flex items-start gap-3 mb-2">
                        <div className={`p-2 rounded-lg ${
                          capsule.status === 'active'
                            ? 'bg-primary-900/30'
                            : capsule.status === 'expired'
                            ? 'bg-yellow-900/30'
                            : 'bg-red-900/30'
                        }`}>
                          <UserGroupIcon className={`w-5 h-5 ${
                            capsule.status === 'active'
                              ? 'text-primary-400'
                              : capsule.status === 'expired'
                              ? 'text-yellow-400'
                              : 'text-red-400'
                          }`} />
                        </div>
                        <div className="flex-1 min-w-0">
                          <div className="font-medium text-white truncate">{capsule.name}</div>
                          <div className="text-xs text-slate-400">
                            {capsule.participants.length} participants • {capsule.resources.length} resources
                          </div>
                        </div>
                      </div>
                      <span className={`badge text-xs ${
                        capsule.status === 'active'
                          ? 'badge-success'
                          : capsule.status === 'expired'
                          ? 'badge-warning'
                          : capsule.status === 'suspended'
                          ? 'badge-error'
                          : 'bg-slate-700 text-slate-300'
                      }`}>
                        {capsule.status.toUpperCase()}
                      </span>
                    </button>
                  )
                })}
              </div>
            )}
          </div>
        </div>

        {/* Capsule Details */}
        <div className="lg:col-span-2 space-y-6">
          {selectedCapsule ? (
            <>
              {/* Capsule Info */}
              <div className="card">
                <div className="flex items-start justify-between mb-6">
                  <div>
                    <h2 className="text-xl font-bold text-white">{selectedCapsule.name}</h2>
                    {selectedCapsule.description && (
                      <p className="text-slate-400 mt-1">{selectedCapsule.description}</p>
                    )}
                  </div>
                  <div className="flex gap-2">
                    {selectedCapsule.status === 'active' && (
                      <>
                        <button
                          onClick={() => handleSuspendCapsule(selectedCapsule.capsule_id)}
                          className="btn btn-sm bg-yellow-900/30 text-yellow-400 hover:bg-yellow-900/50 border border-yellow-700"
                        >
                          <ExclamationTriangleIcon className="w-4 h-4" />
                        </button>
                        <button
                          onClick={() => handleArchiveCapsule(selectedCapsule.capsule_id)}
                          className="btn btn-sm bg-red-900/30 text-red-400 hover:bg-red-900/50 border border-red-700"
                        >
                          <ArchiveBoxIcon className="w-4 h-4" />
                        </button>
                      </>
                    )}
                  </div>
                </div>

                <div className="grid grid-cols-2 gap-4 mb-6">
                  <div>
                    <label className="label">Created</label>
                    <div className="text-white">{formatTimestamp(selectedCapsule.created_at)}</div>
                  </div>
                  <div>
                    <label className="label">Expires</label>
                    <div className="text-white">
                      {selectedCapsule.expires_at
                        ? formatTimestamp(selectedCapsule.expires_at)
                        : 'Never'}
                    </div>
                  </div>
                </div>

                <div className="grid grid-cols-3 gap-4">
                  <div className="text-center p-4 bg-slate-900/50 rounded-lg">
                    <div className="text-2xl font-bold text-white">{selectedCapsule.participants.length}</div>
                    <div className="text-sm text-slate-400">Participants</div>
                  </div>
                  <div className="text-center p-4 bg-slate-900/50 rounded-lg">
                    <div className="text-2xl font-bold text-white">{selectedCapsule.resources.length}</div>
                    <div className="text-sm text-slate-400">Resources</div>
                  </div>
                  <div className="text-center p-4 bg-slate-900/50 rounded-lg">
                    <div className="text-2xl font-bold text-white">{selectedCapsule.policies.length}</div>
                    <div className="text-sm text-slate-400">Policies</div>
                  </div>
                </div>
              </div>

              {/* Participants */}
              <div className="card">
                <div className="flex items-center justify-between mb-6">
                  <h3 className="text-lg font-semibold text-white">Participants</h3>
                  {selectedCapsule.status === 'active' && (
                    <button
                      onClick={() => setShowAddParticipant(true)}
                      className="btn btn-sm btn-primary flex items-center gap-1"
                    >
                      <UserPlusIcon className="w-4 h-4" />
                      Add
                    </button>
                  )}
                </div>

                <div className="space-y-3">
                  {selectedCapsule.participants.map((participant) => (
                    <div
                      key={participant.identity_id}
                      className="p-4 bg-slate-900/50 rounded-lg border border-slate-700"
                    >
                      <div className="flex items-start justify-between">
                        <div className="flex-1">
                          <div className="flex items-center gap-2 mb-1">
                            <span className="text-white font-medium">{participant.identity_id}</span>
                            <span className="badge badge-success">{participant.role.toUpperCase()}</span>
                          </div>
                          <div className="text-sm text-slate-400 mb-2">
                            Joined: {formatTimestamp(participant.joined_at)}
                            {participant.last_active && (
                              <span className="ml-4">Last active: {formatTimestamp(participant.last_active)}</span>
                            )}
                          </div>
                          <div className="flex flex-wrap gap-2">
                            {participant.permissions.map((perm) => (
                              <span
                                key={perm}
                                className="px-2 py-1 bg-primary-900/30 text-primary-300 text-xs rounded border border-primary-700"
                              >
                                {perm}
                              </span>
                            ))}
                          </div>
                        </div>
                      </div>
                    </div>
                  ))}
                </div>
              </div>

              {/* Resources */}
              <div className="card">
                <div className="flex items-center justify-between mb-6">
                  <h3 className="text-lg font-semibold text-white">Resources</h3>
                  {selectedCapsule.status === 'active' && (
                    <button
                      onClick={() => setShowAddResource(true)}
                      className="btn btn-sm btn-primary flex items-center gap-1"
                    >
                      <FolderPlusIcon className="w-4 h-4" />
                      Add
                    </button>
                  )}
                </div>

                {selectedCapsule.resources.length === 0 ? (
                  <div className="text-center py-8 text-slate-400">
                    <DocumentIcon className="w-12 h-12 mx-auto mb-3 opacity-50" />
                    <p className="text-sm">No resources added</p>
                  </div>
                ) : (
                  <div className="space-y-3">
                    {selectedCapsule.resources.map((resource) => (
                      <div
                        key={resource.resource_id}
                        className="p-4 bg-slate-900/50 rounded-lg border border-slate-700"
                      >
                        <div className="flex items-start justify-between">
                          <div className="flex-1">
                            <div className="flex items-center gap-2 mb-1">
                              <DocumentIcon className="w-5 h-5 text-primary-400" />
                              <span className="text-white font-medium">{resource.resource_type}</span>
                              <span className={`badge ${
                                resource.access_level === 'admin'
                                  ? 'badge-error'
                                  : resource.access_level === 'write'
                                  ? 'badge-warning'
                                  : 'badge-success'
                              }`}>
                                {resource.access_level.toUpperCase()}
                              </span>
                            </div>
                            <div className="text-sm text-slate-400 font-mono break-all">
                              {resource.resource_ref}
                            </div>
                            <div className="text-xs text-slate-500 mt-2">
                              Added: {formatTimestamp(resource.added_at)}
                            </div>
                          </div>
                        </div>
                      </div>
                    ))}
                  </div>
                )}
              </div>

              {/* Activity Log */}
              <div className="card">
                <h3 className="text-lg font-semibold text-white mb-6">Activity Log</h3>
                {capsuleActivity.length === 0 ? (
                  <div className="text-center py-8 text-slate-400">
                    <ClockIcon className="w-12 h-12 mx-auto mb-3 opacity-50" />
                    <p className="text-sm">No activity recorded</p>
                  </div>
                ) : (
                  <div className="space-y-2">
                    {capsuleActivity.map((activity) => (
                      <div
                        key={activity.activity_id}
                        className="p-3 bg-slate-900/50 rounded-lg border border-slate-700"
                      >
                        <div className="flex items-start justify-between">
                          <div className="flex-1">
                            <div className="flex items-center gap-2 mb-1">
                              <span className="text-white font-medium">{activity.action}</span>
                              <span className={`badge text-xs ${
                                activity.result === 'allowed'
                                  ? 'badge-success'
                                  : 'badge-error'
                              }`}>
                                {activity.result.toUpperCase()}
                              </span>
                            </div>
                            {activity.resource_id && (
                              <div className="text-sm text-slate-400 mb-1">
                                Resource: {activity.resource_id}
                              </div>
                            )}
                            <div className="text-xs text-slate-500">
                              {formatTimestamp(activity.timestamp)} • {activity.identity_id.substring(0, 16)}...
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
                <LockClosedIcon className="w-16 h-16 mx-auto mb-4 opacity-50" />
                <p>Select a capsule to view details</p>
              </div>
            </div>
          )}
        </div>
      </div>

      {/* Create Capsule Modal */}
      {showCreateCapsule && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="card max-w-lg w-full mx-4">
            <div className="flex items-center justify-between mb-6">
              <h2 className="text-xl font-semibold text-white">Create Collaboration Capsule</h2>
              <button
                onClick={() => setShowCreateCapsule(false)}
                className="text-slate-400 hover:text-white"
              >
                <XCircleIcon className="w-6 h-6" />
              </button>
            </div>

            <div className="space-y-4">
              <div>
                <label className="label">Capsule Name</label>
                <input
                  type="text"
                  value={capsuleName}
                  onChange={(e) => setCapsuleName(e.target.value)}
                  placeholder="e.g., Q1 Partnership Project"
                  className="input w-full"
                />
              </div>

              <div>
                <label className="label">Description (Optional)</label>
                <textarea
                  value={capsuleDescription}
                  onChange={(e) => setCapsuleDescription(e.target.value)}
                  placeholder="Describe the purpose of this collaboration capsule..."
                  className="input w-full h-24"
                />
              </div>

              <div className="bg-primary-900/20 border border-primary-700 rounded-lg p-4">
                <div className="flex items-start gap-3">
                  <ShieldCheckIcon className="w-5 h-5 text-primary-400 flex-shrink-0 mt-0.5" />
                  <div className="text-sm">
                    <div className="text-primary-300 font-medium mb-1">Zero-Trust Architecture</div>
                    <div className="text-slate-400 text-xs">
                      Capsules are ephemeral, identity-bound, and enforce least-privilege access.
                      All actions are cryptographically logged and auditable.
                    </div>
                  </div>
                </div>
              </div>

              <div className="flex gap-3">
                <button
                  onClick={() => setShowCreateCapsule(false)}
                  className="btn btn-secondary flex-1"
                >
                  Cancel
                </button>
                <button
                  onClick={handleCreateCapsule}
                  disabled={!capsuleName.trim()}
                  className="btn btn-primary flex-1"
                >
                  Create Capsule
                </button>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Add Participant Modal */}
      {showAddParticipant && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="card max-w-lg w-full mx-4">
            <div className="flex items-center justify-between mb-6">
              <h2 className="text-xl font-semibold text-white">Add Participant</h2>
              <button
                onClick={() => setShowAddParticipant(false)}
                className="text-slate-400 hover:text-white"
              >
                <XCircleIcon className="w-6 h-6" />
              </button>
            </div>

            <div className="space-y-4">
              <div>
                <label className="label">Email or Identity ID</label>
                <input
                  type="text"
                  value={participantEmail}
                  onChange={(e) => setParticipantEmail(e.target.value)}
                  placeholder="user@example.com"
                  className="input w-full"
                />
              </div>

              <div>
                <label className="label">Role</label>
                <select
                  value={participantRole}
                  onChange={(e) => setParticipantRole(e.target.value)}
                  className="input w-full"
                >
                  <option value="member">Member</option>
                  <option value="contributor">Contributor</option>
                  <option value="admin">Admin</option>
                </select>
              </div>

              <div>
                <label className="label">Permissions</label>
                <div className="space-y-2">
                  {['read', 'write', 'delete', 'invite', 'manage'].map((perm) => (
                    <label
                      key={perm}
                      className="flex items-center gap-3 p-3 bg-slate-900/50 rounded-lg border border-slate-700 cursor-pointer hover:border-primary-600"
                    >
                      <input
                        type="checkbox"
                        checked={participantPermissions.includes(perm)}
                        onChange={() => togglePermission(perm)}
                        className="form-checkbox h-4 w-4 text-primary-500 rounded border-slate-600"
                      />
                      <span className="text-white">{perm.toUpperCase()}</span>
                    </label>
                  ))}
                </div>
              </div>

              <div className="flex gap-3">
                <button
                  onClick={() => setShowAddParticipant(false)}
                  className="btn btn-secondary flex-1"
                >
                  Cancel
                </button>
                <button
                  onClick={handleAddParticipant}
                  disabled={!participantEmail.trim()}
                  className="btn btn-primary flex-1"
                >
                  Add Participant
                </button>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Add Resource Modal */}
      {showAddResource && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="card max-w-lg w-full mx-4">
            <div className="flex items-center justify-between mb-6">
              <h2 className="text-xl font-semibold text-white">Add Resource</h2>
              <button
                onClick={() => setShowAddResource(false)}
                className="text-slate-400 hover:text-white"
              >
                <XCircleIcon className="w-6 h-6" />
              </button>
            </div>

            <div className="space-y-4">
              <div>
                <label className="label">Resource Type</label>
                <select
                  value={resourceType}
                  onChange={(e) => setResourceType(e.target.value)}
                  className="input w-full"
                >
                  <option value="document">Document</option>
                  <option value="contract">Smart Contract</option>
                  <option value="repository">Code Repository</option>
                  <option value="dataset">Dataset</option>
                  <option value="api_endpoint">API Endpoint</option>
                </select>
              </div>

              <div>
                <label className="label">Resource Reference</label>
                <input
                  type="text"
                  value={resourceRef}
                  onChange={(e) => setResourceRef(e.target.value)}
                  placeholder="e.g., doc_12345, contract_abc, https://api.example.com/data"
                  className="input w-full"
                />
              </div>

              <div>
                <label className="label">Access Level</label>
                <select
                  value={resourceAccess}
                  onChange={(e) => setResourceAccess(e.target.value as any)}
                  className="input w-full"
                >
                  <option value="read">Read Only</option>
                  <option value="write">Read & Write</option>
                  <option value="admin">Admin</option>
                </select>
              </div>

              <div className="flex gap-3">
                <button
                  onClick={() => setShowAddResource(false)}
                  className="btn btn-secondary flex-1"
                >
                  Cancel
                </button>
                <button
                  onClick={handleAddResource}
                  disabled={!resourceRef.trim()}
                  className="btn btn-primary flex-1"
                >
                  Add Resource
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
