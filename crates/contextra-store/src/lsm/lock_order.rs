//! # Sperrenregister & Rangbasierte Lock-Hierarchie (Ring 1)
//!
//! Formale Definition der globalen Lock-Akquisitionsreihenfolge im LSM-Speicher gemäß
//! SPEC §3.5, SPEC §5.4, MV-08 und Invariante `INV-LOCK-ORDER-1`:
//!
//! ```text
//! Dokument-Lock (Intent)
//!       ↓
//! Commit-Barriere / commit_mutex
//!       ↓
//! Flush-Barriere / flush_mutex
//!       ↓
//! Zustands-RwLock (state)
//!       ↓
//! Journal / WAL (truncate_lock)
//!       ↓
//! Speicherebene (sstables-RwLock)
//!       ↓
//! MemTable-Shards (parking_lot::RwLock)
//! ```
//!
//! Jede Verletzung dieser Reihenfolge führt zu deterministischen Verklemmungen (Deadlocks).
//!
//! # Sperren-Register
//!
//! | Rang | Konstante | Primitiv | Feld / Instanz | Haltedauer-Schranke | Begründung |
//! |---|---|---|---|---|---|
//! | 100 | [`RANK_DOCUMENT_INTENT`] | `std::sync::Mutex` | `LsmStorage::intent_locks` | < 10 µs (RAM) | Exklusive Reservierung von Schlüsseln vor Commit |
//! | 200 | [`RANK_COMMIT_MUTEX`] | `tokio::sync::Mutex` | `LsmStorage::commit_mutex` | < 50 ms (Group Commit Batching) | Serialisiert Validierung und Transaktions-Commit |
//! | 300 | [`RANK_FLUSH_MUTEX`] | `tokio::sync::Mutex` | `LsmStorage::flush_mutex` | < 500 ms (I/O Flush) | Verhindert parallele Memtable-Flushes |
//! | 400 | [`RANK_STATE_RWLOCK`] | `tokio::sync::RwLock` | `LsmStorage::state` | < 50 µs (Write), < 5 µs (Read) | Schützt aktive/unveränderliche Memtables |
//! | 500 | [`RANK_WAL_TRUNCATE`] | `tokio::sync::Mutex` | `Wal::truncate_lock` | < 10 ms (I/O Truncate) | Schützt WAL-Größe und Segment-Abschneiden |
//! | 600 | [`RANK_SSTABLES_RWLOCK`] | `tokio::sync::RwLock` | `LsmStorage::sstables` | < 100 µs (Write), < 10 µs (Read) | Schützt aktive SSTable-Reader-Liste |
//! | 700 | [`RANK_MEMTABLE_SHARDS`] | `parking_lot::RwLock` | `MemTable::shards` | < 5 µs (RAM) | Schützt BTreeMap-Shards im RAM (nie über `.await`) |
//! | 800 | [`RANK_MANIFEST_MUTEX`] | `tokio::sync::Mutex` | `Manifest::file` | < 20 ms (I/O Append) | Schützt Manifest-Datei-Appends |
//! | 900 | [`RANK_OBSERVER_REGISTRY`] | `parking_lot::RwLock` | `ObserverRegistry::observers` | < 5 µs (RAM) | Registrierung von WAL-Beobachtern |
//! | 950 | [`RANK_BLOCK_CACHE`] | `parking_lot::RwLock` | `SieveCacheBackend::state` | < 2 µs (RAM) | Sharded LRU/SIEVE Cache-Hits und Evictions |
//!
//! # Regeln für Sperren
//! 1. **Kein synchroner Guard über `.await` (`clippy::await_holding_lock`):**
//!    Guards von `parking_lot` und `std::sync` dürfen niemals über ein `.await` gehalten werden.
//! 2. **Tokio-Guards mit Zeitschranke:**
//!    Asynchrone `tokio::sync`-Guards dürfen nur gehalten werden, solange strikte Deadlock-Freiheit
//!    durch Einhaltung obiger Rangordnung garantiert ist.

/// Rang 100: Transaktionale Absichtssperren auf Dokument-/Schlüsselebene (`intent_locks`).
pub const RANK_DOCUMENT_INTENT: u32 = 100;

/// Rang 200: Globaler Commit-Mutex des Stores (`commit_mutex`), entspricht Commit-Barriere.
pub const RANK_COMMIT_MUTEX: u32 = 200;

/// Rang 300: Flush-Mutex des Stores (`flush_mutex`), serialisiert Flush-Vorgänge.
pub const RANK_FLUSH_MUTEX: u32 = 300;

/// Rang 400: LSM-Zustand (`state`), schützt MemTable-Zeiger und unveränderliche MemTables.
pub const RANK_STATE_RWLOCK: u32 = 400;

/// Rang 500: WAL-Truncate-Sperre (`wal.truncate_lock`), serialisiert Append/Truncate im WAL.
pub const RANK_WAL_TRUNCATE: u32 = 500;

/// Rang 600: SSTable-Katalog (`sstables`), schützt die Liste aktiver L0..Ln SSTables.
pub const RANK_SSTABLES_RWLOCK: u32 = 600;

/// Rang 700: MemTable-Shards (`MemTable::shards`), atomare RAM-Modifikationen.
pub const RANK_MEMTABLE_SHARDS: u32 = 700;

/// Rang 800: Manifest-Append-Sperre (`Manifest::file`).
pub const RANK_MANIFEST_MUTEX: u32 = 800;

/// Rang 900: Observer-Registry-Sperre (`ObserverRegistry::observers`).
pub const RANK_OBSERVER_REGISTRY: u32 = 900;

/// Rang 950: SSTable-Block-Cache-Sperre (`SieveCacheBackend::state`).
pub const RANK_BLOCK_CACHE: u32 = 950;
