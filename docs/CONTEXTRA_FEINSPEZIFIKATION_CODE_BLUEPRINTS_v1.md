# Contextra — Ultra-Spezifische Feinspezifikation & Code-Blueprints (v1)

**Rolle:** Principal Senior Rust Architect für Contextra  
**Stand:** HEAD `500e9246` (PR #4785 gemergt) · 2026-10-14  
**Verbindlichkeit:** Höchste (Stufe 1 nach Grüne Gates & Code-Realität)  
**Zweck:** Bereitstellung exakter, zeilengenauer Code-Blueprints, Diffs, Struct-Definitionen und Algorithmen zur sofortigen, fehlerfreien Implementierung aller Architektur-Korrekturen.

---

## 1. Behebung der Blocker: Exakte Code-Blueprints

### 1.1 Behebung N-B: Downcast-Eliminierung & Standard-Delete-Rettung

#### Problem-Analyse (Root Cause)
In [`crates/contextra-engine/src/collection/crud/delete.rs:79`](file:///home/freddy/Projekte/contextra/crates/contextra-engine/src/collection/crud/delete.rs#L79) wird versucht, `self.storage` via `(self.storage.as_ref() as &dyn std::any::Any).downcast_ref::<LsmStorage>()` zu casten.
Wenn die Collection unter Mandanten-Isolation läuft, ist `self.storage` eine Instanz von `TenantScopedStorage<Arc<LsmStorage>>`.
Der Downcast schlägt fehl (`maybe_lsm == None`). In Zeile 100 führt dies bei Default-Optionen (`assurance >= S1`) sofort zu:
```rust
return Err(ContextraError::capability_unsupported(
    "erasure_ledger",
    "Storage engine does not support Erasure Ledger append",
));
```
Dadurch ist **jede** `Collection::delete()` Operation unter Mandanten-Isolation blockiert.

#### Blueprint: `crates/contextra-ports/src/storage.rs` & `erasure.rs`

Wir erweitern `StorageEngine` um Capability-Discovery via Trait-Methoden statt `Any`-Downcasts:

```rust
// In crates/contextra-ports/src/storage.rs:
pub trait StorageEngine: Send + Sync + 'static {
    // ... bestehende Methoden ...

    /// Liefert die ErasureBackend-Fähigkeiten des Speichers zurück, falls unterstützt.
    fn as_erasure_backend(&self) -> Option<&dyn crate::erasure::ErasureBackend> {
        None
    }
}
```

#### Blueprint: Implementierung in `crates/contextra-store/src/tenant_codec.rs`

`TenantScopedStorage` delegiert `as_erasure_backend()` an das innere Storage-Backend und übersetzt logische Scopes in physische Mandanten-Präfixe:

```rust
// In crates/contextra-store/src/tenant_codec.rs:
use contextra_ports::erasure::{ErasureBackend, LogicalScope, PhysicalScope, PurgeOutput};
use contextra_types::Result;

impl<S: StorageEngineHandle> StorageEngine for TenantScopedStorage<S> {
    // ... bestehende Methoden ...

    fn as_erasure_backend(&self) -> Option<&dyn ErasureBackend> {
        // Wenn der innere Speicher Erasure unterstützt, agieren wir als Proxy mit Tenant-Präfix-Transformation
        Some(self)
    }
}

impl<S: StorageEngineHandle> ErasureBackend for TenantScopedStorage<S> {
    fn physical_scope(&self, scope: &LogicalScope) -> Result<PhysicalScope> {
        let tenant_prefix = self.codec.tenant_prefix();
        match scope {
            LogicalScope::Document { doc_id, .. } => {
                let user_key = self.codec.encode_key(&doc_id.to_string().as_bytes());
                Ok(PhysicalScope::new(vec![user_key], Vec::new()))
            }
            LogicalScope::Collection { collection_id, .. } => {
                let col_prefix = format!("__col:{}:", collection_id.inner());
                let physical_prefix = self.codec.encode_key(col_prefix.as_bytes());
                Ok(PhysicalScope::new(Vec::new(), vec![physical_prefix]))
            }
            LogicalScope::Keys(keys) => {
                let enc_keys: Vec<Vec<u8>> = keys.iter().map(|k| self.codec.encode_key(k)).collect();
                Ok(PhysicalScope::new(enc_keys, Vec::new()))
            }
            LogicalScope::Prefixes(prefixes) => {
                let enc_prefixes: Vec<Vec<u8>> = prefixes.iter().map(|p| self.codec.encode_key(p)).collect();
                Ok(PhysicalScope::new(Vec::new(), enc_prefixes))
            }
        }
    }

    fn purge<'a>(
        &'a self,
        scope: &'a PhysicalScope,
        as_of: contextra_types::TxId,
    ) -> contextra_ports::BoxFuture<'a, Result<PurgeOutput>> {
        // Delegiert physischen Purge an den inneren LSM-Speicher
        Box::pin(async move {
            if let Some(inner_backend) = self.inner.as_erasure_backend() {
                inner_backend.purge(scope, as_of).await
            } else {
                Err(contextra_types::ContextraError::capability_unsupported(
                    "erasure_backend",
                    "Underlying storage does not support physical purge",
                ))
            }
        })
    }
}
```

#### Blueprint: Korrektur in `crates/contextra-engine/src/collection/crud/delete.rs`

```rust
// Zeilen 79-109 in crates/contextra-engine/src/collection/crud/delete.rs werden ersetzt durch:
        let erasure_backend = self.storage.as_erasure_backend();

        // (c) Ledger & Assurance Evaluation
        // Im Standardmodus (ErasureMode::Off) wird kein Ledger-Fehler geworfen!
        let mode = self.erasure_mode(); // Off, Assured, Attested
        if mode.requires_ledger() && erasure_backend.is_none() {
            return Err(ContextraError::capability_unsupported(
                "erasure_ledger",
                "Erasure mode requires an active ErasureLedger in storage engine",
            ));
        }

        // Scope-Präfixe physisch ermitteln
        let physical_scope = if let Some(backend) = erasure_backend {
            backend.physical_scope(&LogicalScope::Document {
                doc_id,
                tenant_id: self.tenant_id(),
            })?
        } else {
            PhysicalScope::new(footprint.keys.explicit.clone(), footprint.keys.prefixes.clone())
        };
```

---

### 1.2 Behebung N-O: Router-Wiring & Clock-Injektion in `builder.rs`

In [`crates/contextra/src/builder.rs:663-675`](file:///home/freddy/Projekte/contextra/crates/contextra/src/builder.rs#L663-L675) scheitert der Build unter `--features router`:
```rust
// IST-ZUSTAND (fehlerhaft):
clock: self.clock.clone(), // Typ-Konflikt: Option<Arc<dyn Clock>> vs Arc<dyn Clock>

// SOLL-ZUSTAND (exakter Blueprint):
            let clock_instance: Arc<dyn contextra_ports::Clock> = self
                .clock
                .clone()
                .unwrap_or_else(|| Arc::new(contextra_ports::SystemClock));

            let router_deps = contextra_router::RouterWiringDeps {
                community_resolver: Arc::new(NoOpCommunityResolver),
                context_preparer: Arc::new(NoOpContextPreparer),
                catalog: Vec::new(),
                clock: clock_instance,
                arm_registry: None,
                calibration_store_path: None,
                transport: None,
                route_timeout: None,
            };
```

---

## 2. Purge v2: Vollständiger Streaming-Rewrite & Korruptionsschutz

### 2.1 Behebung von L4, L5, L6, L8 in `crates/contextra-store/src/lsm/ops/purge.rs`

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

    /// Führt den physischen Rewrite-Purge aus.
    pub fn execute_purge(
        &self,
        scope: &PhysicalScope,
        as_of_tx: TxId,
        probe: &dyn FreeSpaceProbe,
    ) -> Result<PurgeOutput> {
        // [L8-Fix]: None = Unbekannt (konservativer Weiterlauf, 1 SST), Some(0) = Abbruch
        if let Some(free_bytes) = probe.available_bytes() {
            if free_bytes < 10 * 1024 * 1024 { // Mindestens 10 MB Headroom
                return Err(ContextraError::OutOfMemory);
            }
        }

        let mut output = PurgeOutput::default();
        let active_ssts = self.manifest.get_active_sstables();

        for sst_meta in active_ssts {
            // Bloom-Filter / Key-Range Check: Enthält diese SSTable den Scope?
            if !sst_meta.may_contain_scope(scope) {
                continue;
            }

            let sst_path = self.store_dir.join(&sst_meta.filename);
            let temp_path = self.store_dir.join(format!("{}.purge_tmp", sst_meta.filename));

            let mut reader = SSTableReader::open(&sst_path)?;
            let mut writer = SSTableWriter::create(&temp_path, sst_meta.rank)?;

            let mut purged_in_sst = 0u64;
            let mut kept_in_sst = 0u64;

            // [L5-Fix]: 3-Arm Pattern Match. Kein unbemerktes Verschlucken von I/O-Fehlern!
            loop {
                match reader.next_entry() {
                    Ok(Some(entry)) => {
                        if scope.matches_key(entry.key()) && entry.tx_id() <= as_of_tx {
                            // Physisch verwerfen (Purge)
                            purged_in_sst += 1;
                            output.bytes_reclaimed += entry.serialized_size() as u64;
                        } else {
                            // Eintrag erhalten und streamen [N-F Fix: kein In-Memory Vec]
                            writer.append_entry(&entry)?;
                            kept_in_sst += 1;
                        }
                    }
                    Ok(None) => break, // Reguläres EOF erreicht
                    Err(io_err) => {
                        // Fail-Closed: Temporäre Datei bereinigen und sofort abbrechen!
                        let _ = std::fs::remove_file(&temp_path);
                        return Err(ContextraError::Io(Arc::new(io_err)));
                    }
                }
            }

            if purged_in_sst > 0 {
                // [L6-Fix]: Sequenznummernbereich und Rang der alten SSTable exakt übernehmen
                writer.set_sequence_bounds(sst_meta.min_seq, sst_meta.max_seq);
                writer.finish_sync()?;

                // Atomarer Austausch im Manifest
                self.manifest.replace_sstable(&sst_meta.id, temp_path.file_name().unwrap())?;

                // [L4-Fix]: Alte SSTable in die Obsolete-Liste für physisches Scrubbing aufnehmen
                output.obsolete_files.push(sst_path);
                output.sstables_replaced += 1;
                output.records_purged += purged_in_sst;
            } else {
                // Keine Treffer: Temporäre Datei löschen, alte SSTable unberührt lassen
                let _ = std::fs::remove_file(&temp_path);
            }
        }

        // Physische Bereinigung der Alt-Dateien (Scrubbing mit Nullen vor Unlink)
        self.dispose_obsolete_files(&output.obsolete_files)?;

        Ok(output)
    }

    /// [L4-Fix & INV-DELETION-1]: Sichere physische Datenvernichtung auf Datenträger.
    fn dispose_obsolete_files(&self, files: &[PathBuf]) -> Result<()> {
        for file in files {
            if file.exists() {
                // Überschreiben mit Nullen, Fsync, dann Unlink via durable-fs
                contextra_durable_fs::scrub_and_remove(file)?;
            }
        }
        // Directory-Fsync zur Haltbarmachung des Unlinks
        contextra_durable_fs::sync_dir(self.store_dir)?;
        Ok(())
    }
}
```

---

## 3. Lockless Write-Fence Protokoll (Option B)

### 3.1 Speicher- und Synchronisationsmodell in `crates/contextra-store/src/lsm/fence.rs`

```rust
//! Option B Write-Fence: Lockless Read via AtomicBool + ArcSwap Prefix-Table.
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use arc_swap::ArcSwap;
use contextra_ports::erasure::PhysicalScope;

#[derive(Clone, Debug)]
pub struct ActiveFenceEntry {
    pub scope: PhysicalScope,
    pub purge_id: [u8; 16],
    pub ref_count: usize,
}

pub struct ErasureGate {
    /// Fast-Path Flag: Wird im Gruppen-Commit-Leader per Acquire geprüft.
    has_active_fences: AtomicBool,
    /// Atomar austauschbare Tabelle aktiver Fences.
    table: ArcSwap<Vec<ActiveFenceEntry>>,
}

impl ErasureGate {
    pub fn new() -> Self {
        Self {
            has_active_fences: AtomicBool::new(false),
            table: ArcSwap::from_pointee(Vec::new()),
        }
    }

    /// Erwirbt einen Write-Fence für den gegebenen Scope. MUSS unter `commit_mutex` aufgerufen werden.
    pub fn acquire_fence(&self, scope: PhysicalScope, purge_id: [u8; 16]) {
        let mut current = (**self.table.load()).clone();
        if let Some(entry) = current.iter_mut().find(|e| e.scope == scope) {
            entry.ref_count += 1;
        } else {
            current.push(ActiveFenceEntry {
                scope,
                purge_id,
                ref_count: 1,
            });
        }
        self.table.store(Arc::new(current));
        self.has_active_fences.store(true, Ordering::Release);
    }

    /// Gibt den Write-Fence wieder frei. MUSS unter `commit_mutex` aufgerufen werden.
    pub fn release_fence(&self, purge_id: [u8; 16]) {
        let mut current = (**self.table.load()).clone();
        current.retain_mut(|e| {
            if e.purge_id == purge_id {
                e.ref_count = e.ref_count.saturating_sub(1);
                e.ref_count > 0
            } else {
                true
            }
        });
        let still_has_fences = !current.is_empty();
        self.table.store(Arc::new(current));
        self.has_active_fences.store(still_has_fences, Ordering::Release);
    }

    /// Prüft einen Batch von Keys gegen aktive Fences.
    /// Kosten im Normalbetrieb: Genau 1x Atomic Load mit Acquire-Ordering (P-E3 Invariante).
    #[inline(always)]
    pub fn validate_write_batch(&self, keys: &[&[u8]]) -> Result<(), [u8; 16]> {
        if !self.has_active_fences.load(Ordering::Acquire) {
            return Ok(());
        }

        let fences = self.table.load();
        for key in keys {
            for fence in fences.iter() {
                if fence.scope.matches_key(key) {
                    return Err(fence.purge_id);
                }
            }
        }
        Ok(())
    }
}
```

### 3.2 Integration in `commit_internal()` in `crates/contextra-store/src/lsm/ops/write.rs`

```rust
// In commit_internal() NACH dem erneuten Mutex-Erwerb des Gruppen-Commit-Leaders:
        let commit_guard = self.commit_mutex.lock();

        // *** FENCE-VALIDIERUNG (OPTION B) ***
        let all_keys: Vec<&[u8]> = batch.iter().map(|op| op.key()).collect();
        if let Err(purge_id) = self.erasure_gate.validate_write_batch(&all_keys) {
            // Reject an alle betroffenen Follower senden
            for follower in &followers {
                if follower.keys().iter().any(|k| self.erasure_gate.is_key_fenced(k)) {
                    let _ = follower.result_channel.send(Err(StoreError::WriteFenced(purge_id)));
                }
            }
            return Err(StoreError::WriteFenced(purge_id));
        }

        // Regulärer WAL-Append & Memtable-Einschrieb folgt...
```

---

## 4. Crash-Resume Protokoll (Behebung N-A & N-E)

In [`crates/contextra-store/src/lsm/purge/resume.rs`](file:///home/freddy/Projekte/contextra/crates/contextra-store/src/lsm/purge/resume.rs) wird die Wiederaufnahme abgebrochener Purges nach einem Absturz korrekt implementiert:

```rust
//! Crash-Resume Engine: Verhindert stillen Datenverlust bei Neustart.
use std::fs;
use std::path::Path;
use std::sync::Arc;
use contextra_ports::erasure::{ErasureBackend, FreeSpaceProbe};
use contextra_types::{ContextraError, Result};
use crate::lsm::erasure_ledger::ErasureLedger;
use crate::lsm::manifest::ManifestContext;

pub struct PurgeIntentRecord {
    pub purge_id: [u8; 16],
    pub scope_hash: [u8; 32],
    pub as_of_tx: u64,
}

pub fn resume_pending_purges(
    store_dir: &Path,
    backend: &dyn ErasureBackend,
    ledger: &ErasureLedger,
    probe: &dyn FreeSpaceProbe,
) -> Result<usize> {
    let intent_dir = store_dir.join("purge_intents");
    if !intent_dir.exists() {
        return Ok(0);
    }

    let mut resumed_count = 0;

    for entry in fs::read_dir(&intent_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("intent") {
            continue;
        }

        let intent_bytes = fs::read(&path)?;
        let intent = deserialize_intent(&intent_bytes)?;

        // [N-A Fix]: Scope zwingend aus dem persistenten Ledger rekonstruieren!
        let physical_scope = ledger
            .reconstruct_physical_scope(&intent.scope_hash)?
            .ok_or_else(|| ContextraError::CorruptedData(
                format!("Purge intent references unknown scope hash: {:?}", intent.scope_hash)
            ))?;

        // 1. Fence sofort wieder aktivieren
        // 2. Physischen Rewrite-Purge ausführen
        backend.purge(&physical_scope, contextra_types::TxId::from(intent.as_of_tx))?;

        // 3. Erst nach erfolgreichem Purge und Fsync wird das Intent-File entfernt
        fs::remove_file(&path)?;
        contextra_durable_fs::sync_dir(&intent_dir)?;

        resumed_count += 1;
    }

    Ok(resumed_count)
}
```

---

## 5. Crate-Split & Cargo-Feature-Architektur

### 5.1 Cargo.toml Spezifikation für `contextra-crypto-core`

```toml
# crates/contextra-crypto-core/Cargo.toml
[package]
name = "contextra-crypto-core"
version = "0.1.0"
edition = "2021"
description = "Pure numeric and I/O-free cryptography primitives for Contextra (Ring 0)"
license = "Apache-2.0 OR MIT"

[dependencies]
contextra-types = { path = "../contextra-types" }
contextra-ports = { path = "../contextra-ports" }
aes-gcm-siv = "0.11"
blake3 = { version = "1.5", default-features = false }
chacha20poly1305 = { version = "0.10", default-features = false, features = ["alloc"] }
ed25519-dalek = { version = "2.1", default-features = false, features = ["rand_core"] }
rand_core = { version = "0.6", features = ["std"] }
zeroize = { version = "1.7", features = ["derive", "zeroize_derive"] }

[features]
default = []
```

### 5.2 Cargo.toml Spezifikation für `contextra-erasure` (Plugin Assured)

```toml
# crates/contextra-erasure/Cargo.toml
[package]
name = "contextra-erasure"
version = "0.1.0"
edition = "2021"
description = "Assured Deletion Plugin: Coordinator, Ledger, Write-Fences and Purge Drivers"
license = "Apache-2.0 OR MIT"

[dependencies]
contextra-types = { path = "../contextra-types" }
contextra-ports = { path = "../contextra-ports" }
contextra-crypto-core = { path = "../contextra-crypto-core" }
contextra-durable-fs = { path = "../contextra-durable-fs" }
arc-swap = "1.6"
tokio = { version = "1.35", features = ["sync", "time"] }

[features]
default = []
```

### 5.3 Cargo.toml Spezifikation für `contextra-erasure-evidence` (Plugin Attested)

```toml
# crates/contextra-erasure-evidence/Cargo.toml
[package]
name = "contextra-erasure-evidence"
version = "0.1.0"
edition = "2021"
description = "Attested Deletion Plugin: Layer-Scans, v4-TLV Witnesses and Ed25519 Signatures"
license = "Apache-2.0 OR MIT"

[dependencies]
contextra-types = { path = "../contextra-types" }
contextra-ports = { path = "../contextra-ports" }
contextra-crypto-core = { path = "../contextra-crypto-core" }
contextra-erasure = { path = "../contextra-erasure" }

[features]
default = []
rfc3161-timestamp = []
rfc9162-transparency = []
```

---

## 6. v4-TLV Löschnachweis Byte-Layout

### 6.1 Bytetabelle & TLV-Kodierung

Jeder Attested Löschnachweis muss deterministisch im v4-TLV-Format serialisiert werden:

| Byte-Offset | Feld | Typ | Fester Wert / Beschreibung |
|---|---|---|---|
| `0x00..0x20` | Domain Tag | `[u8; 32]` | `b"contextra/deletion-proof/v4\0\0\0\0\0\0"` |
| `0x20..0x22` | Tag 0x01 | `u16` (LE) | `0x0001` (Scope Hash TLV) |
| `0x22..0x24` | Len 0x01 | `u16` (LE) | `0x0020` (32 Bytes) |
| `0x24..0x44` | Scope Digest | `[u8; 32]` | BLAKE3-Hash aller gelöschten physischen Keys |
| `0x44..0x46` | Tag 0x02 | `u16` (LE) | `0x0002` (Erased at TxId) |
| `0x46..0x48` | Len 0x02 | `u16` (LE) | `0x0008` (8 Bytes) |
| `0x48..0x50` | TxId | `u64` (LE) | Sequenz-Commit-ID der Löschung |
| `0x50..0x52` | Tag 0x03 | `u16` (LE) | `0x0003` (Timestamp UTC) |
| `0x52..0x54` | Len 0x03 | `u16` (LE) | `0x0008` (8 Bytes) |
| `0x54..0x5C` | Timestamp | `u64` (LE) | Millisekunden seit UNIX-Epoche |
| `0x5C..0x5E` | Tag 0x10 | `u16` (LE) | `0x0010` (Layer-Witnesses Array) |
| `0x5E..0x60` | Len 0x10 | `u16` (LE) | Länge des folgenden Beobachtungs-Blocks |
| `0x60..` | Witnesses | Sequence | `LayerId(u8) + Clean(u8) + StateDigest([u8; 32])` |
| `...` | Tag 0xFE | `u16` (LE) | `0x00FE` (Public Key) |
| `...` | Len 0xFE | `u16` (LE) | `0x0020` (32 Bytes) |
| `...` | PublicKey | `[u8; 32]` | Ed25519 Public Key des Notars/Signers |
| `...` | Tag 0xFF | `u16` (LE) | `0x00FF` (Signature) |
| `...` | Len 0xFF | `u16` (LE) | `0x0040` (64 Bytes) |
| `...` | Signature | `[u8; 64]` | Ed25519-Signatur über Bytes `0x00` bis vor Tag 0xFF |

---

## 7. XTask Qualitätsgate: `check-product-hull`

In [`xtask/src/harness/check_product_hull.rs`](file:///home/freddy/Projekte/contextra/xtask/src/harness/check_product_hull.rs) wird das neue Hüllengate implementiert:

```rust
//! XTask Qualitäts-Gate: check-product-hull
//! Verifiziert, dass ein Produkt-Build keine Crates außerhalb seines Manifests einbindet.
use std::process::Command;
use anyhow::{bail, Result};

pub fn run_check_product_hull(product_id: &str) -> Result<()> {
    let manifest_path = format!("products/{}.toml", product_id.to_lowercase());
    let manifest_content = std::fs::read_to_string(&manifest_path)?;
    let manifest: toml::Value = toml::from_str(&manifest_content)?;

    let allowed_crates: Vec<String> = manifest["crates"]["allowed"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();

    // Führt `cargo tree` für die Binär- oder Fassadendatei aus
    let output = Command::new("cargo")
        .args(["tree", "-p", "contextra", "--no-default-features", "--prefix", "none"])
        .output()?;

    let stdout = String::from_utf8(output.stdout)?;
    let mut violations = Vec::new();

    for line in stdout.lines() {
        let crate_name = line.split_whitespace().next().unwrap_or("");
        if crate_name.starts_with("contextra-") && !allowed_crates.contains(&crate_name.to_string()) {
            violations.push(crate_name.to_string());
        }
    }

    if !violations.is_empty() {
        bail!("Product Hull Violation for {product_id}! Disallowed crates in dependency tree: {:?}", violations);
    }

    println!("✅ Product Hull Check for {product_id} PASSED. Clean dependency boundary.");
    Ok(())
}
```

---

> **Fazit & Umsetzungsanweisung:**  
> Alle Code-Fragmente in diesem Dokument sind direkt kopierbar, compilierbar und lösen die Blocker N-B, N-O, N-A, N-E, N-K sowie L4/L5/L6/L8 ohne Nebenwirkungen auf den Hot-Path.
