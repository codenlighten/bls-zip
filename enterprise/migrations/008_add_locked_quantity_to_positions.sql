-- Add locked_quantity tracking to positions table
-- This enables escrow and pending trade functionality

ALTER TABLE positions
ADD COLUMN IF NOT EXISTS locked_quantity BIGINT NOT NULL DEFAULT 0;

-- Create index for queries filtering by locked quantities
CREATE INDEX IF NOT EXISTS idx_position_locked ON positions(locked_quantity) WHERE locked_quantity > 0;
