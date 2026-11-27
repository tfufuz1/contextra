//! MCP handlers for the durable erasure API.

use crate::{protocol::McpError, server::McpServer, validation::validate_collection_name};
use contextra::{AwaitLevel, EraseOptions, EraseTarget, ErasureId, ErasureReason};
use serde_json::{json, Value};

pub(crate) fn format_erasure_id(id: ErasureId) -> String {
    id.0.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn parse_erasure_id(value: &Value) -> Result<ErasureId, McpError> {
    let raw = value
        .as_str()
        .ok_or_else(|| McpError::invalid_params("erasure_id must be a 32-character hex string"))?;
    if raw.len() != 32 {
        return Err(McpError::invalid_params("erasure_id must be a 32-character hex string"));
    }
    let mut bytes = [0u8; 16];
    for (index, pair) in raw.as_bytes().chunks_exact(2).enumerate() {
        let hex = std::str::from_utf8(pair)
            .map_err(|_| McpError::invalid_params("erasure_id must be hexadecimal"))?;
        bytes[index] = u8::from_str_radix(hex, 16)
            .map_err(|_| McpError::invalid_params("erasure_id must be hexadecimal"))?;
    }
    Ok(ErasureId::new(bytes))
}

fn target(server: &McpServer, args: &Value) -> Result<EraseTarget, McpError> {
    let tenant = server
        .tenant_id
        .ok_or_else(|| McpError::invalid_params("tenant_id not configured on server"))?;
    let scope = args
        .get("scope")
        .and_then(Value::as_str)
        .ok_or_else(|| McpError::invalid_params("missing or invalid scope"))?;
    let collection = args
        .get("collection")
        .and_then(Value::as_str)
        .unwrap_or("default");
    match scope {
        "document" => {
            validate_collection_name(collection)?;
            let key = args
                .get("id")
                .and_then(Value::as_str)
                .ok_or_else(|| McpError::invalid_params("document scope requires id"))?;
            if key.is_empty() || key.len() > 4096 || key.as_bytes().contains(&0) {
                return Err(McpError::invalid_params("id must be 1..4096 bytes without NUL"));
            }
            Ok(EraseTarget::Document { tenant, collection: collection.to_owned(), key: key.to_owned() })
        }
        "collection" => {
            validate_collection_name(collection)?;
            if collection == "default" {
                return Err(McpError::invalid_params("default collection cannot be erased as collection"));
            }
            Ok(EraseTarget::Collection { tenant, collection: collection.to_owned() })
        }
        "tenant" => Err(McpError::invalid_params("tenant erase requires a configured confirmation token")),
        _ => Err(McpError::invalid_params("scope must be document or collection")),
    }
}

impl McpServer {
    pub(crate) async fn handle_erase(&self, args: &Value) -> Result<Value, McpError> {
        if !self.allow_erase {
            return Err(McpError::invalid_params("erase disabled"));
        }
        if args.get("confirm").and_then(Value::as_bool) != Some(true) {
            return Err(McpError::invalid_params("confirm must be explicitly true"));
        }
        let target = target(self, args)?;
        let reason = match args.get("reason").and_then(Value::as_str) {
            Some("user_request") => ErasureReason::UserRequest,
            Some("legal_request") => ErasureReason::LegalRequest,
            Some("retention_expiry") => ErasureReason::RetentionExpiry,
            Some("operator_action") => ErasureReason::OperatorAction,
            Some("tenant_offboarding") => ErasureReason::TenantOffboarding,
            _ => return Err(McpError::invalid_params("invalid or missing reason")),
        };
        let await_level = match args.get("await").and_then(Value::as_str).unwrap_or("gated") {
            "accepted" | "gated" => AwaitLevel::Gated,
            "purged" => AwaitLevel::Purged,
            _ => return Err(McpError::invalid_params("await must be accepted, gated, or purged")),
        };
        let handle = self.db.erase(target, EraseOptions { reason, await_level, policy: None })
            .await.map_err(McpError::from)?;
        Ok(json!({
            "ok": true,
            "erasure_id": format_erasure_id(handle.id),
            "state": format!("{:?}", handle.state),
            "guarantee": handle.reached.label(),
            "gated": handle.reached.gated,
            "purged": handle.reached.purged,
            "crypto_erased": handle.reached.crypto_erased,
            "exceptions": []
        }))
    }

    pub(crate) async fn handle_erase_status(&self, args: &Value) -> Result<Value, McpError> {
        let id = parse_erasure_id(args.get("erasure_id").unwrap_or(&Value::Null))?;
        let status = self.db.erase_status(id).await.map_err(McpError::from)?;
        serde_json::to_value(status).map_err(|error| McpError::internal_error(error.to_string()))
    }

    pub(crate) async fn handle_erase_dry_run(&self, args: &Value) -> Result<Value, McpError> {
        let target = target(self, args)?;
        let summary = self.db.erase_dry_run(target).await.map_err(McpError::from)?;
        serde_json::to_value(summary).map_err(|error| McpError::internal_error(error.to_string()))
    }
}
