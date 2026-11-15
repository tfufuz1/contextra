//! Modul
//!
//! # Zweck & Architektur
//! Stellt Parser-, Extraktions- oder Hilfsfunktionen fuer die strukturierte Dokumentverarbeitung bereit.
//!
//! # Invarianten
//! - Zero-Panic: Alle Operationen sind defensiv aufgebaut.
//! - Fail-Closed: Fehlerhafte oder unvollstaendige Daten fuehren zu typisierten Fehlern.
//! - Kein externer I/O ueber Netzwerk oder unkontrollierte Dateisystempfade.

// Export-Modul für alle Format-Extraktor Stubs hinter #[cfg(feature)] Schaltern (WP M4).
// Invarianten: Zero-Panic, bedingte Kompilation je Format-Feature.

#[cfg(feature = "fmt-text")]
pub mod text;

#[cfg(feature = "fmt-web")]
pub mod web;

#[cfg(feature = "fmt-docx")]
pub mod docx;

#[cfg(feature = "fmt-pptx")]
pub mod pptx;

#[cfg(feature = "fmt-sheets")]
pub mod sheets;

#[cfg(feature = "fmt-odf")]
pub mod odf;

#[cfg(feature = "fmt-pdf")]
pub mod pdf;

#[cfg(feature = "fmt-ole")]
pub mod ole;

#[cfg(feature = "fmt-email")]
pub mod email;

#[cfg(feature = "fmt-structured")]
pub mod structured;
