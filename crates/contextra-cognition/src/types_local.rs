//! Local abstractions for types to decouple `contextra-cognition`.
//
// FILE-CONTEXT
// ZWECK: Lokale Re-Exports und Typ-Aliase für contextra-types Domänenstrukturen.
// INVARIANTEN: Bitgleiche Repräsentationen; Mandantensicherheit (TenantId).
// STAND: 2026-11-11

// SECTION: Types Re-Exports
// ANCHOR[contextra-cognition:LOC-TYP-001]
pub use contextra_types as types;
pub use types::{
    ContextChunk, ContextWindow, ContextraError, DocId, EntityId, Result, TenantId, TokenBudget,
    TxId,
};
// !SECTION: Types Re-Exports
