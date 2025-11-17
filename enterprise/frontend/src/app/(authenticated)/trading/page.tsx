'use client'

import { useState, useEffect } from 'react'
import { api } from '@/lib/api'
import type { IdentityProfile, MarketListing, MarketOrder, AssetType } from '@/types'
import {
  BuildingLibraryIcon,
  PlusIcon,
  ShoppingCartIcon,
  CheckCircleIcon,
  ClockIcon,
  XMarkIcon,
  ShieldCheckIcon,
  GlobeAltIcon,
} from '@heroicons/react/24/outline'

export default function InternalMarketsPage() {
  const [identity, setIdentity] = useState<IdentityProfile | null>(null)
  const [listings, setListings] = useState<MarketListing[]>([])
  const [myOrders, setMyOrders] = useState<MarketOrder[]>([])
  const [loading, setLoading] = useState(true)
  const [selectedSector, setSelectedSector] = useState<string>('all')

  // Modals
  const [showCreateListingModal, setShowCreateListingModal] = useState(false)
  const [showOrderModal, setShowOrderModal] = useState(false)
  const [selectedListing, setSelectedListing] = useState<MarketListing | null>(null)

  // Listing form
  const [assetId, setAssetId] = useState('')
  const [assetType, setAssetType] = useState<AssetType>('equity_token')
  const [quantity, setQuantity] = useState('')
  const [pricePerUnit, setPricePerUnit] = useState('')
  const [creating, setCreating] = useState(false)

  // Order form
  const [orderQuantity, setOrderQuantity] = useState('')
  const [ordering, setOrdering] = useState(false)

  useEffect(() => {
    loadData()
  }, [selectedSector])

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

      // Load market listings
      const listingsResponse = await api.getMarketListings(
        selectedSector !== 'all' ? selectedSector : undefined
      )
      if (listingsResponse.data) {
        setListings(listingsResponse.data)
      }

      // Load my orders
      const ordersResponse = await api.getMyOrders(user.identity_id)
      if (ordersResponse.data) {
        setMyOrders(ordersResponse.data)
      }

      setLoading(false)
    } catch (err) {
      console.error('Error loading markets:', err)
      setLoading(false)
    }
  }

  const handleCreateListing = async () => {
    if (!identity || !assetId || !quantity || !pricePerUnit) return

    setCreating(true)
    try {
      const response = await api.createListing({
        seller_id: identity.identity_id,
        asset_id: assetId,
        asset_type: assetType,
        quantity: parseInt(quantity),
        price_per_unit: parseFloat(pricePerUnit),
        currency: 'BLS',
        compliance_requirements: [],
        jurisdictions: [],
      })

      if (!response.error) {
        setShowCreateListingModal(false)
        setAssetId('')
        setQuantity('')
        setPricePerUnit('')
        await loadData()
      }
    } catch (err) {
      console.error('Error creating listing:', err)
    }
    setCreating(false)
  }

  const handlePlaceOrder = async () => {
    if (!identity || !selectedListing || !orderQuantity) return

    setOrdering(true)
    try {
      const response = await api.createMarketOrder({
        buyer_id: identity.identity_id,
        listing_id: selectedListing.listing_id,
        quantity: parseInt(orderQuantity),
      })

      if (!response.error) {
        setShowOrderModal(false)
        setSelectedListing(null)
        setOrderQuantity('')
        await loadData()
      }
    } catch (err) {
      console.error('Error placing order:', err)
    }
    setOrdering(false)
  }

  const formatAssetType = (assetType: any): string => {
    if (typeof assetType === 'string') {
      return assetType.replace(/_/g, ' ').toUpperCase()
    }
    if (typeof assetType === 'object' && assetType.custom) {
      return assetType.custom
    }
    return 'UNKNOWN'
  }

  const formatSector = (sector: any): string => {
    if (!sector) return 'General'
    if (typeof sector === 'string') {
      return sector.replace(/_/g, ' ').split(' ').map(w => w.charAt(0).toUpperCase() + w.slice(1)).join(' ')
    }
    if (typeof sector === 'object' && sector.custom) {
      return sector.custom
    }
    return 'General'
  }

  const getStatusBadge = (status: string) => {
    const badges = {
      active: { class: 'badge-success' },
      filled: { class: 'badge bg-blue-900/50 text-blue-300 border border-blue-700' },
      cancelled: { class: 'badge bg-slate-700 text-slate-400' },
      expired: { class: 'badge bg-slate-700 text-slate-400' },
      pending: { class: 'badge-warning' },
      approved: { class: 'badge-success' },
      settled: { class: 'badge-success' },
      rejected: { class: 'badge-error' },
    }
    return badges[status as keyof typeof badges] || badges.active
  }

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="flex flex-col items-center gap-4">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-primary-500"></div>
          <div className="text-slate-400">Loading markets...</div>
        </div>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      {/* Page Header */}
      <div>
        <h1 className="text-3xl font-bold text-white">Internal Markets</h1>
        <p className="text-slate-400 mt-1">
          Regulated asset exchange for enterprise - NOT a cryptocurrency exchange
        </p>
      </div>

      {/* Stats */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <div className="card">
          <div className="text-sm text-slate-400 mb-1">Active Listings</div>
          <div className="text-2xl font-bold text-white">
            {listings.filter(l => l.status === 'active').length}
          </div>
        </div>
        <div className="card">
          <div className="text-sm text-slate-400 mb-1">My Orders</div>
          <div className="text-2xl font-bold text-primary-400">{myOrders.length}</div>
        </div>
        <div className="card">
          <div className="text-sm text-slate-400 mb-1">Pending Orders</div>
          <div className="text-2xl font-bold text-yellow-400">
            {myOrders.filter(o => o.status === 'pending').length}
          </div>
        </div>
        <div className="card">
          <div className="text-sm text-slate-400 mb-1">Settled</div>
          <div className="text-2xl font-bold text-green-400">
            {myOrders.filter(o => o.status === 'settled').length}
          </div>
        </div>
      </div>

      {/* Sector Filter */}
      <div className="flex flex-wrap gap-2">
        {['all', 'carbon_credits', 'real_estate', 'equity', 'utilities', 'healthcare', 'ticketing'].map((sector) => (
          <button
            key={sector}
            onClick={() => setSelectedSector(sector)}
            className={`btn btn-sm ${
              selectedSector === sector
                ? 'btn-primary'
                : 'bg-slate-800 text-slate-300 hover:bg-slate-700'
            }`}
          >
            {sector === 'all' ? 'All Sectors' : formatSector(sector)}
          </button>
        ))}
      </div>

      {/* Market Listings */}
      <div className="card">
        <div className="flex items-center justify-between mb-6">
          <div>
            <h2 className="text-xl font-semibold text-white">Market Listings</h2>
            <p className="text-sm text-slate-400 mt-1">
              Regulated asset trading: Carbon Credits, Real Estate, Equities, Utilities, Healthcare, Ticketing
            </p>
          </div>
          <button
            onClick={() => setShowCreateListingModal(true)}
            className="btn btn-primary flex items-center gap-2"
          >
            <PlusIcon className="w-5 h-5" />
            Create Listing
          </button>
        </div>

        {listings.length === 0 ? (
          <div className="text-center py-12">
            <BuildingLibraryIcon className="w-16 h-16 mx-auto mb-4 text-slate-600" />
            <h3 className="text-lg font-semibold text-white mb-2">No Listings Available</h3>
            <p className="text-slate-400 mb-6">
              Be the first to list an asset in this sector
            </p>
            <button
              onClick={() => setShowCreateListingModal(true)}
              className="btn btn-secondary mx-auto"
            >
              Create First Listing
            </button>
          </div>
        ) : (
          <div className="space-y-4">
            {listings.map((listing) => {
              const statusBadge = getStatusBadge(listing.status)

              return (
                <div
                  key={listing.listing_id}
                  className="p-4 bg-slate-900/50 rounded-lg border border-slate-700 hover:border-primary-500/50 transition-colors"
                >
                  <div className="flex items-start justify-between">
                    <div className="flex-1">
                      <div className="flex items-center gap-3 mb-2">
                        <h3 className="font-semibold text-white">{formatAssetType(listing.asset_type)}</h3>
                        <span className={`badge ${statusBadge.class}`}>
                          {listing.status.toUpperCase()}
                        </span>
                      </div>

                      <div className="grid grid-cols-2 md:grid-cols-4 gap-4 mb-3">
                        <div>
                          <div className="text-xs text-slate-400">Quantity</div>
                          <div className="font-medium text-white">{listing.quantity.toLocaleString()}</div>
                        </div>
                        <div>
                          <div className="text-xs text-slate-400">Price per Unit</div>
                          <div className="font-medium text-white">{listing.price_per_unit} {listing.currency}</div>
                        </div>
                        <div>
                          <div className="text-xs text-slate-400">Total Value</div>
                          <div className="font-medium text-primary-400">
                            {(listing.quantity * listing.price_per_unit).toLocaleString()} {listing.currency}
                          </div>
                        </div>
                        <div>
                          <div className="text-xs text-slate-400">Min. Quantity</div>
                          <div className="font-medium text-white">
                            {listing.min_quantity || 1}
                          </div>
                        </div>
                      </div>

                      {listing.compliance_requirements.length > 0 && (
                        <div className="flex items-center gap-2 mb-2">
                          <ShieldCheckIcon className="w-4 h-4 text-primary-400" />
                          <span className="text-xs text-slate-400">
                            Compliance: {listing.compliance_requirements.join(', ')}
                          </span>
                        </div>
                      )}

                      {listing.jurisdictions.length > 0 && (
                        <div className="flex items-center gap-2 mb-2">
                          <GlobeAltIcon className="w-4 h-4 text-secondary-400" />
                          <span className="text-xs text-slate-400">
                            Jurisdictions: {listing.jurisdictions.join(', ')}
                          </span>
                        </div>
                      )}

                      <div className="text-xs text-slate-500">
                        Listed: {new Date(listing.created_at).toLocaleDateString()}
                        {listing.expires_at && (
                          <span className="ml-4">
                            Expires: {new Date(listing.expires_at).toLocaleDateString()}
                          </span>
                        )}
                      </div>
                    </div>

                    {listing.status === 'active' && listing.seller_id !== identity?.identity_id && (
                      <button
                        onClick={() => {
                          setSelectedListing(listing)
                          setShowOrderModal(true)
                        }}
                        className="btn btn-primary flex items-center gap-2 ml-4"
                      >
                        <ShoppingCartIcon className="w-5 h-5" />
                        Buy
                      </button>
                    )}
                  </div>
                </div>
              )
            })}
          </div>
        )}
      </div>

      {/* My Orders */}
      {myOrders.length > 0 && (
        <div className="card">
          <h2 className="text-xl font-semibold text-white mb-6">My Orders</h2>
          <div className="space-y-3">
            {myOrders.map((order) => {
              const statusBadge = getStatusBadge(order.status)

              return (
                <div
                  key={order.order_id}
                  className="p-4 bg-slate-900/50 rounded-lg border border-slate-700"
                >
                  <div className="flex items-center justify-between">
                    <div>
                      <div className="flex items-center gap-3 mb-2">
                        <span className="font-medium text-white">Order #{order.order_id.substring(0, 8)}</span>
                        <span className={`badge ${statusBadge.class}`}>
                          {order.status.toUpperCase()}
                        </span>
                      </div>
                      <div className="text-sm text-slate-400">
                        Quantity: {order.quantity} • Total: {order.total_price.toLocaleString()} BLS
                      </div>
                      <div className="text-xs text-slate-500 mt-1">
                        Placed: {new Date(order.created_at).toLocaleString()}
                      </div>
                      {order.settlement_tx && (
                        <div className="text-xs text-primary-400 font-mono mt-1">
                          TX: {order.settlement_tx.substring(0, 32)}...
                        </div>
                      )}
                    </div>
                  </div>
                </div>
              )
            })}
          </div>
        </div>
      )}

      {/* Create Listing Modal */}
      {showCreateListingModal && (
        <div className="fixed inset-0 bg-slate-900/80 backdrop-blur-sm z-50 flex items-center justify-center p-4">
          <div className="card max-w-md w-full">
            <h3 className="text-xl font-bold text-white mb-6">Create Market Listing</h3>

            <div className="space-y-4">
              <div>
                <label className="label">Asset ID</label>
                <input
                  type="text"
                  value={assetId}
                  onChange={(e) => setAssetId(e.target.value)}
                  className="input"
                  placeholder="asset_123abc"
                />
              </div>

              <div>
                <label className="label">Asset Type</label>
                <select
                  value={assetType}
                  onChange={(e) => setAssetType(e.target.value as AssetType)}
                  className="input"
                >
                  <option value="equity_token">Equity Token</option>
                  <option value="carbon_credit">Carbon Credit</option>
                  <option value="utility_token">Utility Token</option>
                  <option value="nft">NFT</option>
                  <option value="subscription_pass">Subscription Pass</option>
                </select>
              </div>

              <div>
                <label className="label">Quantity</label>
                <input
                  type="number"
                  value={quantity}
                  onChange={(e) => setQuantity(e.target.value)}
                  className="input"
                  placeholder="1000"
                />
              </div>

              <div>
                <label className="label">Price per Unit (BLS)</label>
                <input
                  type="number"
                  step="0.01"
                  value={pricePerUnit}
                  onChange={(e) => setPricePerUnit(e.target.value)}
                  className="input"
                  placeholder="10.50"
                />
              </div>

              <div className="p-4 bg-primary-900/20 rounded-lg border border-primary-700">
                <div className="flex items-center gap-2 mb-1">
                  <ShieldCheckIcon className="w-4 h-4 text-primary-400" />
                  <span className="text-sm font-medium text-primary-300">Regulated Market</span>
                </div>
                <p className="text-xs text-primary-300">
                  All listings are subject to Boundless compliance rules and jurisdiction requirements
                </p>
              </div>

              <div className="flex gap-3 pt-4">
                <button
                  onClick={() => {
                    setShowCreateListingModal(false)
                    setAssetId('')
                    setQuantity('')
                    setPricePerUnit('')
                  }}
                  className="btn btn-secondary flex-1"
                >
                  Cancel
                </button>
                <button
                  onClick={handleCreateListing}
                  disabled={creating || !assetId || !quantity || !pricePerUnit}
                  className="btn btn-primary flex-1"
                >
                  {creating ? 'Creating...' : 'Create Listing'}
                </button>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Place Order Modal */}
      {showOrderModal && selectedListing && (
        <div className="fixed inset-0 bg-slate-900/80 backdrop-blur-sm z-50 flex items-center justify-center p-4">
          <div className="card max-w-md w-full">
            <h3 className="text-xl font-bold text-white mb-6">Place Order</h3>

            <div className="space-y-4">
              <div className="p-4 bg-slate-900/50 rounded-lg border border-slate-700">
                <div className="font-semibold text-white mb-2">
                  {formatAssetType(selectedListing.asset_type)}
                </div>
                <div className="text-sm text-slate-400">
                  Price: {selectedListing.price_per_unit} BLS per unit
                </div>
                <div className="text-sm text-slate-400">
                  Available: {selectedListing.quantity.toLocaleString()}
                </div>
              </div>

              <div>
                <label className="label">Quantity</label>
                <input
                  type="number"
                  value={orderQuantity}
                  onChange={(e) => setOrderQuantity(e.target.value)}
                  className="input"
                  placeholder={`Min: ${selectedListing.min_quantity || 1}`}
                  min={selectedListing.min_quantity || 1}
                  max={selectedListing.quantity}
                />
              </div>

              {orderQuantity && (
                <div className="p-4 bg-primary-900/20 rounded-lg border border-primary-700">
                  <div className="text-sm font-medium text-primary-300 mb-1">Order Total</div>
                  <div className="text-2xl font-bold text-white">
                    {(parseInt(orderQuantity) * selectedListing.price_per_unit).toLocaleString()} BLS
                  </div>
                </div>
              )}

              <div className="flex gap-3 pt-4">
                <button
                  onClick={() => {
                    setShowOrderModal(false)
                    setSelectedListing(null)
                    setOrderQuantity('')
                  }}
                  className="btn btn-secondary flex-1"
                >
                  Cancel
                </button>
                <button
                  onClick={handlePlaceOrder}
                  disabled={ordering || !orderQuantity}
                  className="btn btn-primary flex-1"
                >
                  {ordering ? 'Placing Order...' : 'Place Order'}
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
