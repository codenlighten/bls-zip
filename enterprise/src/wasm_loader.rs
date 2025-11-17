// WASM Bytecode Loader for Contract Deployment
//
// Loads compiled WASM bytecode for contract templates.
// Supports both embedded bytecode (for testing) and external files.

use crate::error::{EnterpriseError, Result};
use std::fs;
use std::path::PathBuf;

/// Simple placeholder WASM module for testing
/// This is a minimal valid WASM binary that does nothing
/// In production, this should be replaced with actual compiled contract bytecode
const PLACEHOLDER_WASM: &[u8] = &[
    0x00, 0x61, 0x73, 0x6d, // Magic number: \0asm
    0x01, 0x00, 0x00, 0x00, // Version: 1
];

/// WASM loader for contract templates
pub struct WasmLoader {
    /// Base directory for WASM files
    wasm_dir: Option<PathBuf>,
    /// Use embedded placeholder bytecode
    use_placeholder: bool,
}

impl WasmLoader {
    /// Create a new WASM loader
    ///
    /// If `wasm_dir` is None, will use embedded placeholder bytecode
    pub fn new(wasm_dir: Option<PathBuf>) -> Self {
        let use_placeholder = wasm_dir.is_none();

        if use_placeholder {
            tracing::warn!(
                "WASM loader initialized with placeholder bytecode. \
                Set WASM_DIR environment variable to load real contract bytecode."
            );
        }

        Self {
            wasm_dir,
            use_placeholder,
        }
    }

    /// Create WASM loader from environment
    ///
    /// Checks WASM_DIR environment variable
    pub fn from_env() -> Self {
        let wasm_dir = std::env::var("WASM_DIR")
            .ok()
            .filter(|s| !s.is_empty())
            .map(PathBuf::from);

        Self::new(wasm_dir)
    }

    /// Load WASM bytecode for a template
    ///
    /// Template names:
    /// - "identity_access_control"
    /// - "app_authorization"
    /// - "asset_escrow"
    /// - "multisig_wallet"
    pub fn load_template(&self, template_name: &str) -> Result<Vec<u8>> {
        if self.use_placeholder {
            tracing::warn!(
                "Loading placeholder WASM for template '{}'. \
                This is NOT a real contract - for testing only!",
                template_name
            );
            return Ok(PLACEHOLDER_WASM.to_vec());
        }

        // Load from external file
        let wasm_dir = self.wasm_dir.as_ref()
            .ok_or_else(|| EnterpriseError::ConfigError(
                "WASM directory not configured".to_string()
            ))?;

        let wasm_filename = format!("{}.wasm", template_name);
        let wasm_path = wasm_dir.join(&wasm_filename);

        if !wasm_path.exists() {
            return Err(EnterpriseError::ConfigError(
                format!(
                    "WASM file not found: {}. Expected at: {}",
                    wasm_filename,
                    wasm_path.display()
                )
            ));
        }

        tracing::info!(
            "Loading WASM bytecode for template '{}' from {}",
            template_name,
            wasm_path.display()
        );

        let wasm_bytes = fs::read(&wasm_path)
            .map_err(|e| EnterpriseError::ConfigError(
                format!("Failed to read WASM file {}: {}", wasm_path.display(), e)
            ))?;

        // Basic validation: check WASM magic number
        if wasm_bytes.len() < 4 || &wasm_bytes[0..4] != b"\0asm" {
            return Err(EnterpriseError::ConfigError(
                format!("Invalid WASM file: {} (missing magic number)", wasm_path.display())
            ));
        }

        tracing::info!(
            "Loaded {} bytes of WASM bytecode for template '{}'",
            wasm_bytes.len(),
            template_name
        );

        Ok(wasm_bytes)
    }

    /// Check if using placeholder bytecode
    pub fn is_placeholder_mode(&self) -> bool {
        self.use_placeholder
    }

    /// Get WASM directory path
    pub fn wasm_dir(&self) -> Option<&PathBuf> {
        self.wasm_dir.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_placeholder_loader() {
        let loader = WasmLoader::new(None);
        assert!(loader.is_placeholder_mode());

        let wasm = loader.load_template("test_template").unwrap();
        assert_eq!(&wasm[0..4], b"\0asm"); // Valid WASM magic number
    }

    #[test]
    fn test_missing_directory() {
        let loader = WasmLoader::new(Some(PathBuf::from("/nonexistent/path")));
        assert!(!loader.is_placeholder_mode());

        let result = loader.load_template("test_template");
        assert!(result.is_err());
    }

    #[test]
    fn test_env_loader_no_env() {
        // Without WASM_DIR set, should use placeholder
        std::env::remove_var("WASM_DIR");
        let loader = WasmLoader::from_env();
        assert!(loader.is_placeholder_mode());
    }
}
