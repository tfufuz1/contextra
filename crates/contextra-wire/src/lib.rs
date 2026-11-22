// FILE-CONTEXT
// STAND: 2026-10-27T00:00:00Z
// ZWECK: Main entrypoint for IPC framing and zero-copy wire adapters
// RELEVANTE SPEZIFIKATION: P7 (Zero-Panic), P2 (Unsafe-Inseln), REQ-QA-010

//! Ring 0 Unsafe Island: Auto-generated FlatBuffers IPC bindings and zero-copy adapters for Contextra.

#![deny(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]

pub mod generated;

pub mod adapter;
pub mod jsonrpc;
pub mod verification;

pub use adapter::*;
pub use generated::contextra_generated;
pub use generated::contextra_generated::contextra::ipc::*;
pub use jsonrpc::*;
pub use verification::*;
