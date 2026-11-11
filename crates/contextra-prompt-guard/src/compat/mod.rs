//! Kompatibilitätsschicht und Konvertierungen für Workspace-Integration.
//!
//! Gemäß Invariante I3 und Architektur-Ebene E0 finden Konvertierungen zu externen
//! Typen isoliert in diesem Modul statt. Da `contextra-prompt-guard` als E0-Baustein
//! über `allowed_internal = []` verfügt, werden keine direkten Workspace-Abhängigkeiten
//! im Kern importiert.

pub mod error;
