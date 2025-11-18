'use client'

import { useState, useEffect } from 'react'
import { useRouter, usePathname } from 'next/navigation'
import Link from 'next/link'
import { api } from '@/lib/api'
import {
  HomeIcon,
  UserCircleIcon,
  WalletIcon,
  ChartBarIcon,
  Cog6ToothIcon,
  BellIcon,
  ArrowRightOnRectangleIcon,
  Bars3Icon,
  XMarkIcon,
  ShieldCheckIcon,
  CommandLineIcon,
  CpuChipIcon,
  BookOpenIcon,
  UserGroupIcon,
  DocumentTextIcon,
} from '@heroicons/react/24/outline'

export default function AuthenticatedLayout({
  children,
}: {
  children: React.ReactNode
}) {
  const router = useRouter()
  const pathname = usePathname()
  const [sidebarOpen, setSidebarOpen] = useState(false)
  const [user, setUser] = useState<any>(null)
  const [notifications, setNotifications] = useState(0)

  useEffect(() => {
    const userStr = localStorage.getItem('user_identity')
    if (userStr) {
      setUser(JSON.parse(userStr))
    } else {
      router.push('/')
    }

    // Load notifications count
    loadNotifications()
  }, [router])

  const loadNotifications = async () => {
    if (!user) return

    const response = await api.getUnreadCount(user.identity_id)
    if (response.data) {
      setNotifications(response.data.count)
    }
  }

  const handleLogout = async () => {
    const sessionId = localStorage.getItem('session_id')
    await api.logout(sessionId || undefined)
    localStorage.removeItem('user_identity')
    localStorage.removeItem('session_id')
    router.push('/')
  }

  const navigation = [
    { name: 'Dashboard', href: '/dashboard', icon: HomeIcon },
    { name: 'Analytics', href: '/analytics', icon: ChartBarIcon },
    { name: 'Identity (CIVA)', href: '/identity', icon: UserCircleIcon },
    { name: 'Assets & Apps', href: '/wallet', icon: WalletIcon },
    { name: 'Contracts', href: '/contracts', icon: DocumentTextIcon },
    { name: 'Documents', href: '/documents', icon: Cog6ToothIcon },
    { name: 'Development', href: '/development', icon: CommandLineIcon },
    { name: 'AI Agents', href: '/ai-agents', icon: CpuChipIcon },
    { name: 'Knowledge', href: '/knowledge', icon: BookOpenIcon },
    { name: 'Collaboration', href: '/collaboration', icon: UserGroupIcon },
    { name: 'Admin', href: '/admin', icon: ShieldCheckIcon },
  ]

  return (
    <div className="min-h-screen">
      {/* Mobile sidebar */}
      <div
        className={`fixed inset-0 bg-slate-900/80 backdrop-blur-sm z-40 lg:hidden transition-opacity ${
          sidebarOpen ? 'opacity-100' : 'opacity-0 pointer-events-none'
        }`}
        onClick={() => setSidebarOpen(false)}
      />

      {/* Sidebar */}
      <div
        className={`fixed inset-y-0 left-0 w-64 bg-slate-900 border-r border-slate-800 z-50 transform transition-transform lg:translate-x-0 ${
          sidebarOpen ? 'translate-x-0' : '-translate-x-full'
        }`}
      >
        <div className="flex flex-col h-full">
          {/* Logo */}
          <div className="flex items-center gap-3 p-6 border-b border-primary-900/50">
            <div className="w-10 h-10 bg-navy-900 border-2 border-primary-500 rounded-lg flex items-center justify-center shadow-lg shadow-primary-500/30">
              <div className="text-2xl font-black text-primary-500">B</div>
            </div>
            <div>
              <div className="font-black text-primary-500 text-sm tracking-wide">BOUNDLESS</div>
              <div className="text-xs text-secondary-400">E² Multipass</div>
            </div>
          </div>

          {/* Navigation */}
          <nav className="flex-1 p-4 space-y-1">
            {navigation.map((item) => {
              const isActive = pathname === item.href
              return (
                <Link
                  key={item.name}
                  href={item.href}
                  className={`flex items-center gap-3 px-4 py-3 rounded-lg transition-colors ${
                    isActive
                      ? 'bg-gradient-to-r from-primary-500 to-secondary-500 text-navy-900 font-bold'
                      : 'text-slate-300 hover:bg-slate-800 hover:text-white'
                  }`}
                  onClick={() => setSidebarOpen(false)}
                >
                  <item.icon className="w-5 h-5" />
                  <span className="font-medium">{item.name}</span>
                </Link>
              )
            })}
          </nav>

          {/* User section */}
          <div className="p-4 border-t border-slate-800">
            <div className="flex items-center gap-3 mb-3">
              <div className="w-10 h-10 bg-slate-700 rounded-full flex items-center justify-center">
                <UserCircleIcon className="w-6 h-6 text-slate-300" />
              </div>
              <div className="flex-1 min-w-0">
                <div className="text-sm font-medium text-white truncate">
                  {user?.legal_name}
                </div>
                <div className="text-xs text-slate-400 truncate">
                  {user?.email}
                </div>
              </div>
            </div>
            <button
              onClick={handleLogout}
              className="w-full flex items-center gap-2 px-4 py-2 text-sm text-slate-300 hover:bg-slate-800 rounded-lg transition-colors"
            >
              <ArrowRightOnRectangleIcon className="w-5 h-5" />
              Sign Out
            </button>
          </div>
        </div>
      </div>

      {/* Main content */}
      <div className="lg:pl-64">
        {/* Top bar */}
        <header className="sticky top-0 z-30 bg-slate-900/80 backdrop-blur-sm border-b border-slate-800">
          <div className="flex items-center justify-between px-4 py-4">
            <button
              onClick={() => setSidebarOpen(true)}
              className="lg:hidden p-2 text-slate-400 hover:text-white"
            >
              <Bars3Icon className="w-6 h-6" />
            </button>

            <div className="flex-1" />

            <div className="flex items-center gap-4">
              <button className="relative p-2 text-slate-400 hover:text-white">
                <BellIcon className="w-6 h-6" />
                {notifications > 0 && (
                  <span className="absolute top-0 right-0 w-5 h-5 bg-red-500 rounded-full text-xs text-white flex items-center justify-center">
                    {notifications}
                  </span>
                )}
              </button>
            </div>
          </div>
        </header>

        {/* Page content */}
        <main className="p-6">
          {children}
        </main>
      </div>
    </div>
  )
}
