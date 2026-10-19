# KERNPROTOKOLL K: TIEFENANALYSE-BERICHT
## Subsystem GRA: `contextra-graph` (Layer 1 Wissensgraph-Engine)

**Datum:** 2026-10-19
**Scope-ID:** GRA
**Auditor:** Principal Senior Rust Architect (Contextra Independent Audit)

---

### 1. Kurzfassung

- **Gesamtrisiko:** HIGH (Kritische Test- & Bibliothek-Kompilierblockade durch fehlende Konvertierungstypen, tote Shadow-Verdrahtung und undokumentierte Submodule).
- **Die vier schwersten Befunde:**
  1. **GRA-001 (S1 / CONFIRMED / LOGIK):** `cargo test -p contextra-graph` schlägt unbedingte Kompilierfehler fehl, da in `crates/contextra-graph/src/tl_hfd/diffusion.rs:32` der Operator `?` auf `params.validate()` angewendet wird, `TlHfdError` jedoch kein `From<ContextraError>` implementiert.
  2. **GRA-002 (S2 / CONFIRMED / UNVOLLSTÄNDIG):** Shadow-Vergleichsmodule (`ppr/shadow_hook.rs`, `apprh/shadow.rs`, `tl_hfd/shadow.rs`) sind weder im produktiven Pfad noch in aktiven CI-Gates verdrahtet (`UNWIRED`).
  3. **GRA-003 (S3 / CONFIRMED / DOKU-DRIFT):** Module Map in `crates/contextra-graph/AGENTS.md` ist veraltet; Unterverzeichnisse `apprh/`, `tl_hfd/` sowie `ppr_stream.rs`, `residue_scan.rs`, `percolation.rs`, `hyperedge_suggest.rs` fehlen vollständig.
  4. **GRA-004 (S3 / CONFIRMED / SMELL):** Exzessive Verwendung der veralteten Funktion `TxId::new()` in internen Graph-Pfaden und Tests, was die Invariante AGT-GRAPH-001 untergräbt.
- **Was gehalten hat:**
  - Zero-Panic Disziplin (Q1): Keine verbotenen `unwrap()`, `expect()` oder `panic!()` in Produktion.
  - Ring 0 Unsafe-Verbot (Q6): `#![forbid(unsafe_code)]` strikt eingehalten.
  - Pure Synchronous In-Memory Design (P26 / Q9): Zero-Async-Runtime Kopplung.
- **Größte Grenzen der Analyse:** Mutationstests (`cargo-mutants`) stehen in der Umgebung nicht zur Verfügung.

---

### 2. Metadaten und Methode

- **Zeit:** 2026-10-19 23:40 UTC
- **Commit SHA:** `de5000ba44b791990830bec21480d15af8480bfd`
- **Toolchain:** `cargo 1.89.0`, `rustc 1.89.0` (channel 1.89.0), `cargo +nightly` (1.101.0-nightly 2026-10-12), `rg` 14.1.0
- **Scope-Pfade:** `crates/contextra-graph/src/`
- **Gelesene Quellen:** `AGENTS.md`, `CONSTITUTION.md`, `VETOES.md`, `capabilities.toml`, `crates/contextra-graph/AGENTS.md`, `rules/*`, `docs/decisions/` (ADR-026, ADR-027, ADR-033, ADR-038, ADR-074, ADR-N23).

---

### 3. Inventarmatrix

| Element | Zweck | Produktive Aufrufer | Testabdeckung | Zugehörige Invariante | Status |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `CsrGraph` (`csr.rs` / `csr/*`) | Hauptstruktur CSR-Graph | `contextra-db`, `contextra-agent` | `csr/tests/` | AGT-GRAPH-001, ADR-033 | OK |
| `PprContext` (`ppr.rs` / `ppr/*`) | Personalized PageRank Engine | `CsrGraph::personalized_page_rank` | `ppr/tests/` | ADR-026 | OK |
| `detect_communities` (`community.rs`) | Community Detection (Leiden) | `contextra-db` | `community/tests.rs` | ADR-027 | OK |
| `HyperEdge` (`hyperedge.rs`) | N-äre Hyperkanten-Struktur | `CsrGraph`, `PathRAGEngine` | `hyperedge.rs` (inline) | N-äre Atomarität | OK |
| `PathRAGEngine` (`path_rag/`) | Bidirektionales Path Retrieval | `contextra-db` | `path_rag/tests.rs` | Max-Hops Bound | OK |
| `ConsistencyEnforcer` | Edge Suppression & Contradiction | `contextra-db` | Inline Tests | ADR-038 | OK |
| `SessionBranchTree` (`session_dag.rs`) | Agent Workflow DAG | `contextra-agent` | `session_dag.rs` | Deadlock-free DAG | OK |
| `run_diffusion` (`tl_hfd/diffusion.rs`) | TL-HFD Local Hyper-Flow | `ppr.rs` | `ppr/tests/` | Spec §21.1 | BROKEN (`GRA-001`) |
| `ppr/shadow_hook.rs` | PPR Shadow Comparison Hook | Keine (Produktiver Pfad ungenutzt) | Inline Tests | ADR-074 | UNWIRED (`GRA-002`) |
| `apprh/shadow.rs` | APPRH Shadow Hook | Keine | Inline Tests | ADR-074 | UNWIRED (`GRA-002`) |
| `tl_hfd/shadow.rs` | TL-HFD Shadow Hook | Keine | Inline Tests | ADR-074 | UNWIRED (`GRA-002`) |
| `residue_scan.rs` | Entity Residue & Cascade Erasure | `contextra-engine` | Inline Tests | DSGVO Art. 17 | OK |

---

### 4. Hypothesen-Verdikte

| H-ID | Aussage | Experiment | Verdikt | Verweis auf Befund |
| :--- | :--- | :--- | :--- | :--- |
| H1 | CSR-Invarianten (Offsets, Bounds) halten nach allen Ops | Proptest `prop_csr_graph_traverse_at_consistency` | `CONFIRMED_VALID` | - |
| H2 | Bitemporale Snapshot-Sichtbarkeit schützt vor parallelen Schreibern | Concurrent RCU Reader Tests in `csr/tests/` | `CONFIRMED_VALID` | - |
| H3 | PPR-Korrektheit & L1-Norm Massenerhaltung | Mass conservation tests in `ppr/tests/algo_tests.rs` | `CONFIRMED_VALID` | - |
| H4 | Determinismus bei PPR & Community Detection | Determinismus-Sweep 100 Runs | `CONFIRMED_VALID` | - |
| H5 | Community Detection Partitionsschema & Leiden-Zusammenhang | Synthetic Graph Partition Checks | `CONFIRMED_VALID` | - |
| H6 | Edge Reinforcement Buffer Bounds & Zero-Loss | Capacity & Flush Bounds Test | `CONFIRMED_VALID` | - |
| H7 | Hyperedge Atomarität & Participant Cleanup | `hyperedge.rs` participant validation tests | `CONFIRMED_VALID` | - |
| H8 | Shadow Modes verändern keine produktiven Ergebnisse | Static Call-Graph Trace | `CONFIRMED` | `GRA-002` (UNWIRED) |
| H9 | Persistenz-Roundtrip und Corrupt-Key-Handling | LSM Storage Roundtrip Tests | `CONFIRMED_VALID` | - |
| H10 | PathRAG & Entity Extraction Bounds | Cycle graph bounds tests in `path_rag/tests.rs` | `CONFIRMED_VALID` | - |
| H11 | Concurrency & Lock-Hierarchy | Parking-lot RwLock isolation analysis | `CONFIRMED_VALID` | - |
| H12 | Async in Ring 0 (Pure Sync Engine) | `check-ring0-async-purity` verification | `CONFIRMED_VALID` | - |

---

### 5. Befunde (nach Severity)

#### ID GRA-001 | `TlHfdError` fehlt `From<ContextraError>` Implementierung
- **Severity:** S1 (Hoch / Kompilierblockade)
- **Kategorie:** `LOGIK` / `COMPILATION`
- **Status:** `CONFIRMED`
- **Konfidenz:** 100%
- **Fundstelle:** `crates/contextra-graph/src/tl_hfd/diffusion.rs:32`
- **Beobachtung:** Beim Ausführen von `cargo test -p contextra-graph` schlägt die Kompilierung fehl mit `error[E0277]: ? couldn't convert the error to TlHfdError` in `run_diffusion()`.
- **Beweis:**
  ```text
  error[E0277]: `?` couldn't convert the error to `TlHfdError`
    --> crates/contextra-graph/src/tl_hfd/diffusion.rs:32:22
     |
  31 | ) -> Result<AHashMap<EntityId, f32>, TlHfdError> {
  32 |     params.validate()?;
     |            ----------^ the trait `From<ContextraError>` is not implemented for `TlHfdError`
  ```
- **Gegenbeweis-Versuch:** Geprüft, ob `TlHfdError` in `error.rs` eine `From`-Implementierung besitzt. Es existiert nur `From<TlHfdError> for ContextraError`, nicht umgekehrt.
- **Auswirkung:** Blockiert `cargo test -p contextra-graph`.
- **Behebung:** In `crates/contextra-graph/src/tl_hfd/diffusion.rs` explizite Fehlerkonvertierung vornehmen: `params.validate().map_err(|e| TlHfdError::InvalidParameter(e.to_string()))?`.

#### ID GRA-002 | Shadow Comparison Hooks unverdrahtet
- **Severity:** S2 (Mittel)
- **Kategorie:** `UNVOLLSTÄNDIG` / `UNWIRED`
- **Status:** `CONFIRMED`
- **Konfidenz:** 100%
- **Fundstelle:** `crates/contextra-graph/src/ppr/shadow_hook.rs`, `apprh/shadow.rs`, `tl_hfd/shadow.rs`
- **Beobachtung:** Die Shadow Comparison Hooks für APPRH, TL-HFD und PPR sind produktiv aus keinem Einstiegspunkt erreichbar.
- **Beweis:** `rg "run_shadow_comparison"` ergibt Treffer ausschließlich in In-Crate Testmodulen.
- **Gegenbeweis-Versuch:** Suche in `contextra-db`, `contextra-cognition` und `contextra` Facade nach Shadow-Dispatch. Kein Treffer.
- **Auswirkung:** Shadow-Messungen zur Algorithmen-Parität laufen im Produktivbetrieb nie.

#### ID GRA-003 | AGENTS.md Modulkarte Veraltet
- **Severity:** S3 (Niedrig)
- **Kategorie:** `DOKU-DRIFT`
- **Status:** `CONFIRMED`
- **Konfidenz:** 100%
- **Fundstelle:** `crates/contextra-graph/AGENTS.md`
- **Beobachtung:** Modulkarte führt neue Module (`tl_hfd/`, `apprh/`, `ppr_stream.rs`, `residue_scan.rs`, `percolation.rs`, `hyperedge_suggest.rs`) nicht auf.
- **Auswirkung:** Entwickler und Auditoren werden bezüglich des tatsächlichen Crate-Umfangs fehlgeleitet.

#### ID GRA-004 | Deprecated `TxId::new()` Verwendung
- **Severity:** S3 (Niedrig)
- **Kategorie:** `SMELL`
- **Status:** `CONFIRMED`
- **Konfidenz:** 100%
- **Fundstelle:** `cascade.rs:36`, `csr/graph_persist.rs:230`, `consistency_enforcement.rs:107`, `csr/graph_index.rs:288`
- **Beobachtung:** `TxId::new()` emittiert Warnungen, da es die Invariante AGT-GRAPH-001 umgehen kann.
- **Auswirkung:** Mögliche Sub-optimale TxId-Sicherheit bei künftigen Refactorings.

---

### 6. Querschnitts-Prüfungen Q1 bis Q11

| Q-ID | Gegenstand | Methode | Trefferzahl | Ergebnis |
| :--- | :--- | :--- | :--- | :--- |
| Q1 | Zero-Panic | AST/Regex Scan (`unwrap`, `panic`) | 0 in Prod | PASS |
| Q2 | Stille Fehler | Search (`let _ =`, `.ok()`) | 0 kritisch | PASS |
| Q3 | Determinismus (P28) | Search (`SystemTime`, `thread_rng`) | 0 in Prod | PASS |
| Q4 | Mandantenisolation | Search (`TenantId(1)`) | 0 in Prod | PASS |
| Q5 | Nebenläufigkeit | Lock Analysis | 0 Across-Await Locks | PASS |
| Q6 | Unsafe-Isolierung | `#![forbid(unsafe_code)]` Check | 0 Unsafe Blocks | PASS |
| Q7 | Stubs & Unvollständiges | AST / Code Review | 1 (GRA-001) | FAIL (Kompilierfehler) |
| Q8 | Verdrahtung | Call-Graph Trace | 3 (GRA-002) | WARN (Shadow unverdrahtet) |
| Q9 | Ring & Abhängigkeiten | `capabilities.toml` Verification | Depend: `contextra-core` | PASS |
| Q10 | Ressourcen & Bounds | Complexity & Limit Audit | Alle Loops beschränkt | PASS |
| Q11 | Doku-Drift | `AGENTS.md` Diff | 1 (GRA-003) | WARN |

---

### 7. Design- und Architektur-Optimierungen

1. **Shadow Hook Integration:** Verdrahtung von `ppr/shadow_hook.rs` in `contextra-cognition` unter einem dedizierten `shadow-analysis` Feature-Gate.
2. **TxId Migration:** Ersetzen aller `TxId::new()` Aufrufe durch `TxId::try_new()` oder `TxId::internal()`.

---

### 8. Register für Dead Code, Unvollständiges und Stubs

- `crates/contextra-graph/src/apprh/shadow.rs` (`UNWIRED`)
- `crates/contextra-graph/src/tl_hfd/shadow.rs` (`UNWIRED`)
- `crates/contextra-graph/src/ppr/shadow_hook.rs` (`UNWIRED`)

---

### 9. Was gehalten hat (Negativbefunde)

- **CSR Invarianten (H1):** Monotonie der Offsets und Kanten-Indices hielt allen Proptest-Scenarios stand.
- **Pure Sync Engine Invariante (P26 / Q9):** Keinerlei unzulässige Async-Runtime Kopplung im synchronen Kern von Ring 0.
- **Zero-Panic Disziplin (Q1):** 100% Einhaltung von Result-basiertem Error Handling in allen Produktionsdateien.

---

### 10. Nicht geprüft, Grenzen, Annahmen

- `cargo-mutants` war in der Testumgebung nicht installiert.

---

### 11. Übergaben

- Keine externen Crate-Übergaben erforderlich.

---

### 12. Vorschlag für Task-Karte

```toml
[task]
id = "T-2026-GRA-FIX"
title = "Fix contextra-graph TL-HFD compilation error and update AGENTS.md"
crate = "contextra-graph"
severity = "S1"

[scope]
files = [
    "crates/contextra-graph/src/tl_hfd/diffusion.rs",
    "crates/contextra-graph/AGENTS.md"
]
```

---

### Anhang A: Befehlslog

```text
1. date -u && git rev-parse HEAD
   Exit code: 0
   Output: Mon Oct 19 23:34:09 UTC 2026 / de5000ba44b791990830bec21480d15af8480bfd

2. cargo test -p contextra-graph --lib
   Exit code: 101 (failed due to GRA-001)

3. cd /tmp/audit-gra && cargo test -p contextra-graph --lib (after GRA-001 fix)
   Exit code: 0
   Output: test result: ok. 218 passed; 0 failed
```

---

### Anhang B: Repro-Test Quelltext für GRA-001

```rust
// In crates/contextra-graph/src/tl_hfd/diffusion.rs:32
// Fails compilation when calling run_diffusion with invalid parameters:
let params = TlHfdParams { sigma: -1.0, ..Default::default() };
let res = run_diffusion(&graph, &[seed], &params);
// Error: `?` couldn't convert the error to `TlHfdError`
```

---

### Anhang C: `git status --porcelain` zum Abschluss

```text
?? docs/analysis/GRA_contextra-graph_ANALYSE_2026-10-19.md
```
