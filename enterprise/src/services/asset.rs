// Asset & Market Service - Token management and internal trading

use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;

use crate::error::{EnterpriseError, Result};
use crate::models::*;

pub struct AssetService {
    db: PgPool,
    blockchain_rpc_url: String,
    http_client: reqwest::Client,
}

impl AssetService {
    pub fn new(db: PgPool, blockchain_rpc_url: String) -> Self {
        Self {
            db,
            blockchain_rpc_url,
            http_client: reqwest::Client::new(),
        }
    }

    /// Define new asset
    pub async fn define_asset(
        &self,
        asset_type: AssetType,
        symbol: String,
        name: String,
        total_supply: i64,
        metadata: serde_json::Value,
    ) -> Result<AssetDefinition> {
        // Validate inputs
        if symbol.is_empty() || name.is_empty() {
            return Err(EnterpriseError::InvalidInput("Symbol and name cannot be empty".to_string()));
        }

        // Create asset
        let asset_id = Uuid::new_v4();

        sqlx::query!(
            r#"
            INSERT INTO asset_definitions
            (asset_id, name, symbol, asset_type, total_supply, circulating_supply, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
            asset_id,
            name,
            symbol,
            format!("{:?}", asset_type),
            total_supply,
            0i64,
            metadata
        )
        .execute(&self.db)
        .await
        .map_err(|e| EnterpriseError::from_db_error(e))?;

        Ok(AssetDefinition {
            asset_id,
            asset_type,
            symbol,
            name,
            total_supply,
            circulating_supply: 0,
            metadata,
            created_at: Utc::now(),
        })
    }

    /// Get asset by ID
    pub async fn get_asset(&self, asset_id: Uuid) -> Result<AssetDefinition> {
        let row = sqlx::query!(
            r#"
            SELECT asset_id, name, symbol, asset_type, total_supply, circulating_supply, metadata, created_at
            FROM asset_definitions
            WHERE asset_id = $1
            "#,
            asset_id
        )
        .fetch_optional(&self.db)
        .await
        .map_err(|e| EnterpriseError::from_db_error(e))?
        .ok_or_else(|| EnterpriseError::AssetNotFound(asset_id.to_string()))?;

        let asset_type: AssetType = serde_json::from_value(
            serde_json::Value::String(row.asset_type)
        ).unwrap_or(AssetType::Native);

        Ok(AssetDefinition {
            asset_id: row.asset_id,
            asset_type,
            symbol: row.symbol,
            name: row.name,
            total_supply: row.total_supply,
            circulating_supply: row.circulating_supply,
            metadata: row.metadata.unwrap_or(serde_json::json!({})),
            created_at: row.created_at,
        })
    }

    /// List all assets
    pub async fn list_assets(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<AssetDefinition>> {
        let rows = sqlx::query!(
            r#"
            SELECT asset_id, name, symbol, asset_type, total_supply, circulating_supply, metadata, created_at
            FROM asset_definitions
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset
        )
        .fetch_all(&self.db)
        .await
        .map_err(|e| EnterpriseError::from_db_error(e))?;

        let assets = rows
            .into_iter()
            .map(|row| {
                let asset_type: AssetType = serde_json::from_value(
                    serde_json::Value::String(row.asset_type)
                ).unwrap_or(AssetType::Native);

                AssetDefinition {
                    asset_id: row.asset_id,
                    asset_type,
                    symbol: row.symbol,
                    name: row.name,
                    total_supply: row.total_supply,
                    circulating_supply: row.circulating_supply,
                    metadata: row.metadata.unwrap_or(serde_json::json!({})),
                    created_at: row.created_at,
                }
            })
            .collect();

        Ok(assets)
    }

    /// Get asset positions for a wallet
    pub async fn get_positions(&self, wallet_id: Uuid) -> Result<Vec<AssetPosition>> {
        let rows = sqlx::query!(
            r#"
            SELECT position_id, wallet_id, asset_id, quantity, average_cost
            FROM positions
            WHERE wallet_id = $1
            "#,
            wallet_id
        )
        .fetch_all(&self.db)
        .await
        .map_err(|e| EnterpriseError::from_db_error(e))?;

        let positions = rows
            .into_iter()
            .map(|row| AssetPosition {
                position_id: row.position_id,
                wallet_id: row.wallet_id,
                asset_id: row.asset_id,
                quantity: row.quantity,
                average_cost: row.average_cost,
            })
            .collect();

        Ok(positions)
    }

    /// Create market order
    pub async fn create_order(
        &self,
        wallet_id: Uuid,
        asset_id: Uuid,
        order_type: OrderType,
        quantity: i64,
        price: i64,
    ) -> Result<MarketOrder> {
        // Verify asset exists
        self.get_asset(asset_id).await?;

        // For sell orders, verify sufficient balance
        if order_type == OrderType::Sell {
            let positions = self.get_positions(wallet_id).await?;
            let position = positions.iter().find(|p| p.asset_id == asset_id);

            if let Some(pos) = position {
                if pos.quantity < quantity {
                    return Err(EnterpriseError::InsufficientBalance);
                }
            } else {
                return Err(EnterpriseError::InsufficientBalance);
            }
        }

        // Create order
        let order_id = Uuid::new_v4();
        let status = OrderStatus::Open;

        sqlx::query!(
            r#"
            INSERT INTO market_orders
            (order_id, wallet_id, asset_id, order_type, quantity, price, filled_quantity, status, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
            order_id,
            wallet_id,
            asset_id,
            format!("{:?}", order_type),
            quantity,
            price,
            0i64,
            format!("{:?}", status),
            Utc::now(),
            Utc::now()
        )
        .execute(&self.db)
        .await
        .map_err(|e| EnterpriseError::from_db_error(e))?;

        let order = MarketOrder {
            order_id,
            wallet_id,
            asset_id,
            order_type,
            quantity,
            price,
            filled_quantity: 0,
            status,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // Try to match orders
        self.match_orders(asset_id).await?;

        Ok(order)
    }

    /// Get order by ID
    pub async fn get_order(&self, order_id: Uuid) -> Result<MarketOrder> {
        let row = sqlx::query!(
            r#"
            SELECT order_id, wallet_id, asset_id, order_type, quantity, price, filled_quantity, status, created_at, updated_at
            FROM market_orders
            WHERE order_id = $1
            "#,
            order_id
        )
        .fetch_optional(&self.db)
        .await
        .map_err(|e| EnterpriseError::from_db_error(e))?
        .ok_or_else(|| EnterpriseError::OrderNotFound(order_id.to_string()))?;

        let order_type = if row.order_type == "Buy" { OrderType::Buy } else { OrderType::Sell };
        let status = match row.status.as_str() {
            "Open" => OrderStatus::Open,
            "PartiallyFilled" => OrderStatus::PartiallyFilled,
            "Filled" => OrderStatus::Filled,
            "Cancelled" => OrderStatus::Cancelled,
            _ => OrderStatus::Open,
        };

        Ok(MarketOrder {
            order_id: row.order_id,
            wallet_id: row.wallet_id,
            asset_id: row.asset_id,
            order_type,
            quantity: row.quantity,
            price: row.price,
            filled_quantity: row.filled_quantity,
            status,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }

    /// Get orders for wallet
    pub async fn get_wallet_orders(
        &self,
        wallet_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<MarketOrder>> {
        let rows = sqlx::query!(
            r#"
            SELECT order_id, wallet_id, asset_id, order_type, quantity, price, filled_quantity, status, created_at, updated_at
            FROM market_orders
            WHERE wallet_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            wallet_id,
            limit,
            offset
        )
        .fetch_all(&self.db)
        .await
        .map_err(|e| EnterpriseError::from_db_error(e))?;

        let orders = rows
            .into_iter()
            .map(|row| {
                let order_type = if row.order_type == "Buy" { OrderType::Buy } else { OrderType::Sell };
                let status = match row.status.as_str() {
                    "Open" => OrderStatus::Open,
                    "PartiallyFilled" => OrderStatus::PartiallyFilled,
                    "Filled" => OrderStatus::Filled,
                    "Cancelled" => OrderStatus::Cancelled,
                    _ => OrderStatus::Open,
                };

                MarketOrder {
                    order_id: row.order_id,
                    wallet_id: row.wallet_id,
                    asset_id: row.asset_id,
                    order_type,
                    quantity: row.quantity,
                    price: row.price,
                    filled_quantity: row.filled_quantity,
                    status,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                }
            })
            .collect();

        Ok(orders)
    }

    /// Cancel order
    pub async fn cancel_order(&self, order_id: Uuid) -> Result<()> {
        let result = sqlx::query!(
            "UPDATE market_orders SET status = $1, updated_at = $2 WHERE order_id = $3 AND status = $4",
            "Cancelled",
            Utc::now(),
            order_id,
            "Open"
        )
        .execute(&self.db)
        .await
        .map_err(|e| EnterpriseError::from_db_error(e))?;

        if result.rows_affected() == 0 {
            return Err(EnterpriseError::OrderNotFound(order_id.to_string()));
        }

        Ok(())
    }

    /// Get orderbook for an asset
    pub async fn get_orderbook(&self, asset_id: Uuid) -> Result<OrderBook> {
        // Get all open buy orders sorted by price descending
        let buy_orders = sqlx::query!(
            r#"
            SELECT order_id, wallet_id, quantity, price, filled_quantity
            FROM market_orders
            WHERE asset_id = $1 AND order_type = 'Buy' AND status = 'Open'
            ORDER BY price DESC, created_at ASC
            LIMIT 50
            "#,
            asset_id
        )
        .fetch_all(&self.db)
        .await
        .map_err(|e| EnterpriseError::from_db_error(e))?;

        // Get all open sell orders sorted by price ascending
        let sell_orders = sqlx::query!(
            r#"
            SELECT order_id, wallet_id, quantity, price, filled_quantity
            FROM market_orders
            WHERE asset_id = $1 AND order_type = 'Sell' AND status = 'Open'
            ORDER BY price ASC, created_at ASC
            LIMIT 50
            "#,
            asset_id
        )
        .fetch_all(&self.db)
        .await
        .map_err(|e| EnterpriseError::from_db_error(e))?;

        Ok(OrderBook {
            asset_id,
            bids: buy_orders.into_iter().map(|row| (row.price, row.quantity - row.filled_quantity)).collect(),
            asks: sell_orders.into_iter().map(|row| (row.price, row.quantity - row.filled_quantity)).collect(),
        })
    }

    /// Match orders (internal matching engine)
    pub async fn match_orders(&self, asset_id: Uuid) -> Result<Vec<MarketOrder>> {
        // Get best buy order (highest price)
        let best_buy = sqlx::query!(
            r#"
            SELECT order_id, wallet_id, quantity, price, filled_quantity
            FROM market_orders
            WHERE asset_id = $1 AND order_type = 'Buy' AND status = 'Open'
            ORDER BY price DESC, created_at ASC
            LIMIT 1
            "#,
            asset_id
        )
        .fetch_optional(&self.db)
        .await
        .map_err(|e| EnterpriseError::from_db_error(e))?;

        // Get best sell order (lowest price)
        let best_sell = sqlx::query!(
            r#"
            SELECT order_id, wallet_id, quantity, price, filled_quantity
            FROM market_orders
            WHERE asset_id = $1 AND order_type = 'Sell' AND status = 'Open'
            ORDER BY price ASC, created_at ASC
            LIMIT 1
            "#,
            asset_id
        )
        .fetch_optional(&self.db)
        .await
        .map_err(|e| EnterpriseError::from_db_error(e))?;

        let mut matched_orders = vec![];

        // If prices overlap, execute trade
        if let (Some(buy), Some(sell)) = (best_buy, best_sell) {
            if buy.price >= sell.price {
                // Calculate trade quantity
                let buy_remaining = buy.quantity - buy.filled_quantity;
                let sell_remaining = sell.quantity - sell.filled_quantity;
                let trade_quantity = buy_remaining.min(sell_remaining);
                let trade_price = sell.price; // Taker pays maker's price

                // Execute trade on chain (simplified)
                self.execute_trade(buy.order_id, sell.order_id, trade_quantity, trade_price).await?;

                // Update filled quantities
                self.update_order_fill(buy.order_id, buy.filled_quantity + trade_quantity).await?;
                self.update_order_fill(sell.order_id, sell.filled_quantity + trade_quantity).await?;

                // Get updated orders
                matched_orders.push(self.get_order(buy.order_id).await?);
                matched_orders.push(self.get_order(sell.order_id).await?);
            }
        }

        Ok(matched_orders)
    }

    /// Execute trade (settle on Boundless chain)
    /// FIX: Implemented blockchain integration for asset transfers
    async fn execute_trade(
        &self,
        buy_order_id: Uuid,
        sell_order_id: Uuid,
        quantity: i64,
        price: i64,
    ) -> Result<String> {
        let buy_order = self.get_order(buy_order_id).await?;
        let sell_order = self.get_order(sell_order_id).await?;

        // Get wallet addresses for buyer and seller
        let buyer_address = self.get_wallet_address(buy_order.wallet_id).await?;
        let seller_address = self.get_wallet_address(sell_order.wallet_id).await?;

        // Create blockchain transaction with asset transfer metadata
        // Asset transfers are encoded in the transaction data field
        let asset_transfer_data = serde_json::json!({
            "type": "asset_transfer",
            "asset_id": buy_order.asset_id.to_string(),
            "from_wallet": sell_order.wallet_id.to_string(),
            "to_wallet": buy_order.wallet_id.to_string(),
            "from_address": seller_address,
            "to_address": buyer_address,
            "quantity": quantity,
            "price": price,
            "buy_order_id": buy_order_id.to_string(),
            "sell_order_id": sell_order_id.to_string(),
            "timestamp": Utc::now().timestamp(),
        });

        // Encode as bytes for transaction data field
        let data_bytes = serde_json::to_vec(&asset_transfer_data)
            .map_err(|e| EnterpriseError::ValidationError(format!("Failed to encode asset transfer data: {}", e)))?;

        // Submit asset transfer metadata transaction to blockchain
        // This creates an immutable record of the asset transfer on-chain
        let tx_hash = self.submit_asset_transfer_to_blockchain(
            &seller_address,
            &buyer_address,
            data_bytes,
        ).await?;

        // Update buyer's position (add assets)
        self.update_position(buy_order.wallet_id, buy_order.asset_id, quantity, price).await?;

        // Update seller's position (subtract assets)
        self.update_position(sell_order.wallet_id, sell_order.asset_id, -quantity, price).await?;

        // Log successful blockchain settlement
        tracing::info!(
            "Asset transfer settled on blockchain: tx_hash={}, asset_id={}, from={}, to={}, quantity={}",
            tx_hash, buy_order.asset_id, seller_address, buyer_address, quantity
        );

        Ok(tx_hash)
    }

    /// Update order fill status
    async fn update_order_fill(&self, order_id: Uuid, new_filled: i64) -> Result<()> {
        let order = self.get_order(order_id).await?;

        let new_status = if new_filled >= order.quantity {
            "Filled"
        } else if new_filled > 0 {
            "PartiallyFilled"
        } else {
            "Open"
        };

        sqlx::query!(
            "UPDATE market_orders SET filled_quantity = $1, status = $2, updated_at = $3 WHERE order_id = $4",
            new_filled,
            new_status,
            Utc::now(),
            order_id
        )
        .execute(&self.db)
        .await
        .map_err(|e| EnterpriseError::from_db_error(e))?;

        Ok(())
    }

    /// Update asset position
    async fn update_position(
        &self,
        wallet_id: Uuid,
        asset_id: Uuid,
        quantity_delta: i64,
        price: i64,
    ) -> Result<()> {
        // Check if position exists
        let existing = sqlx::query!(
            "SELECT position_id, quantity, average_cost FROM positions WHERE wallet_id = $1 AND asset_id = $2",
            wallet_id,
            asset_id
        )
        .fetch_optional(&self.db)
        .await
        .map_err(|e| EnterpriseError::from_db_error(e))?;

        if let Some(pos) = existing {
            // Update existing position
            let new_quantity = pos.quantity + quantity_delta;
            let new_avg_cost = if quantity_delta > 0 {
                // Buying - update average cost
                ((pos.average_cost * pos.quantity) + (price * quantity_delta)) / new_quantity
            } else {
                // Selling - keep same average cost
                pos.average_cost
            };

            sqlx::query!(
                "UPDATE positions SET quantity = $1, average_cost = $2 WHERE position_id = $3",
                new_quantity,
                new_avg_cost,
                pos.position_id
            )
            .execute(&self.db)
            .await
            .map_err(|e| EnterpriseError::from_db_error(e))?;
        } else {
            // Create new position
            let position_id = Uuid::new_v4();
            sqlx::query!(
                r#"
                INSERT INTO positions
                (position_id, wallet_id, asset_id, quantity, average_cost)
                VALUES ($1, $2, $3, $4, $5)
                "#,
                position_id,
                wallet_id,
                asset_id,
                quantity_delta,
                price
            )
            .execute(&self.db)
            .await
            .map_err(|e| EnterpriseError::from_db_error(e))?;
        }

        Ok(())
    }

    /// Issue asset to a wallet (mint new tokens)
    pub async fn issue_asset(
        &self,
        asset_id: Uuid,
        to_wallet: Uuid,
        amount: u64,
    ) -> Result<()> {
        // Verify asset exists
        let asset = self.get_asset(asset_id).await?;

        // Convert u64 to i64
        let amount_i64: i64 = amount.try_into()
            .map_err(|_| EnterpriseError::InvalidInput("Amount too large".to_string()))?;

        // Update circulating supply
        sqlx::query!(
            "UPDATE asset_definitions SET circulating_supply = circulating_supply + $1 WHERE asset_id = $2",
            amount_i64,
            asset_id
        )
        .execute(&self.db)
        .await
        .map_err(|e| EnterpriseError::from_db_error(e))?;

        // Add to wallet balance
        self.update_position(to_wallet, asset_id, amount_i64, 0).await?;

        Ok(())
    }

    /// Transfer asset between wallets
    pub async fn transfer_asset(
        &self,
        asset_id: Uuid,
        from_wallet: Uuid,
        to_wallet: Uuid,
        amount: u64,
    ) -> Result<()> {
        // Verify asset exists
        self.get_asset(asset_id).await?;

        // Convert u64 to i64
        let amount_i64: i64 = amount.try_into()
            .map_err(|_| EnterpriseError::InvalidInput("Amount too large".to_string()))?;

        // Verify from_wallet has sufficient balance
        let from_position = sqlx::query!(
            "SELECT quantity FROM positions WHERE wallet_id = $1 AND asset_id = $2",
            from_wallet,
            asset_id
        )
        .fetch_optional(&self.db)
        .await
        .map_err(|e| EnterpriseError::from_db_error(e))?
        .ok_or(EnterpriseError::InsufficientBalance)?;

        if from_position.quantity < amount_i64 {
            return Err(EnterpriseError::InsufficientBalance);
        }

        // Subtract from sender
        self.update_position(from_wallet, asset_id, -amount_i64, 0).await?;

        // Add to receiver
        self.update_position(to_wallet, asset_id, amount_i64, 0).await?;

        // Anchor transfer to blockchain
        match self.anchor_transfer_to_chain(asset_id, from_wallet, to_wallet, amount).await {
            Ok(tx_hash) => {
                tracing::info!("Asset transfer anchored to blockchain: {}", tx_hash);
            }
            Err(e) => {
                // Log error but don't fail the transfer - the database state is already updated
                tracing::warn!("Failed to anchor transfer to blockchain: {}", e);
            }
        }

        Ok(())
    }

    /// Anchor an asset transfer to the Boundless blockchain
    async fn anchor_transfer_to_chain(
        &self,
        asset_id: Uuid,
        from_wallet: Uuid,
        to_wallet: Uuid,
        amount: u64,
    ) -> Result<String> {
        // Create transfer proof data
        let proof_data = format!(
            "TRANSFER:{}:{}:{}:{}",
            asset_id, from_wallet, to_wallet, amount
        );

        // Hash the proof data using SHA3-256
        use sha3::{Digest, Sha3_256};
        let mut hasher = Sha3_256::new();
        hasher.update(proof_data.as_bytes());
        let proof_hash = hasher.finalize();
        let proof_hash_hex = hex::encode(proof_hash);

        // Convert from_wallet ID to 32-byte array for blockchain
        let from_wallet_bytes = from_wallet.as_bytes();
        let mut identity_hash = [0u8; 32];
        let mut id_hasher = Sha3_256::new();
        id_hasher.update(from_wallet_bytes);
        identity_hash.copy_from_slice(&id_hasher.finalize());

        // Create metadata with transfer details
        let metadata = serde_json::json!({
            "asset_id": asset_id.to_string(),
            "from_wallet": from_wallet.to_string(),
            "to_wallet": to_wallet.to_string(),
            "amount": amount,
        });

        // Create anchor proof request
        let request = serde_json::json!({
            "identity_id": hex::encode(identity_hash),
            "proof_type": "asset_ownership",
            "proof_hash": proof_hash_hex,
            "metadata": metadata,
        });

        // Submit to blockchain RPC
        let url = format!("{}/api/v1/proof/anchor", self.blockchain_rpc_url);
        let response = self
            .http_client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                EnterpriseError::Internal(format!("Failed to submit proof to blockchain: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(EnterpriseError::Internal(format!(
                "Blockchain RPC error {}: {}",
                status, error_text
            )));
        }

        // Parse response
        let response_data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| EnterpriseError::Internal(format!("Failed to parse RPC response: {}", e)))?;

        let tx_hash = response_data
            .get("tx_hash")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                EnterpriseError::Internal("Missing tx_hash in RPC response".to_string())
            })?
            .to_string();

        Ok(tx_hash)
    }

    /// Get asset balance for a wallet
    pub async fn get_balance(
        &self,
        wallet_id: Uuid,
        asset_id: Uuid,
    ) -> Result<AssetBalance> {
        // Get or create balance record
        let position = sqlx::query!(
            r#"
            SELECT position_id, wallet_id, asset_id, quantity, locked_quantity
            FROM positions
            WHERE wallet_id = $1 AND asset_id = $2
            "#,
            wallet_id,
            asset_id
        )
        .fetch_optional(&self.db)
        .await
        .map_err(|e| EnterpriseError::from_db_error(e))?;

        if let Some(pos) = position {
            Ok(AssetBalance {
                balance_id: pos.position_id,
                wallet_id: pos.wallet_id,
                asset_id: pos.asset_id,
                total_quantity: pos.quantity,
                locked_quantity: pos.locked_quantity,
            })
        } else {
            // Return zero balance if position doesn't exist
            Ok(AssetBalance {
                balance_id: Uuid::new_v4(),
                wallet_id,
                asset_id,
                total_quantity: 0,
                locked_quantity: 0,
            })
        }
    }

    /// Get trade history for an asset
    pub async fn get_trades(
        &self,
        asset_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Trade>> {
        let rows = sqlx::query!(
            r#"
            SELECT trade_id, asset_id, buy_order_id, sell_order_id, quantity, price, buyer_wallet_id, seller_wallet_id, created_at
            FROM trades
            WHERE asset_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            asset_id,
            limit,
            offset
        )
        .fetch_all(&self.db)
        .await
        .map_err(|e| EnterpriseError::from_db_error(e))?;

        let trades = rows
            .into_iter()
            .map(|row| Trade {
                trade_id: row.trade_id,
                asset_id: row.asset_id,
                buy_order_id: row.buy_order_id,
                sell_order_id: row.sell_order_id,
                quantity: row.quantity,
                price: row.price,
                buyer_wallet_id: row.buyer_wallet_id,
                seller_wallet_id: row.seller_wallet_id,
                created_at: row.created_at,
            })
            .collect();

        Ok(trades)
    }

    /// Get wallet address for blockchain transactions
    /// Helper function to get the first Boundless address for a wallet
    async fn get_wallet_address(&self, wallet_id: Uuid) -> Result<String> {
        let row = sqlx::query!(
            "SELECT blockchain_address FROM wallet_keys WHERE wallet_id = $1 ORDER BY created_at ASC LIMIT 1",
            wallet_id
        )
        .fetch_optional(&self.db)
        .await
        .map_err(|e| EnterpriseError::from_db_error(e))?
        .ok_or_else(|| EnterpriseError::WalletNotFound(wallet_id.to_string()))?;

        Ok(row.blockchain_address)
    }

    /// Submit asset transfer transaction to Boundless blockchain
    /// Creates an on-chain record of the asset transfer in the transaction data field
    async fn submit_asset_transfer_to_blockchain(
        &self,
        from_address: &str,
        to_address: &str,
        data_bytes: Vec<u8>,
    ) -> Result<String> {
        // Create transaction request with asset transfer metadata
        let request = serde_json::json!({
            "from": from_address,
            "to": to_address,
            "amount": 0,  // Asset transfers use metadata, not native token amount
            "data": hex::encode(&data_bytes),
        });

        // Submit to blockchain via HTTP REST API
        let url = format!("{}/api/v1/transactions/submit", self.blockchain_rpc_url);
        let response = self
            .http_client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                EnterpriseError::Internal(format!("Failed to submit asset transfer to blockchain: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(EnterpriseError::Internal(format!(
                "Blockchain API error {}: {}",
                status, error_text
            )));
        }

        // Parse response to get transaction hash
        let response_data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| EnterpriseError::Internal(format!("Failed to parse blockchain response: {}", e)))?;

        // Extract transaction hash from response
        let tx_hash = response_data
            .get("tx_hash")
            .or_else(|| response_data.get("transaction_hash"))
            .or_else(|| response_data.get("hash"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| EnterpriseError::Internal("Missing transaction hash in blockchain response".to_string()))?
            .to_string();

        Ok(tx_hash)
    }
}
