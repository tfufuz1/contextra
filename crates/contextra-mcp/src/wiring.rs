//! McpWiring composition root module for contextra-mcp facade integration.

use crate::server::McpServer;
use contextra::Contextra;
use contextra_ports::EmbeddingProvider;
use contextra_types::ContextraError;
use std::sync::Arc;

/// Dependencies required to construct an [`McpWiring`] composition root.
pub struct McpWiringDeps {
    /// Contextra database engine handle.
    pub db: Arc<Contextra>,
    /// Embedding provider for vector search and ingestion.
    pub embedder: Arc<dyn EmbeddingProvider>,
}

/// Composition root entry point for `contextra-mcp`.
pub struct McpWiring {
    /// Fully configured McpServer instance.
    pub server: Arc<McpServer>,
}

impl McpWiring {
    /// Builds an [`McpWiring`] instance from dependencies.
    pub fn build(deps: McpWiringDeps) -> Result<Self, ContextraError> {
        let server = McpServer::new(deps.db, deps.embedder)?;
        Ok(Self {
            server: Arc::new(server),
        })
    }
}
