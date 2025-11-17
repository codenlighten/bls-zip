#![cfg_attr(not(feature = "std"), no_std, no_main)]

/// Multi-party escrow contract with time locks and dispute resolution
#[ink::contract]
mod boundless_escrow {
    use ink::prelude::vec::Vec;

    /// Escrow states
    #[derive(Debug, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub enum EscrowState {
        /// Funds deposited, awaiting release
        Active,
        /// Funds released to beneficiary
        Released,
        /// Funds returned to depositor
        Refunded,
        /// Dispute raised, awaiting arbitration
        Disputed,
    }

    /// Escrow agreement
    #[ink(storage)]
    pub struct BoundlessEscrow {
        /// Depositor (payer)
        depositor: AccountId,
        /// Beneficiary (payee)
        beneficiary: AccountId,
        /// Optional arbiter for dispute resolution
        arbiter: Option<AccountId>,
        /// Escrowed amount
        amount: Balance,
        /// Current state
        state: EscrowState,
        /// Release deadline (timestamp)
        release_deadline: Timestamp,
        /// Auto-refund deadline (timestamp)
        refund_deadline: Timestamp,
        /// Condition description
        condition: String,
    }

    /// Events
    #[ink(event)]
    pub struct Deposited {
        #[ink(topic)]
        depositor: AccountId,
        amount: Balance,
    }

    #[ink(event)]
    pub struct Released {
        #[ink(topic)]
        beneficiary: AccountId,
        amount: Balance,
    }

    #[ink(event)]
    pub struct Refunded {
        #[ink(topic)]
        depositor: AccountId,
        amount: Balance,
    }

    #[ink(event)]
    pub struct DisputeRaised {
        #[ink(topic)]
        raised_by: AccountId,
    }

    #[ink(event)]
    pub struct DisputeResolved {
        in_favor_of: AccountId,
    }

    /// Escrow errors
    #[derive(Debug, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub enum Error {
        NotDepositor,
        NotBeneficiary,
        NotArbiter,
        InvalidState,
        DeadlineNotReached,
        DeadlinePassed,
        InsufficientFunds,
        TransferFailed,
        NoArbiter,
    }

    pub type Result<T> = core::result::Result<T, Error>;

    impl BoundlessEscrow {
        /// Create a new escrow agreement
        #[ink(constructor, payable)]
        pub fn new(
            beneficiary: AccountId,
            arbiter: Option<AccountId>,
            release_deadline_secs: u64,
            refund_deadline_secs: u64,
            condition: String,
        ) -> Self {
            let depositor = Self::env().caller();
            let amount = Self::env().transferred_value();
            let now = Self::env().block_timestamp();

            Self::env().emit_event(Deposited {
                depositor,
                amount,
            });

            Self {
                depositor,
                beneficiary,
                arbiter,
                amount,
                state: EscrowState::Active,
                release_deadline: now + release_deadline_secs * 1000,
                refund_deadline: now + refund_deadline_secs * 1000,
                condition,
            }
        }

        /// Get escrow details
        #[ink(message)]
        pub fn get_details(&self) -> (
            AccountId,
            AccountId,
            Balance,
            EscrowState,
            Timestamp,
            Timestamp,
        ) {
            (
                self.depositor,
                self.beneficiary,
                self.amount,
                self.state.clone(),
                self.release_deadline,
                self.refund_deadline,
            )
        }

        /// Get condition
        #[ink(message)]
        pub fn condition(&self) -> String {
            self.condition.clone()
        }

        /// Release funds to beneficiary (can be called by depositor or arbiter)
        #[ink(message)]
        pub fn release(&mut self) -> Result<()> {
            let caller = self.env().caller();
            let now = self.env().block_timestamp();

            // Check state
            if self.state != EscrowState::Active {
                return Err(Error::InvalidState);
            }

            // Check authorization
            let is_depositor = caller == self.depositor;
            let is_arbiter = self.arbiter.map_or(false, |a| a == caller);

            if !is_depositor && !is_arbiter {
                return Err(Error::NotDepositor);
            }

            // Check deadline
            if now < self.release_deadline && !is_arbiter {
                return Err(Error::DeadlineNotReached);
            }

            // Transfer funds
            if self.env().transfer(self.beneficiary, self.amount).is_err() {
                return Err(Error::TransferFailed);
            }

            self.state = EscrowState::Released;

            self.env().emit_event(Released {
                beneficiary: self.beneficiary,
                amount: self.amount,
            });

            Ok(())
        }

        /// Refund funds to depositor (can be called by beneficiary or arbiter)
        #[ink(message)]
        pub fn refund(&mut self) -> Result<()> {
            let caller = self.env().caller();
            let now = self.env().block_timestamp();

            // Check state
            if self.state != EscrowState::Active {
                return Err(Error::InvalidState);
            }

            // Check authorization
            let is_beneficiary = caller == self.beneficiary;
            let is_arbiter = self.arbiter.map_or(false, |a| a == caller);
            let is_depositor = caller == self.depositor;

            // Depositor can refund after refund deadline
            let can_auto_refund = is_depositor && now >= self.refund_deadline;

            if !is_beneficiary && !is_arbiter && !can_auto_refund {
                return Err(Error::NotBeneficiary);
            }

            // Transfer funds
            if self.env().transfer(self.depositor, self.amount).is_err() {
                return Err(Error::TransferFailed);
            }

            self.state = EscrowState::Refunded;

            self.env().emit_event(Refunded {
                depositor: self.depositor,
                amount: self.amount,
            });

            Ok(())
        }

        /// Raise a dispute (can be called by depositor or beneficiary)
        #[ink(message)]
        pub fn raise_dispute(&mut self) -> Result<()> {
            let caller = self.env().caller();

            // Check state
            if self.state != EscrowState::Active {
                return Err(Error::InvalidState);
            }

            // Check if arbiter exists
            if self.arbiter.is_none() {
                return Err(Error::NoArbiter);
            }

            // Check authorization
            if caller != self.depositor && caller != self.beneficiary {
                return Err(Error::NotDepositor);
            }

            self.state = EscrowState::Disputed;

            self.env().emit_event(DisputeRaised {
                raised_by: caller,
            });

            Ok(())
        }

        /// Resolve dispute (only arbiter can call)
        #[ink(message)]
        pub fn resolve_dispute(&mut self, in_favor_of_beneficiary: bool) -> Result<()> {
            let caller = self.env().caller();

            // Check state
            if self.state != EscrowState::Disputed {
                return Err(Error::InvalidState);
            }

            // Check authorization
            let arbiter = self.arbiter.ok_or(Error::NoArbiter)?;
            if caller != arbiter {
                return Err(Error::NotArbiter);
            }

            // Resolve in favor of beneficiary or depositor
            if in_favor_of_beneficiary {
                if self.env().transfer(self.beneficiary, self.amount).is_err() {
                    return Err(Error::TransferFailed);
                }
                self.state = EscrowState::Released;
            } else {
                if self.env().transfer(self.depositor, self.amount).is_err() {
                    return Err(Error::TransferFailed);
                }
                self.state = EscrowState::Refunded;
            }

            self.env().emit_event(DisputeResolved {
                in_favor_of: if in_favor_of_beneficiary {
                    self.beneficiary
                } else {
                    self.depositor
                },
            });

            Ok(())
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn new_works() {
            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();

            ink::env::test::set_value_transferred::<ink::env::DefaultEnvironment>(1000);

            let escrow = BoundlessEscrow::new(
                accounts.bob,
                Some(accounts.charlie),
                3600,
                7200,
                "Deliver widgets".to_string(),
            );

            let (depositor, beneficiary, amount, state, _, _) = escrow.get_details();
            assert_eq!(depositor, accounts.alice);
            assert_eq!(beneficiary, accounts.bob);
            assert_eq!(amount, 1000);
            assert_eq!(state, EscrowState::Active);
        }

        #[ink::test]
        fn release_works() {
            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
            ink::env::test::set_value_transferred::<ink::env::DefaultEnvironment>(1000);

            let mut escrow = BoundlessEscrow::new(
                accounts.bob,
                None,
                0, // No deadline
                7200,
                "Test".to_string(),
            );

            // Set enough balance for the contract
            ink::env::test::set_account_balance::<ink::env::DefaultEnvironment>(
                escrow.env().account_id(),
                1000,
            );

            assert!(escrow.release().is_ok());
            let (_, _, _, state, _, _) = escrow.get_details();
            assert_eq!(state, EscrowState::Released);
        }
    }
}
