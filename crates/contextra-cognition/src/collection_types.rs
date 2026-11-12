//! # Modul: collection_types.rs
//!
//! **Zweck:** Lokale Datentypen für Dokumente und Metadaten in contextra-cognition (SPEC §6.4, S-63).
//!
//! **Architektur:** Gehört zur Crate `contextra-cognition` (Ebene E2).

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// Dokumentenstruktur für gespeicherte Entitäten und Chunks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredDocument {
    /// Eindeutiger Dokumentbezeichner.
    pub id: String,
    /// Vektoreinbettung.
    pub embedding: Vec<f32>,
    /// Optionale Metadaten im JSON-Format.
    pub metadata: Option<serde_json::Value>,
}

/// Leichtgewichtige Metadaten für Dokumente (ohne Embedding-Vektor).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredDocumentMeta {
    /// Eindeutiger Dokumentbezeichner.
    pub id: String,
    /// Optionale Metadaten im JSON-Format.
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
