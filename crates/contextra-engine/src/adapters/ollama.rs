//! Adapter bridge for Ollama client interfaces to `contextra-ports`.

use contextra_ports::BoxFuture;
use contextra_types::Result;

/// Neutral Ollama generator contract bridge.
pub trait OllamaBridge: Send + Sync {
    /// Dispatches prompt generation to an underlying model provider.
    fn generate<'a>(&'a self, prompt: &'a str) -> BoxFuture<'a, Result<String>>;
}
