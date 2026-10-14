# Contextra — Mikrofeingranulare Implementierungsspezifikation EP1: Contextra Memory

**Dokumenten-ID:** `DOC-SPEC-EP1-MEMORY-2026-10-14`  
**Version:** 1.0 · **Datum:** 14. Oktober 2026  
**Autor:** Principal Senior Rust Architect für Contextra  
**Status:** Verbindliche, normative Implementierungsanweisung für Release 1.0  
**Geltungsbereich (Disjunktive Crates):**  
- **Ring 0:** [`contextra-types`](file:///home/freddy/Projekte/contextra/crates/contextra-types), [`contextra-ports`](file:///home/freddy/Projekte/contextra/crates/contextra-ports), [`contextra-mvcc`](file:///home/freddy/Projekte/contextra/crates/contextra-mvcc), [`contextra-durable-fs`](file:///home/freddy/Projekte/contextra/crates/contextra-durable-fs), [`contextra-simd`](file:///home/freddy/Projekte/contextra/crates/contextra-simd), [`contextra-sys`](file:///home/freddy/Projekte/contextra/crates/contextra-sys)  
- **Ring 1:** [`contextra-store`](file:///home/freddy/Projekte/contextra/crates/contextra-store)  
- **Ring 0/Algorithmen:** [`contextra-vector`](file:///home/freddy/Projekte/contextra/crates/contextra-vector), [`contextra-text`](file:///home/freddy/Projekte/contextra/crates/contextra-text), [`contextra-rank`](file:///home/freddy/Projekte/contextra/crates/contextra-rank)  
- **Ring 3:** [`contextra-engine`](file:///home/freddy/Projekte/contextra/crates/contextra-engine) (Kern-Module)  
- **Ring 4:** [`contextra`](file:///home/freddy/Projekte/contextra/crates/contextra) (Haupt-Fassade), [`contextra-mcp`](file:///home/freddy/Projekte/contextra/crates/contextra-mcp) (Kern-Endpunkte `search`, `remember`, `forget`)

---

## 1. Architektonische Grenzziehung & Disjunktion

Contextra Memory (EP1) ist das marktreife Fundament des Gesamtsystems: eine eingebettete, zustandslose Single-Node Hybrid-Datenbank mit strikter Mandanten-Isolation, Transaktionssicherheit (MVCC/SSI) und deterministischer Performance.

**Disjunktionsregeln für EP1:**
1. **Kein Löschnachweis-Krypto:** EP1 enthält **keine** Ed25519-Signaturlogik, keine Transparenz-Bäume und keine v4-TLV-Beweiserzeugung (vollständig nach EP2 verlagert). Löschung in EP1 erfolgt ausschließlich über Tombstones und physisches Purging (Purge v2).
2. **Keine Graph-Diffusion / Hyperedges:** CSR-Graphentraversierung und Hyperkanten sind in EP1 inaktiv (nach EP3 verlagert).
3. **Keine lokalen Embedder-Modelle:** Lokale Inference (Candle, ONNX) und KV-Cache-Quantisierung sind nicht Bestandteil von EP1 (nach EP4 verlagert).
4. **Keine WASM-Sandbox:** Merging erfolgt rein über native Rust-Operatoren; keine dynamische Bytecode-Ausführung (nach EP6 verlagert).

---

## 2. Modul- und Datei-Feinspezifikationen

### 2.1 Komponente 1: Port-Erweiterung & `TenantScopedStorage` (Behebung N-B & N-K)

#### Problem
In [`crates/contextra-engine/src/collection/crud/delete.rs:79`](file:///home/freddy/Projekte/contextra/crates/contextra-engine/src/collection/crud/delete.rs#L79) scheitert `downcast_ref::<LsmStorage>()` unter Mandanten-Isolation, da `self.storage` vom Typ `TenantScopedStorage<Arc<LsmStorage>>` ist. Dies führt zum sofortigen Fehler `CapabilityUnsupported` (N-B). Zudem matchen logische Präfixe im Drop-Purge keine physischen mandantenkodierten Keys (N-K).

#### Datei: `crates/contextra-ports/src/storage.rs`
Wir erweitern den Trait `StorageEngine` um Capability-Discovery ohne Downcasts:
```rust
use crate::erasure::ErasureBackend;

pub trait StorageEngine: Send + Sync + 'static {
    // ... bestehende Methoden (put, get, delete, commit, rollback) ...

    /// Ermöglicht den Zugriff auf physische Lösch- und Purge-Fähigkeiten.
    /// Standardmäßig liefert diese Methode `None`.
    fn as_erasure_backend(&self) -> Option<&dyn ErasureBackend> {
        None
    }
}
```

#### Datei: `crates/contextra-store/src/tenant_codec.rs`
`TenantScopedStorage` implementiert `ErasureBackend` als transparenter Proxy mit Key-Präfix-Transformation:
```rust
use contextra_ports::erasure::{ErasureBackend, LogicalScope, PhysicalScope, PurgeOutput};
use contextra_types::{ContextraError, Result, TxId};
use crate::tenant_codec::TenantScopedStorage;

impl<S: contextra_ports::StorageEngine + 'static> contextra_ports::StorageEngine for TenantScopedStorage<S> {
    // ... Weiterleitung der CRUD-Methoden mit self.codec.encode_key(...) ...

    fn as_erasure_backend(&self) -> Option<&dyn ErasureBackend> {
        // Delegiert die Fähigkeit, falls der innere Speicher Erasure unterstützt
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
            LogicalScope::Document { doc_id, .. } => {
                let doc_key = format!("doc:{doc_id}");
                let physical_key = self.codec.encode_key(doc_key.as_bytes());
                Ok(PhysicalScope::from_explicit_keys(vec![physical_key]))
            }
            LogicalScope::Collection { collection_id, .. } => {
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
    ) -> contextra_ports::BoxFuture<'a, Result<PurgeOutput>> {
        Box::pin(async move {
            if let Some(inner_backend) = self.inner.as_erasure_backend() {
                inner_backend.purge(scope, as_of).await
            } else {
                Err(ContextraError::capability_unsupported(
                    "erasure_backend",
                    "Underlying storage does not support physical purge",
                ))
            }
        })
    }
}
```

---

### 2.2 Komponente 2: Purge v2 Streaming-Rewrite Engine (Behebung L4, L5, L6, L8)

#### Problem
- **L4:** Obsolete SSTables wurden im Purge nicht entsorgt.
- **L5:** Lesefehler wurden still per `while let Ok(Some(..))` ignoriert.
- **L6:** SSTable-Positionsverlust und Rangverlust nach Rewrite.
- **L8:** `FreeSpaceProbe = None` wurde als 0 Byte interpretiert und brach ab.

#### Datei: `crates/contextra-store/src/lsm/ops/purge.rs`
```rust
//! Purge v2 Engine: Deterministischer Streaming-Rewrite mit Fail-Closed Semantik.
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
            let temp_path = self.store_dir.join(format!("{}.purge_tmp", sst_meta.filename));

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
                            // [N-F Fix]: Kein In-Memory Zwischenspeichern im Vec, sondern Streaming
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
                self.manifest.replace_sstable(&sst_meta.id, temp_path.file_name().unwrap())?;

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

---

### 2.3 Komponente 3: HNSW-Index Arena Zeroisierung & VETO-F02 Einhaltung

#### Problem
In `crates/contextra-vector` entfernt `remove_with_graph_repair` Kanten topologisch, überschreibt jedoch die Slot-Daten in der Contiguous Edge Arena nicht mit Nullen. Bei Speicherinspektionen könnten Vektordaten rekonstruiert werden.

#### Datei: `crates/contextra-vector/src/hnsw/deletion.rs`
```rust
impl HnswIndexCore {
    /// Physische Datenlöschung aus dem Vektorindex unter Einhaltung von VETO-F02 (Tombstone-Pruning).
    pub fn purge_vector_slot(&self, node_id: usize) -> Result<(), ContextraError> {
        let mut arena = self.arena.write();
        
        // 1. Topologische Entkoppelung: Kanten entfernen ohne unzulässiges Rewiring (VETO-F02)
        self.prune_tombstoned_connections(node_id, &mut arena)?;

        // 2. Physische Nullung des Speicher-Slots (RAM-Scrubbing)
        if let Some(slot_slice) = arena.get_slot_mut(node_id) {
            // Überschreiben des gesamten Vektorslots mit Nullen
            slot_slice.fill(0u8);
        }

        // 3. Slot als frei markieren für zukünftige Allokationen
        arena.mark_freed(node_id);
        
        self.deleted_snapshot.remove(node_id);
        Ok(())
    }
}
```

---

### 2.4 Komponente 4: Bi-Temporale Filter-Korrektur (Behebung `ENG-FILTER-001`)

#### Problem
In [`crates/contextra-engine/src/temporal_filter.rs:118`](file:///home/freddy/Projekte/contextra/crates/contextra-engine/src/temporal_filter.rs#L118) wird `let as_of_tx = TxId::new(as_of_timestamp);` ausgeführt. Ein Unix-Timestamp (z. B. `1728900000`) wird fälschlich als monotoe System-`TxId` (z. B. `42`) interpretiert.

#### Datei: `crates/contextra-engine/src/temporal_filter.rs`
```rust
/// Wendet den bi-temporalen Filter an.
/// 
/// - `as_of_tx`: System-Transaktionsstand (Sequenzzeit)
/// - `as_of_business`: Optionaler Unix-Zeitstempel (Geschäftszeit)
pub fn apply_temporal_validity_filter_at(
    results: Vec<crate::SearchResult>,
    as_of_tx: TxId,
    as_of_business: Option<u64>,
) -> Vec<crate::SearchResult> {
    results
        .into_iter()
        .filter(|res| {
            let window = extract_validity_window(res.metadata.as_ref());
            window.is_valid_bitemporal(as_of_tx, as_of_business)
        })
        .collect()
}
```

---

### 2.5 Komponente 5: Konsolidierung von `SnapshotFloor` & Bounded Retrieval (Behebung `MVCC-S01..S04`)

#### Problem
Doppelte Deklaration des Traits `SnapshotFloor` in `floor.rs` und `snapshot.rs`. Veraltete Methoden `register()` und `register_at()` müssen entfallen.

#### Datei: `crates/contextra-mvcc/src/floor.rs`
Als einzige kanonische Quelle für `SnapshotFloor`:
```rust
pub trait SnapshotFloor: Send + Sync {
    /// Gibt die kleinste aktive Transaktions-ID zurück.
    fn floor(&self) -> TxId;
}

impl<T: SnapshotFloor> SnapshotFloor for Arc<T> {
    #[inline(always)]
    fn floor(&self) -> TxId {
        (**self).floor()
    }
}
```

#### Datei: `crates/contextra-mvcc/src/snapshot.rs`
Bereinigung veralteter Methoden:
```rust
// Entfernen von register() und register_at()
// Ausschließlich acquire() bleibt als sichere API erhalten:
impl SnapshotTracker {
    pub fn acquire(&self, tx_id: TxId) -> ReadSnapshotGuard {
        self.active_snapshots.fetch_add(1, Ordering::SeqCst);
        ReadSnapshotGuard {
            tracker: self.clone(),
            snapshot_seq: tx_id,
        }
    }
}
```

---

### 2.6 Komponente 6: Background-Reaper Lifecycle Manager

#### Problem
In `pub_fn_ohne_aufrufer.csv` gelistet: `start_expiry_reaper`, `start_orphan_reaper`, `start_thermostat_reaper` wurden nie im produktiven Laufzeitpfad von `Contextra::open` instanziiert.

#### Datei: `crates/contextra-engine/src/background_workers/manager.rs`
```rust
use std::sync::Arc;
use tokio::sync::watch;
use tokio::task::JoinHandle;

pub struct BackgroundWorkersManager {
    shutdown_tx: watch::Sender<bool>,
    handles: Vec<JoinHandle<()>>,
}

impl BackgroundWorkersManager {
    pub fn start<S: contextra_ports::StorageEngine + 'static>(
        storage: Arc<S>,
        clock: Arc<dyn contextra_ports::Clock>,
    ) -> Self {
        let (shutdown_tx, shutdown_rx) = watch::channel(false);
        let mut handles = Vec::new();

        // 1. Expiry Reaper
        let rx1 = shutdown_rx.clone();
        let s1 = storage.clone();
        let c1 = clock.clone();
        handles.push(tokio::spawn(async move {
            crate::background_workers::expiry_workers::run_expiry_loop(s1, c1, rx1).await;
        }));

        // 2. Orphan Reaper
        let rx2 = shutdown_rx.clone();
        let s2 = storage.clone();
        handles.push(tokio::spawn(async move {
            crate::background_workers::orphan_workers::run_orphan_loop(s2, rx2).await;
        }));

        Self { shutdown_tx, handles }
    }

    pub async fn shutdown(self) {
        let _ = self.shutdown_tx.send(true);
        for h in self.handles {
            let _ = h.await;
        }
    }
}
```

---

### 2.7 Komponente 7: `ContextraBuilder::verify_wiring` (Behebung S5)

#### Problem
7 Schalter im `ContextraBuilder` speicherten Werte, die in `build()` ungenutzt blieben (`let _ = adapt_enabled;`).

#### Datei: `crates/contextra/src/builder.rs`
```rust
impl ContextraBuilder {
    pub fn build(self) -> Result<ContextraDb, ContextraError> {
        // Verbindliche Prüfung: Kein Schalter darf ins Leere laufen
        self.verify_wiring()?;

        let clock: Arc<dyn contextra_ports::Clock> = self
            .clock
            .unwrap_or_else(|| Arc::new(contextra_ports::SystemClock));

        let id_gen: Arc<dyn contextra_ports::IdGen> = self
            .id_gen
            .unwrap_or_else(|| Arc::new(contextra_ports::SystemIdGen));

        // Initialisierung von Store, MVCC, Indices
        // ...
        Ok(ContextraDb { /* ... */ })
    }

    fn verify_wiring(&self) -> Result<(), ContextraError> {
        if self.adapt_enabled && !cfg!(feature = "adapt") {
            return Err(ContextraError::configuration_error(
                "with_adapt(true) requested but feature 'adapt' is disabled",
            ));
        }
        if self.privacy_enabled && !cfg!(feature = "privacy") {
            return Err(ContextraError::configuration_error(
                "with_privacy(true) requested but feature 'privacy' is disabled",
            ));
        }
        Ok(())
    }
}
```

---

### 2.8 Komponente 8: Asynchroner MCP JSON-RPC Dispatch & Mandanten-Auflösung

#### Problem
In `crates/contextra-mcp/src/server_dispatch.rs:77` führte `block_in_place(|| rt.block_on(...))` zu Worker-Thread-Blockaden. Zudem war `TenantId(1)` hardcodiert.

#### Datei: `crates/contextra-mcp/src/server_dispatch.rs`
```rust
impl McpServer {
    /// Reines asynchrones Dispatching ohne block_in_place
    pub async fn dispatch_request(&self, req: JsonRpcRequest) -> JsonRpcResponse {
        // 1. Mandanten-Extraktion aus Metadaten / Auth-Token (Default: Fallback auf konfigurierten Tenant)
        let tenant_id = req
            .params
            .get("tenant_id")
            .and_then(|v| v.as_u64())
            .and_then(|id| contextra_types::TenantId::try_new(id).ok())
            .unwrap_or(self.default_tenant_id);

        // 2. Asynchrone Tool-Ausführung
        match req.method.as_str() {
            "tools/call" => {
                let result = self.execute_tool_async(tenant_id, req.params).await;
                JsonRpcResponse::from_result(req.id, result)
            }
            _ => JsonRpcResponse::method_not_found(req.id),
        }
    }
}
```

---

## 3. Akzeptanzkriterien & Qualitätsgates (EP1)

1. **Kernel Verification Gate:**
   ```bash
   cargo xtask verify --tier kernel
   ```
   Muss mit Exit-Code 0 abschließen.
2. **Orphan-Symbol-Bereinigung:**
   Keine ungenutzten `pub fn` im Scope von EP1 ohne Begründung.
3. **Doktrin-Scan:**
   `cargo xtask doctrine-scan --crate contextra-store --crate contextra-engine` liefert 0 `unwrap()`/`expect()` in Produktionspfaden.
4. **Mandanten-Isolation Test:**
   `cargo test -p contextra --test tenant_isolation` verifiziert vollständige Datentrennung zwischen `TenantId(1)` und `TenantId(2)`.
