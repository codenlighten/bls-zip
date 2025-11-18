'use client'

import { useEffect, useState } from 'react'
import { api } from '@/lib/api'
import type {
  DashboardSummary,
  BlockchainMetricsSnapshot,
  UserActivityMetrics,
  SystemPerformanceMetrics,
  SustainabilityMetrics,
} from '@/types'
import {
  ChartBarIcon,
  CpuChipIcon,
  UsersIcon,
  ServerIcon,
  ArrowPathIcon,
  BeakerIcon,
  CheckCircleIcon,
  ClockIcon,
  ExclamationTriangleIcon,
  BoltIcon,
  LeafIcon,
} from '@heroicons/react/24/outline'

export default function AnalyticsPage() {
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [dashboard, setDashboard] = useState<DashboardSummary | null>(null)
  const [autoRefresh, setAutoRefresh] = useState(false)

  const fetchMetrics = async () => {
    try {
      setError(null)
      const response = await api.getDashboardMetrics()

      if (response.error) {
        setError(response.error)
        return
      }

      if (response.data?.dashboard) {
        setDashboard(response.data.dashboard)
      }
    } catch (err) {
      setError('Failed to fetch metrics')
      console.error('Error fetching metrics:', err)
    } finally {
      setLoading(false)
    }
  }

  const collectMetrics = async () => {
    try {
      const response = await api.collectAllMetrics()
      if (response.error) {
        setError(response.error)
      } else {
        // Refresh dashboard after collection
        await fetchMetrics()
      }
    } catch (err) {
      setError('Failed to collect metrics')
    }
  }

  useEffect(() => {
    fetchMetrics()
  }, [])

  useEffect(() => {
    if (autoRefresh) {
      const interval = setInterval(fetchMetrics, 30000) // Refresh every 30 seconds
      return () => clearInterval(interval)
    }
  }, [autoRefresh])

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="flex flex-col items-center gap-4">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-primary-500"></div>
          <div className="text-slate-400">Loading analytics...</div>
        </div>
      </div>
    )
  }

  if (error) {
    return (
      <div className="space-y-6">
        <div>
          <h1 className="text-3xl font-bold text-white">Platform Analytics</h1>
          <p className="text-slate-400 mt-1">
            Real-time metrics and insights for Boundless E2 Multipass
          </p>
        </div>
        <div className="card">
          <div className="flex items-center gap-3 text-red-400">
            <ExclamationTriangleIcon className="w-6 h-6" />
            <div>
              <strong>Error:</strong> {error}
            </div>
          </div>
          <button
            onClick={fetchMetrics}
            className="mt-4 btn btn-primary flex items-center gap-2"
          >
            <ArrowPathIcon className="w-5 h-5" />
            Retry
          </button>
        </div>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      {/* Page Header */}
      <div>
        <h1 className="text-3xl font-bold text-white">Platform Analytics</h1>
        <p className="text-slate-400 mt-1">
          Real-time metrics and insights for Boundless E2 Multipass
        </p>
      </div>

      {/* Control Bar */}
      <div className="flex flex-wrap items-center justify-between gap-4">
        <div className="flex items-center gap-4">
          <label className="flex items-center gap-2 text-sm text-slate-300">
            <input
              type="checkbox"
              checked={autoRefresh}
              onChange={(e) => setAutoRefresh(e.target.checked)}
              className="form-checkbox h-4 w-4 text-primary-500 rounded border-slate-600 bg-slate-800"
            />
            Auto-refresh (30s)
          </label>
          {dashboard && (
            <div className="text-sm text-slate-500 flex items-center gap-2">
              <ClockIcon className="w-4 h-4" />
              Last updated: {new Date(dashboard.timestamp).toLocaleString()}
            </div>
          )}
        </div>
        <div className="flex gap-3">
          <button
            onClick={collectMetrics}
            className="btn btn-primary flex items-center gap-2"
          >
            <BeakerIcon className="w-5 h-5" />
            Collect Metrics
          </button>
          <button
            onClick={fetchMetrics}
            className="btn btn-secondary flex items-center gap-2"
          >
            <ArrowPathIcon className="w-5 h-5" />
            Refresh
          </button>
        </div>
      </div>

      {/* Blockchain Metrics */}
      {dashboard?.blockchain && (
        <BlockchainMetricsCard metrics={dashboard.blockchain} />
      )}

      {/* User Activity Metrics */}
      {dashboard?.users && (
        <UserActivityCard metrics={dashboard.users} />
      )}

      {/* System Performance Metrics */}
      {dashboard?.system && (
        <SystemPerformanceCard metrics={dashboard.system} />
      )}

      {/* Sustainability Metrics */}
      {dashboard?.sustainability && (
        <SustainabilityCard metrics={dashboard.sustainability} />
      )}
    </div>
  )
}

// Blockchain Metrics Card
function BlockchainMetricsCard({ metrics }: { metrics: BlockchainMetricsSnapshot }) {
  return (
    <div className="card">
      <div className="flex items-center gap-3 mb-6">
        <div className="p-2 bg-primary-900/30 rounded-lg">
          <CpuChipIcon className="w-6 h-6 text-primary-400" />
        </div>
        <h2 className="text-xl font-semibold text-white">Blockchain Metrics</h2>
      </div>
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        <MetricDisplay
          label="Block Height"
          value={metrics.block_height.toLocaleString()}
        />
        <MetricDisplay
          label="Total Transactions"
          value={metrics.total_transactions.toLocaleString()}
        />
        <MetricDisplay
          label="Active Contracts"
          value={metrics.active_contracts.toLocaleString()}
        />
        <MetricDisplay
          label="Peer Count"
          value={metrics.peer_count.toLocaleString()}
        />
      </div>

      {/* TPS Metrics */}
      <div className="mt-6 pt-6 border-t border-slate-700">
        <h3 className="text-lg font-medium mb-4 text-white flex items-center gap-2">
          <BoltIcon className="w-5 h-5 text-primary-400" />
          Throughput (TPS)
        </h3>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          <MetricDisplay
            label="1 Minute"
            value={metrics.tps_1min?.toFixed(2) || 'N/A'}
            unit="TPS"
            size="sm"
          />
          <MetricDisplay
            label="1 Hour"
            value={metrics.tps_1hour?.toFixed(2) || 'N/A'}
            unit="TPS"
            size="sm"
          />
          <MetricDisplay
            label="24 Hours"
            value={metrics.tps_24hour?.toFixed(2) || 'N/A'}
            unit="TPS"
            size="sm"
          />
        </div>
      </div>
    </div>
  )
}

// User Activity Card
function UserActivityCard({ metrics }: { metrics: UserActivityMetrics }) {
  return (
    <div className="card">
      <div className="flex items-center gap-3 mb-6">
        <div className="p-2 bg-primary-900/30 rounded-lg">
          <UsersIcon className="w-6 h-6 text-primary-400" />
        </div>
        <h2 className="text-xl font-semibold text-white">User Activity</h2>
      </div>
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        <MetricDisplay
          label="Total Users"
          value={metrics.total_users.toLocaleString()}
        />
        <MetricDisplay
          label="Active (1h)"
          value={metrics.active_users_1hour.toLocaleString()}
        />
        <MetricDisplay
          label="Active (24h)"
          value={metrics.active_users_24hour.toLocaleString()}
        />
        <MetricDisplay
          label="Active (7d)"
          value={metrics.active_users_7days.toLocaleString()}
        />
      </div>

      <div className="mt-6 pt-6 border-t border-slate-700">
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          <MetricDisplay
            label="New Users (24h)"
            value={metrics.new_users_24hour.toLocaleString()}
            size="sm"
          />
          <MetricDisplay
            label="Total Sessions"
            value={metrics.total_sessions.toLocaleString()}
            size="sm"
          />
          <MetricDisplay
            label="Avg Session Duration"
            value={metrics.avg_session_duration_minutes?.toFixed(1) || 'N/A'}
            unit="min"
            size="sm"
          />
        </div>
      </div>
    </div>
  )
}

// System Performance Card
function SystemPerformanceCard({ metrics }: { metrics: SystemPerformanceMetrics }) {
  return (
    <div className="card">
      <div className="flex items-center gap-3 mb-6">
        <div className="p-2 bg-primary-900/30 rounded-lg">
          <ServerIcon className="w-6 h-6 text-primary-400" />
        </div>
        <h2 className="text-xl font-semibold text-white">System Performance</h2>
      </div>
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        <MetricDisplay
          label="API Requests/min"
          value={metrics.api_requests_1min.toFixed(0)}
        />
        <MetricDisplay
          label="Avg Response Time"
          value={metrics.avg_response_time_ms.toFixed(0)}
          unit="ms"
        />
        <MetricDisplay
          label="Error Rate"
          value={metrics.error_rate_percent.toFixed(2)}
          unit="%"
          alert={metrics.error_rate_percent > 5}
        />
        <MetricDisplay
          label="DB Connections"
          value={metrics.database_connections.toLocaleString()}
        />
      </div>

      {/* Additional System Metrics */}
      <div className="mt-6 pt-6 border-t border-slate-700">
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          <MetricDisplay
            label="Memory Usage"
            value={metrics.memory_usage_mb?.toFixed(0) || 'N/A'}
            unit="MB"
            size="sm"
          />
          <MetricDisplay
            label="CPU Usage"
            value={metrics.cpu_usage_percent?.toFixed(1) || 'N/A'}
            unit="%"
            size="sm"
          />
          <MetricDisplay
            label="Disk Usage"
            value={metrics.disk_usage_percent?.toFixed(1) || 'N/A'}
            unit="%"
            size="sm"
          />
        </div>
      </div>
    </div>
  )
}

// Sustainability Card
function SustainabilityCard({ metrics }: { metrics: SustainabilityMetrics }) {
  const getGradeColor = (grade?: string) => {
    if (!grade) return 'badge'
    if (grade.startsWith('A')) return 'badge badge-success'
    if (grade.startsWith('B')) return 'badge bg-blue-900/50 text-blue-300 border border-blue-700'
    if (grade.startsWith('C')) return 'badge badge-warning'
    if (grade.startsWith('D')) return 'badge bg-orange-900/50 text-orange-300 border border-orange-700'
    return 'badge badge-error'
  }

  return (
    <div className="card">
      <div className="flex items-center gap-3 mb-6">
        <div className="p-2 bg-green-900/30 rounded-lg">
          <LeafIcon className="w-6 h-6 text-green-400" />
        </div>
        <h2 className="text-xl font-semibold text-white">Sustainability Metrics</h2>
      </div>

      {/* Overall Score & Grade */}
      <div className="mb-6 p-4 bg-gradient-to-r from-green-900/20 to-blue-900/20 rounded-lg border border-green-900/30">
        <div className="flex justify-between items-center">
          <div>
            <div className="text-sm text-slate-400">Overall Sustainability Score</div>
            <div className="text-3xl font-bold text-white">
              {metrics.overall_sustainability_score?.toFixed(1) || 'N/A'}/100
            </div>
          </div>
          <div className={`px-6 py-3 rounded-lg text-2xl font-bold ${getGradeColor(metrics.sustainability_grade)}`}>
            {metrics.sustainability_grade || 'N/A'}
          </div>
        </div>
      </div>

      {/* Energy Metrics */}
      <div className="mb-6">
        <h3 className="text-lg font-medium mb-4 text-white">Energy Consumption</h3>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          <MetricDisplay
            label="Power Consumption"
            value={metrics.estimated_power_consumption_kwh?.toFixed(3) || 'N/A'}
            unit="kWh"
            size="sm"
          />
          <MetricDisplay
            label="Energy per TX"
            value={metrics.energy_per_transaction_wh?.toFixed(4) || 'N/A'}
            unit="Wh"
            size="sm"
          />
        </div>
      </div>

      {/* Carbon Footprint */}
      <div className="mb-6">
        <h3 className="text-lg font-medium mb-4 text-white">Carbon Footprint</h3>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          <MetricDisplay
            label="Total Carbon"
            value={metrics.estimated_carbon_kg?.toFixed(3) || 'N/A'}
            unit="kg CO₂"
            size="sm"
          />
          <MetricDisplay
            label="Carbon per TX"
            value={metrics.carbon_per_transaction_g?.toFixed(4) || 'N/A'}
            unit="g CO₂"
            size="sm"
          />
        </div>
      </div>

      {/* Efficiency Metrics */}
      <div>
        <h3 className="text-lg font-medium mb-4 text-white">Efficiency Scores</h3>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          <EfficiencyBar
            label="Storage"
            value={metrics.storage_efficiency_percent || 0}
          />
          <EfficiencyBar
            label="Network"
            value={metrics.network_efficiency_percent || 0}
          />
          <EfficiencyBar
            label="Computation"
            value={metrics.computation_efficiency_percent || 0}
          />
        </div>
      </div>
    </div>
  )
}

// Reusable Metric Display Component
function MetricDisplay({
  label,
  value,
  unit,
  alert,
  size = 'default',
}: {
  label: string
  value: string | number
  unit?: string
  alert?: boolean
  size?: 'default' | 'sm'
}) {
  const sizeClasses = size === 'sm'
    ? 'text-xl'
    : 'text-2xl'

  return (
    <div className={`p-4 rounded-lg border ${alert ? 'bg-red-900/20 border-red-700' : 'bg-slate-900/50 border-slate-700'}`}>
      <div className="text-sm font-medium text-slate-400 mb-2">{label}</div>
      <div className={`${sizeClasses} font-bold ${alert ? 'text-red-400' : 'text-white'}`}>
        {value}
        {unit && <span className="text-sm text-slate-500 ml-1">{unit}</span>}
      </div>
    </div>
  )
}

// Efficiency Bar Component
function EfficiencyBar({ label, value }: { label: string; value: number }) {
  const getColor = (val: number) => {
    if (val >= 90) return 'bg-green-500'
    if (val >= 70) return 'bg-blue-500'
    if (val >= 50) return 'bg-yellow-500'
    return 'bg-orange-500'
  }

  return (
    <div className="p-4 rounded-lg bg-slate-900/50 border border-slate-700">
      <div className="flex justify-between items-center mb-2">
        <div className="text-sm font-medium text-slate-400">{label}</div>
        <div className="text-lg font-bold text-white">{value.toFixed(1)}%</div>
      </div>
      <div className="w-full bg-slate-800 rounded-full h-3">
        <div
          className={`${getColor(value)} h-3 rounded-full transition-all duration-300`}
          style={{ width: `${Math.min(100, Math.max(0, value))}%` }}
        />
      </div>
    </div>
  )
}
