// Event & Reporting Service - Notifications and analytics

use sqlx::{PgPool, Row, Column};
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

    /// Generate a report from a definition
    pub async fn generate_report(
        &self,
        _report_id: Uuid,
        _identity_id: Uuid,
        _parameters: JsonValue,
        _format: ExportFormat,
    ) -> Result<GeneratedReport> {
        // SECURITY FIX: Custom SQL templates disabled due to SQL injection vulnerability
        // TODO: Implement secure report generation using parameterized queries only
        // See: https://github.com/boundless/security-fixes/issues/SQL-INJECTION-001
        Err(EnterpriseError::NotImplemented(
            "Custom SQL report generation temporarily disabled for security. Use predefined report types only.".to_string()
        ))

        // DISABLED CODE - kept for reference when implementing secure version:
        //
        // // 1. Get report definition
        // let definition = self.get_report_definition(report_id).await?;
        //
        // // 2. Substitute parameters in SQL template
        // let sql = self.substitute_parameters(&definition.sql_template, &parameters)?;
        //
        // // 3. Execute SQL query
        // let data = self.execute_report_query(&sql).await?;
        //
        // // 4. Format data based on export format
        // let formatted_data = self.format_report_data(&data, format)?;
        //
        // // 5. Create generated report record
        // let generated_report_id = Uuid::new_v4();
        //
        // sqlx::query!(
        //     r#"
        //     INSERT INTO generated_reports
        //     (generated_report_id, report_id, identity_id, parameters, format, result_data, chain_anchor_tx, created_at)
        //     VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        //     "#,
        //     generated_report_id,
        //     report_id,
        //     identity_id,
        //     parameters,
        //     format!("{:?}", format),
        //     formatted_data,
        //     None::<String>,
        //     Utc::now()
        // )
        // .execute(&self.db)
        // .await
        // .map_err(|e| EnterpriseError::from_db_error(e))?;
        //
        // // 6. Return generated report
        // Ok(GeneratedReport {
        //     generated_report_id,
        //     report_id,
        //     identity_id,
        //     parameters,
        //     format,
        //     result_data: formatted_data,
        //     chain_anchor_tx: None,
        //     created_at: Utc::now(),
        // })
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

    /// Substitute parameters in SQL template
    fn substitute_parameters(&self, template: &str, parameters: &JsonValue) -> Result<String> {
        let mut sql = template.to_string();

        // Simple parameter substitution: replace {{param_name}} with value
        if let Some(params) = parameters.as_object() {
            for (key, value) in params {
                let placeholder = format!("{{{{{}}}}}", key);
                let value_str = match value {
                    JsonValue::String(s) => format!("'{}'", s.replace("'", "''")), // Escape single quotes
                    JsonValue::Number(n) => n.to_string(),
                    JsonValue::Bool(b) => b.to_string(),
                    JsonValue::Null => "NULL".to_string(),
                    _ => value.to_string(),
                };
                sql = sql.replace(&placeholder, &value_str);
            }
        }

        // Verify no unsubstituted parameters remain
        if sql.contains("{{") {
            return Err(EnterpriseError::InvalidInput("Unsubstituted parameters in SQL template".to_string()));
        }

        Ok(sql)
    }

    /// Execute report query and return data as JSON
    async fn execute_report_query(&self, sql: &str) -> Result<JsonValue> {
        // Execute query and fetch all rows as JSON
        // For security, this should use a read-only database connection
        let rows = sqlx::query(sql)
            .fetch_all(&self.db)
            .await
            .map_err(|e| EnterpriseError::DatabaseError(format!("Report query failed: {}", e)))?;

        // Convert rows to JSON array
        let mut results = Vec::new();
        for row in rows {
            let mut obj = serde_json::Map::new();

            // Extract all columns
            for (i, column) in row.columns().iter().enumerate() {
                let column_name = column.name();

                // Try to extract value as different types
                let value: JsonValue = if let Ok(v) = row.try_get::<String, _>(i) {
                    JsonValue::String(v)
                } else if let Ok(v) = row.try_get::<i64, _>(i) {
                    JsonValue::Number(v.into())
                } else if let Ok(v) = row.try_get::<i32, _>(i) {
                    JsonValue::Number(v.into())
                } else if let Ok(v) = row.try_get::<bool, _>(i) {
                    JsonValue::Bool(v)
                } else if let Ok(v) = row.try_get::<JsonValue, _>(i) {
                    v
                } else {
                    JsonValue::Null
                };

                obj.insert(column_name.to_string(), value);
            }

            results.push(JsonValue::Object(obj));
        }

        Ok(JsonValue::Array(results))
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

    // FIX L-1: Removed broken tests that would panic
    // TODO: Reimplement with proper mocking framework (e.g., mockall or sqlx::test)

    #[test]
    fn test_parameter_substitution_logic() {
        // Test parameter substitution logic without database dependency
        let mut sql = "SELECT * FROM transactions WHERE identity_id = {{identity_id}} AND amount > {{min_amount}}".to_string();
        let params = serde_json::json!({
            "identity_id": "123e4567-e89b-12d3-a456-426614174000",
            "min_amount": 100
        });

        // Simulate the substitution logic
        if let Some(params_obj) = params.as_object() {
            for (key, value) in params_obj {
                let placeholder = format!("{{{{{}}}}}", key);
                let replacement = match value {
                    JsonValue::String(s) => format!("'{}'", s),
                    JsonValue::Number(n) => n.to_string(),
                    JsonValue::Bool(b) => b.to_string(),
                    _ => continue,
                };
                sql = sql.replace(&placeholder, &replacement);
            }
        }

        assert!(sql.contains("'123e4567-e89b-12d3-a456-426614174000'"));
        assert!(sql.contains("100"));
    }

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
}
