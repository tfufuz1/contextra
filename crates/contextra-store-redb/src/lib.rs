//! redb `StorageEngine` backend adapter for Contextra.
//!
//! Provides a persistent, transactional KV storage engine implementation on top of [`redb`].
//!
//! # Overview
//! The `RedbStorageEngine` adapter maps Contextra's [`StorageEngine`] trait to `redb`,
//! providing ACID transactions, MVCC snapshot reads at specific sequence numbers, and
//! rollback support.
//!
//! # Honest Deletion Semantics
//! In redb, deleting an entry (`table.remove()`) releases the containing B-tree pages to redb's
//! internal free-list. The raw bytes of deleted keys and values remain in the underlying `.redb` file
//! until overwritten by future page allocations.
//! Physical disk zeroing/purging is **not** supported by redb.
//!
//! Therefore, forget receipts on this backend can reach at most stage **S1** (Logical free-listing)
//! unless crypto-shredding is used. The adapter exposes this capability via
//! [`RedbStorageEngine::deletion_capabilities`].
//!
//! # Table of Semantic Deviations
//!
//! | Feature / Semantic | redb Behaviour | Consequence / Impact |
//! | :--- | :--- | :--- |
//! | **Physical Purge / Erasure** | `table.remove()` releases B-tree pages to internal free-lists; deleted bytes remain on disk in free-list pages until overwritten by future writes. | Physical disk scrubbing is unavailable. Forget receipts achieve at most stage **S1** (Logical free-listing) unless crypto-shredding is used. |
//! | **Memtable Flush** | `flush()` is a no-op sync call. Writes are committed directly to redb transaction log/B-tree file. | No background memtable flush or L0 compaction needed; every commit is immediately durable on disk. |
//! | **Checkpoint Pinning** | Historical MVCC versions are retained per key in table. `pin_checkpoint()` / `unpin_checkpoint()` are no-ops. | Historical versions remain stored in redb until explicitly overwritten or truncated via `rollback_to_tx`. |
//! | **SSI Read-Set Tracking** | Staged uncommitted writes apply sequentially at `commit()`. Read sets are not tracked (`supports_ssi_tracking()` returns `false`). | Optimistic SSI read-conflict detection is not active; serializability is enforced via redb write lock sequencing. |

#![forbid(unsafe_code)]

pub mod capabilities;
pub mod engine;

pub use capabilities::{DeletionStage, RedbDeletionCapabilities};
pub use engine::RedbStorageEngine;
