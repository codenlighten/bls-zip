// Application Module Registry - Pluggable enterprise applications

use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;

use crate::error::{EnterpriseError, Result};
use crate::models::*;

pub struct ApplicationService {
    db: PgPool,
}

impl ApplicationService {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    /// Register new application module
    pub async fn register_application(
        &self,
        name: String,
        description: String,
        category: AppCategory,
        api_base_url: String,
        required_scopes: Vec<String>,
        on_chain_contract_ref: Option<String>,
    ) -> Result<ApplicationModule> {
        // 1. Validate inputs
        if name.is_empty() {
            return Err(EnterpriseError::InvalidInput("Application name cannot be empty".to_string()));
        }

        if api_base_url.is_empty() {
            return Err(EnterpriseError::InvalidInput("API base URL cannot be empty".to_string()));
        }

        // 2. Create application ID
        let app_id = Uuid::new_v4();

        // 3. Insert into application_modules table
        sqlx::query!(
            r#"
            INSERT INTO application_modules
            (app_id, name, description, category, api_base_url, required_scopes, on_chain_contract_ref, enabled, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
            app_id,
            name,
            description,
            format!("{:?}", category),
            api_base_url,
            &required_scopes,
            on_chain_contract_ref,
            true,
            Utc::now(),
            Utc::now()
        )
        .execute(&self.db)
        .await
        .map_err(|e| EnterpriseError::from_db_error(e))?;

        // 4. Return ApplicationModule
        Ok(ApplicationModule {
            app_id,
            name,
            description,
            category,
            api_base_url,
            required_scopes,
            on_chain_contract_ref,
            enabled: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    /// Get application by ID
    pub async fn get_application(&self, app_id: Uuid) -> Result<ApplicationModule> {
        let row = sqlx::query!(
            r#"
            SELECT app_id, name, description, category, api_base_url, required_scopes, on_chain_contract_ref, enabled, created_at, updated_at
            FROM application_modules
            WHERE app_id = $1
            "#,
            app_id
        )
        .fetch_optional(&self.db)
        .await
        .map_err(|e| EnterpriseError::from_db_error(e))?
        .ok_or_else(|| EnterpriseError::ApplicationNotFound(app_id.to_string()))?;

        let category: AppCategory = serde_json::from_value(
            serde_json::Value::String(row.category)
        ).unwrap_or(AppCategory::Finance);

        Ok(ApplicationModule {
            app_id: row.app_id,
            name: row.name,
            description: row.description.unwrap_or_default(),
            category,
            api_base_url: row.api_base_url,
            required_scopes: row.required_scopes.unwrap_or_default(),
            on_chain_contract_ref: row.on_chain_contract_ref,
            enabled: row.enabled,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }

    /// List all applications
    pub async fn list_applications(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<ApplicationModule>> {
        let rows = sqlx::query!(
            r#"
            SELECT app_id, name, description, category, api_base_url, required_scopes, on_chain_contract_ref, enabled, created_at, updated_at
            FROM application_modules
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset
        )
        .fetch_all(&self.db)
        .await
        .map_err(|e| EnterpriseError::from_db_error(e))?;

        let applications = rows
            .into_iter()
            .map(|row| {
                let category: AppCategory = serde_json::from_value(
                    serde_json::Value::String(row.category)
                ).unwrap_or(AppCategory::Finance);

                ApplicationModule {
                    app_id: row.app_id,
                    name: row.name,
                    description: row.description.unwrap_or_default(),
                    category,
                    api_base_url: row.api_base_url,
                    required_scopes: row.required_scopes.unwrap_or_default(),
                    on_chain_contract_ref: row.on_chain_contract_ref,
                    enabled: row.enabled,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                }
            })
            .collect();

        Ok(applications)
    }

    /// Enable or disable application
    pub async fn set_enabled(&self, app_id: Uuid, enabled: bool) -> Result<()> {
        let result = sqlx::query!(
            "UPDATE application_modules SET enabled = $1, updated_at = $2 WHERE app_id = $3",
            enabled,
            Utc::now(),
            app_id
        )
        .execute(&self.db)
        .await
        .map_err(|e| EnterpriseError::from_db_error(e))?;

        if result.rows_affected() == 0 {
            return Err(EnterpriseError::ApplicationNotFound(app_id.to_string()));
        }

        Ok(())
    }

    /// Update application
    pub async fn update_application(
        &self,
        app_id: Uuid,
        name: Option<String>,
        description: Option<String>,
        api_base_url: Option<String>,
        required_scopes: Option<Vec<String>>,
    ) -> Result<()> {
        // Verify application exists
        self.get_application(app_id).await?;

        // Build dynamic update query
        let mut updates = vec!["updated_at = $1".to_string()];
        let mut param_index = 2;

        if name.is_some() {
            updates.push(format!("name = ${}", param_index));
            param_index += 1;
        }
        if description.is_some() {
            updates.push(format!("description = ${}", param_index));
            param_index += 1;
        }
        if api_base_url.is_some() {
            updates.push(format!("api_base_url = ${}", param_index));
            param_index += 1;
        }
        if required_scopes.is_some() {
            updates.push(format!("required_scopes = ${}", param_index));
        }

        // For simplicity, update all fields if provided
        if let (Some(n), Some(d), Some(a), Some(s)) = (&name, &description, &api_base_url, &required_scopes) {
            sqlx::query!(
                "UPDATE application_modules SET name = $1, description = $2, api_base_url = $3, required_scopes = $4, updated_at = $5 WHERE app_id = $6",
                n,
                d,
                a,
                s as &[String],
                Utc::now(),
                app_id
            )
            .execute(&self.db)
            .await
            .map_err(|e| EnterpriseError::from_db_error(e))?;
        }

        Ok(())
    }

    /// Log application event
    pub async fn log_event(
        &self,
        app_id: Uuid,
        identity_id: Uuid,
        event_type: String,
        event_data: serde_json::Value,
    ) -> Result<ApplicationEvent> {
        // Verify application exists
        self.get_application(app_id).await?;

        // Create event
        let event_id = Uuid::new_v4();

        sqlx::query!(
            r#"
            INSERT INTO application_events
            (event_id, app_id, identity_id, event_type, event_data, created_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            event_id,
            app_id,
            identity_id,
            event_type,
            event_data,
            Utc::now()
        )
        .execute(&self.db)
        .await
        .map_err(|e| EnterpriseError::from_db_error(e))?;

        Ok(ApplicationEvent {
            event_id,
            app_id,
            identity_id,
            event_type,
            event_data,
            created_at: Utc::now(),
        })
    }

    /// Get events for application
    pub async fn get_events(
        &self,
        app_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<ApplicationEvent>> {
        let rows = sqlx::query!(
            r#"
            SELECT event_id, app_id, identity_id, event_type, event_data, created_at
            FROM application_events
            WHERE app_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            app_id,
            limit,
            offset
        )
        .fetch_all(&self.db)
        .await
        .map_err(|e| EnterpriseError::from_db_error(e))?;

        let events = rows
            .into_iter()
            .map(|row| ApplicationEvent {
                event_id: row.event_id,
                app_id: row.app_id,
                identity_id: row.identity_id,
                event_type: row.event_type,
                event_data: row.event_data.unwrap_or(serde_json::json!({})),
                created_at: row.created_at,
            })
            .collect();

        Ok(events)
    }

    /// Get events for identity across all applications
    pub async fn get_identity_events(
        &self,
        identity_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<ApplicationEvent>> {
        let rows = sqlx::query!(
            r#"
            SELECT event_id, app_id, identity_id, event_type, event_data, created_at
            FROM application_events
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

        let events = rows
            .into_iter()
            .map(|row| ApplicationEvent {
                event_id: row.event_id,
                app_id: row.app_id,
                identity_id: row.identity_id,
                event_type: row.event_type,
                event_data: row.event_data.unwrap_or(serde_json::json!({})),
                created_at: row.created_at,
            })
            .collect();

        Ok(events)
    }

    /// Delete application (soft delete by disabling)
    pub async fn delete_application(&self, app_id: Uuid) -> Result<()> {
        self.set_enabled(app_id, false).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_application_registration() {
        // Test would verify application registration logic
    }
}
