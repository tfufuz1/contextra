use crate::protocol::McpError;
use contextra_types::TenantId;
use serde_json::Value;

/// Resolves `TenantId` dynamically from request arguments (`tenant_id` or `tenant`),
/// environment variable `CONTEXTRA_TENANT_ID`, or falls back to `TenantId::SYSTEM`.
pub fn extract_tenant_from_args_or_env(args: Option<&Value>) -> TenantId {
    if let Some(args_val) = args {
        if let Some(t_val) = args_val.get("tenant_id").or_else(|| args_val.get("tenant")) {
            if let Some(num) = t_val.as_u64() {
                if let Ok(tid) = TenantId::try_new(num) {
                    return tid;
                }
            } else if let Some(s) = t_val.as_str() {
                if let Ok(num) = s.parse::<u64>() {
                    if let Ok(tid) = TenantId::try_new(num) {
                        return tid;
                    }
                }
            }
        }
    }

    if let Ok(env_str) = std::env::var("CONTEXTRA_TENANT_ID") {
        if let Ok(num) = env_str.trim().parse::<u64>() {
            if let Ok(tid) = TenantId::try_new(num) {
                return tid;
            }
        }
    }

    TenantId::SYSTEM
}

/// Checks whether database write access is explicitly enabled via environment variable `CONTEXTRA_MCP_ALLOW_WRITE`.
pub fn is_write_allowed_by_env() -> bool {
    std::env::var("CONTEXTRA_MCP_ALLOW_WRITE")
        .map(|v| {
            let s = v.trim().to_lowercase();
            s == "1" || s == "true" || s == "yes"
        })
        .unwrap_or(false)
}

/// MCP-Server mit stdio-Transport (JSON-RPC 2.0).
///
/// stdout ist dem Protokoll vorbehalten — Logs gehen ausschließlich nach stderr.
pub fn validate_collection_name(name: &str) -> Result<(), McpError> {
    if name.trim().is_empty() || name.len() > 256 {
        return Err(McpError::invalid_params(format!(
            "Invalid collection name length: {}",
            name.len()
        )));
    }
    if name.contains('\0') || name.contains(':') || name.contains('/') {
        return Err(McpError::invalid_params(format!(
            "Collection name '{name}' contains forbidden characters"
        )));
    }
    Ok(())
}
