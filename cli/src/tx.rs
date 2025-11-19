use anyhow::{bail, Result};
use boundless_core::{Signature, Transaction, TxInput, TxOutput};
use boundless_crypto::{Falcon512, MlDsa44};
use jsonrpsee::core::client::ClientT;
use jsonrpsee::http_client::HttpClient;
use jsonrpsee::rpc_params;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha3::{Digest, Sha3_256};
use std::fs;
use std::path::Path;

/// UTXO data from REST API
#[derive(Debug, Clone, Serialize, Deserialize)]
struct UtxoData {
    pub tx_hash: String,
    pub output_index: u32,
    pub amount: u64,
    pub block_height: u64,
    pub script: Option<String>,
}

/// Response from /api/v1/utxos/:address
#[derive(Debug, Deserialize)]
struct UtxoListResponse {
    pub utxos: Vec<UtxoData>,
}

pub async fn send_transaction(
    client: &HttpClient,
    rpc_url: &str,
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

    // Load private key (ML-DSA or Falcon)
    let priv_key_hex = fs::read_to_string(key_file)?;
    let priv_key_bytes = hex::decode(priv_key_hex.trim())?;

    // Load public key from .pub file
    let pub_key_file = key_file.with_extension("pub");
    let pub_key_hex = fs::read_to_string(&pub_key_file)
        .map_err(|e| anyhow::anyhow!("Failed to read public key file {}: {}", pub_key_file.display(), e))?;
    let public_key = hex::decode(pub_key_hex.trim())?;

    // Determine key type based on private key size
    // ML-DSA-44: 2528 bytes, Falcon-512: ~1281 bytes
    let signature_fn: Box<dyn Fn(&[u8]) -> Result<Signature>> = 
        if priv_key_bytes.len() == 2528 {
            // ML-DSA-44 key
            println!("  🔐 Using ML-DSA-44 signature");
            
            let secret_key = priv_key_bytes.clone();
            Box::new(move |message: &[u8]| -> Result<Signature> {
                let signer = MlDsa44::new()?;
                let sig_bytes = signer.sign(message, &secret_key)?;
                Ok(Signature::MlDsa(sig_bytes))
            })
        } else if priv_key_bytes.len() >= 1200 && priv_key_bytes.len() <= 1300 {
            // Falcon-512 key (approximately 1281 bytes)
            println!("  🔐 Using Falcon-512 signature");
            
            let secret_key = priv_key_bytes.clone();
            Box::new(move |message: &[u8]| -> Result<Signature> {
                let signer = Falcon512::new()?;
                let sig_bytes = signer.sign(message, &secret_key)?;
                Ok(Signature::Falcon(sig_bytes))
            })
        } else {
            bail!(
                "Invalid private key size: {} bytes. Expected ML-DSA (2528) or Falcon (~1281)",
                priv_key_bytes.len()
            );
        };

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

    // Query UTXOs from blockchain REST API
    println!("  🔍 Querying UTXOs...");
    let rest_url = rpc_url.replace(":9933", ":3001"); // Convert RPC port to REST port
    let utxo_url = format!("{}/api/v1/utxos/{}", rest_url, hex::encode(sender_address));

    let utxo_list: UtxoListResponse = ureq::get(&utxo_url)
        .call()
        .map_err(|e| anyhow::anyhow!("Failed to query UTXOs: {}", e))?
        .into_json()
        .map_err(|e| anyhow::anyhow!("Failed to parse UTXO response: {}", e))?;

    if utxo_list.utxos.is_empty() {
        bail!("No UTXOs available for this address. Address may have zero balance or no transactions.");
    }

    println!("  Found {} UTXOs", utxo_list.utxos.len());

    // UTXO selection: Greedy algorithm - select smallest UTXOs until we have enough
    // Estimate fee (simple: 1000 satoshis per input + 500 base fee)
    let base_fee = 500u64;
    let per_input_fee = 1000u64;

    let mut selected_utxos = Vec::new();
    let mut total_input = 0u64;
    let mut sorted_utxos = utxo_list.utxos.clone();
    sorted_utxos.sort_by_key(|u| u.amount); // Sort by amount (smallest first)

    for utxo in sorted_utxos {
        selected_utxos.push(utxo.clone());
        total_input += utxo.amount;

        let estimated_fee = base_fee + (selected_utxos.len() as u64 * per_input_fee);
        let required = amount + estimated_fee;

        if total_input >= required {
            break;
        }
    }

    let final_fee = base_fee + (selected_utxos.len() as u64 * per_input_fee);
    let required_total = amount + final_fee;

    if total_input < required_total {
        bail!(
            "Insufficient funds: have {} satoshis, need {} (amount: {}, fee: {})",
            total_input,
            required_total,
            amount,
            final_fee
        );
    }

    println!("  Selected {} UTXOs (total: {} satoshis)", selected_utxos.len(), total_input);
    println!("  Fee: {} satoshis", final_fee);

    // Create transaction outputs
    let mut outputs = vec![TxOutput {
        amount,
        recipient_pubkey_hash: recipient_address,
        script: None,
    }];

    // Add change output if needed
    let change = total_input - required_total;
    if change > 0 {
        println!("  Change: {} satoshis", change);
        outputs.push(TxOutput {
            amount: change,
            recipient_pubkey_hash: sender_address,
            script: None,
        });
    }

    // Create transaction inputs from selected UTXOs
    let mut tx_inputs = Vec::new();
    for utxo in &selected_utxos {
        let tx_hash_bytes = hex::decode(&utxo.tx_hash)?;
        if tx_hash_bytes.len() != 32 {
            bail!("Invalid UTXO tx_hash length: {}", tx_hash_bytes.len());
        }
        let mut tx_hash_array = [0u8; 32];
        tx_hash_array.copy_from_slice(&tx_hash_bytes);

        tx_inputs.push(TxInput {
            previous_output_hash: tx_hash_array,
            output_index: utxo.output_index,
            signature: Signature::Classical(vec![]), // Will be filled after signing
            public_key: public_key.clone(),
            nonce: None,
        });
    }

    // Create unsigned transaction
    let mut tx = Transaction::new(
        1, // version
        tx_inputs,
        outputs,
        chrono::Utc::now().timestamp() as u64,
        None,
    );

    // Sign transaction
    println!("  🔐 Signing transaction...");
    let signing_hash = tx.signing_hash();
    let signature = signature_fn(&signing_hash)?;

    // Update all inputs with the same signature (single-sig)
    for input in &mut tx.inputs {
        input.signature = signature.clone();
    }

    println!("  📦 Transaction created:");
    println!("     TX Hash: {}", hex::encode(tx.hash()));
    println!("     Inputs: {}", tx.inputs.len());
    println!("     Outputs: {}", tx.outputs.len());
    println!("     Size: {} bytes", tx.size_bytes());

    // Serialize transaction
    let tx_bytes = bincode::serialize(&tx)?;
    let tx_hex = hex::encode(&tx_bytes);

    // Submit to RPC
    println!("  📡 Submitting to network...");
    let response: Value = client
        .request("chain_submitTransaction", rpc_params![tx_hex])
        .await?;

    // Response format: {"tx_hash": "..."}
    if let Some(tx_hash) = response["tx_hash"].as_str() {
        println!("  ✅ Transaction submitted successfully!");
        println!("     TX Hash: {}", tx_hash);
    } else {
        bail!("Transaction submission failed: unexpected response format");
    }

    Ok(())
}
