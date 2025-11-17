use anyhow::{bail, Result};
use boundless_crypto::{Falcon512, MlDsa44};
use sha3::{Digest, Sha3_256};
use std::fs;
use std::path::Path;

pub fn generate_keypair(algorithm: &str, output: &Path) -> Result<()> {
    println!("🔑 Generating {} keypair...", algorithm);

    let (public_key, secret_key) = match algorithm.to_lowercase().as_str() {
        "ml-dsa" | "ml-dsa-44" => {
            let signer = MlDsa44::new()?;
            signer.keypair()?
        }
        "falcon" | "falcon-512" => {
            let signer = Falcon512::new()?;
            signer.keypair()?
        }
        _ => bail!("Unsupported algorithm: {}. Use ml-dsa or falcon", algorithm),
    };

    // Create output directory if it doesn't exist
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }

    // Save private key
    let priv_path = output.with_extension("priv");
    fs::write(&priv_path, hex::encode(&secret_key))?;
    println!("🔐 Private key saved to: {}", priv_path.display());

    // Save public key
    let pub_path = output.with_extension("pub");
    fs::write(&pub_path, hex::encode(&public_key))?;
    println!("🔓 Public key saved to: {}", pub_path.display());

    // Calculate and display address (hash of public key)
    let mut hasher = Sha3_256::new();
    hasher.update(&public_key);
    let address = hasher.finalize();
    println!("📫 Address: {}", hex::encode(address));

    println!("✅ Keypair generated successfully!");
    println!("");
    println!("⚠️  IMPORTANT: Keep your private key file secure and never share it!");

    Ok(())
}
