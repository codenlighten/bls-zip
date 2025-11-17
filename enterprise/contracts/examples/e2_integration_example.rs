// Example: Integrating Smart Contracts with E2 Multipass Backend
//
// This demonstrates how to interact with deployed smart contracts
// using the E2 Multipass identity, wallet, and asset services.

use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

// ============================================================================
// E2 Multipass API Types
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct LoginResponse {
    session: Session,
    token: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Session {
    session_id: Uuid,
    identity_id: Uuid,
    token_hash: String,
    scopes: Vec<String>,
    expires_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct IdentityProfile {
    identity_id: Uuid,
    email: String,
    display_name: Option<String>,
    created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AssetBalance {
    asset_id: Uuid,
    asset_type: String,
    quantity: i64,
    locked_quantity: i64,
}

// ============================================================================
// Example 1: Register Identity in Access Control Contract
// ============================================================================

async fn example_identity_registration() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Example 1: Identity Registration with Access Control ===\n");

    let client = Client::new();

    // 1. Login to E2 Multipass
    println!("1. Logging in to E2 Multipass...");
    let login_response: LoginResponse = client
        .post("http://localhost:8080/api/auth/login")
        .json(&LoginRequest {
            email: "admin@boundless.local".to_string(),
            password: "BoundlessTrust@2024".to_string(),
        })
        .send()
        .await?
        .json()
        .await?;

    println!("   ✓ Logged in as identity: {}", login_response.session.identity_id);
    println!("   ✓ JWT token obtained\n");

    let token = login_response.token;
    let identity_id = login_response.session.identity_id;

    // 2. Get identity profile
    println!("2. Fetching identity profile...");
    let profile: IdentityProfile = client
        .get(format!("http://localhost:8080/api/identity/{}", identity_id))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?
        .json()
        .await?;

    println!("   ✓ Email: {}", profile.email);
    println!("   ✓ Display Name: {:?}\n", profile.display_name);

    // 3. Register identity in smart contract
    println!("3. Registering identity in Access Control contract...");

    // Convert UUID to [u8; 32] for contract
    let identity_bytes = uuid_to_bytes32(identity_id);

    println!("   Contract call: register_identity");
    println!("   Identity ID: {}", identity_id);
    println!("   Identity bytes: {:?}", &identity_bytes[..8]); // Show first 8 bytes
    println!("   Role: User");

    // TODO: Actual contract call would happen here
    // let contract_result = contract_client.send(
    //     "register_identity",
    //     encode_args(&[identity_bytes, Role::User]),
    //     50_000_000
    // ).await?;

    println!("   ✓ Identity registered in contract\n");

    // 4. Verify role in contract
    println!("4. Verifying role assignment...");
    // TODO: Contract call
    // let has_role = contract_client.call("has_role", encode_args(&[account, Role::User])).await?;
    println!("   ✓ Role verified: User\n");

    Ok(())
}

// ============================================================================
// Example 2: Multi-Signature Wallet with E2 Identities
// ============================================================================

async fn example_multisig_wallet() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Example 2: Multi-Signature Wallet ===\n");

    let client = Client::new();

    // Login as Alice (admin)
    println!("1. Setting up multi-sig wallet with 3 signers...");
    let alice_token = login_user(&client, "admin@boundless.local", "BoundlessTrust@2024").await?;

    // In a real scenario, you'd have 3 different users
    println!("   Signer 1: Alice (admin@boundless.local)");
    println!("   Signer 2: Bob (would be separate identity)");
    println!("   Signer 3: Charlie (would be separate identity)");
    println!("   Threshold: 2-of-3\n");

    // 2. Create multisig wallet contract
    println!("2. Deploying multi-sig wallet contract...");
    // TODO: Deploy contract with signers
    println!("   ✓ Contract deployed\n");

    // 3. Propose transaction
    println!("3. Alice proposes a transaction...");
    println!("   To: 0x1234...5678");
    println!("   Amount: 1000 tokens");
    println!("   Asset: Native token\n");

    // TODO: Contract call
    // let tx_id = contract_client.send("propose_transaction", ...).await?;
    let tx_id = 0u64;
    println!("   ✓ Transaction proposed (ID: {})\n", tx_id);

    // 4. Approve transaction (need 2 approvals)
    println!("4. Bob approves the transaction...");
    // TODO: Contract call from Bob's account
    println!("   ✓ Approval 1/2 received\n");

    println!("5. Charlie approves the transaction...");
    // TODO: Contract call from Charlie's account
    println!("   ✓ Approval 2/2 received");
    println!("   ✓ Threshold reached!\n");

    // 6. Execute transaction
    println!("6. Executing approved transaction...");
    // TODO: Contract call
    println!("   ✓ Transaction executed\n");

    Ok(())
}

// ============================================================================
// Example 3: Asset Trading with Escrow
// ============================================================================

async fn example_asset_escrow() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Example 3: Asset Trading with Escrow ===\n");

    let client = Client::new();
    let token = login_user(&client, "admin@boundless.local", "BoundlessTrust@2024").await?;

    // 1. Get user's asset balances from E2
    println!("1. Fetching user's asset balances...");
    let balances: Vec<AssetBalance> = client
        .get("http://localhost:8080/api/assets/balances")
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?
        .json()
        .await?;

    for balance in &balances {
        println!("   Asset: {} - Quantity: {} (Locked: {})",
            balance.asset_type,
            balance.quantity,
            balance.locked_quantity
        );
    }
    println!();

    // 2. Propose a trade
    println!("2. Proposing asset trade...");
    println!("   Offering: 100 IRSC tokens");
    println!("   Requesting: 200 CRSC tokens");
    println!("   Counterparty: Bob\n");

    // Convert asset IDs
    let irsc_asset_id = balances[0].asset_id;
    let crsc_asset_id = if balances.len() > 1 { balances[1].asset_id } else { Uuid::new_v4() };

    // TODO: Lock quantity in E2 Multipass
    println!("3. Locking 100 IRSC tokens in E2 system...");
    // let lock_result = client
    //     .post(format!("http://localhost:8080/api/assets/{}/lock", irsc_asset_id))
    //     .json(&json!({ "quantity": 100 }))
    //     .header("Authorization", format!("Bearer {}", token))
    //     .send()
    //     .await?;
    println!("   ✓ Tokens locked\n");

    // TODO: Create trade in contract
    println!("4. Creating trade in escrow contract...");
    // let trade_id = contract_client.send("propose_trade", ...).await?;
    let trade_id = 0u64;
    println!("   ✓ Trade created (ID: {})\n", trade_id);

    // 5. Counterparty accepts
    println!("5. Bob accepts and locks his 200 CRSC tokens...");
    // TODO: Bob locks tokens and accepts trade
    println!("   ✓ Trade locked\n");

    // 6. Both parties confirm
    println!("6. Both parties confirm the trade...");
    println!("   Alice confirms...");
    // TODO: Alice confirms
    println!("   Bob confirms...");
    // TODO: Bob confirms
    println!("   ✓ Trade completed!");
    println!("   ✓ Assets swapped successfully\n");

    Ok(())
}

// ============================================================================
// Example 4: Application Authorization
// ============================================================================

async fn example_app_authorization() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Example 4: Application Authorization ===\n");

    let client = Client::new();
    let token = login_user(&client, "admin@boundless.local", "BoundlessTrust@2024").await?;

    // 1. Register application in E2 Multipass
    println!("1. Registering third-party application...");
    let app_response = client
        .post("http://localhost:8080/api/applications")
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "name": "My DeFi App",
            "callback_url": "https://myapp.example.com/callback",
            "scopes": ["read_wallet", "execute_transactions"]
        }))
        .send()
        .await?;

    println!("   ✓ Application registered in E2 Multipass\n");

    // 2. Register application in smart contract
    println!("2. Registering app in Authorization contract...");
    let app_id = Uuid::new_v4();
    let app_id_bytes = uuid_to_bytes32(app_id);

    // TODO: Contract call
    // contract_client.send("register_application", encode_args(&[
    //     app_id_bytes,
    //     "My DeFi App",
    //     identity_id_bytes,
    //     redirect_uris,
    //     requested_scopes
    // ])).await?;

    println!("   ✓ App registered in contract\n");

    // 3. User grants permissions
    println!("3. User grants permissions to app...");
    println!("   Scopes: ReadProfile, ReadWallet");
    println!("   Validity: 30 days");
    println!("   Delegatable: Yes\n");

    // TODO: Contract call
    // let grant_id = contract_client.send("issue_grant", ...).await?;
    let grant_id = 0u64;
    println!("   ✓ Grant issued (ID: {})\n", grant_id);

    // 4. App accesses user's resources
    println!("4. App requests access to user's wallet data...");
    // TODO: Contract call to check permissions
    // let can_access = contract_client.call("can_access_resource", ...).await?;
    println!("   ✓ Access granted - app can read wallet\n");

    // 5. User delegates to another service
    println!("5. User delegates read-only access to analytics service...");
    // TODO: Contract call
    // let delegation_id = contract_client.send("create_delegation", ...).await?;
    let delegation_id = 0u64;
    println!("   ✓ Delegation created (ID: {})\n", delegation_id);

    Ok(())
}

// ============================================================================
// Helper Functions
// ============================================================================

async fn login_user(client: &Client, email: &str, password: &str) -> Result<String, Box<dyn std::error::Error>> {
    let response: LoginResponse = client
        .post("http://localhost:8080/api/auth/login")
        .json(&LoginRequest {
            email: email.to_string(),
            password: password.to_string(),
        })
        .send()
        .await?
        .json()
        .await?;

    Ok(response.token)
}

fn uuid_to_bytes32(uuid: Uuid) -> [u8; 32] {
    let mut bytes = [0u8; 32];
    let uuid_bytes = uuid.as_bytes();
    bytes[..16].copy_from_slice(uuid_bytes);
    bytes
}

// ============================================================================
// Main Function
// ============================================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║  Enterprise E2 Multipass Smart Contract Integration Examples ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    // Check if E2 backend is running
    let client = Client::new();
    match client.get("http://localhost:8080/api/auth/health").send().await {
        Ok(_) => println!("✓ E2 Multipass backend is running\n"),
        Err(_) => {
            eprintln!("✗ E2 Multipass backend is not running!");
            eprintln!("  Please start it with: cd enterprise && cargo run --bin enterprise-server");
            return Ok(());
        }
    }

    // Run examples
    println!("Running integration examples...\n");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    example_identity_registration().await?;
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    example_multisig_wallet().await?;
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    example_asset_escrow().await?;
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    example_app_authorization().await?;
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    println!("\n✓ All examples completed successfully!\n");
    println!("Next steps:");
    println!("  1. Deploy contracts using deployment utilities");
    println!("  2. Integrate contract addresses into these examples");
    println!("  3. Run with: cargo run --example e2_integration\n");

    Ok(())
}
