use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use jsonrpsee::http_client::{HttpClient, HttpClientBuilder};
use std::path::PathBuf;

mod keygen;
mod query;
mod tx;

#[derive(Parser)]
#[command(name = "boundless-cli")]
#[command(about = "Boundless BLS Blockchain CLI", long_about = None)]
#[command(version = "0.1.0")]
struct Cli {
    /// RPC endpoint URL
    #[arg(long, default_value = "http://127.0.0.1:9933", global = true)]
    rpc_url: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a new keypair
    Keygen {
        /// Algorithm to use: ml-dsa, falcon, ed25519
        #[arg(short, long, default_value = "ml-dsa")]
        algorithm: String,

        /// Output file path (will create .pub and .priv files)
        #[arg(short, long)]
        output: PathBuf,
    },

    /// Query blockchain data
    Query {
        #[command(subcommand)]
        query_type: QueryType,
    },

    /// Check account balance
    Balance {
        /// Account address (hex-encoded 32 bytes)
        address: String,
    },

    /// Send tokens to an address
    Send {
        /// Recipient address (hex-encoded 32 bytes)
        to: String,

        /// Amount to send (in base units)
        amount: u64,

        /// Private key file
        #[arg(short, long)]
        key: PathBuf,
    },
}

#[derive(Subcommand)]
enum QueryType {
    /// Get blockchain info
    Info,

    /// Get current block height
    Height,

    /// Get block by height or hash
    Block {
        /// Block height or hash
        identifier: String,
    },

    /// Get transaction by hash
    Tx {
        /// Transaction hash (hex-encoded)
        hash: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Keygen { algorithm, output } => {
            keygen::generate_keypair(&algorithm, &output)?;
        }
        Commands::Query { query_type } => {
            let client = create_rpc_client(&cli.rpc_url)?;
            query::handle_query(&client, query_type).await?;
        }
        Commands::Balance { address } => {
            let client = create_rpc_client(&cli.rpc_url)?;
            query::check_balance(&client, &address).await?;
        }
        Commands::Send { to, amount, key } => {
            let client = create_rpc_client(&cli.rpc_url)?;
            tx::send_transaction(&client, &cli.rpc_url, &to, amount, &key).await?;
        }
    }

    Ok(())
}

fn create_rpc_client(url: &str) -> Result<HttpClient> {
    HttpClientBuilder::default()
        .build(url)
        .context("Failed to create RPC client")
}
