use crate::protocol::McpError;

/// Checks whether database write access is explicitly enabled via environment variable `MEMFUSE_MCP_ALLOW_WRITE`.
pub fn is_write_allowed_by_env() -> bool {
    std::env::var("MEMFUSE_MCP_ALLOW_WRITE")
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
