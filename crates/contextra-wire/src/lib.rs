// FILE-CONTEXT
// STAND: 2026-09-27T22:49:25Z (SESSION: 72f4c80d)
// ZWECK: Ring 0 Unsafe Island: Auto-generated FlatBuffers IPC bindings and zero-copy adapters.
// INVARIANTEN: Unsafe code isolated to generated FlatBuffers bindings; safe abstractions in adapter and jsonrpc.
// SIEHE AUCH: docs/audits/contextra-wire_AUDIT_2026-09-27.md

//! Ring 0 Unsafe Island: Auto-generated FlatBuffers IPC bindings and zero-copy adapters for Contextra.

#![allow(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

pub mod contextra_generated;

pub mod adapter;
pub mod jsonrpc;

pub use contextra_generated::contextra::ipc::*;
pub use jsonrpc::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};
