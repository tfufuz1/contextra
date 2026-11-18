# contextra-db — Implementierungsspezifikation (Soll-Zustand)

**Ebene:** E2 (Integration & Orchestrierung)  
**Kern-Closure:** `types`, `ports`, `mvcc`, `store`, `vector`, `text`, `graph`, `rank`, `checkpoint`  
**Status:** Normativ (Soll-Zustand für Entwickler)  
**Version:** 2.0  
**Geltungsbereich:** `crates/contextra-db`  
**Referenzdokumente:** `docs/SPEC.md` §2, §3, §4, §5, §6, §7, §8, §9, §10, §11, §13, §14; `ANALYSIS-E0-CONTRADICTIONS.md`; `ALTLASTEN-AUDIT.md` (AP-07 bis AP-12)  

---

## 1. Zweck & Architekturgarantien

`contextra-db` ist das integrierende Herzstück und der Multi-Index-Transaktionsmotor von Contextra. Als Ebene-E2-Komponente bündelt es alle autonomen Domänenbausteine zu einer konsistenten, transaktionalen und bitemporalen Hybrid-Search-Engine:
- **Multi-Index Orchestrierung:** Atomare Koordination von LSM-Store (`contextra-store`), HNSW-Vektorindex (`contextra-vector`), BM25-Volltextindex (`contextra-text`) und CSR-Wissensgraph (`contextra-graph`).
- **Hybrid-Search-Pipeline:** 4-Signal-Suche (Dense-Vektor, Sparse-Text, Graph-Konnektivität, Temporal-Decay) mit RRF-Fusion (`contextra-rank`) und Auffüllen auf $k$.
- **Transaktionskoordination (2PC / SSI):** Atomare Schreibtransaktionen über alle Indizes mit Serializable Snapshot Isolation (SSI) und Intent-Logging.
- **Synchroner ReadGate-Schutz (SPEC §6):** Durchsetzung des In-Memory Hot-Path `ReadGate` an **ausnahmslos jedem Lesepfad** vor der Entschlüsselung.
- **Bitemporale Zeitreise (SPEC §9):** Unabhängige Abfragen nach Transaktionszeit (`as_of`) und Gültigkeitszeit (`valid_at`).

### Beseitigung aller Altlasten (AP-07 bis AP-12)
1. **AP-07 (Sandbox):** `merge_operator_adapter.rs` wird vollständig entkoppelt; keine Bezüge zu `contextra-sandbox`.
2. **AP-08 (Vault):** `volatile_vault.rs` und die Abhängigkeit auf `contextra-vault` werden bereinigt.
3. **AP-09 (Adapt):** Verweise auf `contextra_adapt::PidController` in `builder.rs` werden durch den lokalen `PidLatencyController` ersetzt.
4. **AP-10 (ONNX):** In `lifecycle.rs` (Z. 196–206) wird der unkompilierbare `contextra_infer_onnx`-Block entfernt.
5. **AP-11 & AP-12 (Cognition & Decay):** Kommentare in `cognition/` und `decay_controller.rs` werden neutralisiert.
6. **Bereinigung der `features = ["workspace"]`:** Alle E1-Abhängigkeiten in `Cargo.toml` werden ohne die veralteten `workspace`-Features eingebunden.
7. **R2-Konformität:** `contextra-testkit` (E3) wird aus den `[dev-dependencies]` entfernt.

### Architekturgarantien (R1–R6, INV-1 bis INV-12)
1. **`#![forbid(unsafe_code)]`:** 100 % speichersicher.
2. **Zero-Panic-Policy:** Keine Panics; alle Methoden liefern `Result<T, ContextraError>`.
3. **Fail-Closed (SPEC §6.1):** Bei unklarem Gate-Zustand wird der Zugriff immer abgewiesen (`Erased`).
4. **INV-5 Determinismus:** Gleiche Suchanfragen liefern identische Trefferreihenfolgen.
5. **Leere Default-Features (R3):** `default = []`.

---

## 2. Architektur & Komponenten

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                contextra-db                                 │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │                              Contextra                              │   │
│   │                                                                     │   │
│   │   ┌──────────────────────┐             ┌────────────────────────┐   │   │
│   │   │    CollectionMap     │             │  TenantCollectionMap   │   │   │
│   │   │  (Default Namespace) │             │  (Isolated Per Tenant) │   │   │
│   │   └──────────┬───────────┘             └───────────┬────────────┘   │   │
│   │              │                                     │                │   │
│   │              ▼                                     ▼                │   │
│   │   ┌─────────────────────────────────────────────────────────────┐   │   │
│   │   │                         Collection                          │   │   │
│   │   │                                                             │   │   │
│   │   │   ┌──────────────┐ ┌──────────────┐ ┌───────────────────┐   │   │
│   │   │   │  LsmStorage  │ │  HnswIndex   │ │   InvertedIndex   │   │   │
│   │   │   │  (Store-L1)  │ │ (Vector-L1)  │ │     (Text-L1)     │   │   │
│   │   │   └──────────────┘ └──────────────┘ └───────────────────┘   │   │
│   │   │   ┌──────────────┐ ┌──────────────┐ ┌───────────────────┐   │   │
│   │   │   │   CsrGraph   │ │  Checkpoint  │ │    KvLockTable    │   │   │
│   │   │   │  (Graph-L1)  │ │  (Backup-L1) │ │ (Per-Key Mutexes) │   │   │
│   │   │   └──────────────┘ └──────────────┘ └───────────────────┘   │   │
│   │   └─────────────────────────────────────────────────────────────┘   │   │
│   │                                                                     │   │
│   │   ┌──────────────────────┐             ┌────────────────────────┐   │   │
│   │   │   HybridSearcher     │             │ PidLatencyController   │   │   │
│   │   │ (4-Signal / RRF)     │             │ (Dynamic Target Lat.)  │   │   │
│   │   └──────────────────────┘             └────────────────────────┘   │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 3. Durchsetzung des ReadGate an allen Lesepfaden (SPEC §6.2)

Jeder Lesepfad MUSS vor der Rückgabe oder Entschlüsselung synchron $O(1)$ gegen das `ReadGate` prüfen:

| Lesepfad | Gate-Check-Stelle | Verhalten bei `GateDecision::Erased` |
|---|---|---|
| `get`, `get_at_snapshot` | Vor Entschlüsselung des Payloads | Sofortiger Abbruch mit `Err(ContextraError::Erased)` |
| `scan`, `scan_prefix` | Vor jedem iterierten Eintrag | Eintrag wird still ausgelassen (nicht unterscheidbar von nie existent) |
| Vektor-/Textsuche | Auf Kandidatenebene vor Fusion | Kandidat wird verworfen; Auffüllen auf $k$ wird ausgelöst |
| Hybrid-Suche | Vor Fusion und Ranking | Kandidat wird ignoriert; Rank-Pipeline sieht ihn nicht |
| Graph-Traversal | Je Kante und Zielknoten | Pfad bricht an diesem Knoten ab |
| Export | Je Eintrag | Gesperrte Einträge werden ausgelassen |

---

## 4. Schreibpfad & 2-Phase-Commit

```rust
use contextra_ids::{CollectionId, DocId, TenantId};
use contextra_privacy::{DataClass, Validated};
use contextra_types::ContextraError;

pub struct InsertParams<'a> {
    pub tenant_id: TenantId,
    pub collection_id: CollectionId,
    pub doc_id: DocId,
    pub data_class: DataClass,
    pub payload: &'a [u8],
}

impl ContextraDb {
    /// Führt einen transaktionalen Multi-Index-Schreibvorgang durch.
    pub async fn insert(&self, params: InsertParams<'_>) -> Result<(), ContextraError> {
        // 1. DataClass Validierung
        match params.data_class {
            DataClass::K => {
                // Bei Enforced Policy: Muss Validated sein
            }
            DataClass::P => {
                // Adressierung muss SubjectRef gebunden sein
            }
            _ => {}
        }

        // 2. CommitBarrier erwerben (shared)
        let _permit = self.commit_barrier.shared().await;

        // 3. AEAD-Verschlüsselung des Payloads via contextra-crypto

        // 4. Multi-Index Commit (WAL -> MemTable -> Vector -> Text -> Graph)
        Ok(())
    }
}
```

---

## 5. `Cargo.toml` Spezifikation

```toml
[package]
name = "contextra-db"
version.workspace = true
edition.workspace = true
license.workspace = true
rust-version.workspace = true
description = "Contextra — Embedded hybrid-search engine, multi-index transactions, and GDPR gate integration"

[package.metadata.contextra]
ring = "2"
level = 2

[lints]
workspace = true

[dependencies]
contextra-ids = { workspace = true }
contextra-env = { workspace = true }
contextra-types = { workspace = true }
contextra-ports = { workspace = true }
contextra-crypto = { workspace = true }
contextra-privacy = { workspace = true }
contextra-erasure = { workspace = true }
contextra-mvcc = { workspace = true }
contextra-store = { workspace = true }
contextra-vector = { workspace = true }
contextra-text = { workspace = true }
contextra-graph = { workspace = true }
contextra-rank = { workspace = true }
contextra-checkpoint = { workspace = true }
tokio = { workspace = true }
parking_lot = { workspace = true }
ahash = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
thiserror = { workspace = true }
tracing = { workspace = true }

[dev-dependencies]
tempfile = { workspace = true }
proptest = "1.4"
tokio = { workspace = true, features = ["full", "test-util"] }
criterion = { workspace = true }

[features]
default = []
```

---

## 6. Implementierungs-Arbeitspakete für Entwickler

1. **`Cargo.toml` Bereinigung:**
   - Bereinigen aller `features = ["workspace"]` bei `contextra-store`, `contextra-vector`, `contextra-text`, `contextra-rank`, `contextra-graph`.
   - `contextra-testkit` aus `[dev-dependencies]` entfernen (R2).
   - `default = []` setzen (R3).
   - `ring = "2"` in metadata setzen.
2. **Altlasten-Bereinigung (`AP-07` bis `AP-12`):**
   - AP-07: `merge_operator_adapter.rs` leeren und Stub hinterlegen.
   - AP-08: `volatile_vault.rs` leeren.
   - AP-09: `builder.rs` auf `PidLatencyController` umstellen.
   - AP-10: `lifecycle.rs` ONNX-Block entfernen.
   - AP-11/12: Kommentare bereinigen.
3. **ReadGate-Prüfpunkte auditieren:**
   - Verifizieren, dass alle 8 Stellen aus SPEC §6.2 vor dem Zugriff den synchronen Gate-Check durchführen.
4. **Verifikation:**
   - `cargo test -p contextra-db`
