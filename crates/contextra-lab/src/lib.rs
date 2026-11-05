#![forbid(unsafe_code)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
#![warn(missing_docs)]

//! Unabhängige Prüfinstanz für die Löschmessung, ohne Produktions-Decoder.
//!
//! # Architektur-Rolle: Ring dev (Unabhängige Prüfinstanz)
//!
//! Gemäß SPEC §8.16, §12.3, Anhang F sowie AR-4:
//! - Unabhängige Neuimplementierung aller Byte-Formate (Anhang F.1 bis F.10).
//! - Unabhängige Kryptografie-Aufrufe über externe, geprüfte Crates (`crypto_ref`).
//! - Null Abhängigkeiten zu Workspace-Crates (kein types, ports, store, engine, erasure).
//! - Typisierte Messlogik für MK1 (Struktur), MK2 (Rohscan) und MK3 (Schlüsselprobe).

pub mod crypto_ref;
pub mod error;
pub mod fixture;
pub mod format;
pub mod scanner;
