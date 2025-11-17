// Multi-Asset Support for Enterprise Integration
//
// Enables the blockchain to support multiple asset types beyond the native BLS token
// including equity tokens, utility tokens, carbon credits, and custom assets.

use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use std::collections::HashMap;

/// Asset types supported by the system
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AssetType {
    /// Native BLS blockchain token
    Native,
    /// Equity/ownership tokens
    Equity,
    /// Utility tokens for services
    Utility,
    /// Governance/voting tokens
    Governance,
    /// Carbon credits
    CarbonCredit,
    /// Reward points
    Reward,
    /// Stablecoin
    Stablecoin,
    /// Custom asset type
    Custom(String),
}

impl AssetType {
    pub fn as_str(&self) -> &str {
        match self {
            AssetType::Native => "native",
            AssetType::Equity => "equity",
            AssetType::Utility => "utility",
            AssetType::Governance => "governance",
            AssetType::CarbonCredit => "carbon_credit",
            AssetType::Reward => "reward",
            AssetType::Stablecoin => "stablecoin",
            AssetType::Custom(s) => s.as_str(),
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "native" => AssetType::Native,
            "equity" => AssetType::Equity,
            "utility" => AssetType::Utility,
            "governance" => AssetType::Governance,
            "carbon_credit" => AssetType::CarbonCredit,
            "reward" => AssetType::Reward,
            "stablecoin" => AssetType::Stablecoin,
            _ => AssetType::Custom(s.to_string()),
        }
    }
}

/// Asset definition
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetDefinition {
    /// Unique asset identifier
    pub asset_id: [u8; 32],

    /// Asset type
    pub asset_type: AssetType,

    /// Asset name (e.g., "Company Equity", "Carbon Credits")
    pub name: String,

    /// Asset symbol (e.g., "EQTY", "CARB")
    pub symbol: String,

    /// Number of decimal places (0-18)
    pub decimals: u8,

    /// Total supply (in smallest unit)
    pub total_supply: u64,

    /// Issuer identity (who created the asset)
    pub issuer: [u8; 32],

    /// Block height where asset was created
    pub created_at: u64,

    /// Asset metadata (JSON)
    pub metadata: Vec<u8>,

    /// Whether the asset is transferable
    pub transferable: bool,

    /// Whether the asset is burnable
    pub burnable: bool,

    /// Whether the asset is mintable (can increase supply)
    pub mintable: bool,
}

impl AssetDefinition {
    /// Create a new asset definition
    pub fn new(
        asset_type: AssetType,
        name: String,
        symbol: String,
        decimals: u8,
        total_supply: u64,
        issuer: [u8; 32],
        created_at: u64,
        metadata: Vec<u8>,
        transferable: bool,
        burnable: bool,
        mintable: bool,
    ) -> Self {
        // Generate asset ID from issuer + symbol + timestamp
        let mut hasher = Sha3_256::new();
        hasher.update(&issuer);
        hasher.update(symbol.as_bytes());
        hasher.update(&created_at.to_le_bytes());
        let asset_id = hasher.finalize().into();

        Self {
            asset_id,
            asset_type,
            name,
            symbol,
            decimals,
            total_supply,
            issuer,
            created_at,
            metadata,
            transferable,
            burnable,
            mintable,
        }
    }

    /// Get asset ID as hex string
    pub fn asset_id_hex(&self) -> String {
        hex::encode(self.asset_id)
    }

    /// Validate asset definition
    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Asset name cannot be empty".to_string());
        }

        if self.symbol.is_empty() {
            return Err("Asset symbol cannot be empty".to_string());
        }

        if self.decimals > 18 {
            return Err("Decimals cannot exceed 18".to_string());
        }

        if self.total_supply == 0 {
            return Err("Total supply must be greater than 0".to_string());
        }

        Ok(())
    }
}

/// Asset balance for an account
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetBalance {
    /// Asset identifier
    pub asset_id: [u8; 32],

    /// Balance amount
    pub balance: u64,
}

/// Multi-asset storage and management
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AssetRegistry {
    /// Map of asset_id -> AssetDefinition
    assets: HashMap<[u8; 32], AssetDefinition>,

    /// Map of (account_address, asset_id) -> balance
    balances: HashMap<([u8; 32], [u8; 32]), u64>,

    /// Map of issuer -> list of asset_ids
    issuer_assets: HashMap<[u8; 32], Vec<[u8; 32]>>,
}

impl AssetRegistry {
    /// Create new asset registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a new asset
    pub fn register_asset(&mut self, asset: AssetDefinition) -> Result<(), String> {
        // Validate asset
        asset.validate()?;

        // Check if asset already exists
        if self.assets.contains_key(&asset.asset_id) {
            return Err("Asset already registered".to_string());
        }

        // Store asset
        let asset_id = asset.asset_id;
        let issuer = asset.issuer;
        let total_supply = asset.total_supply;

        self.assets.insert(asset_id, asset);

        // Add to issuer index
        self.issuer_assets
            .entry(issuer)
            .or_insert_with(Vec::new)
            .push(asset_id);

        // Mint initial supply to issuer
        self.balances.insert((issuer, asset_id), total_supply);

        Ok(())
    }

    /// Get asset definition
    pub fn get_asset(&self, asset_id: &[u8; 32]) -> Option<&AssetDefinition> {
        self.assets.get(asset_id)
    }

    /// Get all assets issued by an account
    pub fn get_issuer_assets(&self, issuer: &[u8; 32]) -> Vec<&AssetDefinition> {
        self.issuer_assets
            .get(issuer)
            .map(|asset_ids| {
                asset_ids
                    .iter()
                    .filter_map(|id| self.assets.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get balance of an asset for an account
    pub fn get_balance(&self, account: &[u8; 32], asset_id: &[u8; 32]) -> u64 {
        *self.balances.get(&(*account, *asset_id)).unwrap_or(&0)
    }

    /// Get all asset balances for an account
    pub fn get_account_balances(&self, account: &[u8; 32]) -> Vec<AssetBalance> {
        self.balances
            .iter()
            .filter_map(|((acc, asset_id), balance)| {
                if acc == account && *balance > 0 {
                    Some(AssetBalance {
                        asset_id: *asset_id,
                        balance: *balance,
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    /// Transfer asset from one account to another
    pub fn transfer(
        &mut self,
        from: &[u8; 32],
        to: &[u8; 32],
        asset_id: &[u8; 32],
        amount: u64,
    ) -> Result<(), String> {
        // Check if asset exists
        let asset = self
            .get_asset(asset_id)
            .ok_or_else(|| "Asset not found".to_string())?;

        // Check if asset is transferable
        if !asset.transferable {
            return Err("Asset is not transferable".to_string());
        }

        // Check sender balance
        let sender_balance = self.get_balance(from, asset_id);
        if sender_balance < amount {
            return Err("Insufficient balance".to_string());
        }

        // Perform transfer
        let new_sender_balance = sender_balance - amount;
        let receiver_balance = self.get_balance(to, asset_id);
        let new_receiver_balance = receiver_balance
            .checked_add(amount)
            .ok_or_else(|| "Balance overflow".to_string())?;

        // Update balances
        if new_sender_balance == 0 {
            self.balances.remove(&(*from, *asset_id));
        } else {
            self.balances.insert((*from, *asset_id), new_sender_balance);
        }
        self.balances.insert((*to, *asset_id), new_receiver_balance);

        Ok(())
    }

    /// Burn (destroy) asset tokens
    pub fn burn(
        &mut self,
        from: &[u8; 32],
        asset_id: &[u8; 32],
        amount: u64,
    ) -> Result<(), String> {
        // Check if asset exists and is burnable
        let asset = self
            .get_asset(asset_id)
            .ok_or_else(|| "Asset not found".to_string())?;

        if !asset.burnable {
            return Err("Asset is not burnable".to_string());
        }

        // Check balance
        let balance = self.get_balance(from, asset_id);
        if balance < amount {
            return Err("Insufficient balance".to_string());
        }

        // Burn tokens
        let new_balance = balance - amount;
        if new_balance == 0 {
            self.balances.remove(&(*from, *asset_id));
        } else {
            self.balances.insert((*from, *asset_id), new_balance);
        }

        // Update total supply in asset definition
        if let Some(asset_def) = self.assets.get_mut(asset_id) {
            asset_def.total_supply = asset_def.total_supply.saturating_sub(amount);
        }

        Ok(())
    }

    /// Mint (create) new asset tokens
    pub fn mint(&mut self, to: &[u8; 32], asset_id: &[u8; 32], amount: u64) -> Result<(), String> {
        // Check if asset exists and is mintable
        let asset = self
            .get_asset(asset_id)
            .ok_or_else(|| "Asset not found".to_string())?;

        if !asset.mintable {
            return Err("Asset is not mintable".to_string());
        }

        // Get current balance
        let balance = self.get_balance(to, asset_id);
        let new_balance = balance
            .checked_add(amount)
            .ok_or_else(|| "Balance overflow".to_string())?;

        // Mint tokens
        self.balances.insert((*to, *asset_id), new_balance);

        // Update total supply in asset definition
        if let Some(asset_def) = self.assets.get_mut(asset_id) {
            asset_def.total_supply = asset_def
                .total_supply
                .checked_add(amount)
                .ok_or_else(|| "Total supply overflow".to_string())?;
        }

        Ok(())
    }

    /// Get total number of registered assets
    pub fn total_assets(&self) -> usize {
        self.assets.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_definition() {
        let issuer = [1u8; 32];
        let asset = AssetDefinition::new(
            AssetType::Equity,
            "Company Equity".to_string(),
            "EQTY".to_string(),
            2,
            1_000_000,
            issuer,
            100,
            vec![],
            true,
            false,
            false,
        );

        assert_eq!(asset.name, "Company Equity");
        assert_eq!(asset.symbol, "EQTY");
        assert_eq!(asset.decimals, 2);
        assert_eq!(asset.total_supply, 1_000_000);
        assert!(asset.validate().is_ok());
    }

    #[test]
    fn test_asset_registry() {
        let mut registry = AssetRegistry::new();

        let issuer = [1u8; 32];
        let asset = AssetDefinition::new(
            AssetType::Utility,
            "Utility Token".to_string(),
            "UTIL".to_string(),
            18,
            1_000_000,
            issuer,
            100,
            vec![],
            true,
            true,
            false,
        );

        let asset_id = asset.asset_id;

        // Register asset
        assert!(registry.register_asset(asset).is_ok());

        // Check issuer balance (should have total supply)
        assert_eq!(registry.get_balance(&issuer, &asset_id), 1_000_000);

        // Transfer some tokens
        let recipient = [2u8; 32];
        assert!(registry
            .transfer(&issuer, &recipient, &asset_id, 100_000)
            .is_ok());

        assert_eq!(registry.get_balance(&issuer, &asset_id), 900_000);
        assert_eq!(registry.get_balance(&recipient, &asset_id), 100_000);
    }

    #[test]
    fn test_non_transferable_asset() {
        let mut registry = AssetRegistry::new();

        let issuer = [1u8; 32];
        let asset = AssetDefinition::new(
            AssetType::Governance,
            "Gov Token".to_string(),
            "GOV".to_string(),
            0,
            1000,
            issuer,
            100,
            vec![],
            false, // Not transferable
            false,
            false,
        );

        let asset_id = asset.asset_id;
        assert!(registry.register_asset(asset).is_ok());

        // Try to transfer
        let recipient = [2u8; 32];
        assert!(registry
            .transfer(&issuer, &recipient, &asset_id, 100)
            .is_err());
    }

    #[test]
    fn test_burn_tokens() {
        let mut registry = AssetRegistry::new();

        let issuer = [1u8; 32];
        let asset = AssetDefinition::new(
            AssetType::Utility,
            "Burnable Token".to_string(),
            "BURN".to_string(),
            0,
            1000,
            issuer,
            100,
            vec![],
            true,
            true, // Burnable
            false,
        );

        let asset_id = asset.asset_id;
        assert!(registry.register_asset(asset).is_ok());

        // Burn tokens
        assert!(registry.burn(&issuer, &asset_id, 500).is_ok());
        assert_eq!(registry.get_balance(&issuer, &asset_id), 500);
    }

    #[test]
    fn test_asset_types() {
        assert_eq!(AssetType::Native.as_str(), "native");
        assert_eq!(AssetType::Equity.as_str(), "equity");
        assert_eq!(AssetType::CarbonCredit.as_str(), "carbon_credit");

        assert_eq!(AssetType::from_str("native"), AssetType::Native);
        assert_eq!(AssetType::from_str("equity"), AssetType::Equity);

        match AssetType::from_str("custom_type") {
            AssetType::Custom(s) => assert_eq!(s, "custom_type"),
            _ => panic!("Expected custom asset type"),
        }
    }
}
