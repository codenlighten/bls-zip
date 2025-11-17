'use client'

import { useState, useEffect } from 'react'
import { api } from '@/lib/api'
import type { IdentityProfile, KnowledgeNode, KnowledgeSearchResult } from '@/types'
import {
  BookOpenIcon,
  PlusIcon,
  XCircleIcon,
  MagnifyingGlassIcon,
  CodeBracketIcon,
  DocumentTextIcon,
  MapIcon,
  LightBulbIcon,
  WrenchScrewdriverIcon,
  LinkIcon,
  GlobeAltIcon,
  UserGroupIcon,
  LockClosedIcon,
  TrashIcon,
  PencilIcon,
} from '@heroicons/react/24/outline'

export default function KnowledgePage() {
  const [identity, setIdentity] = useState<IdentityProfile | null>(null)
  const [nodes, setNodes] = useState<KnowledgeNode[]>([])
  const [selectedNode, setSelectedNode] = useState<KnowledgeNode | null>(null)
  const [searchResults, setSearchResults] = useState<KnowledgeNode[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState('')
  const [searchQuery, setSearchQuery] = useState('')
  const [searching, setSearching] = useState(false)

  const [showCreateNode, setShowCreateNode] = useState(false)
  const [nodeTitle, setNodeTitle] = useState('')
  const [nodeContent, setNodeContent] = useState('')
  const [nodeType, setNodeType] = useState('document')
  const [nodeTags, setNodeTags] = useState<string[]>([])
  const [tagInput, setTagInput] = useState('')
  const [accessLevel, setAccessLevel] = useState<'private' | 'team' | 'organization' | 'public'>('private')

  useEffect(() => {
    loadKnowledgeData()
  }, [])

  const loadKnowledgeData = async () => {
    try {
      const userStr = localStorage.getItem('user_identity')
      if (!userStr) {
        setError('No user session found')
        setLoading(false)
        return
      }

      const user: IdentityProfile = JSON.parse(userStr)
      setIdentity(user)

      const response = await api.getKnowledgeNodes(user.identity_id)
      if (response.error) {
        console.error('Error loading knowledge:', response.error)
      } else if (response.data) {
        setNodes(response.data)
      }

      setLoading(false)
    } catch (err) {
      console.error('Error loading knowledge:', err)
      setError('Failed to load knowledge vault')
      setLoading(false)
    }
  }

  const handleSearch = async () => {
    if (!searchQuery.trim()) {
      setSearchResults([])
      return
    }

    setSearching(true)
    try {
      const response = await api.searchKnowledge({
        query: searchQuery,
        limit: 20,
      })

      if (response.data?.nodes) {
        setSearchResults(response.data.nodes)
      }
    } catch (err) {
      console.error('Error searching:', err)
    } finally {
      setSearching(false)
    }
  }

  const handleCreateNode = async () => {
    if (!identity || !nodeTitle.trim() || !nodeContent.trim()) return

    try {
      const response = await api.createKnowledgeNode({
        identity_id: identity.identity_id,
        node_type: nodeType,
        title: nodeTitle.trim(),
        content: nodeContent.trim(),
        tags: nodeTags,
        access_level: accessLevel,
      })

      if (response.error) {
        alert(`Failed to create node: ${response.error}`)
      } else if (response.data) {
        setNodes([response.data, ...nodes])
        setShowCreateNode(false)
        setNodeTitle('')
        setNodeContent('')
        setNodeTags([])
      }
    } catch (err) {
      console.error('Error creating node:', err)
      alert('Failed to create knowledge node')
    }
  }

  const handleDeleteNode = async (nodeId: string) => {
    if (!confirm('Are you sure you want to delete this knowledge node?')) {
      return
    }

    try {
      const response = await api.deleteKnowledgeNode(nodeId)
      if (response.error) {
        alert(`Failed to delete node: ${response.error}`)
      } else {
        setNodes(nodes.filter(n => n.node_id !== nodeId))
        if (selectedNode?.node_id === nodeId) {
          setSelectedNode(null)
        }
      }
    } catch (err) {
      console.error('Error deleting node:', err)
      alert('Failed to delete node')
    }
  }

  const addTag = () => {
    if (tagInput.trim() && !nodeTags.includes(tagInput.trim())) {
      setNodeTags([...nodeTags, tagInput.trim()])
      setTagInput('')
    }
  }

  const removeTag = (tag: string) => {
    setNodeTags(nodeTags.filter(t => t !== tag))
  }

  const getNodeTypeIcon = (type: any) => {
    const typeStr = typeof type === 'string' ? type : 'custom'
    switch (typeStr) {
      case 'code_snippet': return CodeBracketIcon
      case 'api_reference': return MapIcon
      case 'architecture_diagram': return MapIcon
      case 'decision_record': return LightBulbIcon
      case 'troubleshooting_guide': return WrenchScrewdriverIcon
      default: return DocumentTextIcon
    }
  }

  const getAccessIcon = (level: string) => {
    switch (level) {
      case 'public': return GlobeAltIcon
      case 'organization': return UserGroupIcon
      case 'team': return UserGroupIcon
      default: return LockClosedIcon
    }
  }

  const formatTimestamp = (timestamp: string): string => {
    const date = new Date(timestamp)
    return date.toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
      year: 'numeric'
    })
  }

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="flex flex-col items-center gap-4">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-primary-500"></div>
          <div className="text-slate-400">Loading knowledge vault...</div>
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

  const displayNodes = searchQuery.trim() ? searchResults : nodes

  return (
    <div className="space-y-6">
      {/* Page Header */}
      <div>
        <h1 className="text-3xl font-bold text-white">Enterprise Knowledge Vault</h1>
        <p className="text-slate-400 mt-1">
          Identity-aware knowledge graph with semantic search and relation tracking
        </p>
      </div>

      {/* Search Bar */}
      <div className="card">
        <div className="flex gap-3">
          <div className="relative flex-1">
            <MagnifyingGlassIcon className="absolute left-3 top-1/2 transform -translate-y-1/2 w-5 h-5 text-slate-400" />
            <input
              type="text"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              onKeyPress={(e) => e.key === 'Enter' && handleSearch()}
              placeholder="Search knowledge base..."
              className="input w-full pl-10"
            />
          </div>
          <button
            onClick={handleSearch}
            disabled={searching}
            className="btn btn-secondary"
          >
            {searching ? 'Searching...' : 'Search'}
          </button>
          <button
            onClick={() => setShowCreateNode(true)}
            className="btn btn-primary flex items-center gap-2"
          >
            <PlusIcon className="w-5 h-5" />
            Add Node
          </button>
        </div>
      </div>

      {/* Knowledge Nodes */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* Nodes List */}
        <div className="lg:col-span-2">
          <div className="card">
            <h2 className="text-lg font-semibold text-white mb-4">
              {searchQuery.trim() ? `Search Results (${displayNodes.length})` : `All Nodes (${nodes.length})`}
            </h2>

            {displayNodes.length === 0 ? (
              <div className="text-center py-12 text-slate-400">
                <BookOpenIcon className="w-16 h-16 mx-auto mb-4 opacity-50" />
                <p>{searchQuery.trim() ? 'No results found' : 'No knowledge nodes yet'}</p>
                <p className="text-sm mt-2">
                  {searchQuery.trim() ? 'Try a different search query' : 'Create your first knowledge node'}
                </p>
              </div>
            ) : (
              <div className="space-y-3">
                {displayNodes.map((node) => {
                  const Icon = getNodeTypeIcon(node.node_type)
                  const AccessIcon = getAccessIcon(node.access_level)
                  return (
                    <div
                      key={node.node_id}
                      className={`p-4 rounded-lg border cursor-pointer transition-colors ${
                        selectedNode?.node_id === node.node_id
                          ? 'bg-primary-900/30 border-primary-600'
                          : 'bg-slate-900/50 border-slate-700 hover:border-slate-600'
                      }`}
                      onClick={() => setSelectedNode(node)}
                    >
                      <div className="flex items-start gap-3">
                        <div className="p-2 bg-primary-900/30 rounded-lg flex-shrink-0">
                          <Icon className="w-5 h-5 text-primary-400" />
                        </div>
                        <div className="flex-1 min-w-0">
                          <div className="flex items-center gap-2 mb-1">
                            <h3 className="font-medium text-white truncate">{node.title}</h3>
                            <AccessIcon className="w-4 h-4 text-slate-400 flex-shrink-0" />
                          </div>
                          <p className="text-sm text-slate-400 line-clamp-2 mb-2">{node.content}</p>
                          <div className="flex flex-wrap gap-2 mb-2">
                            {node.tags.slice(0, 3).map((tag) => (
                              <span
                                key={tag}
                                className="px-2 py-1 bg-slate-800 text-slate-300 text-xs rounded"
                              >
                                {tag}
                              </span>
                            ))}
                            {node.tags.length > 3 && (
                              <span className="px-2 py-1 bg-slate-800 text-slate-300 text-xs rounded">
                                +{node.tags.length - 3} more
                              </span>
                            )}
                          </div>
                          <div className="flex items-center justify-between">
                            <div className="text-xs text-slate-500">
                              {formatTimestamp(node.created_at)}
                            </div>
                            {node.relations.length > 0 && (
                              <div className="flex items-center gap-1 text-xs text-slate-400">
                                <LinkIcon className="w-3 h-3" />
                                {node.relations.length} relations
                              </div>
                            )}
                          </div>
                        </div>
                      </div>
                    </div>
                  )
                })}
              </div>
            )}
          </div>
        </div>

        {/* Node Details */}
        <div className="lg:col-span-1">
          {selectedNode ? (
            <div className="card sticky top-6">
              <div className="flex items-start justify-between mb-4">
                <h3 className="text-lg font-semibold text-white">Node Details</h3>
                <button
                  onClick={() => handleDeleteNode(selectedNode.node_id)}
                  className="text-red-400 hover:text-red-300"
                >
                  <TrashIcon className="w-5 h-5" />
                </button>
              </div>

              <div className="space-y-4">
                <div>
                  <label className="label">Title</label>
                  <div className="text-white">{selectedNode.title}</div>
                </div>

                <div>
                  <label className="label">Type</label>
                  <div className="text-white">
                    {typeof selectedNode.node_type === 'string'
                      ? selectedNode.node_type.replace(/_/g, ' ').toUpperCase()
                      : 'CUSTOM'}
                  </div>
                </div>

                <div>
                  <label className="label">Access Level</label>
                  <div className="flex items-center gap-2 text-white">
                    {(() => {
                      const Icon = getAccessIcon(selectedNode.access_level)
                      return <Icon className="w-4 h-4" />
                    })()}
                    {selectedNode.access_level.toUpperCase()}
                  </div>
                </div>

                <div>
                  <label className="label">Content</label>
                  <div className="text-sm text-slate-300 bg-slate-900/50 p-3 rounded border border-slate-700 max-h-48 overflow-y-auto">
                    {selectedNode.content}
                  </div>
                </div>

                <div>
                  <label className="label">Tags</label>
                  <div className="flex flex-wrap gap-2">
                    {selectedNode.tags.map((tag) => (
                      <span
                        key={tag}
                        className="px-2 py-1 bg-primary-900/30 text-primary-300 text-xs rounded border border-primary-700"
                      >
                        {tag}
                      </span>
                    ))}
                  </div>
                </div>

                <div>
                  <label className="label">Relations</label>
                  {selectedNode.relations.length === 0 ? (
                    <div className="text-sm text-slate-400">No relations</div>
                  ) : (
                    <div className="space-y-2">
                      {selectedNode.relations.map((rel, idx) => (
                        <div
                          key={idx}
                          className="p-2 bg-slate-900/50 rounded text-sm text-slate-300 border border-slate-700"
                        >
                          <div className="flex items-center gap-2">
                            <LinkIcon className="w-4 h-4" />
                            <span>{rel.relation_type}</span>
                            <span className="text-xs text-slate-500">
                              (strength: {rel.strength})
                            </span>
                          </div>
                        </div>
                      ))}
                    </div>
                  )}
                </div>

                <div className="pt-4 border-t border-slate-700">
                  <div className="text-xs text-slate-500">
                    Created: {formatTimestamp(selectedNode.created_at)}
                  </div>
                  <div className="text-xs text-slate-500 mt-1">
                    Updated: {formatTimestamp(selectedNode.updated_at)}
                  </div>
                </div>
              </div>
            </div>
          ) : (
            <div className="card sticky top-6">
              <div className="text-center py-12 text-slate-400">
                <BookOpenIcon className="w-12 h-12 mx-auto mb-3 opacity-50" />
                <p className="text-sm">Select a node to view details</p>
              </div>
            </div>
          )}
        </div>
      </div>

      {/* Create Node Modal */}
      {showCreateNode && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 overflow-y-auto">
          <div className="card max-w-2xl w-full mx-4 my-8">
            <div className="flex items-center justify-between mb-6">
              <h2 className="text-xl font-semibold text-white">Create Knowledge Node</h2>
              <button
                onClick={() => setShowCreateNode(false)}
                className="text-slate-400 hover:text-white"
              >
                <XCircleIcon className="w-6 h-6" />
              </button>
            </div>

            <div className="space-y-4">
              <div>
                <label className="label">Title</label>
                <input
                  type="text"
                  value={nodeTitle}
                  onChange={(e) => setNodeTitle(e.target.value)}
                  placeholder="e.g., Authentication Architecture"
                  className="input w-full"
                />
              </div>

              <div>
                <label className="label">Type</label>
                <select
                  value={nodeType}
                  onChange={(e) => setNodeType(e.target.value)}
                  className="input w-full"
                >
                  <option value="document">Document</option>
                  <option value="code_snippet">Code Snippet</option>
                  <option value="api_reference">API Reference</option>
                  <option value="architecture_diagram">Architecture Diagram</option>
                  <option value="decision_record">Decision Record</option>
                  <option value="troubleshooting_guide">Troubleshooting Guide</option>
                </select>
              </div>

              <div>
                <label className="label">Content</label>
                <textarea
                  value={nodeContent}
                  onChange={(e) => setNodeContent(e.target.value)}
                  placeholder="Enter the content of this knowledge node..."
                  className="input w-full h-32"
                />
              </div>

              <div>
                <label className="label">Access Level</label>
                <select
                  value={accessLevel}
                  onChange={(e) => setAccessLevel(e.target.value as any)}
                  className="input w-full"
                >
                  <option value="private">Private (Only Me)</option>
                  <option value="team">Team</option>
                  <option value="organization">Organization</option>
                  <option value="public">Public</option>
                </select>
              </div>

              <div>
                <label className="label">Tags</label>
                <div className="flex gap-2 mb-2">
                  <input
                    type="text"
                    value={tagInput}
                    onChange={(e) => setTagInput(e.target.value)}
                    onKeyPress={(e) => e.key === 'Enter' && addTag()}
                    placeholder="Add tag and press Enter"
                    className="input flex-1"
                  />
                  <button onClick={addTag} className="btn btn-secondary">
                    Add
                  </button>
                </div>
                {nodeTags.length > 0 && (
                  <div className="flex flex-wrap gap-2">
                    {nodeTags.map((tag) => (
                      <span
                        key={tag}
                        className="inline-flex items-center gap-2 px-3 py-1 bg-primary-900/30 text-primary-300 text-sm rounded border border-primary-700"
                      >
                        {tag}
                        <button
                          onClick={() => removeTag(tag)}
                          className="text-primary-400 hover:text-primary-300"
                        >
                          ×
                        </button>
                      </span>
                    ))}
                  </div>
                )}
              </div>

              <div className="bg-primary-900/20 border border-primary-700 rounded-lg p-4">
                <div className="text-sm">
                  <div className="text-primary-300 font-medium mb-1">Semantic Search</div>
                  <div className="text-slate-400 text-xs">
                    Knowledge nodes are indexed for semantic search with AI-powered relation suggestions.
                    All nodes are cryptographically bound to your identity.
                  </div>
                </div>
              </div>

              <div className="flex gap-3">
                <button
                  onClick={() => setShowCreateNode(false)}
                  className="btn btn-secondary flex-1"
                >
                  Cancel
                </button>
                <button
                  onClick={handleCreateNode}
                  disabled={!nodeTitle.trim() || !nodeContent.trim()}
                  className="btn btn-primary flex-1"
                >
                  Create Node
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
