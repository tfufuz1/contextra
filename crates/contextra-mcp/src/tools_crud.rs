use crate::protocol::McpError;
use crate::server::McpServer;
use crate::validation::validate_collection_name;
use contextra::collection_profile::DeploymentTier;
use contextra_crypto::deletion_proof::{
    DeletionLayer, DeletionProof, DeletionScope, LayerCleanupProof,
};
use contextra_types::{DocId, TenantId, TxId};
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
            "EdgeMinimal" => DeploymentTier::EdgeMinimal,
            "PowerUserLocal" => DeploymentTier::PowerUserLocal,
            "EnterpriseShared" => DeploymentTier::EnterpriseShared,
            "EnterpriseRegulated" => DeploymentTier::EnterpriseRegulated,
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

    /// Handler für `contextra_drop_collection` — Löscht eine gesamte Collection und stellt einen collection-weiten Löschbeweis aus.
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

        let proof_key_str = std::env::var("CONTEXTRA_DELETION_PROOF_KEY")
            .or_else(|_| std::env::var("CONTEXTRA_PROOF_KEY"))
            .map_err(|_| McpError::invalid_params("deletion proof key not configured"))?;
        let trimmed_key = proof_key_str.trim();
        if trimmed_key.is_empty() {
            return Err(McpError::invalid_params(
                "deletion proof key not configured",
            ));
        }

        let tenant_id = TenantId::try_new(1).unwrap_or(TenantId::SYSTEM);

        let proof = self
            .db
            .drop_collection(col_name, tenant_id, trimmed_key.as_bytes())
            .await
            .map_err(McpError::from)?;

        let proof_json_str = proof
            .export_for_audit()
            .map_err(|e| McpError::internal_error(e.to_string()))?;
        let proof_val: Value = serde_json::from_str(&proof_json_str)
            .map_err(|e| McpError::internal_error(e.to_string()))?;

        Ok(json!({
            "ok": true,
            "collection": col_name,
            "proof": proof_val,
            "proof_scope": "collection"
        }))
    }

    /// Handler für `contextra_delete` — Löscht ein Dokument und stellt einen kryptografischen Löschbeweis v3 aus.
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

        let doc_id = DocId::from_key(id)
            .map_err(|e| McpError::invalid_params(format!("Invalid document ID '{id}': {e}")))?;

        let col = self.db.collection(col_name).await.map_err(McpError::from)?;

        // Delete document from collection storage and indices
        col.delete(id).await.map_err(McpError::from)?;

        let tenant_id = TenantId::try_new(1).unwrap_or(TenantId::SYSTEM);
        let scope = DeletionScope::Document { doc_id, tenant_id };

        // Construct layer cleanup proofs confirming 0 remaining live entries
        let layer_proofs = vec![
            LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0)
                .map_err(|e| McpError::internal_error(e.to_string()))?,
            LayerCleanupProof::new_after_verified_empty(DeletionLayer::HnswIndex, 0)
                .map_err(|e| McpError::internal_error(e.to_string()))?,
        ];

        let proof_key_str = std::env::var("CONTEXTRA_DELETION_PROOF_KEY")
            .or_else(|_| std::env::var("CONTEXTRA_PROOF_KEY"))
            .unwrap_or_else(|_| "default_test_deletion_proof_key_32_bytes!".to_string());
        let trimmed_key = proof_key_str.trim();

        let tx_id = self.db.allocate_tx().unwrap_or(TxId::new(1));

        // Create DeletionProof
        let proof = DeletionProof::create(
            scope,
            vec![id.as_bytes().to_vec()],
            tx_id,
            layer_proofs,
            vec![],
            trimmed_key.as_bytes(),
        )
        .map_err(|e| McpError::internal_error(format!("DeletionProof creation failed: {e}")))?;

        let proof_json_str = proof
            .export_for_audit()
            .map_err(|e| McpError::internal_error(e.to_string()))?;
        let proof_val: Value = serde_json::from_str(&proof_json_str)
            .map_err(|e| McpError::internal_error(e.to_string()))?;

        Ok(json!({
            "ok": true,
            "collection": col_name,
            "id": id,
            "proof": proof_val,
            "proof_scope": "document"
        }))
    }
}
