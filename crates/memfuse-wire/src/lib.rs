//! Ring 0 Unsafe Island: Auto-generated FlatBuffers IPC bindings and zero-copy adapters for MemFuse.

#![allow(unsafe_code)]
#![allow(unsafe_op_in_unsafe_fn)]
#![allow(clippy::all)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::panic)]
#![allow(clippy::undocumented_unsafe_blocks)]

#[allow(clippy::all)]
#[allow(missing_docs)]
#[allow(unused_imports)]
#[allow(unsafe_code)]
#[allow(mismatched_lifetime_syntaxes)]
pub mod memfuse_generated;

pub mod adapter;
pub mod jsonrpc;

pub use jsonrpc::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};
pub use memfuse_generated::mem_fuse::ipc::*;
