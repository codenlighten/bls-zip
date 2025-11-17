'use client'

import { useState, useEffect, FormEvent } from 'react'
import { useRouter } from 'next/navigation'
import { api } from '@/lib/api'
import { ShieldCheckIcon, LockClosedIcon, EnvelopeIcon, ExclamationTriangleIcon } from '@heroicons/react/24/outline'

export default function LoginPage() {
  const router = useRouter()

  // Check if auto-fill is enabled in development
  const autoFillEnabled = process.env.NEXT_PUBLIC_AUTO_FILL_ADMIN === 'true'
  const defaultEmail = process.env.NEXT_PUBLIC_DEV_ADMIN_EMAIL || ''
  const defaultPassword = process.env.NEXT_PUBLIC_DEV_ADMIN_PASSWORD || ''

  const [email, setEmail] = useState(autoFillEnabled ? defaultEmail : '')
  const [password, setPassword] = useState(autoFillEnabled ? defaultPassword : '')
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState('')

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault()
    setError('')
    setLoading(true)

    try {
      const response = await api.login({ email, password })

      if (response.error) {
        setError(response.error)
        setLoading(false)
        return
      }

      if (response.data) {
        // Token is already set by api.login()
        // Now fetch the full identity profile
        const identityResponse = await api.getIdentity(response.data.session.identity_id)

        if (identityResponse.data) {
          localStorage.setItem('user_identity', JSON.stringify(identityResponse.data))
          localStorage.setItem('session_id', response.data.session.session_id)
          router.push('/dashboard')
        } else {
          setError('Failed to load user profile')
          setLoading(false)
        }
      }
    } catch (err) {
      setError('An unexpected error occurred')
      setLoading(false)
    }
  }

  return (
    <div className="min-h-screen flex items-center justify-center p-4">
      <div className="w-full max-w-md">
        {/* Logo and Header */}
        <div className="text-center mb-8">
          <div className="inline-flex items-center justify-center w-20 h-20 bg-navy-900 border-4 border-primary-500 rounded-xl mb-4 shadow-lg shadow-primary-500/50 relative">
            <div className="text-5xl font-black text-primary-500" style={{fontFamily: 'Arial Black, sans-serif'}}>B</div>
          </div>
          <h1 className="text-3xl font-black text-primary-500 mb-2" style={{letterSpacing: '0.05em'}}>
            BOUNDLESS
          </h1>
          <p className="text-secondary-400 font-medium text-sm tracking-wide">
            E² MULTIPASS
          </p>
          <p className="text-slate-400 text-xs mt-2">
            Enterprise Operating System for Blockchain
          </p>
        </div>

        {/* Development Mode Warning */}
        {autoFillEnabled && (
          <div className="mb-6 bg-yellow-900/20 border-2 border-yellow-600 rounded-lg p-4">
            <div className="flex items-start gap-3">
              <ExclamationTriangleIcon className="w-6 h-6 text-yellow-500 flex-shrink-0 mt-0.5" />
              <div>
                <h3 className="text-yellow-500 font-semibold text-sm mb-1">Development Mode Active</h3>
                <p className="text-yellow-200/80 text-xs leading-relaxed">
                  Admin credentials are pre-filled for development. Email: <span className="font-mono text-yellow-300">{defaultEmail}</span>
                  <br />
                  <span className="text-yellow-400 font-medium">⚠️ This feature is disabled in production builds.</span> Change password immediately after first login.
                </p>
              </div>
            </div>
          </div>
        )}

        {/* Login Card */}
        <div className="card">
          <h2 className="text-xl font-semibold text-white mb-6">Sign In</h2>

          <form onSubmit={handleSubmit} className="space-y-4">
            {error && (
              <div className="bg-red-900/20 border border-red-700 rounded-lg p-3 text-red-300 text-sm">
                {error}
              </div>
            )}

            <div>
              <label htmlFor="email" className="label">
                <EnvelopeIcon className="w-4 h-4 inline mr-2" />
                Email Address
              </label>
              <input
                id="email"
                type="email"
                value={email}
                onChange={(e) => setEmail(e.target.value)}
                className="input"
                placeholder="you@company.com"
                required
                autoComplete="email"
              />
            </div>

            <div>
              <label htmlFor="password" className="label">
                <LockClosedIcon className="w-4 h-4 inline mr-2" />
                Password
              </label>
              <input
                id="password"
                type="password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                className="input"
                placeholder="••••••••"
                required
                autoComplete="current-password"
              />
            </div>

            <div className="flex items-center justify-between text-sm">
              <label className="flex items-center text-slate-300">
                <input type="checkbox" className="mr-2 rounded" />
                Remember me
              </label>
              <a href="#" className="text-primary-400 hover:text-primary-300">
                Forgot password?
              </a>
            </div>

            <button
              type="submit"
              disabled={loading}
              className="w-full btn btn-primary disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {loading ? 'Signing in...' : 'Sign In'}
            </button>
          </form>

          <div className="mt-6 pt-6 border-t border-slate-700">
            <p className="text-center text-sm text-slate-400">
              Don't have an account?{' '}
              <a href="#" className="text-primary-400 hover:text-primary-300 font-medium">
                Contact your administrator
              </a>
            </p>
          </div>
        </div>

        {/* Features */}
        <div className="mt-8 grid grid-cols-3 gap-4 text-center">
          <div>
            <div className="text-2xl font-bold bg-gradient-to-r from-primary-400 to-secondary-400 bg-clip-text text-transparent">CIVA</div>
            <div className="text-xs text-slate-400">Identity Attestation</div>
          </div>
          <div>
            <div className="text-2xl font-bold bg-gradient-to-r from-primary-400 to-secondary-400 bg-clip-text text-transparent">PQC</div>
            <div className="text-xs text-slate-400">Quantum-Safe</div>
          </div>
          <div>
            <div className="text-2xl font-bold bg-gradient-to-r from-primary-400 to-secondary-400 bg-clip-text text-transparent">App-Aware</div>
            <div className="text-xs text-slate-400">Ecosystem</div>
          </div>
        </div>

        {/* Footer */}
        <div className="mt-8 text-center text-xs text-slate-500">
          <p>Enterprise Blockchain • <span className="text-primary-400">IRSC/CRSC Tokens</span> • Multi-Asset Support</p>
          <p className="mt-1">© 2024 Boundless E² Multipass - All rights reserved</p>
        </div>
      </div>
    </div>
  )
}
