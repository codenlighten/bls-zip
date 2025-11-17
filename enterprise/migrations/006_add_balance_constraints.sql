-- Migration 006: Add balance constraints to prevent double-spend
--
-- FIX C-3: Balance Update Race Condition (Double-Spend Vulnerability)
--
-- This migration adds CHECK constraints to ensure balances can never go negative
-- Prevents double-spend attacks via concurrent transfer requests

-- Add CHECK constraint to prevent negative balances
ALTER TABLE wallet_balances
ADD CONSTRAINT check_non_negative_balance
CHECK (unlocked_amount >= 0 AND total_amount >= 0);

-- Add CHECK constraint to ensure locked amount is valid
ALTER TABLE wallet_balances
ADD CONSTRAINT check_locked_amount
CHECK (locked_amount >= 0 AND locked_amount <= total_amount);

-- Comments
COMMENT ON CONSTRAINT check_non_negative_balance ON wallet_balances
IS 'Prevents negative balances - critical for double-spend prevention';

COMMENT ON CONSTRAINT check_locked_amount ON wallet_balances
IS 'Ensures locked_amount is valid and within total_amount';
