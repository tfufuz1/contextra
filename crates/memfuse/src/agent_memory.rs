// FILE-CONTEXT
// ZWECK: Agent Memory Facade (Ring 4) — Provides a clean remember/recall/forget/relate API for AI Agents.
// INVARIANTEN: #![forbid(unsafe_code)]; Zero panic doctrine (.unwrap/.expect forbidden); Wraps MemFuse engine without changing business logic.

use memfuse_core::error::MemFuseError;
use memfuse_db::{MemFuse, ProvenanceRecord, SearchResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;
use std::sync::Arc;

/// Opaque identifier for a memory stored via `AgentMemory`.
///
/// Wraps the underlying document ID string used by `MemFuse`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MemoryId(String);

impl MemoryId {
    /// Creates a new `MemoryId` from a string identifier.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Returns a string slice reference to the underlying memory ID.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for MemoryId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for MemoryId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl AsRef<str> for MemoryId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for MemoryId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Representation of a recalled memory record.
///
/// Derived from `SearchResult` provided by `MemFuse::search_text` or hybrid queries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    /// Document identifier string.
    pub id: String,
    /// Relevance / similarity score.
    pub score: f32,
    /// Optional metadata associated with the memory entry.
    pub metadata: Option<Value>,
    /// Matched signal types (e.g., text, vector, graph).
    pub matched_signals: Vec<String>,
    /// Detailed score provenance breakdown.
    pub provenance: Option<ProvenanceRecord>,
}

impl From<SearchResult> for Memory {
    fn from(res: SearchResult) -> Self {
        Self {
            id: res.id,
            score: res.score,
            metadata: res.metadata,
            matched_signals: res.matched_signals,
            provenance: res.provenance,
        }
    }
}

/// High-level facade for agent-oriented memory management (`remember`, `recall`, `forget`, `relate`).
///
/// `AgentMemory` wraps an underlying `MemFuse` engine instance, providing agent-friendly terms
/// for text memory persistence, semantic retrieval, deletion, and knowledge graph relations.
#[derive(Clone)]
pub struct AgentMemory {
    engine: Arc<MemFuse>,
}

impl AgentMemory {
    /// Constructs a new `AgentMemory` wrapper around a `MemFuse` engine instance.
    pub fn new(engine: MemFuse) -> Self {
        Self {
            engine: Arc::new(engine),
        }
    }

    /// Constructs a new `AgentMemory` wrapper around an `Arc<MemFuse>` engine instance.
    pub fn new_arc(engine: Arc<MemFuse>) -> Self {
        Self { engine }
    }

    /// Returns a reference to the underlying `MemFuse` engine instance.
    pub fn engine(&self) -> &MemFuse {
        &self.engine
    }

    /// Stores a text memory entry with optional JSON metadata.
    ///
    /// Generates a unique memory identifier and delegates directly to `MemFuse::insert_text_only`.
    pub async fn remember(
        &self,
        text: &str,
        metadata: Option<Value>,
    ) -> Result<MemoryId, MemFuseError> {
        let id = uuid::Uuid::new_v4().to_string();
        self.engine.insert_text_only(&id, text, metadata).await?;
        Ok(MemoryId(id))
    }

    /// Retrieves top-k matching memories for a natural language query text string.
    ///
    /// Delegates directly to `MemFuse::search_text` and transforms each `SearchResult` into a `Memory`.
    pub async fn recall(&self, query: &str, k: usize) -> Result<Vec<Memory>, MemFuseError> {
        let results = self.engine.search_text(query, k).await?;
        Ok(results.into_iter().map(Memory::from).collect())
    }

    /// Deletes a memory entry by its `MemoryId`.
    ///
    /// Delegates directly to `MemFuse::delete`.
    pub async fn forget(&self, id: &MemoryId) -> Result<(), MemFuseError> {
        self.engine.delete(id.as_str()).await
    }

    /// Establishes a directed knowledge-graph relationship between two memories with a given label.
    ///
    /// Delegates directly to `MemFuse::relate`.
    pub async fn relate(
        &self,
        from: &MemoryId,
        to: &MemoryId,
        label: &str,
    ) -> Result<(), MemFuseError> {
        self.engine.relate(from.as_str(), to.as_str(), label).await
    }
}
