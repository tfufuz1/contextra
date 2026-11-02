//! Systemweite Kapazitäts- und Größengrenzen (SPEC 14.4, D-87, TY-14).
//!
//! Überschreitungen führen immer zu `Error::Limit`, niemals zu stillem Abschneiden.

/// Maximale Anzahl an Dokumenten je Store-Instanz (50.000.000).
pub const MAX_DOCS_PER_STORE: u64 = 50_000_000;

/// Standard-Obergrenze für Dokumentengröße in Bytes (4 MiB).
pub const MAX_DOC_BYTES_DEFAULT: usize = 4 * 1024 * 1024;

/// Harte maximale Dokumentengröße in Bytes (64 MiB).
pub const MAX_DOC_BYTES_HARD: usize = 64 * 1024 * 1024;

/// Maximale Anzahl aktiver MVCC-Snapshots je Mandant (1024).
pub const MAX_SNAPSHOTS_PER_TENANT: usize = 1024;

/// Maximale Dimension für Einbettungsvektoren (4096).
pub const MAX_DIMENSION: usize = 4096;

/// Maximale Anzahl von Termen in einer Suchanfrage (256).
pub const MAX_QUERY_TERMS: usize = 256;

/// Maximaler Top-k-Wert für Retrieval (1000).
pub const MAX_K: usize = 1000;
