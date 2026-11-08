//! # Key-Value Speicherabstraktion (Ring 1)
//!
//! Enthaelt Untermodule fuer Segmentierung und Bereinigungsmodi (Tombstone vs. CryptoShredding).

// FILE-CONTEXT
// ZWECK: Key-Value Segment-Verwaltung und Bereinigungsmodi (TombstoneOnly, CryptoShred).
// INVARIANTEN:
// - INV-KV-DELETE-1: Bereinigungsnachweis auf Key-Value-Seite ist ausschließlich für `CryptoShred`-Segmente möglich, niemals für `TombstoneOnly`.

pub mod delete_mode;
pub mod segment;

pub use delete_mode::KvDeleteMode;
pub use segment::{KvSegmentConfig, KvSegmentManager, KvSegmentPayload};
