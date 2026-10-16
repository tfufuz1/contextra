---
source: CONTEXTRA_SPEC_v4_MASTER.md
chapter: "05"
---
## 5. Speicherschicht: LSM-Tree, WAL und Block-Cache

### 5.1 Grundprinzip und Lock-Hierarchie

Keine Zustandsänderung wird im Speicher sichtbar gemacht, bevor sie physisch in das Write-Ahead-Log geschrieben
und mit dem Datenträger synchronisiert wurde (WAL-First, P2). Der Systemzustand muss sich allein aus dem Log
rekonstruieren lassen (deterministische Recovery, P3). Schreibzugriffe sperren nicht die gesamte Collection, sondern
nur die betroffenen Schlüssel über eine key-granulare Lock-Hierarchie:

```
collections (RwLock) → kv_locks (schlüssel-granular, KvKeyLocks) → embedder (RwLock)
```

Diese Hierarchie ist für **Einzelschlüssel**-Mutationen ausgelegt und deadlockfrei bewiesen. Jede künftige
Mutation, die mehrere Schlüssel gleichzeitig unter `kv_locks` hält, muss diesen Beweis für den Mehrschlüsselfall
gesondert führen — sie darf sich nicht per Analogieschluss auf den Einzelschlüsselfall berufen (konkret
angewendet in §6.5, H2).

### 5.2 Modulstruktur `contextra-store`

```
crates/contextra-store/src/
├── lib.rs
├── lsm.rs               # LSM-Tree, Compaction
├── wal.rs                # Write-Ahead-Log + HMAC-Kette
├── wal_flusher.rs        # begrenzter MPSC-Group-Commit-Actor (§5.3)
├── block_cache/
│   ├── mod.rs            # BlockCacheBackend-Trait
│   ├── lru.rs            # LruBlockCacheBackend (Default)
│   └── sieve.rs          # SieveCacheBackend (Opt-in, `block-cache-v2`)
├── kv_locks.rs           # KvKeyLocks (§5.2a)
└── error.rs
```

### 5.2a Key-granulares Locking: `kv_locks.rs` (normativ)

**Änderung Fassung 2.1:** (a) Die Zuordnung Schlüssel → Shard liegt ausschließlich in `KvKeyLocks` (`key_hash`,
eine Hasher-Instanz mit festen Seeds). Der Aufrufer hasht nie selbst. Fassung 2 hashte in H2 mit
`ahash::RandomState::new().hash_one(e)`; jede `RandomState`-Instanz ist zufällig geseedet (mit ahash 0.8 geprüft:
zwei Aufrufe liefern verschiedene Werte, auch prozessübergreifend), also konnten zwei Threads dieselbe Entität
auf verschiedene Shards abbilden — dann fehlt der gegenseitige Ausschluss. (b) `acquire` liefert `Result`
statt `.unwrap()` (§4(1)). (c) `acquire_multi_sorted` sortiert und dedupliziert selbst; die Eingabe muss nicht
vorsortiert sein.

```rust
use std::hash::{BuildHasher, Hash};
use std::sync::{RwLock, RwLockWriteGuard};

/// Sperrenhierarchie (verbindlich, systemweit einzuhalten):
///   collections (RwLock) → kv_locks (schlüssel-granular) → embedder (RwLock)
pub struct KvKeyLocks {
    shards: Vec<RwLock<()>>,
    shard_mask: u64,
    hasher: ahash::RandomState, // EINE Instanz, feste Seeds (§4(3))
}

pub struct KeyGuard<'a> { _guard: RwLockWriteGuard<'a, ()> }
pub struct MultiKeyGuard<'a> { _guards: Vec<RwLockWriteGuard<'a, ()>> }

#[derive(Debug, thiserror::Error)]
pub enum LockError {
    #[error("lock poisoned")] Poisoned,
    #[error("lock acquisition timed out")] Timeout,
    #[error("shard index out of range")] ShardOutOfRange,
}

impl KvKeyLocks {
    pub fn new(shard_count_pow2: u32) -> Self {
        let n = 1u64 << shard_count_pow2;
        Self {
            shards: (0..n).map(|_| RwLock::new(())).collect(),
            shard_mask: n - 1,
            hasher: ahash::RandomState::with_seeds(
                0x9E37_79B9_7F4A_7C15, 0xBF58_476D_1CE4_E5B9,
                0x94D0_49BB_1331_11EB, 0x2545_F491_4F6C_DD1D,
            ),
        }
    }

    /// Einziger zulässiger Weg, aus einem Schlüssel einen Lock-Hash zu erzeugen.
    pub fn key_hash<T: Hash + ?Sized>(&self, key: &T) -> u64 { self.hasher.hash_one(key) }

    fn shard_for(&self, key_hash: u64) -> usize { (key_hash & self.shard_mask) as usize }

    pub fn acquire(&self, key_hash: u64) -> Result<KeyGuard<'_>, LockError> {
        let shard = self.shards.get(self.shard_for(key_hash)).ok_or(LockError::ShardOutOfRange)?;
        Ok(KeyGuard { _guard: shard.write().map_err(|_| LockError::Poisoned)? })
    }

    /// H2-Pflichtmethode: Erwirbt N Shards STRIKT in aufsteigender Shard-Index-Reihenfolge.
    pub fn acquire_multi_sorted(&self, key_hashes: &[u64]) -> Result<MultiKeyGuard<'_>, LockError> {
        let mut idx: Vec<usize> = key_hashes.iter().map(|h| self.shard_for(*h)).collect();
        idx.sort_unstable();
        idx.dedup();
        let mut guards = Vec::with_capacity(idx.len());
        for i in idx {
            let shard = self.shards.get(i).ok_or(LockError::ShardOutOfRange)?;
            guards.push(shard.write().map_err(|_| LockError::Poisoned)?);
        }
        Ok(MultiKeyGuard { _guards: guards })
    }
}
```

**Testpflicht (AK-14):** `crates/contextra-store/tests/kv_locks_stable_shard.rs` — `key_hash(&x)` ist für dieselbe
Instanz stabil, und zwei nebenläufige `acquire` auf dieselbe Entität schließen sich aus (Regression zu
`RandomState::new()` pro Aufruf).

**Loom-Testpflicht:** `crates/contextra-store/tests/loom_multi_key_lock.rs` MUSS unter `#[cfg(loom)]` zwei
nebenläufige `acquire_multi_sorted`-Aufrufe mit überlappenden, unterschiedlich sortierten Schlüsselmengen
modellieren und deren Terminierung ohne Deadlock nachweisen.

### 5.3 WAL-Pipe: begrenzter MPSC-Group-Commit-Actor, `wal_flusher.rs` (normativ)

Die klassische Implementierung leidet unter geteilter Eigentümerschaft am File-Handle, was zu HMAC-Ketten-Forks
und stillen Datenverlusten führen kann. Zielarchitektur: **genau ein Eigentümer** des File-Handles, von `fsync`
und der Fortschreibung der HMAC-Kette (der Flusher-Task), viele Produzenten, eine **begrenzte** Queue.

**Änderung Fassung 2.1 (ersetzt den SPSC-Ring-Puffer der Fassung 2):** Die WAL hat mehrere Produzenten (jeder
schreibende Thread), ein SPSC-Ring passt dafür nicht. `Box<[MaybeUninit<WalEntry>]>` ist außerdem ohne `unsafe`
nicht lesbar, `contextra-store` ist aber keine Unsafe-Insel (§4(2)). Und `WalEntry.hmac_prev` vom Produzenten zu
setzen, brächte den HMAC-Ketten-Fork zurück, den der Entwurf verhindern soll. Der Ist-Zustand im Repo ist bereits
ein MPSC-Actor mit `oneshot`-Ack nach `sync_all` und Group Commit, aber mit `unbounded_channel` (keine
Backpressure). Zielzustand: gleiche Struktur, begrenzte Queue.

**Vertrag (P2, Durability):** `append` kehrt erst **nach** dem `fsync` zurück, der den Eintrag enthält. „Entkoppelt"
ist nur die Wartezeit auf die Queue-Kapazität; die Latenz eines Commits umfasst immer mindestens einen `fsync`
(Group Commit amortisiert ihn über alle wartenden Einträge).

```rust
use bytes::Bytes;
use tokio::sync::{mpsc, oneshot};

pub const DEFAULT_WAL_QUEUE_CAPACITY: usize = 1_024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WalSeq(pub u64);

pub struct WalAppend {
    pub payload: Bytes, // OHNE hmac_prev: Sequenz und Kettenglied vergibt ausschließlich der Flusher
    pub ack: oneshot::Sender<Result<WalSeq, WalError>>,
}

pub enum WalCommand {
    Append(WalAppend),
    Seal { ack: oneshot::Sender<Result<(), WalError>> },
}

#[derive(Clone)]
pub struct WalHandle { tx: mpsc::Sender<WalCommand> }

pub struct WalFlusherConfig {
    pub queue_capacity: usize, // > 0, Default DEFAULT_WAL_QUEUE_CAPACITY
    pub max_batch_entries: usize,
    pub batch_window: std::time::Duration,
}

impl WalHandle {
    pub fn channel(queue_capacity: usize) -> Result<(WalHandle, mpsc::Receiver<WalCommand>), WalError> {
        if queue_capacity == 0 { return Err(WalError::InvalidCapacity); }
        let (tx, rx) = mpsc::channel(queue_capacity);
        Ok((WalHandle { tx }, rx))
    }

    /// Wartet bei voller Queue (Backpressure) und kehrt erst NACH fsync zurück.
    pub async fn append(&self, payload: Bytes) -> Result<WalSeq, WalError> {
        let (ack, rx) = oneshot::channel();
        self.tx.send(WalCommand::Append(WalAppend { payload, ack }))
            .await.map_err(|_| WalError::FlusherClosed)?;
        rx.await.map_err(|_| WalError::FlusherClosed)?
    }

    /// Nicht wartend: volle Queue ⇒ `Backpressure`.
    pub fn try_append(&self, payload: Bytes)
        -> Result<oneshot::Receiver<Result<WalSeq, WalError>>, WalError>
    {
        let (ack, rx) = oneshot::channel();
        match self.tx.try_send(WalCommand::Append(WalAppend { payload, ack })) {
            Ok(()) => Ok(rx),
            Err(mpsc::error::TrySendError::Full(_)) => Err(WalError::Backpressure),
            Err(mpsc::error::TrySendError::Closed(_)) => Err(WalError::FlusherClosed),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum WalError {
    #[error("wal queue full (backpressure)")] Backpressure,
    #[error("wal flusher closed")] FlusherClosed,
    #[error("wal queue capacity must be > 0")] InvalidCapacity,
    #[error("hmac chain fork detected at sequence {0}")] HmacChainFork(u64),
}
```

**Flusher-Schleife (Körper gemäß Prosa):** Auf den ersten `recv().await` folgt ein Drain per `try_recv` bis
`max_batch_entries` oder Ablauf von `batch_window`; alle Einträge werden geschrieben, die Kette im Flusher
fortgeschrieben, **ein** `sync_data`/`sync_all` ausgeführt, danach werden alle `ack`s beantwortet. Ein Fehler
beantwortet alle Acks des Batches mit `Err`. Der Flusher ist der einzige Aufrufer von `fsync`.

**Deadlock-Regel (§4(4)):** Produzenten halten beim `send().await` keinen Lock, den der Flusher benötigt. Die
Kettenzustands-Mutex des Ist-Zustands (`last_hmac`, produzentenseitig) entfällt im Zielzustand, weil die Kette im
Flusher fortgeschrieben wird; bis dahin gilt: der Flusher greift nie auf diese Mutex zu.

**Testpflicht (AK-13):** `crates/contextra-store/tests/wal_backpressure.rs` — bei voller Queue liefert `try_append`
`Backpressure`, `append` wartet; ein bestätigter `append` ist nach simuliertem Absturz (Fault-VFS) wiederherstellbar;
ein unbestätigter darf fehlen. `loom_group_commit.rs` (§15.3) bleibt bestehen.

**⚠️ Opus-Optimierung 0.1 — WAL-Replay-Panic entschärfen (Stufe 0, gering):**
Die Replay-Routine liest die Dateigröße einmalig vor dem `mmap`, prüft Zugriffsgrenzen aber gegen diesen separat
gehaltenen Wert statt gegen die tatsächliche Länge der gemappten Region. Maßnahme: Dateigröße ausschließlich aus
`mmap.len()` ableiten, alle Slice-Zugriffe auf `mmap.get(a..b)` mit `.ok_or(WalCorruption)` umstellen. Die zweite
parallele Scan-Implementierung auf denselben Hilfsfunktions-Pfad reduzieren.

**⚠️ Opus-Optimierung 0.5 — Recovery-Pfad differenzieren (Stufe 0, mittel):**
Den Intent-Datensatz um einen expliziten Ergebnisstatus (committed/aborted) erweitern und bei Repair-on-Open
auswerten, statt pauschal vorwärts zu committen.

### 5.4 Block-Cache: `BlockCacheBackend`-Trait (normativ)

```rust
pub trait BlockCacheBackend<K, V>: Send + Sync {
    fn get(&self, key: &K) -> Option<V>;
    fn insert(&self, key: K, value: V);
    fn capacity(&self) -> usize;
    fn len(&self) -> usize;
}
```

**`block_cache/lru.rs` (🟢 Produktions-Default):**

```rust
pub struct LruBlockCacheBackend<K, V> {
    inner: RwLock<lru::LruCache<K, V>>,
}

impl<K: Hash + Eq + Clone, V: Clone + Send + Sync> BlockCacheBackend<K, V>
    for LruBlockCacheBackend<K, V>
{
    fn get(&self, key: &K) -> Option<V> {
        // Cache-Hit erfordert Write-Lock, da LRU-Reordering mutiert (P25-Verstoß, dokumentiert als
        // bewusster Trade-off des Default-Pfads — siehe SieveCacheBackend für den lock-armen Pfad).
        // Poison-tolerant (§4(1)): ein Cache enthält nur rekonstruierbare Daten.
        self.inner.write().unwrap_or_else(std::sync::PoisonError::into_inner).get(key).cloned()
    }
    fn insert(&self, key: K, value: V) {
        self.inner.write().unwrap_or_else(std::sync::PoisonError::into_inner).put(key, value);
    }
    fn capacity(&self) -> usize {
        self.inner.read().unwrap_or_else(std::sync::PoisonError::into_inner).cap().get()
    }
    fn len(&self) -> usize {
        self.inner.read().unwrap_or_else(std::sync::PoisonError::into_inner).len()
    }
}
```

**`block_cache/quick_cache.rs` (🟡 Opt-in `block-cache-v2`, produktiv im Code vorhanden):**

Der tatsächlich implementierte lock-günstige Pfad wrappt den `quick_cache`-Crate (S3-FIFO-artige Eviction über
drei FIFO-Warteschlangen: Small ≈ 10 % Kapazität mit „Quick Demotion" für One-Hit-Wonders, Main, Ghost) als
`QuickCacheBlockCacheBackend`. Dies ist der Stand, der durch `FINAL_12` §0.3 am Code verifiziert ist — `LRU`
bleibt der Produktions-Default, `block-cache-v2` schaltet auf `QuickCacheBlockCacheBackend` um.

```rust
use quick_cache::sync::Cache as QuickCache;

pub struct QuickCacheBlockCacheBackend<K, V> {
    inner: QuickCache<K, V>,
}

impl<K, V> BlockCacheBackend<K, V> for QuickCacheBlockCacheBackend<K, V>
where
    K: std::hash::Hash + Eq + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    fn get(&self, key: &K) -> Option<V> { self.inner.get(key) }
    fn insert(&self, key: K, value: V) { self.inner.insert(key, value); }
    fn capacity(&self) -> usize { self.inner.capacity() as usize }
    fn len(&self) -> usize { self.inner.len() }
}
```

**`block_cache/sieve.rs` (🔴 Zielarchitektur, löst `QuickCacheBlockCacheBackend` perspektivisch ab):**

SIEVE verzichtet auf Listen-Neuordnung bei Lesetreffern: Ein Treffer setzt nur ein `visited`-Bit. Eviction läuft
über einen umlaufenden Zeiger („Hand"): gesetztes Bit ⇒ begnadigt (Bit gelöscht, bleibt im Cache), gelöschtes
Bit ⇒ verdrängt.

**Verbindliche Einordnung:** `quick_cache` bleibt die deklarierte Abhängigkeit und `QuickCacheBlockCacheBackend`
der Inhalt von `block-cache-v2`, bis `SieveCacheBackend` denselben Trait implementiert, denselben Loom- und
Benchmark-Nachweis wie `QuickCacheBlockCacheBackend` erbringt und per ADR als Ablösung beschlossen wird. Bis
dahin ist `SieveCacheBackend` ein Zielentwurf unter `#[cfg(feature = "block-cache-sieve-experimental")]`.

**Änderung Fassung 2.1 (ersetzt die `crossbeam-epoch`-Skizze):** Eine Liste aus `Atomic<SieveNode>` braucht
`unsafe` (`Shared::deref`), `contextra-store` ist aber keine Unsafe-Insel (§4(2)); `Shared<'static, _>` ist zudem
`!Send` und `!Sync` (mit `crossbeam-epoch` 0.9 geprüft), der Trait verlangt `Send + Sync`. Sicherer Entwurf: Der
Index ist `scc::HashMap<K, Arc<SieveNode<V>>>`, `visited` liegt im Node, die Reihenfolge-Struktur (Slab mit
Index-Verkettung und `hand`) liegt unter einem `Mutex` und wird **nur im Miss-/Evict-Pfad** berührt. Ein Treffer
nimmt keinen Mutex. Er ist damit lock-arm, nicht wait-free (ein Bucket-Lesezugriff der `scc::HashMap`).
Byte-Kapazität: `size_bytes` im Node, `used_bytes` als `AtomicUsize`, Eviction bis `used_bytes ≤ capacity_bytes`.
Der Trait bleibt unverändert; `QuickCacheBlockCacheBackend` gewichtet im Repo bereits per `Weighter`.

```rust
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

pub struct SieveNode<V> {
    pub value: V,
    pub size_bytes: usize,
    visited: AtomicBool,
}

/// Index-verkettete Liste in einem Vec-Slab (safe Rust) plus `hand`; nur unter dem Mutex berührt.
struct SieveOrder<K> { slots: Vec<Option<K>>, hand: usize }

pub struct SieveCacheBackend<K, V> {
    index: scc::HashMap<K, Arc<SieveNode<V>>>,
    order: Mutex<SieveOrder<K>>,
    capacity_bytes: usize,
    used_bytes: AtomicUsize,
}

impl<K, V> BlockCacheBackend<K, V> for SieveCacheBackend<K, V>
where
    K: std::hash::Hash + Eq + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    fn get(&self, key: &K) -> Option<V> {
        // Hit-Pfad: kein Mutex, keine Listenmutation, ein Relaxed-Store.
        self.index.read(key, |_, node| {
            node.visited.store(true, Ordering::Relaxed);
            node.value.clone()
        })
    }
    fn insert(&self, key: K, value: V) { unimplemented!() } // Miss-Pfad: Mutex, Eviction bis used_bytes ≤ capacity_bytes
    fn capacity(&self) -> usize { self.capacity_bytes }
    fn len(&self) -> usize { self.index.len() }
}
```

**Nachweis vor Aktivierung:** Konformitätstest gegen einen Referenz-Simulator des SIEVE-Algorithmus (gleiche
Trefferfolge bei gleicher Zugriffsfolge), Loom-Test des Miss-Pfads, Benchmark gegen `QuickCacheBlockCacheBackend`.

**Sharding:** Alle Backends werden über ein `ShardedBlockCache<K, V, B: BlockCacheBackend<K,V>>` mit
konfigurierbarer Shard-Zahl (Default 16, `ahash`-basiertes Routing) gekapselt.

**⚠️ Opus-Optimierung 1.7 — Byte-basierte Cache-Kapazität (Stufe 1, mittel):**
Kapazität byte-basiert statt eintragsbasiert führen (Eviction anhand der tatsächlichen Bytegröße) — gilt für
`QuickCacheBlockCacheBackend` und die perspektivische `SieveCacheBackend` gleichermaßen.

### 5.5 Öffentliche Storage-API

```rust
pub struct LsmStore {
    wal: WalHandle,
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

**⚠️ Opus-Optimierung 1.4 — SSTable Zero-Copy-Slice (Stufe 1, trivial):**
Beim Lesen eines Datenblocks nach CRC-Prüfung: referenzzählendes Slicing statt vollständiger Kopie.

**⚠️ Opus-Optimierung 2.4 — Manifest-Batch-Fsync (Stufe 2, mittel):**
Zusammengehörige Manifest-Änderungen in einem Batch, ein `fsync` pro Zustandsübergang statt pro Einzeleintrag.

**⚠️ Opus-Optimierung 1.6 — MemTable Range-Sharding (Stufe 1, hoch):**
Sharding-Grenzen aus dem Namensraum-Präfix ableiten, sodass Flush sortierfrei und Präfix-Scan auf eine Partition
beschränkt wird.

### 5.6 Compaction/MANIFEST-Atomarität (⚠️ Opus-Optimierung 0.6, Stufe 0, gering — STO-A)

**Einordnung:** Dies ist einer der fünf systemweit schwerwiegendsten Befunde des Architektur-Reviews, auf
derselben Prioritätsstufe wie der WAL-Replay-Panic-Fix (§5.3, Opus 0.1), und gehört ebenso in Stufe 0.

**Problem:** `maybe_compact()` schreibt Manifest-Änderungen nicht als einen atomaren Übergang, sondern als
Sequenz: (1) `manifest.append(Add { output_path })`, (2) In-Memory-Swap, (3) `manifest.append(Remove { old })` +
Löschen der alten Dateien — Fehler in (3) werden nur geloggt, nicht propagiert. Ein Absturz oder ein regulärer
Abbruch zwischen (1) und (3) hinterlässt ein MANIFEST, das die gemergte **und** alle Input-SSTables als gültig
führt. Beim Recovery werden beide geladen; bei einer vollständigen Kompaktierung verwirft der Merge Tombstones —
die alten SSTables enthalten die gelöschten Versionen jedoch noch. **Gelöschte Daten können nach einem Absturz
zurückkehren.** Für ein System mit kryptographischer `DeletionProof`-Zusicherung (Art. 17 DSGVO / Machine
Unlearning) ist das keine reine Storage-Performance-Frage, sondern ein Bruch des Sicherheitsversprechens aus §10.

Zusätzlich: Ein beschädigtes oder nicht ladbares MANIFEST degradiert aktuell still auf „lade jede `.sst`-Datei im
Verzeichnis" — ein Verstoß gegen die projektweite „No Silent Failures"-Doktrin (P4/P6) — und die Shadowing-
Reihenfolge zwischen Merge-Ausgabe und ihren Inputs wird nicht persistiert, sondern nach Recovery lexikografisch
neu geraten.

**Lösung (verbindlich):**

```rust
/// Ein Zustandsübergang = ein Record, ein fsync. Ersetzt die bisherige Add/Remove-Paar-Sequenz.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestEntry {
    pub kind: ManifestEntryKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ManifestEntryKind {
    /// Atomarer Kompaktierungs-Übergang: entfernte Dateien, neue Datei, Position (löst die
    /// Shadowing-Reihenfolge-Frage — die Position gehört in den Record, nicht in die Recovery-Heuristik).
    Replace { removed: Vec<PathBuf>, added: PathBuf, position: u32 },
    Add { path: PathBuf },
}

impl Manifest {
    /// Geschrieben und ge-fsynct NACH erfolgreichem Merge, VOR dem In-Memory-Swap.
    /// Ein halb geschriebener Record fällt über die CRC-Prüfung heraus — der Vorzustand gilt dann als aktuell.
    pub fn commit_replace(&self, removed: Vec<PathBuf>, added: PathBuf, position: u32)
        -> Result<(), ManifestError>;

    /// MUSS `Err` propagieren — kein Fallback auf Verzeichnis-Scan bei Ladefehler.
    pub fn load(path: &Path) -> Result<Vec<ManifestEntry>, ManifestError>;

    /// Rollover per `MANIFEST.new` + atomarem `rename` statt unbegrenztem Wachstum der Historie —
    /// Startzeit wird proportional zu den aktiven SSTables statt zur vollständigen Historie.
    pub fn rollover(&self) -> Result<(), ManifestError>;
}

#[derive(Debug, thiserror::Error)]
pub enum ManifestError {
    #[error("manifest record failed CRC check at offset {0}, previous state retained")]
    CorruptRecord(u64),
    #[error(transparent)] Io(#[from] std::io::Error),
}
```

Kandidatenauswahl für Compaction wird zusätzlich in **einem** Lock-Fenster gelesen und verwendet (nicht über
drei getrennte Fenster hinweg), um den zugehörigen Out-of-Bounds-Panic-Pfad bei nebenläufiger Compaction
auszuschließen.

**Testpflicht:** `crates/contextra-store/tests/manifest_crash_no_resurrection.rs` — simuliert einen Absturz nach
Schritt (1) im alten Modell (bzw. nach dem `commit_replace`-`fsync` im neuen Modell) und belegt, dass nach
Recovery keine Tombstone-Version aus den alten SSTables sichtbar wird.

---

<a id="6-graph"></a>
