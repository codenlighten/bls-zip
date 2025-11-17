-- Seed admin user for Boundless Enterprise Multipass
-- Email: yourfriends@smartledger.solutions
-- Password: BoundlessTrust

-- Note: Password is hashed using Argon2
-- In a production system, you would hash this server-side
-- For development, we'll insert the plain password and let the app hash it

-- Insert admin identity
INSERT INTO identity_profiles (
    identity_id,
    full_name,
    email,
    phone,
    country_code,
    date_of_birth,
    verification_status,
    kyc_level,
    created_at,
    updated_at
) VALUES (
    'a0000000-0000-0000-0000-000000000001'::uuid,
    'Smart Ledger Solutions Admin',
    'yourfriends@smartledger.solutions',
    '+1-555-0100',
    'USA',
    NULL,
    'verified',
    3,
    NOW(),
    NOW()
) ON CONFLICT (email) DO NOTHING;

-- Insert multipass credentials
-- Password: "BoundlessTrust" hashed with Argon2
-- Hash generated using: echo -n "BoundlessTrust" | argon2 somesalt -id
INSERT INTO multipass_credentials (
    credential_id,
    identity_id,
    username,
    password_hash,
    totp_secret,
    backup_codes,
    require_2fa,
    locked,
    failed_attempts,
    last_login_at,
    created_at,
    updated_at
) VALUES (
    'b0000000-0000-0000-0000-000000000001'::uuid,
    'a0000000-0000-0000-0000-000000000001'::uuid,
    'yourfriends@smartledger.solutions',
    -- This is a placeholder - the app will hash the password on first login
    '$argon2id$v=19$m=19456,t=2,p=1$c29tZXNhbHQ$qLml4oZB/6wHPQzKLC6vKp6WYoL7bEqM6rJLCEuH6zw',
    NULL,
    ARRAY[]::text[],
    false,
    false,
    0,
    NULL,
    NOW(),
    NOW()
) ON CONFLICT (identity_id) DO NOTHING;

-- Verify insertion
SELECT identity_id, full_name, email, verification_status, kyc_level
FROM identity_profiles
WHERE email = 'yourfriends@smartledger.solutions';
