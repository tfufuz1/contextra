// FILE-CONTEXT
// STAND: 2026-10-27T00:00:00Z
// ZWECK: Hardened FlatBuffers buffer verification and safe accessor functions
// RELEVANTE SPEZIFIKATION: REQ-WIR-001, REQ-WIR-002, P7 (Zero-Panic)

//! Safe and hardened FlatBuffers verification wrappers.
//!
//! # FlatBuffers Verifikations- und Zugriffs-Primitiven
//!
//! **Ring & Klasse:** Ring 0, Klasse Kern `[K]`.
//!
//! ## Zweck
//! Dieses Modul implementiert gehärtete Verifikationsprüfungen für eingehende FlatBuffers-Nachrichten.
//! Bevor auf ein FlatBuffers-Objekt zugegriffen werden darf, muss der Puffer vollständig auf
//! Puffergrenzen (Bounds), zulässige Tabellen-Offsets, Ausrichtung (Alignment) und Schachtelungstiefe
//! validiert werden.
//!
//! ## Sicherheitsinvariante & Zero-Panic-Garantie
//! Gemäß REQ-WIR-002 und P7 werden ungültige oder bösartige Puffer (z. B. Depth-Bombs, fehlerhafte Offsets)
//! deterministisch abgefangen und führen zu einem Fehler (`FlatBufferError`). Es treten niemals Panics auf.

use crate::generated::contextra_generated::contextra::ipc::{
    Embedding, HyperEdge, HyperEdgeId, RoleBinding, RoleId, ScoredDocument, SearchResponse,
    VectorIndexUpdate,
};
use flatbuffers::VerifierOptions;
use std::fmt;

// ANCHOR[contextra-wire:VER-001]
/// Error type for FlatBuffers verification failures.
///
/// Signalisiert Verletzungen der Spezifikation oder Sicherheitsgrenzen beim Einlesen von Byte-Puffern.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FlatBufferError {
    /// Buffer is empty.
    /// Der übergebene Byte-Puffer ist leer.
    EmptyBuffer,
    /// Buffer exceeds configured maximum message size.
    /// Der Puffer überschreitet die maximal erlaubte Nachrichtengröße (`max_size`).
    BufferTooLarge { size: usize, max_size: usize },
    /// FlatBuffers verifier check failed.
    /// Die interne Verifikation der FlatBuffers-Tabellenstruktur schlug fehl.
    VerificationFailed(String),
    /// Expected file identifier was missing or mismatched.
    /// Die erwartete Dateikennung (`file_identifier "CTRA"`) fehlt oder weicht ab.
    IdentifierMismatch,
}

impl fmt::Display for FlatBufferError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyBuffer => write!(f, "FlatBuffer payload is empty"),
            Self::BufferTooLarge { size, max_size } => write!(
                f,
                "FlatBuffer size ({size} bytes) exceeds maximum limit ({max_size} bytes)"
            ),
            Self::VerificationFailed(reason) => {
                write!(f, "FlatBuffer verification failed: {reason}")
            }
            Self::IdentifierMismatch => write!(f, "FlatBuffer file identifier mismatch"),
        }
    }
}

impl std::error::Error for FlatBufferError {}

// ANCHOR[contextra-wire:VER-002]
/// Configuration parameters for FlatBuffers verification.
///
/// Definiert Schutzgrenzen gegen Denial-of-Service-Angriffe (DoS) durch übergroße oder
/// tief geschachtelte Strukturen.
#[derive(Debug, Clone)]
pub struct FlatBufferConfig {
    /// Maximum allowed buffer size in bytes (default: 10 MiB).
    /// Maximal erlaubte Puffergröße in Bytes (Standard: 10 MiB).
    pub max_message_size: usize,
    /// Maximum depth of nested FlatBuffers tables (default: 64).
    /// Maximal erlaubte Schachtelungstiefe für verknüpfte FlatBuffers-Tabellen (Standard: 64).
    pub max_depth: usize,
    /// Maximum number of tables evaluated during verification (default: 1,000,000).
    /// Maximal geprüfte Anzahl an Tabellen während des Verifikationsdurchlaufs (Standard: 1.000.000).
    pub max_tables: usize,
}

impl Default for FlatBufferConfig {
    // ANCHOR[contextra-wire:VER-003]
    /// Erzeugt Standardwerte mit sicheren Obergrenzen für den Produktionseinsatz.
    fn default() -> Self {
        Self {
            max_message_size: 10 * 1024 * 1024,
            max_depth: 64,
            max_tables: 1_000_000,
        }
    }
}

// ANCHOR[contextra-wire:VER-004]
/// Generic verified root accessor for any FlatBuffers root table `T`.
///
/// # Ablauf von `verify_flatbuffer`:
/// 1. Puffer prüfen: Ist der Puffer leer, wird sofort `Err(FlatBufferError::EmptyBuffer)` zurückgegeben.
/// 2. Längenprüfung: Überschreitet `buf.len()` das Limit `config.max_message_size`, wird mit `Err(FlatBufferError::BufferTooLarge)` abgelehnt.
/// 3. Verifier konfigurieren: Erstellen der `VerifierOptions` mit Limits für Tiefe, Tabellenanzahl und scheinbare Größe.
/// 4. Strukturprüfung via `flatbuffers::root_with_opts::<T>`: Prüft Offsets, VTables und Feldausrichtung im Byte-Array.
/// 5. Bei erfolgreicher Verifikation Rückgabe des typsicheren Wurzel-Objekts, sonst Umwandlung in `FlatBufferError::VerificationFailed`.
///
/// # Fehlerfälle
/// - Liefert `FlatBufferError::EmptyBuffer` bei leerem Slice.
/// - Liefert `FlatBufferError::BufferTooLarge`, wenn die Maximallänge überschritten ist.
/// - Liefert `FlatBufferError::VerificationFailed`, wenn Offsets außerhalb des Puffers liegen oder Daten korrupt sind.
pub fn verify_flatbuffer<'a, T>(
    buf: &'a [u8],
    config: &FlatBufferConfig,
) -> Result<T::Inner, FlatBufferError>
where
    T: flatbuffers::Follow<'a> + flatbuffers::Verifiable + 'a,
{
    if buf.is_empty() {
        return Err(FlatBufferError::EmptyBuffer);
    }
    if buf.len() > config.max_message_size {
        return Err(FlatBufferError::BufferTooLarge {
            size: buf.len(),
            max_size: config.max_message_size,
        });
    }

    let opts = VerifierOptions {
        max_depth: config.max_depth,
        max_tables: config.max_tables,
        max_apparent_size: config.max_message_size,
        ..Default::default()
    };

    flatbuffers::root_with_opts::<T>(&opts, buf)
        .map_err(|e| FlatBufferError::VerificationFailed(format!("{e:?}")))
}

// ANCHOR[contextra-wire:VER-005]
/// Verifies and parses a `SearchResponse` root table from bytes.
///
/// # Ablauf von `verify_and_parse_search_response`:
/// 1. Übergibt Puffer und Konfiguration an `verify_flatbuffer::<SearchResponse<'a>>`.
/// 2. Validiert Header, Result-Vektor und Dokumente.
/// 3. Gibt bei Erfolg die verifizierte `SearchResponse` zurück.
pub fn verify_and_parse_search_response<'a>(
    buf: &'a [u8],
    config: &FlatBufferConfig,
) -> Result<SearchResponse<'a>, FlatBufferError> {
    verify_flatbuffer::<SearchResponse<'a>>(buf, config)
}

// ANCHOR[contextra-wire:VER-006]
/// Verifies and parses an `Embedding` root table from bytes.
///
/// # Ablauf von `verify_and_parse_embedding`:
/// 1. Validiert Puffergrenzen und Tabellenstruktur für `Embedding`.
/// 2. Stellt sicher, dass Vektor-Floats bzw. quantisierte Bytes im Pufferbereich liegen.
/// 3. Gibt das verifizierte `Embedding` zurück.
pub fn verify_and_parse_embedding<'a>(
    buf: &'a [u8],
    config: &FlatBufferConfig,
) -> Result<Embedding<'a>, FlatBufferError> {
    verify_flatbuffer::<Embedding<'a>>(buf, config)
}

// ANCHOR[contextra-wire:VER-007]
/// Verifies and parses a `ScoredDocument` root table from bytes.
///
/// # Ablauf von `verify_and_parse_scored_document`:
/// 1. Validiert Puffergrenzen und Tabellenstruktur für `ScoredDocument`.
/// 2. Verifiziert Dokument-ID, Score, Metadaten-String und optionales Embedding.
/// 3. Gibt das verifizierte `ScoredDocument` zurück.
pub fn verify_and_parse_scored_document<'a>(
    buf: &'a [u8],
    config: &FlatBufferConfig,
) -> Result<ScoredDocument<'a>, FlatBufferError> {
    verify_flatbuffer::<ScoredDocument<'a>>(buf, config)
}

// ANCHOR[contextra-wire:VER-008]
/// Verifies and parses a `VectorIndexUpdate` root table from bytes.
///
/// # Ablauf von `verify_and_parse_vector_index_update`:
/// 1. Validiert Puffergrenzen und Tabellenstruktur für `VectorIndexUpdate`.
/// 2. Verifiziert Dokument-ID, Operationstyp und Vektordaten.
/// 3. Gibt das verifizierte `VectorIndexUpdate` zurück.
pub fn verify_and_parse_vector_index_update<'a>(
    buf: &'a [u8],
    config: &FlatBufferConfig,
) -> Result<VectorIndexUpdate<'a>, FlatBufferError> {
    verify_flatbuffer::<VectorIndexUpdate<'a>>(buf, config)
}

// ANCHOR[contextra-wire:VER-009]
/// Verifies and parses a `RoleId` root table from bytes.
///
/// # Ablauf von `verify_and_parse_role_id`:
/// 1. Validiert Puffergrenzen und Tabellenstruktur für `RoleId`.
/// 2. Verifiziert die u32-Rollenidentifikation.
/// 3. Gibt das verifizierte `RoleId` zurück.
pub fn verify_and_parse_role_id<'a>(
    buf: &'a [u8],
    config: &FlatBufferConfig,
) -> Result<RoleId<'a>, FlatBufferError> {
    verify_flatbuffer::<RoleId<'a>>(buf, config)
}

// ANCHOR[contextra-wire:VER-010]
/// Verifies and parses a `HyperEdgeId` root table from bytes.
///
/// # Ablauf von `verify_and_parse_hyper_edge_id`:
/// 1. Validiert Puffergrenzen und Tabellenstruktur für `HyperEdgeId`.
/// 2. Verifiziert die u64-Hyperkanten-ID.
/// 3. Gibt das verifizierte `HyperEdgeId` zurück.
pub fn verify_and_parse_hyper_edge_id<'a>(
    buf: &'a [u8],
    config: &FlatBufferConfig,
) -> Result<HyperEdgeId<'a>, FlatBufferError> {
    verify_flatbuffer::<HyperEdgeId<'a>>(buf, config)
}

// ANCHOR[contextra-wire:VER-011]
/// Verifies and parses a `RoleBinding` root table from bytes.
///
/// # Ablauf von `verify_and_parse_role_binding`:
/// 1. Validiert Puffergrenzen und Tabellenstruktur für `RoleBinding`.
/// 2. Verifiziert Rollen-ID und zugeordnete Knoten-IDs.
/// 3. Gibt das verifizierte `RoleBinding` zurück.
pub fn verify_and_parse_role_binding<'a>(
    buf: &'a [u8],
    config: &FlatBufferConfig,
) -> Result<RoleBinding<'a>, FlatBufferError> {
    verify_flatbuffer::<RoleBinding<'a>>(buf, config)
}

// ANCHOR[contextra-wire:VER-012]
/// Verifies and parses a `HyperEdge` root table from bytes.
///
/// # Ablauf von `verify_and_parse_hyper_edge`:
/// 1. Validiert Puffergrenzen und Tabellenstruktur für `HyperEdge`.
/// 2. Verifiziert Hyperkanten-ID, Attribute und Rollenbindungen.
/// 3. Gibt die verifizierte `HyperEdge` zurück.
pub fn verify_and_parse_hyper_edge<'a>(
    buf: &'a [u8],
    config: &FlatBufferConfig,
) -> Result<HyperEdge<'a>, FlatBufferError> {
    verify_flatbuffer::<HyperEdge<'a>>(buf, config)
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
mod tests {
    use super::*;

    // Prüft, dass die Verifikation von SearchResponse bei leerem Puffer mit EmptyBuffer fehlschlägt.
    #[test]
    fn test_verify_and_parse_search_response_empty_buffer() {
        let config = FlatBufferConfig::default();
        let err = verify_and_parse_search_response(&[], &config).unwrap_err();
        assert_eq!(err, FlatBufferError::EmptyBuffer);
    }

    // Prüft, dass SearchResponse bei Überschreitung von max_message_size mit BufferTooLarge abgelehnt wird.
    #[test]
    fn test_verify_and_parse_search_response_oversized_buffer() {
        let config = FlatBufferConfig {
            max_message_size: 10,
            ..Default::default()
        };
        let buf = vec![0u8; 20];
        let err = verify_and_parse_search_response(&buf, &config).unwrap_err();
        assert_eq!(
            err,
            FlatBufferError::BufferTooLarge {
                size: 20,
                max_size: 10
            }
        );
    }

    // Prüft, dass die Verifikation von Embedding bei leerem Puffer mit EmptyBuffer fehlschlägt.
    #[test]
    fn test_verify_and_parse_embedding_empty_buffer() {
        let config = FlatBufferConfig::default();
        let err = verify_and_parse_embedding(&[], &config).unwrap_err();
        assert_eq!(err, FlatBufferError::EmptyBuffer);
    }

    // Prüft, dass die Verifikation von ScoredDocument bei leerem Puffer mit EmptyBuffer fehlschlägt.
    #[test]
    fn test_verify_and_parse_scored_document_empty_buffer() {
        let config = FlatBufferConfig::default();
        let err = verify_and_parse_scored_document(&[], &config).unwrap_err();
        assert_eq!(err, FlatBufferError::EmptyBuffer);
    }

    // Prüft, dass die Verifikation von VectorIndexUpdate bei leerem Puffer mit EmptyBuffer fehlschlägt.
    #[test]
    fn test_verify_and_parse_vector_index_update_empty_buffer() {
        let config = FlatBufferConfig::default();
        let err = verify_and_parse_vector_index_update(&[], &config).unwrap_err();
        assert_eq!(err, FlatBufferError::EmptyBuffer);
    }

    // Prüft, dass die Verifikation von RoleId bei leerem Puffer mit EmptyBuffer fehlschlägt.
    #[test]
    fn test_verify_and_parse_role_id_empty_buffer() {
        let config = FlatBufferConfig::default();
        let err = verify_and_parse_role_id(&[], &config).unwrap_err();
        assert_eq!(err, FlatBufferError::EmptyBuffer);
    }

    // Prüft, dass die Verifikation von HyperEdgeId bei leerem Puffer mit EmptyBuffer fehlschlägt.
    #[test]
    fn test_verify_and_parse_hyper_edge_id_empty_buffer() {
        let config = FlatBufferConfig::default();
        let err = verify_and_parse_hyper_edge_id(&[], &config).unwrap_err();
        assert_eq!(err, FlatBufferError::EmptyBuffer);
    }

    // Prüft, dass die Verifikation von RoleBinding bei leerem Puffer mit EmptyBuffer fehlschlägt.
    #[test]
    fn test_verify_and_parse_role_binding_empty_buffer() {
        let config = FlatBufferConfig::default();
        let err = verify_and_parse_role_binding(&[], &config).unwrap_err();
        assert_eq!(err, FlatBufferError::EmptyBuffer);
    }

    // Prüft, dass die Verifikation von HyperEdge bei leerem Puffer mit EmptyBuffer fehlschlägt.
    #[test]
    fn test_verify_and_parse_hyper_edge_empty_buffer() {
        let config = FlatBufferConfig::default();
        let err = verify_and_parse_hyper_edge(&[], &config).unwrap_err();
        assert_eq!(err, FlatBufferError::EmptyBuffer);
    }
}
