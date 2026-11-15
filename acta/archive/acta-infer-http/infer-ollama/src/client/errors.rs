use crate::error::OllamaError;

/// Helper to check if a reqwest error is a transient network error (timeout or connection error).
pub fn is_transient_network_error(err: &reqwest::Error) -> bool {
    err.is_timeout() || err.is_connect()
}

/// Classifies a `reqwest::Error` into structured `OllamaError` variants.
pub fn classify_reqwest_error(e: reqwest::Error, base_url: &str, context: &str) -> OllamaError {
    if e.is_connect() {
        OllamaError::Network(format!(
            "Ollama is not reachable at {base_url}: {e}. Ensure Ollama is running (`ollama serve`)."
        ))
    } else if is_transient_network_error(&e) {
        OllamaError::Timeout(format!("{context} network error at {base_url}: {e}"))
    } else {
        OllamaError::Server(format!("{context} network error at {base_url}: {e}"))
    }
}

/// Helper to classify transient network errors for retry.
pub fn is_transient_error(e: &OllamaError) -> bool {
    match e {
        OllamaError::Network(_) | OllamaError::Timeout(_) => true,
        OllamaError::Server(msg) | OllamaError::Internal(msg) => {
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
