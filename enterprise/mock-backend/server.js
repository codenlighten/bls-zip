const express = require('express');
const cors = require('cors');
const bodyParser = require('body-parser');

const app = express();
const PORT = 8080;

// Middleware
app.use(cors());
app.use(bodyParser.json());

// Request logging middleware
app.use((req, res, next) => {
  console.log(`[${new Date().toISOString()}] ${req.method} ${req.path}`);
  next();
});

// Mock admin user (from ADMIN_SETUP.md)
const ADMIN_USER = {
  identity_id: 'admin_identity_001',
  full_name: 'Smart Ledger Solutions Admin',
  email: 'yourfriends@smartledger.solutions',
  password: 'BoundlessTrust', // In real system, this would be hashed
  phone: '+1-555-0100',
  country_code: 'USA',
  verification_status: 'verified',
  kyc_level: 3,
  created_at: new Date().toISOString(),
  updated_at: new Date().toISOString()
};

// Mock session store
const sessions = new Map();

// ============================================================================
// AUTHENTICATION ENDPOINTS
// ============================================================================

// POST /api/auth/login
app.post('/api/auth/login', (req, res) => {
  const { email, password } = req.body;

  console.log(`[LOGIN] Attempt: ${email}`);

  if (email === ADMIN_USER.email && password === ADMIN_USER.password) {
    const session_id = `session_${Date.now()}`;
    const token = `token_${Date.now()}_${Math.random().toString(36).substring(7)}`;

    sessions.set(session_id, {
      session_id,
      identity_id: ADMIN_USER.identity_id,
      token,
      created_at: new Date().toISOString(),
      expires_at: new Date(Date.now() + 24 * 60 * 60 * 1000).toISOString() // 24 hours
    });

    console.log(`[LOGIN] Success: ${email}`);

    res.json({
      data: {
        token,
        session: {
          session_id,
          identity_id: ADMIN_USER.identity_id,
          expires_at: new Date(Date.now() + 24 * 60 * 60 * 1000).toISOString()
        }
      }
    });
  } else {
    console.log(`[LOGIN] Failed: ${email}`);
    res.status(401).json({
      error: 'Invalid email or password'
    });
  }
});

// POST /api/auth/logout
app.post('/api/auth/logout', (req, res) => {
  const authHeader = req.headers.authorization;
  if (authHeader) {
    const token = authHeader.split(' ')[1];
    // Find and remove session
    for (const [session_id, session] of sessions.entries()) {
      if (session.token === token) {
        sessions.delete(session_id);
        break;
      }
    }
  }

  res.json({ data: { success: true } });
});

// ============================================================================
// IDENTITY ENDPOINTS
// ============================================================================

// GET /api/identity/:id
app.get('/api/identity/:id', (req, res) => {
  const { id } = req.params;

  if (id === ADMIN_USER.identity_id) {
    res.json({
      data: {
        identity_id: ADMIN_USER.identity_id,
        full_name: ADMIN_USER.full_name,
        email: ADMIN_USER.email,
        phone: ADMIN_USER.phone,
        country_code: ADMIN_USER.country_code,
        verification_status: ADMIN_USER.verification_status,
        kyc_level: ADMIN_USER.kyc_level,
        created_at: ADMIN_USER.created_at,
        updated_at: ADMIN_USER.updated_at
      }
    });
  } else {
    res.status(404).json({
      error: 'Identity not found'
    });
  }
});

// GET /api/identity/lookup
app.get('/api/identity/lookup', (req, res) => {
  const { email } = req.query;

  if (email === ADMIN_USER.email) {
    res.json({
      data: {
        identity_id: ADMIN_USER.identity_id
      }
    });
  } else {
    res.status(404).json({
      error: 'Identity not found'
    });
  }
});

// ============================================================================
// WALLET ENDPOINTS (Mock Data)
// ============================================================================

// GET /api/wallets/identity/:id
app.get('/api/wallets/identity/:id', (req, res) => {
  res.json({
    data: [{
      wallet_id: 'wallet_admin_001',
      identity_id: req.params.id,
      boundless_addresses: [
        {
          address: 'bls1admin000000000000000000000000',
          label: 'Admin Main Wallet'
        }
      ],
      labels: ['admin', 'primary'],
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString()
    }]
  });
});

// ============================================================================
// HEALTH CHECK
// ============================================================================

app.get('/health', (req, res) => {
  res.json({
    status: 'ok',
    service: 'Boundless E² Multipass Mock Backend',
    timestamp: new Date().toISOString(),
    blockchain_connected: true,
    database_connected: true
  });
});

// ============================================================================
// START SERVER
// ============================================================================

app.listen(PORT, () => {
  console.log('='.repeat(70));
  console.log('🚀 Boundless E² Multipass Mock Backend Server');
  console.log('='.repeat(70));
  console.log(`Server running at: http://localhost:${PORT}`);
  console.log(`Health check: http://localhost:${PORT}/health`);
  console.log('');
  console.log('Admin Credentials:');
  console.log(`  Email:    ${ADMIN_USER.email}`);
  console.log(`  Password: ${ADMIN_USER.password}`);
  console.log('');
  console.log('⚠️  This is a MOCK backend for frontend demonstration only');
  console.log('⚠️  Not connected to real blockchain or database');
  console.log('='.repeat(70));
});
