// Enterprise Multipass - Core Data Models
// All models use Boundless primitives for cryptography and chain interactions

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================================
// 1. IDENTITY & ATTESTATION SERVICE
// ============================================================================

/// Core identity profile with KYC/AML status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityProfile {
    pub identity_id: Uuid,
    pub full_name: String,
    pub email: String,
    pub phone: Option<String>,
    pub country_code: Option<String>,
    pub date_of_birth: Option<NaiveDate>,
    pub verification_status: String,
    pub kyc_level: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Identity attestation (proof of KYC, address, assets, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityAttestation {
    pub attestation_id: Uuid,
    pub identity_id: Uuid,
    pub attestation_type: AttestationType,
    /// References to evidence (document hashes, external IDs)
    pub evidence_refs: Vec<String>,
    pub issuer: String,
    pub status: AttestationStatus,
    pub valid_from: DateTime<Utc>,
    pub valid_to: Option<DateTime<Utc>>,
    /// Boundless chain transaction hash anchoring this attestation
    pub chain_anchor_tx: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "attestation_type", rename_all = "snake_case")]
pub enum AttestationType {
    KycVerified,
    AccreditedInvestor,
    Employment,
    Education,
    Kyc,
    AddressProof,
    IncomeProof,
    AssetOwnership,
    SocialGraph,
    ProfessionalCredential,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "attestation_status", rename_all = "lowercase")]
pub enum AttestationStatus {
    Valid,
    Expired,
    Revoked,
}

// ============================================================================
// 2. WALLET SERVICE
// ============================================================================

/// Enterprise wallet account with multi-asset support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletAccount {
    pub wallet_id: Uuid,
    pub identity_id: Uuid,
    /// Boundless PQC addresses (can have multiple for different purposes)
    pub boundless_addresses: Vec<BoundlessAddress>,
    pub labels: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundlessAddress {
    /// The actual Boundless blockchain address (32-byte pubkey hash)
    pub address: String,
    pub derivation_path: String,
    pub public_key: String,
    /// Derivation path or metadata
    pub metadata: Option<String>,
    pub label: Option<String>,
}

/// Wallet balance for a specific asset type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletBalance {
    pub balance_id: Uuid,
    pub wallet_id: Uuid,
    pub asset_type: AssetType,
    pub total_amount: i64,
    pub locked_amount: i64,
    pub unlocked_amount: i64,
    pub last_sync_at: DateTime<Utc>,
}

/// Transaction record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletTransaction {
    pub tx_id: Uuid,
    pub wallet_id: Uuid,
    /// Boundless chain transaction hash
    pub chain_tx_hash: String,
    pub asset_type: AssetType,
    pub amount: i64,
    pub direction: TxDirection,
    pub to_address: Option<String>,
    pub from_address: Option<String>,
    pub status: TxStatus,
    pub block_height: Option<i64>,
    pub confirmations: i32,
    pub fee: i64,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "tx_direction", rename_all = "lowercase")]
pub enum TxDirection {
    Incoming,
    Outgoing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "tx_status", rename_all = "lowercase")]
pub enum TxStatus {
    Pending,
    Confirmed,
    Failed,
}

// ============================================================================
// 3. MULTIPASS AUTH/SSO SERVICE
// ============================================================================

/// Multipass credential (login credentials)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultipassCredential {
    pub credential_id: Uuid,
    pub identity_id: Uuid,
    pub username: String,
    pub password_hash: String,
    pub totp_secret: Option<String>,
    pub backup_codes: Vec<String>,
    pub require_2fa: bool,
    pub locked: bool,
    pub failed_attempts: i32,
    pub last_login_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Active session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultipassSession {
    pub session_id: Uuid,
    pub identity_id: Uuid,
    pub token_hash: String,
    pub scopes: Vec<String>,
    pub device_info: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub revoked: bool,
}

// ============================================================================
// 4. APPLICATION MODULE REGISTRY
// ============================================================================

/// Registered application module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationModule {
    pub app_id: Uuid,
    pub name: String,
    pub description: String,
    pub category: AppCategory,
    pub api_base_url: String,
    /// Required scopes/permissions
    pub required_scopes: Vec<String>,
    /// Boundless smart contract reference (if applicable)
    pub on_chain_contract_ref: Option<String>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "app_category", rename_all = "snake_case")]
pub enum AppCategory {
    Security,
    Invoicing,
    Ticketing,
    Healthcare,
    SupplyChain,
    Compliance,
    Finance,
    Marketplace,
}

/// Application event log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationEvent {
    pub event_id: Uuid,
    pub app_id: Uuid,
    pub identity_id: Uuid,
    pub event_type: String,
    pub event_data: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

// ============================================================================
// 5. ASSET & MARKET SERVICE
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "asset_type", rename_all = "snake_case")]
pub enum AssetType {
    /// Native Boundless coin
    Native,
    /// Utility token
    UtilityToken,
    /// Equity token (private)
    EquityToken,
    /// Carbon credit / ESG token
    CarbonCredit,
    /// NFT
    Nft,
    /// Subscription pass
    SubscriptionPass,
}

impl AssetType {
    /// FIX H-4: Safe conversion to database string (prevents SQL injection)
    /// Use this instead of format!("{:?}", asset_type) for database queries
    pub fn to_db_string(&self) -> &'static str {
        match self {
            AssetType::Native => "Native",
            AssetType::UtilityToken => "UtilityToken",
            AssetType::EquityToken => "EquityToken",
            AssetType::CarbonCredit => "CarbonCredit",
            AssetType::Nft => "Nft",
            AssetType::SubscriptionPass => "SubscriptionPass",
        }
    }
}

/// Asset definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetDefinition {
    pub asset_id: Uuid,
    pub asset_type: AssetType,
    pub symbol: String,
    pub name: String,
    pub total_supply: i64,
    pub circulating_supply: i64,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

/// Asset position (wallet's holding of a specific asset)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetPosition {
    pub position_id: Uuid,
    pub wallet_id: Uuid,
    pub asset_id: Uuid,
    pub quantity: i64,
    pub average_cost: i64,
}

/// Asset balance - internal asset holdings for enterprise-issued assets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetBalance {
    pub balance_id: Uuid,
    pub wallet_id: Uuid,
    pub asset_id: Uuid,
    pub total_quantity: i64,
    pub locked_quantity: i64,
}

/// Market order for internal B2B/P2P trading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketOrder {
    pub order_id: Uuid,
    pub wallet_id: Uuid,
    pub asset_id: Uuid,
    pub order_type: OrderType,
    pub quantity: i64,
    pub price: i64,
    pub filled_quantity: i64,
    pub status: OrderStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Position - aggregated holdings with average cost
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub position_id: Uuid,
    pub wallet_id: Uuid,
    pub asset_id: Uuid,
    pub quantity: i64,
    pub average_cost: i64,
}

/// Trade - executed trade history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub trade_id: Uuid,
    pub asset_id: Uuid,
    pub buy_order_id: Uuid,
    pub sell_order_id: Uuid,
    pub quantity: i64,
    pub price: i64,
    pub buyer_wallet_id: Uuid,
    pub seller_wallet_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "order_type", rename_all = "lowercase")]
pub enum OrderType {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "order_status", rename_all = "lowercase")]
pub enum OrderStatus {
    Open,
    PartiallyFilled,
    Filled,
    Cancelled,
}

/// Order book for an asset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBook {
    pub asset_id: Uuid,
    pub bids: Vec<(i64, i64)>,  // (price, quantity)
    pub asks: Vec<(i64, i64)>,  // (price, quantity)
}

// ============================================================================
// 6. EVENT & REPORTING SERVICE
// ============================================================================

/// Notification to user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub notification_id: Uuid,
    pub identity_id: Uuid,
    pub notification_type: NotificationType,
    pub title: String,
    pub message: String,
    pub read: bool,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "notification_type", rename_all = "snake_case")]
pub enum NotificationType {
    Info,
    SecurityAlert,
    PaymentReceived,
    PaymentSent,
    PermissionChange,
    ReportReady,
    SystemUpdate,
    ApplicationEvent,
}

/// Report definition (template)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportDefinition {
    pub report_id: Uuid,
    pub name: String,
    pub description: String,
    pub report_type: ReportType,
    pub sql_template: String,
    pub parameters: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "report_type", rename_all = "snake_case")]
pub enum ReportType {
    Transaction,
    Financial,
    Asset,
    Application,
    Security,
    Compliance,
}

/// Generated report instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportInstance {
    pub report_id: Uuid,
    pub report_def_id: Uuid,
    pub identity_id: Uuid,
    pub parameters: serde_json::Value,
    pub generated_at: DateTime<Utc>,
    /// Report data (JSON or reference to file)
    pub data: serde_json::Value,
}

/// Generated report - stored report results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedReport {
    pub generated_report_id: Uuid,
    pub report_id: Uuid,
    pub identity_id: Uuid,
    pub parameters: serde_json::Value,
    pub format: ExportFormat,
    pub result_data: String,
    pub chain_anchor_tx: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportFormat {
    JSON,
    CSV,
    PDF,
}

// ============================================================================
// 7. HARDWAREPASS SERVICE (Optional)
// ============================================================================

/// Physical hardware device (NFC card, secure element)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwarePass {
    pub device_id: Uuid,
    pub identity_id: Uuid,
    pub device_type: String,
    /// Public key or secure element identifier
    pub public_key: String,
    pub capabilities: Vec<String>, // Changed from Option<Vec<String>> - will unwrap in service layer
    pub status: HardwareStatus,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "hardware_capability", rename_all = "snake_case")]
pub enum HardwareCapability {
    Nfc,
    Biometric,
    SecureElement,
    Display,
    LoginOnly,
    SignTx,
    UnlockDoors,
    AccessControl,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "hardware_status", rename_all = "lowercase")]
pub enum HardwareStatus {
    Active,
    Lost,
    Revoked,
}
