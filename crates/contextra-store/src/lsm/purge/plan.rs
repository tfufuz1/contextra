// FILE-CONTEXT
// STAND: 2026-10-11
// ZWECK: Purge-Phasen P0..P6 Enum.

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PurgePhase {
    P0Init,
    P1Intent,
    P2FenceAcquired,
    P3Flush,
    P4PurgeCompaction,
    P5Dispose,
    P6WalCallback,
    P7Complete,
}
