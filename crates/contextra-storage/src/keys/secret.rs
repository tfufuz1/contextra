//! # Secret32: Geschützte 32-Byte-Geheimnisse
//!
//! SPEC §4, §15.2, SD-56:
//! Undurchsichtiges Handle für 32-Byte-Geheimnisse (KEK, DEK, Wurzelschlüssel).
//! Implementiert bewusst weder `Debug`, `Clone` noch `Serialize`.
//! Beim `Drop` wird der Speicher best-effort mit Nullen überschrieben.

pub use contextra_storage_ports::{Secret32, SecretCell};
