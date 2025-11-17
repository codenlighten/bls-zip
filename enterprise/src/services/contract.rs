// Contract Service - Smart Contract Deployment and Management
//
// This service provides integration between E2 Multipass and smart contracts,
// enabling contract deployment, interaction, and lifecycle management.

use crate::error::{EnterpriseError, Result};
use crate::models::*;
use crate::blockchain::BlockchainClient;
use crate::transaction::deployment::{DeployerKey, DeploymentBuilder};
use crate::wasm_loader::WasmLoader;
use crate::abi::{ContractAbi, AbiFunction, encode_call, decode_return, load_abi_from_json};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use std::collections::HashMap;
use std::sync::Arc;

/// Contract deployment status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "contract_status", rename_all = "snake_case")]
pub enum ContractStatus {
    Pending,
    Deploying,
    Deployed,
    Failed,
    Paused,
    Terminated,
}

/// Contract template type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "contract_template_type", rename_all = "snake_case")]
pub enum ContractTemplateType {
    IdentityAccessControl,
    MultisigWallet,
    AssetEscrow,
    AppAuthorization,
    Custom,
}

/// Smart contract record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contract {
    pub contract_id: Uuid,
    pub identity_id: Uuid,
    pub template_type: ContractTemplateType,
    pub name: String,
    pub description: Option<String>,
    pub wasm_hash: String,
    pub contract_address: Option<String>,
    pub abi_json: Option<serde_json::Value>,
    pub constructor_args: Option<serde_json::Value>,
    pub status: ContractStatus,
    pub gas_used: Option<i64>,
    pub deployment_tx_hash: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub deployed_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Contract interaction record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractInteraction {
    pub interaction_id: Uuid,
    pub contract_id: Uuid,
    pub identity_id: Uuid,
    pub method_name: String,
    pub method_args: Option<serde_json::Value>,
    pub tx_hash: Option<String>,
    pub status: String,
    pub gas_used: Option<i64>,
    pub result: Option<serde_json::Value>,
    pub error_message: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Contract deployment request
#[derive(Debug, Deserialize)]
pub struct DeployContractRequest {
    pub template_type: ContractTemplateType,
    pub name: String,
    pub description: Option<String>,
    pub constructor_args: Option<serde_json::Value>,
    pub gas_limit: Option<i64>,
}

/// Contract interaction request
#[derive(Debug, Deserialize)]
pub struct ContractCallRequest {
    pub method_name: String,
    pub method_args: Option<serde_json::Value>,
    pub gas_limit: Option<i64>,
}

/// Contract template metadata for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateMetadata {
    pub template_id: String,
    pub template_name: String,
    pub category: String,
    pub description: String,
    pub version: String,
    pub jurisdiction: Vec<String>,
    pub natural_language_terms: String,
    pub code_hash: String,
    pub parameters: Vec<TemplateParameter>,
    pub is_verified: bool,
    pub created_by: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Template parameter definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateParameter {
    pub param_name: String,
    pub param_type: String,
    pub description: String,
    pub required: bool,
    pub default_value: Option<serde_json::Value>,
}

/// Contract Service
pub struct ContractService {
    pool: PgPool,
    blockchain_client: Option<Arc<BlockchainClient>>,
    abi_cache: HashMap<ContractTemplateType, ContractAbi>,
}

impl ContractService {
    /// Create a new contract service
    ///
    /// If blockchain_client is None, the service will operate in MOCK MODE
    /// and return simulated responses for development/testing.
    pub fn new(pool: PgPool, blockchain_client: Option<Arc<BlockchainClient>>) -> Self {
        if blockchain_client.is_none() {
            tracing::warn!(
                "ContractService initialized in MOCK MODE - contracts will not be deployed to blockchain. \
                Set BOUNDLESS_HTTP_URL environment variable to enable real blockchain integration."
            );
        }

        // Load ABI definitions for all contract templates
        let mut abi_cache = HashMap::new();
        let templates = vec![
            (ContractTemplateType::IdentityAccessControl, "identity_access_control"),
            (ContractTemplateType::MultisigWallet, "multisig_wallet"),
            (ContractTemplateType::AssetEscrow, "asset_escrow"),
            (ContractTemplateType::AppAuthorization, "app_authorization"),
        ];

        for (template_type, name) in templates {
            if let Ok(abi) = Self::load_contract_abi(name) {
                abi_cache.insert(template_type, abi);
            } else {
                tracing::warn!("Failed to load ABI for template: {}", name);
            }
        }

        Self { pool, blockchain_client, abi_cache }
    }

    /// Get available contract templates
    pub fn get_templates(&self, category_filter: Option<String>) -> Result<Vec<TemplateMetadata>> {
        let mut templates = vec![
            TemplateMetadata {
                template_id: "identity_access_control".to_string(),
                template_name: "Identity Access Control".to_string(),
                category: "business".to_string(),
                description: "Role-based access control (RBAC) with E2 identity verification. Manage permissions for enterprise applications with KYC verification and attestation checking.".to_string(),
                version: "1.0.0".to_string(),
                jurisdiction: vec!["GLOBAL".to_string()],
                natural_language_terms: "This smart contract provides role-based access control for enterprise applications. Users can be assigned roles (Owner, Admin, User, Guest) with different permission levels. The contract integrates with E2 identity verification to ensure only verified identities can access protected resources.".to_string(),
                code_hash: "0x1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c2d3e4f5a6b7c8d9e0f1a2b".to_string(),
                parameters: vec![
                    TemplateParameter {
                        param_name: "owner".to_string(),
                        param_type: "identity_id".to_string(),
                        description: "E2 identity ID of the contract owner".to_string(),
                        required: true,
                        default_value: None,
                    },
                    TemplateParameter {
                        param_name: "default_role".to_string(),
                        param_type: "string".to_string(),
                        description: "Default role for new users (user, guest)".to_string(),
                        required: false,
                        default_value: Some(serde_json::json!("user")),
                    },
                ],
                is_verified: true,
                created_by: "Boundless BLS".to_string(),
                created_at: "2024-01-15T00:00:00Z".to_string(),
                updated_at: "2024-01-15T00:00:00Z".to_string(),
            },
            TemplateMetadata {
                template_id: "multisig_wallet".to_string(),
                template_name: "Multi-Signature Wallet".to_string(),
                category: "business".to_string(),
                description: "M-of-N signature requirements with daily spending limits and time-locked transactions. Secure treasury management for corporate wallets requiring multiple approvals.".to_string(),
                version: "1.0.0".to_string(),
                jurisdiction: vec!["GLOBAL".to_string()],
                natural_language_terms: "This smart contract creates a multi-signature wallet requiring M-of-N signatures for transactions. It includes daily spending limits, time-locked transactions, and integrates with E2 identity for signer management. Perfect for corporate treasuries and shared wallets.".to_string(),
                code_hash: "0x2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c2d3e4f5a6b7c8d9e0f1a2b3c".to_string(),
                parameters: vec![
                    TemplateParameter {
                        param_name: "signers".to_string(),
                        param_type: "identity_id[]".to_string(),
                        description: "List of E2 identity IDs authorized to sign transactions".to_string(),
                        required: true,
                        default_value: None,
                    },
                    TemplateParameter {
                        param_name: "required_signatures".to_string(),
                        param_type: "number".to_string(),
                        description: "Number of signatures required to approve a transaction".to_string(),
                        required: true,
                        default_value: None,
                    },
                    TemplateParameter {
                        param_name: "daily_limit".to_string(),
                        param_type: "number".to_string(),
                        description: "Maximum amount that can be transferred per day".to_string(),
                        required: false,
                        default_value: Some(serde_json::json!(1000000)),
                    },
                ],
                is_verified: true,
                created_by: "Boundless BLS".to_string(),
                created_at: "2024-01-15T00:00:00Z".to_string(),
                updated_at: "2024-01-15T00:00:00Z".to_string(),
            },
            TemplateMetadata {
                template_id: "asset_escrow".to_string(),
                template_name: "Asset Escrow".to_string(),
                category: "business".to_string(),
                description: "P2P asset trading with atomic swaps, dispute resolution, and multi-asset bundles. Secure asset trading and peer-to-peer exchanges using E2's locked quantity system.".to_string(),
                version: "1.0.0".to_string(),
                jurisdiction: vec!["GLOBAL".to_string()],
                natural_language_terms: "This smart contract enables secure peer-to-peer asset trading with escrow protection. Assets are locked in the contract until both parties fulfill their obligations. Includes dispute resolution mechanisms and supports multi-asset bundle trades.".to_string(),
                code_hash: "0x3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c2d3e4f5a6b7c8d9e0f1a2b3c4d".to_string(),
                parameters: vec![
                    TemplateParameter {
                        param_name: "buyer_id".to_string(),
                        param_type: "identity_id".to_string(),
                        description: "E2 identity ID of the buyer".to_string(),
                        required: true,
                        default_value: None,
                    },
                    TemplateParameter {
                        param_name: "seller_id".to_string(),
                        param_type: "identity_id".to_string(),
                        description: "E2 identity ID of the seller".to_string(),
                        required: true,
                        default_value: None,
                    },
                    TemplateParameter {
                        param_name: "asset_id".to_string(),
                        param_type: "string".to_string(),
                        description: "ID of the asset being traded".to_string(),
                        required: true,
                        default_value: None,
                    },
                    TemplateParameter {
                        param_name: "price".to_string(),
                        param_type: "number".to_string(),
                        description: "Agreed price for the asset".to_string(),
                        required: true,
                        default_value: None,
                    },
                ],
                is_verified: true,
                created_by: "Boundless BLS".to_string(),
                created_at: "2024-01-15T00:00:00Z".to_string(),
                updated_at: "2024-01-15T00:00:00Z".to_string(),
            },
            TemplateMetadata {
                template_id: "app_authorization".to_string(),
                template_name: "App Authorization".to_string(),
                category: "business".to_string(),
                description: "OAuth-like authorization framework with scoped permissions and time-limited grants. Delegate permissions to third-party applications securely.".to_string(),
                version: "1.0.0".to_string(),
                jurisdiction: vec!["GLOBAL".to_string()],
                natural_language_terms: "This smart contract provides an OAuth-like authorization system for third-party applications. Users can grant scoped permissions to applications with time-limited access tokens. Integrates with E2's application service for app registration and management.".to_string(),
                code_hash: "0x4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c2d3e4f5a6b7c8d9e0f1a2b3c4d5e".to_string(),
                parameters: vec![
                    TemplateParameter {
                        param_name: "user_id".to_string(),
                        param_type: "identity_id".to_string(),
                        description: "E2 identity ID granting the authorization".to_string(),
                        required: true,
                        default_value: None,
                    },
                    TemplateParameter {
                        param_name: "app_id".to_string(),
                        param_type: "string".to_string(),
                        description: "Application ID receiving authorization".to_string(),
                        required: true,
                        default_value: None,
                    },
                    TemplateParameter {
                        param_name: "scopes".to_string(),
                        param_type: "string[]".to_string(),
                        description: "List of permission scopes being granted".to_string(),
                        required: true,
                        default_value: None,
                    },
                    TemplateParameter {
                        param_name: "expires_in".to_string(),
                        param_type: "number".to_string(),
                        description: "Authorization expiry time in seconds".to_string(),
                        required: false,
                        default_value: Some(serde_json::json!(86400)),
                    },
                ],
                is_verified: true,
                created_by: "Boundless BLS".to_string(),
                created_at: "2024-01-15T00:00:00Z".to_string(),
                updated_at: "2024-01-15T00:00:00Z".to_string(),
            },
        ];

        // Filter by category if provided
        if let Some(category) = category_filter {
            templates.retain(|t| t.category == category);
        }

        Ok(templates)
    }

    /// Deploy a new smart contract
    pub async fn deploy_contract(
        &self,
        identity_id: Uuid,
        request: DeployContractRequest,
    ) -> Result<Contract> {
        let contract_id = Uuid::new_v4();

        // Get WASM bytecode for template
        let wasm_bytes = self.get_template_wasm(&request.template_type)?;
        let wasm_hash = self.calculate_wasm_hash(&wasm_bytes);

        // Create contract record
        let contract = sqlx::query_as!(
            Contract,
            r#"
            INSERT INTO contracts (
                contract_id, identity_id, template_type, name, description,
                wasm_hash, constructor_args, status, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW())
            RETURNING
                contract_id,
                identity_id,
                template_type as "template_type: ContractTemplateType",
                name,
                description,
                wasm_hash,
                contract_address,
                abi_json,
                constructor_args,
                status as "status: ContractStatus",
                gas_used,
                deployment_tx_hash,
                metadata,
                created_at,
                deployed_at
            "#,
            contract_id,
            identity_id,
            request.template_type as ContractTemplateType,
            request.name,
            request.description,
            wasm_hash,
            request.constructor_args,
            ContractStatus::Pending as ContractStatus,
        )
        .fetch_one(&self.pool)
        .await?;

        // Deploy to blockchain (real or mock depending on configuration)
        if let Some(client) = &self.blockchain_client {
            // Real blockchain deployment
            tracing::info!(
                "Deploying contract {} to blockchain using real Boundless node",
                contract_id
            );

            // Update status to Deploying
            sqlx::query!(
                r#"
                UPDATE contracts
                SET status = $1
                WHERE contract_id = $2
                "#,
                ContractStatus::Deploying as ContractStatus,
                contract_id,
            )
            .execute(&self.pool)
            .await?;

            // Load deployer key from environment
            let deployer_key = DeployerKey::from_env().map_err(|e| {
                tracing::error!("Failed to load deployer key: {}", e);
                e
            })?;

            // Create deployment builder
            let deployment_builder = DeploymentBuilder::new(client.clone());

            // Deploy contract (includes: UTXO query, tx building, signing, submission, polling)
            match deployment_builder.deploy_contract(&deployer_key, wasm_bytes).await {
                Ok(contract_address) => {
                    tracing::info!(
                        "Contract {} deployed successfully at address {}",
                        contract_id,
                        contract_address
                    );

                    // Mark as deployed with real contract address
                    self.mark_deployed(
                        contract_id,
                        contract_address.clone(),
                        contract_address, // tx_hash = contract_address in our scheme
                        0, // gas_used tracking not yet implemented
                    ).await?;

                    // Fetch updated contract
                    self.get_contract(contract_id).await
                }
                Err(e) => {
                    tracing::error!("Contract {} deployment failed: {}", contract_id, e);

                    // Mark as failed
                    sqlx::query!(
                        r#"
                        UPDATE contracts
                        SET status = $1
                        WHERE contract_id = $2
                        "#,
                        ContractStatus::Failed as ContractStatus,
                        contract_id,
                    )
                    .execute(&self.pool)
                    .await?;

                    Err(e)
                }
            }
        } else {
            // Mock mode for development/testing
            tracing::warn!(
                "MOCK MODE: Contract {} deployment simulated (blockchain client not configured)",
                contract_id
            );

            self.mark_deployed(
                contract_id,
                format!("0x{}", hex::encode(&contract_id.as_bytes()[..20])),
                "0x0000000000000000000000000000000000000000000000000000000000000000".to_string(),
                request.gas_limit.unwrap_or(50_000_000),
            ).await?;

            Ok(contract)
        }
    }

    /// Get contract by ID
    pub async fn get_contract(&self, contract_id: Uuid) -> Result<Contract> {
        let contract = sqlx::query_as!(
            Contract,
            r#"
            SELECT
                contract_id,
                identity_id,
                template_type as "template_type: ContractTemplateType",
                name,
                description,
                wasm_hash,
                contract_address,
                abi_json,
                constructor_args,
                status as "status: ContractStatus",
                gas_used,
                deployment_tx_hash,
                metadata,
                created_at,
                deployed_at
            FROM contracts
            WHERE contract_id = $1
            "#,
            contract_id
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(EnterpriseError::NotFound("Contract not found".to_string()))?;

        Ok(contract)
    }

    /// List contracts for an identity
    pub async fn list_contracts(&self, identity_id: Uuid) -> Result<Vec<Contract>> {
        let contracts = sqlx::query_as!(
            Contract,
            r#"
            SELECT
                contract_id,
                identity_id,
                template_type as "template_type: ContractTemplateType",
                name,
                description,
                wasm_hash,
                contract_address,
                abi_json,
                constructor_args,
                status as "status: ContractStatus",
                gas_used,
                deployment_tx_hash,
                metadata,
                created_at,
                deployed_at
            FROM contracts
            WHERE identity_id = $1
            ORDER BY created_at DESC
            "#,
            identity_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(contracts)
    }

    /// Call a contract method (read-only)
    pub async fn call_contract(
        &self,
        identity_id: Uuid,
        contract_id: Uuid,
        request: ContractCallRequest,
    ) -> Result<serde_json::Value> {
        // Verify contract exists and is deployed
        let contract = self.get_contract(contract_id).await?;

        if contract.status != ContractStatus::Deployed {
            return Err(EnterpriseError::Internal(
                "Contract is not deployed".to_string()
            ));
        }

        // Get ABI function for this method call
        let abi_function = self.get_abi_function(&contract.template_type, &request.method_name)?;

        // Verify this is a read-only call
        if !abi_function.is_read_only() {
            return Err(EnterpriseError::ValidationError(
                format!("Method '{}' is not a read-only function. Use send_transaction instead.", request.method_name)
            ));
        }

        // Call contract (real or mock depending on configuration)
        let response = if let Some(client) = &self.blockchain_client {
            // Real blockchain contract call
            tracing::info!(
                "Calling contract {} method '{}' on blockchain",
                contract_id,
                request.method_name
            );

            // Encode the function call using ABI
            let call_data = encode_call(&abi_function, &request.method_args.clone().unwrap_or(serde_json::json!({})))?;

            // Query contract via RPC
            let contract_address = contract.contract_address.as_ref()
                .ok_or_else(|| EnterpriseError::Internal("Contract has no address".to_string()))?;

            match client.query_contract(contract_address, &call_data).await {
                Ok(response_bytes) => {
                    // Decode response using ABI
                    let decoded_result = decode_return(&abi_function, &response_bytes)?;

                    serde_json::json!({
                        "success": true,
                        "result": decoded_result,
                        "method": request.method_name
                    })
                }
                Err(e) => {
                    tracing::error!("Contract call failed: {}", e);
                    serde_json::json!({
                        "success": false,
                        "error": format!("Contract call failed: {}", e),
                        "method": request.method_name
                    })
                }
            }
        } else {
            // Mock mode for development/testing
            tracing::warn!(
                "MOCK MODE: Contract call simulated (blockchain client not configured)"
            );

            serde_json::json!({
                "success": true,
                "result": {
                    "message": "Contract call successful (mocked)",
                    "method": request.method_name,
                    "gas_used": 10000
                }
            })
        };

        // Record interaction
        let gas_used = if self.blockchain_client.is_some() { None } else { Some(10000) };
        let status = if self.blockchain_client.is_some() { "pending_implementation" } else { "success" };

        self.record_interaction(
            contract_id,
            identity_id,
            request.method_name.clone(),
            request.method_args.clone(),
            None, // No tx hash for read-only calls
            status.to_string(),
            gas_used,
            Some(response.clone()),
            None,
        ).await?;

        Ok(response)
    }

    /// Send a transaction to a contract (state-changing)
    pub async fn send_transaction(
        &self,
        identity_id: Uuid,
        contract_id: Uuid,
        request: ContractCallRequest,
    ) -> Result<serde_json::Value> {
        // Verify contract exists and is deployed
        let contract = self.get_contract(contract_id).await?;

        if contract.status != ContractStatus::Deployed {
            return Err(EnterpriseError::Internal(
                "Contract is not deployed".to_string()
            ));
        }

        // Get ABI function for this method call
        let abi_function = self.get_abi_function(&contract.template_type, &request.method_name)?;

        // Verify this is NOT a read-only call
        if abi_function.is_read_only() {
            return Err(EnterpriseError::ValidationError(
                format!("Method '{}' is a read-only function. Use call_contract instead.", request.method_name)
            ));
        }

        // Send transaction (real or mock depending on configuration)
        let (response, tx_hash, status, gas_used) = if let Some(client) = &self.blockchain_client {
            // Real blockchain transaction
            tracing::info!(
                "Sending transaction to contract {} method '{}'",
                contract_id,
                request.method_name
            );

            // Encode the function call using ABI
            let call_data = encode_call(&abi_function, &request.method_args.clone().unwrap_or(serde_json::json!({})))?;

            let contract_address = contract.contract_address.as_ref()
                .ok_or_else(|| EnterpriseError::Internal("Contract has no address".to_string()))?;

            // Build and send contract transaction
            // NOTE: This requires user key management which is being deferred
            // For now, we'll use the deployer key as a placeholder
            tracing::warn!(
                "Using deployer key for contract transaction - user key management not yet implemented"
            );

            match DeployerKey::from_env() {
                Ok(deployer_key) => {
                    // Build transaction with contract call data
                    match client.send_contract_transaction(contract_address, &call_data, &deployer_key).await {
                        Ok(tx_hash_value) => {
                            let resp = serde_json::json!({
                                "success": true,
                                "tx_hash": tx_hash_value,
                                "method": request.method_name
                            });
                            (resp, Some(tx_hash_value), "submitted".to_string(), Some(50000))
                        }
                        Err(e) => {
                            tracing::error!("Contract transaction failed: {}", e);
                            let resp = serde_json::json!({
                                "success": false,
                                "error": format!("Transaction failed: {}", e),
                                "method": request.method_name
                            });
                            (resp, None, "failed".to_string(), None)
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to load deployer key: {}", e);
                    let resp = serde_json::json!({
                        "success": false,
                        "error": format!("Key management error: {}", e),
                        "method": request.method_name
                    });
                    (resp, None, "failed".to_string(), None)
                }
            }
        } else {
            // Mock mode for development/testing
            tracing::warn!(
                "MOCK MODE: Contract transaction simulated (blockchain client not configured)"
            );

            let mock_tx_hash = format!("0x{}", hex::encode(&Uuid::new_v4().as_bytes()));
            let resp = serde_json::json!({
                "success": true,
                "tx_hash": mock_tx_hash,
                "gas_used": 50000
            });

            (resp, Some(mock_tx_hash), "success".to_string(), Some(50000))
        };

        // Record interaction
        self.record_interaction(
            contract_id,
            identity_id,
            request.method_name.clone(),
            request.method_args.clone(),
            tx_hash,
            status,
            gas_used,
            Some(response.clone()),
            None,
        ).await?;

        Ok(response)
    }

    /// Get contract interactions
    pub async fn get_interactions(
        &self,
        contract_id: Uuid,
    ) -> Result<Vec<ContractInteraction>> {
        let interactions = sqlx::query_as!(
            ContractInteraction,
            r#"
            SELECT
                interaction_id,
                contract_id,
                identity_id,
                method_name,
                method_args,
                tx_hash,
                status,
                gas_used,
                result,
                error_message,
                created_at
            FROM contract_interactions
            WHERE contract_id = $1
            ORDER BY created_at DESC
            LIMIT 100
            "#,
            contract_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(interactions)
    }

    // === Internal Helper Functions ===

    /// Mark contract as deployed
    async fn mark_deployed(
        &self,
        contract_id: Uuid,
        contract_address: String,
        tx_hash: String,
        gas_used: i64,
    ) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE contracts
            SET
                status = $1,
                contract_address = $2,
                deployment_tx_hash = $3,
                gas_used = $4,
                deployed_at = NOW()
            WHERE contract_id = $5
            "#,
            ContractStatus::Deployed as ContractStatus,
            contract_address,
            tx_hash,
            gas_used,
            contract_id,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Record contract interaction
    #[allow(clippy::too_many_arguments)]
    async fn record_interaction(
        &self,
        contract_id: Uuid,
        identity_id: Uuid,
        method_name: String,
        method_args: Option<serde_json::Value>,
        tx_hash: Option<String>,
        status: String,
        gas_used: Option<i64>,
        result: Option<serde_json::Value>,
        error_message: Option<String>,
    ) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO contract_interactions (
                interaction_id, contract_id, identity_id, method_name,
                method_args, tx_hash, status, gas_used, result,
                error_message, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, NOW())
            "#,
            Uuid::new_v4(),
            contract_id,
            identity_id,
            method_name,
            method_args,
            tx_hash,
            status,
            gas_used,
            result,
            error_message,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get WASM bytecode for template
    fn get_template_wasm(&self, template_type: &ContractTemplateType) -> Result<Vec<u8>> {
        let template_name = match template_type {
            ContractTemplateType::IdentityAccessControl => "identity_access_control",
            ContractTemplateType::MultisigWallet => "multisig_wallet",
            ContractTemplateType::AssetEscrow => "asset_escrow",
            ContractTemplateType::AppAuthorization => "app_authorization",
            ContractTemplateType::Custom => "custom",
        };

        // Load WASM bytecode using WasmLoader
        let wasm_loader = WasmLoader::from_env();
        wasm_loader.load_template(template_name)
    }

    /// Calculate WASM hash
    fn calculate_wasm_hash(&self, wasm_bytes: &[u8]) -> String {
        use sha3::{Digest, Sha3_256};
        let mut hasher = Sha3_256::new();
        hasher.update(wasm_bytes);
        let result = hasher.finalize();
        hex::encode(result)
    }

    /// Load contract ABI from JSON file
    fn load_contract_abi(template_name: &str) -> Result<ContractAbi> {
        // Try loading from contracts/abis directory
        let abi_path = format!("contracts/abis/{}.json", template_name);

        let abi_json = std::fs::read_to_string(&abi_path)
            .map_err(|e| EnterpriseError::Internal(
                format!("Failed to load ABI file {}: {}", abi_path, e)
            ))?;

        load_abi_from_json(&abi_json)
    }

    /// Get ABI function by name
    fn get_abi_function(&self, template_type: &ContractTemplateType, method_name: &str) -> Result<AbiFunction> {
        let abi = self.abi_cache.get(template_type)
            .ok_or_else(|| EnterpriseError::Internal(
                format!("ABI not found for contract template: {:?}", template_type)
            ))?;

        abi.functions.iter()
            .find(|f| f.name == method_name)
            .cloned()
            .ok_or_else(|| EnterpriseError::ValidationError(
                format!("Method '{}' not found in contract ABI", method_name)
            ))
    }
}
