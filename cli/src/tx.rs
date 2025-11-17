use anyhow::{bail, Result};
use boundless_core::{Signature, Transaction, TxInput, TxOutput};
use ed25519_dalek::{Signer, SigningKey};
use jsonrpsee::core::client::ClientT;
use jsonrpsee::http_client::HttpClient;
use jsonrpsee::rpc_params;
use serde_json::Value;
use sha3::{Digest, Sha3_256};
use std::fs;
use std::path::Path;

pub async fn send_transaction(
    client: &HttpClient,
    to: &str,
    amount: u64,
    key_file: &Path,
) -> Result<()> {
    println!("💸 Preparing transaction...");
    println!("  Recipient: {}", to);
    println!(
        "  Amount: {} BLS ({} base units)",
        amount as f64 / 1e8,
        amount
    );

    // Load private key (32-byte Ed25519 seed)
    let priv_key_hex = fs::read_to_string(key_file)?;
    let priv_key_bytes = hex::decode(priv_key_hex.trim())?;

    if priv_key_bytes.len() != 32 {
        bail!(
            "Invalid private key: expected 32 bytes, got {}",
            priv_key_bytes.len()
        );
    }

    let key_array: [u8; 32] = priv_key_bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("Failed to convert private key to array"))?;
    let signing_key = SigningKey::from_bytes(&key_array);
    let verifying_key = signing_key.verifying_key();
    let public_key = verifying_key.to_bytes().to_vec();

    // Calculate sender address (SHA3-256 hash of public key)
    let mut hasher = Sha3_256::new();
    hasher.update(&public_key);
    let sender_address: [u8; 32] = hasher.finalize().into();

    println!("  From: {}", hex::encode(sender_address));

    // Parse recipient address
    let recipient_bytes = hex::decode(to)?;
    if recipient_bytes.len() != 32 {
        bail!("Invalid recipient address: expected 32 bytes hex");
    }
    let recipient_address: [u8; 32] = recipient_bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("Failed to convert recipient address to array"))?;

    // NOTE: This is a simplified implementation
    // In production, you would:
    // 1. Query chain_getBalance to check available funds
    // 2. Query for UTXOs (requires additional RPC method)
    // 3. Select appropriate UTXOs to spend
    //
    // For now, this creates a placeholder transaction structure
    // that demonstrates the signing flow. Full UTXO handling requires
    // additional RPC endpoints for UTXO enumeration.

    println!(
        "  ⚠️  Note: Simplified transaction creation (UTXO tracking not yet implemented in RPC)"
    );
    println!("  ⚠️  This will fail at submission without proper UTXOs");

    // Create transaction output
    let output = TxOutput {
        amount,
        recipient_pubkey_hash: recipient_address,
        script: None,
    };

    // Create a placeholder input (in production, this would reference actual UTXOs)
    let placeholder_input = TxInput {
        previous_output_hash: [0u8; 32], // Would be actual UTXO tx hash
        output_index: 0,
        signature: Signature::Classical(vec![]), // Filled after signing
        public_key: public_key.clone(),
        nonce: None,
    };

    // Create unsigned transaction
    let mut tx = Transaction::new(
        1, // version
        vec![placeholder_input],
        vec![output],
        chrono::Utc::now().timestamp() as u64,
        None,
    );

    // Sign transaction
    println!("  🔐 Signing transaction...");
    let signing_hash = tx.signing_hash();
    let signature = signing_key.sign(&signing_hash);

    // Update transaction with signature
    tx.inputs[0].signature = Signature::Classical(signature.to_bytes().to_vec());

    println!("  📦 Transaction created:");
    println!("     TX Hash: {}", hex::encode(tx.hash()));
    println!("     Size: {} bytes", tx.size_bytes());

    // Serialize transaction
    let tx_bytes = bincode::serialize(&tx)?;
    let tx_hex = hex::encode(&tx_bytes);

    // Submit to RPC
    println!("  📡 Submitting to network...");
    let response: Value = client
        .request("chain_submitTransaction", rpc_params![tx_hex])
        .await?;

    if response["success"].as_bool().unwrap_or(false) {
        println!("  ✅ Transaction submitted successfully!");
        println!(
            "     TX Hash: {}",
            response["tx_hash"].as_str().unwrap_or("unknown")
        );
    } else {
        let error_msg = response["message"].as_str().unwrap_or("Unknown error");
        bail!("Transaction submission failed: {}", error_msg);
    }

    Ok(())
}
