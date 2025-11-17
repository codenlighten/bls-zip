-- Migration 007: Add UNIQUE constraint to session token hash
--
-- FIX H-6: Session Token Hash Not Unique
--
-- This ensures token_hash collisions cannot cause authentication confusion
-- Each session token hash must be unique across all sessions

-- Add UNIQUE constraint to token_hash
ALTER TABLE multipass_sessions
ADD CONSTRAINT unique_session_token_hash
UNIQUE (token_hash);

-- Comments
COMMENT ON CONSTRAINT unique_session_token_hash ON multipass_sessions
IS 'Prevents token hash collisions - critical for authentication security';
