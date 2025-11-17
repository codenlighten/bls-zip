// Account structure for state management
use serde::{Deserialize, Serialize};

/// Account state in the blockchain
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Account {
    /// Account balance
    pub balance: u64,

    /// Transaction nonce (to prevent replay attacks)
    pub nonce: u64,

    /// Code hash (for smart contract accounts)
    pub code_hash: Option<[u8; 32]>,

    /// Storage root (Merkle root of contract storage)
    pub storage_root: Option<[u8; 32]>,
}

impl Account {
    /// Create a new empty account
    pub fn new() -> Self {
        Self {
            balance: 0,
            nonce: 0,
            code_hash: None,
            storage_root: None,
        }
    }

    /// Create an account with initial balance
    pub fn with_balance(balance: u64) -> Self {
        Self {
            balance,
            nonce: 0,
            code_hash: None,
            storage_root: None,
        }
    }

    /// Check if this is a contract account
    pub fn is_contract(&self) -> bool {
        self.code_hash.is_some()
    }

    /// Increment the nonce
    pub fn increment_nonce(&mut self) {
        self.nonce = self.nonce.saturating_add(1);
    }

    /// Add to balance
    pub fn add_balance(&mut self, amount: u64) -> Result<(), String> {
        self.balance = self.balance.checked_add(amount).ok_or("Balance overflow")?;
        Ok(())
    }

    /// Subtract from balance
    pub fn sub_balance(&mut self, amount: u64) -> Result<(), String> {
        self.balance = self
            .balance
            .checked_sub(amount)
            .ok_or("Insufficient balance")?;
        Ok(())
    }
}

impl Default for Account {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_creation() {
        let account = Account::new();
        assert_eq!(account.balance, 0);
        assert_eq!(account.nonce, 0);
        assert!(!account.is_contract());
    }

    #[test]
    fn test_account_balance_operations() {
        let mut account = Account::with_balance(1000);

        assert!(account.add_balance(500).is_ok());
        assert_eq!(account.balance, 1500);

        assert!(account.sub_balance(200).is_ok());
        assert_eq!(account.balance, 1300);

        assert!(account.sub_balance(2000).is_err()); // Insufficient balance
    }

    #[test]
    fn test_nonce_increment() {
        let mut account = Account::new();
        assert_eq!(account.nonce, 0);

        account.increment_nonce();
        assert_eq!(account.nonce, 1);

        account.increment_nonce();
        assert_eq!(account.nonce, 2);
    }
}
