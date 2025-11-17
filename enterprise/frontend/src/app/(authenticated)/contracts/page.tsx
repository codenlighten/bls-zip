'use client'

import { useState, useEffect } from 'react'
import { api } from '@/lib/api'
import type { IdentityProfile, ContractTemplate, DeployedContract, ContractParty } from '@/types'
import {
  DocumentTextIcon,
  PlusIcon,
  CheckCircleIcon,
  ClockIcon,
  XMarkIcon,
  DocumentCheckIcon,
  BuildingOfficeIcon,
  HomeIcon,
  UserGroupIcon,
  BriefcaseIcon,
  ShieldCheckIcon,
  PencilSquareIcon,
  HashtagIcon,
} from '@heroicons/react/24/outline'

export default function ContractsPage() {
  const [identity, setIdentity] = useState<IdentityProfile | null>(null)
  const [templates, setTemplates] = useState<ContractTemplate[]>([])
  const [deployedContracts, setDeployedContracts] = useState<DeployedContract[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState('')
  const [selectedCategory, setSelectedCategory] = useState<string>('all')

  // Modals
  const [showTemplateModal, setShowTemplateModal] = useState(false)
  const [showDeployModal, setShowDeployModal] = useState(false)
  const [showContractDetails, setShowContractDetails] = useState(false)
  const [selectedTemplate, setSelectedTemplate] = useState<ContractTemplate | null>(null)
  const [selectedContract, setSelectedContract] = useState<DeployedContract | null>(null)

  // Deploy form
  const [deployParams, setDeployParams] = useState<Record<string, any>>({})
  const [deployParties, setDeployParties] = useState<ContractParty[]>([])
  const [deploying, setDeploying] = useState(false)

  useEffect(() => {
    loadData()
  }, [])

  const loadData = async () => {
    try {
      setLoading(true)
      const userStr = localStorage.getItem('user_identity')
      if (!userStr) {
        setError('No user session found')
        setLoading(false)
        return
      }

      const user: IdentityProfile = JSON.parse(userStr)
      setIdentity(user)

      // Load contract templates
      const templatesResponse = await api.getContractTemplates()
      if (templatesResponse.data) {
        setTemplates(templatesResponse.data)
      }

      // Load deployed contracts
      const contractsResponse = await api.getDeployedContracts(user.identity_id)
      if (contractsResponse.data) {
        setDeployedContracts(contractsResponse.data)
      }

      setLoading(false)
    } catch (err) {
      console.error('Error loading contracts:', err)
      setError('Failed to load contract data')
      setLoading(false)
    }
  }

  const handleDeployContract = async () => {
    if (!selectedTemplate || !identity) return

    setDeploying(true)
    try {
      const response = await api.deployContract({
        template_id: selectedTemplate.template_id,
        deployer_id: identity.identity_id,
        parties: deployParties,
        parameters: deployParams,
      })

      if (!response.error) {
        setShowDeployModal(false)
        setSelectedTemplate(null)
        setDeployParams({})
        setDeployParties([])
        await loadData()
      }
    } catch (err) {
      console.error('Error deploying contract:', err)
    }
    setDeploying(false)
  }

  const handleSignContract = async (contract_id: string) => {
    if (!identity) return

    try {
      await api.signContract(contract_id, identity.identity_id)
      await loadData()
    } catch (err) {
      console.error('Error signing contract:', err)
    }
  }

  const getCategoryIcon = (category: any) => {
    if (typeof category === 'string') {
      switch (category) {
        case 'business':
          return BriefcaseIcon
        case 'real_estate':
          return HomeIcon
        case 'family':
        case 'personal':
          return UserGroupIcon
        case 'employment':
          return BuildingOfficeIcon
        default:
          return DocumentTextIcon
      }
    }
    return DocumentTextIcon
  }

  const formatCategory = (category: any): string => {
    if (typeof category === 'string') {
      return category.replace(/_/g, ' ').split(' ').map(w => w.charAt(0).toUpperCase() + w.slice(1)).join(' ')
    }
    if (typeof category === 'object' && category.custom) {
      return category.custom
    }
    return 'Unknown'
  }

  const getStatusBadge = (status: string) => {
    const badges = {
      draft: { class: 'badge bg-slate-700 text-slate-300', icon: PencilSquareIcon },
      pending_signatures: { class: 'badge-warning', icon: ClockIcon },
      active: { class: 'badge-success', icon: CheckCircleIcon },
      completed: { class: 'badge bg-blue-900/50 text-blue-300 border border-blue-700', icon: CheckCircleIcon },
      terminated: { class: 'badge bg-slate-700 text-slate-400', icon: XMarkIcon },
      disputed: { class: 'badge-error', icon: XMarkIcon },
    }
    return badges[status as keyof typeof badges] || badges.draft
  }

  const filteredTemplates = selectedCategory === 'all'
    ? templates
    : templates.filter(t => {
        const cat = typeof t.category === 'string' ? t.category : t.category.custom
        return cat === selectedCategory
      })

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="flex flex-col items-center gap-4">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-primary-500"></div>
          <div className="text-slate-400">Loading contracts...</div>
        </div>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      {/* Page Header */}
      <div>
        <h1 className="text-3xl font-bold text-white">Smart Contracts</h1>
        <p className="text-slate-400 mt-1">
          Template-based contracts with jurisdiction tagging and natural language terms
        </p>
      </div>

      {/* Stats */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <div className="card">
          <div className="text-sm text-slate-400 mb-1">Active Contracts</div>
          <div className="text-2xl font-bold text-white">
            {deployedContracts.filter(c => c.status === 'active').length}
          </div>
        </div>
        <div className="card">
          <div className="text-sm text-slate-400 mb-1">Pending Signatures</div>
          <div className="text-2xl font-bold text-yellow-400">
            {deployedContracts.filter(c => c.status === 'pending_signatures').length}
          </div>
        </div>
        <div className="card">
          <div className="text-sm text-slate-400 mb-1">Available Templates</div>
          <div className="text-2xl font-bold text-white">{templates.length}</div>
        </div>
        <div className="card">
          <div className="text-sm text-slate-400 mb-1">Total Contracts</div>
          <div className="text-2xl font-bold text-white">{deployedContracts.length}</div>
        </div>
      </div>

      {/* Deployed Contracts */}
      <div className="card">
        <div className="flex items-center justify-between mb-6">
          <h2 className="text-xl font-semibold text-white">My Contracts</h2>
          <button
            onClick={() => setShowTemplateModal(true)}
            className="btn btn-primary flex items-center gap-2"
          >
            <PlusIcon className="w-5 h-5" />
            Deploy Contract
          </button>
        </div>

        {deployedContracts.length === 0 ? (
          <div className="text-center py-12">
            <DocumentTextIcon className="w-16 h-16 mx-auto mb-4 text-slate-600" />
            <h3 className="text-lg font-semibold text-white mb-2">No Contracts Yet</h3>
            <p className="text-slate-400 mb-6">
              Deploy your first smart contract from our template library
            </p>
            <button
              onClick={() => setShowTemplateModal(true)}
              className="btn btn-secondary mx-auto"
            >
              Browse Templates
            </button>
          </div>
        ) : (
          <div className="space-y-4">
            {deployedContracts.map((contract) => {
              const Icon = getCategoryIcon(contract.template.category)
              const statusBadge = getStatusBadge(contract.status)
              const myParty = contract.parties.find(p => p.identity_id === identity?.identity_id)

              return (
                <div
                  key={contract.contract_id}
                  className="p-4 bg-slate-900/50 rounded-lg border border-slate-700 hover:border-primary-500/50 transition-colors cursor-pointer"
                  onClick={() => {
                    setSelectedContract(contract)
                    setShowContractDetails(true)
                  }}
                >
                  <div className="flex items-start gap-4">
                    <div className="w-12 h-12 bg-gradient-to-br from-primary-500 to-secondary-600 rounded-lg flex items-center justify-center shrink-0">
                      <Icon className="w-6 h-6 text-white" />
                    </div>
                    <div className="flex-1">
                      <div className="flex items-start justify-between mb-2">
                        <div>
                          <h3 className="font-semibold text-white">{contract.template.template_name}</h3>
                          <p className="text-sm text-slate-400">
                            {formatCategory(contract.template.category)} • Version {contract.template.version}
                          </p>
                        </div>
                        <div className="flex items-center gap-2">
                          <span className={`badge ${statusBadge.class}`}>
                            {contract.status.replace(/_/g, ' ').toUpperCase()}
                          </span>
                          {contract.chain_address && (
                            <span className="badge bg-primary-900/50 text-primary-300 border border-primary-700">
                              On-Chain
                            </span>
                          )}
                        </div>
                      </div>

                      <p className="text-sm text-slate-300 mb-3">{contract.natural_language_summary}</p>

                      <div className="flex items-center gap-4 text-sm">
                        <div className="text-slate-400">
                          Parties: {contract.parties.length}
                        </div>
                        <div className="text-slate-400">•</div>
                        <div className="text-slate-400">
                          Signatures: {contract.parties.filter(p => p.signed).length}/{contract.parties.length}
                        </div>
                        {myParty && !myParty.signed && contract.status === 'pending_signatures' && (
                          <>
                            <div className="text-slate-400">•</div>
                            <button
                              onClick={(e) => {
                                e.stopPropagation()
                                handleSignContract(contract.contract_id)
                              }}
                              className="text-primary-400 hover:text-primary-300 font-medium"
                            >
                              Sign Contract
                            </button>
                          </>
                        )}
                      </div>

                      {contract.template.jurisdiction.length > 0 && (
                        <div className="mt-2 text-xs text-slate-500">
                          Jurisdictions: {contract.template.jurisdiction.join(', ')}
                        </div>
                      )}
                    </div>
                  </div>
                </div>
              )
            })}
          </div>
        )}
      </div>

      {/* Template Modal */}
      {showTemplateModal && (
        <div className="fixed inset-0 bg-slate-900/80 backdrop-blur-sm z-50 flex items-center justify-center p-4">
          <div className="card max-w-5xl w-full max-h-[85vh] overflow-y-auto">
            <div className="flex items-center justify-between mb-6">
              <div>
                <h3 className="text-2xl font-bold text-white">Contract Templates</h3>
                <p className="text-sm text-slate-400 mt-1">Browse and deploy verified smart contract templates</p>
              </div>
              <button onClick={() => setShowTemplateModal(false)} className="text-slate-400 hover:text-white">
                <XMarkIcon className="w-6 h-6" />
              </button>
            </div>

            {/* Category Filter */}
            <div className="flex flex-wrap gap-2 mb-6">
              {['all', 'business', 'real_estate', 'employment', 'family', 'personal', 'service_agreement'].map((cat) => (
                <button
                  key={cat}
                  onClick={() => setSelectedCategory(cat)}
                  className={`btn btn-sm ${
                    selectedCategory === cat
                      ? 'btn-primary'
                      : 'bg-slate-800 text-slate-300 hover:bg-slate-700'
                  }`}
                >
                  {cat === 'all' ? 'All Categories' : formatCategory(cat)}
                </button>
              ))}
            </div>

            {/* Templates Grid */}
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              {filteredTemplates.map((template) => {
                const Icon = getCategoryIcon(template.category)

                return (
                  <div
                    key={template.template_id}
                    className="p-4 bg-slate-900/50 rounded-lg border border-slate-700 hover:border-primary-500/50 transition-colors"
                  >
                    <div className="flex items-start gap-3 mb-3">
                      <div className="w-12 h-12 bg-gradient-to-br from-primary-500 to-secondary-600 rounded-lg flex items-center justify-center shrink-0">
                        <Icon className="w-6 h-6 text-white" />
                      </div>
                      <div className="flex-1">
                        <div className="flex items-start justify-between">
                          <div>
                            <h4 className="font-semibold text-white">{template.template_name}</h4>
                            <p className="text-xs text-slate-400">
                              {formatCategory(template.category)} • v{template.version}
                            </p>
                          </div>
                          {template.is_verified && (
                            <CheckCircleIcon className="w-5 h-5 text-green-400" title="Verified Template" />
                          )}
                        </div>
                      </div>
                    </div>

                    <p className="text-sm text-slate-300 mb-3">{template.description}</p>

                    {template.jurisdiction.length > 0 && (
                      <div className="flex flex-wrap gap-1 mb-3">
                        {template.jurisdiction.map((jur, idx) => (
                          <span key={idx} className="badge bg-slate-800 text-slate-300 text-xs">
                            {jur}
                          </span>
                        ))}
                      </div>
                    )}

                    <div className="flex items-center gap-2 mb-3 text-xs">
                      <HashtagIcon className="w-4 h-4 text-slate-500" />
                      <span className="text-slate-500 font-mono">
                        {template.code_hash.substring(0, 16)}...
                      </span>
                    </div>

                    <button
                      onClick={() => {
                        setSelectedTemplate(template)
                        setShowTemplateModal(false)
                        setShowDeployModal(true)
                        // Initialize params with defaults
                        const params: Record<string, any> = {}
                        template.parameters.forEach(p => {
                          if (p.default_value !== undefined) {
                            params[p.param_name] = p.default_value
                          }
                        })
                        setDeployParams(params)
                      }}
                      className="w-full btn btn-primary btn-sm"
                    >
                      Deploy Contract
                    </button>
                  </div>
                )
              })}
            </div>

            {filteredTemplates.length === 0 && (
              <div className="text-center py-12 text-slate-400">
                No templates found in this category
              </div>
            )}
          </div>
        </div>
      )}

      {/* Deploy Modal */}
      {showDeployModal && selectedTemplate && (
        <div className="fixed inset-0 bg-slate-900/80 backdrop-blur-sm z-50 flex items-center justify-center p-4">
          <div className="card max-w-3xl w-full max-h-[85vh] overflow-y-auto">
            <div className="flex items-center justify-between mb-6">
              <div>
                <h3 className="text-xl font-bold text-white">Deploy: {selectedTemplate.template_name}</h3>
                <p className="text-sm text-slate-400">{formatCategory(selectedTemplate.category)} Contract</p>
              </div>
              <button onClick={() => {
                setShowDeployModal(false)
                setSelectedTemplate(null)
                setDeployParams({})
                setDeployParties([])
              }} className="text-slate-400 hover:text-white">
                <XMarkIcon className="w-6 h-6" />
              </button>
            </div>

            {/* Natural Language Terms */}
            <div className="mb-6">
              <h4 className="text-lg font-semibold text-white mb-3">Contract Terms</h4>
              <div className="p-4 bg-slate-900/50 rounded-lg border border-slate-700">
                <p className="text-sm text-slate-300 whitespace-pre-wrap">
                  {selectedTemplate.natural_language_terms}
                </p>
              </div>
            </div>

            {/* Parameters */}
            {selectedTemplate.parameters.length > 0 && (
              <div className="mb-6">
                <h4 className="text-lg font-semibold text-white mb-3">Contract Parameters</h4>
                <div className="space-y-4">
                  {selectedTemplate.parameters.map((param) => (
                    <div key={param.param_name}>
                      <label className="label">
                        {param.param_name} {param.required && <span className="text-red-400">*</span>}
                      </label>
                      <p className="text-xs text-slate-400 mb-2">{param.description}</p>
                      <input
                        type="text"
                        value={deployParams[param.param_name] || ''}
                        onChange={(e) => setDeployParams({
                          ...deployParams,
                          [param.param_name]: e.target.value
                        })}
                        className="input"
                        placeholder={param.default_value?.toString() || ''}
                        required={param.required}
                      />
                    </div>
                  ))}
                </div>
              </div>
            )}

            {/* Parties */}
            <div className="mb-6">
              <h4 className="text-lg font-semibold text-white mb-3">Contract Parties</h4>
              <div className="space-y-3">
                {deployParties.map((party, idx) => (
                  <div key={idx} className="flex gap-3">
                    <input
                      type="text"
                      value={party.identity_id}
                      onChange={(e) => {
                        const newParties = [...deployParties]
                        newParties[idx].identity_id = e.target.value
                        setDeployParties(newParties)
                      }}
                      className="input flex-1"
                      placeholder="Identity ID"
                    />
                    <input
                      type="text"
                      value={party.role}
                      onChange={(e) => {
                        const newParties = [...deployParties]
                        newParties[idx].role = e.target.value
                        setDeployParties(newParties)
                      }}
                      className="input w-1/3"
                      placeholder="Role"
                    />
                    <button
                      onClick={() => setDeployParties(deployParties.filter((_, i) => i !== idx))}
                      className="btn bg-red-900/30 text-red-300 border-red-700"
                    >
                      Remove
                    </button>
                  </div>
                ))}
                <button
                  onClick={() => setDeployParties([...deployParties, { identity_id: '', role: '', signed: false }])}
                  className="btn btn-secondary w-full"
                >
                  Add Party
                </button>
              </div>
            </div>

            {/* Code Hash */}
            <div className="mb-6 p-4 bg-slate-900/50 rounded-lg border border-slate-700">
              <div className="flex items-center gap-2 mb-2">
                <ShieldCheckIcon className="w-5 h-5 text-primary-400" />
                <span className="text-sm font-medium text-white">Code Integrity</span>
              </div>
              <div className="text-xs text-slate-400 font-mono break-all">
                Hash: {selectedTemplate.code_hash}
              </div>
            </div>

            {/* Actions */}
            <div className="flex gap-3">
              <button
                onClick={() => {
                  setShowDeployModal(false)
                  setSelectedTemplate(null)
                  setDeployParams({})
                  setDeployParties([])
                }}
                className="btn btn-secondary flex-1"
              >
                Cancel
              </button>
              <button
                onClick={handleDeployContract}
                disabled={deploying}
                className="btn btn-primary flex-1"
              >
                {deploying ? 'Deploying...' : 'Deploy Contract'}
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Contract Details Modal */}
      {showContractDetails && selectedContract && (
        <div className="fixed inset-0 bg-slate-900/80 backdrop-blur-sm z-50 flex items-center justify-center p-4">
          <div className="card max-w-4xl w-full max-h-[85vh] overflow-y-auto">
            <div className="flex items-center justify-between mb-6">
              <div>
                <h3 className="text-2xl font-bold text-white">{selectedContract.template.template_name}</h3>
                <p className="text-sm text-slate-400">{formatCategory(selectedContract.template.category)} Contract</p>
              </div>
              <button onClick={() => {
                setShowContractDetails(false)
                setSelectedContract(null)
              }} className="text-slate-400 hover:text-white">
                <XMarkIcon className="w-6 h-6" />
              </button>
            </div>

            {/* Status */}
            <div className="grid grid-cols-3 gap-4 mb-6">
              <div className="card bg-slate-900/50">
                <div className="text-sm text-slate-400 mb-1">Status</div>
                <div className={`font-semibold ${
                  selectedContract.status === 'active' ? 'text-green-400' :
                  selectedContract.status === 'pending_signatures' ? 'text-yellow-400' : 'text-slate-300'
                }`}>
                  {selectedContract.status.replace(/_/g, ' ').toUpperCase()}
                </div>
              </div>
              <div className="card bg-slate-900/50">
                <div className="text-sm text-slate-400 mb-1">Deployed</div>
                <div className="font-semibold text-white">
                  {new Date(selectedContract.created_at).toLocaleDateString()}
                </div>
              </div>
              <div className="card bg-slate-900/50">
                <div className="text-sm text-slate-400 mb-1">Version</div>
                <div className="font-semibold text-white">{selectedContract.template.version}</div>
              </div>
            </div>

            {/* Natural Language Summary */}
            <div className="mb-6">
              <h4 className="text-lg font-semibold text-white mb-3">Summary</h4>
              <div className="p-4 bg-slate-900/50 rounded-lg border border-slate-700">
                <p className="text-sm text-slate-300">{selectedContract.natural_language_summary}</p>
              </div>
            </div>

            {/* Parties */}
            <div className="mb-6">
              <h4 className="text-lg font-semibold text-white mb-3">Parties & Signatures</h4>
              <div className="space-y-3">
                {selectedContract.parties.map((party, idx) => (
                  <div key={idx} className="p-4 bg-slate-900/50 rounded-lg border border-slate-700">
                    <div className="flex items-center justify-between">
                      <div>
                        <div className="font-medium text-white">{party.role}</div>
                        <div className="text-sm text-slate-400 font-mono">{party.identity_id.substring(0, 32)}...</div>
                      </div>
                      <div className="flex items-center gap-2">
                        {party.signed ? (
                          <>
                            <CheckCircleIcon className="w-5 h-5 text-green-400" />
                            <span className="text-green-400 text-sm">Signed</span>
                          </>
                        ) : (
                          <>
                            <ClockIcon className="w-5 h-5 text-yellow-400" />
                            <span className="text-yellow-400 text-sm">Pending</span>
                          </>
                        )}
                      </div>
                    </div>
                    {party.signed && party.signed_at && (
                      <div className="text-xs text-slate-500 mt-2">
                        Signed on: {new Date(party.signed_at).toLocaleString()}
                      </div>
                    )}
                  </div>
                ))}
              </div>
            </div>

            {/* On-Chain Info */}
            {selectedContract.chain_address && (
              <div className="mb-6 p-4 bg-primary-900/20 rounded-lg border border-primary-700">
                <div className="flex items-center gap-2 mb-2">
                  <ShieldCheckIcon className="w-5 h-5 text-primary-400" />
                  <span className="text-sm font-medium text-primary-300">On-Chain Contract</span>
                </div>
                <div className="text-xs text-primary-300 font-mono break-all">
                  Address: {selectedContract.chain_address}
                </div>
                {selectedContract.deployment_tx && (
                  <div className="text-xs text-primary-400 font-mono break-all mt-1">
                    TX: {selectedContract.deployment_tx}
                  </div>
                )}
              </div>
            )}

            {/* Actions */}
            <div className="flex gap-3">
              <button
                onClick={() => {
                  setShowContractDetails(false)
                  setSelectedContract(null)
                }}
                className="btn btn-primary flex-1"
              >
                Close
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
