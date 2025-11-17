# Boundless Enterprise Multipass - Frontend

> Modern, responsive web interface for the Enterprise Multipass identity and asset management system

## Overview

The Enterprise Multipass Frontend is a Next.js 14 application providing a comprehensive dashboard for identity verification, multi-asset wallet management, trading, and enterprise administration.

## Features

### ✅ Implemented
- **Authentication** - Secure login with JWT tokens
- **Dashboard** - Overview of assets, KYC status, and recent activity
- **Identity Management** - View identity profile, KYC status, and attestations
- **Wallet Management** - Multi-asset balances, transaction history, transfers
- **Responsive Design** - Mobile-friendly interface with Tailwind CSS
- **Type-Safe API Client** - Full TypeScript coverage with API wrapper

### 🚧 Planned
- **Trading Interface** - Order book, market orders, trade history
- **Admin Panel** - Application module management, user permissions
- **Reports & Analytics** - Compliance reports, charts, export functionality
- **WebAuthn Support** - Biometric and hardware key authentication
- **Real-time Updates** - WebSocket notifications and live data

## Tech Stack

- **Framework**: Next.js 14 (App Router)
- **Language**: TypeScript 5.3
- **Styling**: Tailwind CSS 3.4
- **Icons**: Heroicons 2.1
- **State Management**: React Query (TanStack Query)
- **Charts**: Recharts
- **Forms**: React Hook Form + Zod validation

## Project Structure

```
frontend/
├── src/
│   ├── app/                      # Next.js App Router
│   │   ├── (authenticated)/      # Protected routes
│   │   │   ├── dashboard/        # Main dashboard
│   │   │   ├── identity/         # Identity & KYC management
│   │   │   ├── wallet/           # Wallet & transactions
│   │   │   ├── trading/          # Trading interface (placeholder)
│   │   │   ├── admin/            # Admin panel (placeholder)
│   │   │   ├── reports/          # Reports (placeholder)
│   │   │   └── layout.tsx        # Authenticated layout with sidebar
│   │   ├── layout.tsx            # Root layout
│   │   ├── page.tsx              # Login page
│   │   └── globals.css           # Global styles
│   ├── components/               # Reusable React components
│   ├── lib/                      # Utilities and helpers
│   │   └── api.ts                # API client wrapper
│   └── types/                    # TypeScript type definitions
│       └── index.ts              # All API types
├── public/                       # Static assets
├── package.json                  # Dependencies
├── tsconfig.json                 # TypeScript config
├── tailwind.config.ts            # Tailwind config
├── next.config.js                # Next.js config
└── .env.example                  # Environment variables template
```

## Getting Started

### Prerequisites

- Node.js 18+
- npm or yarn
- Boundless Enterprise Multipass backend running (http://localhost:8080)

### Installation

1. **Install dependencies:**
   ```bash
   cd enterprise/frontend
   npm install
   ```

2. **Configure environment:**
   ```bash
   cp .env.example .env.local
   # Edit .env.local with your settings
   ```

3. **Run development server:**
   ```bash
   npm run dev
   ```

4. **Open browser:**
   ```
   http://localhost:3001
   ```

### Environment Variables

Create a `.env.local` file:

```env
# API Configuration
NEXT_PUBLIC_API_URL=http://localhost:8080

# Application Configuration
NEXT_PUBLIC_APP_NAME=Boundless Enterprise Multipass
NEXT_PUBLIC_APP_VERSION=0.1.0

# Feature Flags
NEXT_PUBLIC_ENABLE_WEBAUTHN=true
NEXT_PUBLIC_ENABLE_NFC=false
NEXT_PUBLIC_ENABLE_TRADING=true
```

## Development

### Running in Development Mode

```bash
npm run dev
```

The app will be available at `http://localhost:3001`

### Building for Production

```bash
npm run build
npm start
```

### Type Checking

```bash
npm run type-check
```

### Linting

```bash
npm run lint
```

## Key Pages

### Login (`/`)

- Email/password authentication
- JWT token management
- Session persistence
- Remember me functionality

### Dashboard (`/dashboard`)

- Portfolio overview with total assets value
- KYC verification status
- Recent transactions
- Quick action cards
- Statistics grid (Total Assets, Active Wallets, Pending TX, KYC Status)

### Identity (`/identity`)

- Personal information display
- KYC status with visual indicators
- Attestations management
- Boundless blockchain address
- AML risk score
- Security settings (2FA, Hardware keys, Sessions)

### Wallet (`/wallet`)

- Multi-asset balance display
- Available vs. Locked amounts
- Transaction history (deposits, withdrawals, transfers)
- Send/Receive functionality
- Real-time balance updates

### Trading (`/trading`)

*Coming soon* - Order book, market orders, trade execution

### Admin (`/admin`)

*Coming soon* - Application registry, user management

### Reports (`/reports`)

*Coming soon* - Analytics, compliance reports, exports

## API Integration

### API Client (`src/lib/api.ts`)

The API client provides type-safe wrappers for all backend endpoints:

```typescript
import { api } from '@/lib/api'

// Authentication
const response = await api.login({ email, password })

// Identity
const identity = await api.getIdentity(id)
const attestations = await api.getAttestations(identityId)

// Wallet
const balances = await api.getWalletBalances(walletId)
const transactions = await api.getWalletTransactions(walletId)

// Trading
const orders = await api.getOrders(walletId)
const orderbook = await api.getOrderBook(assetId)
```

All API responses follow the `ApiResponse<T>` pattern:

```typescript
interface ApiResponse<T> {
  data?: T
  error?: string
  message?: string
}
```

## Styling Guidelines

### Using Tailwind Utilities

The project uses custom Tailwind utilities defined in `globals.css`:

```tsx
// Buttons
<button className="btn btn-primary">Primary Action</button>
<button className="btn btn-secondary">Secondary Action</button>
<button className="btn btn-success">Success</button>
<button className="btn btn-danger">Danger</button>

// Cards
<div className="card">
  <h2>Card Title</h2>
  <p>Card content</p>
</div>

// Inputs
<input className="input" placeholder="Enter text" />

// Labels
<label className="label">Field Label</label>

// Badges
<span className="badge badge-success">Verified</span>
<span className="badge badge-warning">Pending</span>
<span className="badge badge-error">Rejected</span>
<span className="badge badge-info">Info</span>
```

### Color Scheme

```css
Primary: #0ea5e9 (primary-500)
Success: #10b981 (green-500)
Warning: #f59e0b (yellow-500)
Error: #ef4444 (red-500)
Background: Slate gradient (900-950)
Text: White/Slate-300/Slate-400
```

## Security Considerations

1. **Token Storage**: JWT tokens stored in localStorage (consider httpOnly cookies for production)
2. **HTTPS Only**: Always use HTTPS in production
3. **CORS**: Configure appropriate CORS policies on backend
4. **CSP**: Content Security Policy headers recommended
5. **Input Validation**: Client-side validation + server-side validation required

## Performance Optimization

- **Code Splitting**: Automatic route-based code splitting
- **Image Optimization**: Next.js Image component for optimized images
- **Lazy Loading**: Dynamic imports for heavy components
- **Caching**: API response caching with React Query

## Deployment

### Vercel (Recommended)

```bash
npm install -g vercel
vercel
```

### Docker

```dockerfile
FROM node:18-alpine AS builder
WORKDIR /app
COPY package*.json ./
RUN npm ci
COPY . .
RUN npm run build

FROM node:18-alpine
WORKDIR /app
COPY --from=builder /app/.next ./.next
COPY --from=builder /app/node_modules ./node_modules
COPY --from=builder /app/package.json ./package.json
COPY --from=builder /app/public ./public

EXPOSE 3001
CMD ["npm", "start"]
```

Build and run:
```bash
docker build -t enterprise-frontend .
docker run -p 3001:3001 enterprise-frontend
```

### Environment-Specific Configuration

**Production:**
- Set `NEXT_PUBLIC_API_URL` to production API endpoint
- Enable `NEXT_PUBLIC_ENABLE_WEBAUTHN` for enhanced security
- Use CDN for static assets

**Staging:**
- Point to staging API
- Enable all feature flags for testing

**Development:**
- Use local API (localhost:8080)
- Hot reload enabled
- Development-only features

## Troubleshooting

### API Connection Issues

```bash
# Check if backend is running
curl http://localhost:8080/api/auth/session

# Check CORS configuration
# Ensure backend allows origin: http://localhost:3001
```

### Build Errors

```bash
# Clear .next cache
rm -rf .next

# Reinstall dependencies
rm -rf node_modules
npm install

# Type check
npm run type-check
```

### Styling Issues

```bash
# Rebuild Tailwind
npx tailwindcss -i ./src/app/globals.css -o ./dist/output.css --watch
```

## Roadmap

### Phase 1: Core Features (Current)
- ✅ Authentication & Login
- ✅ Dashboard overview
- ✅ Identity management
- ✅ Wallet balances & transactions
- ✅ Responsive design

### Phase 2: Trading & Markets
- [ ] Order book display
- [ ] Place market/limit orders
- [ ] Trade history
- [ ] Price charts (TradingView/Recharts)

### Phase 3: Advanced Features
- [ ] WebAuthn biometric login
- [ ] NFC card integration
- [ ] Real-time WebSocket updates
- [ ] Advanced charts & analytics

### Phase 4: Enterprise Features
- [ ] Multi-user admin panel
- [ ] Role-based access control
- [ ] Compliance reporting
- [ ] CSV/PDF export
- [ ] Audit logs

## Contributing

1. Follow the existing code structure
2. Use TypeScript for all new files
3. Add proper type definitions
4. Follow the Tailwind utility-first approach
5. Test on mobile devices
6. Update this README for new features

## Support

For issues and questions:
- File an issue on GitHub
- Contact: enterprise@boundless-bls.com
- Documentation: https://docs.boundless-bls.com

## License

MIT OR Apache-2.0

---

**Built with Next.js 14 for the Boundless BLS Blockchain**
