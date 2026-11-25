use contextra_types::ContextraError;

/// Helper to check if a reqwest error is a transient network error (timeout or connection error).
pub fn is_transient_network_error(err: &reqwest::Error) -> bool {
    err.is_timeout() || err.is_connect()
}

/// Classifies a `reqwest::Error` into structured `ContextraError` variants (`Io` for connection refused or timeouts, `Storage` otherwise).
pub fn classify_reqwest_error(e: reqwest::Error, base_url: &str, context: &str) -> ContextraError {
    if e.is_connect() {
        ContextraError::Io(std::io::Error::new(
            std::io::ErrorKind::ConnectionRefused,
            format!(
                "Ollama is not reachable at {base_url}: {e}. Ensure Ollama is running (`ollama serve`)."
            ),
        ))
    } else if is_transient_network_error(&e) {
        ContextraError::Io(std::io::Error::new(
            std::io::ErrorKind::TimedOut,
            format!("{context} network error at {base_url}: {e}"),
        ))
    } else {
        ContextraError::Storage(format!("{context} network error at {base_url}: {e}"))
    }
}

/// Helper to classify transient network errors for retry.
///
/// Returns true only for transient network failures (I/O error, connection reset, timeout)
/// or 5xx server errors (500, 502, 503, 504).
/// Returns false for 4xx client errors (400 Invalid Input, 404 Not Found, etc.).
pub fn is_transient_error(e: &ContextraError) -> bool {
    match e {
        ContextraError::Io(_) => true,
        ContextraError::Storage(msg) | ContextraError::Internal(msg) => {
            let l = msg.to_lowercase();
            l.contains("503")
                || l.contains("500")
                || l.contains("502")
                || l.contains("504")
                || l.contains("timeout")
                || l.contains("connect")
                || l.contains("connection reset")
                || l.contains("broken pipe")
        }
        _ => false,
    }
}
