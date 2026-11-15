//! Modul
//!
//! # Zweck & Architektur
//! Stellt Parser-, Extraktions- oder Hilfsfunktionen fuer die strukturierte Dokumentverarbeitung bereit.
//!
//! # Invarianten
//! - Zero-Panic: Alle Operationen sind defensiv aufgebaut.
//! - Fail-Closed: Fehlerhafte oder unvollstaendige Daten fuehren zu typisierten Fehlern.
//! - Kein externer I/O ueber Netzwerk oder unkontrollierte Dateisystempfade.

#![forbid(unsafe_code)]
// Root-Modul von acta-ingest (Adapter-Crate für Format-Extraktion und Chunking).
// Invarianten: Zero-Panic, deterministisch, kein I/O nach außen, #![forbid(unsafe_code)].

#[cfg(feature = "workspace")]
pub mod compat;

pub mod canonical;
pub mod chunking;
pub mod detect;
pub mod error;
pub mod extractor;
pub mod format;
pub mod formats;
pub mod ir;
pub mod limits;
pub mod pipeline;
pub mod registry;
pub mod safe;

pub use chunking::types::{ContextChunk, ContextChunkLink, DocId};
pub use error::{IngestError, LimitKind};

#[cfg(feature = "fuzz")]
#[doc(hidden)]
pub mod fuzz_support;
