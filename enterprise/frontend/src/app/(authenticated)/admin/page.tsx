'use client'

import { Cog6ToothIcon } from '@heroicons/react/24/outline'

export default function AdminPage() {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold text-white">Admin</h1>
        <p className="text-slate-400 mt-1">
          Manage applications and permissions
        </p>
      </div>

      <div className="card text-center py-20">
        <Cog6ToothIcon className="w-20 h-20 text-primary-400 mx-auto mb-4" />
        <h2 className="text-2xl font-bold text-white mb-2">
          Admin Panel Coming Soon
        </h2>
        <p className="text-slate-400 max-w-md mx-auto">
          The admin interface will allow you to manage application modules,
          user permissions, and system configuration.
        </p>
      </div>
    </div>
  )
}
