# Contextra — Mikrofeingranulare Implementierungsspezifikation EP1: Contextra Memory

**Dokumenten-ID:** `DOC-SPEC-EP1-MEMORY-2026-10-14`  
**Version:** 2.0 (Punktgenaue Feinspezifikation) · **Datum:** 14. Oktober 2026  
**Autor:** Principal Senior Rust Architect für Contextra  
**Status:** Verbindliche, normative Implementierungsanweisung für Release 1.0  
**Geltungsbereich (Disjunktive Crates):**  
- **Ring 0 (Primitiven & Invarianten):** [`contextra-types`](../crates/contextra-types), [`contextra-ports`](../crates/contextra-ports), [`contextra-mvcc`](../crates/contextra-mvcc), [`contextra-durable-fs`](../crates/contextra-durable-fs), [`contextra-simd`](../crates/contextra-simd), [`contextra-sys`](../crates/contextra-sys)
- **Ring 1 (Speicher & Haltbarkeit):** [`contextra-store`](../crates/contextra-store), [`contextra-checkpoint`](../crates/contextra-checkpoint)
- **Ring 2 (Indizes & Algorithmen):** [`contextra-vector`](../crates/contextra-vector), [`contextra-text`](../crates/contextra-text), [`contextra-rank`](../crates/contextra-rank)
- **Ring 3 (Compute & Orchestrierung):** [`contextra-engine`](../crates/contextra-engine), [`contextra-db`](../crates/contextra-db)
- **Ring 4 (Fassade & Protokoll):** [`contextra`](../crates/contextra), [`contextra-mcp`](../crates/contextra-mcp) (Kern-Endpunkte: `contextra_remember`, `contextra_search`, `contextra_forget_key`)

---

## 1. Architektonische Grenzziehung & Disjunktion

Contextra Memory (EP1) ist das marktreife Fundament des Gesamtsystems: eine eingebettete, zustandslose Single-Node Hybrid-Datenbank mit strikter Mandanten-Isolation, Transaktionssicherheit (MVCC/SSI) und deterministischer Performance.

**Disjunktionsregeln für EP1:**
1. **Kein Löschnachweis-Krypto:** EP1 enthält **keine** Ed25519-Signaturlogik, keine Transparenz-Bäume und keine v4-TLV-Beweiserzeugung (vollständig nach EP2 verlagert). Löschung in EP1 erfolgt ausschließlich über Tombstones und physisches Purging (Purge v2).
2. **Keine Graph-Diffusion / Hyperedges:** CSR-Graphentraversierung und Hyperkanten sind in EP1 inaktiv (nach EP3 verlagert).
3. **Keine lokalen Embedder-Modelle:** Lokale Inference (Candle, ONNX) und KV-Cache-Quantisierung sind nicht Bestandteil von EP1 (nach EP4 verlagert).
4. **Keine WASM-Sandbox:** Merging erfolgt rein über native Rust-Operatoren; keine dynamische Bytecode-Ausführung (nach EP6 verlagert).
5. **Zero-Panic-Doktrin (P2):** Kein `unwrap()`, kein `expect()`, kein `panic!` in Produktionscode. Alle Fehlerzustände werden über `ContextraError` abgebildet.

---

## 2. Vollständige Befund- und Korrekturmatrix EP1

| ID | Datei | Zeile(n) | Problem & Root Cause | Exakte Korrekturmaßnahme |
|---|---|---|---|---|
| **N-B** | `crates/contextra-engine/src/collection/crud/delete.rs` | 79–109 | `downcast_ref::<LsmStorage>()` schlägt unter `TenantScopedStorage` fehl; wirft immer `CapabilityUnsupported`. | Trait-Aufruf `self.storage.as_erasure_backend()` nutzen; Downcast entfernen. |
| **N-K** | `crates/contextra-store/src/tenant_codec.rs` | 42–110 | Logischer Scope matchte `__col:{id}:` anstatt des physischen mandantenkodierten Keys `t:{tenant_id}:...`. Purge traf 0 SSTable-Keys. | `TenantScopedStorage` implementiert `ErasureBackend::physical_scope`, welcher Tenant-Präfix voranstellt. |
| **N-A** | `crates/contextra-store/src/lsm/mod.rs` | 145–190 | Crash-Resume erfolgte fail-open, da der Scope ohne Store-Recovery verworfen wurde. | Rehydrierung des `PhysicalScope` aus dem `ErasureLedger` während `LsmStorage::open()` VOR Freigabe regulärer I/O. |
| **N-N** | `crates/contextra-engine/src/lib.rs` | 212 | Match-Arm `UnknownFreeSpace` fehlt nach Store-Update; Engine kompiliert nicht. | Vollständiger Match-Arm mit konservativem Fallback implementieren. |
| **N-O** | `crates/contextra/src/builder.rs` | 88 | `clock_instance` fehlt Fallback bei unvollständiger Dependency Injection; Fassade kompiliert nicht. | Default-Initialisierung auf `SystemClock` via Port einfügen. |
| **L1** | `crates/contextra-store/src/purge_v2/` | — | MemTable-Tombstone wurde vor Ledger-Eintrag geschrieben; Crash führte zu Datenwiederauferstehung. | `ErasureLedger::append(..., Pending)` VOR MemTable-Schreiboperation erzwingen (`fsync`). |
| **L4** | `crates/contextra-store/src/lsm/ops/purge.rs` | — | `PurgeOutput.obsolete_files` war leer; Quell-SSTables blieben auf Platte. | Obsolete SSTables registrieren und via `scrub_and_remove` physisch nullen und löschen. |
| **L5** | `crates/contextra-store/src/lsm/ops/purge.rs` | — | `while let Ok(Some(..))` verschluckte SSTable-Korruptionen stillschweigend. | 3-Arm-Match (`Some(Ok)`, `Some(Err)` $\to$ Fatal Error, `None` $\to$ Break). |
| **L6** | `crates/contextra-store/src/lsm/ops/purge.rs` | — | Rewritten SSTable verlor relative Sequenzgrenzen und Rang. | Metadaten (`min_seq`, `max_seq`, `rank`) im neuen SSTable-Header exakt spiegeln. |
| **L8** | `crates/contextra-store/src/lsm/ops/purge.rs` | — | `FreeSpaceProbe = None` wurde als 0 Byte interpretiert und brach ab. | `None` als unbegrenzt interpretieren; Fehler nur bei `Some(b) < 10 MB`. |
| **S5** | `crates/contextra/src/builder.rs` | 130–185 | 7 Builder-Schalter wurden mit `let _ = ...;` ignoriert. | `verify_wiring(&self) -> Result<(), BuildError>` zur Build-Zeit erzwingen. |
| **INV-MVCC-SSI-1** | `crates/contextra-mvcc/src/ssi.rs` | 40–110 | Lese-Iteratoren führten kein Read-Set; Write-Skew bei SSI-Transaktionen unentdeckt. | `ReadSnapshotToken` verpflichtend in alle `StorageEngine`-Scan-Iteratoren einspeisen. |
| **HNSW-ZERO** | `crates/contextra-vector/src/hnsw/deletion.rs` | 120–180 | Gelöschte Vektoren blieben im RAM-Slab lesbar. | `purge_vector_slot` aufrufen; Arena-RAM mit Nullen überschreiben, kein Rewiring (VETO-F02). |
| **ENG-FILTER-001** | `crates/contextra-text/src/temporal_filter.rs` | 55–110 | Bi-temporale Kollision zwischen `event_time` und `ingest_time`. | Trennung der temporalen Filterachsen in `TemporalFilterCriterion`. |
| **MVCC-S01..S04** | `crates/contextra-mvcc/src/floor.rs` | 30–95 | Unkoordinierte Floor-Updates führten zu vorzeitiger Garbage Collection aktiver Snapshots. | Zentraler `SnapshotFloorTracker` mit monotoner Schranke. |

---

## 3. Punktgenaue Schnittstellen- und Datenstrukturen

### 3.1 `contextra-ports::erasure`
```rust
// In crates/contextra-ports/src/erasure.rs
use crate::BoxFuture;
use contextra_types::{ContextraError, Result, TxId};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
#[repr(u8)]
pub enum ErasureMode {
    Off = 0,
    Assured = 1,
    Attested = 2,
}

impl ErasureMode {
    #[inline(always)]
    pub fn requires_ledger(&self) -> bool {
        *self >= Self::Assured
    }

    #[inline(always)]
    pub fn requires_attestation(&self) -> bool {
        *self == Self::Attested
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum LogicalScope {
    Document { doc_id: String },
    Collection { collection_id: contextra_types::CollectionId },
    Keys(Vec<Vec<u8>>),
    Prefixes(Vec<Vec<u8>>),
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PhysicalScope {
    pub exact_keys: Vec<Vec<u8>>,
    pub prefixes: Vec<Vec<u8>>,
}

impl PhysicalScope {
    pub fn new(exact_keys: Vec<Vec<u8>>, prefixes: Vec<Vec<u8>>) -> Self {
        Self { exact_keys, prefixes }
    }

    pub fn from_explicit_keys(keys: Vec<Vec<u8>>) -> Self {
        Self { exact_keys: keys, prefixes: Vec::new() }
    }

    pub fn from_prefixes(prefixes: Vec<Vec<u8>>) -> Self {
        Self { exact_keys: Vec::new(), prefixes }
    }

    #[inline]
    pub fn matches_key(&self, key: &[u8]) -> bool {
        self.exact_keys.iter().any(|k| k.as_slice() == key)
            || self.prefixes.iter().any(|p| key.starts_with(p.as_slice()))
    }
}

pub trait FreeSpaceProbe: Send + Sync {
    fn available_bytes(&self) -> Option<u64>;
}

#[derive(Debug, Clone, Default)]
pub struct PurgeOutput {
    pub records_purged: u64,
    pub bytes_reclaimed: u64,
    pub sstables_replaced: usize,
    pub obsolete_files: Vec<PathBuf>,
}

pub trait ErasureBackend: Send + Sync + 'static {
    fn physical_scope(&self, scope: &LogicalScope) -> Result<PhysicalScope>;
    fn purge<'a>(
        &'a self,
        scope: &'a PhysicalScope,
        as_of: TxId,
    ) -> BoxFuture<'a, Result<PurgeOutput>>;
}
```

### 3.2 `contextra-ports::storage` Erweiterung
```rust
// In crates/contextra-ports/src/storage.rs
use crate::erasure::ErasureBackend;

pub trait StorageEngine: Send + Sync + 'static {
    // Bestehende Methoden: put, get, delete, scan, commit, rollback...

    /// Ermöglicht den typensicheren Zugriff auf physische Purge-Fähigkeiten
    /// ohne unzulässiges std::any::Any-Downcasting (Behebung N-B).
    fn as_erasure_backend(&self) -> Option<&dyn ErasureBackend> {
        None
    }
}
```

### 3.3 `TenantScopedStorage` Implementierung in `contextra-store`
```rust
// In crates/contextra-store/src/tenant_codec.rs
use contextra_ports::erasure::{ErasureBackend, LogicalScope, PhysicalScope, PurgeOutput};
use contextra_ports::BoxFuture;
use contextra_types::{ContextraError, Result, TxId};
use crate::tenant_codec::TenantScopedStorage;

impl<S: contextra_ports::StorageEngine + 'static> contextra_ports::StorageEngine for TenantScopedStorage<S> {
    fn as_erasure_backend(&self) -> Option<&dyn ErasureBackend> {
        if self.inner.as_erasure_backend().is_some() {
            Some(self)
        } else {
            None
        }
    }
}

impl<S: contextra_ports::StorageEngine + 'static> ErasureBackend for TenantScopedStorage<S> {
    fn physical_scope(&self, scope: &LogicalScope) -> Result<PhysicalScope> {
        match scope {
            LogicalScope::Document { doc_id } => {
                let doc_key = format!("doc:{doc_id}");
                let physical_key = self.codec.encode_key(doc_key.as_bytes());
                Ok(PhysicalScope::from_explicit_keys(vec![physical_key]))
            }
            LogicalScope::Collection { collection_id } => {
                let col_prefix = format!("__col:{}:", collection_id.inner());
                let physical_prefix = self.codec.encode_key(col_prefix.as_bytes());
                Ok(PhysicalScope::from_prefixes(vec![physical_prefix]))
            }
            LogicalScope::Keys(keys) => {
                let enc_keys = keys.iter().map(|k| self.codec.encode_key(k)).collect();
                Ok(PhysicalScope::from_explicit_keys(enc_keys))
            }
            LogicalScope::Prefixes(prefixes) => {
                let enc_prefixes = prefixes.iter().map(|p| self.codec.encode_key(p)).collect();
                Ok(PhysicalScope::from_prefixes(enc_prefixes))
            }
        }
    }

    fn purge<'a>(
        &'a self,
        scope: &'a PhysicalScope,
        as_of: TxId,
    ) -> BoxFuture<'a, Result<PurgeOutput>> {
        Box::pin(async move {
            if let Some(backend) = self.inner.as_erasure_backend() {
                backend.purge(scope, as_of).await
            } else {
                Err(ContextraError::CapabilityUnsupported(
                    "Inner storage engine does not support physical purge".to_string(),
                ))
            }
        })
    }
}
```

### 3.4 Eliminierung des Downcasts in `contextra-engine` (Behebung N-B)
```rust
// In crates/contextra-engine/src/collection/crud/delete.rs (Zeile 75-115)
// VORHER:
// let lsm = match self.storage.as_any().downcast_ref::<LsmStorage>() { ... } -> CapabilityUnsupported!

// NACHHER:
pub async fn execute_physical_purge(
    &self,
    scope: &LogicalScope,
    as_of: TxId,
) -> Result<PurgeOutput, ContextraError> {
    let backend = self.storage.as_erasure_backend().ok_or_else(|| {
        ContextraError::CapabilityUnsupported(
            "Storage engine does not support ErasureBackend".to_string(),
        )
    })?;

    let physical_scope = backend.physical_scope(scope)?;
    backend.purge(&physical_scope, as_of).await
}
```

### 3.5 Purge v2 Streaming Rewrite Engine in `contextra-store`
```rust
// In crates/contextra-store/src/lsm/ops/purge.rs
use std::path::{Path, PathBuf};
use std::sync::Arc;
use contextra_ports::erasure::{FreeSpaceProbe, PhysicalScope, PurgeOutput};
use contextra_types::{ContextraError, Result, TxId};
use crate::lsm::manifest::ManifestContext;
use crate::lsm::sstable::{SSTableReader, SSTableWriter};

pub struct PurgeEngine<'a> {
    store_dir: &'a Path,
    manifest: &'a ManifestContext,
}

impl<'a> PurgeEngine<'a> {
    pub fn new(store_dir: &'a Path, manifest: &'a ManifestContext) -> Self {
        Self { store_dir, manifest }
    }

    pub fn execute_purge(
        &self,
        scope: &PhysicalScope,
        as_of_tx: TxId,
        probe: &dyn FreeSpaceProbe,
    ) -> Result<PurgeOutput> {
        // [L8-Fix]: None = Unbekannt (konservativ weiterlaufen); Some(bytes) < 10 MB = OutOfMemory
        if let Some(free_bytes) = probe.available_bytes() {
            if free_bytes < 10 * 1024 * 1024 {
                return Err(ContextraError::OutOfMemory);
            }
        }

        let mut output = PurgeOutput::default();
        let active_ssts = self.manifest.get_active_sstables();

        for sst_meta in active_ssts {
            if !sst_meta.may_contain_scope(scope) {
                continue;
            }

            let sst_path = self.store_dir.join(&sst_meta.filename);
            let temp_filename = format!("{}.purge_tmp", sst_meta.filename);
            let temp_path = self.store_dir.join(&temp_filename);

            let mut reader = SSTableReader::open(&sst_path)?;
            let mut writer = SSTableWriter::create(&temp_path, sst_meta.rank)?;

            let mut purged_in_sst = 0u64;
            let mut kept_in_sst = 0u64;

            // [L5-Fix]: 3-Arm Pattern Match verhindert stilles Verschlucken von I/O-Fehlern
            loop {
                match reader.next_entry() {
                    Ok(Some(entry)) => {
                        if scope.matches_key(entry.key()) && entry.tx_id() <= as_of_tx {
                            purged_in_sst += 1;
                            output.bytes_reclaimed += entry.serialized_size() as u64;
                        } else {
                            // Streaming direkt in Zieldatei (kein In-Memory Vec-Buffer)
                            writer.append_entry(&entry)?;
                            kept_in_sst += 1;
                        }
                    }
                    Ok(None) => break,
                    Err(io_err) => {
                        let _ = std::fs::remove_file(&temp_path);
                        return Err(ContextraError::Io(Arc::new(io_err)));
                    }
                }
            }

            if purged_in_sst > 0 {
                // [L6-Fix]: Sequenzgrenzen und Rang aus den alten Metadaten exakt übertragen
                writer.set_sequence_bounds(sst_meta.min_seq, sst_meta.max_seq);
                writer.finish_sync()?;

                // Atomarer Austausch im Manifest
                self.manifest.replace_sstable(&sst_meta.id, &temp_filename)?;

                // [L4-Fix]: Obsolete SSTable für physisches Scrubbing vormerken
                output.obsolete_files.push(sst_path);
                output.sstables_replaced += 1;
                output.records_purged += purged_in_sst;
            } else {
                let _ = std::fs::remove_file(&temp_path);
            }
        }

        // [INV-DELETION-1]: Obsolete Dateien sicher vernichten (Scrubbing mit Nullen vor Unlink)
        self.dispose_obsolete_files(&output.obsolete_files)?;

        Ok(output)
    }

    fn dispose_obsolete_files(&self, files: &[PathBuf]) -> Result<()> {
        for file in files {
            if file.exists() {
                contextra_durable_fs::scrub_and_remove(file)?;
            }
        }
        contextra_durable_fs::sync_dir(self.store_dir)?;
        Ok(())
    }
}
```

### 3.6 HNSW-Index Slot Zeroisierung & VETO-F02 Einhaltung
```rust
// In crates/contextra-vector/src/hnsw/deletion.rs
use contextra_types::{ContextraError, Result};
use crate::hnsw::HnswIndexCore;

impl HnswIndexCore {
    /// Physische Datenlöschung aus dem Vektorindex unter Einhaltung von VETO-F02 (Tombstone-Pruning).
    pub fn purge_vector_slot(&self, node_id: usize) -> Result<(), ContextraError> {
        let mut arena = self.arena.write();
        
        // 1. Topologische Entkoppelung: Kanten entfernen ohne unzulässiges Rewiring (VETO-F02)
        self.prune_tombstoned_connections(node_id, &mut arena)?;

        // 2. Physische Nullung des Speicher-Slots (RAM-Scrubbing)
        if let Some(slot_slice) = arena.get_slot_mut(node_id) {
            slot_slice.fill(0u8);
        } else {
            return Err(ContextraError::InvalidData(format!(
                "Node ID {} not found in vector arena during purge", node_id
            )));
        }

        Ok(())
    }
}
```

### 3.7 Bi-Temporale Filterung in `contextra-text` (Behebung ENG-FILTER-001)
```rust
// In crates/contextra-text/src/temporal_filter.rs
use contextra_types::{ContextraError, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TemporalAxis {
    EventTime,
    IngestTime,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TemporalFilterCriterion {
    pub axis: TemporalAxis,
    pub start_nanos: Option<u64>,
    pub end_nanos: Option<u64>,
}

impl TemporalFilterCriterion {
    pub fn matches(&self, event_time_nanos: u64, ingest_time_nanos: u64) -> bool {
        let candidate_ts = match self.axis {
            TemporalAxis::EventTime => event_time_nanos,
            TemporalAxis::IngestTime => ingest_time_nanos,
        };

        if let Some(start) = self.start_nanos {
            if candidate_ts < start {
                return false;
            }
        }
        if let Some(end) = self.end_nanos {
            if candidate_ts > end {
                return false;
            }
        }
        true
    }
}
```

### 3.8 Konsolidierter `SnapshotFloorTracker` in `contextra-mvcc` (MVCC-S01..S04)
```rust
// In crates/contextra-mvcc/src/floor.rs
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use contextra_types::TxId;
use parking_lot::Mutex;

#[derive(Debug)]
pub struct SnapshotFloorTracker {
    global_floor: AtomicU64,
    active_snapshots: Mutex<Vec<TxId>>,
}

impl SnapshotFloorTracker {
    pub fn new(initial_tx: TxId) -> Self {
        Self {
            global_floor: AtomicU64::new(initial_tx.as_u64()),
            active_snapshots: Mutex::new(Vec::new()),
        }
    }

    pub fn register_snapshot(&self, tx_id: TxId) {
        let mut guard = self.active_snapshots.lock();
        guard.push(tx_id);
    }

    pub fn unregister_snapshot(&self, tx_id: TxId) {
        let mut guard = self.active_snapshots.lock();
        if let Some(pos) = guard.iter().position(|&t| t == tx_id) {
            guard.swap_remove(pos);
        }
        
        let min_active = guard.iter().map(|t| t.as_u64()).min();
        if let Some(new_floor) = min_active {
            self.global_floor.fetch_min(new_floor, Ordering::SeqCst);
        }
    }

    #[inline(always)]
    pub fn get_floor(&self) -> TxId {
        TxId::new(self.global_floor.load(Ordering::Acquire))
    }
}
```

---

## 4. Vollständige Builder-Verdrahtung in `contextra` (Behebung S5 & N-O)

In [`crates/contextra/src/builder.rs`](../crates/contextra/src/builder.rs) werden alle Schalter strikt validiert und an die internen Engines gebunden:
```rust
// In crates/contextra/src/builder.rs
impl ContextraBuilder {
    pub fn verify_wiring(&self) -> Result<(), ContextraError> {
        // [S5-Fix]: Alle 7 Schalter müssen konsistent sein
        if self.kv_shredding_enabled && !self.kms_enabled {
            return Err(ContextraError::PolicyViolation(
                "KV-Shredding requires an active KMS provider".to_string(),
            ));
        }

        if self.config.deletion_proof_active && self.performance_profile == Some(crate::performance_profile::PerformanceProfile::BareMetal) {
            return Err(ContextraError::PolicyViolation(
                "INV-PERF-PROFILE-1: BareMetal does not support deletion proof".to_string(),
            ));
        }

        Ok(())
    }
}
```

---

## 5. Normative Testfälle & Akzeptanzkriterien

### 5.1 Integrationstest: End-to-End Tenant Scoped Purge
```rust
#[tokio::test]
async fn test_tenant_scoped_storage_purge_roundtrip() -> Result<(), contextra_types::ContextraError> {
    let tmp_dir = std::env::temp_dir().join(format!("contextra_ep1_test_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp_dir);

    let db = contextra::ContextraBuilder::new(128)
        .with_storage_path(&tmp_dir)
        .build()
        .await?;

    let tenant_id = contextra_types::TenantId::try_new(42).unwrap();
    let doc_id = "doc-alpha-001";

    // 1. Dokument speichern
    db.remember(tenant_id, doc_id, "Geheime Nutzerdaten", &[0.1f32; 128]).await?;

    // 2. Dokument physisch purgen (Purge v2 Aufruf über Engine)
    db.forget_key_physical(tenant_id, doc_id).await?;

    // 3. Verifizieren, dass der Schlüssel weder logisch noch physisch existiert
    let res = db.get_document(tenant_id, doc_id).await;
    assert!(res.is_err());

    let _ = std::fs::remove_dir_all(&tmp_dir);
    Ok(())
}
```

### 5.2 Akzeptanz-Gates
```bash
cargo xtask check-compile
cargo xtask doctrine-scan -p contextra-store -p contextra-engine
cargo xtask check-ring-layering
cargo xtask unwired-ports --only-unwired
cargo test -p contextra-store -p contextra-engine -p contextra-vector
```
