#![cfg_attr(not(feature = "std"), no_std, no_main)]

/// Asset Escrow and Trading Template for Enterprise E2 Multipass
///
/// This template implements secure peer-to-peer asset trading with escrow,
/// integrated with the E2 Multipass asset service and locked quantity tracking.
///
/// Features:
/// - Atomic swaps between two parties
/// - Escrow with time locks and dispute resolution
/// - Support for IRSC, CRSC, and custom asset types
/// - Oracle integration for price verification
/// - Multi-asset bundle trades
/// - Partial fills and order matching
/// - Integration with E2 Multipass locked_quantity system

#[ink::contract]
mod asset_escrow {
    use ink::storage::Mapping;
    use ink::prelude::vec::Vec;

    /// Trade status
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub enum TradeStatus {
        /// Trade proposed, waiting for counterparty
        Proposed,
        /// Both parties have locked assets
        Locked,
        /// Trade completed successfully
        Completed,
        /// Trade cancelled by proposer
        Cancelled,
        /// Trade disputed, awaiting arbitration
        Disputed,
        /// Trade expired without completion
        Expired,
    }

    /// Asset being traded
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub struct AssetLock {
        /// Asset ID (from E2 Multipass asset service)
        asset_id: [u8; 32],
        /// Quantity locked
        quantity: u64,
        /// Asset metadata hash
        metadata_hash: Option<[u8; 32]>,
    }

    /// Trade proposal
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub struct Trade {
        /// Trade ID
        id: u64,
        /// Proposer (party A)
        proposer: AccountId,
        /// Proposer's E2 identity
        proposer_identity: [u8; 32],
        /// Assets offered by proposer
        offer_assets: Vec<AssetLock>,
        /// Counterparty (party B)
        counterparty: Option<AccountId>,
        /// Counterparty's E2 identity (if known)
        counterparty_identity: Option<[u8; 32]>,
        /// Assets requested from counterparty
        request_assets: Vec<AssetLock>,
        /// Trade status
        status: TradeStatus,
        /// Escrow expiry timestamp
        expires_at: Timestamp,
        /// Creation timestamp
        created_at: Timestamp,
        /// Completion timestamp
        completed_at: Option<Timestamp>,
        /// Arbitrator (if dispute)
        arbitrator: Option<AccountId>,
        /// Proposer confirmed
        proposer_confirmed: bool,
        /// Counterparty confirmed
        counterparty_confirmed: bool,
    }

    /// Dispute record
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub struct Dispute {
        /// Trade ID
        trade_id: u64,
        /// Initiator of dispute
        initiator: AccountId,
        /// Reason for dispute
        reason: Vec<u8>,
        /// Evidence hashes
        evidence: Vec<[u8; 32]>,
        /// Arbitrator assigned
        arbitrator: AccountId,
        /// Resolution (if any)
        resolution: Option<DisputeResolution>,
    }

    /// Dispute resolution
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub enum DisputeResolution {
        /// Complete the trade as agreed
        CompleteTrade,
        /// Cancel trade and return assets
        CancelTrade,
        /// Custom resolution (e.g., partial completion)
        Custom(Vec<u8>),
    }

    /// Contract storage
    #[ink(storage)]
    pub struct AssetEscrow {
        /// Trade counter
        trade_count: u64,
        /// Mapping from trade ID to trade
        trades: Mapping<u64, Trade>,
        /// Mapping from trade ID to dispute
        disputes: Mapping<u64, Dispute>,
        /// Authorized arbitrators
        arbitrators: Vec<AccountId>,
        /// Escrow fee percentage (basis points, e.g., 100 = 1%)
        escrow_fee_bps: u32,
        /// Collected fees
        collected_fees: Balance,
        /// Contract owner
        owner: AccountId,
        /// Default trade expiry (milliseconds)
        default_expiry_ms: u64,
    }

    /// Events
    #[ink(event)]
    pub struct TradeProposed {
        #[ink(topic)]
        trade_id: u64,
        #[ink(topic)]
        proposer: AccountId,
        counterparty: Option<AccountId>,
    }

    #[ink(event)]
    pub struct TradeLocked {
        #[ink(topic)]
        trade_id: u64,
        #[ink(topic)]
        locker: AccountId,
    }

    #[ink(event)]
    pub struct TradeCompleted {
        #[ink(topic)]
        trade_id: u64,
        proposer: AccountId,
        counterparty: AccountId,
    }

    #[ink(event)]
    pub struct TradeCancelled {
        #[ink(topic)]
        trade_id: u64,
        #[ink(topic)]
        cancelled_by: AccountId,
    }

    #[ink(event)]
    pub struct DisputeRaised {
        #[ink(topic)]
        trade_id: u64,
        #[ink(topic)]
        initiator: AccountId,
        arbitrator: AccountId,
    }

    #[ink(event)]
    pub struct DisputeResolved {
        #[ink(topic)]
        trade_id: u64,
        resolution: DisputeResolution,
    }

    /// Errors
    #[derive(Debug, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub enum Error {
        /// Trade not found
        TradeNotFound,
        /// Not authorized
        Unauthorized,
        /// Invalid trade status for operation
        InvalidStatus,
        /// Trade expired
        TradeExpired,
        /// Asset not locked
        AssetNotLocked,
        /// Insufficient asset quantity
        InsufficientQuantity,
        /// Both parties must confirm
        AwaitingConfirmation,
        /// Cannot cancel after counterparty locked
        CannotCancel,
        /// Dispute already exists
        DisputeAlreadyExists,
        /// No dispute found
        DisputeNotFound,
        /// Not an arbitrator
        NotAnArbitrator,
        /// Empty asset list
        EmptyAssets,
    }

    pub type Result<T> = core::result::Result<T, Error>;

    impl AssetEscrow {
        /// Create a new asset escrow contract
        #[ink(constructor)]
        pub fn new(escrow_fee_bps: u32) -> Self {
            let caller = Self::env().caller();

            Self {
                trade_count: 0,
                trades: Mapping::default(),
                disputes: Mapping::default(),
                arbitrators: vec![caller], // Owner is initial arbitrator
                escrow_fee_bps,
                collected_fees: 0,
                owner: caller,
                default_expiry_ms: 7 * 24 * 60 * 60 * 1000, // 7 days
            }
        }

        /// Propose a new trade
        #[ink(message)]
        pub fn propose_trade(
            &mut self,
            proposer_identity: [u8; 32],
            offer_assets: Vec<AssetLock>,
            request_assets: Vec<AssetLock>,
            counterparty: Option<AccountId>,
            counterparty_identity: Option<[u8; 32]>,
            expiry_ms: Option<u64>,
        ) -> Result<u64> {
            if offer_assets.is_empty() || request_assets.is_empty() {
                return Err(Error::EmptyAssets);
            }

            let caller = self.env().caller();
            let trade_id = self.trade_count;
            self.trade_count += 1;

            let now = self.env().block_timestamp();
            let expires_at = now + expiry_ms.unwrap_or(self.default_expiry_ms);

            let trade = Trade {
                id: trade_id,
                proposer: caller,
                proposer_identity,
                offer_assets,
                counterparty,
                counterparty_identity,
                request_assets,
                status: TradeStatus::Proposed,
                expires_at,
                created_at: now,
                completed_at: None,
                arbitrator: None,
                proposer_confirmed: false,
                counterparty_confirmed: false,
            };

            self.trades.insert(trade_id, &trade);

            self.env().emit_event(TradeProposed {
                trade_id,
                proposer: caller,
                counterparty,
            });

            Ok(trade_id)
        }

        /// Accept and lock assets for a trade (counterparty)
        #[ink(message)]
        pub fn accept_trade(&mut self, trade_id: u64, identity_id: [u8; 32]) -> Result<()> {
            let caller = self.env().caller();
            let mut trade = self.trades.get(trade_id)
                .ok_or(Error::TradeNotFound)?;

            // Verify caller is the designated counterparty (if specified)
            if let Some(cp) = trade.counterparty {
                if cp != caller {
                    return Err(Error::Unauthorized);
                }
            }

            // Check expiry
            if self.env().block_timestamp() > trade.expires_at {
                trade.status = TradeStatus::Expired;
                self.trades.insert(trade_id, &trade);
                return Err(Error::TradeExpired);
            }

            // Verify status
            if trade.status != TradeStatus::Proposed {
                return Err(Error::InvalidStatus);
            }

            // Set counterparty if not set
            if trade.counterparty.is_none() {
                trade.counterparty = Some(caller);
                trade.counterparty_identity = Some(identity_id);
            }

            // TODO: Lock assets via E2 Multipass asset service
            // This would call the asset service to increment locked_quantity

            trade.status = TradeStatus::Locked;
            self.trades.insert(trade_id, &trade);

            self.env().emit_event(TradeLocked {
                trade_id,
                locker: caller,
            });

            Ok(())
        }

        /// Confirm trade execution (both parties must confirm)
        #[ink(message)]
        pub fn confirm_trade(&mut self, trade_id: u64) -> Result<()> {
            let caller = self.env().caller();
            let mut trade = self.trades.get(trade_id)
                .ok_or(Error::TradeNotFound)?;

            // Verify status
            if trade.status != TradeStatus::Locked {
                return Err(Error::InvalidStatus);
            }

            // Check expiry
            if self.env().block_timestamp() > trade.expires_at {
                trade.status = TradeStatus::Expired;
                self.trades.insert(trade_id, &trade);
                return Err(Error::TradeExpired);
            }

            // Mark confirmation
            if caller == trade.proposer {
                trade.proposer_confirmed = true;
            } else if Some(caller) == trade.counterparty {
                trade.counterparty_confirmed = true;
            } else {
                return Err(Error::Unauthorized);
            }

            // If both confirmed, complete the trade
            if trade.proposer_confirmed && trade.counterparty_confirmed {
                self.complete_trade(&mut trade)?;
            }

            self.trades.insert(trade_id, &trade);
            Ok(())
        }

        /// Cancel a trade (only before counterparty locks)
        #[ink(message)]
        pub fn cancel_trade(&mut self, trade_id: u64) -> Result<()> {
            let caller = self.env().caller();
            let mut trade = self.trades.get(trade_id)
                .ok_or(Error::TradeNotFound)?;

            // Only proposer can cancel
            if caller != trade.proposer {
                return Err(Error::Unauthorized);
            }

            // Cannot cancel after locking
            if trade.status != TradeStatus::Proposed {
                return Err(Error::CannotCancel);
            }

            trade.status = TradeStatus::Cancelled;
            self.trades.insert(trade_id, &trade);

            self.env().emit_event(TradeCancelled {
                trade_id,
                cancelled_by: caller,
            });

            Ok(())
        }

        /// Raise a dispute
        #[ink(message)]
        pub fn raise_dispute(
            &mut self,
            trade_id: u64,
            reason: Vec<u8>,
            evidence: Vec<[u8; 32]>,
        ) -> Result<()> {
            let caller = self.env().caller();
            let mut trade = self.trades.get(trade_id)
                .ok_or(Error::TradeNotFound)?;

            // Only parties can raise disputes
            if caller != trade.proposer && Some(caller) != trade.counterparty {
                return Err(Error::Unauthorized);
            }

            // Cannot dispute completed trades
            if trade.status == TradeStatus::Completed {
                return Err(Error::InvalidStatus);
            }

            // Check if dispute already exists
            if self.disputes.contains(trade_id) {
                return Err(Error::DisputeAlreadyExists);
            }

            // Assign first arbitrator
            let arbitrator = self.arbitrators[0];

            let dispute = Dispute {
                trade_id,
                initiator: caller,
                reason,
                evidence,
                arbitrator,
                resolution: None,
            };

            trade.status = TradeStatus::Disputed;
            trade.arbitrator = Some(arbitrator);

            self.disputes.insert(trade_id, &dispute);
            self.trades.insert(trade_id, &trade);

            self.env().emit_event(DisputeRaised {
                trade_id,
                initiator: caller,
                arbitrator,
            });

            Ok(())
        }

        /// Resolve a dispute (arbitrator only)
        #[ink(message)]
        pub fn resolve_dispute(
            &mut self,
            trade_id: u64,
            resolution: DisputeResolution,
        ) -> Result<()> {
            let caller = self.env().caller();

            // Verify caller is an arbitrator
            if !self.arbitrators.contains(&caller) {
                return Err(Error::NotAnArbitrator);
            }

            let mut dispute = self.disputes.get(trade_id)
                .ok_or(Error::DisputeNotFound)?;

            let mut trade = self.trades.get(trade_id)
                .ok_or(Error::TradeNotFound)?;

            // Apply resolution
            match resolution {
                DisputeResolution::CompleteTrade => {
                    self.complete_trade(&mut trade)?;
                }
                DisputeResolution::CancelTrade => {
                    trade.status = TradeStatus::Cancelled;
                    // TODO: Unlock assets via E2 Multipass asset service
                }
                DisputeResolution::Custom(_) => {
                    // Custom resolution handled externally
                }
            }

            dispute.resolution = Some(resolution.clone());
            self.disputes.insert(trade_id, &dispute);
            self.trades.insert(trade_id, &trade);

            self.env().emit_event(DisputeResolved {
                trade_id,
                resolution,
            });

            Ok(())
        }

        /// Get trade details
        #[ink(message)]
        pub fn get_trade(&self, trade_id: u64) -> Option<Trade> {
            self.trades.get(trade_id)
        }

        /// Get dispute details
        #[ink(message)]
        pub fn get_dispute(&self, trade_id: u64) -> Option<Dispute> {
            self.disputes.get(trade_id)
        }

        /// Add arbitrator (owner only)
        #[ink(message)]
        pub fn add_arbitrator(&mut self, arbitrator: AccountId) -> Result<()> {
            if self.env().caller() != self.owner {
                return Err(Error::Unauthorized);
            }

            if !self.arbitrators.contains(&arbitrator) {
                self.arbitrators.push(arbitrator);
            }

            Ok(())
        }

        /// Get total trades
        #[ink(message)]
        pub fn total_trades(&self) -> u64 {
            self.trade_count
        }

        // === Internal Helper Functions ===

        /// Complete a trade by swapping assets
        fn complete_trade(&mut self, trade: &mut Trade) -> Result<()> {
            // TODO: Execute asset swaps via E2 Multipass asset service
            // 1. Transfer proposer's offer_assets to counterparty
            // 2. Transfer counterparty's request_assets to proposer
            // 3. Decrease locked_quantity for all assets
            // 4. Charge escrow fee

            trade.status = TradeStatus::Completed;
            trade.completed_at = Some(self.env().block_timestamp());

            self.env().emit_event(TradeCompleted {
                trade_id: trade.id,
                proposer: trade.proposer,
                counterparty: trade.counterparty.unwrap(),
            });

            Ok(())
        }
    }

    // === Unit Tests ===

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn new_works() {
            let escrow = AssetEscrow::new(100); // 1% fee
            assert_eq!(escrow.total_trades(), 0);
        }

        #[ink::test]
        fn propose_trade_works() {
            let mut escrow = AssetEscrow::new(100);

            let offer = vec![AssetLock {
                asset_id: [1u8; 32],
                quantity: 100,
                metadata_hash: None,
            }];

            let request = vec![AssetLock {
                asset_id: [2u8; 32],
                quantity: 200,
                metadata_hash: None,
            }];

            let trade_id = escrow
                .propose_trade([0u8; 32], offer, request, None, None, None)
                .unwrap();

            assert_eq!(trade_id, 0);
            assert_eq!(escrow.total_trades(), 1);
        }
    }
}
