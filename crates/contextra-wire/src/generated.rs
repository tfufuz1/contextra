// FILE-CONTEXT
// STAND: 2026-10-27T00:00:00Z
// ZWECK: Isolation module for auto-generated FlatBuffers IPC code
// RELEVANTE SPEZIFIKATION: @generated, siehe check-flatbuffers-drift

//! # Isolationsmodul für automatisch generierten FlatBuffers-Code
//!
//! **Ring:** Ring 0 (Unsafe-Insel 3 von 3)
//!
//! ## Zweck
//! Dieses Modul bindet die Datei `contextra_generated.rs` ein. Da der FlatBuffers-Compiler (`flatc`)
//! Code erzeugt, der nicht alle strengen Clippy- und Sicherheitsrichtlinien des Projekts erfüllt
//! (z. B. fehlende Safety-Kommentare oder Warnungen vor nicht-minimalen Lebenszeiten), dient diese
//! Datei als Isolationsschicht, um Lints lokal zu deaktivieren, ohne die globale Crate-Sicherheit zu schwächen.

// ANCHOR[contextra-wire:GEN-001]
// @generated, siehe check-flatbuffers-drift
// NOTE: Gezieltes Unterdrücken von Compiler- und Clippy-Warnungen für maschinell erzeugten Code.
#![allow(
    unsafe_code,
    unused_imports,
    clippy::all,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::needless_lifetimes,
    clippy::extra_unused_lifetimes,
    clippy::doc_markdown,
    clippy::derivable_impls,
    clippy::missing_safety_doc,
    clippy::wrong_self_convention
)]

// ANCHOR[contextra-wire:GEN-002]
// NOTE: Pfadverknüpfung zur generierten Rust-Datei.
#[path = "contextra_generated.rs"]
pub mod contextra_generated;
