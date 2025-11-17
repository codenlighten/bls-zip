# Enterprise Multipass Frontend - Quick Setup Guide

## Installation

### 1. Navigate to frontend directory
```bash
cd C:\Users\ripva\Desktop\boundless-bls-platform\enterprise\frontend
```

### 2. Install dependencies
```bash
npm install
```

### 3. Create environment file
```bash
copy .env.example .env.local
```

### 4. Start development server
```bash
npm run dev
```

### 5. Open in browser
```
http://localhost:3001
```

## Default Login (Demo)

When connected to the backend API, use your enterprise credentials.

For development/testing without backend:
- The frontend will show connection errors
- You can still view the UI layout and design

## Available Scripts

- `npm run dev` - Start development server on port 3001
- `npm run build` - Build for production
- `npm start` - Start production server
- `npm run lint` - Run ESLint
- `npm run type-check` - Run TypeScript type checking

## Features Checklist

### ✅ Completed
- [x] Login page with authentication
- [x] Dashboard with statistics
- [x] Identity management page
- [x] Wallet balances and transactions
- [x] Responsive sidebar navigation
- [x] Type-safe API client
- [x] Tailwind CSS styling

### 🚧 To Be Implemented
- [ ] Trading interface
- [ ] Admin panel
- [ ] Reports and analytics
- [ ] WebAuthn integration
- [ ] Real-time notifications
- [ ] NFC card support

## Connecting to Backend

The frontend expects the Enterprise Multipass backend API to be running at:
```
http://localhost:8080
```

Make sure the backend is started before using the frontend:
```bash
cd ../  # Back to enterprise directory
cargo run --release
```

## Troubleshooting

### Port already in use
```bash
# If port 3001 is in use, specify a different port:
npm run dev -- -p 3002
```

### Module not found errors
```bash
# Clear node_modules and reinstall
rm -rf node_modules
npm install
```

### TypeScript errors
```bash
# Run type check to see all errors
npm run type-check
```

## Next Steps

1. **Start the backend API** (see `../README.md`)
2. **Run this frontend** (`npm run dev`)
3. **Login with your credentials**
4. **Explore the dashboard**

For detailed documentation, see [README.md](./README.md)
