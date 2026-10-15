use crate::protocol::McpError;
use contextra_types::TenantId;
use serde_json::Value;

/// Validates that any tenant parameter provided in request arguments strictly matches the active session tenant.
///
/// Returns `Ok(session_tenant)` if valid or unprovided, or an `McpError` if a foreign tenant ID is supplied.
pub fn validate_request_tenant(
    session_tenant: TenantId,
    args: Option<&Value>,
) -> Result<TenantId, McpError> {
    if let Some(args_val) = args {
        if let Some(t_val) = args_val.get("tenant_id").or_else(|| args_val.get("tenant")) {
            let req_tenant_num = if let Some(num) = t_val.as_u64() {
                Some(num)
            } else if let Some(s) = t_val.as_str() {
                s.parse::<u64>().ok()
            } else {
                None
            };

            if let Some(num) = req_tenant_num {
                if num != session_tenant.inner() {
                    return Err(McpError::invalid_params(format!(
                        "Tenant ID mismatch: request requested tenant {num}, but server session is bound to tenant {}",
                        session_tenant.inner()
                    )));
                }
            } else {
                return Err(McpError::invalid_params(
                    "Invalid tenant parameter format in request arguments",
                ));
            }
        }
    }

    Ok(session_tenant)
}

/// Resolves session `TenantId` from environment (`CONTEXTRA_TENANT_ID`), failing closed if absent, invalid, or 0.
pub fn resolve_session_tenant_from_env() -> Result<TenantId, contextra_types::ContextraError> {
    let env_str = std::env::var("CONTEXTRA_TENANT_ID").map_err(|_| {
        contextra_types::ContextraError::InvalidInput(
            "CONTEXTRA_TENANT_ID environment variable is missing. Server requires an explicit non-zero session tenant.".to_string(),
        )
    })?;

    let num = env_str.trim().parse::<u64>().map_err(|_| {
        contextra_types::ContextraError::InvalidInput(format!(
            "Invalid CONTEXTRA_TENANT_ID '{env_str}'. Must be a valid non-zero u64 integer."
        ))
    })?;

    if num == 0 {
        return Err(contextra_types::ContextraError::InvalidInput(
            "TenantId 0 (SYSTEM) is prohibited for MCP server session (INV-TENANT-1).".to_string(),
        ));
    }

    TenantId::try_new(num).map_err(|e| {
        contextra_types::ContextraError::InvalidInput(format!("Invalid TenantId: {e}"))
    })
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
