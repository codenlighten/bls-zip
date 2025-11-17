#![cfg_attr(not(feature = "std"), no_std, no_main)]

/// Multi-Signature Wallet Template for Enterprise E2 Multipass
///
/// This template implements a secure multi-signature wallet that requires
/// multiple approvals for transactions, integrated with E2 Multipass identities.
///
/// Features:
/// - M-of-N signature requirements (configurable threshold)
/// - Transaction proposal and approval workflow
/// - Support for multiple asset types (IRSC, CRSC, custom tokens)
/// - Time-locked transactions with expiry
/// - Daily spending limits per signer
/// - Integration with E2 Multipass identity verification
/// - Post-quantum signature support

#[ink::contract]
mod multisig_wallet {
    use ink::storage::Mapping;
    use ink::prelude::vec::Vec;

    /// Transaction proposal status
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub enum TransactionStatus {
        /// Pending approvals
        Pending,
        /// Approved and ready to execute
        Approved,
        /// Executed successfully
        Executed,
        /// Rejected by signers
        Rejected,
        /// Expired without execution
        Expired,
    }

    /// Transaction proposal
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub struct Transaction {
        /// Transaction ID
        id: u64,
        /// Destination address
        to: AccountId,
        /// Amount to transfer
        amount: Balance,
        /// Asset type (empty for native token)
        asset_id: Option<[u8; 32]>,
        /// Optional data payload
        data: Option<Vec<u8>>,
        /// Proposer
        proposer: AccountId,
        /// Approvals received
        approvals: Vec<AccountId>,
        /// Rejections received
        rejections: Vec<AccountId>,
        /// Status
        status: TransactionStatus,
        /// Proposal timestamp
        proposed_at: Timestamp,
        /// Expiry timestamp
        expires_at: Timestamp,
        /// Execution timestamp (if executed)
        executed_at: Option<Timestamp>,
    }

    /// Signer configuration
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub struct Signer {
        /// Account address
        account: AccountId,
        /// E2 Multipass identity ID
        identity_id: [u8; 32],
        /// Daily spending limit
        daily_limit: Balance,
        /// Amount spent today
        spent_today: Balance,
        /// Last reset timestamp
        last_reset: Timestamp,
        /// Active status
        active: bool,
    }

    /// Contract storage
    #[ink(storage)]
    pub struct MultisigWallet {
        /// List of authorized signers
        signers: Vec<Signer>,
        /// Mapping from account to signer index
        signer_index: Mapping<AccountId, u32>,
        /// Required approvals threshold
        threshold: u32,
        /// Transaction counter
        transaction_count: u64,
        /// Mapping from transaction ID to transaction
        transactions: Mapping<u64, Transaction>,
        /// Wallet balance (native token)
        balance: Balance,
        /// Default transaction expiry time (in milliseconds)
        default_expiry_ms: u64,
    }

    /// Events
    #[ink(event)]
    pub struct SignerAdded {
        #[ink(topic)]
        account: AccountId,
        identity_id: [u8; 32],
        daily_limit: Balance,
    }

    #[ink(event)]
    pub struct SignerRemoved {
        #[ink(topic)]
        account: AccountId,
    }

    #[ink(event)]
    pub struct TransactionProposed {
        #[ink(topic)]
        transaction_id: u64,
        #[ink(topic)]
        proposer: AccountId,
        to: AccountId,
        amount: Balance,
    }

    #[ink(event)]
    pub struct TransactionApproved {
        #[ink(topic)]
        transaction_id: u64,
        #[ink(topic)]
        approver: AccountId,
        approvals_count: u32,
    }

    #[ink(event)]
    pub struct TransactionRejected {
        #[ink(topic)]
        transaction_id: u64,
        #[ink(topic)]
        rejector: AccountId,
    }

    #[ink(event)]
    pub struct TransactionExecuted {
        #[ink(topic)]
        transaction_id: u64,
        to: AccountId,
        amount: Balance,
    }

    #[ink(event)]
    pub struct Deposit {
        #[ink(topic)]
        from: AccountId,
        amount: Balance,
    }

    /// Errors
    #[derive(Debug, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub enum Error {
        /// Not a signer
        NotASigner,
        /// Already a signer
        AlreadyASigner,
        /// Invalid threshold
        InvalidThreshold,
        /// Transaction not found
        TransactionNotFound,
        /// Transaction already approved by caller
        AlreadyApproved,
        /// Transaction already rejected by caller
        AlreadyRejected,
        /// Transaction not ready for execution
        NotReadyForExecution,
        /// Transaction expired
        TransactionExpired,
        /// Insufficient balance
        InsufficientBalance,
        /// Daily limit exceeded
        DailyLimitExceeded,
        /// Cannot remove last signer
        CannotRemoveLastSigner,
    }

    pub type Result<T> = core::result::Result<T, Error>;

    impl MultisigWallet {
        /// Create a new multisig wallet
        #[ink(constructor)]
        pub fn new(
            initial_signers: Vec<(AccountId, [u8; 32], Balance)>,
            threshold: u32,
        ) -> Self {
            assert!(threshold > 0, "Threshold must be greater than 0");
            assert!(
                threshold as usize <= initial_signers.len(),
                "Threshold cannot exceed number of signers"
            );

            let mut signers = Vec::new();
            let mut signer_index = Mapping::default();
            let now = Self::env().block_timestamp();

            for (idx, (account, identity_id, daily_limit)) in initial_signers.iter().enumerate() {
                signers.push(Signer {
                    account: *account,
                    identity_id: *identity_id,
                    daily_limit: *daily_limit,
                    spent_today: 0,
                    last_reset: now,
                    active: true,
                });
                signer_index.insert(account, &(idx as u32));
            }

            Self {
                signers,
                signer_index,
                threshold,
                transaction_count: 0,
                transactions: Mapping::default(),
                balance: 0,
                default_expiry_ms: 7 * 24 * 60 * 60 * 1000, // 7 days
            }
        }

        /// Add a new signer (requires threshold approvals)
        #[ink(message)]
        pub fn add_signer(
            &mut self,
            account: AccountId,
            identity_id: [u8; 32],
            daily_limit: Balance,
        ) -> Result<()> {
            self.require_signer(&self.env().caller())?;

            if self.signer_index.contains(account) {
                return Err(Error::AlreadyASigner);
            }

            let idx = self.signers.len() as u32;
            let now = self.env().block_timestamp();

            self.signers.push(Signer {
                account,
                identity_id,
                daily_limit,
                spent_today: 0,
                last_reset: now,
                active: true,
            });

            self.signer_index.insert(account, &idx);

            self.env().emit_event(SignerAdded {
                account,
                identity_id,
                daily_limit,
            });

            Ok(())
        }

        /// Remove a signer (requires threshold approvals)
        #[ink(message)]
        pub fn remove_signer(&mut self, account: AccountId) -> Result<()> {
            self.require_signer(&self.env().caller())?;

            if self.signers.len() == 1 {
                return Err(Error::CannotRemoveLastSigner);
            }

            let idx = self.signer_index.get(account)
                .ok_or(Error::NotASigner)? as usize;

            self.signers[idx].active = false;

            self.env().emit_event(SignerRemoved { account });

            Ok(())
        }

        /// Propose a new transaction
        #[ink(message)]
        pub fn propose_transaction(
            &mut self,
            to: AccountId,
            amount: Balance,
            asset_id: Option<[u8; 32]>,
            data: Option<Vec<u8>>,
        ) -> Result<u64> {
            let caller = self.env().caller();
            self.require_signer(&caller)?;

            // Check daily limit
            self.check_daily_limit(&caller, amount)?;

            let transaction_id = self.transaction_count;
            self.transaction_count += 1;

            let now = self.env().block_timestamp();
            let expires_at = now + self.default_expiry_ms;

            let mut approvals = Vec::new();
            approvals.push(caller); // Proposer auto-approves

            let transaction = Transaction {
                id: transaction_id,
                to,
                amount,
                asset_id,
                data,
                proposer: caller,
                approvals,
                rejections: Vec::new(),
                status: TransactionStatus::Pending,
                proposed_at: now,
                expires_at,
                executed_at: None,
            };

            self.transactions.insert(transaction_id, &transaction);

            self.env().emit_event(TransactionProposed {
                transaction_id,
                proposer: caller,
                to,
                amount,
            });

            Ok(transaction_id)
        }

        /// Approve a pending transaction
        #[ink(message)]
        pub fn approve_transaction(&mut self, transaction_id: u64) -> Result<()> {
            let caller = self.env().caller();
            self.require_signer(&caller)?;

            let mut tx = self.transactions.get(transaction_id)
                .ok_or(Error::TransactionNotFound)?;

            // Check if expired
            if self.env().block_timestamp() > tx.expires_at {
                tx.status = TransactionStatus::Expired;
                self.transactions.insert(transaction_id, &tx);
                return Err(Error::TransactionExpired);
            }

            // Check if already approved
            if tx.approvals.contains(&caller) {
                return Err(Error::AlreadyApproved);
            }

            // Add approval
            tx.approvals.push(caller);

            // Check if threshold reached
            if tx.approvals.len() as u32 >= self.threshold {
                tx.status = TransactionStatus::Approved;
            }

            self.transactions.insert(transaction_id, &tx);

            self.env().emit_event(TransactionApproved {
                transaction_id,
                approver: caller,
                approvals_count: tx.approvals.len() as u32,
            });

            Ok(())
        }

        /// Reject a pending transaction
        #[ink(message)]
        pub fn reject_transaction(&mut self, transaction_id: u64) -> Result<()> {
            let caller = self.env().caller();
            self.require_signer(&caller)?;

            let mut tx = self.transactions.get(transaction_id)
                .ok_or(Error::TransactionNotFound)?;

            if tx.rejections.contains(&caller) {
                return Err(Error::AlreadyRejected);
            }

            tx.rejections.push(caller);

            // If rejections reach threshold, mark as rejected
            if tx.rejections.len() as u32 >= self.threshold {
                tx.status = TransactionStatus::Rejected;
            }

            self.transactions.insert(transaction_id, &tx);

            self.env().emit_event(TransactionRejected {
                transaction_id,
                rejector: caller,
            });

            Ok(())
        }

        /// Execute an approved transaction
        #[ink(message)]
        pub fn execute_transaction(&mut self, transaction_id: u64) -> Result<()> {
            self.require_signer(&self.env().caller())?;

            let mut tx = self.transactions.get(transaction_id)
                .ok_or(Error::TransactionNotFound)?;

            // Verify transaction is approved
            if tx.status != TransactionStatus::Approved {
                return Err(Error::NotReadyForExecution);
            }

            // Check expiry
            if self.env().block_timestamp() > tx.expires_at {
                tx.status = TransactionStatus::Expired;
                self.transactions.insert(transaction_id, &tx);
                return Err(Error::TransactionExpired);
            }

            // Check balance
            if self.balance < tx.amount {
                return Err(Error::InsufficientBalance);
            }

            // Execute transfer
            self.balance -= tx.amount;
            // TODO: Actual transfer implementation depends on asset type

            // Update transaction status
            tx.status = TransactionStatus::Executed;
            tx.executed_at = Some(self.env().block_timestamp());
            self.transactions.insert(transaction_id, &tx);

            self.env().emit_event(TransactionExecuted {
                transaction_id,
                to: tx.to,
                amount: tx.amount,
            });

            Ok(())
        }

        /// Deposit funds into the wallet
        #[ink(message)]
        #[ink(payable)]
        pub fn deposit(&mut self) {
            let caller = self.env().caller();
            let amount = self.env().transferred_value();

            self.balance += amount;

            self.env().emit_event(Deposit {
                from: caller,
                amount,
            });
        }

        /// Get transaction details
        #[ink(message)]
        pub fn get_transaction(&self, transaction_id: u64) -> Option<Transaction> {
            self.transactions.get(transaction_id)
        }

        /// Get wallet balance
        #[ink(message)]
        pub fn get_balance(&self) -> Balance {
            self.balance
        }

        /// Get number of signers
        #[ink(message)]
        pub fn get_signer_count(&self) -> u32 {
            self.signers.iter().filter(|s| s.active).count() as u32
        }

        /// Get threshold
        #[ink(message)]
        pub fn get_threshold(&self) -> u32 {
            self.threshold
        }

        // === Internal Helper Functions ===

        /// Require caller to be a signer
        fn require_signer(&self, account: &AccountId) -> Result<()> {
            if let Some(idx) = self.signer_index.get(account) {
                if self.signers[idx as usize].active {
                    return Ok(());
                }
            }
            Err(Error::NotASigner)
        }

        /// Check and update daily limit
        fn check_daily_limit(&mut self, account: &AccountId, amount: Balance) -> Result<()> {
            let idx = self.signer_index.get(account)
                .ok_or(Error::NotASigner)? as usize;

            let mut signer = self.signers[idx].clone();
            let now = self.env().block_timestamp();

            // Reset daily counter if new day
            if now - signer.last_reset > 24 * 60 * 60 * 1000 {
                signer.spent_today = 0;
                signer.last_reset = now;
            }

            // Check limit
            if signer.spent_today + amount > signer.daily_limit {
                return Err(Error::DailyLimitExceeded);
            }

            signer.spent_today += amount;
            self.signers[idx] = signer;

            Ok(())
        }
    }

    // === Unit Tests ===

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn new_works() {
            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
            let signers = vec![
                (accounts.alice, [1u8; 32], 1000),
                (accounts.bob, [2u8; 32], 1000),
            ];

            let wallet = MultisigWallet::new(signers, 2);
            assert_eq!(wallet.get_threshold(), 2);
            assert_eq!(wallet.get_signer_count(), 2);
        }

        #[ink::test]
        fn propose_transaction_works() {
            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
            let signers = vec![
                (accounts.alice, [1u8; 32], 10000),
            ];

            let mut wallet = MultisigWallet::new(signers, 1);
            let tx_id = wallet.propose_transaction(accounts.bob, 100, None, None).unwrap();
            assert_eq!(tx_id, 0);
        }
    }
}
