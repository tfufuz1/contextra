# contextra-testkit

Test Fixtures, Mock Engines, In-Memory Reference Implementations und Port-Konformitätssuiten.

Normativitätsklasse: **Klasse T** (Tooling, Werkzeug / reine Testhilfe; SPEC v4.0 §5.4, `capabilities.toml`).

## Rolle in einem Satz
`contextra-testkit` stellt Test-Mocks, In-Memory Reference Stores, Injektions-Fixtures und wiederverwendbare Vertragstests (Konformitätssuiten) ausschließlich für Testumgebungen bereit.

## Ring, Klasse & Reifegrad
- **Ring:** Werkzeug / Tooling (außerhalb der Ringe 0–4)
- **Klasse:** Klasse T (reine Testhilfe)
- **Reifegrad:** `stable`
- **Sicherheits-Invariante (INV-EMBED-1):** Ausschließlich als `[dev-dependencies]` in Produktions-Crates zugelassen (SPEC §5.4).
- **Klasse-A-Verbot (E-12, B2):** Klasse-A-Bausteine dürfen `contextra-testkit` **nicht** als Dev-Dependency einbinden. Sie pflegen ihre Fault-Injection und Mocks lokal (siehe Kopiervorlage unten).

## Generic Port Conformance Suites (`contextra_testkit::conformance`)

Das Modul `conformance` stellt generische Konformitätssuiten für Port-Traits bereit.
Downstream-Adapter-Crates (z. B. LSM Storage, HNSW Vector Index, Inverted Text Index, CSR Graph) nutzen diese Suiten in ihren eigenen Integrationstests (`#[cfg(test)] mod conformance`), um korrekte Vertragserfüllung abzusichern.

### Verfügbare Suiten

1. **`run_storage_engine_suite`**: Put/Get/Delete, Overwrites, Lexikographische Scan-Reihenfolge, Mandanten-Präfix-Trennung, 1 MB Werte, Leerschlüssel/-werte, Atomare Batches, Uncommitted Tx Isolation.
2. **`run_vector_index_suite`**: Insert & Search Distanz-Sortierung, Deletion/Tombstone-Konsistenz (`all_doc_ids`), Upsert, Dimension-Mismatch Fehler, NaN-Rejection, $k > \text{size}$, DocId Handling.
3. **`run_text_index_suite`**: Indexierung & Suche, Deletion (nie wieder auffindbar), Umlaute/ß-Normalisierung, Leeranfrage-Behandlung.
4. **`run_graph_index_suite`**: Kante einfügen/löschen, Nachbarschaft (`neighbors`), Graph-Traversierung (BFS-Hops), Selbstschleifen-Behandlung.
5. **`run_clock_suite`**: Monotonie (`monotonic_nanos`), Gültige Unix-Nanosekunden.
6. **`run_rng_suite`**: Entropie-Verteilung, `fill_bytes` Abdeckung, `next_unit_f64` im Intervall $[0.0, 1.0)$.
7. **`run_metrics_sink_suite`**: Fehlerfreie Erfassung von Counter, Gauge und Histogramm Metrics.
8. **`run_vfs_suite`**: Atomares Schreiben (`write_atomic`), Leseverhalten, Dateilängen, Verzeichnis-Lifecycle (`create_dir_all`, `sync_dir`, `list_dir`) und Bereinigungsverhalten (`remove_file`).

### Feature-Flags

- `default = ["testkit", "erasure-conformance"]`
- `testkit`: Aktiviert Standard-Testfixtures und Mocks.
- `erasure-conformance`: Kapselt Löschzeugen-Generatoren (`mint_witness_for_tests`) für Entkopplungsprüfungen (§6).

### Baukasten-Export & Isolationsbeweis

- **Isolierter Export:** `python3 tools/export_crate.py contextra-testkit --out /tmp/export-testkit`
- **Entnahme-Probe:** `python3 tools/extract_probe.py contextra-testkit --test`

---

## Anleitung: FaultVfs als Kopiervorlage für Klasse-A-Crates

Gemäß **SPEC v4.0 Regel E-12 / B2** dürfen Klasse-A-Crates keine Abhängigkeit auf `contextra-testkit` besitzen.
Wenn ein Klasse-A-Crate deterministische I/O- oder `fsync`-Fehlerinjektion benötigt, kopiere die folgende Vorlage lokal nach `tests/common/fault_vfs.rs`:

```rust
use std::collections::BTreeMap;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use parking_lot::RwLock;

#[derive(Debug, Clone, Default)]
pub struct LocalFaultConfig {
    pub fail_writes_after: Option<usize>,
    pub fail_syncs_after: Option<usize>,
    pub fail_reads_after: Option<usize>,
    pub fail_all: bool,
}

#[derive(Debug, Clone, Default)]
pub struct LocalFaultVfs {
    files: Arc<RwLock<BTreeMap<PathBuf, Vec<u8>>>>,
    read_count: Arc<AtomicUsize>,
    write_count: Arc<AtomicUsize>,
    sync_count: Arc<AtomicUsize>,
    config: Arc<RwLock<LocalFaultConfig>>,
    crashed: Arc<AtomicBool>,
}

impl LocalFaultVfs {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_config(&self, config: LocalFaultConfig) {
        *self.config.write() = config;
    }

    pub fn trigger_crash(&self) {
        self.crashed.store(true, Ordering::SeqCst);
    }

    pub fn reset_counters(&self) {
        self.read_count.store(0, Ordering::SeqCst);
        self.write_count.store(0, Ordering::SeqCst);
        self.sync_count.store(0, Ordering::SeqCst);
        self.crashed.store(false, Ordering::SeqCst);
    }

    pub fn write_file(&self, path: &Path, data: &[u8]) -> io::Result<()> {
        if self.crashed.load(Ordering::SeqCst) {
            return Err(io::Error::new(ErrorKind::BrokenPipe, "VFS simulated crash"));
        }
        let cfg = self.config.read();
        if cfg.fail_all {
            return Err(io::Error::other("fail_all"));
        }
        let current = self.write_count.fetch_add(1, Ordering::SeqCst);
        if let Some(limit) = cfg.fail_writes_after {
            if current >= limit {
                return Err(io::Error::new(ErrorKind::WriteZero, "Disk write failure"));
            }
        }
        self.files.write().insert(path.to_path_buf(), data.to_vec());
        Ok(())
    }

    pub fn read_file(&self, path: &Path) -> io::Result<Vec<u8>> {
        if self.crashed.load(Ordering::SeqCst) {
            return Err(io::Error::new(ErrorKind::BrokenPipe, "VFS simulated crash"));
        }
        let current = self.read_count.fetch_add(1, Ordering::SeqCst);
        let cfg = self.config.read();
        if let Some(limit) = cfg.fail_reads_after {
            if current >= limit {
                return Err(io::Error::new(ErrorKind::UnexpectedEof, "Disk read failure"));
            }
        }
        self.files.read().get(path).cloned()
            .ok_or_else(|| io::Error::new(ErrorKind::NotFound, "File not found"))
    }

    pub fn sync(&self) -> io::Result<()> {
        if self.crashed.load(Ordering::SeqCst) {
            return Err(io::Error::new(ErrorKind::BrokenPipe, "VFS simulated crash"));
        }
        let current = self.sync_count.fetch_add(1, Ordering::SeqCst);
        let cfg = self.config.read();
        if let Some(limit) = cfg.fail_syncs_after {
            if current >= limit {
                return Err(io::Error::other("Simulated fsync failure"));
            }
        }
        Ok(())
    }
}
```

---

## Minimalbeispiel (Klasse B/C Integration)

```toml
[dev-dependencies]
contextra-testkit = { workspace = true }
```

```rust
#[cfg(test)]
mod tests {
    use contextra_testkit::assert_conformance;
    use contextra_testkit::conformance::*;

    #[test]
    fn test_clock_conformance() {
        let clock = MyClock::new();
        let res = run_clock_suite(&clock);
        assert_conformance!(res);
    }
}
```
