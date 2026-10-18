//! `confirm` CLI subcommand connecting over IPC to issue out-of-band `contextra/confirm`.

use contextra_types::ContextraError;
use std::path::Path;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tracing::info;

/// Result from `confirm` IPC request.
#[derive(Debug, serde::Deserialize)]
pub struct ConfirmResponsePayload {
    pub confirmed: bool,
    #[serde(default)]
    pub deleted: u64,
}

#[derive(Debug, serde::Deserialize)]
struct JsonRpcResponse {
    result: Option<ConfirmResponsePayload>,
    error: Option<serde_json::Value>,
}

/// Executes `contextra/confirm` method over IPC to Primary process.
pub async fn execute_confirm(
    endpoint_path: &Path,
    token: &str,
    max_line_bytes: usize,
) -> Result<ConfirmResponsePayload, ContextraError> {
    #[cfg(unix)]
    let mut stream = tokio::net::UnixStream::connect(endpoint_path)
        .await
        .map_err(|e| {
            ContextraError::Internal(format!(
                "Failed to connect to Primary IPC endpoint at {}: {e}",
                endpoint_path.display()
            ))
        })?;

    #[cfg(windows)]
    let mut stream = tokio::net::windows::named_pipe::ClientOptions::new()
        .open(endpoint_path)
        .map_err(|e| {
            ContextraError::Internal(format!(
                "Failed to open Primary Named Pipe at {}: {e}",
                endpoint_path.display()
            ))
        })?;

    let rpc_req = format!(
        "{{\"jsonrpc\":\"2.0\",\"method\":\"contextra/confirm\",\"params\":{{\"token\":{}}},\"id\":1}}\n",
        serde_json::to_string(token).map_err(|e| ContextraError::InvalidInput(e.to_string()))?
    );

    stream.write_all(rpc_req.as_bytes()).await?;
    stream.flush().await?;

    let (reader, _writer) = stream.into_split();
    let mut buf_reader = BufReader::new(reader);
    let mut line = String::new();

    let mut handle = (&mut buf_reader).take(max_line_bytes as u64);
    let bytes_read = handle.read_line(&mut line).await?;

    if bytes_read == 0 {
        return Err(ContextraError::Internal(
            "Primary process closed connection before responding to confirm request.".to_string(),
        ));
    }

    let parsed: JsonRpcResponse = serde_json::from_str(&line).map_err(|e| {
        ContextraError::Internal(format!("Invalid JSON-RPC response from Primary IPC: {e}"))
    })?;

    if let Some(err) = parsed.error {
        return Err(ContextraError::InvalidInput(format!(
            "Confirmation rejected by Primary: {err}"
        )));
    }

    if let Some(res) = parsed.result {
        info!(
            confirmed = res.confirmed,
            deleted = res.deleted,
            "Received confirm response from Primary process."
        );
        if !res.confirmed {
            return Err(ContextraError::InvalidInput(
                "Confirmation declined by Primary (confirmed = false).".to_string(),
            ));
        }
        Ok(res)
    } else {
        Err(ContextraError::Internal(
            "Response contained neither result nor error.".to_string(),
        ))
    }
}
