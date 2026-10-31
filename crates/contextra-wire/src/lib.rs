// FILE-CONTEXT
// STAND: 2026-10-27T00:00:00Z
// ZWECK: Main entrypoint for IPC framing and zero-copy wire adapters
// RELEVANTE SPEZIFIKATION: P7 (Zero-Panic), P2 (Unsafe-Inseln), REQ-QA-010

//! # Crate `contextra-wire`
//!
//! **Ring & Klasse:** Ring 0 (Unsafe-Insel 3 von 3), Klasse Kern `[K]`.
//!
//! ## Zweck
//! `contextra-wire` stellt performante Zero-Copy-Serialisierungs- und Deserialisierungs-Primitiven
//! für die Interprozesskommunikation (IPC = Inter-Process Communication) bereit. Es kapselt:
//! 1. Generierten FlatBuffers-Code (`contextra_generated`), der direkt auf Byte-Puffern operiert.
//! 2. Gehärtete Verifikationsfunktionen (`verification`), die Puffer vor dem Zugriff strikt validieren.
//! 3. Zero-Copy-Adapter (`adapter`), wie den referenzgezählten `WireBuffer`.
//! 4. Einen robusten, angriffssicheren JSON-RPC-2.0-Parser (`jsonrpc`) ohne Panics.
//!
//! ## Einbindung und Abhängigkeiten
//! - **Abhängigkeiten (normal):** `bytes`, `flatbuffers`, `serde`, `serde_json`.
//! - **Wird eingebunden von:** `contextra-core`, `contextra-mcp`, `contextra-py`, `contextra-router`.
//! - **Ring-0-Restriktionen:** Gemäß Regel 3 aus `CLAUDE.md` dürfen Zeit, Zufall und IDs nicht direkt
//!   erzeugt werden; I/O erfolgt über Ports oder übergebene Puffer.
//!
//! ## Sicherheitsinvariante & Zero-Panic-Garantie
//! Die Crate erzwingt `#![deny(clippy::unwrap_used)]`, `#![deny(clippy::expect_used)]` und
//! `#![deny(clippy::panic)]`. Fehler werden konsequent über `Result` gemeldet (Fail-Closed).

// ANCHOR[contextra-wire:LIB-001]
// NOTE: Compiler-Lints zur Absicherung von Ring 0. Unsafe-Code ist in handgeschriebenem Code
// strikt untersagt und wird ausschließlich im isolierten Submodul `generated` für FlatBuffers erlaubt.
#![deny(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]

// ANCHOR[contextra-wire:LIB-002]
// NOTE: Kapselt den extern generierten FlatBuffers-Code ab, um Warnungen zu isolieren.
pub mod generated;

// ANCHOR[contextra-wire:LIB-003]
// NOTE: Öffentliche Submodule für Zero-Copy-Buffer, JSON-RPC und Verifikation.
pub mod adapter;
pub mod jsonrpc;
pub mod verification;

// Re-Exports für bequemen Zugriff auf die Kern-Typen von außen.
pub use adapter::*;
pub use generated::contextra_generated;
pub use generated::contextra_generated::contextra::ipc::*;
pub use jsonrpc::*;
pub use verification::*;
