'use client'

import { useState, useEffect } from 'react'
import { api } from '@/lib/api'
import type { IdentityProfile, IdentityAttestation, HardwarePass } from '@/types'
import {
  ShieldCheckIcon,
  CheckCircleIcon,
  XCircleIcon,
  ClockIcon,
  PlusIcon,
  KeyIcon,
  FingerprintIcon,
  DevicePhoneMobileIcon,
  CpuChipIcon,
  WifiIcon,
  LockClosedIcon,
  TrashIcon,
} from '@heroicons/react/24/outline'

export default function IdentityPage() {
  const [identity, setIdentity] = useState<IdentityProfile | null>(null)
  const [attestations, setAttestations] = useState<IdentityAttestation[]>([])
  const [hardwarePasses, setHardwarePasses] = useState<HardwarePass[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState('')
  const [showRegisterDevice, setShowRegisterDevice] = useState(false)
  const [deviceName, setDeviceName] = useState('')
  const [deviceCapabilities, setDeviceCapabilities] = useState<string[]>([])
  const [registering, setRegistering] = useState(false)

  useEffect(() => {
    loadIdentityData()
  }, [])

  const loadIdentityData = async () => {
    try {
      const userStr = localStorage.getItem('user_identity')
      if (!userStr) {
        setError('No user session found')
        setLoading(false)
        return
      }

      const user: IdentityProfile = JSON.parse(userStr)
      setIdentity(user)

      // Load attestations
      const attestationResponse = await api.getAttestations(user.identity_id)
      if (attestationResponse.error) {
        console.error('Error loading attestations:', attestationResponse.error)
      } else if (attestationResponse.data) {
        setAttestations(attestationResponse.data)
      }

      // Load hardware passes
      const passesResponse = await api.getHardwarePasses(user.identity_id)
      if (passesResponse.error) {
        console.error('Error loading hardware passes:', passesResponse.error)
      } else if (passesResponse.data) {
        setHardwarePasses(passesResponse.data)
      }

      setLoading(false)
    } catch (err) {
      console.error('Error loading identity data:', err)
      setError('Failed to load identity data')
      setLoading(false)
    }
  }

  const handleRegisterDevice = async () => {
    if (!identity || !deviceName.trim()) return

    setRegistering(true)
    try {
      // In production, the public key would be extracted from the NFC device via WebNFC API
      // or provided by a hardware security module. For now, we request it from the backend
      // which will generate a proper PQC key pair (ML-KEM-768 or ML-DSA-44) and provision
      // it to the hardware device securely.

      // The backend API endpoint handles the secure key generation and device provisioning
      const response = await api.registerHardwareDevice({
        identity_id: identity.identity_id,
        device_name: deviceName.trim(),
        public_key: '', // Backend generates PQC keypair and returns public key
        capabilities: deviceCapabilities,
      })

      if (response.error) {
        alert(`Failed to register device: ${response.error}`)
      } else if (response.data) {
        setHardwarePasses([...hardwarePasses, response.data])
        setShowRegisterDevice(false)
        setDeviceName('')
        setDeviceCapabilities([])
      }
    } catch (err) {
      console.error('Error registering device:', err)
      alert('Failed to register device')
    } finally {
      setRegistering(false)
    }
  }

  const handleRevokeDevice = async (deviceId: string) => {
    if (!confirm('Are you sure you want to revoke this device? This action cannot be undone.')) {
      return
    }

    try {
      const response = await api.revokeHardwareDevice(deviceId)
      if (response.error) {
        alert(`Failed to revoke device: ${response.error}`)
      } else {
        setHardwarePasses(hardwarePasses.filter(p => p.device_id !== deviceId))
      }
    } catch (err) {
      console.error('Error revoking device:', err)
      alert('Failed to revoke device')
    }
  }

  const toggleCapability = (capability: string) => {
    if (deviceCapabilities.includes(capability)) {
      setDeviceCapabilities(deviceCapabilities.filter(c => c !== capability))
    } else {
      setDeviceCapabilities([...deviceCapabilities, capability])
    }
  }

  const getKycStatusBadge = (status: string) => {
    const badges = {
      verified: { class: 'badge-success', icon: CheckCircleIcon, text: 'Verified' },
      pending: { class: 'badge-warning', icon: ClockIcon, text: 'Pending' },
      rejected: { class: 'badge-error', icon: XCircleIcon, text: 'Rejected' },
      revoked: { class: 'badge-error', icon: XCircleIcon, text: 'Revoked' },
    }
    const badge = badges[status as keyof typeof badges] || badges.pending
    return (
      <span className={`badge ${badge.class} flex items-center gap-1`}>
        <badge.icon className="w-4 h-4" />
        {badge.text}
      </span>
    )
  }

  const getAttestationStatusBadge = (status: string) => {
    const badges = {
      valid: { class: 'badge-success', text: 'Valid' },
      expired: { class: 'badge-warning', text: 'Expired' },
      revoked: { class: 'badge-error', text: 'Revoked' },
    }
    const badge = badges[status as keyof typeof badges] || badges.valid
    return <span className={`badge ${badge.class}`}>{badge.text}</span>
  }

  const formatAttestationType = (type: any): string => {
    if (typeof type === 'string') {
      return type.replace(/_/g, ' ').toUpperCase()
    }
    if (typeof type === 'object' && type.custom) {
      return type.custom.toUpperCase()
    }
    return 'UNKNOWN'
  }

  const getRiskLabel = (score: number): { text: string; class: string } => {
    if (score < 30) return { text: 'Low Risk', class: 'badge-success' }
    if (score < 70) return { text: 'Medium Risk', class: 'badge-warning' }
    return { text: 'High Risk', class: 'badge-error' }
  }

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="flex flex-col items-center gap-4">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-primary-500"></div>
          <div className="text-slate-400">Loading identity data...</div>
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

  const riskLabel = getRiskLabel(identity.aml_risk_score)

  return (
    <div className="space-y-6">
      {/* Page Header */}
      <div>
        <h1 className="text-3xl font-bold text-white">Identity & KYC</h1>
        <p className="text-slate-400 mt-1">
          Manage your verified identity and attestations
        </p>
      </div>

      {/* Identity Card */}
      <div className="card">
        <div className="flex items-start justify-between mb-6">
          <div className="flex items-center gap-4">
            <div className="w-16 h-16 bg-gradient-to-br from-primary-500 to-secondary-600 rounded-full flex items-center justify-center">
              <ShieldCheckIcon className="w-8 h-8 text-white" />
            </div>
            <div>
              <h2 className="text-2xl font-bold text-white">
                {identity.legal_name}
              </h2>
              <p className="text-slate-400">{identity.email}</p>
            </div>
          </div>
          {getKycStatusBadge(identity.kyc_status)}
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          <div>
            <label className="label">Organization</label>
            <div className="text-white">{identity.org_name || 'Individual'}</div>
          </div>
          <div>
            <label className="label">Root PKI Key ID</label>
            <div className="text-white font-mono text-sm break-all">
              {identity.root_pki_key_id}
            </div>
          </div>
          <div>
            <label className="label">AML Risk Score</label>
            <div className="flex items-center gap-2">
              <div className="text-white font-semibold">
                {identity.aml_risk_score.toFixed(1)}
              </div>
              <span className={`badge ${riskLabel.class}`}>{riskLabel.text}</span>
            </div>
          </div>
          <div>
            <label className="label">Member Since</label>
            <div className="text-white">
              {new Date(identity.created_at).toLocaleDateString('en-US', {
                year: 'numeric',
                month: 'long',
                day: 'numeric'
              })}
            </div>
          </div>
          <div className="md:col-span-2">
            <label className="label">Identity ID</label>
            <div className="text-white font-mono text-sm break-all">
              {identity.identity_id}
            </div>
          </div>
        </div>
      </div>

      {/* CIVA 3-Layer Attestation Model */}
      <div className="card">
        <div className="flex items-center justify-between mb-6">
          <div>
            <h2 className="text-xl font-semibold text-white">CIVA Attestations</h2>
            <p className="text-sm text-slate-400 mt-1">
              3-Layer Consumer Identity Virtual Attestation Model
            </p>
          </div>
          <button className="btn btn-primary flex items-center gap-2">
            <PlusIcon className="w-5 h-5" />
            Request Attestation
          </button>
        </div>

        {attestations.length === 0 ? (
          <div className="text-center py-12 text-slate-400">
            <ShieldCheckIcon className="w-16 h-16 mx-auto mb-4 opacity-50" />
            <p>No attestations yet</p>
            <p className="text-sm mt-2">
              Build your CIVA profile with identity proofs, compliance checks, and capability credentials
            </p>
          </div>
        ) : (
          <div className="space-y-6">
            {/* Layer 1: Identity Proof */}
            <div>
              <div className="flex items-center gap-2 mb-4">
                <div className="w-8 h-8 bg-green-900/30 rounded-full flex items-center justify-center">
                  <span className="text-green-400 font-bold text-sm">1</span>
                </div>
                <div>
                  <h3 className="font-semibold text-white">Layer 1: Identity Proof</h3>
                  <p className="text-xs text-slate-400">KYC, Biometrics, Government IDs</p>
                </div>
              </div>
              <div className="space-y-3 ml-10">
                {attestations
                  .filter(a => {
                    const type = formatAttestationType(a.attestation_type).toLowerCase()
                    return type.includes('kyc') || type.includes('identity') || type.includes('biometric') || type.includes('government')
                  })
                  .map((attestation) => (
                    <div
                      key={attestation.attestation_id}
                      className="p-4 bg-slate-900/50 rounded-lg border border-green-900/30"
                    >
                      <div className="flex items-start justify-between">
                        <div className="flex-1">
                          <div className="flex items-center gap-3 mb-2 flex-wrap">
                            <span className="badge bg-green-900/50 text-green-300 border border-green-700">
                              {formatAttestationType(attestation.attestation_type)}
                            </span>
                            {getAttestationStatusBadge(attestation.status)}
                            {attestation.chain_anchor_tx && (
                              <span className="badge bg-primary-900/50 text-primary-300 border border-primary-700">
                                On-Chain
                              </span>
                            )}
                          </div>
                          <div className="text-white font-medium mb-1">
                            Issuer: {attestation.issuer}
                          </div>
                          <div className="text-sm text-slate-400">
                            Valid from: {new Date(attestation.valid_from).toLocaleDateString()}
                            {attestation.valid_to && (
                              <span className="ml-4">
                                Expires: {new Date(attestation.valid_to).toLocaleDateString()}
                              </span>
                            )}
                          </div>
                          {attestation.evidence_refs.length > 0 && (
                            <div className="text-xs text-slate-500 mt-2">
                              {attestation.evidence_refs.length} evidence document(s)
                            </div>
                          )}
                        </div>
                      </div>
                    </div>
                  ))}
                {attestations.filter(a => {
                  const type = formatAttestationType(a.attestation_type).toLowerCase()
                  return type.includes('kyc') || type.includes('identity') || type.includes('biometric') || type.includes('government')
                }).length === 0 && (
                  <div className="text-sm text-slate-500 italic">No identity proof attestations</div>
                )}
              </div>
            </div>

            {/* Layer 2: Risk & Compliance */}
            <div>
              <div className="flex items-center gap-2 mb-4">
                <div className="w-8 h-8 bg-yellow-900/30 rounded-full flex items-center justify-center">
                  <span className="text-yellow-400 font-bold text-sm">2</span>
                </div>
                <div>
                  <h3 className="font-semibold text-white">Layer 2: Risk & Compliance</h3>
                  <p className="text-xs text-slate-400">AML, Sanctions Screening, PEP, Fraud Scoring</p>
                </div>
              </div>
              <div className="space-y-3 ml-10">
                {attestations
                  .filter(a => {
                    const type = formatAttestationType(a.attestation_type).toLowerCase()
                    return type.includes('aml') || type.includes('sanction') || type.includes('pep') || type.includes('compliance') || type.includes('fraud')
                  })
                  .map((attestation) => (
                    <div
                      key={attestation.attestation_id}
                      className="p-4 bg-slate-900/50 rounded-lg border border-yellow-900/30"
                    >
                      <div className="flex items-start justify-between">
                        <div className="flex-1">
                          <div className="flex items-center gap-3 mb-2 flex-wrap">
                            <span className="badge bg-yellow-900/50 text-yellow-300 border border-yellow-700">
                              {formatAttestationType(attestation.attestation_type)}
                            </span>
                            {getAttestationStatusBadge(attestation.status)}
                            {attestation.chain_anchor_tx && (
                              <span className="badge bg-primary-900/50 text-primary-300 border border-primary-700">
                                On-Chain
                              </span>
                            )}
                          </div>
                          <div className="text-white font-medium mb-1">
                            Issuer: {attestation.issuer}
                          </div>
                          <div className="text-sm text-slate-400">
                            Valid from: {new Date(attestation.valid_from).toLocaleDateString()}
                            {attestation.valid_to && (
                              <span className="ml-4">
                                Expires: {new Date(attestation.valid_to).toLocaleDateString()}
                              </span>
                            )}
                          </div>
                        </div>
                      </div>
                    </div>
                  ))}
                {attestations.filter(a => {
                  const type = formatAttestationType(a.attestation_type).toLowerCase()
                  return type.includes('aml') || type.includes('sanction') || type.includes('pep') || type.includes('compliance') || type.includes('fraud')
                }).length === 0 && (
                  <div className="text-sm text-slate-500 italic">No compliance attestations</div>
                )}
              </div>
            </div>

            {/* Layer 3: Attributes & Capabilities */}
            <div>
              <div className="flex items-center gap-2 mb-4">
                <div className="w-8 h-8 bg-blue-900/30 rounded-full flex items-center justify-center">
                  <span className="text-blue-400 font-bold text-sm">3</span>
                </div>
                <div>
                  <h3 className="font-semibold text-white">Layer 3: Attributes & Capabilities</h3>
                  <p className="text-xs text-slate-400">Credentials, Licenses, Memberships, Qualifications</p>
                </div>
              </div>
              <div className="space-y-3 ml-10">
                {attestations
                  .filter(a => {
                    const type = formatAttestationType(a.attestation_type).toLowerCase()
                    return !type.includes('kyc') && !type.includes('identity') && !type.includes('biometric') &&
                           !type.includes('government') && !type.includes('aml') && !type.includes('sanction') &&
                           !type.includes('pep') && !type.includes('compliance') && !type.includes('fraud')
                  })
                  .map((attestation) => (
                    <div
                      key={attestation.attestation_id}
                      className="p-4 bg-slate-900/50 rounded-lg border border-blue-900/30"
                    >
                      <div className="flex items-start justify-between">
                        <div className="flex-1">
                          <div className="flex items-center gap-3 mb-2 flex-wrap">
                            <span className="badge bg-blue-900/50 text-blue-300 border border-blue-700">
                              {formatAttestationType(attestation.attestation_type)}
                            </span>
                            {getAttestationStatusBadge(attestation.status)}
                            {attestation.chain_anchor_tx && (
                              <span className="badge bg-primary-900/50 text-primary-300 border border-primary-700">
                                On-Chain
                              </span>
                            )}
                          </div>
                          <div className="text-white font-medium mb-1">
                            Issuer: {attestation.issuer}
                          </div>
                          <div className="text-sm text-slate-400">
                            Valid from: {new Date(attestation.valid_from).toLocaleDateString()}
                            {attestation.valid_to && (
                              <span className="ml-4">
                                Expires: {new Date(attestation.valid_to).toLocaleDateString()}
                              </span>
                            )}
                          </div>
                        </div>
                      </div>
                    </div>
                  ))}
                {attestations.filter(a => {
                  const type = formatAttestationType(a.attestation_type).toLowerCase()
                  return !type.includes('kyc') && !type.includes('identity') && !type.includes('biometric') &&
                         !type.includes('government') && !type.includes('aml') && !type.includes('sanction') &&
                         !type.includes('pep') && !type.includes('compliance') && !type.includes('fraud')
                }).length === 0 && (
                  <div className="text-sm text-slate-500 italic">No capability attestations</div>
                )}
              </div>
            </div>
          </div>
        )}
      </div>

      {/* NFC / Hardware Pass Management */}
      <div className="card">
        <div className="flex items-center justify-between mb-6">
          <div>
            <h2 className="text-xl font-semibold text-white">NFC / Hardware Pass</h2>
            <p className="text-sm text-slate-400 mt-1">
              Physical multipass cards for offline verification and secure operations
            </p>
          </div>
          <button
            onClick={() => setShowRegisterDevice(true)}
            className="btn btn-primary flex items-center gap-2"
          >
            <PlusIcon className="w-5 h-5" />
            Register Device
          </button>
        </div>

        {hardwarePasses.length === 0 ? (
          <div className="text-center py-12 text-slate-400">
            <CpuChipIcon className="w-16 h-16 mx-auto mb-4 opacity-50" />
            <p>No hardware devices registered</p>
            <p className="text-sm mt-2">
              Register NFC cards or security keys for offline verification and secure login
            </p>
          </div>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            {hardwarePasses.map((pass) => (
              <div
                key={pass.device_id}
                className={`p-4 rounded-lg border ${
                  pass.status === 'active'
                    ? 'bg-slate-900/50 border-slate-700'
                    : pass.status === 'lost'
                    ? 'bg-yellow-900/10 border-yellow-700'
                    : 'bg-red-900/10 border-red-700'
                }`}
              >
                <div className="flex items-start justify-between mb-3">
                  <div className="flex items-center gap-3">
                    <div className={`p-2 rounded-lg ${
                      pass.status === 'active'
                        ? 'bg-primary-900/30'
                        : pass.status === 'lost'
                        ? 'bg-yellow-900/30'
                        : 'bg-red-900/30'
                    }`}>
                      <CpuChipIcon className={`w-6 h-6 ${
                        pass.status === 'active'
                          ? 'text-primary-400'
                          : pass.status === 'lost'
                          ? 'text-yellow-400'
                          : 'text-red-400'
                      }`} />
                    </div>
                    <div>
                      <div className="font-medium text-white">Device #{pass.device_id.substring(0, 8)}</div>
                      <div className="text-xs text-slate-400">
                        Registered {new Date(pass.issued_at).toLocaleDateString()}
                      </div>
                    </div>
                  </div>
                  <span className={`badge ${
                    pass.status === 'active'
                      ? 'badge-success'
                      : pass.status === 'lost'
                      ? 'badge-warning'
                      : 'badge-error'
                  }`}>
                    {pass.status.toUpperCase()}
                  </span>
                </div>

                <div className="mb-3">
                  <div className="text-xs text-slate-500 mb-1">Capabilities</div>
                  <div className="flex flex-wrap gap-2">
                    {pass.capabilities.map((cap) => (
                      <span
                        key={cap}
                        className="inline-flex items-center gap-1 px-2 py-1 bg-primary-900/30 text-primary-300 text-xs rounded border border-primary-700"
                      >
                        {cap === 'login_only' && <KeyIcon className="w-3 h-3" />}
                        {cap === 'sign_tx' && <LockClosedIcon className="w-3 h-3" />}
                        {cap === 'unlock_doors' && <WifiIcon className="w-3 h-3" />}
                        {cap === 'access_control' && <ShieldCheckIcon className="w-3 h-3" />}
                        {cap.replace(/_/g, ' ').toUpperCase()}
                      </span>
                    ))}
                  </div>
                </div>

                <div className="mb-3">
                  <div className="text-xs text-slate-500">Public Key</div>
                  <div className="text-xs text-slate-400 font-mono break-all">
                    {pass.public_key.substring(0, 40)}...
                  </div>
                </div>

                {pass.last_used && (
                  <div className="text-xs text-slate-500 mb-3">
                    Last used: {new Date(pass.last_used).toLocaleString()}
                  </div>
                )}

                {pass.status === 'active' && (
                  <button
                    onClick={() => handleRevokeDevice(pass.device_id)}
                    className="btn btn-sm bg-red-900/30 text-red-400 hover:bg-red-900/50 border border-red-700 w-full flex items-center justify-center gap-2"
                  >
                    <TrashIcon className="w-4 h-4" />
                    Revoke Device
                  </button>
                )}
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Security Settings */}
      <div className="card">
        <h2 className="text-xl font-semibold text-white mb-6">
          Security Settings
        </h2>
        <div className="space-y-4">
          <div className="flex items-center justify-between p-4 bg-slate-900/50 rounded-lg border border-slate-700">
            <div className="flex items-center gap-3">
              <FingerprintIcon className="w-6 h-6 text-primary-400" />
              <div>
                <div className="font-medium text-white">WebAuthn / Biometrics</div>
                <div className="text-sm text-slate-400">
                  Use fingerprint or face recognition
                </div>
              </div>
            </div>
            <button className="btn btn-secondary">Configure</button>
          </div>
          <div className="flex items-center justify-between p-4 bg-slate-900/50 rounded-lg border border-slate-700">
            <div className="flex items-center gap-3">
              <KeyIcon className="w-6 h-6 text-primary-400" />
              <div>
                <div className="font-medium text-white">Session Management</div>
                <div className="text-sm text-slate-400">
                  View and revoke active sessions
                </div>
              </div>
            </div>
            <button className="btn btn-secondary">Manage Sessions</button>
          </div>
        </div>
      </div>

      {/* Register Device Modal */}
      {showRegisterDevice && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="card max-w-lg w-full mx-4">
            <div className="flex items-center justify-between mb-6">
              <h2 className="text-xl font-semibold text-white">Register Hardware Device</h2>
              <button
                onClick={() => {
                  setShowRegisterDevice(false)
                  setDeviceName('')
                  setDeviceCapabilities([])
                }}
                className="text-slate-400 hover:text-white"
              >
                <XCircleIcon className="w-6 h-6" />
              </button>
            </div>

            <div className="space-y-4">
              <div>
                <label className="label">Device Name</label>
                <input
                  type="text"
                  value={deviceName}
                  onChange={(e) => setDeviceName(e.target.value)}
                  placeholder="e.g., Office NFC Card, YubiKey 5"
                  className="input w-full"
                />
              </div>

              <div>
                <label className="label">Capabilities</label>
                <p className="text-xs text-slate-500 mb-2">
                  Select what this device can be used for
                </p>
                <div className="space-y-2">
                  {[
                    { value: 'login_only', label: 'Login Only', description: 'Basic authentication' },
                    { value: 'sign_tx', label: 'Sign Transactions', description: 'Approve blockchain transactions' },
                    { value: 'unlock_doors', label: 'Unlock Doors', description: 'Physical access control' },
                    { value: 'access_control', label: 'Access Control', description: 'Full access control capabilities' },
                  ].map((cap) => (
                    <label
                      key={cap.value}
                      className="flex items-center gap-3 p-3 bg-slate-900/50 rounded-lg border border-slate-700 cursor-pointer hover:border-primary-600"
                    >
                      <input
                        type="checkbox"
                        checked={deviceCapabilities.includes(cap.value)}
                        onChange={() => toggleCapability(cap.value)}
                        className="form-checkbox h-4 w-4 text-primary-500 rounded border-slate-600"
                      />
                      <div className="flex-1">
                        <div className="text-white font-medium">{cap.label}</div>
                        <div className="text-xs text-slate-400">{cap.description}</div>
                      </div>
                    </label>
                  ))}
                </div>
              </div>

              <div className="bg-primary-900/20 border border-primary-700 rounded-lg p-4">
                <div className="flex items-start gap-3">
                  <WifiIcon className="w-5 h-5 text-primary-400 flex-shrink-0 mt-0.5" />
                  <div className="text-sm">
                    <div className="text-primary-300 font-medium mb-1">Offline Verification</div>
                    <div className="text-slate-400 text-xs">
                      Hardware passes enable offline identity verification using NFC technology.
                      Tap your card to authenticate without internet connectivity.
                    </div>
                  </div>
                </div>
              </div>

              <div className="flex gap-3">
                <button
                  onClick={() => {
                    setShowRegisterDevice(false)
                    setDeviceName('')
                    setDeviceCapabilities([])
                  }}
                  className="btn btn-secondary flex-1"
                >
                  Cancel
                </button>
                <button
                  onClick={handleRegisterDevice}
                  disabled={!deviceName.trim() || deviceCapabilities.length === 0 || registering}
                  className="btn btn-primary flex-1"
                >
                  {registering ? 'Registering...' : 'Register Device'}
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
