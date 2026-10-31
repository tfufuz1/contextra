//! Modul
//!
//! # Zweck & Architektur
//! Komponente des  Subsystems (Ring 4 Adapter-Schicht).
//! Stellt Parser-, Extraktions- oder Hilfsfunktionen fuer die strukturierte Dokumentverarbeitung bereit.
//!
//! # Invarianten
//! - Zero-Panic: Alle Operationen sind defensiv aufgebaut.
//! - Fail-Closed: Fehlerhafte oder unvollstaendige Daten fuehren zu typisierten Fehlern.
//! - Kein externer I/O ueber Netzwerk oder unkontrollierte Dateisystempfade.

#![forbid(unsafe_code)]
// FILE-CONTEXT
// ZWECK: Root-Modul von contextra-ingest (Adapter-Crate für Format-Extraktion und Chunking).
// INVARIANTEN: Zero-Panic, deterministisch, kein I/O nach außen, #![forbid(unsafe_code)].
// STAND: TS:2026-10-31T00:00:00Z

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

#[doc(hidden)]
pub mod fuzz_support;
