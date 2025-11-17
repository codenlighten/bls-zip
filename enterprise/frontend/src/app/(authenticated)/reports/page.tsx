'use client'

import { DocumentTextIcon } from '@heroicons/react/24/outline'

export default function ReportsPage() {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold text-white">Reports</h1>
        <p className="text-slate-400 mt-1">
          Analytics and compliance reporting
        </p>
      </div>

      <div className="card text-center py-20">
        <DocumentTextIcon className="w-20 h-20 text-primary-400 mx-auto mb-4" />
        <h2 className="text-2xl font-bold text-white mb-2">
          Reports Coming Soon
        </h2>
        <p className="text-slate-400 max-w-md mx-auto">
          The reports interface will provide analytics, compliance reports,
          and export functionality in multiple formats.
        </p>
      </div>
    </div>
  )
}
