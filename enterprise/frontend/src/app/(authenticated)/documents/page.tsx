'use client'

import { useState, useEffect } from 'react'
import { api } from '@/lib/api'
import type { IdentityProfile, Document, DocumentThread, SecureMessage } from '@/types'
import {
  DocumentTextIcon,
  FolderIcon,
  ChatBubbleLeftRightIcon,
  PlusIcon,
  LockClosedIcon,
  ShieldCheckIcon,
  PaperClipIcon,
  UserGroupIcon,
  CheckCircleIcon,
  ArrowUpTrayIcon,
} from '@heroicons/react/24/outline'

export default function DocumentsPage() {
  const [identity, setIdentity] = useState<IdentityProfile | null>(null)
  const [documents, setDocuments] = useState<Document[]>([])
  const [threads, setThreads] = useState<DocumentThread[]>([])
  const [selectedThread, setSelectedThread] = useState<DocumentThread | null>(null)
  const [messages, setMessages] = useState<SecureMessage[]>([])
  const [loading, setLoading] = useState(true)
  const [view, setView] = useState<'documents' | 'messages'>('documents')

  // Modals
  const [showUploadModal, setShowUploadModal] = useState(false)
  const [showNewThreadModal, setShowNewThreadModal] = useState(false)

  // Upload form
  const [documentName, setDocumentName] = useState('')
  const [documentType, setDocumentType] = useState('personal')
  const [documentTags, setDocumentTags] = useState('')
  const [isImmutable, setIsImmutable] = useState(false)
  const [uploading, setUploading] = useState(false)

  // Thread form
  const [threadTopic, setThreadTopic] = useState('')
  const [threadParticipants, setThreadParticipants] = useState('')
  const [creating, setCreating] = useState(false)

  // Message form
  const [messageContent, setMessageContent] = useState('')
  const [sending, setSending] = useState(false)

  useEffect(() => {
    loadData()
  }, [])

  useEffect(() => {
    if (selectedThread) {
      loadMessages(selectedThread.thread_id)
    }
  }, [selectedThread])

  const loadData = async () => {
    try {
      setLoading(true)
      const userStr = localStorage.getItem('user_identity')
      if (!userStr) {
        setLoading(false)
        return
      }

      const user: IdentityProfile = JSON.parse(userStr)
      setIdentity(user)

      // Load documents
      const docsResponse = await api.getDocuments(user.identity_id)
      if (docsResponse.data) {
        setDocuments(docsResponse.data)
      }

      // Load message threads
      const threadsResponse = await api.getThreads(user.identity_id)
      if (threadsResponse.data) {
        setThreads(threadsResponse.data)
      }

      setLoading(false)
    } catch (err) {
      console.error('Error loading documents:', err)
      setLoading(false)
    }
  }

  const loadMessages = async (thread_id: string) => {
    try {
      const response = await api.getMessages(thread_id)
      if (response.data) {
        setMessages(response.data)
      }
    } catch (err) {
      console.error('Error loading messages:', err)
    }
  }

  const handleUpload = async () => {
    if (!identity || !documentName) return

    setUploading(true)
    try {
      // In real implementation, would encrypt file content here
      const response = await api.uploadDocument({
        owner_id: identity.identity_id,
        document_name: documentName,
        document_type: documentType as any,
        encrypted_content: 'encrypted_content_placeholder',
        tags: documentTags ? documentTags.split(',').map(t => t.trim()) : [],
        is_immutable: isImmutable,
      })

      if (!response.error) {
        setShowUploadModal(false)
        setDocumentName('')
        setDocumentType('personal')
        setDocumentTags('')
        setIsImmutable(false)
        await loadData()
      }
    } catch (err) {
      console.error('Error uploading document:', err)
    }
    setUploading(false)
  }

  const handleCreateThread = async () => {
    if (!identity || !threadTopic) return

    setCreating(true)
    try {
      const participants = threadParticipants
        ? threadParticipants.split(',').map(p => p.trim())
        : []
      participants.push(identity.identity_id)

      const response = await api.createThread({
        created_by: identity.identity_id,
        participants,
        topic: threadTopic,
      })

      if (!response.error) {
        setShowNewThreadModal(false)
        setThreadTopic('')
        setThreadParticipants('')
        await loadData()
      }
    } catch (err) {
      console.error('Error creating thread:', err)
    }
    setCreating(false)
  }

  const handleSendMessage = async () => {
    if (!identity || !selectedThread || !messageContent) return

    setSending(true)
    try {
      const response = await api.sendMessage(selectedThread.thread_id, {
        sender_id: identity.identity_id,
        content: messageContent,
      })

      if (!response.error) {
        setMessageContent('')
        await loadMessages(selectedThread.thread_id)
      }
    } catch (err) {
      console.error('Error sending message:', err)
    }
    setSending(false)
  }

  const formatBytes = (bytes: number): string => {
    if (bytes === 0) return '0 Bytes'
    const k = 1024
    const sizes = ['Bytes', 'KB', 'MB', 'GB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return Math.round(bytes / Math.pow(k, i) * 100) / 100 + ' ' + sizes[i]
  }

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="flex flex-col items-center gap-4">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-primary-500"></div>
          <div className="text-slate-400">Loading documents...</div>
        </div>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      {/* Page Header */}
      <div>
        <h1 className="text-3xl font-bold text-white">Documents & Messaging</h1>
        <p className="text-slate-400 mt-1">
          Encrypted document storage and authenticated private messaging
        </p>
      </div>

      {/* View Tabs */}
      <div className="flex gap-2">
        <button
          onClick={() => setView('documents')}
          className={`btn ${view === 'documents' ? 'btn-primary' : 'bg-slate-800 text-slate-300'}`}
        >
          <FolderIcon className="w-5 h-5 mr-2" />
          Documents ({documents.length})
        </button>
        <button
          onClick={() => setView('messages')}
          className={`btn ${view === 'messages' ? 'btn-primary' : 'bg-slate-800 text-slate-300'}`}
        >
          <ChatBubbleLeftRightIcon className="w-5 h-5 mr-2" />
          Messages ({threads.length})
        </button>
      </div>

      {/* Documents View */}
      {view === 'documents' && (
        <div className="card">
          <div className="flex items-center justify-between mb-6">
            <h2 className="text-xl font-semibold text-white">Encrypted Document Locker</h2>
            <button
              onClick={() => setShowUploadModal(true)}
              className="btn btn-primary flex items-center gap-2"
            >
              <ArrowUpTrayIcon className="w-5 h-5" />
              Upload Document
            </button>
          </div>

          {documents.length === 0 ? (
            <div className="text-center py-12">
              <DocumentTextIcon className="w-16 h-16 mx-auto mb-4 text-slate-600" />
              <h3 className="text-lg font-semibold text-white mb-2">No Documents Yet</h3>
              <p className="text-slate-400 mb-6">
                Upload encrypted documents with immutability and integrity guarantees
              </p>
              <button
                onClick={() => setShowUploadModal(true)}
                className="btn btn-secondary mx-auto"
              >
                Upload First Document
              </button>
            </div>
          ) : (
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
              {documents.map((doc) => (
                <div
                  key={doc.document_id}
                  className="p-4 bg-slate-900/50 rounded-lg border border-slate-700 hover:border-primary-500/50 transition-colors"
                >
                  <div className="flex items-start gap-3 mb-3">
                    <div className="w-12 h-12 bg-gradient-to-br from-primary-500 to-secondary-600 rounded-lg flex items-center justify-center shrink-0">
                      <DocumentTextIcon className="w-6 h-6 text-white" />
                    </div>
                    <div className="flex-1 min-w-0">
                      <h3 className="font-semibold text-white truncate">{doc.document_name}</h3>
                      <p className="text-xs text-slate-400">
                        {typeof doc.document_type === 'string' ? doc.document_type.toUpperCase() : doc.document_type.custom}
                      </p>
                    </div>
                  </div>

                  <div className="space-y-2 mb-3">
                    <div className="flex items-center gap-2 text-xs">
                      <LockClosedIcon className="w-4 h-4 text-green-400" />
                      <span className="text-green-400">Encrypted</span>
                    </div>
                    {doc.is_immutable && (
                      <div className="flex items-center gap-2 text-xs">
                        <ShieldCheckIcon className="w-4 h-4 text-blue-400" />
                        <span className="text-blue-400">Immutable</span>
                      </div>
                    )}
                    {doc.chain_anchor_tx && (
                      <div className="flex items-center gap-2 text-xs">
                        <CheckCircleIcon className="w-4 h-4 text-primary-400" />
                        <span className="text-primary-400">On-Chain Anchored</span>
                      </div>
                    )}
                  </div>

                  <div className="text-xs text-slate-500 mb-2">
                    Size: {formatBytes(doc.size_bytes)}
                  </div>

                  {doc.tags.length > 0 && (
                    <div className="flex flex-wrap gap-1 mb-3">
                      {doc.tags.map((tag, idx) => (
                        <span key={idx} className="badge bg-slate-800 text-slate-300 text-xs">
                          {tag}
                        </span>
                      ))}
                    </div>
                  )}

                  <div className="text-xs text-slate-500">
                    {doc.permissions.length} permission(s)
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      )}

      {/* Messages View */}
      {view === 'messages' && (
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          {/* Thread List */}
          <div className="lg:col-span-1">
            <div className="card">
              <div className="flex items-center justify-between mb-4">
                <h2 className="text-lg font-semibold text-white">Threads</h2>
                <button
                  onClick={() => setShowNewThreadModal(true)}
                  className="btn btn-sm btn-primary"
                >
                  <PlusIcon className="w-4 h-4" />
                </button>
              </div>

              {threads.length === 0 ? (
                <div className="text-center py-8 text-slate-400">
                  <ChatBubbleLeftRightIcon className="w-12 h-12 mx-auto mb-3 opacity-50" />
                  <p className="text-sm">No threads yet</p>
                </div>
              ) : (
                <div className="space-y-2">
                  {threads.map((thread) => (
                    <div
                      key={thread.thread_id}
                      onClick={() => setSelectedThread(thread)}
                      className={`p-3 rounded-lg cursor-pointer transition-colors ${
                        selectedThread?.thread_id === thread.thread_id
                          ? 'bg-primary-900/30 border border-primary-700'
                          : 'bg-slate-900/50 border border-slate-700 hover:border-slate-600'
                      }`}
                    >
                      <div className="font-medium text-white text-sm truncate">{thread.topic}</div>
                      <div className="flex items-center gap-2 text-xs text-slate-400 mt-1">
                        <UserGroupIcon className="w-3 h-3" />
                        {thread.participants.length} participants
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>
          </div>

          {/* Messages */}
          <div className="lg:col-span-2">
            <div className="card h-[600px] flex flex-col">
              {selectedThread ? (
                <>
                  <div className="border-b border-slate-700 pb-4 mb-4">
                    <h2 className="text-lg font-semibold text-white">{selectedThread.topic}</h2>
                    <p className="text-sm text-slate-400">
                      {selectedThread.participants.length} participants • {selectedThread.status}
                    </p>
                  </div>

                  <div className="flex-1 overflow-y-auto space-y-4 mb-4">
                    {messages.length === 0 ? (
                      <div className="text-center py-12 text-slate-400">
                        No messages yet. Start the conversation!
                      </div>
                    ) : (
                      messages.map((msg) => {
                        const isMyMessage = msg.sender_id === identity?.identity_id

                        return (
                          <div
                            key={msg.message_id}
                            className={`flex ${isMyMessage ? 'justify-end' : 'justify-start'}`}
                          >
                            <div
                              className={`max-w-[70%] p-3 rounded-lg ${
                                isMyMessage
                                  ? 'bg-primary-600 text-white'
                                  : 'bg-slate-800 text-slate-200'
                              }`}
                            >
                              <div className="text-sm break-words">{msg.encrypted_content}</div>
                              <div className="flex items-center gap-2 mt-2">
                                <div className={`text-xs ${isMyMessage ? 'text-primary-200' : 'text-slate-500'}`}>
                                  {new Date(msg.timestamp).toLocaleTimeString()}
                                </div>
                                {msg.attachments.length > 0 && (
                                  <>
                                    <div className="text-xs">•</div>
                                    <PaperClipIcon className="w-3 h-3" />
                                    <div className="text-xs">{msg.attachments.length}</div>
                                  </>
                                )}
                              </div>
                            </div>
                          </div>
                        )
                      })
                    )}
                  </div>

                  <div className="flex gap-3">
                    <input
                      type="text"
                      value={messageContent}
                      onChange={(e) => setMessageContent(e.target.value)}
                      onKeyPress={(e) => e.key === 'Enter' && handleSendMessage()}
                      className="input flex-1"
                      placeholder="Type your message..."
                    />
                    <button
                      onClick={handleSendMessage}
                      disabled={sending || !messageContent}
                      className="btn btn-primary"
                    >
                      {sending ? 'Sending...' : 'Send'}
                    </button>
                  </div>
                </>
              ) : (
                <div className="flex items-center justify-center h-full text-slate-400">
                  Select a thread to view messages
                </div>
              )}
            </div>
          </div>
        </div>
      )}

      {/* Upload Modal */}
      {showUploadModal && (
        <div className="fixed inset-0 bg-slate-900/80 backdrop-blur-sm z-50 flex items-center justify-center p-4">
          <div className="card max-w-md w-full">
            <h3 className="text-xl font-bold text-white mb-6">Upload Document</h3>

            <div className="space-y-4">
              <div>
                <label className="label">Document Name</label>
                <input
                  type="text"
                  value={documentName}
                  onChange={(e) => setDocumentName(e.target.value)}
                  className="input"
                  placeholder="Contract.pdf"
                />
              </div>

              <div>
                <label className="label">Document Type</label>
                <select
                  value={documentType}
                  onChange={(e) => setDocumentType(e.target.value)}
                  className="input"
                >
                  <option value="contract">Contract</option>
                  <option value="invoice">Invoice</option>
                  <option value="receipt">Receipt</option>
                  <option value="certificate">Certificate</option>
                  <option value="report">Report</option>
                  <option value="compliance">Compliance</option>
                  <option value="personal">Personal</option>
                </select>
              </div>

              <div>
                <label className="label">Tags (comma-separated)</label>
                <input
                  type="text"
                  value={documentTags}
                  onChange={(e) => setDocumentTags(e.target.value)}
                  className="input"
                  placeholder="legal, confidential, 2024"
                />
              </div>

              <div className="flex items-center gap-2">
                <input
                  type="checkbox"
                  checked={isImmutable}
                  onChange={(e) => setIsImmutable(e.target.checked)}
                  className="rounded"
                />
                <label className="text-sm text-slate-300">
                  Make immutable (cannot be modified after upload)
                </label>
              </div>

              <div className="flex gap-3 pt-4">
                <button
                  onClick={() => {
                    setShowUploadModal(false)
                    setDocumentName('')
                    setDocumentType('personal')
                    setDocumentTags('')
                    setIsImmutable(false)
                  }}
                  className="btn btn-secondary flex-1"
                >
                  Cancel
                </button>
                <button
                  onClick={handleUpload}
                  disabled={uploading || !documentName}
                  className="btn btn-primary flex-1"
                >
                  {uploading ? 'Uploading...' : 'Upload'}
                </button>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* New Thread Modal */}
      {showNewThreadModal && (
        <div className="fixed inset-0 bg-slate-900/80 backdrop-blur-sm z-50 flex items-center justify-center p-4">
          <div className="card max-w-md w-full">
            <h3 className="text-xl font-bold text-white mb-6">New Message Thread</h3>

            <div className="space-y-4">
              <div>
                <label className="label">Topic</label>
                <input
                  type="text"
                  value={threadTopic}
                  onChange={(e) => setThreadTopic(e.target.value)}
                  className="input"
                  placeholder="Discussion topic"
                />
              </div>

              <div>
                <label className="label">Participants (comma-separated identity IDs)</label>
                <textarea
                  value={threadParticipants}
                  onChange={(e) => setThreadParticipants(e.target.value)}
                  className="input"
                  rows={3}
                  placeholder="identity_id_1, identity_id_2"
                />
              </div>

              <div className="flex gap-3 pt-4">
                <button
                  onClick={() => {
                    setShowNewThreadModal(false)
                    setThreadTopic('')
                    setThreadParticipants('')
                  }}
                  className="btn btn-secondary flex-1"
                >
                  Cancel
                </button>
                <button
                  onClick={handleCreateThread}
                  disabled={creating || !threadTopic}
                  className="btn btn-primary flex-1"
                >
                  {creating ? 'Creating...' : 'Create Thread'}
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
