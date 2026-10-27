// FILE-CONTEXT
// STAND: 2026-10-27T00:00:00Z
// ZWECK: Main entrypoint for IPC framing and zero-copy wire adapters
// RELEVANTE SPEZIFIKATION: P7 (Zero-Panic), P2 (Unsafe-Inseln), REQ-QA-010

//! Ring 0 Unsafe Island: Auto-generated FlatBuffers IPC bindings and zero-copy adapters for Contextra.

#![deny(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

#[allow(
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
pub mod contextra_generated;

pub mod adapter;
pub mod jsonrpc;

pub use contextra_generated::Contextra::IPC::*;
