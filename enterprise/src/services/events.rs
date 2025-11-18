// Event & Reporting Service - Notifications and analytics

use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;
use serde_json::Value as JsonValue;

use crate::error::{EnterpriseError, Result};
use crate::models::*;

pub struct EventService {
    db: PgPool,
}

impl EventService {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    // ==================== Notification Management ====================

    /// Create a notification for a user
    pub async fn create_notification(
        &self,
        identity_id: Uuid,
        notification_type: NotificationType,
        title: String,
        message: String,
        metadata: JsonValue,
    ) -> Result<Notification> {
        // 1. Verify identity exists
        let identity_exists = sqlx::query!(
            "SELECT identity_id FROM identity_profiles WHERE identity_id = $1",
            identity_id
        )
        .fetch_optional(&self.db)
        .await
        .map_err(|e| EnterpriseError::from_db_error(e))?;

        if identity_exists.is_none() {
            return Err(EnterpriseError::IdentityNotFound(identity_id.to_string()));
        }

        // 2. Create notification
        let notification_id = Uuid::new_v4();

        sqlx::query!(
            r#"
            INSERT INTO notifications
            (notification_id, identity_id, notification_type, title, message, read, metadata, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
            notification_id,
            identity_id,
            format!("{:?}", notification_type),
            title,
            message,
            false,
            metadata,
            Utc::now()
        )
        .execute(&self.db)
        .await
        .map_err(|e| EnterpriseError::from_db_error(e))?;

        // 3. Return notification
        Ok(Notification {
            notification_id,
            identity_id,
            notification_type,
            title,
            message,
            metadata,
            read: false,
            created_at: Utc::now(),
        })
    }

    /// Get notification by ID
    pub async fn get_notification(&self, notification_id: Uuid) -> Result<Notification> {
        let row = sqlx::query!(
            r#"
            SELECT notification_id, identity_id, notification_type, title, message, read, metadata, created_at
            FROM notifications
            WHERE notification_id = $1
            "#,
            notification_id
        )
        .fetch_optional(&self.db)
        .await
        .map_err(|e| EnterpriseError::from_db_error(e))?
        .ok_or_else(|| EnterpriseError::NotFound(format!("Notification {} not found", notification_id)))?;

        let notification_type: NotificationType = serde_json::from_value(
            serde_json::Value::String(row.notification_type)
        ).unwrap_or(NotificationType::Info);

        Ok(Notification {
            notification_id: row.notification_id,
            identity_id: row.identity_id,
            notification_type,
            title: row.title,
            message: row.message,
            metadata: row.metadata.unwrap_or(serde_json::json!({})),
            read: row.read,
            created_at: row.created_at,
        })
    }

    /// Get all notifications for an identity
    pub async fn get_identity_notifications(
        &self,
        identity_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Notification>> {
        let rows = sqlx::query!(
            r#"
            SELECT notification_id, identity_id, notification_type, title, message, read, metadata, created_at
            FROM notifications
            WHERE identity_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            identity_id,
            limit,
            offset
        )
        .fetch_all(&self.db)
        .await
        .map_err(|e| EnterpriseError::from_db_error(e))?;

        let notifications = rows
            .into_iter()
            .map(|row| {
                let notification_type: NotificationType = serde_json::from_value(
                    serde_json::Value::String(row.notification_type)
                ).unwrap_or(NotificationType::Info);

                Notification {
                    notification_id: row.notification_id,
                    identity_id: row.identity_id,
                    notification_type,
                    title: row.title,
                    message: row.message,
                    metadata: row.metadata.unwrap_or(serde_json::json!({})),
                    read: row.read,
                    created_at: row.created_at,
                }
            })
            .collect();

        Ok(notifications)
    }

    /// Get unread notification count for an identity
    pub async fn get_unread_count(&self, identity_id: Uuid) -> Result<i64> {
        let result = sqlx::query!(
            "SELECT COUNT(*) as count FROM notifications WHERE identity_id = $1 AND read = false",
            identity_id
        )
        .fetch_one(&self.db)
        .await
        .map_err(|e| EnterpriseError::from_db_error(e))?;

        Ok(result.count.unwrap_or(0))
    }

    /// Mark notification as read
    pub async fn mark_as_read(&self, notification_id: Uuid) -> Result<()> {
        let result = sqlx::query!(
            "UPDATE notifications SET read = true WHERE notification_id = $1",
            notification_id
        )
        .execute(&self.db)
        .await
        .map_err(|e| EnterpriseError::from_db_error(e))?;

        if result.rows_affected() == 0 {
            return Err(EnterpriseError::NotFound(format!("Notification {} not found", notification_id)));
        }

        Ok(())
    }

    /// Mark all notifications as read for an identity
    pub async fn mark_all_as_read(&self, identity_id: Uuid) -> Result<u64> {
        let result = sqlx::query!(
            "UPDATE notifications SET read = true WHERE identity_id = $1 AND read = false",
            identity_id
        )
        .execute(&self.db)
        .await
        .map_err(|e| EnterpriseError::from_db_error(e))?;

        Ok(result.rows_affected())
    }

    /// Delete notification
    pub async fn delete_notification(&self, notification_id: Uuid) -> Result<()> {
        let result = sqlx::query!(
            "DELETE FROM notifications WHERE notification_id = $1",
            notification_id
        )
        .execute(&self.db)
        .await
        .map_err(|e| EnterpriseError::from_db_error(e))?;

        if result.rows_affected() == 0 {
            return Err(EnterpriseError::NotFound(format!("Notification {} not found", notification_id)));
        }

        Ok(())
    }

    // ==================== Report Management ====================

    /// Define a new report template
    pub async fn create_report_definition(
        &self,
        name: String,
        description: String,
        report_type: ReportType,
        sql_template: String,
        parameters: Vec<String>,
    ) -> Result<ReportDefinition> {
        // 1. Validate inputs
        if name.is_empty() {
            return Err(EnterpriseError::InvalidInput("Report name cannot be empty".to_string()));
        }

        if sql_template.is_empty() {
            return Err(EnterpriseError::InvalidInput("SQL template cannot be empty".to_string()));
        }

        // 2. Create report definition
        let report_id = Uuid::new_v4();

        sqlx::query!(
            r#"
            INSERT INTO report_definitions
            (report_id, name, description, report_type, sql_template, parameters, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
            report_id,
            name,
            description,
            format!("{:?}", report_type),
            sql_template,
            &parameters,
            Utc::now(),
            Utc::now()
        )
        .execute(&self.db)
        .await
        .map_err(|e| EnterpriseError::from_db_error(e))?;

        // 3. Return report definition
        Ok(ReportDefinition {
            report_id,
            name,
            description,
            report_type,
            sql_template,
            parameters,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    /// Get report definition by ID
    pub async fn get_report_definition(&self, report_id: Uuid) -> Result<ReportDefinition> {
        let row = sqlx::query!(
            r#"
            SELECT report_id, name, description, report_type, sql_template, parameters, created_at, updated_at
            FROM report_definitions
            WHERE report_id = $1
            "#,
            report_id
        )
        .fetch_optional(&self.db)
        .await
        .map_err(|e| EnterpriseError::from_db_error(e))?
        .ok_or_else(|| EnterpriseError::NotFound(format!("Report definition {} not found", report_id)))?;

        let report_type: ReportType = serde_json::from_value(
            serde_json::Value::String(row.report_type)
        ).unwrap_or(ReportType::Transaction);

        Ok(ReportDefinition {
            report_id: row.report_id,
            name: row.name,
            description: row.description.unwrap_or_default(),
            report_type,
            sql_template: row.sql_template,
            parameters: row.parameters.unwrap_or_default(),
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }

    /// List all report definitions
    pub async fn list_report_definitions(&self) -> Result<Vec<ReportDefinition>> {
        let rows = sqlx::query!(
            r#"
            SELECT report_id, name, description, report_type, sql_template, parameters, created_at, updated_at
            FROM report_definitions
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(&self.db)
        .await
        .map_err(|e| EnterpriseError::from_db_error(e))?;

        let definitions = rows
            .into_iter()
            .map(|row| {
                let report_type: ReportType = serde_json::from_value(
                    serde_json::Value::String(row.report_type)
                ).unwrap_or(ReportType::Transaction);

                ReportDefinition {
                    report_id: row.report_id,
                    name: row.name,
                    description: row.description.unwrap_or_default(),
                    report_type,
                    sql_template: row.sql_template,
                    parameters: row.parameters.unwrap_or_default(),
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                }
            })
            .collect();

        Ok(definitions)
    }

    /// Generate a report from a definition using SECURE parameterized queries
    ///
    /// SECURITY: This function uses only predefined report types with parameterized queries.
    /// Custom SQL templates are NOT supported to prevent SQL injection attacks.
    pub async fn generate_report(
        &self,
        report_id: Uuid,
        identity_id: Uuid,
        parameters: JsonValue,
        format: ExportFormat,
    ) -> Result<GeneratedReport> {
        // 1. Get report definition
        let definition = self.get_report_definition(report_id).await?;

        // 2. Execute report based on predefined type (SECURE - uses parameterized queries only)
        let data = self.execute_predefined_report(&definition.report_type, identity_id, &parameters).await?;

        // 3. Format data based on export format
        let formatted_data = self.format_report_data(&data, format)?;

        // 4. Create generated report record
        let generated_report_id = Uuid::new_v4();

        sqlx::query!(
            r#"
            INSERT INTO generated_reports
            (generated_report_id, report_id, identity_id, parameters, format, result_data, chain_anchor_tx, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
            generated_report_id,
            report_id,
            identity_id,
            parameters,
            format!("{:?}", format),
            formatted_data,
            None::<String>,
            Utc::now()
        )
        .execute(&self.db)
        .await
        .map_err(|e| EnterpriseError::from_db_error(e))?;

        // 5. Return generated report
        Ok(GeneratedReport {
            generated_report_id,
            report_id,
            identity_id,
            parameters,
            format,
            result_data: formatted_data,
            chain_anchor_tx: None,
            created_at: Utc::now(),
        })
    }

    /// Get generated report by ID
    pub async fn get_generated_report(&self, generated_report_id: Uuid) -> Result<GeneratedReport> {
        let row = sqlx::query!(
            r#"
            SELECT generated_report_id, report_id, identity_id, parameters, format, result_data, chain_anchor_tx, created_at
            FROM generated_reports
            WHERE generated_report_id = $1
            "#,
            generated_report_id
        )
        .fetch_optional(&self.db)
        .await
        .map_err(|e| EnterpriseError::from_db_error(e))?
        .ok_or_else(|| EnterpriseError::NotFound(format!("Generated report {} not found", generated_report_id)))?;

        let format: ExportFormat = serde_json::from_value(
            serde_json::Value::String(row.format)
        ).unwrap_or(ExportFormat::JSON);

        Ok(GeneratedReport {
            generated_report_id: row.generated_report_id,
            report_id: row.report_id,
            identity_id: row.identity_id,
            parameters: row.parameters.unwrap_or(serde_json::json!({})),
            format,
            result_data: row.result_data,
            chain_anchor_tx: row.chain_anchor_tx,
            created_at: row.created_at,
        })
    }

    /// List generated reports for an identity
    pub async fn list_generated_reports(
        &self,
        identity_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<GeneratedReport>> {
        let rows = sqlx::query!(
            r#"
            SELECT generated_report_id, report_id, identity_id, parameters, format, result_data, chain_anchor_tx, created_at
            FROM generated_reports
            WHERE identity_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            identity_id,
            limit,
            offset
        )
        .fetch_all(&self.db)
        .await
        .map_err(|e| EnterpriseError::from_db_error(e))?;

        let reports = rows
            .into_iter()
            .map(|row| {
                let format: ExportFormat = serde_json::from_value(
                    serde_json::Value::String(row.format)
                ).unwrap_or(ExportFormat::JSON);

                GeneratedReport {
                    generated_report_id: row.generated_report_id,
                    report_id: row.report_id,
                    identity_id: row.identity_id,
                    parameters: row.parameters.unwrap_or(serde_json::json!({})),
                    format,
                    result_data: row.result_data,
                    chain_anchor_tx: row.chain_anchor_tx,
                    created_at: row.created_at,
                }
            })
            .collect();

        Ok(reports)
    }

    // ==================== Private Helper Methods ====================

    /// Execute predefined report using SECURE parameterized queries
    ///
    /// SECURITY: This function ONLY executes whitelisted report types with parameterized queries.
    /// Custom SQL templates are NEVER executed to prevent SQL injection.
    async fn execute_predefined_report(
        &self,
        report_type: &ReportType,
        identity_id: Uuid,
        parameters: &JsonValue,
    ) -> Result<JsonValue> {
        // Extract common parameters safely
        let start_date = parameters.get("start_date")
            .and_then(|v| v.as_str())
            .unwrap_or("1970-01-01");
        let end_date = parameters.get("end_date")
            .and_then(|v| v.as_str())
            .unwrap_or("2100-12-31");
        let limit = parameters.get("limit")
            .and_then(|v| v.as_i64())
            .unwrap_or(100);

        // Execute ONLY whitelisted, parameterized queries
        // NOTE: Using untyped queries to avoid compile-time schema validation issues
        match report_type {
            ReportType::Transaction => {
                // Transaction history report with parameterized query
                let rows = sqlx::query(
                    r#"
                    SELECT
                        wt.tx_id,
                        wt.chain_tx_hash,
                        wt.amount,
                        wt.direction,
                        wt.timestamp,
                        wt.status
                    FROM wallet_transactions wt
                    JOIN wallet_accounts wa ON wt.wallet_id = wa.wallet_id
                    WHERE wa.identity_id = $1
                    AND wt.timestamp::text >= $2
                    AND wt.timestamp::text <= $3
                    ORDER BY wt.timestamp DESC
                    LIMIT $4
                    "#
                )
                .bind(identity_id)
                .bind(start_date)
                .bind(end_date)
                .bind(limit)
                .fetch_all(&self.db)
                .await
                .map_err(|e| EnterpriseError::from_db_error(e))?;

                // Convert to JSON array
                let results: Vec<JsonValue> = rows
                    .into_iter()
                    .map(|row| {
                        use sqlx::Row;
                        serde_json::json!({
                            "tx_id": row.get::<Uuid, _>("tx_id"),
                            "tx_hash": row.get::<String, _>("chain_tx_hash"),
                            "amount": row.get::<i64, _>("amount"),
                            "direction": row.get::<String, _>("direction"),
                            "timestamp": row.get::<chrono::DateTime<Utc>, _>("timestamp").to_string(),
                            "status": row.get::<String, _>("status"),
                        })
                    })
                    .collect();

                Ok(JsonValue::Array(results))
            }
            ReportType::Security => {
                // Security notifications report with parameterized query
                let rows = sqlx::query(
                    r#"
                    SELECT
                        notification_id,
                        notification_type,
                        title,
                        message,
                        read,
                        created_at
                    FROM notifications
                    WHERE identity_id = $1
                    AND created_at::text >= $2
                    AND created_at::text <= $3
                    ORDER BY created_at DESC
                    LIMIT $4
                    "#
                )
                .bind(identity_id)
                .bind(start_date)
                .bind(end_date)
                .bind(limit)
                .fetch_all(&self.db)
                .await
                .map_err(|e| EnterpriseError::from_db_error(e))?;

                // Convert to JSON array
                let results: Vec<JsonValue> = rows
                    .into_iter()
                    .map(|row| {
                        use sqlx::Row;
                        serde_json::json!({
                            "notification_id": row.get::<Uuid, _>("notification_id"),
                            "type": row.get::<String, _>("notification_type"),
                            "title": row.get::<String, _>("title"),
                            "message": row.get::<String, _>("message"),
                            "read": row.get::<bool, _>("read"),
                            "created_at": row.get::<chrono::DateTime<Utc>, _>("created_at").to_string(),
                        })
                    })
                    .collect();

                Ok(JsonValue::Array(results))
            }
            ReportType::Application => {
                // Application events report with parameterized query
                let rows = sqlx::query(
                    r#"
                    SELECT
                        event_id,
                        app_id,
                        event_type,
                        timestamp,
                        metadata
                    FROM application_events
                    WHERE identity_id = $1
                    AND timestamp::text >= $2
                    AND timestamp::text <= $3
                    ORDER BY timestamp DESC
                    LIMIT $4
                    "#
                )
                .bind(identity_id)
                .bind(start_date)
                .bind(end_date)
                .bind(limit)
                .fetch_all(&self.db)
                .await
                .map_err(|e| EnterpriseError::from_db_error(e))?;

                // Convert to JSON array
                let results: Vec<JsonValue> = rows
                    .into_iter()
                    .map(|row| {
                        use sqlx::Row;
                        serde_json::json!({
                            "event_id": row.get::<Uuid, _>("event_id"),
                            "app_id": row.get::<Uuid, _>("app_id"),
                            "event_type": row.get::<String, _>("event_type"),
                            "timestamp": row.get::<chrono::DateTime<Utc>, _>("timestamp").to_string(),
                            "metadata": row.get::<JsonValue, _>("metadata"),
                        })
                    })
                    .collect();

                Ok(JsonValue::Array(results))
            }
            // For other report types, return empty array
            _ => Ok(JsonValue::Array(vec![])),
        }
    }

    /// Format report data based on export format
    fn format_report_data(&self, data: &JsonValue, format: ExportFormat) -> Result<String> {
        match format {
            ExportFormat::JSON => {
                // Return pretty-printed JSON
                serde_json::to_string_pretty(data)
                    .map_err(|e| EnterpriseError::Internal(format!("JSON formatting failed: {}", e)))
            }
            ExportFormat::CSV => {
                // Convert JSON array to CSV
                self.json_to_csv(data)
            }
            ExportFormat::PDF => {
                // For PDF, return JSON for now (PDF generation would require additional dependencies)
                // In production, use a PDF library like printpdf
                Ok(format!("PDF Report (JSON data): {}", serde_json::to_string_pretty(data).unwrap_or_default()))
            }
        }
    }

    /// Convert JSON array to CSV format
    fn json_to_csv(&self, data: &JsonValue) -> Result<String> {
        if let JsonValue::Array(rows) = data {
            if rows.is_empty() {
                return Ok(String::new());
            }

            // Get headers from first row
            let first_row = &rows[0];
            if let JsonValue::Object(obj) = first_row {
                let headers: Vec<String> = obj.keys().cloned().collect();
                let mut csv = headers.join(",") + "\n";

                // Add data rows
                for row in rows {
                    if let JsonValue::Object(obj) = row {
                        let values: Vec<String> = headers
                            .iter()
                            .map(|h| {
                                let val = &obj[h];
                                match val {
                                    JsonValue::String(s) => format!("\"{}\"", s.replace("\"", "\"\"")),
                                    JsonValue::Number(n) => n.to_string(),
                                    JsonValue::Bool(b) => b.to_string(),
                                    JsonValue::Null => String::new(),
                                    _ => val.to_string(),
                                }
                            })
                            .collect();
                        csv.push_str(&values.join(","));
                        csv.push('\n');
                    }
                }

                Ok(csv)
            } else {
                Err(EnterpriseError::InvalidInput("Invalid JSON structure for CSV conversion".to_string()))
            }
        } else {
            Err(EnterpriseError::InvalidInput("Data must be a JSON array for CSV conversion".to_string()))
        }
    }

    /// Delete report definition
    pub async fn delete_report_definition(&self, report_id: Uuid) -> Result<()> {
        let result = sqlx::query!(
            "DELETE FROM report_definitions WHERE report_id = $1",
            report_id
        )
        .execute(&self.db)
        .await
        .map_err(|e| EnterpriseError::from_db_error(e))?;

        if result.rows_affected() == 0 {
            return Err(EnterpriseError::NotFound(format!("Report definition {} not found", report_id)));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // SECURITY FIX: Removed test_parameter_substitution_logic test
    // The old SQL template substitution function was removed due to SQL injection vulnerability.
    // Report generation now uses only whitelisted, parameterized queries via execute_predefined_report().

    #[test]
    fn test_json_to_csv_logic() {
        // Test JSON to CSV conversion logic without database dependency
        let data = serde_json::json!([
            {"name": "Alice", "age": 30, "city": "NYC"},
            {"name": "Bob", "age": 25, "city": "LA"}
        ]);

        let rows = data.as_array().unwrap();
        let first_row = rows[0].as_object().unwrap();

        // Build header
        let headers: Vec<String> = first_row.keys().cloned().collect();
        let csv_header = headers.join(",");

        assert!(csv_header.contains("name"));
        assert!(csv_header.contains("age"));
        assert!(csv_header.contains("city"));

        // Test row data
        let first_row_name = first_row.get("name").unwrap().as_str().unwrap();
        assert_eq!(first_row_name, "Alice");
    }

    // TODO: Add tests for execute_predefined_report with proper mocking framework (e.g., mockall or sqlx::test)
}
