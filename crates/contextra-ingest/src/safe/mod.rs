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

// FILE-CONTEXT
// ZWECK: Platzhalter-Modul für sichere Parsing-Helfer (Archiv, XML, OPC) (WP M4 / Platzhalter für P00B).
// INVARIANTEN: Zero-Panic, bedingte Kompilation.
// STAND: TS:2026-10-31T00:00:00Z

#[cfg(feature = "zip-container")]
pub mod archive;

#[cfg(feature = "xml")]
pub mod xml;

#[cfg(all(feature = "zip-container", feature = "xml"))]
pub mod opc;
