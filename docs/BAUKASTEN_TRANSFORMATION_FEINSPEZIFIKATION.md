# Umfassende Feinspezifikation: Transformation in einen autarken Rust-Baukasten

**Dokumenten-ID:** `DOC-SPEC-BAUKASTEN-TRANSFORMATION-2026-FINAL`
**Rolle:** Principal Senior Rust Engineer & Systemarchitekt
**Datum:** 8. November 2026
**Status:** Definitiv spezifiziert / Umsetzungsleitfaden
**Geltungsbereich:** Gesamter Workspace (36 Crates), insbesondere `contextra-checkpoint`, `contextra-cognition`, `contextra-erasure`, `contextra-core`, `contextra-engine`, `contextra-crypto`, `contextra-vector`, `contextra-text`, `contextra-store`.

---

## 1. Executive Summary & Zielbild der Baukasten-Architektur

Das Gesamtrepository `contextra` wird von einem monolithisch verflochtenen Produkt in einen **vollständig modularen, autarken Rust-Baukasten (Lego-Toolkit)** überführt.

### Die 4 Kardinal-Invarianten der Transformation:
1. **Zero-Monolith & Zero Code-Verlust:**
   Algorithmen und mathematisch-kryptographische Verfahren dürfen unter keinen Umständen gelöscht oder vereinfacht werden. Historisch hochreife Algorithmen (z. B. 2-Bit KIVI Quantisierung, SnapKV Beobachtungsfenster, Irminsul Trie Eviction, DeletionProof v4 nach BSI TR-02102, HNSW LayerScanner Kanten-Zeroization, Reciprocal Rank Fusion, Conformal Prediction) werden sauber in isolierte Bausteine gekapselt.
2. **Auflösung monolithischer Fassaden:**
   Die Kunstgebilde `contextra-core` (256 LOC Re-Export-Hub) und `contextra-engine` (22.246 LOC Nadelöhr) werden aufgelöst bzw. auf schlanke, entkoppelte Services mit schmalen, konsumentenseitigen Traits zerlegt.
3. **Strikte Domänentrennung & DSGVO-Kapselung:**
   Speicher- und Indexbausteine (`contextra-store`, `contextra-vector`, `contextra-text`) stellen *ausschließlich* neutrale, mathematische Operationen bereit (`insert`, `search`, `remove`, `vacuum`). Sämtliche Löschketten, DSGVO-Art.-17-Sagas, Witness-Generationen, BSI-Audit-Reports und Krypto-Shredding-Verfahren residieren exklusiv in `contextra-erasure`.
4. **Harte Concurrency-Hygiene in Tokio:**
   Vollständige Eliminierung von `lock_across_await` (INV-LOCK-ORDER-1) zur Vermeidung von Thread-Starvation und Inversions-Deadlocks; Beseitigung aller blockierenden `block_on`-Aufrufe in asynchronen Pfaden in `contextra-checkpoint` und `contextra-cognition`.

---

## 2. Empirische Befunde & Faktenbasis aus Diagnostics und Git-Historie

Die Codebase-Diagnosen und der Git-Verlauf offenbaren den genauen Ist-Zustand und die Abweichungen zu vorherigen Berichten:

### 2.1 Diagnostics-Fakten (`docs/diagnostics/`)
- **Concurrency-Auditor (`concurrency_lock_auditor.py`):**
  - `contextra-checkpoint`: 14 `lock_across_await`-Meldungen (6 in `src/store.rs`, 8 in `tests/`).
  - `contextra-cognition`: 1 `lock_across_await`-Meldung in `session.rs:271` sowie 2 gefährliche `block_on`-Aufrufe in `consolidation_executor.rs` (Zeile 216 mit Noop-Waker / `block_in_place` und Zeile 673 innerhalb `spawn_blocking` kombiniert mit `blocking_lock`).
  - `contextra-checkpoint` hat in `store.rs` (Zeilen 348–391) eine dreifach verschachtelte Fallback-Kaskade mit `block_in_place`, `std::thread::spawn` und `block_on` im Konstruktor.
- **Erasure-Leak-Prober (`erasure_leak_prober.py`):**
  - Insgesamt 113 Verstöße gegen Regel M7 im Speicher- und Datenkern:
    - `contextra-crypto`: 66 Verstöße (`DeletionProof`, `RevocationLog`, BSI-Referenzen in Unit- und Fuzz-Tests).
    - `contextra-engine`: 25 Verstöße (Saga-Orchestrierung, `DeletionProof`, `KeyProvider`).
    - `contextra-audit-export`: 20 Verstöße (BSI-Mapping und DSGVO-Referenzen).
    - `contextra-store`: 2 Verstöße in `kv/segment.rs` (Legacy `kv_shredding` Referenzen).
- **Abhängigkeits-Graph (`cargo_dependency_graph.py`):**
  - Aktuell 0 zyklische Abhängigkeiten, jedoch extreme Hub-Bindung: 27 von 36 Crates hängen direkt an `contextra-types`, wodurch ein isoliertes Kopieren in Drittprojekte verhindert wird.

### 2.2 Erkenntnisse aus der Git-Historie (Historische Reife & Verluste)
- **Commit `fc9d59b2f` (SOTA Algorithmen-Feinspezifikation 2026):**
  Definiert hochreife Algorithmen: KIVI 2-Bit Quantisierung für KV-Caches, SnapKV Observation Windowing, Irminsul Radix-Trie mit Tier-2 Spill-over, Spektrale Hyperedge Deduplizierung, Budgeted k-Path Diffusion (PathRAG), sowie DeletionProof v4 mit 7-Schichten-Witness (L1 MemTable bis L7 Disk Zeroize) nach BSI TR-02102.
- **Commit `a43391fa5` (Crypto DeletionProof Modularisierung):**
  Enthält die vollständig ausgereifte, modulare Verifikations- und Erzeugungslogik für kryptographische Löschnachweise (Ed25519, Golden-Vectors, Nonce-Bindung). Diese Logik wurde in `contextra-crypto` deponiert und muss sauber nach `contextra-erasure` transferiert werden.
- **Commit `7963b3363` (Vector HNSW Purge & Scanner Isolation):**
  `HnswPurger` implementiert Kantenbereinigung über alle Schichten, Neuwahl des Entry-Points und Memory-Zeroization. Das Primitiv wurde als `#[deprecated]` markiert und muss als neutrales `vacuum()`-Verfahren im Vektor-Crate verbleiben, während der Orchestrierungs-Trigger nach `contextra-erasure` wandert.
- **Commit `0304f9a18` (Rückstellung):**
  Hier wurden zuvor hunderte XTask-Prüfungen und komplexe Verknüpfungen zurückgesetzt. Dies bestätigt: Monolithische Test-Frameworks müssen durch schlanke Cargo-Bordmittel ersetzt werden.

---

## 3. Fassaden-Auflösung (`contextra-core`, `contextra-engine`)

### 3.1 Auflösung von `contextra-core`
`contextra-core` (256 LOC) fungiert lediglich als Typ- und Port-Re-Export-Hub (`types`, `ports`, `mvcc`, `wire`).
- **Maßnahme:** Alle externen Konsumenten (`contextra`, `contextra-engine`, `contextra-db`) importieren direkt aus den atomaren Crates (`contextra-types`, `contextra-ports`).
- `crates/contextra-core` wird aus dem Workspace entfernt. Es findet kein Algorithmenverlust statt.

### 3.2 Zerlegung von `contextra-engine` (22.246 LOC Monolith)
`contextra-engine` verbindet heute als Nadelöhr Speicher, Vektoren, Text, Reranker, Checkpoint und Löschung.
- **Zielbild:** Zerlegung in drei autarke Schichten:
  1. **`contextra-hybrid` (Neuer autarker Baustein):**
     Reine Suchorchestrierung von Vektor- + BM25-Ergebnissen via Reciprocal Rank Fusion (RRF) und Conformal Calibration. Völlig entkoppelt von konkreten Engines über konsumentenseitige Traits.
  2. **`contextra-transaction`:**
     Generische 2-Phasen-Commit-Koordination (Store, Vektor, Text) basierend auf neutralen `CommitStage`-Interfaces.
  3. **Löschung nach `contextra-erasure`:**
     Die gesamte `erasure.rs` (600 LOC) wird aus der Engine extrahiert.

### 3.3 Konsumentenseitige Trait-Muster (Consumer Traits Pattern)
Statt alle Crates an einen riesigen, globalen Trait-Hub in `contextra-ports` zu ketten, definieren koordinierende Bausteine minimale Traits lokal:

```rust
// In contextra-hybrid/src/lib.rs - Autarker Baustein ohne Zwang zu contextra-types
pub trait VectorIndexReader<K> {
    type Error: std::error::Error + Send + Sync + 'static;
    fn search_knn(&self, query: &[f32], top_k: usize)
        -> impl std::future::Future<Output = Result<Vec<(K, f32)>, Self::Error>> + Send;
}

pub trait TextIndexReader<K> {
    type Error: std::error::Error + Send + Sync + 'static;
    fn search_bm25(&self, query: &str, top_k: usize)
        -> impl std::future::Future<Output = Result<Vec<(K, f32)>, Self::Error>> + Send;
}

pub struct HybridSearchEngine<V, T, K> {
    vector_reader: Arc<V>,
    text_reader: Arc<T>,
    _phantom: std::marker::PhantomData<K>,
}
```

---

## 4. Concurrency-Hygiene: Beseitigung von `lock_across_await` und `block_on`

### 4.1 `contextra-checkpoint`: Scope-Verkürzung & Beseitigung von Konstruktor-`block_on`

#### A. Behebung von `lock_across_await` in `store.rs`
Die 6 Guard-Meldungen in `store.rs` (Zeilen 421, 536, 613, 758, 865, 912) betreffen `_guard` auf `write_lock` und `hwm_lock` (`tokio::sync::Mutex<()>`).
- **Differenzierung:** `tokio::sync::Mutex` darf über `.await` gehalten werden, wenn Serialisierung das Ziel ist.
- **Kritische Gefahr:** Wo synchrone `parking_lot::RwLock`-Guards (`index`) vor asynchroner I/O (`storage.put(...).await`) gehalten werden, droht unmittelbarer Deadlock.
- **Spezifiziertes Refactoring:**
```rust
// ✅ REFACTORED: parking_lot Guard-Scope endet strikt VOR dem ersten .await
pub async fn save_checkpoint_clean<S: StorageEngine>(
    &self,
    meta: CheckpointMeta,
) -> Result<(), CheckpointError> {
    // 1. Asynchroner I/O Pfad ohne synchronen Lock
    let manifest_bytes = serde_json::to_vec(&meta)
        .map_err(|e| CheckpointError::Serialization(e.to_string()))?;
    self.storage.put(meta.tx_id, b"checkpoint_meta", &manifest_bytes).await
        .map_err(|e| CheckpointError::Storage(e.to_string()))?;

    // 2. Speicher-Index atomar und kurz aktualisieren
    {
        let mut index_guard = self.index.write();
        index_guard.insert(meta.name.clone(), meta);
    } // Guard wird hier deterministisch VOR jedem weiteren .await gedroppt

    Ok(())
}
```

#### B. Beseitigung des Konstruktor-`block_on` (`store.rs:348-391`)
- **Problem:** `new_with_orphan_registry` versucht synchron zu sein und nutzt `block_in_place(block_on)`. In einer `current_thread`-Runtime (häufig in Tests, Microservices oder FFI) stürzt dies unweigerlich mit Panik ab.
- **Spezifizierte Lösung:** Asynchroner Primär-Konstruktor `open(...)`, synchroner Konstruktor hinter dediziertes Feature `blocking`:
```rust
impl<S: StorageEngine> PersistentCheckpointStore<S> {
    /// Reiner, sicherer asynchroner Einstiegspunkt für Tokio-Umgebungen.
    pub async fn open(
        storage: Arc<S>,
        namespace: impl Into<String>,
        orphan_registry: Arc<InstanceOrphanRegistry>,
    ) -> Result<Self, CheckpointError> {
        let mut store = Self::new_empty(storage, namespace, orphan_registry);
        store.recover_existing_checkpoints().await?;
        Ok(store)
    }
}

#[cfg(feature = "blocking")]
pub mod blocking {
    use super::*;

    /// Synchroner Konstruktor: Läuft strikt in einem isolierten OS-Thread,
    /// weigert sich bei laufender Tokio-Runtime, um Deadlocks/Panics auszuschließen.
    pub fn open_sync<S: StorageEngine + 'static>(
        storage: Arc<S>,
        namespace: impl Into<String>,
        orphan_registry: Arc<InstanceOrphanRegistry>,
    ) -> Result<PersistentCheckpointStore<S>, CheckpointError> {
        if tokio::runtime::Handle::try_current().is_ok() {
            return Err(CheckpointError::RuntimeConfiguration(
                "open_sync darf nicht innerhalb einer aktiven Tokio-Runtime aufgerufen werden. Bitte 'PersistentCheckpointStore::open().await' verwenden."
            ));
        }
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|e| CheckpointError::Runtime(e.to_string()))?;
            rt.block_on(PersistentCheckpointStore::open(storage, namespace, orphan_registry))
        })
        .join()
        .map_err(|_| CheckpointError::ThreadPanic)?
    }
}
```

### 4.2 `contextra-cognition`: Beseitigung von Sync/Async-Vermischung

#### A. Behebung von `score_grounding` (`consolidation_executor.rs:211-221`)
- **Problem:** `Future::poll` mit Noop-Waker und Fallback auf `block_in_place(block_on)`.
- **Spezifizierte Lösung:** Die Funktion `score_grounding` wird vollständig asynchron:
```rust
// Vorher: pub fn score_grounding(...) -> Result<f32, ContextraError>
// Nachher:
pub async fn score_grounding<V: GroundingValidator>(
    validator: &V,
    source: &Node,
    target: &Node,
) -> Result<f32, CognitionError> {
    validator.validate_grounding(source, target).await
        .map_err(|e| CognitionError::Validation(e.to_string()))
}
```

#### B. Behebung von `spawn_blocking` + `blocking_lock` + `block_on` (`consolidation_executor.rs:670-675`)
- **Problem:** In einem Tokio-Blocking-Thread wird ein Lock genommen und dann `handle.block_on(async { ... })` aufgerufen.
- **Spezifizierte Lösung:** Übergabe des Zustands (Ownership-Transfer) oder asynchroner Lock:
```rust
// Vorher:
// tokio::task::spawn_blocking(move || {
//     let mut tracker_g = stability_tracker_clone.blocking_lock();
//     handle.block_on(async { ... })
// });

// Nachher (Direkt im Async-Kontext):
let stage_res = {
    let mut tracker = stability_tracker.lock().await;
    execute_leanrag_aggregation_stage_async(
        collection.as_ref(),
        &nodes,
        &edges,
        &cfg,
        &mut *tracker,
    ).await
};
```

#### C. Behebung von `session.rs:271` (`lock_across_await`)
- Verkürzung des Lock-Scopes vor dem Flush- bzw. Sync-Aufruf:
```rust
let active_descriptors = {
    let mut g = self.session_state.lock();
    std::mem::take(&mut g.pending_descriptors)
}; // Lock freigegeben
self.storage.flush_descriptors(&active_descriptors).await?;
```

---

## 5. DSGVO-Entflechtung & Reifegradorchestrierung in `contextra-erasure`

### 5.1 Das Neutralitäts-Prinzip der Speicherindizes
Gemäß Baukasten-Architektur darf kein Index- oder Store-Crate rechtliche oder prozessuale Löschkettenbegriffe enthalten:
- **Index-Schnittstelle:** Bietet nur elementare Methoden:
  - `insert(id, data)`
  - `remove(id) -> Result<bool>` (markiert als entfernt oder löscht aus Routing)
  - `vacuum() -> Result<VacuumStats>` (physische Rekomprimierung, Kanten-Zeroization und Platzfreigabe – völlig neutral benannt)
- **Eliminierung aus `vector`, `text`, `store`:**
  - `HnswPurger`, `TextPurger`, `LsmPurger` und `DeletionProof` werden vollständig aus den Indizes entfernt.
  - Der Zeroize-Code für Vektoren verbleibt neutral im Wartungsverfahren `HnswIndex::vacuum_tombstones()`.

### 5.2 Kapselung aller Lösch-, Nachweis- und Shredding-Logik in `contextra-erasure`
Die gesamte Orchestrierung wandert nach `contextra-erasure`:
1. **Wiederbelebung und Kapselung von `DeletionProof` v4 (aus `contextra-crypto`):**
   - Das 7-Schichten-Witness-Modell (MemTable, HNSW, TextIndex, Relations, ErasureLedger, WAL, Disk Zero-Scrub) wird als autarkes Modul `contextra_erasure::proof` geführt.
   - BSI TR-02102 konforme Ed25519-Signierung und externe Verifikation ohne Abhängigkeit zu Speicher-Interna.
2. **Krypto-Shredding Engine:**
   - Schlüssel-Revokation (`RevocationLog`, `KeyRegistry`) wird als Submodul `contextra_erasure::shredding` gekapselt.
   - Vernichtung des DEK (Data Encryption Key) rendert persistierte Chunks mathematisch unlesbar.
3. **Audit-Export & AVV-Integration:**
   - `contextra-audit-export` und `contextra-avv-generator` werden als Satelliten-Module eng an `contextra-erasure` angebunden bzw. re-exportiert.

```rust
// In contextra-erasure/src/saga.rs
pub struct ErasureSagaOrchestrator<V, T, S> {
    vector_index: Arc<V>,
    text_index: Arc<T>,
    storage: Arc<S>,
    key_manager: Arc<KeyShredder>,
}

impl<V, T, S> ErasureSagaOrchestrator<V, T, S>
where
    V: VectorVacuumTarget,
    T: TextVacuumTarget,
    S: StorageVacuumTarget,
{
    pub async fn execute_art17_erasure(&self, target: EraseTarget) -> Result<ErasureReport, ErasureError> {
        // 1. Physischer Krypto-Schlüssel-Shredding (sofortige Unlesbarkeit)
        let key_witness = self.key_manager.shred_key(&target).await?;

        // 2. Neutrale Wartung auf Indizes triggern
        self.vector_index.vacuum_key(&target).await?;
        self.text_index.vacuum_key(&target).await?;
        self.storage.vacuum_key(&target).await?;

        // 3. BSI TR-02102 konformen DeletionProof v4 erzeugen
        let proof = DeletionProofV4::generate(target, key_witness);

        // 4. Unveränderlichen Bericht ausstellen
        Ok(ErasureReport::new(proof))
    }
}
```

---

## 6. Crate-Design für maximale Autarkie & Standalone-Portierbarkeit

Damit Entwickler beliebige Crates per `cp -r crates/<name> /mein/neues/projekt` übernehmen können:

### 6.1 Beseitigung von `workspace = true` in kopierbaren Bausteinen
Jeder autarke Baustein deklariert in seiner `Cargo.toml` konkrete Versionsnummern:
```toml
# Beispiel: crates/contextra-rank/Cargo.toml
[package]
name = "contextra-rank"
version = "0.2.0"
edition = "2021"
description = "Autarke Ranking- und Fusion-Bibliothek (RRF, DiBud, Conformal Calibration)"
license = "MIT OR Apache-2.0"

[dependencies]
thiserror = "2.0"
serde = { version = "1.0", features = ["derive"], optional = true }
statrs = { version = "0.17", optional = true }

[features]
default = ["std"]
std = []
conformal = ["dep:statrs"]
serde = ["dep:serde"]
```

### 6.2 Standard-Typen First (Primitive & Standard Types)
- Keine Zwangsabhängigkeit zu `contextra_types::DocId` oder `contextra_types::Score`.
- Verwendung von `u64`, `&[f32]`, `&str`, `f32` in den Kern-APIs.
- Ein lokales, schlankes `thiserror`-Enum pro Baustein (`RankError`, `CheckpointError`, `VectorError`).

### 6.3 Automatisierter Isolationsbeweis in der CI (`tools/isolation_proof.py`)
Jeder Baustein muss in einer isolierten Sandbox ohne Workspace-Kontext kompilieren und testen:
```bash
python3 tools/isolation_proof.py -c contextra-rank
python3 tools/isolation_proof.py -c contextra-simd
python3 tools/isolation_proof.py -c contextra-wire
python3 tools/isolation_proof.py -c contextra-checkpoint
```

---

## 7. Phasenbasierter Migrationsplan & Akzeptanzkriterien

| Phase | Aufgabenstellung | Betroffene Crates | Akzeptanzkriterium |
|---|---|---|---|
| **Phase 1** | **Concurrency-Sanierung**<br>• Guard-Scope Verkürzung vor `.await`<br>• Beseitigung von `block_on` in Konstruktoren & Pipelines<br>• Einführung von `open().await` | `contextra-checkpoint`,<br>`contextra-cognition` | • `concurrency_lock_auditor.py` meldet 0 unzulässige Locks über `.await`<br>• Alle Tests grün unter `current_thread` Runtime |
| **Phase 2** | **Fassaden-Auflösung**<br>• `contextra-core` löschen<br>• Importe auf atomare Crates umbiegen<br>• `contextra-hybrid` auskoppeln | `contextra-core`,<br>`contextra-engine`,<br>`contextra` | • `contextra-core` entfernt<br>• `cargo tree -p contextra-cognition` enthält keine `contextra-engine`-Abhängigkeit mehr |
| **Phase 3** | **DSGVO- & Krypto-Entflechtung**<br>• DeletionProof v4 & Krypto-Shredding nach `contextra-erasure`<br>• Indizes strikt auf `insert`/`remove`/`vacuum`<br>• Bereinigung von M7-Leaks | `contextra-crypto`,<br>`contextra-vector`,<br>`contextra-text`,<br>`contextra-store`,<br>`contextra-erasure` | • `erasure_leak_prober.py` meldet 0 M7-Kernel-Verletzungen in Indizes und Stores |
| **Phase 4** | **Autarkie- & Baukasten-Zertifizierung**<br>• `workspace = true` in isolierten Crates auflösen<br>• Standalone-Beispiele & Tests validieren | Gesamter Workspace | • `tools/isolation_proof.py` besteht für alle Bausteine<br>• Zero-Panic Policy verifiziert |
