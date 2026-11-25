---
source: CONTEXTRA_SPEC_v4_MASTER.md
chapter: "06a"
---
## 6. Wissensgraph-Datenmodell: binäre Kanten und n-äre Hyperkanten

### 6.1 Binäre Kanten als Grundmodell (🟢)

Der Wissensgraph wird primär als gerichteter, gewichteter Graph in einer CSR-Struktur gehalten:

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
```

`EdgeType` ist als `#[non_exhaustive] enum { Default }` deklariert. Kanten tragen sowohl transaktionale (MVCC)
als auch fachliche (Business-Zeit) Gültigkeit — bi-temporal.

`DocId` (Default `u64`, feature-gated `u128` via BLAKE3-Truncation) und `EntityId` (`u64`) bilden die gemeinsame
Identitätsgrundlage:

```rust
#[cfg(not(feature = "docid-128"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DocId(pub u64);

#[cfg(feature = "docid-128")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(C, align(16))]
pub struct DocId(pub u128);

impl DocId {
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
```

### 6.2 Modulstruktur `contextra-graph`

```
crates/contextra-graph/src/
├── lib.rs
├── csr.rs             # CsrGraph, GraphInner, ArcSwap-RCU
├── edge.rs            # Edge, EdgeType-Nutzung, PersistedEdgePayload
├── hyperedge.rs       # HyperEdge, RoleBinding, RoleId, HyperEdgeId — 🔴
├── ppr.rs             # Forward-Push (Andersen-Chung-Lang)
├── path_rag.rs        # PathGraph, bidirektionale Suche, Hyperkanten-Expansion
├── community.rs       # Leiden, Stern-Expansion-Projektion
├── cascade.rs         # Cascade-Invalidierung binär + Hyperkanten
└── error.rs
```

### 6.3 RCU-Snapshot-Architektur: `csr.rs` (normativ)

**Änderungen Fassung 2.1:** (1) Hyperkanten-Payloads sind `Arc`-geteilt (§6.4): Der Klon in
`InnerWriteGuard::drop` (Ist-Zustand: tiefer Klon samt `Vec<RoleBinding>` jeder Hyperkante) kopiert dann nur
Referenzzähler. (2) Die Speicherschätzung rechnet **capacity-basiert** und liefert zusätzlich die **Spitze** des
Rebuilds. Der Ist-Zustand zählt nur `len()·size_of` und übersieht Tabellen-Overhead und Wachstumsspitze.
(3) Snapshot-Regel S1 (§4(7)): Der Snapshot ist unveränderlich, `scc::HashMap` gehört nicht in `GraphInner`.

**Restkosten (bewusst offen):** Ein Klon von `GraphInner` bleibt O(Anzahl Hyperkanten + Entitäten) an
Referenzzähler-Inkrementen und Tabellen-Allokation pro Veröffentlichung. Abhilfe ist Veröffentlichung pro
Schreib-Batch statt pro Einzelmutation oder eine persistente Map (strukturelles Sharing); beides ist nicht Teil
dieser Fassung.

```rust
use arc_swap::ArcSwap;
use ahash::AHashMap;
use std::mem::size_of;
use std::sync::Arc;

#[derive(Clone)] // sicher: jeder Klon erhöht nur Referenzzähler bzw. kopiert Tabellenstruktur, keine Payloads
pub struct GraphInner {
    pub adjacency: Vec<Vec<Edge>>,
    pub node_index: AHashMap<EntityId, usize>,
    // H1: Teil von GraphInner, NICHT separat — automatisch vom ArcSwap miterfasst.
    pub hyperedges: AHashMap<HyperEdgeId, Arc<HyperEdge>>,       // Payload geteilt (§6.4)
    pub hyperedge_index: AHashMap<EntityId, Arc<[HyperEdgeId]>>, // unveränderlicher Slice je Entität
    pub hyperedge_order: Arc<[HyperEdgeId]>,                     // aufsteigend sortiert (§4(3), §7.5)
}

pub struct MemoryEstimate { pub shared_payload_bytes: usize, pub private_bytes: usize }
impl MemoryEstimate {
    pub fn total(&self) -> usize { self.shared_payload_bytes.saturating_add(self.private_bytes) }
}

/// Obere Schranke für eine hashbrown-Tabelle: Buckets = nextpow2(⌊8·cap/7⌋ + 1),
/// je Bucket size_of::<(K, V)>() + 1 Kontrollbyte, plus 16 Byte Gruppenpuffer.
pub fn table_bytes<K, V>(capacity: usize) -> usize {
    if capacity == 0 { return 0; }
    let buckets = (capacity.saturating_mul(8) / 7 + 1).next_power_of_two().max(4);
    buckets.saturating_mul(size_of::<(K, V)>() + 1).saturating_add(16)
}

impl GraphInner {
    /// `presized = true`: Kapazität = len() (so MUSS ein Rebuild seine Maps und Vektoren anlegen).
    fn estimate(&self, presized: bool) -> MemoryEstimate {
        let cap = |len: usize, capacity: usize| if presized { len } else { capacity };
        let shared_payload_bytes: usize = self.hyperedges.values()
            .map(|h| size_of::<HyperEdge>() + h.participants.len() * size_of::<RoleBinding>()).sum::<usize>()
            + self.hyperedge_index.values().map(|v| v.len() * size_of::<HyperEdgeId>()).sum::<usize>();
        let adjacency_bytes = cap(self.adjacency.len(), self.adjacency.capacity()) * size_of::<Vec<Edge>>()
            + self.adjacency.iter().map(|v| cap(v.len(), v.capacity()) * size_of::<Edge>()).sum::<usize>();
        let private_bytes = adjacency_bytes
            + table_bytes::<EntityId, usize>(cap(self.node_index.len(), self.node_index.capacity()))
            + table_bytes::<HyperEdgeId, Arc<HyperEdge>>(cap(self.hyperedges.len(), self.hyperedges.capacity()))
            + table_bytes::<EntityId, Arc<[HyperEdgeId]>>(cap(self.hyperedge_index.len(), self.hyperedge_index.capacity()))
            + self.hyperedge_order.len() * size_of::<HyperEdgeId>();
        MemoryEstimate { shared_payload_bytes, private_bytes }
    }

    /// Residenz dieses Snapshots (H1: inkl. Hyperkanten, capacity-basiert).
    pub fn estimate_memory_bytes(&self) -> usize { self.estimate(false).total() }

    /// Spitze bei `compact()`: alter Snapshot bleibt bis zum Swap (und für laufende Leser) erreichbar,
    /// der neue wird daneben aufgebaut. Geteilte Payloads zählen einmal.
    pub fn estimate_compaction_peak_bytes(&self) -> usize {
        self.estimate(false).total().saturating_add(self.estimate(true).private_bytes)
    }
}

pub struct CsrGraph {
    inner: ArcSwap<GraphInner>,
    kv_locks: Arc<contextra_store::KvKeyLocks>,
}

impl CsrGraph {
    /// Atomarer Snapshot-Austausch. Leser sehen NIE einen gemischten Alt-/Neu-Zustand.
    /// `rebuild` MUSS Maps mit `with_capacity(len)` vorbelegen (sonst gilt die Spitzenschätzung nicht).
    pub fn compact(&self) -> Result<(), GraphError> {
        let old = self.inner.load();
        let new_inner = Self::rebuild(&old)?;
        self.inner.store(Arc::new(new_inner));
        Ok(())
    }

    pub fn compact_async(&self, max_compaction_peak_memory_mb: usize)
        -> tokio::task::JoinHandle<Result<(), GraphError>>
    {
        // Budgetprüfung gegen die SPITZE, nicht gegen die Residenz (§4(5)).
        let estimated = self.inner.load().estimate_compaction_peak_bytes() / (1024 * 1024);
        if estimated > max_compaction_peak_memory_mb {
            return tokio::spawn(async move {
                Err(GraphError::CompactionBudgetExceeded {
                    budget_mb: max_compaction_peak_memory_mb,
                    estimated_mb: estimated,
                })
            });
        }
        unimplemented!()
    }

    pub fn neighbors_with_weights(&self, id: EntityId) -> Vec<(EntityId, f32)> { unimplemented!() }

    /// Sekundärindex-Zugriff, additiv — keine Kopie: liefert den geteilten Slice.
    pub fn hyperedges_for_entity(&self, id: EntityId) -> Option<Arc<[HyperEdgeId]>> {
        self.inner.load().hyperedge_index.get(&id).cloned()
    }
}
```

**Kosten des Indexes:** Der Slice je Entität ist unveränderlich; ein Einfügen ersetzt ihn durch einen neuen Slice
der Länge n+1 (O(Grad der Entität)). Für Hub-Entitäten mit sehr vielen Hyperkanten ist das der teuerste Teil des
Schreibpfads und in `binary_edge_regression.rs`/`hyperedge_memory_budget.rs` (AK-2, AK-8) mitzumessen.

**⚠️ Opus-Optimierung 2.1 — Inkrementelle Graph-Kompaktierung (Stufe 2, hoch):**
PPR-Pfad löst bei jeder Anfrage vollständigen CSR-Rebuild aus. Ziel: append-only Delta-Segmente plus
periodischer Merge im Hintergrund.

**⚠️ Opus-Optimierung 2.2 — CSR-Sentinel statt `Option` (Stufe 2, mittel):**
Mehrere Kantenspalten als `Vec<Option<T>>` — ca. Halbierung des Speicherbedarfs pro Kante durch Sentinel-Werte.

### 6.4 Hyperkanten: Datenstruktur (🔴 vollständig spezifiziert)

**Designentscheidung (verbindlich):** Es wird **kein** generisches RDF-Reifikations-Pattern verwendet.
Stattdessen wird eine kohärente, erstklassige Rust-Struktur mit Zero-Copy-Deserialisierung via FlatBuffers/Mmap
spezifiziert. **Zwei Repräsentationen, ein Persistenzformat:** Der Schreibpfad (`relate_n_ary`) konstruiert eine
neue Hyperkante ohnehin aus frisch übergebenen Daten — dort ist eine besitzende Struktur korrekt und einfach.
Der Lesepfad (`hyperedges_for_entity`/Traversal) dereferenziert dagegen bei **jeder** Anfrage potenziell
tausende bereits persistierter Hyperkanten aus dem RCU-Snapshot; hier erzwingt eine besitzende `Vec<RoleBinding>`
pro gelesener Hyperkante eine Heap-Kopie, obwohl die Daten bereits deserialisiert im Snapshot-Speicher liegen.
Die Lesesicht referenziert diesen Speicher stattdessen zero-copy:

```rust
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HyperEdgeId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RoleId(pub u32);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoleBinding {
    pub role: RoleId,
    pub entity: EntityId,
}

/// Referenzzählender Slice auf einen zusammenhängenden `RoleBinding`-Bereich innerhalb des
/// Mmap-/RCU-Snapshot-Speichers. Hält nur Pointer + Länge + geteilten Referenzzähler auf den
/// zugrundeliegenden `Arc`-Puffer — kein `Vec`-Allocation-Overhead beim Lesen.
#[derive(Clone)]
pub struct ArcSlice<T> {
    backing: Arc<[T]>,
    start: u32,
    len: u32,
}

impl<T> ArcSlice<T> {
    /// Ganze Sicht auf einen geteilten Puffer — nur Referenzzähler-Inkrement, keine Kopie.
    pub fn whole(backing: Arc<[T]>) -> Self {
        let len = u32::try_from(backing.len()).unwrap_or(u32::MAX);
        Self { backing, start: 0, len }
    }
    /// Validierter Teilbereich; `None` bei ungültigem Bereich (kein Panic, §4(1)).
    pub fn new(backing: Arc<[T]>, start: u32, len: u32) -> Option<Self> {
        let end = (start as usize).checked_add(len as usize)?;
        backing.get(start as usize..end)?;
        Some(Self { backing, start, len })
    }
}

impl<T> std::ops::Deref for ArcSlice<T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        let start = self.start as usize;
        let end = start.saturating_add(self.len as usize);
        self.backing.get(start..end).unwrap_or(&[]) // Konstruktoren validieren; hier defensiv statt Indexierung
    }
}

/// Snapshot-Variante. `participants` ist ein geteilter Puffer (`Arc<[RoleBinding]>`): Klone des Snapshots und
/// Lesesichten teilen ihn, statt ihn zu kopieren (§4(7)). serde benötigt das Feature `rc`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HyperEdge {
    pub id: HyperEdgeId,
    pub predicate: EdgeType,
    pub participants: Arc<[RoleBinding]>,       // min. 2, validiert in `relate_n_ary`; size_of::<RoleBinding>() = 16
    pub weight: f32,
    pub tx_valid_from: Option<TxId>,
    pub tx_valid_to: Option<TxId>,
    pub business_valid_from: Option<i64>,
    pub business_valid_to: Option<i64>,
    pub source_doc_id: Option<DocId>,
}

/// Zero-Copy-Lesesicht — Traversal-Hotpath (`hyperedges_for_entity`, PPR-Expansion, §7.2).
/// `participants` referenziert den RCU-Snapshot-Speicher direkt statt ihn zu kopieren.
#[derive(Clone)]
pub struct HyperEdgeView {
    pub id: HyperEdgeId,
    pub predicate: EdgeType,
    pub participants: ArcSlice<RoleBinding>,
    pub weight: f32,
    pub source_doc_id: Option<DocId>,
}

impl HyperEdge {
    /// Erzeugt die Zero-Copy-Sicht ohne die `participants` zu kopieren — teilt sich den `Arc`
    /// mit dem im `GraphInner` gehaltenen Original (siehe §6.3, H1).
    pub fn as_view(&self) -> HyperEdgeView {
        HyperEdgeView {
            id: self.id,
            predicate: self.predicate,
            // Fassung 2 kopierte hier (`Arc::from(self.participants.as_slice())`) — jetzt Referenzzähler-Inkrement.
            participants: ArcSlice::whole(Arc::clone(&self.participants)),
            weight: self.weight,
            source_doc_id: self.source_doc_id,
        }
    }
}

/// Rollen-Interner — dieselbe Interning-Strategie wie EdgeType/Prädikate.
pub struct RoleInterner {
    forward: scc::HashMap<String, RoleId>,
    backward: scc::HashMap<RoleId, String>,
    next_id: std::sync::atomic::AtomicU32,
}

impl RoleInterner {
    pub fn intern(&self, name: &str) -> RoleId;
    pub fn resolve(&self, id: RoleId) -> Option<String>;
}
```

**Konsequenz für `GraphInner` (§6.3, H1):** `hyperedges: AHashMap<HyperEdgeId, Arc<HyperEdge>>` und
`hyperedge_index: AHashMap<EntityId, Arc<[HyperEdgeId]>>` — das `Arc` auf `HyperEdge` und das `Arc<[RoleBinding]>`
darin machen `as_view()` und den Snapshot-Klon zero-copy-fähig (Test AK-9), da mehrere `HyperEdgeView`s denselben
Teilnehmer-Speicher teilen können, ohne dass ihre Lebensdauer an eine geliehene Referenz auf `GraphInner` gebunden ist
(wichtig, weil `GraphInner` selbst per `ArcSwap` ausgetauscht wird, siehe H1-Lösung unten).

**Persistenz:** Neues LSM-Präfix `__graph:hyperedge:`, Value = FlatBuffers-serialisiertes `HyperEdge`.
Sekundärindex `__graph:hyperedge_by_entity:{EntityId} -> Vec<HyperEdgeId>`. Additiv — kein Breaking Change.

**API-Oberfläche:**
```rust
impl Collection {
    /// Binärer Pfad — bleibt Hotpath, KEINE interne Umleitung auf `relate_n_ary`.
    pub fn relate(&self, from: EntityId, to: EntityId, predicate: EdgeType, doc_id: DocId) -> Result<(), DbError>;

    /// NEU — Hyperkanten-Schreibpfad.
    pub fn relate_n_ary(
        &self,
        predicate: EdgeType,
        participants: &[(RoleId, EntityId)],
        doc_id: DocId,
    ) -> Result<HyperEdgeId, DbError>;
}
```
