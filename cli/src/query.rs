use anyhow::{Context, Result};
use jsonrpsee::{core::client::ClientT, http_client::HttpClient};
use serde_json::Value;

pub async fn handle_query(client: &HttpClient, query_type: crate::QueryType) -> Result<()> {
    match query_type {
        crate::QueryType::Info => query_info(client).await,
        crate::QueryType::Height => query_height(client).await,
        crate::QueryType::Block { identifier } => query_block(client, &identifier).await,
        crate::QueryType::Tx { hash } => query_transaction(client, &hash).await,
    }
}

async fn query_info(client: &HttpClient) -> Result<()> {
    println!("📊 Querying blockchain info...");

    let response: Value = client
        .request("chain_getInfo", Vec::<()>::new())
        .await
        .context("Failed to query blockchain info")?;

    println!("");
    println!("🔗 Blockchain Information:");
    println!("  Height: {}", response["height"]);
    println!("  Best Block Hash: {}", response["best_block_hash"]);
    println!(
        "  Total Supply: {} BLS ({} base units)",
        response["total_supply"].as_u64().unwrap_or(0) as f64 / 1e8,
        response["total_supply"]
    );
    println!("  Difficulty: {}", response["difficulty"]);

    Ok(())
}

async fn query_height(client: &HttpClient) -> Result<()> {
    let height: u64 = client
        .request("chain_getBlockHeight", Vec::<()>::new())
        .await
        .context("Failed to query block height")?;

    println!("📏 Current block height: {}", height);

    Ok(())
}

async fn query_block(client: &HttpClient, identifier: &str) -> Result<()> {
    println!("📦 Querying block: {}...", identifier);

    let response: Value = if let Ok(height) = identifier.parse::<u64>() {
        // Query by height
        client
            .request("chain_getBlockByHeight", vec![height])
            .await
            .context("Failed to query block by height")?
    } else {
        // Query by hash
        client
            .request("chain_getBlockByHash", vec![identifier])
            .await
            .context("Failed to query block by hash")?
    };

    if response.is_null() {
        println!("❌ Block not found");
        return Ok(());
    }

    println!("");
    println!("🔗 Block Details:");
    println!("  Height: {}", response["header"]["height"]);
    println!("  Hash: {}", response["hash"]);
    println!("  Previous Hash: {}", response["header"]["previous_hash"]);
    println!("  Merkle Root: {}", response["header"]["merkle_root"]);
    println!("  Timestamp: {}", response["header"]["timestamp"]);
    println!(
        "  Difficulty: 0x{:x}",
        response["header"]["difficulty"].as_u64().unwrap_or(0)
    );
    println!("  Nonce: {}", response["header"]["nonce"]);
    println!(
        "  Transactions: {}",
        response["transactions"]
            .as_array()
            .map(|a| a.len())
            .unwrap_or(0)
    );

    Ok(())
}

async fn query_transaction(client: &HttpClient, hash: &str) -> Result<()> {
    println!("💸 Querying transaction: {}...", hash);

    let response: Value = client
        .request("chain_getTransaction", vec![hash])
        .await
        .context("Failed to query transaction")?;

    if response.is_null() {
        println!("❌ Transaction not found");
        return Ok(());
    }

    println!("");
    println!("💰 Transaction Details:");
    println!("  Hash: {}", hash);
    println!("  Version: {}", response["version"]);
    println!(
        "  Inputs: {}",
        response["inputs"].as_array().map(|a| a.len()).unwrap_or(0)
    );
    println!(
        "  Outputs: {}",
        response["outputs"].as_array().map(|a| a.len()).unwrap_or(0)
    );
    println!("  Timestamp: {}", response["timestamp"]);

    Ok(())
}

pub async fn check_balance(client: &HttpClient, address: &str) -> Result<()> {
    println!("💰 Checking balance for address: {}...", address);

    let balance: u64 = client
        .request("chain_getBalance", vec![address])
        .await
        .context("Failed to query balance")?;

    let nonce: u64 = client
        .request("chain_getNonce", vec![address])
        .await
        .context("Failed to query nonce")?;

    println!("");
    println!("📫 Address: {}", address);
    println!(
        "💵 Balance: {} BLS ({} base units)",
        balance as f64 / 1e8,
        balance
    );
    println!("🔢 Nonce: {}", nonce);

    Ok(())
}
