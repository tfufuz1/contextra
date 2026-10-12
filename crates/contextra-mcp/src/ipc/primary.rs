//! Primary vs Proxy process role detection and endpoint file lifecycle.

use contextra::Contextra;
use contextra_types::ContextraError;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::{info, warn};

/// File name for the IPC endpoint pointer inside the database directory.
pub const ENDPOINT_FILE_NAME: &str = "contextra_mcp_endpoint.json";

/// Process role outcome from detection.
#[derive(Clone)]
pub enum ProcessRole {
    /// Process successfully acquired the primary DB lock and is running as Primary.
    Primary {
        db: Arc<Contextra>,
        endpoint_path: PathBuf,
    },
    /// Process failed to acquire DB lock but found an active Primary socket/pipe.
    Proxy { endpoint_path: PathBuf },
}

/// JSON payload stored in endpoint file.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct EndpointDescriptor {
    pub pid: u32,
    pub endpoint_path: String,
}

/// Error type for primary detection fail-closed scenarios.
#[derive(Debug, thiserror::Error)]
pub enum PrimaryDetectionError {
    #[error("Database lock conflict, but endpoint file is missing or invalid: {0}")]
    EndpointInvalid(String),

    #[error("Database lock conflict and Primary endpoint at {0} is unreachable or defunct")]
    PrimaryUnreachable(PathBuf),

    #[error("Contextra open error: {0}")]
    ContextraError(#[from] ContextraError),

    #[error("I/O error during primary detection: {0}")]
    IoError(#[from] std::io::Error),
}

/// Primary detector attempting to open DB or connect to existing Primary endpoint.
pub struct PrimaryDetector;

impl PrimaryDetector {
    /// Attempts to open Contextra DB at `db_path`.
    pub async fn detect_and_open(
        db_path: &Path,
        custom_endpoint: Option<&Path>,
    ) -> Result<ProcessRole, PrimaryDetectionError> {
        let endpoint_file_path = db_path.join(ENDPOINT_FILE_NAME);

        match Contextra::open(db_path).await {
            Ok(db) => {
                let db_arc = Arc::new(db);
                let actual_endpoint = if let Some(p) = custom_endpoint {
                    p.to_path_buf()
                } else {
                    Self::default_socket_path(db_path)
                };

                // Write endpoint descriptor
                let descriptor = EndpointDescriptor {
                    pid: std::process::id(),
                    endpoint_path: actual_endpoint.to_string_lossy().to_string(),
                };
                let content = serde_json::to_string_pretty(&descriptor)
                    .map_err(|e| PrimaryDetectionError::EndpointInvalid(e.to_string()))?;

                tokio::fs::write(&endpoint_file_path, content).await?;
                info!(
                    db_path = %db_path.display(),
                    endpoint = %actual_endpoint.display(),
                    "Acquired primary database lock, written endpoint file."
                );

                Ok(ProcessRole::Primary {
                    db: db_arc,
                    endpoint_path: actual_endpoint,
                })
            }
            Err(ContextraError::Storage(msg))
                if msg.contains("lock")
                    || msg.contains("WouldBlock")
                    || msg.contains("conflict")
                    || msg.contains("held") =>
            {
                info!(
                    db_path = %db_path.display(),
                    reason = %msg,
                    "Database directory locked by another process; checking primary endpoint file."
                );
                Self::verify_proxy_role(&endpoint_file_path).await
            }
            Err(err) => {
                let err_str = err.to_string();
                if err_str.contains("lock")
                    || err_str.contains("WouldBlock")
                    || err_str.contains("conflict")
                {
                    Self::verify_proxy_role(&endpoint_file_path).await
                } else {
                    Err(PrimaryDetectionError::ContextraError(err))
                }
            }
        }
    }

    /// Verifies if endpoint file exists, parses endpoint path, and checks reachability.
    async fn verify_proxy_role(
        endpoint_file_path: &Path,
    ) -> Result<ProcessRole, PrimaryDetectionError> {
        if !endpoint_file_path.exists() {
            return Err(PrimaryDetectionError::EndpointInvalid(format!(
                "Endpoint file not found at {}",
                endpoint_file_path.display()
            )));
        }

        let content = tokio::fs::read_to_string(endpoint_file_path).await?;
        let descriptor: EndpointDescriptor = serde_json::from_str(&content)
            .map_err(|e| PrimaryDetectionError::EndpointInvalid(e.to_string()))?;

        let target_endpoint = PathBuf::from(descriptor.endpoint_path);

        if Self::check_endpoint_reachable(&target_endpoint).await {
            info!(
                endpoint = %target_endpoint.display(),
                "Primary endpoint is active. Starting in Proxy mode."
            );
            Ok(ProcessRole::Proxy {
                endpoint_path: target_endpoint,
            })
        } else {
            warn!(
                endpoint = %target_endpoint.display(),
                "Primary endpoint file exists but endpoint is unreachable. Failing closed."
            );
            Err(PrimaryDetectionError::PrimaryUnreachable(target_endpoint))
        }
    }

    /// Default IPC socket path inside database directory.
    pub fn default_socket_path(db_path: &Path) -> PathBuf {
        #[cfg(unix)]
        {
            db_path.join("contextra_mcp.sock")
        }
        #[cfg(windows)]
        {
            let pipe_name = format!(
                r"\\.\pipe\contextra_mcp_{}",
                db_path.to_string_lossy().replace([':', '\\', '/'], "_")
            );
            PathBuf::from(pipe_name)
        }
    }

    /// Checks whether an IPC endpoint is reachable.
    pub async fn check_endpoint_reachable(endpoint_path: &Path) -> bool {
        #[cfg(unix)]
        {
            tokio::net::UnixStream::connect(endpoint_path).await.is_ok()
        }
        #[cfg(windows)]
        {
            tokio::net::windows::named_pipe::ClientOptions::new()
                .open(endpoint_path)
                .is_ok()
        }
    }
}
