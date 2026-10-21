//! Peer credential validation helper for IPC streams.

#![cfg(unix)]

use contextra_types::ContextraError;
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use tracing::warn;

/// Gets process OS User ID on Unix safely by inspecting metadata of a file owned by this process.
pub fn current_process_uid_from_file(file_path: &Path) -> Result<u32, ContextraError> {
    let meta = std::fs::metadata(file_path)?;
    Ok(meta.uid())
}

/// Checks peer credentials of a connected Unix stream against the expected UID.
pub fn validate_unix_peer_credentials(
    stream: &tokio::net::UnixStream,
    expected_uid: u32,
) -> Result<u32, ContextraError> {
    let cred = stream
        .peer_cred()
        .map_err(|e| ContextraError::InvalidInput(format!("Failed to retrieve peer_cred: {e}")))?;

    let peer_uid = cred.uid();
    if peer_uid != expected_uid {
        warn!(
            peer_uid = peer_uid,
            expected_uid = expected_uid,
            "Peer UID mismatch! Closing connection immediately without response."
        );
        return Err(ContextraError::InvalidInput(format!(
            "Peer UID mismatch: expected {expected_uid}, got {peer_uid}"
        )));
    }

    Ok(peer_uid)
}
