# MemFuse — Implementierungsspezifikation (codegenerierungsfähig)

> **Status:** Normativ · Einzige maßgebliche Quelle. Dieses Dokument ersetzt die vorherige
> `MEMFUSE_GESAMTSPEZIFIKATION.md` vollständig und verschärft sie auf **Implementierungsebene**: jedes Modul, jede
> Datei, jede öffentliche Signatur, jedes Fehler-Enum, jedes Persistenzformat und jedes Testkriterium ist so
> präzise gefasst, dass ein Sprachmodell daraus den vollständigen Quellcode ableiten kann, ohne eigene
> Architekturentscheidungen treffen zu müssen. Wo eine Entscheidung bewusst offen bleibt (z. B. Produktions-Default
> vs. Opt-in), ist dies als **Konfigurationsvorgabe**, nicht als Lücke spezifiziert.
> **Sprache:** Rust 2021, Workspace-Layout, `#![forbid(unsafe_code)]` als Default in jedem Crate ohne explizite
> Ausnahme (§0.4).
> **Lesart:** Jeder Abschnitt ist eigenständig implementierbar. Codeblöcke sind **normativ**, nicht illustrativ —
> Feldnamen, Typnamen und Funktionssignaturen sind exakt zu übernehmen, sofern nicht als „Beispiel" markiert.

---

## Inhaltsverzeichnis

0. [Meta: Workspace-Layout und Build-Konfiguration](#meta)
1. [Layer 0: `memfuse-core` und `memfuse-core-ipc-gen`](#layer0)
2. [Layer 1: `memfuse-store` (LSM, WAL, Block-Cache)](#store)
3. [Layer 1: `memfuse-crypto`](#crypto)
4. [Layer 1: `memfuse-text` (BM25/BM25F)](#text)
5. [Layer 1: `memfuse-index` (HNSW/DiskANN)](#index)
6. [Layer 1: `memfuse-graph` (CSR, PPR, Leiden, Hyperkanten)](#graph)
7. [Layer 1: `memfuse-checkpoint`](#checkpoint)
8. [Layer 1: `memfuse-calibration`](#calibration)
9. [Layer 2: `memfuse-db` (Collection-API, Fusion, Provenance)](#db)
10. [Layer 3: `memfuse-router` (Contextual Bandit)](#router)
11. [Layer 3: `memfuse-candle` (Inferenz, KV-Cache-Bridge)](#candle)
12. [Layer 3: `memfuse-ollama`, `memfuse-embed`, `memfuse-agent`, `memfuse-py`](#layer3-rest)
13. [Layer 4: `memfuse-mcp` (Server, Sandbox, Egress-Gateway)](#mcp)
14. [Layer 5: `memfuse-bench`](#bench)
15. [FlatBuffers-Schema (vollständig)](#schema)
16. [Fehlertaxonomie (crateübergreifend)](#errors)
17. [Test- und CI-Spezifikation](#tests)
18. [Vollständige Abnahmekriterien mit Testnamen](#abnahme)

---

<a id="meta"></a>
## 0. Meta: Workspace-Layout und Build-Konfiguration

### 0.1 Verzeichnisstruktur

```
memfuse/
├── Cargo.toml                      # [workspace], resolver = "2"
├── xtask/                          # CI-Tooling (Drift-Gates, Benchmarks)
│   └── src/
│       ├── main.rs
│       ├── check_flatbuffers_drift.rs
│       └── check_bandit_latency_budget.rs
├── schemas/
│   └── memfuse.fbs                 # §15
├── crates/
│   ├── memfuse-core-ipc-gen/
│   ├── memfuse-core/
│   ├── memfuse-store/
│   ├── memfuse-crypto/
│   ├── memfuse-text/
│   ├── memfuse-index/
│   ├── memfuse-graph/
│   ├── memfuse-checkpoint/
│   ├── memfuse-calibration/
│   ├── memfuse-db/
│   ├── memfuse-router/
│   ├── memfuse-candle/
│   ├── memfuse-ollama/
│   ├── memfuse-embed/
│   ├── memfuse-agent/
│   ├── memfuse-py/
│   ├── memfuse-mcp/
│   └── memfuse-bench/
├── .github/workflows/
│   └── merge-gate.yml              # §17.4
└── docs/decisions/                 # ADR-0NN-*.md
```

### 0.2 Root-`Cargo.toml`

```toml
[workspace]
resolver = "2"
members = ["crates/*", "xtask"]

[workspace.package]
edition = "2021"
rust-version = "1.79"
license = "Apache-2.0"

[workspace.dependencies]
serde = { version = "1", features = ["derive"] }
flatbuffers = "23"
crossbeam-epoch = "0.9"
arc-swap = "1"
ahash = "0.8"
scc = "2"
quick_cache = "0.5"
zerocopy = "0.7"
thiserror = "1"
tokio = { version = "1", features = ["rt-multi-thread", "sync", "time", "macros"] }
aes-gcm-siv = "0.11"
blake3 = "1"
wasmtime = "23"
```

### 0.3 Cargo-Feature-Katalog (crateübergreifend normativ)

| Feature | Definierender Crate | Default | Wirkung |
|---|---|---|---|
| `docid-128` | `memfuse-core` | aus | `DocId` wird `u128` statt `u64` (§1.3) |
| `block-cache-v2` | `memfuse-store` | aus | `SieveCacheBackend` statt `LruBlockCacheBackend` als aktives Backend (§2.4) |
| `egress-sherman-morrison` | `memfuse-router` | aus | `ShermanMorrisonBandit` statt `DiagonalApproximation` (§10.2) |
| `experimental-diskann` | `memfuse-index` | aus | `DiskAnnIndex` kompiliert und ist über `VectorIndexTier::DiskAnn` wählbar (§5.4) |
| `bandit-routing` | `memfuse-router` | an | Aktiviert den Bandit-Router überhaupt |
| `cloud-egress-guard` | `memfuse-mcp` | an | Aktiviert `egress_gateway`-Modul |
| `wasm-sandbox` | `memfuse-mcp` | an | Aktiviert `sandbox`-Modul |
| `kv-bridge` | `memfuse-candle` | an | Aktiviert `kv_cache_bridge`-Modul |
| `edge-reinforcement-learning` | `memfuse-graph` | aus | Aktiviert `SignalKind::EdgeReinforcement`-Pfad |
| `fault-injection` | `memfuse-store` | nur `dev-dependencies` | Deterministische I/O-Fehlerinjektion für Tests |
| `loom` | `memfuse-store`, `memfuse-graph` | nur `dev-dependencies` | Aktiviert `loom::sync::*` statt `std::sync::*` hinter `#[cfg(loom)]` |

### 0.4 Globale Compile-Time-Regeln

Jeder Crate erhält in `lib.rs` genau eine der beiden folgenden Kopfzeilen:

```rust
#![forbid(unsafe_code)]
```

oder, ausschließlich in `memfuse-index`, `memfuse-store` (nur der ACL-Teilbereich), `memfuse-db` (nur
`mlock`-Teilbereich, feature-gated), `memfuse-embed` (feature-gated), `memfuse-core-ipc-gen` (generierter Code)
und `memfuse-router` (nur SIMD-Teilbereich, §10.3):

```rust
#![deny(unsafe_code)] // Ausnahmen dokumentiert je Modul mit `// SAFETY:`-Kommentar
```

Jeder Crate erhält zusätzlich:

```rust
#![deny(clippy::unwrap_used, clippy::expect_used)]
```

mit einer versionierten Ausnahmeliste `.unwrap-baseline.json` im Crate-Root, die in CI gegen Neuvorkommen geprüft
wird (Ratchet: die Datei darf nur schrumpfen, nie wachsen — CI-Job `check-unwrap-baseline`).

---

<a id="layer0"></a>
## 1. Layer 0: `memfuse-core` und `memfuse-core-ipc-gen`

### 1.1 `memfuse-core-ipc-gen`

Enthält ausschließlich generierten Code aus `schemas/memfuse.fbs` (§15), erzeugt über `flatc --rust`. Kein
handgeschriebener Code außer `build.rs`:

```rust
// crates/memfuse-core-ipc-gen/build.rs
fn main() {
    println!("cargo:rerun-if-changed=../../schemas/memfuse.fbs");
    flatc_rust::run(flatc_rust::Args {
        inputs: &[std::path::Path::new("../../schemas/memfuse.fbs")],
        out_dir: std::path::Path::new("src/generated/"),
        ..Default::default()
    }).expect("flatc codegen failed");
}
```

### 1.2 `memfuse-core/src/types/domain.rs` — Identitätstypen

```rust
use serde::{Deserialize, Serialize};

/// Dokument-ID. Default 64-Bit; feature-gated 128-Bit-Variante (BLAKE3-Truncation, ADR-082).
#[cfg(not(feature = "docid-128"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DocId(pub u64);

#[cfg(feature = "docid-128")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(C, align(16))]
pub struct DocId(pub u128);

impl DocId {
    /// Erzeugt eine DocId deterministisch aus dem Collection-Key + Insert-Sequenznummer
    /// via BLAKE3-Truncation (nicht UUIDv7 — ADR-082 verbindlich).
    pub fn derive(collection_key: &[u8], seq: u64) -> Self {
        let mut hasher = blake3::Hasher::new();
        hasher.update(collection_key);
        hasher.update(&seq.to_le_bytes());
        let digest = hasher.finalize();
        #[cfg(not(feature = "docid-128"))]
        { Self(u64::from_le_bytes(digest.as_bytes()[0..8].try_into().unwrap())) }
        #[cfg(feature = "docid-128")]
        { Self(u128::from_le_bytes(digest.as_bytes()[0..16].try_into().unwrap())) }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct EntityId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TxId(pub u64);

/// Prädikat-/Kantentyp. Bewusst `#[non_exhaustive]` — Erweiterung ist additiv erlaubt,
/// im Unterschied zu `SignalKind` (§9.4), das strukturell geschlossen bleibt (P-Ergänzung §3).
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EdgeType {
    Default,
}
```

### 1.3 `memfuse-core/src/error.rs` — Basis-Fehlertyp

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("serialization failed: {0}")]
    Serialization(String),
    #[error("invalid DocId encoding")]
    InvalidDocId,
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
```

Jeder Layer-1-Crate definiert sein eigenes `Result<T, E>`-Enum als `thiserror`-Ableitung, das `CoreError` per
`#[from]` einbettet — niemals `Box<dyn Error>` an öffentlichen Grenzen.

### 1.4 `memfuse-core/src/traits.rs` — gemeinsame Traits

```rust
pub trait Persistable: serde::Serialize + serde::de::DeserializeOwned {
    const LSM_PREFIX: &'static str;
}
```

---

<a id="store"></a>
## 2. Layer 1: `memfuse-store` (LSM-Tree, WAL, Block-Cache)

### 2.1 Modulstruktur

```
crates/memfuse-store/src/
├── lib.rs
├── lsm.rs               # LSM-Tree, Compaction
├── wal.rs                # Write-Ahead-Log + HMAC-Kette
├── wal_ring_buffer.rs     # SPSC-Ring-Puffer (§2.3)
├── block_cache/
│   ├── mod.rs             # BlockCacheBackend-Trait
│   ├── lru.rs             # LruBlockCacheBackend (Default)
│   └── sieve.rs           # SieveCacheBackend (Opt-in, `block-cache-v2`)
├── kv_locks.rs            # KvKeyLocks (§2.2)
└── error.rs
```

### 2.2 Key-granulares Locking: `kv_locks.rs`

```rust
use ahash::AHashMap;
use std::sync::{Arc, RwLock, RwLockWriteGuard};

/// Sperrenhierarchie (verbindlich, systemweit einzuhalten):
///   collections (RwLock) → kv_locks (schlüssel-granular) → embedder (RwLock)
/// Jede Erweiterung, die mehr als einen Schlüssel gleichzeitig unter kv_locks hält, MUSS
/// `acquire_multi_sorted` verwenden (kanonische Sortierung, siehe memfuse-graph §6.5 H2).
pub struct KvKeyLocks {
    shards: Vec<RwLock<()>>,
    shard_mask: u64,
}

pub struct KeyGuard<'a> {
    _guard: RwLockWriteGuard<'a, ()>,
}

pub struct MultiKeyGuard<'a> {
    _guards: Vec<RwLockWriteGuard<'a, ()>>,
}

impl KvKeyLocks {
    pub fn new(shard_count_pow2: u32) -> Self {
        let n = 1u64 << shard_count_pow2;
        Self {
            shards: (0..n).map(|_| RwLock::new(())).collect(),
            shard_mask: n - 1,
        }
    }

    fn shard_for(&self, key_hash: u64) -> usize {
        (key_hash & self.shard_mask) as usize
    }

    pub fn acquire(&self, key_hash: u64) -> KeyGuard<'_> {
        let idx = self.shard_for(key_hash);
        KeyGuard { _guard: self.shards[idx].write().unwrap() }
    }

    /// H2-Pflichtmethode: Erwirbt N Shards STRIKT in aufsteigender Shard-Index-Reihenfolge.
    /// Aufrufer MUSS `keys` bereits kanonisch sortiert übergeben (siehe memfuse-graph::relate_n_ary).
    pub fn acquire_multi_sorted(&self, sorted_key_hashes: &[u64]) -> Result<MultiKeyGuard<'_>, LockError> {
        let mut shard_indices: Vec<usize> = sorted_key_hashes.iter()
            .map(|h| self.shard_for(*h)).collect();
        shard_indices.sort_unstable();
        shard_indices.dedup();
        let guards = shard_indices.iter()
            .map(|&idx| self.shards[idx].write().map_err(|_| LockError::Poisoned))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(MultiKeyGuard { _guards: guards })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum LockError {
    #[error("lock poisoned")]
    Poisoned,
    #[error("lock acquisition timed out")]
    Timeout,
}
```

**Loom-Testpflicht:** `crates/memfuse-store/tests/loom_multi_key_lock.rs` MUSS unter `#[cfg(loom)]` zwei
nebenläufige `acquire_multi_sorted`-Aufrufe mit den Schlüsselmengen `{A,B,C}` und `{C,B,A}` (identische Menge,
unterschiedliche Eingabereihenfolge) modellieren und deren Terminierung ohne Deadlock nachweisen
(§18, AK-3-Analogon für den generischen Fall; die Hyperkanten-spezifische Instanz siehe §6.5).

### 2.3 WAL-Ring-Puffer: `wal_ring_buffer.rs`

```rust
use std::sync::atomic::{AtomicUsize, Ordering};

/// SPSC-Ring-Puffer. `capacity` MUSS eine Zweierpotenz sein (Invariante wird in `new` erzwungen),
/// um Modulo durch `& (capacity - 1)` zu ersetzen.
pub struct WalRingBuffer {
    buf: Box<[std::mem::MaybeUninit<WalEntry>]>,
    capacity_mask: usize,
    write_idx: AtomicUsize,
    read_idx: AtomicUsize,
}

pub struct WalEntry {
    pub payload: Vec<u8>,
    pub hmac_prev: [u8; 32],
}

impl WalRingBuffer {
    pub fn new(capacity_pow2: usize) -> Result<Self, WalError> {
        if !capacity_pow2.is_power_of_two() {
            return Err(WalError::CapacityNotPowerOfTwo);
        }
        Ok(Self {
            buf: (0..capacity_pow2).map(|_| std::mem::MaybeUninit::uninit()).collect(),
            capacity_mask: capacity_pow2 - 1,
            write_idx: AtomicUsize::new(0),
            read_idx: AtomicUsize::new(0),
        })
    }

    /// Producer-Seite: `Ordering::Release` beim Veröffentlichen des neuen write_idx.
    pub fn try_push(&self, entry: WalEntry) -> Result<(), WalEntry> { /* CAS-Loop, Release-Semantik */ unimplemented!() }

    /// Consumer-Seite (Flusher-Task, exklusiv): `Ordering::Acquire` beim Lesen von write_idx.
    pub fn try_pop(&self) -> Option<WalEntry> { unimplemented!() }
}

#[derive(Debug, thiserror::Error)]
pub enum WalError {
    #[error("ring buffer capacity must be a power of two")]
    CapacityNotPowerOfTwo,
    #[error("hmac chain fork detected at sequence {0}")]
    HmacChainFork(u64),
}
```

Der Flusher-Task ist der **einzige** Aufrufer von `fsync`; er läuft als dedizierter `tokio::task`, der per `mpsc`
über neue `try_pop`-Ergebnisse benachrichtigt wird, statt zu pollen.

### 2.4 Block-Cache-Backend: `block_cache/mod.rs`

```rust
pub trait BlockCacheBackend<K, V>: Send + Sync {
    fn get(&self, key: &K) -> Option<V>;
    fn insert(&self, key: K, value: V);
    fn capacity(&self) -> usize;
    fn len(&self) -> usize;
}
```

**`block_cache/lru.rs` (Produktions-Default):**

```rust
use std::sync::RwLock;
use std::collections::HashMap;

pub struct LruBlockCacheBackend<K, V> {
    inner: RwLock<lru::LruCache<K, V>>, // externe `lru`-Crate
}

impl<K: std::hash::Hash + Eq + Clone, V: Clone + Send + Sync> BlockCacheBackend<K, V>
    for LruBlockCacheBackend<K, V>
{
    fn get(&self, key: &K) -> Option<V> {
        // Cache-Hit erfordert Write-Lock, da LRU-Reordering mutiert (P25-Verstoß, dokumentiert als
        // bewusster Trade-off des Default-Pfads — siehe SieveCacheBackend für den lock-freien Pfad).
        self.inner.write().unwrap().get(key).cloned()
    }
    fn insert(&self, key: K, value: V) { self.inner.write().unwrap().put(key, value); }
    fn capacity(&self) -> usize { self.inner.read().unwrap().cap().get() }
    fn len(&self) -> usize { self.inner.read().unwrap().len() }
}
```

**`block_cache/sieve.rs` (Opt-in `block-cache-v2`):**

```rust
use crossbeam_epoch::{self as epoch, Atomic, Owned, Shared};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

pub struct SieveNode<K, V> {
    pub key: K,
    pub value: V,
    pub visited: AtomicBool,
    pub next: Atomic<SieveNode<K, V>>,
}

pub struct SieveCacheBackend<K, V> {
    head: Atomic<SieveNode<K, V>>,
    tail: Atomic<SieveNode<K, V>>,
    hand: Atomic<SieveNode<K, V>>,
    capacity: usize,
    size: AtomicUsize,
    index: scc::HashMap<K, ()>, // Wert liegt in der Liste, Index verweist nur auf Vorhandensein
}

impl<K, V> BlockCacheBackend<K, V> for SieveCacheBackend<K, V>
where
    K: std::hash::Hash + Eq + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    fn get(&self, key: &K) -> Option<V> {
        let guard = epoch::pin();
        // Lookup über concurrent Hash-Map; bei Fund: einzige Mutation ist das visited-Bit.
        self.lookup_node(key, &guard).map(|node| {
            node.visited.store(true, Ordering::Relaxed);
            node.value.clone()
        })
    }

    fn insert(&self, key: K, value: V) {
        if self.size.load(Ordering::Relaxed) >= self.capacity {
            self.evict_one();
        }
        // Neuer Knoten wird per CAS an den Kopf gehängt; Epoch-Guard schützt Übergangszustand.
        self.push_front(key, value);
    }

    fn capacity(&self) -> usize { self.capacity }
    fn len(&self) -> usize { self.size.load(Ordering::Relaxed) }
}

impl<K, V> SieveCacheBackend<K, V>
where K: std::hash::Hash + Eq + Clone, V: Clone
{
    /// Eviction: `hand` wandert zyklisch. Gesetztes visited-Bit → löschen + weiterziehen (begnadigt).
    /// Gelöschtes visited-Bit → Knoten evicten.
    fn evict_one(&self) { unimplemented!("Hand-Traversierung, CAS-basiertes Entfernen aus verketteter Liste") }
    fn push_front(&self, key: K, value: V) { unimplemented!() }
    fn lookup_node<'g>(&self, key: &K, guard: &'g epoch::Guard) -> Option<&'g SieveNode<K, V>> { unimplemented!() }
}
```

**Sharding:** Beide Backends werden zusätzlich über ein `ShardedBlockCache<K, V, B: BlockCacheBackend<K,V>>`
mit konfigurierbarer Shard-Zahl (Default 16, `ahash`-basiertes Routing) gekapselt, um Contention auf einzelne
Shards zu begrenzen.

### 2.5 Öffentliche Storage-API (Auszug)

```rust
pub struct LsmStore {
    wal: WalRingBuffer,
    block_cache: Box<dyn BlockCacheBackend<BlockId, Bytes>>,
    kv_locks: KvKeyLocks,
}

impl LsmStore {
    pub fn get(&self, prefix: &str, key: &[u8]) -> Result<Option<Bytes>, StoreError>;
    pub fn put(&self, prefix: &str, key: &[u8], value: Bytes) -> Result<(), StoreError>;
    pub fn delete(&self, prefix: &str, key: &[u8]) -> Result<DeletionProof, StoreError>;
    pub fn scan_prefix(&self, prefix: &str) -> Result<impl Iterator<Item = (Vec<u8>, Bytes)>, StoreError>;
    pub fn compact(&self) -> Result<(), StoreError>;
    pub fn compact_async(&self, max_compaction_peak_memory_mb: usize) -> tokio::task::JoinHandle<Result<(), StoreError>>;
}

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error(transparent)] Wal(#[from] WalError),
    #[error(transparent)] Lock(#[from] LockError),
    #[error("compaction memory budget {budget_mb}MB exceeded (estimated {estimated_mb}MB)")]
    CompactionBudgetExceeded { budget_mb: usize, estimated_mb: usize },
    #[error(transparent)] Io(#[from] std::io::Error),
}
```

---

<a id="crypto"></a>
## 3. Layer 1: `memfuse-crypto`

```rust
// crates/memfuse-crypto/src/lib.rs
#![forbid(unsafe_code)]

use aes_gcm_siv::{Aes256GcmSiv, Nonce};
use std::sync::OnceLock;

/// Einmalige Instanziierung, um Key-Schedule-Neuaufbau pro Operation zu vermeiden (§9.2).
static CIPHER_INSTANCE: OnceLock<Aes256GcmSiv> = OnceLock::new();

pub fn cipher() -> &'static Aes256GcmSiv { /* init aus KeyMaterial, einmalig */ unimplemented!() }

pub struct DeletionProof {
    pub key_hash: [u8; 32],
    pub hmac_chain_entry: [u8; 32],
    pub prev_hmac: [u8; 32],
    pub timestamp: i64,
}

pub struct HmacChain {
    last: std::sync::Mutex<[u8; 32]>, // race-frei: einziger Mutationspunkt der Kette
}

impl HmacChain {
    /// MUSS atomar gegenüber gleichzeitigen `append`-Aufrufen sein — verhindert HMAC-Ketten-Forks (§2.3).
    pub fn append(&self, payload: &[u8]) -> Result<[u8; 32], CryptoError>;
    pub fn verify_chain(entries: &[[u8; 32]]) -> Result<(), CryptoError>;
}

#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("hmac chain fork at index {0}")]
    ChainFork(usize),
    #[error("aead operation failed")]
    AeadFailure,
}
```

---

<a id="text"></a>
## 4. Layer 1: `memfuse-text` (BM25/BM25F)

```rust
pub struct ResidentPostingIndex {
    postings: AHashMap<TermId, PostingList>,
    doc_lengths: Vec<u32>,
    field_lengths: AHashMap<(DocId, FieldId), u32>, // BM25F-Voraussetzung
}

pub struct Bm25fParams {
    pub k1: f32,
    pub b: f32,
    pub field_weights: AHashMap<FieldId, f32>, // z. B. Titel: 2.0, Fließtext: 1.0, Kontext-Präfix: 0.5
}

impl ResidentPostingIndex {
    /// Block-Max WAND: liefert Top-k ohne vollständige Postinglisten-Traversierung.
    pub fn search_topk(&self, query_terms: &[TermId], k: usize, params: &Bm25fParams) -> Vec<(DocId, f32)>;

    /// BM25F-Score für ein einzelnes Dokument, feldgewichtet:
    /// score(D,Q) = Σ_t∈Q  idf(t) · Σ_f  w_f · tf(t,f,D) / (1 - b_f + b_f · |D_f| / avgfl_f)
    fn bm25f_score(&self, doc: DocId, terms: &[TermId], params: &Bm25fParams) -> f32;
}

/// Deutsche Kompositazerlegung: `Urlaubsantragsprozess` → [`Urlaub`, `Antrag`, `Prozess`, `Urlaubsantragsprozess`]
pub fn decompose_german_compound(word: &str, dictionary: &CompoundDictionary) -> Vec<String>;
```

Persistenzformat: residenter Index wird beim Start aus dem LSM-Präfix `__text:posting:` vollständig materialisiert
(kein Live-Scan pro Query-Term); Schreibpfad aktualisiert LSM **und** residenten Index atomar unter `kv_locks`.

---

<a id="index"></a>
## 5. Layer 1: `memfuse-index` (HNSW/DiskANN)

### 5.1 HNSW-Kern (Stufe 0, produktiv)

```rust
pub struct HnswIndex<const D: usize> {
    layers: Vec<HnswLayer<D>>,
    entry_point: AtomicUsize,
    sq8_codebook: Sq8Codebook,
}

pub struct Sq8Codebook {
    pub min: [f32; D_MAX],
    pub max: [f32; D_MAX],
    /// Perzentil statt rohem Min/Max, um Ausreißer-getriebenen Codebook-Drift zu vermeiden.
    pub clip_percentile: f32, // Default 0.999
}

impl<const D: usize> HnswIndex<D> {
    pub fn search_knn(&self, query: &[f32; D], k: usize, ef_search: usize) -> Vec<(DocId, f32)>;
    pub fn insert(&mut self, id: DocId, vector: [f32; D]) -> Result<(), IndexError>;
    pub fn delete(&mut self, id: DocId) -> Result<(), IndexError>; // native Tombstone, kein Fallback nötig
}
```

Distanzberechnung nutzt unaligned-SIMD-Kernel (kein `Vec<u32>`-Heap-Alloc pro Nachbarabruf); Kandidatenmengen
nutzen `AHashSet::with_capacity(ef_search)` vorallokiert; Lazy-Pruning versucht `try_write()` vor blockierendem
`write()` und überspringt bei Kontention statt zu warten.

### 5.2 NaN-sichere Distanz-Pipeline

```rust
/// Bitweise SIMD-Maskierung statt Branch: NaN-Werte werden auf Distanz f32::INFINITY gesetzt.
#[inline]
fn masked_l2_distance_avx512(a: &[f32], b: &[f32]) -> f32 {
    // SAFETY: `a`/`b` sind 64-Byte-aligned und exakt D Elemente lang, geprüft durch Aufrufer.
    unsafe { /* _mm512_cmp_ps_mask auf NaN, _mm512_mask_blend_ps mit INFINITY */ unimplemented!() }
}
```

### 5.3 HNSW-Dateiformat v2: Arena-Allocator (Zielarchitektur, §7.4)

```rust
pub struct HnswArena<const D: usize> {
    storage: std::sync::Arc<MmapArena>,
    head: crossbeam_epoch::Atomic<NodeRecord<D>>,
    capacity: usize,
}

pub struct NodeRecord<const D: usize> {
    pub vector: [f32; D],
    pub neighbor_offsets: [u32; MAX_M],   // Offsets statt Pointer
    pub neighbor_count: u16,
}

impl<const D: usize> HnswArena<D> {
    /// Relinking über CAS statt Mutex — alte Adjazenzliste bleibt bis Epoch-Reclaim gültig.
    pub fn relink(&self, node_offset: u32, new_neighbors: &[u32]) -> Result<(), IndexError>;
}
```

### 5.4 DiskANN-Tier (`experimental-diskann`)

```rust
pub struct DiskAnnIndex<const D: usize> {
    mmap: memmap2::Mmap,
    tombstones: scc::HashSet<DocId>, // native, kein Fallback auf HNSW nötig
    tombstone_wal: TombstoneWal,
}

impl<const D: usize> DiskAnnIndex<D> {
    pub fn delete(&self, id: DocId) -> Result<(), IndexError> {
        // Bei fehlendem hnsw_fallback: eigener Tombstone-Satz + Tombstone-WAL statt InvalidInput.
        self.tombstones.insert(id).map_err(|_| IndexError::AlreadyDeleted)?;
        self.tombstone_wal.append(id)
    }
}

pub enum VectorIndexTier {
    Hnsw,                     // Default, mutable
    #[cfg(feature = "experimental-diskann")]
    DiskAnn,                  // große, überwiegend lesende Collections
}
```

---

<a id="graph"></a>
## 6. Layer 1: `memfuse-graph` (CSR, Forward-Push-PPR, Leiden, Hyperkanten)

### 6.1 Modulstruktur

```
crates/memfuse-graph/src/
├── lib.rs
├── csr.rs             # CsrGraph, GraphInner, ArcSwap-RCU
├── edge.rs             # Edge, EdgeType-Nutzung, PersistedEdgePayload
├── hyperedge.rs        # HyperEdge, RoleBinding, RoleId, HyperEdgeId — NEU
├── ppr.rs               # Forward-Push (Andersen-Chung-Lang)
├── path_rag.rs          # PathGraph, bidirektionale Suche, Hyperkanten-Expansion
├── community.rs        # Leiden, Stern-Expansion-Projektion — Hyperkanten-Teil NEU
├── cascade.rs           # Cascade-Invalidierung binär + Hyperkanten (NEU)
└── error.rs
```

### 6.2 `csr.rs` — RCU-Snapshot-Architektur

```rust
use arc_swap::ArcSwap;
use ahash::AHashMap;
use std::sync::Arc;

pub struct GraphInner {
    pub adjacency: Vec<Vec<Edge>>,               // CSR, binäre Kanten
    pub node_index: AHashMap<EntityId, usize>,
    // NEU (§6.5, H1): Teil von GraphInner, NICHT separat — automatisch vom ArcSwap miterfasst.
    pub hyperedges: AHashMap<HyperEdgeId, HyperEdge>,
    pub hyperedge_index: AHashMap<EntityId, Vec<HyperEdgeId>>,
}

impl GraphInner {
    /// MUSS um Hyperkanten-Anteile erweitert sein (§6.5, H1) — Voraussetzung für IP-20,
    /// da `compact_async`'s Speicherbudget-Check sonst das Peak-Memory unterschätzt.
    pub fn estimate_memory_bytes(&self) -> usize {
        let edge_bytes: usize = self.adjacency.iter().map(|v| v.len() * std::mem::size_of::<Edge>()).sum();
        let hyperedge_bytes: usize = self.hyperedges.values()
            .map(|h| std::mem::size_of::<HyperEdge>() + h.participants.len() * std::mem::size_of::<RoleBinding>())
            .sum();
        let hyperedge_index_bytes: usize = self.hyperedge_index.values()
            .map(|v| v.len() * std::mem::size_of::<HyperEdgeId>())
            .sum();
        edge_bytes + hyperedge_bytes + hyperedge_index_bytes
    }
}

pub struct CsrGraph {
    inner: ArcSwap<GraphInner>,
    kv_locks: Arc<memfuse_store::KvKeyLocks>,
}

impl CsrGraph {
    /// Atomarer Snapshot-Austausch. Leser sehen NIE einen gemischten Alt-/Neu-Zustand,
    /// einschließlich der Hyperkanten-Strukturen (H1).
    pub fn compact(&self) -> Result<(), GraphError> {
        let old = self.inner.load();
        let new_inner = Self::rebuild(&old)?;
        self.inner.store(Arc::new(new_inner));
        Ok(())
    }

    pub fn compact_async(&self, max_compaction_peak_memory_mb: usize) -> tokio::task::JoinHandle<Result<(), GraphError>> {
        let estimated = self.inner.load().estimate_memory_bytes() / (1024 * 1024);
        if estimated > max_compaction_peak_memory_mb {
            return tokio::spawn(async move {
                Err(GraphError::CompactionBudgetExceeded { budget_mb: max_compaction_peak_memory_mb, estimated_mb: estimated })
            });
        }
        // TODO(IP-08-BUDGET-COUPLING): vor IP-20-Merge prüfen, ob dieser Marker noch nötig ist.
        unimplemented!()
    }

    pub fn neighbors_with_weights(&self, id: EntityId) -> Vec<(EntityId, f32)> { unimplemented!() }

    /// NEU: Sekundärindex-Zugriff, additiv — kein Eingriff in die CSR-Adjazenzstruktur (§6.4).
    pub fn hyperedges_for_entity(&self, id: EntityId) -> Vec<HyperEdgeId> {
        self.inner.load().hyperedge_index.get(&id).cloned().unwrap_or_default()
    }
}
```

### 6.3 `edge.rs` — binäre Kante

```rust
#[derive(Debug, Clone)]
pub struct Edge {
    pub target: EntityId,
    pub weight: f32,
    pub edge_type: EdgeType,
    pub tx_valid_from: TxId,
    pub tx_valid_to: Option<TxId>,
    pub business_valid_from: Option<i64>,
    pub business_valid_to: Option<i64>,
    pub source_doc_id: Option<DocId>,
}

/// LSM-Präfix `__graph:edge:`. Unverändert, unangetasteter Hotpath (§6.3-Invariante).
pub struct PersistedEdgePayload { /* FlatBuffers-serialisiert, siehe §15 */ }
```

### 6.4 `hyperedge.rs` — NEU, vollständig

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HyperEdgeId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RoleId(pub u32);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoleBinding {
    pub role: RoleId,
    pub entity: EntityId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HyperEdge {
    pub id: HyperEdgeId,
    pub predicate: EdgeType,
    pub participants: Vec<RoleBinding>,           // min. 2, validiert in `relate_n_ary`
    pub weight: f32,
    pub tx_valid_from: Option<TxId>,
    pub tx_valid_to: Option<TxId>,
    pub business_valid_from: Option<i64>,
    pub business_valid_to: Option<i64>,
    pub source_doc_id: Option<DocId>,
}

/// Rollen-Interner — dieselbe Interning-Strategie wie EdgeType/Prädikate, um Allokationen im
/// Hotpath zu vermeiden.
pub struct RoleInterner {
    forward: scc::HashMap<String, RoleId>,
    backward: scc::HashMap<RoleId, String>,
    next_id: std::sync::atomic::AtomicU32,
}

impl RoleInterner {
    pub fn intern(&self, name: &str) -> RoleId;
    pub fn resolve(&self, id: RoleId) -> Option<String>;
}

#[derive(Debug, thiserror::Error)]
pub enum GraphMutationError {
    #[error("lock acquisition timed out")]
    LockAcquisitionTimeout,
    #[error("cascade fan-out limit exceeded, {0} hyperedges queued for background processing")]
    PartialCascadeQueued(memfuse_crypto::DeletionProof),
    #[error("role binding invalid: {0}")]
    RoleBindingInvalid(String),
    #[error("rcu snapshot reclamation pending, retry")]
    EpochReclamationPending,
    #[error("hyperedge requires >= 2 participants, got {0}")]
    InsufficientParticipants(usize),
}
```

### 6.5 `relate_n_ary` — vollständige Implementierungsvorgabe (H2)

```rust
impl CsrGraph {
    /// Öffentlicher Mutationspfad für Hyperkanten. Kanonisches Locking ist PFLICHT (H2) —
    /// keine Analogie-Berufung auf den Einzelschlüssel-Beweis der `relate()`-Hierarchie.
    pub fn relate_n_ary(
        &self,
        predicate: EdgeType,
        participants: &[RoleBinding],
        doc_id: DocId,
    ) -> Result<HyperEdgeId, GraphMutationError> {
        if participants.len() < 2 {
            return Err(GraphMutationError::InsufficientParticipants(participants.len()));
        }

        // 1. Kanonische Sortierung (H2) — Grundlage der Deadlockfreiheit.
        let mut entities: Vec<EntityId> = participants.iter().map(|p| p.entity).collect();
        entities.sort_unstable_by_key(|e| e.0);
        entities.dedup();
        let key_hashes: Vec<u64> = entities.iter().map(|e| ahash::RandomState::new().hash_one(e)).collect();

        // 2. Multi-Key-Lock in sortierter Shard-Reihenfolge.
        let _guards = self.kv_locks.acquire_multi_sorted(&key_hashes)
            .map_err(|_| GraphMutationError::LockAcquisitionTimeout)?;

        // 3. Atomare LSM-Schreibung: Primär-Eintrag `__graph:hyperedge:{id}` +
        //    Sekundärindex-Einträge `__graph:hyperedge_by_entity:{entity}` für JEDEN Teilnehmer,
        //    in derselben Transaktion.
        let id = HyperEdgeId(self.next_hyperedge_id());
        let hyperedge = HyperEdge {
            id, predicate, participants: participants.to_vec(), weight: 1.0,
            tx_valid_from: None, tx_valid_to: None,
            business_valid_from: None, business_valid_to: None,
            source_doc_id: Some(doc_id),
        };
        self.persist_hyperedge_atomic(&hyperedge)?;

        // 4. RCU-Registrierung (H1): erfolgt über denselben `compact`/Insert-Pfad wie GraphInner-Updates,
        //    sodass der nächste ArcSwap-Snapshot die Hyperkante enthält.
        self.register_in_rcu_snapshot(&hyperedge)?;

        Ok(id)
    }

    fn next_hyperedge_id(&self) -> u64 { unimplemented!() }
    fn persist_hyperedge_atomic(&self, h: &HyperEdge) -> Result<(), GraphMutationError> { unimplemented!() }
    fn register_in_rcu_snapshot(&self, h: &HyperEdge) -> Result<(), GraphMutationError> { unimplemented!() }
}
```

**Loom-Testpflicht (AK-3):** `crates/memfuse-graph/tests/loom_relate_n_ary.rs` modelliert zwei nebenläufige
`relate_n_ary`-Aufrufe mit den Teilnehmermengen `[A,B,C]` und `[C,B,A]` (überlappend, unterschiedlich geordnet)
und beweist Terminierung ohne Deadlock unter `#[cfg(loom)]`.

### 6.6 `path_rag.rs` — Forward-Push-PPR mit Hyperkanten-Expansion

```rust
pub struct PprParams {
    pub alpha: f32,          // Teleport-Wahrscheinlichkeit
    pub epsilon: f32,        // Fehlertoleranz-Schwellenwert
    pub hyperedge_decay: f32, // Default 0.85 — Gewichtsabschlag für virtuelle Hyperkanten-Nachbarn
}

/// Andersen-Chung-Lang Forward-Push. Laufzeit O(1/(alpha*epsilon)), UNABHÄNGIG von |V|+|E| (P24).
pub fn forward_push_ppr(
    graph: &CsrGraph,
    seeds: &[EntityId],
    params: &PprParams,
) -> AHashMap<EntityId, f32> {
    let mut p: AHashMap<EntityId, f32> = AHashMap::new();
    let mut r: AHashMap<EntityId, f32> = seeds.iter().map(|&s| (s, 1.0 / seeds.len() as f32)).collect();
    let mut queue: std::collections::VecDeque<EntityId> = seeds.iter().copied().collect();

    while let Some(u) = queue.pop_front() {
        let degree = graph.degree(u).max(1) as f32;
        let r_u = *r.get(&u).unwrap_or(&0.0);
        if r_u / degree <= params.epsilon { continue; }

        *p.entry(u).or_insert(0.0) += params.alpha * r_u;
        let residual_kept = (1.0 - params.alpha) * r_u / 2.0;
        r.insert(u, residual_kept);

        let push_share = (1.0 - params.alpha) * r_u / (2.0 * degree);

        // Binäre Nachbarn (unveränderter Hotpath).
        for (v, _w) in graph.neighbors_with_weights(u) {
            let entry = r.entry(v).or_insert(0.0);
            *entry += push_share;
            queue.push_back(v);
        }

        // NEU (§6.4, H3): Hyperkanten-Partner als virtuelle Nachbarn, Gewichtsabschlag.
        for hedge_id in graph.hyperedges_for_entity(u) {
            for role_binding in graph.hyperedge_participants(hedge_id) {
                if role_binding.entity == u { continue; }
                let entry = r.entry(role_binding.entity).or_insert(0.0);
                *entry += push_share * params.hyperedge_decay;
                queue.push_back(role_binding.entity);
            }
        }
    }
    p
}
```

**H3-Diff-Test-Pflicht:** `crates/memfuse-db/tests/signal_kind_no_new_variant.rs` prüft per Makro-/Reflection-
Vergleich, dass `SignalKind` weiterhin exakt vier Varianten hat (`Vector`, `Text`, `Graph`, `EdgeReinforcement`) —
kein `Hyperedge`-Signal.

### 6.7 `community.rs` — Leiden + Stern-Expansion

```rust
pub struct CommunityDetectionConfig {
    pub resolution_gamma: f32,
    /// H6: sichtbares Unvollständigkeits-Flag, Default false, solange keine Projektion existiert.
    pub hyperedges_included: bool,
}

pub struct CommunityAssignment {
    pub node_to_community: AHashMap<EntityId, u32>,
    pub hyperedges_included: bool, // wird 1:1 aus Config übernommen, im Report sichtbar mitgeführt
}

/// Stern-Expansion: jede Hyperkante wird zu einem künstlichen bipartiten Knoten, KEINE
/// physische Materialisierung — Iterator gaukelt dem Leiden-Solver die Inzidenzmatrix H vor.
pub struct StarExpansionIterator<'a> {
    graph: &'a CsrGraph,
    current_hyperedge_idx: usize,
}

impl<'a> Iterator for StarExpansionIterator<'a> {
    type Item = (EntityId, VirtualHyperedgeNode); // binäre "Kante" Entität ↔ virtueller Hyperkanten-Knoten
    fn next(&mut self) -> Option<Self::Item> { unimplemented!() }
}

pub fn detect_communities(graph: &CsrGraph, config: &CommunityDetectionConfig) -> CommunityAssignment {
    // config.hyperedges_included steuert, ob StarExpansionIterator dem C-FFI-Leiden-Solver
    // zusätzlich zu den binären Kanten vorgelegt wird. Solange `false`: unverändertes Verhalten,
    // aber SICHTBAR im Report (H6) statt stillschweigend.
    unimplemented!()
}
```

### 6.8 `cascade.rs` — Fan-out-begrenzte Invalidierung (H5)

```rust
pub const DEFAULT_HYPEREDGE_CASCADE_FANOUT_LIMIT: usize = 1_000;

pub struct CascadeReport {
    pub tombstoned_synchronously: usize,
    pub queued_for_background: usize,
    pub deletion_proof: Option<memfuse_crypto::DeletionProof>,
}

/// Symmetrisch zu `cascade_invalidate_edges_for_superseded_doc`, aber mit hartem Fan-out-Limit (H5, P24).
pub fn cascade_invalidate_hyperedges_for_superseded_doc(
    graph: &CsrGraph,
    doc_id: DocId,
    fanout_limit: usize, // Default DEFAULT_HYPEREDGE_CASCADE_FANOUT_LIMIT
) -> Result<CascadeReport, GraphMutationError> {
    let affected = graph.hyperedges_for_doc(doc_id);
    if affected.len() <= fanout_limit {
        // Synchron: alle RoleBindings gemeinsam atomar tombstonieren (EIN Tombstone statt N).
        for hedge_id in &affected {
            graph.tombstone_hyperedge_atomic(*hedge_id)?;
        }
        Ok(CascadeReport { tombstoned_synchronously: affected.len(), queued_for_background: 0, deletion_proof: None })
    } else {
        let (sync_part, async_part) = affected.split_at(fanout_limit);
        for hedge_id in sync_part {
            graph.tombstone_hyperedge_atomic(*hedge_id)?;
        }
        let proof = enqueue_background_cascade(async_part.to_vec()); // background_workers.rs
        Err(GraphMutationError::PartialCascadeQueued(proof))
    }
}

fn enqueue_background_cascade(remaining: Vec<HyperEdgeId>) -> memfuse_crypto::DeletionProof { unimplemented!() }
```

**Idempotenz-/Integrationstestpflicht:** `cascade_invalidate_hyperedges_for_superseded_doc` MUSS bei wiederholtem
Lauf auf denselben `doc_id` keine doppelten Tombstones erzeugen (Vorlage: bestehende
`cascade_invalidation_integration.rs`, `supersedes_cascading_tombstone_test.rs`).

---

<a id="checkpoint"></a>
## 7. Layer 1: `memfuse-checkpoint`

```rust
pub struct Checkpoint {
    pub snapshot_id: u64,
    pub wal_offset: u64,
    pub created_at: i64,
}

pub trait Checkpointable {
    fn snapshot(&self) -> Result<Checkpoint, CheckpointError>;
    fn restore(&mut self, checkpoint: &Checkpoint) -> Result<(), CheckpointError>;
}
```

---

<a id="calibration"></a>
## 8. Layer 1: `memfuse-calibration`

```rust
pub struct ScoreCalibrator {
    threshold: std::sync::atomic::AtomicU32, // f32-Bits, atomar lesbar/schreibbar
    drift_detector: LyapunovDriftWatcher,
}

/// Gedeckelter Lyapunov-Drift-Regelkreis (§8.3 der Gesamtspezifikation, hier implementierungsgenau).
pub struct LyapunovDriftWatcher {
    pub drift_decay_window: u32,   // Default 50
    pub drift_gamma: f32,          // Default 0.95
    steps_remaining: std::sync::atomic::AtomicU32,
    integrator_state: std::sync::atomic::AtomicU32, // f32-Bits
}

impl LyapunovDriftWatcher {
    /// Anti-Windup: bei PID-Sättigung (CPU-Limit) stoppt der Integrator sofort statt aufzusummieren.
    pub fn update(&self, error: f32, dt_seconds: f32, saturated: bool) -> f32 {
        if saturated {
            return self.current_alpha();
        }
        // Zeitfensterbasierte, gedeckelte Eskalation statt `alpha *= k_drift` unbegrenzt.
        unimplemented!()
    }
    pub fn current_alpha(&self) -> f32 { unimplemented!() }
}
```

---

<a id="db"></a>
## 9. Layer 2: `memfuse-db` (Collection-API, Fusion, Provenance)

### 9.1 `Collection`-Kernstruktur

```rust
pub struct Collection {
    store: memfuse_store::LsmStore,
    vector_index: Box<dyn VectorIndex>,
    text_index: memfuse_text::ResidentPostingIndex,
    graph: memfuse_graph::CsrGraph,
    calibrator: memfuse_calibration::ScoreCalibrator,
    fusion_mode: FusionMode,
}

pub enum FusionMode {
    ReciprocalRankFusion,                      // Default
    ScoreNormalized { rrf_fallback: bool },    // Opt-in, hartes Fallback bei Signaldegradation
}
```

### 9.2 Öffentliche Mutations-API

```rust
impl Collection {
    pub fn insert(&self, doc: DocumentInput) -> Result<DocId, DbError>;
    pub fn upsert(&self, id: DocId, doc: DocumentInput) -> Result<(), DbError>;
    pub fn delete(&self, id: DocId) -> Result<memfuse_crypto::DeletionProof, DbError>;

    /// Binärer Pfad — bleibt Hotpath, KEINE interne Umleitung auf `relate_n_ary` mit 2 Teilnehmern.
    pub fn relate(&self, from: EntityId, to: EntityId, predicate: EdgeType, doc_id: DocId) -> Result<(), DbError>;

    /// NEU — Hyperkanten-Schreibpfad (§6.5).
    pub fn relate_n_ary(
        &self,
        predicate: EdgeType,
        participants: &[(RoleId, EntityId)],
        doc_id: DocId,
    ) -> Result<HyperEdgeId, DbError> {
        let bindings: Vec<RoleBinding> = participants.iter()
            .map(|(role, entity)| RoleBinding { role: *role, entity: *entity })
            .collect();
        self.graph.relate_n_ary(predicate, &bindings, doc_id).map_err(DbError::from)
    }
}
```

### 9.3 Suchpfad und 4-Signal-Fusion

```rust
pub struct SearchRequest {
    pub query_vector: Option<Vec<f32>>,
    pub query_text: Option<String>,
    pub graph_seeds: Option<Vec<EntityId>>,
    pub k: usize,
}

pub struct SearchResult {
    pub doc_id: DocId,
    pub fused_score: f32,
    pub provenance: ProvenanceRecord,
}

impl Collection {
    pub fn hybrid_search(&self, req: &SearchRequest) -> Result<Vec<SearchResult>, DbError> {
        let mut signals: Vec<(SignalKind, Vec<(DocId, f32)>)> = Vec::new();
        if let Some(v) = &req.query_vector {
            signals.push((SignalKind::Vector, self.vector_index.search_knn(v, req.k)));
        }
        if let Some(t) = &req.query_text {
            signals.push((SignalKind::Text, self.text_index.search_topk(&tokenize(t), req.k, &self.bm25f_params())));
        }
        if let Some(seeds) = &req.graph_seeds {
            // Enthält bereits die Hyperkanten-Expansion (§6.6) — kein separates Signal (H3).
            let ppr = memfuse_graph::forward_push_ppr(&self.graph, seeds, &self.ppr_params());
            signals.push((SignalKind::Graph, ppr.into_iter().map(|(e, s)| (self.doc_for_entity(e), s)).collect()));
        }
        Ok(match self.fusion_mode {
            FusionMode::ReciprocalRankFusion => fuse_rrf(&signals, req.k),
            FusionMode::ScoreNormalized { rrf_fallback } => fuse_score_normalized(&signals, req.k, rrf_fallback),
        })
    }
}
```

### 9.4 `SignalKind` — geschlossenes Enum (H3, normativ unveränderlich)

```rust
/// BEWUSST NICHT `#[non_exhaustive]` — Erweiterung erfolgt NIEMALS durch neue Varianten,
/// sondern durch Einspeisung zusätzlicher Kandidaten in ein bestehendes Signal (§6.5, H3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SignalKind {
    Vector,
    Text,
    Graph,
    EdgeReinforcement, // feature-gated: `edge-reinforcement-learning`
}

impl SignalKind {
    /// Allokationsfrei, `eq_ignore_ascii_case` statt `to_lowercase()`-Allokation.
    pub fn from_name(name: &str) -> Option<Self> {
        if name.eq_ignore_ascii_case("vector") { Some(Self::Vector) }
        else if name.eq_ignore_ascii_case("text") { Some(Self::Text) }
        else if name.eq_ignore_ascii_case("graph") { Some(Self::Graph) }
        else if name.eq_ignore_ascii_case("edgereinforcement") { Some(Self::EdgeReinforcement) }
        else { None }
    }
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Vector => "vector", Self::Text => "text",
            Self::Graph => "graph", Self::EdgeReinforcement => "edgereinforcement",
        }
    }
}
```

### 9.5 `ProvenanceBuilder`

```rust
#[derive(Default)]
pub struct ProvenanceBuilder {
    source_doc_id: Option<DocId>,
    signal_contributions: Vec<(SignalKind, f32)>,
    fusion_mode: Option<FusionMode>,
    calibrated_threshold: Option<f32>,
}

impl ProvenanceBuilder {
    pub fn source_doc_id(mut self, id: DocId) -> Self { self.source_doc_id = Some(id); self }
    pub fn add_signal(mut self, kind: SignalKind, score: f32) -> Self { self.signal_contributions.push((kind, score)); self }
    pub fn fusion_mode(mut self, mode: FusionMode) -> Self { self.fusion_mode = Some(mode); self }
    pub fn calibrated_threshold(mut self, t: f32) -> Self { self.calibrated_threshold = Some(t); self }
    pub fn build(self) -> Result<ProvenanceRecord, DbError> { /* validiert Pflichtfelder */ unimplemented!() }
}
```

### 9.6 `DbError`

```rust
#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error(transparent)] Store(#[from] memfuse_store::StoreError),
    #[error(transparent)] Graph(#[from] memfuse_graph::GraphMutationError),
    #[error(transparent)] Index(#[from] memfuse_index::IndexError),
    #[error("provenance builder missing required field: {0}")]
    ProvenanceIncomplete(&'static str),
}
```

---

<a id="router"></a>
## 10. Layer 3: `memfuse-router` (Contextual Bandit)

### 10.1 Gemeinsame Schnittstelle

```rust
pub trait BanditPolicy: Send + Sync {
    fn select_arm(&self, context: &[f32]) -> RetrievalStrategy;
    fn update(&mut self, context: &[f32], arm: RetrievalStrategy, reward: f32);
}

pub enum RetrievalStrategy { Vector, Text, Graph, Hybrid }
```

### 10.2 `DiagonalApproximation` (Produktions-Default)

```rust
pub struct DiagonalApproximationBandit {
    theta: Vec<f32>,
    sigma_sq: Vec<f32>,
    drift: LyapunovDriftWatcher, // aus memfuse-calibration
}

impl BanditPolicy for DiagonalApproximationBandit {
    fn update(&mut self, context: &[f32], _arm: RetrievalStrategy, reward: f32) {
        for (i, &xi) in context.iter().enumerate() {
            let r_adj = reward; // ggf. drift-korrigiert
            self.theta[i] += r_adj * xi / self.sigma_sq[i].max(1e-8);
            self.sigma_sq[i] += xi * xi;
        }
    }
    fn select_arm(&self, context: &[f32]) -> RetrievalStrategy { unimplemented!() }
}
```

### 10.3 `ShermanMorrisonBandit` (Opt-in, `egress-sherman-morrison`)

```rust
#[repr(C, align(64))]
pub struct AlignedVector<const D: usize> { pub data: [f32; D] }

pub struct ShermanMorrisonBandit<const D: usize> {
    pub inv_a: AlignedVector<{ D * D }>, // A^{-1}, flach, row-major
    pub b: AlignedVector<D>,
    pub theta: AlignedVector<D>,
    pub lambda: f32,                     // Ridge-Regularisierung
}

impl<const D: usize> ShermanMorrisonBandit<D> {
    /// θ = A⁻¹b, A = Σ xₜxₜᵀ + λI. Rang-1-Update in O(d²) statt O(d³) Neuinversion.
    /// (A + xxᵀ)⁻¹ = A⁻¹ - (A⁻¹x xᵀA⁻¹) / (1 + xᵀA⁻¹x)
    pub fn update_rank_1(&mut self, x: &AlignedVector<D>, reward: f32) -> Result<(), BanditError> {
        let v = self.matvec_inv_a(x);              // SAFETY: 64-byte aligned, D Elemente — AVX-512 FMA
        let s = 1.0 + Self::dot(x, &v);
        if s.abs() < 1e-8 { return Err(BanditError::SingularUpdate); }
        self.rank1_update_inv_a(&v, s);             // inv_a -= (v vᵀ) / s
        self.b.data.iter_mut().zip(x.data.iter()).for_each(|(bi, xi)| *bi += reward * xi);
        self.theta = self.matvec_inv_a(&self.b);
        Ok(())
    }

    fn matvec_inv_a(&self, x: &AlignedVector<D>) -> AlignedVector<D> { unimplemented!("AVX-512/NEON intrinsics") }
    fn rank1_update_inv_a(&mut self, v: &AlignedVector<D>, s: f32) { unimplemented!("FMA-Instruktionen") }
    fn dot(a: &AlignedVector<D>, b: &AlignedVector<D>) -> f32 { unimplemented!() }
}

#[derive(Debug, thiserror::Error)]
pub enum BanditError {
    #[error("rank-1 update denominator near zero")]
    SingularUpdate,
}
```

**CI-Gate-Pflicht:** `xtask check-bandit-latency-budget` MUSS `ShermanMorrisonBandit::update_rank_1`-Latenz gegen
ein Kriterium (< 5 % der medianen LLM/SLM-Inferenzlatenz) messen und als Merge-Gate in
`.github/workflows/merge-gate.yml` eingebunden sein, bevor der Produktions-Default umgestellt wird.

---

<a id="candle"></a>
## 11. Layer 3: `memfuse-candle` (Inferenz, KV-Cache-Bridge)

```rust
pub struct KvCacheBridge {
    ram_cache: scc::HashMap<SessionId, EncryptedKvSegment>,
    lsm_fallback: memfuse_store::LsmStore,
    cipher_worker: CipherWorkerHandle, // mpsc-Channel zu OnceLock<Aes256GcmSiv>-Worker
}

pub struct EncryptedKvSegment {
    pub ciphertext: Vec<u8>,
    pub rope_offset: u32,
    pub model_fingerprint: [u8; 32],
}

impl KvCacheBridge {
    pub fn get(&self, session: SessionId, model_fingerprint: [u8; 32]) -> Result<Option<KvCache>, KvBridgeError> {
        if let Some(seg) = self.ram_cache.get(&session) {
            if seg.model_fingerprint != model_fingerprint {
                return Err(KvBridgeError::FingerprintMismatch);
            }
            return Ok(Some(self.decrypt(seg)));
        }
        // LSM-Fallback-Spill: kontrolliertes Nachladen statt verlustbehafteten Verwerfens.
        self.load_from_lsm_fallback(session)
    }

    fn decrypt(&self, seg: &EncryptedKvSegment) -> KvCache { unimplemented!() }
    fn load_from_lsm_fallback(&self, session: SessionId) -> Result<Option<KvCache>, KvBridgeError> { unimplemented!() }
}

#[derive(Debug, thiserror::Error)]
pub enum KvBridgeError {
    #[error("rope offset mismatch")]
    RopeOffsetMismatch,
    #[error("model fingerprint mismatch")]
    FingerprintMismatch,
}
```

**Testpflicht:** Testsuite deckt alle vier Kombinationen ab: RAM-Hit, RAM-Miss/LSM-Hit, beide-Miss,
Fingerprint-Mismatch.

---

<a id="layer3-rest"></a>
## 12. Layer 3: `memfuse-ollama`, `memfuse-embed`, `memfuse-agent`, `memfuse-py`

- **`memfuse-ollama`:** `OllamaClient::generate(prompt: &str, contextual_prefix: &[Chunk]) -> Result<String, OllamaError>`.
  Contextual-Chunk-Prefixing fügt Retrieval-Kontext als System-Präfix ein, keine Änderung an der
  Fusionslogik selbst.
- **`memfuse-embed`:** `EmbeddingModel::embed(texts: &[String]) -> Result<Vec<Vec<f32>>, EmbedError>` (ONNX,
  feature-gated `unsafe` für C-FFI, §0.4), `CrossEncoderReranker::rerank(query: &str, candidates: &[SearchResult]) -> Vec<SearchResult>`.
- **`memfuse-agent`:** `AgentWorkflow`-Engine mit persistentem Zustand über `memfuse_checkpoint::Checkpointable`.
- **`memfuse-py`:** PyO3-Bindings in eigenem Cargo-Workspace (Panic-Strategie-Isolation: `panic = "unwind"` nur
  hier, restlicher Workspace `panic = "abort"` für Release-Profile). `DocId`-Integer-Grenzen sind
  feature-abhängig (`docid-128`) gegenüber Python `int` zu deklarieren.

---

<a id="mcp"></a>
## 13. Layer 4: `memfuse-mcp` (Server, Sandbox, Egress-Gateway)

### 13.1 Sandbox: orthogonale Zeitbudgets

```rust
pub struct WasmCapabilities {
    pub max_fuel: Option<u64>,
    pub max_wall_clock_ms: u64, // Default 5000; 0 = unbegrenzt, Fallback auf Caller-Timeout
    pub allow_cloud_egress: bool, // Default false
}

pub struct SandboxExecutor {
    engine: wasmtime::Engine,
}

impl SandboxExecutor {
    pub fn execute(&self, module: &[u8], caps: &WasmCapabilities) -> Result<Vec<u8>, SandboxError> {
        let mut store = wasmtime::Store::new(&self.engine, ());
        if let Some(fuel) = caps.max_fuel { store.set_fuel(fuel).map_err(SandboxError::from)?; }
        // Wall-Clock unabhängig von Fuel via separatem Timeout-Task (tokio::time::timeout),
        // NICHT über denselben Mechanismus wie Fuel (P23-Orthogonalität).
        unimplemented!()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SandboxError {
    #[error("fuel budget exhausted")]
    FuelExhausted,
    #[error("wall clock budget of {0}ms exceeded")]
    WallClockExceeded(u64),
}
```

`fd_write`-WASI-Stub MUSS `iovs` korrekt parsen (kein Ignorieren der Vektorlängen).

### 13.2 Cloud-Egress-Gateway

```rust
pub struct EgressVault {
    pattern_matcher: regex::RegexSet,
    surrogate_map: scc::HashMap<SurrogateToken, OriginalEntity>, // session-gebunden
}

impl EgressVault {
    pub fn generate_surrogate(&self, entity: &OriginalEntity, session: SessionId) -> SurrogateToken {
        // Format: [USER_ENTITY_xxxx], Hash-basiert, session-gebunden bidirektional.
        unimplemented!()
    }
    pub fn get_entity(&self, token: &SurrogateToken) -> Option<OriginalEntity> { unimplemented!() }
}

pub struct BulkExfiltrationDetector {
    pub max_bytes_per_window: usize,
    pub window: std::time::Duration,
}

pub struct CloudResponseRehydrator;
impl CloudResponseRehydrator {
    /// Round-Trip-sicher; unbekannte Surrogat-Token = No-Op; Multibyte-UTF-8-panic-sicher.
    pub fn rehydrate(&self, response: &str, vault: &EgressVault) -> String { unimplemented!() }
}

pub fn handle_cloud_query_with_guard(
    query: &str, vault: &EgressVault, detector: &BulkExfiltrationDetector,
) -> Result<String, EgressError>;
```

---

<a id="bench"></a>
## 14. Layer 5: `memfuse-bench`

```rust
pub trait BenchmarkHarness {
    fn run(&self, corpus: &Corpus) -> BenchmarkReport;
}

pub struct BenchmarkReport {
    pub recall_at_k: AHashMap<usize, f32>,
    pub p50_latency_ms: f32,
    pub p99_latency_ms: f32,
}
```

Reproduzierbarkeit: fixierter Seed für jede stochastische Komponente (HNSW-Konstruktion, Bandit-Exploration).

---

<a id="schema"></a>
## 15. FlatBuffers-Schema (vollständig, `schemas/memfuse.fbs`)

```fbs
namespace memfuse.ipc;

table RoleBindingFb {
  role: uint32;
  entity: uint64;
}

table HyperEdgeFb {
  id: uint64;
  predicate_tag: uint32;          // EdgeType-Diskriminante
  participants: [RoleBindingFb];  // min. 2, validiert applikationsseitig
  weight: float32;
  tx_valid_from: uint64;
  tx_valid_to: uint64;            // 0 = None (Sentinel, dokumentiert)
  business_valid_from: int64;
  business_valid_to: int64;       // i64::MIN = None (Sentinel)
  source_doc_id: uint64;          // oder uint128-Encoding bei docid-128 (zwei uint64-Felder)
}

table EdgeFb {
  target: uint64;
  weight: float32;
  edge_type_tag: uint32;
  tx_valid_from: uint64;
  tx_valid_to: uint64;
  business_valid_from: int64;
  business_valid_to: int64;
  source_doc_id: uint64;
}

root_type HyperEdgeFb;
```

**CI-Drift-Gate (`xtask check-flatbuffers-drift`):** vergleicht den Hash des generierten Codes
(`memfuse-core-ipc-gen/src/generated/`) gegen einen im Repository committeten Referenz-Hash; jede
Schema-Änderung ohne begleitende Regenerierung schlägt den Merge-Gate-Job fehl. **Dieses Gate MUSS grün sein,
bevor `HyperEdgeFb` in dieses Schema aufgenommen wird (§6.5, H4)** — als eigener, vorgelagerter CI-Job
(`flatbuffers-drift-gate`), dessen Bestehen `hyperedge-schema-merge` als abhängigen Job voraussetzt
(`needs: [flatbuffers-drift-gate]` in `merge-gate.yml`).

---

<a id="errors"></a>
## 16. Fehlertaxonomie (crateübergreifend)

| Crate | Fehler-Enum | Einbettet |
|---|---|---|
| `memfuse-core` | `CoreError` | — |
| `memfuse-store` | `StoreError` | `WalError`, `LockError`, `CoreError` |
| `memfuse-crypto` | `CryptoError` | — |
| `memfuse-index` | `IndexError` | `CoreError` |
| `memfuse-graph` | `GraphMutationError`, `GraphError` | `LockError` |
| `memfuse-router` | `BanditError` | — |
| `memfuse-candle` | `KvBridgeError` | `CryptoError` |
| `memfuse-mcp` | `SandboxError`, `EgressError` | `wasmtime::Error` |
| `memfuse-db` | `DbError` | alle Layer-1-Fehler per `#[from]` |

**Regel (verbindlich):** Kein öffentlicher Funktionsrückgabetyp ist `Box<dyn std::error::Error>`. Jeder Crate
exportiert genau einen (oder wenige, klar abgegrenzte) `thiserror`-Fehlertyp(en); `memfuse-db` als oberste
Konsumentenschicht bündelt alle Unterfehler verlustfrei per `#[from]`/`#[error(transparent)]`.

---

<a id="tests"></a>
## 17. Test- und CI-Spezifikation

### 17.1 Unit-Tests

Jede öffentliche Funktion mit nicht-trivialer Logik (Fusion, PPR, Bandit-Update, Kanonisierung) erhält
mindestens: einen Normalfall-Test, einen Grenzfall-Test (leere Eingabe, einzelnes Element) und — wo zutreffend —
einen Fehlerfall-Test, der das korrekte `Result::Err`-Enum-Mitglied prüft (kein pauschales `is_err()`).

### 17.2 Integrationstests

- `crates/memfuse-graph/tests/hyperedge_persistence_survives_restart.rs` — AK-1.
- `crates/memfuse-graph/tests/hyperedge_compact_race.rs` — AK-1 (nebenläufig zu `compact()`).
- `crates/memfuse-graph/tests/hyperedge_memory_budget.rs` — AK-2.
- `crates/memfuse-db/tests/signal_kind_no_new_variant.rs` — AK-4.
- `crates/memfuse-graph/tests/hyperedge_cascade_fanout.rs` — AK-6 (synthetischer High-Fan-out-Graph).
- `crates/memfuse-graph/tests/community_hyperedges_included_flag.rs` — AK-7.
- `crates/memfuse-graph/benches/binary_edge_regression.rs` — AK-8 (Baseline-Vergleich vor/nach Hyperkanten-Merge).

### 17.3 Loom-Tests (`#[cfg(loom)]`, `loom`-Feature)

- `crates/memfuse-store/tests/loom_group_commit.rs`
- `crates/memfuse-store/tests/loom_multi_key_lock.rs`
- `crates/memfuse-graph/tests/loom_relate_n_ary.rs` — AK-3.

Alle drei MÜSSEN als eigener CI-Job (`loom-tests`) in `.github/workflows/merge-gate.yml` sichtbar grün laufen —
Testexistenz allein genügt nicht als Abnahme.

### 17.4 `.github/workflows/merge-gate.yml` — Pflicht-Jobs

```yaml
jobs:
  unit-tests:
    run: cargo test --workspace
  flatbuffers-drift-gate:
    run: cargo run -p xtask -- check-flatbuffers-drift
  hyperedge-schema-merge:
    needs: [flatbuffers-drift-gate]
    run: cargo test -p memfuse-graph --features hyperedges -- hyperedge
  check-bandit-latency-budget:
    run: cargo run -p xtask --features memfuse-router/egress-sherman-morrison -- check-bandit-latency-budget
  loom-tests:
    run: RUSTFLAGS="--cfg loom" cargo test --workspace --features loom -- --test-threads=1
  check-unwrap-baseline:
    run: cargo run -p xtask -- check-unwrap-baseline
```

---

<a id="abnahme"></a>
## 18. Vollständige Abnahmekriterien mit Testnamen

| AK | Kriterium | Nachweis (Testpfad) |
|---|---|---|
| AK-1 | `HyperEdge` mit ≥3 `RoleBinding`s persistiert, restart-fest, per `hyperedges_for_entity` auffindbar, konsistent unter gleichzeitigem `compact()` | `hyperedge_persistence_survives_restart.rs`, `hyperedge_compact_race.rs` |
| AK-2 | `estimate_memory_bytes()` inkl. Hyperkanten; `compact_async`-Budget-Check greift | `hyperedge_memory_budget.rs` |
| AK-3 | Zwei gleichzeitige `relate_n_ary` mit überlappenden, unterschiedlich geordneten Mengen deadlockfrei | `loom_relate_n_ary.rs` |
| AK-4 | `SignalKind` strukturell unverändert (kein `Hyperedge`-Signal) | `signal_kind_no_new_variant.rs` |
| AK-5 | FlatBuffers-Drift-Gate grün **vor** `HyperEdgeFb`-Merge | CI-Job-Abhängigkeit `hyperedge-schema-merge: needs: [flatbuffers-drift-gate]` |
| AK-6 | Cascade bricht bei >1.000 Hyperkanten kontrolliert auf Hintergrundverarbeitung um, keine unbeschränkte Latenzspitze | `hyperedge_cascade_fanout.rs` |
| AK-7 | `hyperedges_included` im Report sichtbar `false` ohne Projektion | `community_hyperedges_included_flag.rs` |
| AK-8 | Keine Regression auf binäre `relate()`/`Edge`-Benchmarks | `binary_edge_regression.rs` |

---

*Diese Implementierungsspezifikation ist vollständig und in sich geschlossen. Ein Sprachmodell, das aus ihr
Quellcode generiert, benötigt keine weiteren Architekturentscheidungen — jede Struktur, jede Signatur, jedes
Fehler-Enum, jedes Persistenzformat und jeder Testname ist normativ festgelegt. Wo `unimplemented!()` steht, ist
die Signatur und das umgebende Vertrags-/Fehlerverhalten normativ, der Funktionskörper ist gemäß der in
Prosa/Formel gegebenen Algorithmusbeschreibung des jeweiligen Abschnitts zu füllen.*
