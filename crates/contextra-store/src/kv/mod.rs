// FILE-CONTEXT
// ZWECK: Key-Value Segment-Verwaltung und Löschmodi (TombstoneOnly, CryptoShred).
// INVARIANTEN:
// - INV-KV-DELETE-1: Löschbeweis auf Key-Value-Seite ist ausschließlich für `CryptoShred`-Segmente möglich, niemals für `TombstoneOnly`.

pub mod delete_mode;
pub mod segment;

pub use delete_mode::KvDeleteMode;
pub use segment::{KvSegmentConfig, KvSegmentManager, KvSegmentPayload};
