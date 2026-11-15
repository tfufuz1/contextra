//! Modul
//!
//! # Zweck & Architektur
//! Stellt Parser-, Extraktions- oder Hilfsfunktionen fuer die strukturierte Dokumentverarbeitung bereit.
//!
//! # Invarianten
//! - Zero-Panic: Alle Operationen sind defensiv aufgebaut.
//! - Fail-Closed: Fehlerhafte oder unvollstaendige Daten fuehren zu typisierten Fehlern.
//! - Kein externer I/O ueber Netzwerk oder unkontrollierte Dateisystempfade.

// Platzhalter-Modul für sichere Parsing-Helfer (Archiv, XML, OPC) (WP M4 / Platzhalter für P00B).
// Invarianten: Zero-Panic, bedingte Kompilation.

#[cfg(feature = "zip-container")]
pub mod archive;

#[cfg(feature = "xml")]
pub mod xml;

#[cfg(all(feature = "zip-container", feature = "xml"))]
pub mod opc;
