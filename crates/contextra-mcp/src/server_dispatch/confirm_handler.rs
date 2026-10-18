use super::channel_kind::ChannelKind;
use super::error_codes::{McpErrorCode, StructuredMcpError};
use super::forget_guard::{ForgetGuard, ForgetScope};
use serde_json::json;

pub async fn handle_confirm<F, Fut>(
    guard: &ForgetGuard,
    principal: &str,
    token: &str,
    channel: ChannelKind,
    execute_deletion: F,
) -> Result<serde_json::Value, StructuredMcpError>
where
    F: FnOnce(ForgetScope, String) -> Fut,
    Fut: std::future::Future<Output = Result<u64, StructuredMcpError>>,
{
    if channel != ChannelKind::LocalIpc {
        return Err(StructuredMcpError::new(
            McpErrorCode::Denied,
            "contextra/confirm is forbidden over host stdio channel (must use LocalIpc)",
        ));
    }

    let (scope, collection) = guard.validate_and_consume_token(principal, token)?;

    let deleted_count = execute_deletion(scope, collection).await?;

    Ok(json!({
        "confirmed": true,
        "deleted": deleted_count
    }))
}
