//! Lokale Ports für contextra-agent (SPEC-v2 §6.3, S-63).

use async_trait::async_trait;
use contextra_types::{DocId, Result};

/// Abstraktion über persistente Agenten-Zustände und Retrieval-Quellen.
#[async_trait]
pub trait ContextStore: Send + Sync {
    /// Speichert einen Key-Value-Zustand.
    async fn put_state(&self, key: &str, value: &[u8]) -> Result<()>;
    /// Liest einen Key-Value-Zustand.
    async fn get_state(&self, key: &str) -> Result<Option<Vec<u8>>>;
    /// Löscht einen Key-Value-Zustand.
    async fn delete_state(&self, key: &str) -> Result<bool>;
}

/// Abstraktion für ephemere Scratchpad-Puffer im RAM.
pub trait ScratchpadStore: Send + Sync {
    /// Fügt dem flüchtigen Speicher einen Chunk hinzu.
    fn append_chunk(&mut self, id: DocId, content: Vec<u8>) -> Result<()>;
    /// Leert den flüchtigen Speicher und überschreibt ihn via Zeroize.
    fn purge(&mut self);
    /// Liefert die aktuell belegten Bytes im RAM.
    fn total_bytes(&self) -> usize;
}
