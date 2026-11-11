// FILE-CONTEXT Header (Format v3)
// ZWECK: Definition lokaler Ports für Zustandsspeicherung (ContextStore) und ephemeren Scratchpad-Speicher (ScratchpadStore).
// INVARIANTEN: Zeroize-Anforderung bei Purge; Thread-Safe Trait-Definitionen (Send + Sync).
// NICHT-OFFENSICHTLICH: Entkoppelt den Agenten-Workflow vollständig von physischen Speicher-Implementierungen.
// HOTSPOTS: ContextStore-Methoden (ll. 20-35), ScratchpadStore (ll. 40-55).
// STAND: TS:2026-11-11T00:25:00Z (SESSION: mod-decouple-agent)

//! Lokale Ports für contextra-agent (SPEC §6.2, S-63).

use contextra_ports::BoxFuture;
use contextra_types::{DocId, Result};

// =========================================================================
// SECTION: ContextStore Port
// =========================================================================

// ANCHOR[contextra-agent:port-context-store]
/// Abstraktion über persistente Agenten-Zustände und Retrieval-Quellen.
pub trait ContextStore: Send + Sync {
    /// Speichert einen Key-Value-Zustand.
    fn put_state<'a>(&'a self, key: &'a str, value: &'a [u8]) -> BoxFuture<'a, Result<()>>;
    /// Liest einen Key-Value-Zustand.
    fn get_state<'a>(&'a self, key: &'a str) -> BoxFuture<'a, Result<Option<Vec<u8>>>>;
    /// Löscht einen Key-Value-Zustand.
    fn delete_state<'a>(&'a self, key: &'a str) -> BoxFuture<'a, Result<bool>>;
}

// !SECTION

// =========================================================================
// SECTION: ScratchpadStore Port
// =========================================================================

// ANCHOR[contextra-agent:port-scratchpad-store]
/// Abstraktion für ephemere Scratchpad-Puffer im RAM.
pub trait ScratchpadStore: Send + Sync {
    /// Fügt dem flüchtigen Speicher einen Chunk hinzu.
    fn append_chunk(&mut self, id: DocId, content: Vec<u8>) -> Result<()>;
    /// Leert den flüchtigen Speicher und überschreibt ihn via Zeroize.
    fn purge(&mut self);
    /// Liefert die aktuell belegten Bytes im RAM.
    fn total_bytes(&self) -> usize;
}

// ANCHOR[contextra-agent:in-memory-scratchpad-store]
/// In-Memory-Referenzimplementierung von [`ScratchpadStore`] für Tests und entkoppelte Ausführung.
#[derive(Debug, Default)]
pub struct InMemoryScratchpadStore {
    chunks: Vec<(DocId, Vec<u8>)>,
}

impl InMemoryScratchpadStore {
    /// Erzeugt einen neuen leeren `InMemoryScratchpadStore`.
    pub fn new() -> Self {
        Self { chunks: Vec::new() }
    }
}

impl ScratchpadStore for InMemoryScratchpadStore {
    fn append_chunk(&mut self, id: DocId, content: Vec<u8>) -> Result<()> {
        self.chunks.push((id, content));
        Ok(())
    }

    fn purge(&mut self) {
        self.chunks.clear();
    }

    fn total_bytes(&self) -> usize {
        self.chunks.iter().map(|(_, c)| c.len()).sum()
    }
}

// !SECTION
