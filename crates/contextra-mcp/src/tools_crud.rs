use crate::protocol::McpError;
use crate::server::McpServer;
use crate::validation::validate_collection_name;
// Deployment tier
use contextra::collection_profile;
use serde_json::{json, Value};

impl McpServer {
    /// Handler für `contextra_create_collection` — Erstellt eine neue Collection mit wählbarem DeploymentTier.
    pub(crate) async fn handle_create_collection(&self, args: &Value) -> Result<Value, McpError> {
        let name = match args.get("collection").or_else(|| args.get("name")) {
            Some(col_val) => {
                let s = col_val.as_str().ok_or_else(|| {
                    McpError::invalid_params("Invalid params: 'collection' must be a string")
                })?;
                if s.trim().is_empty() {
                    return Err(McpError::invalid_params("collection name cannot be empty"));
                }
                validate_collection_name(s)?;
                s
            }
            None => {
                return Err(McpError::invalid_params(
                    "missing required field 'collection' or 'name'",
                ));
            }
        };

        let tier_str = match args.get("deployment_tier") {
            Some(v) => v.as_str().ok_or_else(|| {
                McpError::invalid_params("Invalid params: 'deployment_tier' must be a string")
            })?,
            None => "PowerUserLocal", // Default tier if omitted
        };

        let tier = match tier_str {
            "EdgeMinimal" => contextra::collection_profile::DeploymentTier::EdgeMinimal,
            "PowerUserLocal" => contextra::collection_profile::DeploymentTier::PowerUserLocal,
            "EnterpriseShared" => contextra::collection_profile::DeploymentTier::EnterpriseShared,
            "EnterpriseRegulated" => {
                contextra::collection_profile::DeploymentTier::EnterpriseRegulated
            }
            invalid => {
                return Err(McpError::invalid_params(format!(
                    "Invalid deployment_tier '{invalid}'. Valid options are: 'EdgeMinimal', 'PowerUserLocal', 'EnterpriseShared', 'EnterpriseRegulated'"
                )));
            }
        };

        // Initialize collection profile and resolve configuration
        let mut profile = tier.resolve();

        if let Some(ae_val) = args.get("auto_extraction") {
            let ae_str = ae_val.as_str().ok_or_else(|| {
                McpError::invalid_params(
                    "Invalid params: 'auto_extraction' must be a string ('enabled' or 'disabled')",
                )
            })?;
            match ae_str.to_lowercase().as_str() {
                "enabled" => {
                    profile.auto_extraction =
                        contextra::collection_profile::AutoExtractionMode::Enabled
                }
                "disabled" => {
                    profile.auto_extraction =
                        contextra::collection_profile::AutoExtractionMode::Disabled
                }
                invalid => {
                    return Err(McpError::invalid_params(format!(
                        "Invalid auto_extraction '{invalid}'. Valid options are: 'enabled', 'disabled'"
                    )));
                }
            }
        }

        profile.validate().map_err(|e| {
            McpError::invalid_params(format!("Invalid collection profile configuration: {e}"))
        })?;

        // Initialize / open collection via Contextra DB
        let _col = self.db.collection(name).await.map_err(McpError::from)?;

        Ok(json!({
            "ok": true,
            "collection": name,
            "deployment_tier": tier_str,
            "auto_extraction": format!("{:?}", profile.auto_extraction).to_lowercase(),
            "kv_delete_mode": format!("{:?}", profile.kv_delete_mode)
        }))
    }

    /// Handler für `contextra_drop_collection` — Entfernt eine gesamte Collection.
    pub(crate) async fn handle_drop_collection(&self, args: &Value) -> Result<Value, McpError> {
        let col_name = match args.get("collection").or_else(|| args.get("name")) {
            Some(col_val) => {
                let s = col_val.as_str().ok_or_else(|| {
                    McpError::invalid_params("Invalid params: 'collection' must be a string")
                })?;
                if s.trim().is_empty() {
                    return Err(McpError::invalid_params("collection cannot be empty"));
                }
                validate_collection_name(s)?;
                s
            }
            None => {
                return Err(McpError::invalid_params(
                    "missing required field 'collection'",
                ));
            }
        };

        let confirm = match args.get("confirm") {
            Some(Value::Bool(b)) => *b,
            _ => false,
        };
        if !confirm {
            return Err(McpError::invalid_params(
                "confirm parameter must be explicitly set to true for contextra_drop_collection",
            ));
        }

        let tenant_id = self
            .tenant_id
            .ok_or_else(|| McpError::invalid_params("tenant_id not configured on server"))?;

        let _ = self
            .db
            .drop_collection(col_name, tenant_id, &[])
            .await
            .map_err(McpError::from)?;

        Ok(json!({
            "ok": true,
            "deleted": true,
            "collection": col_name,
        }))
    }

    /// Handler für `contextra_delete` — Löscht ein Einzel-Dokument.
    pub(crate) async fn handle_delete(&self, args: &Value) -> Result<Value, McpError> {
        let col_name = if let Some(col_val) = args.get("collection") {
            let s = col_val.as_str().ok_or_else(|| {
                McpError::invalid_params("Invalid params: 'collection' must be a string")
            })?;
            if s.trim().is_empty() {
                "default"
            } else {
                validate_collection_name(s)?;
                s
            }
        } else {
            "default"
        };

        let id = match args.get("id") {
            Some(v) => {
                let s = v.as_str().ok_or_else(|| {
                    McpError::invalid_params("Invalid params: 'id' must be a string")
                })?;
                if s.trim().is_empty() {
                    return Err(McpError::invalid_params("id cannot be empty"));
                }
                if s.len() > 256 {
                    return Err(McpError::invalid_params(
                        "id length exceeds limit: max 256 chars",
                    ));
                }
                s
            }
            None => {
                return Err(McpError::invalid_params("missing required field 'id'"));
            }
        };

        let _tenant_id = self
            .tenant_id
            .ok_or_else(|| McpError::invalid_params("tenant_id not configured on server"))?;

        let col = self.db.collection(col_name).await.map_err(McpError::from)?;

        // Perform actual document deletion
        col.delete(id).await.map_err(McpError::from)?;

        Ok(json!({
            "ok": true,
            "deleted": true,
            "collection": col_name,
            "id": id,
        }))
    }
}
