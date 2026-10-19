// FILE-CONTEXT
// ZWECK: Sammlung/Collection-Namespace Verwaltung und gemeinsame Hilfsfunktionen.
// INVARIANTEN: Strikte Isolation durch Präfixe; doc_keys (key_type=1) halten nur Metadaten (keine Vektoren).
// NICHT-OFFENSICHTLICH: kv_locks schützt Mutationen zur Vermeidung von TOCTOU-Kollisionsrassen auf Key-Ebene.
// STAND: TS:2026-08-29T17:22:29Z (SESSION: 0dcb9f3b)

//! Logically isolated Collections inside the Contextra database.
// INVARIANT: Logische Isolation (Namespaces).
// PREFIXING: Jeder Key im LSM bekommt das Prefix `__col:{name}:\x00`.

pub mod kv_lock;

#[cfg(not(loom))]
pub mod crud;
#[cfg(not(loom))]
pub mod maintenance;
#[cfg(not(loom))]
pub mod query_builder;
#[cfg(not(loom))]
pub mod relate;
#[cfg(not(loom))]
pub mod search;
#[cfg(not(loom))]
pub mod tx;

#[cfg(not(loom))]
#[cfg(test)]
#[allow(deprecated)]
mod tests;

#[cfg(not(loom))]
use contextra_graph::CsrGraph;
#[cfg(not(loom))]
use contextra_ports::{
    Clock, MetricsSink, NoopMetricsSink, StorageEngine, SystemClock, TextEmbeddingEngine,
    VectorIndex,
};
#[cfg(not(loom))]
use contextra_store::LsmStorage;
#[cfg(not(loom))]
use contextra_text::inverted::InvertedIndex;
#[cfg(not(loom))]
use contextra_text::Language;
use contextra_types::{ContextraError, DocId, Result, TxId};
#[cfg(not(loom))]
use contextra_vector::HnswIndex;
use serde::{Deserialize, Serialize};
#[cfg(not(loom))]
use std::sync::atomic::{AtomicBool, AtomicU64};
#[cfg(not(loom))]
use std::sync::Arc;

/// Report summarizing the results of loading vector index entries from storage.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexLoadReport {
    /// Total number of storage entries scanned.
    pub scanned: u64,
    /// Number of storage entries successfully decoded into `StoredDocument`.
    pub decoded: u64,
    /// Number of storage entries that failed deserialization.
    pub skipped_undecodable: u64,
    /// Number of documents that failed vector index insertion.
    pub failed_inserts: u64,
    /// Number of legacy v1 documents migrated to v2 format on disk during index load.
    #[serde(default)]
    pub migrated_v1: u64,
    /// Duration taken to complete the index load operation.
    pub took: std::time::Duration,
}

/// Format version identifier for on-disk `StoredDocument` payloads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoredDocumentVersion {
    /// Legacy JSON text format (v1).
    V1,
    /// Binary format (v2) with magic header `b"CXSD"` and 0x02 version byte.
    V2,
}

/// Magic header byte signature for binary `StoredDocument` v2 payloads ("CXSD").
pub const STORED_DOCUMENT_MAGIC: [u8; 4] = *b"CXSD";
/// Format version byte for `StoredDocument` v2 payloads.
pub const STORED_DOCUMENT_VERSION_V2: u8 = 2;

/// Policy controlling error handling during collection index loading.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoadPolicy {
    /// Fail immediately on the first undecodable document or failed index insertion.
    Strict,
    /// Allow up to `max_skipped` total skipped records (undecodable + failed inserts).
    /// Fails if total skipped records exceed `max_skipped`.
    Lenient {
        /// Maximum allowable skipped records before returning an error.
        max_skipped: u64,
    },
}

impl Default for LoadPolicy {
    fn default() -> Self {
        Self::Strict
    }
}

/// Configuration for a collection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionConfig {
    /// Configurable backpressure delay in milliseconds when system pressure is Critical.
    /// Default: Some(50)
    pub backpressure_delay_ms: Option<u64>,
}

impl Default for CollectionConfig {
    fn default() -> Self {
        Self {
            backpressure_delay_ms: Some(50),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StoredDocument {
    pub id: String,
    pub embedding: Vec<f32>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Deserialize)]
struct StoredDocumentUnversioned {
    id: String,
    embedding: Vec<f32>,
    metadata: Option<serde_json::Value>,
}

impl StoredDocument {
    /// Encodes this document into v2 binary format:
    /// Magic (`b"CXSD"`, 4B) | Version (`0x02`, 1B) | `id_len` (4B BE) | `id` | `emb_len` (4B BE) | `embedding` (4B LE per f32) | `meta_len` (4B BE) | `metadata` (JSON)
    pub fn encode_v2(&self) -> Result<Vec<u8>> {
        for &val in &self.embedding {
            if !val.is_finite() {
                return Err(ContextraError::invalid_input(
                    "Cannot encode StoredDocument v2: embedding contains non-finite f32 value",
                ));
            }
        }

        let id_bytes = self.id.as_bytes();
        let id_len = u32::try_from(id_bytes.len()).map_err(|_| {
            ContextraError::invalid_input("Document ID length exceeds u32 limit for v2 binary encoding")
        })?;

        let emb_len = u32::try_from(self.embedding.len()).map_err(|_| {
            ContextraError::invalid_input("Embedding vector length exceeds u32 limit for v2 binary encoding")
        })?;

        let meta_bytes = match &self.metadata {
            Some(meta) => serde_json::to_vec(meta).map_err(|e| {
                ContextraError::Serialization(format!("Failed to serialize metadata for v2 binary encoding: {e}"))
            })?,
            None => Vec::new(),
        };

        let meta_len = u32::try_from(meta_bytes.len()).map_err(|_| {
            ContextraError::invalid_input("Metadata byte length exceeds u32 limit for v2 binary encoding")
        })?;

        let total_cap = 5 + 4 + id_bytes.len() + 4 + (self.embedding.len() * 4) + 4 + meta_bytes.len();
        let mut buf = Vec::with_capacity(total_cap);

        buf.extend_from_slice(&STORED_DOCUMENT_MAGIC);
        buf.push(STORED_DOCUMENT_VERSION_V2);

        buf.extend_from_slice(&id_len.to_be_bytes());
        buf.extend_from_slice(id_bytes);

        buf.extend_from_slice(&emb_len.to_be_bytes());
        for &f in &self.embedding {
            buf.extend_from_slice(&f.to_le_bytes());
        }

        buf.extend_from_slice(&meta_len.to_be_bytes());
        if !meta_bytes.is_empty() {
            buf.extend_from_slice(&meta_bytes);
        }

        Ok(buf)
    }

    /// Decodes a `StoredDocument` from bytes, accepting both v1 (JSON text) and v2 binary formats.
    pub fn from_slice(bytes: &[u8]) -> Result<Self> {
        Self::from_slice_versioned(bytes).map(|(doc, _)| doc)
    }

    /// Decodes a `StoredDocument` from bytes and returns the format version along with the document.
    pub fn from_slice_versioned(bytes: &[u8]) -> Result<(Self, StoredDocumentVersion)> {
        if bytes.len() >= 4 && &bytes[0..4] == STORED_DOCUMENT_MAGIC {
            if bytes.len() < 5 {
                return Err(ContextraError::Serialization(
                    "Truncated StoredDocument v2 binary header".into(),
                ));
            }
            let version = bytes[4];
            if version != STORED_DOCUMENT_VERSION_V2 {
                return Err(ContextraError::Serialization(format!(
                    "Unsupported StoredDocument binary version: {version}"
                )));
            }

            let mut offset = 5;

            if bytes.len() < offset + 4 {
                return Err(ContextraError::Serialization(
                    "Truncated StoredDocument v2: missing id_len".into(),
                ));
            }
            let id_len = u32::from_be_bytes([
                bytes[offset],
                bytes[offset + 1],
                bytes[offset + 2],
                bytes[offset + 3],
            ]) as usize;
            offset += 4;

            if bytes.len() < offset + id_len {
                return Err(ContextraError::Serialization(
                    "Truncated StoredDocument v2: missing id bytes".into(),
                ));
            }
            let id = std::str::from_utf8(&bytes[offset..offset + id_len])
                .map_err(|e| {
                    ContextraError::Serialization(format!("Invalid UTF-8 in StoredDocument v2 id: {e}"))
                })?
                .to_string();
            offset += id_len;

            if bytes.len() < offset + 4 {
                return Err(ContextraError::Serialization(
                    "Truncated StoredDocument v2: missing emb_len".into(),
                ));
            }
            let emb_len = u32::from_be_bytes([
                bytes[offset],
                bytes[offset + 1],
                bytes[offset + 2],
                bytes[offset + 3],
            ]) as usize;
            offset += 4;

            let emb_bytes_len = emb_len.checked_mul(4).ok_or_else(|| {
                ContextraError::Serialization("Overflow calculating embedding byte size".into())
            })?;

            if bytes.len() < offset + emb_bytes_len {
                return Err(ContextraError::Serialization(
                    "Truncated StoredDocument v2: missing embedding bytes".into(),
                ));
            }

            let mut embedding = Vec::with_capacity(emb_len);
            for chunk in bytes[offset..offset + emb_bytes_len].chunks_exact(4) {
                let arr: [u8; 4] = chunk.try_into().map_err(|_| {
                    ContextraError::Serialization("Slice conversion failed for f32 byte chunk".into())
                })?;
                let val = f32::from_le_bytes(arr);
                if !val.is_finite() {
                    return Err(ContextraError::Serialization(
                        "Decoded StoredDocument v2 contains non-finite f32 embedding value".into(),
                    ));
                }
                embedding.push(val);
            }
            offset += emb_bytes_len;

            if bytes.len() < offset + 4 {
                return Err(ContextraError::Serialization(
                    "Truncated StoredDocument v2: missing meta_len".into(),
                ));
            }
            let meta_len = u32::from_be_bytes([
                bytes[offset],
                bytes[offset + 1],
                bytes[offset + 2],
                bytes[offset + 3],
            ]) as usize;
            offset += 4;

            if bytes.len() < offset + meta_len {
                return Err(ContextraError::Serialization(
                    "Truncated StoredDocument v2: missing metadata bytes".into(),
                ));
            }

            let metadata = if meta_len > 0 {
                let meta_slice = &bytes[offset..offset + meta_len];
                let val: serde_json::Value = serde_json::from_slice(meta_slice).map_err(|e| {
                    ContextraError::Serialization(format!("Invalid metadata JSON in StoredDocument v2: {e}"))
                })?;
                Some(val)
            } else {
                None
            };
            offset += meta_len;

            if offset != bytes.len() {
                return Err(ContextraError::Serialization(
                    "Trailing bytes after StoredDocument v2 binary payload".into(),
                ));
            }

            Ok((
                StoredDocument {
                    id,
                    embedding,
                    metadata,
                },
                StoredDocumentVersion::V2,
            ))
        } else {
            // Attempt v1 (JSON) deserialization
            let doc: StoredDocumentUnversioned = serde_json::from_slice(bytes).map_err(|e| {
                ContextraError::Serialization(format!(
                    "Failed to deserialize StoredDocument (not v2 binary magic CXSD nor valid v1 JSON): {e}"
                ))
            })?;

            for &val in &doc.embedding {
                if !val.is_finite() {
                    return Err(ContextraError::Serialization(
                        "Decoded StoredDocument v1 contains non-finite f32 embedding value".into(),
                    ));
                }
            }

            Ok((
                StoredDocument {
                    id: doc.id,
                    embedding: doc.embedding,
                    metadata: doc.metadata,
                },
                StoredDocumentVersion::V1,
            ))
        }
    }
}

impl Serialize for StoredDocument {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let bytes = self.encode_v2().map_err(serde::ser::Error::custom)?;
        serializer.serialize_bytes(&bytes)
    }
}

impl<'de> Deserialize<'de> for StoredDocument {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = StoredDocument;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a v1 JSON StoredDocument or v2 binary StoredDocument")
            }

            fn visit_bytes<E>(self, v: &[u8]) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                StoredDocument::from_slice(v).map_err(serde::de::Error::custom)
            }

            fn visit_byte_buf<E>(self, v: Vec<u8>) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                StoredDocument::from_slice(&v).map_err(serde::de::Error::custom)
            }

            fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                StoredDocument::from_slice(v.as_bytes()).map_err(serde::de::Error::custom)
            }

            fn visit_map<M>(self, map: M) -> std::result::Result<Self::Value, M::Error>
            where
                M: serde::de::MapAccess<'de>,
            {
                let helper = StoredDocumentUnversioned::deserialize(
                    serde::de::value::MapAccessDeserializer::new(map),
                )?;
                for &val in &helper.embedding {
                    if !val.is_finite() {
                        return Err(serde::de::Error::custom(
                            "Decoded StoredDocument contains non-finite f32 embedding value",
                        ));
                    }
                }
                Ok(StoredDocument {
                    id: helper.id,
                    embedding: helper.embedding,
                    metadata: helper.metadata,
                })
            }
        }

        deserializer.deserialize_any(Visitor)
    }
}

/// Leichtgewichtige Metadaten (für doc_key, key_type=1) — KEIN Embedding.
/// Wird für DocId-basierte Hydration nach HNSW/BM25-Suche verwendet.
#[derive(Debug, Serialize, Deserialize)]
pub struct StoredDocumentMeta {
    pub id: String,
    pub metadata: Option<serde_json::Value>,
}

impl From<&StoredDocument> for StoredDocumentMeta {
    fn from(doc: &StoredDocument) -> Self {
        Self {
            id: doc.id.clone(),
            metadata: doc.metadata.clone(),
        }
    }
}

/// Parses an LLM response string into an f32 importance score in `[0.0, 1.0]`.
#[deprecated(note = "Parsing LLM responses should happen outside contextra-db")]
pub fn parse_importance_score(response: &str) -> f32 {
    for token in response.split_whitespace() {
        if let Ok(val) = token.parse::<f32>() {
            if val.is_finite() {
                return val.clamp(0.0, 1.0);
            }
        }
        let trimmed = token.trim_matches(|c: char| !c.is_ascii_digit() && c != '.');
        if !trimmed.is_empty() {
            if let Ok(val) = trimmed.parse::<f32>() {
                if val.is_finite() {
                    return val.clamp(0.0, 1.0);
                }
            }
        }
    }
    0.5
}

/// Computes a non-LLM heuristic baseline importance score based on text length and character entropy.
pub fn compute_default_importance(text_opt: Option<&str>) -> contextra_types::ImportanceScore {
    let text = match text_opt {
        Some(t) if !t.is_empty() => t,
        _ => return contextra_types::ImportanceScore::new(0.5),
    };
    let char_count = text.chars().count();
    if char_count == 0 {
        return contextra_types::ImportanceScore::new(0.5);
    }
    let unique_chars = text.chars().collect::<std::collections::HashSet<_>>().len() as f32;
    let entropy_ratio = unique_chars / char_count as f32;
    let len_factor = (char_count as f32 / 500.0).clamp(0.1, 0.8);
    let raw = (len_factor * 0.5) + (entropy_ratio * 0.5);
    contextra_types::ImportanceScore::new(raw)
}

/// Ensures document metadata contains a valid `MemoryImportance` JSON payload.
pub fn ensure_importance_metadata(
    metadata: &mut Option<serde_json::Value>,
    tx: TxId,
    text_opt: Option<&str>,
) {
    let meta_obj = match metadata {
        Some(serde_json::Value::Object(ref mut map)) => map,
        _ => {
            *metadata = Some(serde_json::json!({}));
            if let Some(serde_json::Value::Object(ref mut map)) = metadata {
                map
            } else {
                return;
            }
        }
    };

    // Determine default decay function based on explicit decay_function or memory_type
    let decay = if let Some(decay_val) = meta_obj.get("decay_function") {
        serde_json::from_value::<contextra_types::DecayFunction>(decay_val.clone())
            .unwrap_or(contextra_types::DecayFunction::None)
    } else if let Some(mem_type_val) = meta_obj.get("memory_type") {
        if let Ok(mem_type) =
            serde_json::from_value::<contextra_types::MemoryType>(mem_type_val.clone())
        {
            mem_type.default_decay()
        } else {
            contextra_types::DecayFunction::None
        }
    } else {
        contextra_types::DecayFunction::None
    };

    // Also populate default TTL if memory_type defines one and not already set
    if !meta_obj.contains_key("ttl_tx") {
        if let Some(mem_type_val) = meta_obj.get("memory_type") {
            if let Ok(mem_type) =
                serde_json::from_value::<contextra_types::MemoryType>(mem_type_val.clone())
            {
                if let Some(ttl) = mem_type.default_ttl_tx() {
                    meta_obj.insert("ttl_tx".to_string(), serde_json::json!(ttl));
                }
            }
        }
    }

    if let Some(imp_val) = meta_obj.get("importance").cloned() {
        if serde_json::from_value::<contextra_types::MemoryImportance>(imp_val.clone()).is_ok() {
            return;
        } else if let Some(raw_f64) = imp_val.as_f64() {
            let imp = contextra_types::MemoryImportance::new(
                contextra_types::ImportanceScore::new(raw_f64 as f32),
                decay,
                tx,
            );
            if let Ok(val) = serde_json::to_value(imp) {
                meta_obj.insert("importance".to_string(), val);
            }
            return;
        }
    }

    let base_score = compute_default_importance(text_opt);
    let imp = contextra_types::MemoryImportance::new(base_score, decay, tx);
    if let Ok(val) = serde_json::to_value(imp) {
        meta_obj.insert("importance".to_string(), val);
    }
}

/// Extracts the effective importance score of a document at a given transaction ID.
pub fn extract_effective_importance(metadata: &Option<serde_json::Value>, now_tx: TxId) -> f32 {
    let Some(meta) = metadata else {
        return 1.0;
    };
    let Some(obj) = meta.as_object() else {
        return 1.0;
    };
    let Some(imp_val) = obj.get("importance") else {
        return 1.0;
    };

    if let Ok(imp) = serde_json::from_value::<contextra_types::MemoryImportance>(imp_val.clone()) {
        imp.effective_score(now_tx)
    } else if let Some(raw_f64) = imp_val.as_f64() {
        contextra_types::ImportanceScore::new(raw_f64 as f32).value()
    } else {
        1.0
    }
}

/// Helper to unify how we extract text from metadata.
pub(super) fn extract_text(metadata: &Option<serde_json::Value>) -> Option<String> {
    let mut document_text = String::new();
    if let Some(m) = metadata {
        if let Some(m_obj) = m.as_object() {
            if let Some(s) = m_obj.get("contextual_prefix").and_then(|v| v.as_str()) {
                if !s.is_empty() {
                    document_text.push_str(s);
                    document_text.push_str("\n\n");
                }
            }
            if let Some(s) = m_obj.get("text").and_then(|v| v.as_str()) {
                document_text.push_str(s);
                document_text.push(' ');
            }
            if let Some(s) = m_obj.get("content").and_then(|v| v.as_str()) {
                document_text.push_str(s);
                document_text.push(' ');
            }
        }
    }
    if document_text.is_empty() {
        None
    } else {
        Some(document_text.trim().to_string())
    }
}

/// # Concurrency & Lock Hierarchy
///
/// Lock acquisition within `Collection` follows strict ordering to prevent deadlocks:
///
/// 1. `Collection::kv_locks` (`KvKeyLocks`):
///    Key-granular sharded locks protecting key-level mutations (`insert`, `update`, `delete`, `relate`).
/// 2. `Collection::embedder` (`parking_lot::RwLock`):
///    Read/write lock for the configured `TextEmbeddingEngine`. Never acquired before `kv_locks` if both are needed.
///
/// A logically isolated collection of documents (namespace).
///
/// Each collection provides its own vector index and inverted text index,
/// while sharing the underlying LSM-Tree storage with other collections.
#[cfg(not(loom))]
pub struct Collection<S: StorageEngine = LsmStorage, V: VectorIndex = HnswIndex> {
    pub(super) name: String,
    pub(super) prefix: Vec<u8>,
    pub(super) index: Arc<V>,
    pub(super) text_index: InvertedIndex<S>,
    pub(super) graph_index: Arc<CsrGraph>,
    pub(super) storage: Arc<S>,
    pub(super) next_tx: Arc<AtomicU64>,
    pub(super) dimension: usize,
    pub(super) embedder: parking_lot::RwLock<Option<Arc<dyn TextEmbeddingEngine>>>,
    pub consolidation_guard: Arc<tokio::sync::Mutex<()>>,
    pub(super) kv_locks: Arc<kv_lock::KvKeyLocks>,
    /// Optionaler tenant-isolierter KV-Cache-Store zur automatischen KV-Cache-Bereinigung bei Rollbacks.
    #[cfg(feature = "encryption-at-rest")]
    pub(super) kv_store: Option<Arc<contextra_crypto::TenantIsolatedKvStore>>,
    /// Zählt Graph-Mutationen seit letzter Community Detection.
    pub(super) mutations_since_community_detection: Arc<AtomicU64>,
    /// Trigger-Schwelle: bei Überschreitung wird Community Detection geplant (Default: 100, 0 = deaktiviert).
    pub(super) community_detection_trigger_threshold: Arc<AtomicU64>,
    /// Koordination des Konsolidierungslaufs zur Vermeidung von Double-Triggern (Engine + Scheduler).
    pub(super) consolidation_in_progress: Arc<AtomicBool>,
    /// Auto extraction generator, config, and mode binding for document ingestion hooks.
    pub(super) auto_extraction:
        Arc<parking_lot::RwLock<Option<crud::auto_extraction::AutoExtractionBinding>>>,
    /// Collection configuration (e.g. backpressure settings).
    pub(super) config: parking_lot::RwLock<CollectionConfig>,
    /// Watch receiver for monitoring system pressure levels.
    pub(super) pressure_rx:
        parking_lot::RwLock<Option<tokio::sync::watch::Receiver<contextra_store::SystemPressure>>>,
    /// Injected clock port for time operations.
    pub(super) clock: parking_lot::RwLock<Arc<dyn Clock>>,
    /// Injected metrics sink port for operational observability.
    pub(super) metrics: parking_lot::RwLock<Arc<dyn MetricsSink>>,
    /// Stores report of the last index load operation.
    pub(super) last_load_report: parking_lot::Mutex<Option<IndexLoadReport>>,
}

#[cfg(not(loom))]
impl<S: StorageEngine, V: VectorIndex> Clone for Collection<S, V> {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            prefix: self.prefix.clone(),
            index: self.index.clone(),
            text_index: self.text_index.clone(),
            graph_index: self.graph_index.clone(),
            storage: self.storage.clone(),
            next_tx: self.next_tx.clone(),
            dimension: self.dimension,
            embedder: parking_lot::RwLock::new(self.embedder.read().as_ref().map(Arc::clone)),
            consolidation_guard: self.consolidation_guard.clone(),
            kv_locks: self.kv_locks.clone(),
            #[cfg(feature = "encryption-at-rest")]
            kv_store: self.kv_store.clone(),
            mutations_since_community_detection: self.mutations_since_community_detection.clone(),
            community_detection_trigger_threshold: self
                .community_detection_trigger_threshold
                .clone(),
            consolidation_in_progress: self.consolidation_in_progress.clone(),
            auto_extraction: self.auto_extraction.clone(),
            config: parking_lot::RwLock::new(self.config.read().clone()),
            pressure_rx: parking_lot::RwLock::new(self.pressure_rx.read().clone()),
            clock: parking_lot::RwLock::new(self.clock.read().clone()),
            metrics: parking_lot::RwLock::new(self.metrics.read().clone()),
            last_load_report: parking_lot::Mutex::new(self.last_load_report.lock().clone()),
        }
    }
}

#[cfg(not(loom))]
impl<S: StorageEngine> Collection<S, HnswIndex> {
    /// Convenience constructor for creating a `Collection` with `HnswIndex`.
    pub fn with_hnsw(
        name: String,
        storage: Arc<S>,
        index: Arc<HnswIndex>,
        graph_index: Arc<CsrGraph>,
        next_tx: Arc<AtomicU64>,
        dimension: usize,
        language: Language,
    ) -> Self {
        Self::new(
            name,
            storage,
            index,
            graph_index,
            next_tx,
            dimension,
            language,
        )
    }
}

#[cfg(not(loom))]
impl<S: StorageEngine, V: VectorIndex> Collection<S, V> {
    /// werden als "vergessen" markiert und gelöscht.
    pub const DECAY_DELETION_THRESHOLD: f32 = 0.05;

    /// Creates a new `Collection` instance with explicit language configuration.
    ///
    /// The `language` parameter controls the BM25 tokenizer. Use `Language::German`
    /// for German compound splitting, `Language::English` (default) for standard
    /// whitespace tokenization.
    pub fn new(
        name: String,
        storage: Arc<S>,
        index: Arc<V>,
        graph_index: Arc<CsrGraph>,
        next_tx: Arc<AtomicU64>,
        dimension: usize,
        language: Language,
    ) -> Self {
        let prefix = if name == "default" {
            b"".to_vec()
        } else {
            format!("__col:{}:\x00", name).into_bytes()
        };

        let text_index = InvertedIndex::new_with_language(storage.clone(), &name, language);

        let pressure_rx = None;

        Self {
            name,
            prefix,
            index,
            text_index,
            graph_index,
            storage,
            next_tx,
            dimension,
            embedder: parking_lot::RwLock::new(None),
            consolidation_guard: Arc::new(tokio::sync::Mutex::new(())),
            kv_locks: Arc::new(kv_lock::KvKeyLocks::new()),
            #[cfg(feature = "encryption-at-rest")]
            kv_store: None,
            mutations_since_community_detection: Arc::new(AtomicU64::new(0)),
            community_detection_trigger_threshold: Arc::new(AtomicU64::new(100)),
            consolidation_in_progress: Arc::new(AtomicBool::new(false)),
            auto_extraction: Arc::new(parking_lot::RwLock::new(None)),
            config: parking_lot::RwLock::new(CollectionConfig::default()),
            pressure_rx: parking_lot::RwLock::new(pressure_rx),
            clock: parking_lot::RwLock::new(Arc::new(SystemClock::new())),
            metrics: parking_lot::RwLock::new(Arc::new(NoopMetricsSink)),
            last_load_report: parking_lot::Mutex::new(None),
        }
    }

    /// Sets or overrides the metrics sink implementation (builder version).
    pub fn with_metrics_sink(self, metrics: Arc<dyn MetricsSink>) -> Self {
        *self.metrics.write() = metrics;
        self
    }

    /// Sets or overrides the metrics sink implementation.
    pub fn set_metrics_sink(&self, metrics: Arc<dyn MetricsSink>) {
        *self.metrics.write() = metrics;
    }

    /// Returns a cloned reference to the active metrics sink.
    pub fn metrics_sink(&self) -> Arc<dyn MetricsSink> {
        self.metrics.read().clone()
    }

    /// Returns the report from the most recent index load operation, if any.
    pub fn last_load_report(&self) -> Option<IndexLoadReport> {
        self.last_load_report.lock().clone()
    }

    /// Sets or overrides the clock implementation (builder version).
    pub fn with_clock(self, clock: Arc<dyn Clock>) -> Self {
        *self.clock.write() = clock;
        self
    }

    /// Sets or overrides the clock implementation.
    pub fn set_clock(&self, clock: Arc<dyn Clock>) {
        *self.clock.write() = clock;
    }

    /// Returns a cloned reference to the injected clock.
    pub fn clock(&self) -> Arc<dyn Clock> {
        self.clock.read().clone()
    }

    /// Sets or updates the collection configuration.
    pub fn with_config(self, config: CollectionConfig) -> Self {
        *self.config.write() = config;
        self
    }

    /// Sets or updates the collection configuration.
    pub fn set_config(&self, config: CollectionConfig) {
        *self.config.write() = config;
    }

    /// Returns a copy of the current collection configuration.
    pub fn config(&self) -> CollectionConfig {
        self.config.read().clone()
    }

    /// Sets or overrides the pressure receiver channel.
    pub fn with_pressure_receiver(
        self,
        pressure_rx: tokio::sync::watch::Receiver<contextra_store::SystemPressure>,
    ) -> Self {
        *self.pressure_rx.write() = Some(pressure_rx);
        self
    }

    /// Sets or overrides the pressure receiver channel.
    pub fn set_pressure_receiver(
        &self,
        pressure_rx: tokio::sync::watch::Receiver<contextra_store::SystemPressure>,
    ) {
        *self.pressure_rx.write() = Some(pressure_rx);
    }

    /// Returns a reference handle to the collection's consolidation lock flag.
    pub fn consolidation_in_progress(&self) -> Arc<AtomicBool> {
        self.consolidation_in_progress.clone()
    }

    /// Returns the tenant ID for this collection (extracted from `TenantScopedStorage` if present,
    /// otherwise returning an error if tenant resolution is ambiguous or unresolvable).
    pub fn try_tenant_id(&self) -> Result<contextra_types::TenantId> {
        Err(contextra_types::ContextraError::InvalidInput(
            "TenantId resolution failed: underlying storage engine is not bound to a specific tenant".to_string()
        ))
    }

    /// Generates a collection prefix using TenantKeyCodec.
    pub fn tenant_collection_prefix(
        &self,
        codec: &contextra_store::tenant_codec::TenantKeyCodec,
        collection: &contextra_types::CollectionId,
    ) -> Vec<u8> {
        codec.collection_prefix(collection)
    }

    /// Encodes a graph key using TenantKeyCodec.
    pub fn tenant_encode_graph_key(
        &self,
        codec: &contextra_store::tenant_codec::TenantKeyCodec,
        collection: &contextra_types::CollectionId,
        entity_id: u64,
    ) -> Vec<u8> {
        codec.encode_graph_key(collection, entity_id)
    }

    /// Evaluates point lookup traversal metrics on the underlying LSM storage engine.
    pub async fn point_lookup_metrics(
        &self,
        storage: &contextra_store::LsmStorage,
        key: &[u8],
    ) -> (usize, usize, usize, usize, bool) {
        storage.point_lookup_metrics(key).await
    }

    /// Attaches a `TenantIsolatedKvStore` to the collection for KV cache rollback management.
    #[cfg(feature = "encryption-at-rest")]
    pub fn with_kv_store(mut self, kv_store: Arc<contextra_crypto::TenantIsolatedKvStore>) -> Self {
        self.kv_store = Some(kv_store);
        self
    }

    /// Attaches a `TenantIsolatedKvStore` to the collection for KV cache rollback management.
    #[cfg(feature = "encryption-at-rest")]
    pub fn set_kv_store(&mut self, kv_store: Arc<contextra_crypto::TenantIsolatedKvStore>) {
        self.kv_store = Some(kv_store);
    }

    #[cfg(all(not(feature = "encryption-at-rest"), test))]
    pub fn set_kv_store(&mut self, _kv_store: Arc<contextra_crypto::TenantIsolatedKvStore>) {}

    #[cfg(all(not(feature = "encryption-at-rest"), test))]
    pub fn with_kv_store(self, _kv_store: Arc<contextra_crypto::TenantIsolatedKvStore>) -> Self {
        self
    }

    /// Returns a reference to the attached `TenantIsolatedKvStore`, if configured.
    #[cfg(feature = "encryption-at-rest")]
    pub fn kv_store(&self) -> Option<&Arc<contextra_crypto::TenantIsolatedKvStore>> {
        self.kv_store.as_ref()
    }

    /// Returns a reference to the attached `TenantIsolatedKvStore`, if configured.
    #[cfg(not(feature = "encryption-at-rest"))]
    pub fn kv_store(&self) -> Option<&Arc<contextra_crypto::TenantIsolatedKvStore>> {
        None
    }

    /// Sets the text embedder for this collection (consuming version).
    #[tracing::instrument(level = "trace", skip(self, embedder))]
    pub fn with_embedder(self, embedder: Arc<dyn TextEmbeddingEngine>) -> Self {
        {
            let mut guard = self.embedder.write();
            *guard = Some(embedder);
        }
        self
    }

    /// Configures the text embedder for this collection.
    #[tracing::instrument(level = "trace", skip(self, embedder))]
    pub async fn set_embedder(&self, embedder: Arc<dyn TextEmbeddingEngine>) -> Result<()> {
        let mut guard = self.embedder.write();
        *guard = Some(embedder);
        Ok(())
    }

    /// Returns a reference to the vector index of this collection.
    pub fn vector_index(&self) -> &Arc<V> {
        &self.index
    }

    /// Internal helper to generate namespaced keys.
    /// key_type: 0 = user key, 1 = docid mapping, 2 = relationship, 3 = tx intent, 4 = system/community
    pub fn namespaced_key(&self, key: &[u8], key_type: u8) -> Vec<u8> {
        if self.name == "default" {
            match key_type {
                0 => key.to_vec(),
                1 => {
                    let mut k = Vec::with_capacity(8 + key.len());
                    k.extend_from_slice(b"__docid:");
                    k.extend_from_slice(key);
                    k
                }
                2 => {
                    let mut k = Vec::with_capacity(6 + key.len());
                    k.extend_from_slice(b"__rel:");
                    k.extend_from_slice(key);
                    k
                }
                3 => {
                    let mut k = b"__tx_intent:".to_vec();
                    k.extend_from_slice(key);
                    k
                }
                4 => key.to_vec(),
                _ => key.to_vec(),
            }
        } else {
            let mut k = Vec::with_capacity(self.prefix.len() + 1 + key.len());
            k.extend_from_slice(&self.prefix);
            k.push(key_type);
            k.extend_from_slice(key);
            k
        }
    }

    /// Returns a reference to the underlying storage engine.
    pub fn storage(&self) -> &Arc<S> {
        &self.storage
    }

    /// Returns the namespaced prefix for user document keys in this collection.
    pub fn user_key_prefix(&self) -> Vec<u8> {
        self.namespaced_key(b"", 0)
    }

    /// Returns the bound TenantId for this collection.
    pub fn tenant_id(&self) -> contextra_types::TenantId {
        contextra_types::TenantId::SYSTEM
    }

    /// Returns the name of the collection.
    #[tracing::instrument(level = "trace", skip(self))]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the number of documents in the collection.
    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn len(&self) -> usize {
        self.index.len().await
    }

    /// Returns the vector dimension for this collection.
    #[tracing::instrument(level = "trace", skip(self))]
    pub fn dimension(&self) -> usize {
        self.dimension
    }

    /// Returns true if the collection is empty.
    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn is_empty(&self) -> bool {
        self.index.is_empty().await
    }

    /// Performs a range scan of documents in the collection using the default `Strict` load policy.
    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn load_index(&self) -> Result<()> {
        self.load_index_with_report(LoadPolicy::Strict).await?;
        Ok(())
    }

    /// Performs a range scan of documents in the collection and rebuilds the vector index,
    /// returning an [`IndexLoadReport`] according to the specified [`LoadPolicy`].
    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn load_index_with_report(&self, policy: LoadPolicy) -> Result<IndexLoadReport> {
        let start_ticks = self.clock().monotonic_ticks();

        let scan_prefix = if self.name == "default" {
            b"".to_vec()
        } else {
            let mut p = self.prefix.clone();
            p.push(0); // key_type=0
            p
        };

        let entries = self.storage.scan_prefix(&scan_prefix).await?;
        let tx = self.allocate_tx()?;

        let mut scanned: u64 = 0;
        let mut decoded: u64 = 0;
        let mut skipped_undecodable: u64 = 0;
        let mut failed_inserts: u64 = 0;
        let mut migrated_v1: u64 = 0;

        let mut v1_migrations: Vec<(Vec<u8>, Vec<u8>)> = Vec::new();

        const INDEX_LOAD_SKIPPED_TOTAL: &str = "contextra_index_load_skipped_total";

        for (k, v) in entries {
            if self.name == "default" && k.starts_with(b"__") {
                continue;
            }

            scanned += 1;

            let (stored, version) = match StoredDocument::from_slice_versioned(&v) {
                Ok(res) => res,
                Err(e) => {
                    skipped_undecodable += 1;
                    crate::metrics_names::record_counter_safe(
                        &*self.metrics_sink(),
                        INDEX_LOAD_SKIPPED_TOTAL,
                        1,
                        &[("collection", &self.name)],
                    );

                    let short_hash = blake3::hash(&k).to_hex()[..16].to_string();

                    match policy {
                        LoadPolicy::Strict => {
                            return Err(ContextraError::Index(format!(
                                "Undecodable document at key hash {short_hash}: {e}"
                            )));
                        }
                        LoadPolicy::Lenient { max_skipped } => {
                            let total_skipped = skipped_undecodable + failed_inserts;
                            if total_skipped > max_skipped {
                                return Err(ContextraError::Index(format!(
                                    "Exceeded max_skipped limit ({max_skipped}) with undecodable document at key hash {short_hash}: {e}"
                                )));
                            }
                            continue;
                        }
                    }
                }
            };

            if version == StoredDocumentVersion::V1 {
                let v2_bytes = match stored.encode_v2() {
                    Ok(b) => b,
                    Err(e) => {
                        let short_hash = blake3::hash(&k).to_hex()[..16].to_string();
                        match policy {
                            LoadPolicy::Strict => {
                                return Err(ContextraError::Index(format!(
                                    "Failed to encode v2 migrated payload for doc {} at key hash {short_hash}: {e}",
                                    stored.id
                                )));
                            }
                            LoadPolicy::Lenient { max_skipped } => {
                                skipped_undecodable += 1;
                                let total_skipped = skipped_undecodable + failed_inserts;
                                if total_skipped > max_skipped {
                                    return Err(ContextraError::Index(format!(
                                        "Exceeded max_skipped limit ({max_skipped}) during v1 migration encoding: {e}"
                                    )));
                                }
                                continue;
                            }
                        }
                    }
                };
                v1_migrations.push((k.clone(), v2_bytes));
            }

            decoded += 1;

            let doc_id = DocId::from_key(&stored.id)?;
            if let Err(e) = self.index.insert(tx, doc_id, &stored.embedding).await {
                failed_inserts += 1;
                crate::metrics_names::record_counter_safe(
                    &*self.metrics_sink(),
                    INDEX_LOAD_SKIPPED_TOTAL,
                    1,
                    &[("collection", &self.name)],
                );
                tracing::warn!(doc_id = ?doc_id, error = %e, "Konnte Dokument bei load_index nicht in Index einfügen");

                let short_hash = blake3::hash(&k).to_hex()[..16].to_string();

                match policy {
                    LoadPolicy::Strict => {
                        return Err(ContextraError::Index(format!(
                            "Failed index insert for doc_id {doc_id:?} at key hash {short_hash}: {e}"
                        )));
                    }
                    LoadPolicy::Lenient { max_skipped } => {
                        let total_skipped = skipped_undecodable + failed_inserts;
                        if total_skipped > max_skipped {
                            return Err(ContextraError::Index(format!(
                                "Exceeded max_skipped limit ({max_skipped}) with failed index insert at key hash {short_hash}: {e}"
                            )));
                        }
                    }
                }
            }
        }

        if !v1_migrations.is_empty() {
            let mig_tx = self.allocate_tx()?;
            for (key, v2_data) in v1_migrations {
                if let Err(e) = self.storage.put(mig_tx, &key, &v2_data).await {
                    if let Err(rb_err) = self.storage.rollback(mig_tx).await {
                        tracing::error!("Failed to rollback migration transaction: {rb_err}");
                    }
                    return Err(ContextraError::Index(format!(
                        "Failed to put v2 document in storage during migration: {e}"
                    )));
                }
                migrated_v1 += 1;
            }
            if let Err(e) = self.storage.commit(mig_tx).await {
                if let Err(rb_err) = self.storage.rollback(mig_tx).await {
                    tracing::error!("Failed to rollback migration transaction: {rb_err}");
                }
                return Err(ContextraError::Index(format!(
                    "Failed to commit migration transaction during load_index: {e}"
                )));
            }
            tracing::info!(
                migrated = migrated_v1,
                collection = %self.name,
                "Successfully migrated v1 documents to v2 binary format"
            );
        }

        self.index.commit(tx).await?;

        let end_ticks = self.clock().monotonic_ticks();
        let took_nanos = end_ticks.saturating_sub(start_ticks);
        let took = std::time::Duration::from_nanos(took_nanos);

        let report = IndexLoadReport {
            scanned,
            decoded,
            skipped_undecodable,
            failed_inserts,
            migrated_v1,
            took,
        };

        *self.last_load_report.lock() = Some(report.clone());

        Ok(report)
    }

    /// Migrates old doc_keys (with Embedding) to new doc_keys (only Metadata).
    /// Safe to call multiple times (idempotent).
    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn migrate_doc_keys_v1(&self) -> Result<u64> {
        let prefix = if self.name == "default" {
            b"__docid:".to_vec()
        } else {
            let mut p = self.prefix.clone();
            p.push(1); // docid mapping type
            p
        };

        let entries = self.storage.scan_prefix(&prefix).await?;
        let mut migrated_count = 0;
        let tx = self.allocate_tx()?;

        for (k, v) in entries {
            // Try parsing as full document first (which indicates it needs migration)
            if let Ok(full) = StoredDocument::from_slice(&v) {
                let meta_only = StoredDocumentMeta::from(&full);
                if let Ok(meta_data) = serde_json::to_vec(&meta_only) {
                    self.storage.put(tx, &k, &meta_data).await?;
                    migrated_count += 1;
                }
            }
        }

        if migrated_count > 0 {
            self.storage.commit(tx).await?;
            tracing::info!(
                "Migrated {} legacy doc_keys to new format in collection '{}'",
                migrated_count,
                self.name
            );
        }

        Ok(migrated_count)
    }

    /// Loads text index statistics from storage.
    pub async fn load_text_stats(&self) -> Result<()> {
        self.text_index.load_stats().await
    }

    /// Sets the mutation threshold for auto-triggered community detection (0 = disabled).
    pub fn set_community_detection_trigger_threshold(&self, threshold: u64) {
        self.community_detection_trigger_threshold
            .store(threshold, std::sync::atomic::Ordering::Relaxed);
    }

    /// Builder method to set the mutation threshold for auto-triggered community detection (0 = disabled).
    pub fn with_community_detection_trigger_threshold(self, threshold: u64) -> Self {
        self.set_community_detection_trigger_threshold(threshold);
        self
    }

    /// Returns the configured mutation threshold for auto-triggered community detection.
    pub fn community_detection_trigger_threshold(&self) -> u64 {
        self.community_detection_trigger_threshold
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Returns the number of graph mutations since the last community detection execution.
    pub fn mutations_since_community_detection(&self) -> u64 {
        self.mutations_since_community_detection
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Increments mutation counter and triggers background community detection if threshold is reached.
    pub(super) fn check_and_trigger_community_detection(&self, count: usize) {
        let threshold = self
            .community_detection_trigger_threshold
            .load(std::sync::atomic::Ordering::Relaxed);
        if threshold == 0 || count == 0 {
            return;
        }

        let prev = self
            .mutations_since_community_detection
            .fetch_add(count as u64, std::sync::atomic::Ordering::Relaxed);

        if prev + (count as u64) >= threshold {
            self.mutations_since_community_detection
                .store(0, std::sync::atomic::Ordering::Relaxed);
            let collection_clone = self.clone();
            tokio::spawn(async move {
                match collection_clone.run_community_detection().await {
                    Ok(assignments) => {
                        tracing::info!(
                            communities = assignments.len(),
                            "Auto-triggered community detection completed"
                        );
                    }
                    Err(e) => {
                        tracing::warn!(
                            error = %e,
                            "Auto-triggered community detection failed"
                        );
                    }
                }
            });
        }
    }
}
