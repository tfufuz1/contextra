// FILE-CONTEXT
// STAND: 2026-09-27T22:49:25Z (SESSION: 72f4c80d)
// ZWECK: Ring 0 Unsafe Island: Auto-generated FlatBuffers IPC bindings and zero-copy adapters.
// INVARIANTEN: Unsafe code isolated to generated FlatBuffers bindings; safe abstractions in adapter and jsonrpc.
// SIEHE AUCH: docs/audits/contextra-wire_AUDIT_2026-09-27.md

//! Ring 0 Unsafe Island: Auto-generated FlatBuffers IPC bindings and zero-copy adapters for Contextra.

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
pub mod contextra_generated;

pub mod adapter;
pub mod jsonrpc;

pub use contextra_generated::contextra::ipc::*;
pub use jsonrpc::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};

impl<'a> SearchResponse<'a> {
    /// Finishes building the standard FlatBuffer with `SearchResponse` as the root table.
    #[inline]
    pub fn finish_buffer<'bldr>(
        fbb: &mut flatbuffers::FlatBufferBuilder<'bldr>,
        root: flatbuffers::WIPOffset<SearchResponse<'bldr>>,
    ) {
        finish_search_response_buffer(fbb, root);
    }

    /// Finishes building the size-prefixed FlatBuffer with `SearchResponse` as the root table.
    #[inline]
    pub fn finish_size_prefixed_buffer<'bldr>(
        fbb: &mut flatbuffers::FlatBufferBuilder<'bldr>,
        root: flatbuffers::WIPOffset<SearchResponse<'bldr>>,
    ) {
        finish_size_prefixed_search_response_buffer(fbb, root);
    }

    /// Verifies and parses a size-prefixed buffer with options into a `SearchResponse`.
    #[inline]
    pub fn size_prefixed_root_with_opts<'buf>(
        opts: &flatbuffers::VerifierOptions,
        buf: &'buf [u8],
    ) -> Result<SearchResponse<'buf>, flatbuffers::InvalidFlatbuffer> {
        size_prefixed_root_as_search_response_with_opts(opts, buf)
    }
}
