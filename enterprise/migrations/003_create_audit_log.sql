-- Migration: Create Audit Log Table
-- Purpose: Store comprehensive audit trail for security-critical events

CREATE TABLE IF NOT EXISTS audit_log (
    -- Primary key
    event_id UUID PRIMARY KEY,

    -- Event metadata
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    event_type VARCHAR(50) NOT NULL,

    -- User context (nullable for system events)
    user_id UUID,
    ip_address VARCHAR(45), -- IPv6 compatible
    user_agent TEXT,

    -- Event details
    action VARCHAR(100) NOT NULL,
    resource VARCHAR(255) NOT NULL,
    result VARCHAR(20) NOT NULL,

    -- Additional data
    metadata JSONB DEFAULT '{}'::jsonb,

    -- Indexes for common queries
    CONSTRAINT fk_user FOREIGN KEY (user_id) REFERENCES identity_profiles(identity_id) ON DELETE SET NULL
);

-- Create indexes for performance
CREATE INDEX idx_audit_log_timestamp ON audit_log(timestamp DESC);
CREATE INDEX idx_audit_log_user_id ON audit_log(user_id);
CREATE INDEX idx_audit_log_event_type ON audit_log(event_type);
CREATE INDEX idx_audit_log_ip_address ON audit_log(ip_address);
CREATE INDEX idx_audit_log_result ON audit_log(result);

-- Create index on metadata for JSON queries
CREATE INDEX idx_audit_log_metadata ON audit_log USING GIN (metadata);

-- Create composite index for user audit log queries
CREATE INDEX idx_audit_log_user_timestamp ON audit_log(user_id, timestamp DESC);

-- Create composite index for security event queries
CREATE INDEX idx_audit_log_security_timestamp ON audit_log(event_type, timestamp DESC)
WHERE event_type = 'security_event';

-- Comment the table
COMMENT ON TABLE audit_log IS 'Comprehensive audit trail for all security-critical events';
COMMENT ON COLUMN audit_log.event_id IS 'Unique identifier for the audit event';
COMMENT ON COLUMN audit_log.timestamp IS 'When the event occurred';
COMMENT ON COLUMN audit_log.event_type IS 'Type of event (authentication, authorization, data_access, etc.)';
COMMENT ON COLUMN audit_log.user_id IS 'User who performed the action (nullable for system events)';
COMMENT ON COLUMN audit_log.ip_address IS 'IP address of the requester';
COMMENT ON COLUMN audit_log.user_agent IS 'User agent string of the client';
COMMENT ON COLUMN audit_log.action IS 'Action that was performed';
COMMENT ON COLUMN audit_log.resource IS 'Resource that was affected';
COMMENT ON COLUMN audit_log.result IS 'Result of the operation (success, failure, error)';
COMMENT ON COLUMN audit_log.metadata IS 'Additional contextual information in JSON format';
