//! Bidirectional Proxy forwarding stdin/stdout <-> Unix Domain Socket or Windows Named Pipe.

use contextra_types::ContextraError;
use std::path::Path;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tracing::{debug, error, info, warn};

/// Bounded bidirectional proxy runner forwarding line-by-line stdio <-> Primary IPC endpoint.
pub struct IpcProxy {
    max_line_bytes: usize,
}

impl IpcProxy {
    /// Constructs a new `IpcProxy` with bounded buffer length.
    pub fn new(max_line_bytes: usize) -> Self {
        Self { max_line_bytes }
    }

    /// Connects to Primary IPC endpoint at `endpoint_path` and proxies stdio <-> IPC.
    pub async fn run_proxy(&self, endpoint_path: &Path) -> Result<(), ContextraError> {
        #[cfg(unix)]
        {
            let stream = tokio::net::UnixStream::connect(endpoint_path)
                .await
                .map_err(|e| {
                    ContextraError::Internal(format!(
                        "Proxy failed to connect to Primary IPC endpoint at {}: {e}",
                        endpoint_path.display()
                    ))
                })?;

            info!(
                endpoint = %endpoint_path.display(),
                "Proxy connected to Primary IPC socket. Starting stdio <-> IPC loop."
            );

            self.run_bidirectional_loop(stream).await
        }
        #[cfg(windows)]
        {
            let client = tokio::net::windows::named_pipe::ClientOptions::new()
                .open(endpoint_path)
                .map_err(|e| {
                    ContextraError::Internal(format!(
                        "Proxy failed to open Primary Named Pipe at {}: {e}",
                        endpoint_path.display()
                    ))
                })?;

            info!(
                endpoint = %endpoint_path.display(),
                "Proxy connected to Primary Named Pipe. Starting stdio <-> IPC loop."
            );

            self.run_bidirectional_loop(client).await
        }
    }

    /// Internal bidirectional loop forwarding stdin -> IPC and IPC -> stdout.
    async fn run_bidirectional_loop<S>(&self, stream: S) -> Result<(), ContextraError>
    where
        S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Send + 'static,
    {
        let (ipc_reader, mut ipc_writer) = tokio::io::split(stream);

        let max_line_bytes = self.max_line_bytes;

        // Task 1: stdin -> IPC
        let stdin_task = tokio::spawn(async move {
            let stdin = tokio::io::stdin();
            let mut stdin_reader = BufReader::new(stdin);
            let mut line = String::new();

            loop {
                line.clear();
                let mut handle = (&mut stdin_reader).take(max_line_bytes as u64);
                match handle.read_line(&mut line).await {
                    Ok(0) => {
                        info!("Stdin closed by host process. Closing IPC stream.");
                        let _ = ipc_writer.shutdown().await;
                        break;
                    }
                    Ok(_) => {
                        if line.is_empty() {
                            continue;
                        }
                        if let Err(e) = ipc_writer.write_all(line.as_bytes()).await {
                            error!("Error writing stdin payload to Primary IPC: {e}");
                            break;
                        }
                        if let Err(e) = ipc_writer.flush().await {
                            error!("Error flushing Primary IPC: {e}");
                            break;
                        }
                    }
                    Err(e) => {
                        error!("Error reading stdin in proxy: {e}");
                        break;
                    }
                }
            }
        });

        // Task 2: IPC -> stdout
        let ipc_to_stdout_task = tokio::spawn(async move {
            let mut ipc_buf_reader = BufReader::new(ipc_reader);
            let mut stdout = tokio::io::stdout();
            let mut line = String::new();

            loop {
                line.clear();
                let mut handle = (&mut ipc_buf_reader).take(max_line_bytes as u64);
                match handle.read_line(&mut line).await {
                    Ok(0) => {
                        warn!("Primary IPC connection closed unexpectedly.");
                        let err_msg = "{\"jsonrpc\":\"2.0\",\"error\":{\"code\":-32603,\"message\":\"Primary process disconnected\"},\"id\":null}\n";
                        if let Err(write_err) = stdout.write_all(err_msg.as_bytes()).await {
                            debug!(err = %write_err, "Failed to write disconnect message to stdout");
                        }
                        if let Err(flush_err) = stdout.flush().await {
                            debug!(err = %flush_err, "Failed to flush stdout after disconnect message");
                        }
                        break;
                    }
                    Ok(_) => {
                        if line.is_empty() {
                            continue;
                        }
                        if let Err(e) = stdout.write_all(line.as_bytes()).await {
                            error!("Error writing IPC response to stdout: {e}");
                            break;
                        }
                        if let Err(e) = stdout.flush().await {
                            error!("Error flushing stdout in proxy: {e}");
                            break;
                        }
                    }
                    Err(e) => {
                        error!("Error reading Primary IPC in proxy: {e}");
                        let err_msg = format!(
                            "{{\"jsonrpc\":\"2.0\",\"error\":{{\"code\":-32603,\"message\":{}}},\"id\":null}}\n",
                            serde_json::to_string(&e.to_string()).unwrap_or_default()
                        );
                        if let Err(write_err) = stdout.write_all(err_msg.as_bytes()).await {
                            debug!(err = %write_err, "Failed to write error message to stdout");
                        }
                        if let Err(flush_err) = stdout.flush().await {
                            debug!(err = %flush_err, "Failed to flush stdout after error message");
                        }
                        break;
                    }
                }
            }
        });

        tokio::select! {
            _ = stdin_task => {},
            _ = ipc_to_stdout_task => {},
        }

        Ok(())
    }
}
