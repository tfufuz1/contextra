//! IPC Server implementation managing Unix Domain Socket or Windows Named Pipe listeners.

use crate::config::IpcConfig;
use crate::ipc::peer_cred::validate_unix_peer_credentials;
use contextra_types::ContextraError;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tracing::{error, info, warn};

/// Line processing handler function signature.
pub type LineHandlerFn = Arc<
    dyn Fn(String) -> contextra_ports::BoxFuture<'static, Result<Option<String>, ContextraError>>
        + Send
        + Sync,
>;

/// IPC Listener managing secure socket/pipe connections.
pub struct IpcServer {
    endpoint_path: PathBuf,
    config: IpcConfig,
}

impl IpcServer {
    /// Creates a new `IpcServer` bound to `endpoint_path` with `config`.
    pub fn new(endpoint_path: &Path, config: IpcConfig) -> Self {
        Self {
            endpoint_path: endpoint_path.to_path_buf(),
            config,
        }
    }

    /// Explicitly checks that no TCP or UDP socket listener configuration is attempted.
    pub fn verify_no_network_listener(addr_str: &str) -> Result<(), ContextraError> {
        if addr_str.starts_with("tcp://")
            || addr_str.starts_with("udp://")
            || (addr_str.contains(':') && addr_str.parse::<std::net::SocketAddr>().is_ok())
        {
            return Err(ContextraError::InvalidInput(
                "INV-IPC-1 / §21.0 violation: TCP/UDP network listeners are strictly forbidden for MCP IPC."
                    .to_string(),
            ));
        }
        Ok(())
    }

    /// Runs the IPC listener loop on Unix platforms.
    #[cfg(unix)]
    pub async fn run_unix_listener<F>(&self, handler: F) -> Result<(), ContextraError>
    where
        F: Fn(
                String,
            )
                -> contextra_ports::BoxFuture<'static, Result<Option<String>, ContextraError>>
            + Send
            + Sync
            + 'static,
    {
        use std::os::unix::fs::PermissionsExt;

        // Ensure parent directory permissions 0700
        if let Some(parent) = self.endpoint_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
            std::fs::set_permissions(parent, std::fs::Permissions::from_mode(0o700))?;
        }

        // Clean up stale socket if present
        if self.endpoint_path.exists() {
            let _ = tokio::fs::remove_file(&self.endpoint_path).await;
        }

        let listener = tokio::net::UnixListener::bind(&self.endpoint_path)?;

        // Set socket file permissions 0600
        std::fs::set_permissions(&self.endpoint_path, std::fs::Permissions::from_mode(0o600))?;

        let expected_uid = match self.config.expected_uid {
            Some(uid) => uid,
            None => std::fs::metadata(&self.endpoint_path)?.uid(),
        };

        let max_line_bytes = self.config.max_line_bytes;
        let handler = Arc::new(handler);

        info!(
            endpoint = %self.endpoint_path.display(),
            expected_uid = expected_uid,
            "IPC Unix Domain Socket server started (dir mode 0700, socket mode 0600)."
        );

        loop {
            match listener.accept().await {
                Ok((stream, _addr)) => {
                    // Check peer credentials instantly
                    if let Err(e) = validate_unix_peer_credentials(&stream, expected_uid) {
                        warn!("Peer credential validation failed: {e}. Connection dropped.");
                        drop(stream);
                        continue;
                    }

                    let handler_clone = Arc::clone(&handler);
                    tokio::spawn(async move {
                        if let Err(err) =
                            Self::handle_unix_client(stream, max_line_bytes, handler_clone).await
                        {
                            error!("Error handling IPC client connection: {err}");
                        }
                    });
                }
                Err(e) => {
                    error!("IPC listener accept error: {e}");
                    break;
                }
            }
        }

        Ok(())
    }

    #[cfg(unix)]
    async fn handle_unix_client<F>(
        stream: tokio::net::UnixStream,
        max_line_bytes: usize,
        handler: Arc<F>,
    ) -> Result<(), ContextraError>
    where
        F: Fn(
                String,
            )
                -> contextra_ports::BoxFuture<'static, Result<Option<String>, ContextraError>>
            + Send
            + Sync
            + 'static,
    {
        let (reader, mut writer) = stream.into_split();
        let mut buf_reader = BufReader::new(reader);
        let mut line = String::new();

        loop {
            line.clear();
            let mut handle = (&mut buf_reader).take(max_line_bytes as u64);
            let bytes_read = handle.read_line(&mut line).await?;

            if bytes_read == 0 {
                // Disconnect
                break;
            }

            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            match handler(trimmed.to_string()).await {
                Ok(Some(response_json)) => {
                    let mut out = response_json;
                    if !out.ends_with('\n') {
                        out.push('\n');
                    }
                    writer.write_all(out.as_bytes()).await?;
                    writer.flush().await?;
                }
                Ok(None) => {}
                Err(err) => {
                    let err_json = format!(
                        "{{\"jsonrpc\":\"2.0\",\"error\":{{\"code\":-32603,\"message\":{}}},\"id\":null}}\n",
                        serde_json::to_string(&err.to_string()).unwrap_or_default()
                    );
                    writer.write_all(err_json.as_bytes()).await?;
                    writer.flush().await?;
                }
            }
        }

        Ok(())
    }

    /// Windows Named Pipe Server stub.
    #[cfg(windows)]
    pub async fn run_windows_listener<F>(&self, _handler: F) -> Result<(), ContextraError>
    where
        F: Fn(
                String,
            )
                -> contextra_ports::BoxFuture<'static, Result<Option<String>, ContextraError>>
            + Send
            + Sync
            + 'static,
    {
        info!(
            endpoint = %self.endpoint_path.display(),
            "IPC Windows Named Pipe server initialized."
        );
        Ok(())
    }
}
