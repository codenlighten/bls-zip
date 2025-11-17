#![cfg_attr(not(feature = "std"), no_std, no_main)]

/// Simple fungible token contract for Boundless BLS
#[ink::contract]
mod boundless_token {
    use ink::storage::Mapping;

    /// Token storage
    #[ink(storage)]
    pub struct BoundlessToken {
        /// Total token supply
        total_supply: Balance,
        /// Mapping from account to balance
        balances: Mapping<AccountId, Balance>,
        /// Mapping from (owner, spender) to allowance
        allowances: Mapping<(AccountId, AccountId), Balance>,
        /// Token name
        name: String,
        /// Token symbol
        symbol: String,
        /// Token decimals
        decimals: u8,
        /// Contract owner (for access control)
        owner: AccountId,
    }

    /// Event emitted when tokens are transferred
    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        value: Balance,
    }

    /// Event emitted when allowance is set
    #[ink(event)]
    pub struct Approval {
        #[ink(topic)]
        owner: AccountId,
        #[ink(topic)]
        spender: AccountId,
        value: Balance,
    }

    /// Token errors
    #[derive(Debug, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub enum Error {
        InsufficientBalance,
        InsufficientAllowance,
        Overflow,
    }

    pub type Result<T> = core::result::Result<T, Error>;

    impl BoundlessToken {
        /// Create a new token with initial supply
        #[ink(constructor)]
        pub fn new(
            name: String,
            symbol: String,
            decimals: u8,
            initial_supply: Balance,
        ) -> Self {
            let mut balances = Mapping::default();
            let caller = Self::env().caller();
            balances.insert(caller, &initial_supply);

            Self::env().emit_event(Transfer {
                from: None,
                to: Some(caller),
                value: initial_supply,
            });

            Self {
                total_supply: initial_supply,
                balances,
                allowances: Mapping::default(),
                name,
                symbol,
                decimals,
                owner: caller,
            }
        }

        /// Get token name
        #[ink(message)]
        pub fn name(&self) -> String {
            self.name.clone()
        }

        /// Get token symbol
        #[ink(message)]
        pub fn symbol(&self) -> String {
            self.symbol.clone()
        }

        /// Get token decimals
        #[ink(message)]
        pub fn decimals(&self) -> u8 {
            self.decimals
        }

        /// Get total supply
        #[ink(message)]
        pub fn total_supply(&self) -> Balance {
            self.total_supply
        }

        /// Get balance of account
        #[ink(message)]
        pub fn balance_of(&self, owner: AccountId) -> Balance {
            self.balances.get(&owner).unwrap_or(0)
        }

        /// Transfer tokens to another account
        #[ink(message)]
        pub fn transfer(&mut self, to: AccountId, value: Balance) -> Result<()> {
            let from = self.env().caller();
            self.transfer_from_to(&from, &to, value)
        }

        /// Approve spender to spend tokens on behalf of caller
        #[ink(message)]
        pub fn approve(&mut self, spender: AccountId, value: Balance) -> Result<()> {
            let owner = self.env().caller();
            self.allowances.insert((owner, spender), &value);

            self.env().emit_event(Approval {
                owner,
                spender,
                value,
            });

            Ok(())
        }

        /// Get allowance
        #[ink(message)]
        pub fn allowance(&self, owner: AccountId, spender: AccountId) -> Balance {
            self.allowances.get((owner, spender)).unwrap_or(0)
        }

        /// Transfer tokens from one account to another (requires allowance)
        #[ink(message)]
        pub fn transfer_from(
            &mut self,
            from: AccountId,
            to: AccountId,
            value: Balance,
        ) -> Result<()> {
            let caller = self.env().caller();
            let allowance = self.allowance(from, caller);

            if allowance < value {
                return Err(Error::InsufficientAllowance);
            }

            self.transfer_from_to(&from, &to, value)?;

            // Update allowance
            self.allowances.insert((from, caller), &(allowance - value));

            Ok(())
        }

        /// Internal transfer function
        fn transfer_from_to(
            &mut self,
            from: &AccountId,
            to: &AccountId,
            value: Balance,
        ) -> Result<()> {
            let from_balance = self.balance_of(*from);
            if from_balance < value {
                return Err(Error::InsufficientBalance);
            }

            self.balances.insert(from, &(from_balance - value));

            let to_balance = self.balance_of(*to);
            self.balances.insert(to, &(to_balance.checked_add(value)
                .ok_or(Error::Overflow)?));

            self.env().emit_event(Transfer {
                from: Some(*from),
                to: Some(*to),
                value,
            });

            Ok(())
        }

        /// Mint new tokens (only owner can mint)
        #[ink(message)]
        pub fn mint(&mut self, to: AccountId, value: Balance) -> Result<()> {
            // Access control: only owner can mint
            let caller = self.env().caller();
            if caller != self.owner {
                return Err(Error::InsufficientBalance); // Reuse error for simplicity
            }

            let balance = self.balance_of(to);
            self.balances.insert(to, &(balance.checked_add(value)
                .ok_or(Error::Overflow)?));

            self.total_supply = self.total_supply.checked_add(value)
                .ok_or(Error::Overflow)?;

            self.env().emit_event(Transfer {
                from: None,
                to: Some(to),
                value,
            });

            Ok(())
        }

        /// Get contract owner
        #[ink(message)]
        pub fn owner(&self) -> AccountId {
            self.owner
        }

        /// Burn tokens
        #[ink(message)]
        pub fn burn(&mut self, value: Balance) -> Result<()> {
            let caller = self.env().caller();
            let balance = self.balance_of(caller);

            if balance < value {
                return Err(Error::InsufficientBalance);
            }

            self.balances.insert(caller, &(balance - value));
            self.total_supply -= value;

            self.env().emit_event(Transfer {
                from: Some(caller),
                to: None,
                value,
            });

            Ok(())
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn new_works() {
            let token = BoundlessToken::new(
                "Boundless Token".to_string(),
                "BLS".to_string(),
                18,
                1_000_000,
            );

            assert_eq!(token.total_supply(), 1_000_000);
            assert_eq!(token.name(), "Boundless Token");
            assert_eq!(token.symbol(), "BLS");
            assert_eq!(token.decimals(), 18);
        }

        #[ink::test]
        fn transfer_works() {
            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
            let mut token = BoundlessToken::new(
                "Test".to_string(),
                "TST".to_string(),
                18,
                1000,
            );

            assert_eq!(token.balance_of(accounts.alice), 1000);
            assert!(token.transfer(accounts.bob, 500).is_ok());
            assert_eq!(token.balance_of(accounts.alice), 500);
            assert_eq!(token.balance_of(accounts.bob), 500);
        }

        #[ink::test]
        fn transfer_insufficient_balance_fails() {
            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
            let mut token = BoundlessToken::new(
                "Test".to_string(),
                "TST".to_string(),
                18,
                100,
            );

            let result = token.transfer(accounts.bob, 200);
            assert_eq!(result, Err(Error::InsufficientBalance));
        }

        #[ink::test]
        fn approve_and_transfer_from_works() {
            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
            let mut token = BoundlessToken::new(
                "Test".to_string(),
                "TST".to_string(),
                18,
                1000,
            );

            // Alice approves Bob to spend 200 tokens
            assert!(token.approve(accounts.bob, 200).is_ok());
            assert_eq!(token.allowance(accounts.alice, accounts.bob), 200);

            // Bob transfers from Alice to Charlie
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
            assert!(token.transfer_from(accounts.alice, accounts.charlie, 100).is_ok());

            assert_eq!(token.balance_of(accounts.alice), 900);
            assert_eq!(token.balance_of(accounts.charlie), 100);
            assert_eq!(token.allowance(accounts.alice, accounts.bob), 100);
        }

        #[ink::test]
        fn mint_works() {
            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
            let mut token = BoundlessToken::new(
                "Test".to_string(),
                "TST".to_string(),
                18,
                1000,
            );

            assert!(token.mint(accounts.bob, 500).is_ok());
            assert_eq!(token.total_supply(), 1500);
            assert_eq!(token.balance_of(accounts.bob), 500);
        }

        #[ink::test]
        fn burn_works() {
            let mut token = BoundlessToken::new(
                "Test".to_string(),
                "TST".to_string(),
                18,
                1000,
            );

            assert!(token.burn(300).is_ok());
            assert_eq!(token.total_supply(), 700);
        }
    }
}
