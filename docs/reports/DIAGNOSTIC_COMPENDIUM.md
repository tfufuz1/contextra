# Contextra Workspace Diagnostic Compendium & Systemzustandsbericht

> **Dokument-Status:** Autoritatives Referenzkompendium für Principal- & Core-Engineers
> **Gültigkeitsbereich:** Gesamter Contextra-Workspace (`crates/*`, `tools/*`, `tests/*`)
> **Basis-Quellen:** Alle 60 Diagnose- und Auditberichte in `docs/diagnostics/` (32 Markdown-Reports, 28 JSON-Datensätze)
> **Architektur-Referenzen:** SPEC 20261106, CONSTITUTION, VETOES, ARCHITECTURE, TYPE_REGISTRY
> **Fokus-Disziplinen:** Systemprogrammierung, Durability & Crash Consistency, Concurrency & Async Safety, RAG-Speicherkerne, Zero-Panic, Determinismus

---

## Inhaltsverzeichnis

1. [Executive Summary & Gesamtmetriken](#1-executive-summary--gesamtmetriken)
2. [Quellenverzeichnis der Diagnoseberichte](#2-quellenverzeichnis-der-diagnoseberichte)
3. [Durability, Fsync & Storage-Integrität (INV-FSYNC-1 / M4 / D-55)](#3-durability-fsync--storage-integrität-inv-fsync-1--m4--d-55)
4. [Concurrency, Async Boundaries & Locking-Architektur (INV-LOCK-ORDER-1 / M8)](#4-concurrency-async-boundaries--locking-architektur-inv-lock-order-1--m8)
5. [Memory Safety, Unsafe Boundaries & SIMD-Inseln (M9 / AR-8 / D-29)](#5-memory-safety-unsafe-boundaries--simd-inseln-m9--ar-8--d-29)
6. [Zero-Panic-Disziplin & Unsafe Indexing (P6 / Zero-Panic-Vertrag)](#6-zero-panic-disziplin--unsafe-indexing-p6--zero-panic-vertrag)
7. [Determinismus-Garantien & PRNG-Architektur (P28 / D-06)](#7-determinismus-garantien--prng-architektur-p28--d-06)
8. [Type Registry, Namenskollisionen & Port-Verträge (K3 / A2 / M3 / M7)](#8-type-registry-namenskollisionen--port-verträge-k3--a2--m3--m7)
9. [Anti-Pattern-Diagnostik & Verbotene Wortlaute (§6.4 / D-94 / D-107)](#9-anti-pattern-diagnostik--verbotene-wortlaute-64--d-94--d-107)
10. [Löschdomänen-Entkopplung & Erasure Leaks (M7 / WP-3.5 / Phase 3)](#10-löschdomänen-entkopplung--erasure-leaks-m7--wp-35--phase-3)
11. [Error-Topologie & Datenleck-Prävention (INV-LOG-1 / HR-4)](#11-error-topologie--datenleck-prävention-inv-log-1--hr-4)
12. [Dead Code, verwaiste Symbole & Deprecated Debt (M2 / Ratchet)](#12-dead-code-verwaiste-symbole--deprecated-debt-m2--ratchet)
13. [Test-Matrix, Testpyramide & Spezifikations-Traceability (T-TRACE-1 / AGENTS.md §8)](#13-test-matrix-testpyramide--spezifikations-traceability-t-trace-1--agentsmd-8)
14. [AST-Inventar & Kern-Crate-Kontext-Profile](#14-ast-inventar--kern-crate-kontext-profile)
15. [Konsolidierte Remediation-Roadmap & Architektur-Empfehlungen](#15-konsolidierte-remediation-roadmap--architektur-empfehlungen)

---
## 1. Executive Summary & Gesamtmetriken

Die durchgeführten 10 autonomen Diagnose- und statischen Analyse-Pipelines über alle 37 Crates des Contextra-Workspaces (955 Rust-Quelldateien mit insgesamt 13.395 Items) zeichnen ein detailliertes Bild der Codebase.

Die Architektur befindet sich im Übergang von einer monolithischen Historie hin zu einem strikten Schichtenmodell nach **SPEC §4.4 (Ring 0 bis Ring 4)**:
- **Ring 0 (Basis):** `contextra-types`, `contextra-ports`, `contextra-sys`
- **Ring 1 (Kerne):** `contextra-crypto`, `contextra-simd`, `contextra-durable-fs`, `contextra-mvcc`, `contextra-checkpoint`, `contextra-kvcache`
- **Ring 2 (Indizes & Ranking):** `contextra-store`, `contextra-vector`, `contextra-text`, `contextra-rank`
- **Ring 3 (Orchestrierung & Löschdomäne):** `contextra-engine`, `contextra-erasure`, `contextra-db`
- **Ring 4 (Fassade, Bindings & Ingestion):** `contextra`, `contextra-py`, `contextra-ingest`, `contextra-router`, `contextra-mcp`

### Konsolidierte Workspace-Kennzahlen auf einen Blick

| Kern-Dimension | Gemessener Wert | Zielwert / Spezifikations-Vorgabe | Status / Kritikalität |
|---|---|---|---|
| **Gescannte Rust-Dateien / Items** | 955 Dateien / 13.395 Items | Reines Schichten-Modell | Kontrolliert |
| **Direkte I/O-Zugriffe (`std::fs` / `tokio::fs`)** | 269 Fundstellen in Produktion | 0 (Nur via `contextra-ports::Vfs`) | **KRITISCH (INV-FSYNC-1)** |
| **Verschluckte Sync-Ergebnisse (`let _ = sync_*`)** | 2 Fundstellen in Produktion | 0 (D-55: Sync-Fehler müssen zwingend fehlschlagen) | **BLOCKER (D-55)** |
| **Lock-Across-Await Gefahrenstellen** | 191 Fundstellen (77 in Store) | 0 (INV-LOCK-ORDER-1 / Kein Deadlock-Risiko) | **HOCH (M8)** |
| **Synchrone Blockaden (`block_on` in Runtime)** | 66 Fundstellen in Produktion | 0 (Gefahr des Worker-Thread-Starvation) | **HOCH (M8)** |
| **Unsafe-Code-Blöcke (Gesamt-Workspace)** | 335 Blöcke (269 SIMD, 21 SYS, 45 WIRE) | Strikt isoliert in 3 Unsafe-Inseln | **EINGEHALTEN (M9)** |
| **Fehlende `// SAFETY:` Beweiskommentare** | 25 Stellen in `contextra-simd` | 0 (Jeder Unsafe-Block erfordert expliziten Beweis) | **MITTEL** |
| **Verstöße gegen `#![forbid(unsafe_code)]`** | 2 Crates (`contextra-text`, `contextra-graph`) | 0 außerhalb Unsafe-Inseln | **MITTEL (Testcode-Isolierung)** |
| **Zero-Panic Verletzungen in Produktion** | 520 (davon 455 in Tests/Fuzz/Examples, 65 echt) | 0 Panics/Unwraps in Produktivcode (P6) | **HOCH (P6)** |
| **Nicht-deterministische Primitiven (P28)** | 332 Fundstellen (119 HashSet, 155 HashMap) | 0 ungeseedete Hasher in Kernpfaden | **HOCH (P28)** |
| **Typnamen-Kollisionen im Workspace** | 239 Typnamen in mehreren Crates deklariert | Eindeutige Deklaration in `types`/`ports` | **MITTEL (K3 / A2)** |
| **Fassaden-Leaks (M3-Verletzungen)** | 24 Re-Exporte unautorisierter Subcrates | 0 (Re-Exporte nur über autorisierte Ports) | **HOCH (M3)** |
| **Löschdomänen-Leaks (M7-Verletzungen)** | 113 DSGVO-/Lösch-Referenzen in Kernen | 0 außerhalb `contextra-erasure` | **HOCH (M7 / WP-3.5)** |
| **Fehlende Spezifikations-Traceability** | Nur 160 von 5.150 Tests mit SPEC-Trace (3.1%) | 100% Traceability (T-TRACE-1) | **HOCH (T-TRACE-1)** |
| **Verbotene Wortlaute (§6.4)** | 124 Vorkommen ("garantiert", "DSGVO-konform") | 0 unqualifizierte Compliance-Claims | **MITTEL (§6.4)** |

---
## 2. Quellenverzeichnis der Diagnoseberichte

Alle Befunde in diesem Kompendium basieren auf den 60 erzeugten Diagnose-Dateien im Verzeichnis `docs/diagnostics/`:

### 2.1 Gesamtreports & Tool-Zusammenfassungen
- [`DIAGNOSTIC_SUMMARY.md`](DIAGNOSTIC_SUMMARY.md): Übersicht der 10 Diagnose-Module und Schnellaufrufe.
- [`ast_inventory_workspace.md`](ast_inventory_workspace.md): Vollständiges Inventar aller 13.395 Items über 955 Rust-Dateien.
- [`dependency_graph_workspace.md`](dependency_graph_workspace.md) / [`dependency_graph_workspace.json`](dependency_graph_workspace.json): Überprüfung der Ring-Grenzen (M1/M6).

### 2.2 Durability & I/O-Audits
- [`FSYNC_DURABILITY_AUDIT.md`](FSYNC_DURABILITY_AUDIT.md) / [`fsync_durability_audit.json`](fsync_durability_audit.json): 269 Raw-FS-Funde, Swallowed Syncs, Fsync-Tracking.

### 2.3 Nebenläufigkeit & Async-Sicherheit
- [`ASYNC_BOUNDARY_REPORT.md`](ASYNC_BOUNDARY_REPORT.md) / [`async_boundary_report.json`](async_boundary_report.json): 4.557 Async-Items, Lock-Across-Await, Synchronitätsmandate.
- [`concurrency_audit_workspace.md`](concurrency_audit_workspace.md) / [`concurrency_audit_workspace.json`](concurrency_audit_workspace.json): 646 Concurrency-Items, 191 Lock-across-await, 101 Loom-Punkte.
- [`concurrency_audit_contextra-store.md`](concurrency_audit_contextra-store.md) / [`concurrency_audit_contextra-store.json`](concurrency_audit_contextra-store.json): 216 Concurrency-Befunde im LSM-Speicherkern.

### 2.4 Speichersicherheit & Unsafe
- [`UNSAFE_BOUNDARY_AUDIT.md`](UNSAFE_BOUNDARY_AUDIT.md) / [`unsafe_boundary_audit.json`](unsafe_boundary_audit.json): Überprüfung von `#![forbid(unsafe_code)]` und Unsafe-Inseln.
- [`unsafe_tracer_workspace.md`](unsafe_tracer_workspace.md) / [`unsafe_tracer_workspace.json`](unsafe_tracer_workspace.json): 335 Unsafe-Blöcke, 25 fehlende Beweise, 34 Raw-Pointer-Operationen.
- [`unsafe_tracer_contextra-simd.md`](unsafe_tracer_contextra-simd.md) / [`unsafe_tracer_contextra-simd.json`](unsafe_tracer_contextra-simd.json): Detailanalyse der SIMD-Distanzkerne.

### 2.5 Zero-Panic & Robustheit
- [`ZERO_PANIC_AUDIT.md`](ZERO_PANIC_AUDIT.md) / [`zero_panic_audit.json`](zero_panic_audit.json): 520 Produktions- und 26.242 Test-Fundstellen von Panic/Unwrap/Expect.

### 2.6 Determinismus & PRNG
- [`DETERMINISM_GUARD_AUDIT.md`](DETERMINISM_GUARD_AUDIT.md) / [`determinism_guard_audit.json`](determinism_guard_audit.json): 332 Fundstellen von SipHash, SystemTime, Instant und ungeseedetem RNG.

### 2.7 Verträge, Typ-Register & Schnittstellen
- [`TYPE_REGISTRY_COLLISIONS.md`](TYPE_REGISTRY_COLLISIONS.md) / [`type_registry_collisions.json`](type_registry_collisions.json): 239 kollidierende Typnamen, 3.488 registrierte Typen.
- [`PORT_CONTRACT_AUDIT.md`](PORT_CONTRACT_AUDIT.md) / [`port_contract_audit.json`](port_contract_audit.json): 24 M3-Fassadenleaks, Holder-Domain-Validierung.
- [`contract_diff_workspace.md`](contract_diff_workspace.md) / [`contract_diff_workspace.json`](contract_diff_workspace.json): Drift-Analyse für 45 Port-Traits und 121 Implementierungen.

### 2.8 Anti-Pattern & Invarianten
- [`pattern_diagnostic_workspace.md`](pattern_diagnostic_workspace.md): 12.311 Workspace-Befunde (Zero-Panic, Determinismus, Mmap, Direct I/O, §6.4).
- [`pattern_diagnostic_contextra-types.md`](pattern_diagnostic_contextra-types.md) / [`pattern_diagnostic_contextra-types.json`](pattern_diagnostic_contextra-types.json): 225 Befunde in `contextra-types`.

### 2.9 Löschdomäne & DSGVO-Entkopplung
- [`erasure_leaks_workspace.md`](erasure_leaks_workspace.md) / [`erasure_leaks_workspace.json`](erasure_leaks_workspace.json): 181 Befunde zu DeletionProof, DSGVO, Saga in Speicherkernen.

### 2.10 Fehlertopologie & Logging
- [`ERROR_TOPOLOGY_REPORT.md`](ERROR_TOPOLOGY_REPORT.md) / [`error_topology_report.json`](error_topology_report.json): Data-Leak-Prüfung nach INV-LOG-1.
- [`error_topology_workspace.md`](error_topology_workspace.md) / [`error_topology_workspace.json`](error_topology_workspace.json): 43 Fehler-Enums, 312 Varianten, 42 Konvertierungen.
- [`error_topology_contextra-types.md`](error_topology_contextra-types.md) / [`error_topology_contextra-types.json`](error_topology_contextra-types.json): Topologie des kanonischen `ContextraError`.

### 2.11 Toter Code & Altlasten
- [`DEAD_CODE_ANALYSIS.md`](DEAD_CODE_ANALYSIS.md) / [`dead_code_analysis.json`](dead_code_analysis.json): Workspace-weite Analyse ungenutzter Symbole.
- [`dead_code_contextra-types.md`](dead_code_contextra-types.md) / [`dead_code_contextra-types.json`](dead_code_contextra-types.json): 43 verwaiste Symbole in `types`.
- [`dead_code_contextra-ports.md`](dead_code_contextra-ports.md) / [`dead_code_contextra-ports.json`](dead_code_contextra-ports.json): 28 Befunde (inkl. 8 `#[deprecated]` Relikten).

### 2.12 Test-Matrix & Pyramide
- [`TEST_MATRIX_ADVISOR.md`](TEST_MATRIX_ADVISOR.md) / [`test_matrix_advisor.json`](test_matrix_advisor.json): Pyramiden-Empfehlungen für 37 Crates.
- [`test_matrix_workspace.md`](test_matrix_workspace.md): Abdeckung von 5.150 Tests und T-TRACE-1 Quote.
- [`test_matrix_contextra-types.md`](test_matrix_contextra-types.md) / [`test_matrix_contextra-types.json`](test_matrix_contextra-types.json): Traceability-Analyse in `types`.

### 2.13 Crate-Kontext-Packs (Agentic Context)
- [`context_pack_contextra-types.md`](context_pack_contextra-types.md) / [`.json`](context_pack_contextra-types.json)
- [`context_pack_contextra-ports.md`](context_pack_contextra-ports.md) / [`.json`](context_pack_contextra-ports.json)
- [`context_pack_contextra-store.md`](context_pack_contextra-store.md) / [`.json`](context_pack_contextra-store.json)
- [`context_pack_contextra-engine.md`](context_pack_contextra-engine.md) / [`.json`](context_pack_contextra-engine.json)
- [`context_pack_contextra-erasure.md`](context_pack_contextra-erasure.md) / [`.json`](context_pack_contextra-erasure.json)

---
## 3. Durability, Fsync & Storage-Integrität (INV-FSYNC-1 / M4 / D-55)

### 3.1 Spezifikatorische Grundlagen
- **INV-FSYNC-1 / M4:** Jeder Schreibvorgang auf persistente Speicherbereiche (WAL, Manifest, SSTable, Journal, Vault, Checkpoints) muss über die `contextra-ports::Vfs` Abstraktion erfolgen. Direkte Aufrufe von `std::fs` oder `tokio::fs` sind im gesamten Workspace außerhalb von `contextra-durable-fs` und `contextra-sys` strikt untersagt.
- **D-55 (Fatal Fsync Errors):** Dateisystem-Synchronisationsfehler (`fsync`, `fdatasync`, `FlushFileBuffers`) sind nicht behebbar. Sie vergiften das betroffene Dateihandle (`SyncError`) und dürfen niemals ignoriert oder in `Ok(())` konvertiert werden (`swallowed_sync`).
- **D-83 (Atomic Write Pattern):** Persistente Mutationen müssen das Muster `tmp-Datei schreiben -> fsync(tmp) -> atomic rename -> fsync(parent_dir)` einhalten.

### 3.2 Audit-Befunde (Quantitativ & Qualitativ)

Aus `fsync_durability_audit.json` und `FSYNC_DURABILITY_AUDIT.md`:
- **Gesamtzahl direkter FS-Aufrufe in Produktion:** 269 Fundstellen.
- **Zulässige I/O-Crates:** Nur `contextra-durable-fs` (13 Funde) und `contextra-sys` (56 Funde) besitzen eine Ausnahmegenehmigung (`is_permitted_fs = True`).
- **Unzulässige Raw-FS Nutzung in Produktions-Crates (200 Funde):**
  1. `contextra-store` (105 Funde): Massive Umgehung des Vfs-Ports durch direkte Dateioperationen in Beispielen und Kernmodulen (`examples/chaos_writer.rs`, SSTable-Scanner).
  2. `contextra-vector` (43 Funde): Direkte Aufrufe in `src/diskann/types.rs` (`std::fs::write(&path, ...)`).
  3. `contextra-infer-onnx` (13 Funde) & `contextra-infer-candle` (11 Funde): Direkte Modul- und Weight-Ladevorgänge via `File::open` und `std::fs::create_dir_all`.
  4. `contextra` Fassade (6 Funde): Direkte `std::fs::remove_dir_all(&base_tmp)` Aufrufe in `src/lib.rs:187, 205` und `src/builder.rs:516`.
  5. `contextra-checkpoint` (5 Funde): Direkte Aufrufe `std::fs::write(&self.persist_path, data)` in `src/orphan.rs:251` und `src/hardlink_cloner.rs:62`.
  6. `contextra-mcp` (5 Funde): `std::fs::read` und `std::fs::read_to_string` in `src/config.rs:507` und `src/prompt_injection/guard.rs:189`.
  7. `contextra-license` (3 Funde): Direkte Dateioperationen in `src/signed_gate.rs:239-250`.
  8. `contextra-wire` (2 Funde): In `build.rs:132, 154`.
  9. `contextra-router` (1 Fund): `std::fs::read(path).ok()` in `src/router/lifecycle.rs:67`.

### 3.3 Kritische Swallowed Syncs (Verletzung von D-55)

Es wurden **2 fatale Stellen** identifiziert, an denen Rückgabewerte von Synchronisationsoperationen verworfen werden:
1. **`crates/contextra-crypto/src/revocation_log.rs:529`**:
   ```rust
   let _ = vfs.sync_dir(p);
   ```
   *Analyse:* Beim Anhängen an das Widerrufs-Log wird ein Verzeichnis-Fsync ignoriert. Bei einem Stromausfall direkt nach der Operation kann der Verzeichniseintrag verloren gehen, wodurch Widerrufe unbemerkt verschwinden. Dies verletzt Invariante D-55 und A3.
2. **`crates/contextra-infer-onnx/src/lib.rs:346`**:
   Ignorieren bzw. fehlerhaftes Mapping bei Modell-Temp-Downloads (`f.sync_all()`).

---
## 4. Concurrency, Async Boundaries & Locking-Architektur (INV-LOCK-ORDER-1 / M8)

### 4.1 Spezifikatorische Grundlagen
- **INV-LOCK-ORDER-1 / SPEC §5.4 & MV-08:** Strenge hierarchische Sperrordnung im Gesamtsystem:
  $$\text{Dokumentlock (kv\_locks)} \longrightarrow \text{Commit-Barriere} \longrightarrow \text{Purge-Barriere} \longrightarrow \text{Journal}$$
  Jede Umkehrung führt zu deterministischen Deadlocks.
- **Regel M8 (Async Safety & Concurrency):**
  1. Synchronitätsmandat (`is_sync_mandated`): Reine Rechen-, Typ- und Low-Level-Crates dürfen **keinerlei asynchronen Code** oder Tokio-Runtimes enthalten (`contextra-types`, `contextra-ports`, `contextra-crypto`, `contextra-simd`, `contextra-sys`, `contextra-mvcc`, `contextra-rank`, `contextra-wire`, `contextra-adapt`, `contextra-durable-fs`).
  2. Verbot von synchronen Sperren über `.await`-Punkten (`Lock held across await`): Führt zu Thread-Pool-Exhaustion und Deadlocks.
  3. Verbot von `block_on` innerhalb bestehender Tokio-Runtimes ohne `block_in_place`.

### 4.2 Audit-Befunde (Quantitativ & Qualitativ)

Aus `async_boundary_report.json`, `concurrency_audit_workspace.json` und `concurrency_audit_contextra-store.json`:
- **Gesamtzahl asynchroner Elemente im Workspace:** 4.557 Vorkommen (955 `async fn`, 2.635 `await`, 697 `tokio`-Imports).
- **Synchronitäts-Mandats-Verletzungen:**
  - `contextra-crypto` (17 Funde): Tokio-Imports und Async-Tests in `src/crypto.rs:882-896` sowie `block_on` in `src/kv_segment/store.rs:663`.
  - `contextra-ports` (109 Funde): Async-Methoden in `vector_index.rs` und `vfs.rs`. (Architektonisch notwendig durch dyn-kompatible Futures, erfordert jedoch präzise Kapselung).
  - `contextra-rank` (2 Funde): Async Driver-Funktion `fuse_exact_prefix_async` in `src/dibud/driver.rs:51`.
- **Verstöße gegen "Lock Held Across Await" (191 Fundstellen Gesamt):**
  - `contextra-store`: **94 Stellen** (z. B. `src/scanner/purge.rs:106, 113`, `src/lsm/engine.rs`).
  - `contextra-engine`: **40 Stellen** (z. B. `src/collection/kv_lock.rs:51, 59`).
  - `contextra-checkpoint`: **23 Stellen** (z. B. `src/store.rs:421, 425`).
  - `contextra-erasure`: **20 Stellen** (z. B. `src/saga/core.rs:268, 270`).
  - `contextra-vector`: **14 Stellen** (z. B. `src/diskann/vector_index_impl.rs:18, 25`).
  - `contextra-db`: **3 Stellen** (`src/lib.rs:396, 723`).
  - `contextra-kvcache`: **2 Stellen** (`src/service.rs:505, 506`).
- **Synchrone Blockaden (`block_on` in Produktion, 66 Fundstellen):**
  - `contextra-py` (33 Funde): PyO3 synchronisiert Python-Aufrufe über `rt.block_on(...)` in `src/kv_links.rs`.
  - `contextra-store` (9 Funde): `tokio::task::block_in_place(|| handle.block_on(fut))` und ungeschütztes `rt.block_on` in `src/scanner/sstable.rs:27, 36`.
  - `contextra-checkpoint` (5 Funde): `handle.block_on` in `src/store.rs:353, 365`.
  - `contextra-mcp` (3 Funde), `contextra-infer-candle` (3 Funde), `contextra-engine` (2 Funde).

### 4.3 Nebenläufigkeits-Modellprüfung mit Loom
- Der Workspace besitzt bereits **101 Loom-Verifikationspunkte** (davon 28 in `contextra-store`, 23 in `store/src/lib.rs` und `util.rs`, 17 in `crypto`, 9 in `mvcc`). Dies belegt eine solide Basis für Permutationstests, muss jedoch auf alle Synchronisationsprimitive ausgeweitet werden.

---
## 5. Memory Safety, Unsafe Boundaries & SIMD-Inseln (M9 / AR-8 / D-29)

### 5.1 Spezifikatorische Grundlagen
- **Regel M9 / AR-8 / Unsafe-Insel-Architektur:** Unsafe Rust ist im gesamten Workspace verboten (`#![forbid(unsafe_code)]`), mit exakt drei autorisierten Ausnahmen ("Unsafe Islands"):
  1. `contextra-simd` (Vektor-Distanzberechnungen, AVX2, AVX-512, NEON)
  2. `contextra-sys` (Betriebssystemnahe Primitiven: `mlock`, `fsync`, `acl_win32`, `posix`)
  3. `contextra-wire` (FlatBuffers Serialisierung & Zero-Copy Deserialisierung)
- **// SAFETY: Beweispflicht:** Jeder `unsafe`-Block, jede `unsafe fn` und jede `unsafe impl` muss von einem lückenlosen, überprüfbaren Kommentar mit dem Präfix `// SAFETY:` begleitet sein, der Vorbedingungen und Speicherlayout-Garantien darlegt.

### 5.2 Audit-Befunde (Quantitativ & Qualitativ)

Aus `unsafe_boundary_audit.json`, `UNSAFE_BOUNDARY_AUDIT.md`, `unsafe_tracer_workspace.json` und `unsafe_tracer_contextra-simd.json`:
- **Gesamtzahl Unsafe-Blöcke:** 335 Blöcke (plus Funktions- und Implementierungs-Deklarationen).
- **Status der Unsafe-Inseln:**
  - `contextra-simd`: 269 Unsafe-Blöcke, 18 Raw-Pointer-Operationen. **25 fehlende `// SAFETY:` Beweise** (vor allem in Intrinsics-Faltungen `_mm_add_epi32`, `_mm_cvtsi128_si32`).
  - `contextra-sys`: 21 Unsafe-Blöcke. **0 fehlende Beweise (100% Konformität)**. Alle POSIX- und Win32-Aufrufe sind vorbildlich mit `# Safety` und `// SAFETY:` dokumentiert.
  - `contextra-wire`: 45 Unsafe-Blöcke (FlatBuffer Table-Access). Keine fehlenden Beweise im generierten Kern, aber Heuristik-Warnungen in `build.rs` und Tests.
- **Unzulässige Unsafe-Vorkommen außerhalb der Inseln:**
  - `contextra-text` (5 Unsafe-Vorkommen in `tests/alloc_profiler.rs:27-48`): `unsafe impl GlobalAlloc for CountingAllocator`. Obwohl es sich um Testcode handelt, verletzt die Datei das Root-Attribut `#![forbid(unsafe_code)]`. Eine Ausnahme ist in `known_deviations.rs:148` registriert, sollte jedoch in ein eigenes Tooling-Crate ausgelagert werden.
  - `contextra-graph` (5 Unsafe-Vorkommen in Fuzzing/Testcode).

---
## 6. Zero-Panic-Disziplin & Unsafe Indexing (P6 / Zero-Panic-Vertrag)

### 6.1 Spezifikatorische Grundlagen
- **Regel P6 / Zero-Panic-Vertrag:** Im Produktionscode sind `panic!`, `unwrap()`, `expect()`, `todo!()`, `unimplemented!()` und ungeprüfte Indexzugriffe (`slice[idx]`) ausnahmslos verboten.
- Fehler müssen über `Result<T, ContextraError>` nach oben propagiert werden.
- Statt `[]` müssen `.get()` oder sichere Iteratoren verwendet werden.

### 6.2 Audit-Befunde (Quantitativ & Qualitativ)

Aus `zero_panic_audit.json` und `ZERO_PANIC_AUDIT.md`:
- **Gesamtfunde in Produktion laut Scanner:** 520 Vorkommen.
- **Bereinigte Analyse (Echter Produktionscode vs. Test/Example/Fuzz):**
  Der AST-Scanner erfasste Tests, Fuzz-Targets und Examples, die in `src/` lagen. Eine tiefere Filterung ergab:
  - **455 Funde** entfallen auf interne Modul-Tests (`tests.rs`), Fuzz-Harnesses und CLI-Examples.
  - **65 Funde** liegen in tatsächlichem Produktionscode:
    1. **`contextra-store` (4 Funde):**
       - `src/memtable.rs:48`: `const _: () = assert!(TOMBSTONE_BIT == 1u64 << 63);` (Compile-Time Assert, unkritisch zur Laufzeit).
       - `src/scanner/sstable.rs:35, 47`: `.expect("Failed to create tokio runtime for scanner");` (**KRITISCH:** Kann bei Thread-Erschöpfung den Prozess terminieren!).
       - `src/wal/hmac.rs:217`: `panic!("Legacy WAL key support is disabled or unavailable: {e}");` (**KRITISCH:** Unbehandelte Panik im Recovery-Pfad!).
    2. **`contextra-types` (2 Funde):**
       - `src/types/domain/ids.rs:506, 510`: Compile-Time `assert!` für ID-Basiswerte (unkritisch).
    3. **`contextra-vector` (1 Fund):**
       - `src/diskann/build.rs:759`: Laufzeit-`assert!` beim Index-Graphbau (**KRITISCH**).
    4. **`contextra-wire` (17 Funde):**
       - `src/contextra_generated.rs`: 16 unwrap-Aufrufe auf FlatBuffers-Feldern (`.unwrap()`). (Sollte durch fehlerbehandelnde Accessoren ersetzt werden).
       - `build.rs:46`: `assert!(status.success(), "flatc failed");` (Build-Skript).

---
## 7. Determinismus-Garantien & PRNG-Architektur (P28 / D-06)

### 7.1 Spezifikatorische Grundlagen
- **Regel P28 / Invariante D-06:** Determinismus im Kern.
  - Standard-`HashSet` und `HashMap` nutzen `RandomState` (SipHash-1-3 mit zufälligem Seed pro Prozessstart). Ihre Iterationsreihenfolge ist nicht-deterministisch. Dies führt zu nicht-reproduzierbaren Index-Layouts, Prüfsummen-Drifts und WAL-Replay-Inkonsistenzen.
  - In allen Kernen müssen `std::collections::BTreeSet` / `BTreeMap` oder `ahash::AHashSet` / `AHashMap` mit deterministischem Seed verwendet werden.
  - Zeitmessung und Zufall müssen über `contextra-ports::Clock` und `contextra-ports::Rng` injiziert werden. Direkte Aufrufe von `Instant::now()`, `SystemTime::now()` oder `rand::thread_rng()` sind in Ring 0 bis 3 verboten.

### 7.2 Audit-Befunde (Quantitativ & Qualitativ)

Aus `determinism_guard_audit.json` und `DETERMINISM_GUARD_AUDIT.md`:
- **Gesamtfunde in Produktion:** 332 Fundstellen.
  - `random_hashmap`: 155 Funde
  - `random_hashset`: 119 Funde
  - `instant_now`: 25 Funde
  - `env_var`: 24 Funde
  - `thread_rng`: 7 Funde
  - `system_time_now`: 2 Funde
- **Kritische Kern-Crates mit Verstößen:**
  - `contextra-graph` (151 Funde): Massive Nutzung von Standard-`HashMap`/`HashSet` bei Knoten- und Kanteniterationen.
  - `contextra-store` (29 Funde):
    - `src/system_pressure.rs:163`: `Instant::now()` ungemockt.
    - `src/scanner/sstable.rs:17, 117`: `use std::collections::HashSet;` und `let mut active_paths = HashSet::new();`. Iteration über aktive SSTable-Pfade ist zufällig geordnet!
  - `contextra-engine` (22 Funde):
    - `src/collection/maintenance.rs:172, 209, 816`: `HashSet<DocId>` und `HashMap<EntityId, u64>`. Führt zu nicht-deterministischer Wartungs- und Kompaktierungsreihenfolge.
  - `contextra-vector` (17 Funde): Ungeseedete Sets im HNSW/DiskANN Graph-Traversierungspfad.
  - `contextra-crypto` (1 Fund): `src/deletion_proof/creation.rs:65`: `HashSet::new()` bei Schichtenprüfung (`seen_layers`).
  - `contextra-ports` (3 Funde): `src/clock.rs:30, 45, 54`: Standard-Taktgeber nutzt direkt `Instant::now()` und `SystemTime::now()` (als Fallback-Implementierung des Clock-Ports zulässig, muss aber sauber dokumentiert sein).

---
## 8. Type Registry, Namenskollisionen & Port-Verträge (K3 / A2 / M3 / M7)

### 8.1 Spezifikatorische Grundlagen
- **Regel K3 / A2:** Kanonische Typen dürfen nur an einer einzigen Stelle im Workspace deklariert werden (primär in `contextra-types` bzw. `contextra-ports`). Schatten-Deklarationen ("Duplicate Domain Types") führen zu Typ-Verwechslungen, Serde-Inkompatibilitäten und Code-Aufblähung.
- **Regel M3 (Facade Leaks):** Keine unautorisierten Re-Exporte. Crates dürfen interne Typen abhängiger Crates nicht transparent re-exportieren, wenn dadurch Schichten übersprungen werden.
- **Port-Vertrags-Disziplin (M7):** Alle Schnittstellen müssen vollständige Dokumentationsblöcke `# Fehler` und `# Crash` aufweisen.

### 8.2 Audit-Befunde (Quantitativ & Qualitativ)

Aus `type_registry_collisions.json`, `PORT_CONTRACT_AUDIT.md` und `contract_diff_workspace.md`:
- **Typ-Register-Kollisionen:** 239 Typnamen kollidieren über Crate-Grenzen hinweg (bei 3.488 registrierten Typen).
- **Die gravierendsten Kollisionen:**
  1. `Timestamp`: Deklariert in 3 Crates mit inkompatiblen Repräsentationen:
     - `contextra-ports/src/observer.rs:15`: `pub type Timestamp = u64;`
     - `contextra-types/src/ids.rs:201`: `pub struct Timestamp(pub u64);` (Newtype mit Serde)
     - `contextra-erasure/src/types_local.rs:70`: `pub struct Timestamp(pub u64);` (Lokale Kopie!)
  2. `FileId`:
     - `contextra-types/src/ids.rs:103`: `pub struct FileId(pub [u8; 16]);` (128-Bit UUID-basiert)
     - `contextra-ports/src/keys.rs:12`: `pub struct FileId(pub u64);` (64-Bit Integer!)
     - `contextra-erasure/src/types_local.rs:384`: `pub struct FileId(pub u64);`
     *Gefahr:* Fataler Type-Mismatch bei Konvertierung zwischen Types und Ports/Erasure!
  3. `RangeSet`, `LedgerEntry`, `LedgerSink`, `ReportSigner`, `ReportSignature`, `SignerError`, `FileKey`, `SealCtx`, `Envelope`: Alle vollständig dupliziert zwischen `contextra-ports` und `contextra-erasure/src/types_local.rs`.
- **Fassaden-Leaks (M3-Verletzungen, 24 Funde):**
  - `contextra-db` (7 Leaks): `pub use contextra_engine::collection::{...}` in `src/lib.rs:44, 45`.
  - `contextra-engine` (4 Leaks): `pub use contextra_crypto::deletion_proof::{...}` und `pub use contextra_store::lsm::DurabilityMode` in `src/lib.rs:20, 30`.
  - `contextra-router` (6 Leaks): `pub use contextra_adapt::bandit;` und `lyapunov;` in `src/lib.rs:35, 40`.
- **Port-Vertrags-Dokumentation:**
  - 45 Port-Traits mit 104 Methoden.
  - 60 Methoden fehlt der Abschnitt `# Fehler`.
  - 63 Methoden fehlt der Abschnitt `# Crash`.

---
## 9. Anti-Pattern-Diagnostik & Verbotene Wortlaute (§6.4 / D-94 / D-107)

### 9.1 Spezifikatorische Grundlagen
- **SPEC §6.4 & Invariante T-LINT-1 (Verbotene Wortlaute):**
  Strenges Verbot absoluter Compliance- und Sicherheits-Versprechen im Produktcode, Dokumentationen und Kommentaren. Begriffe wie `"garantiert gelöscht"`, `"100%"`, `"DSGVO-konform"`, `"unwiderruflich"`, `"informationstheoretisch sicher"`, `"anonymisiert"` oder `"zertifiziert"` sind untersagt. Aussagen müssen nach Evidenz-Leveln (EV0–EV4) klassifiziert und Haftungstatbeständen (H1–H9) zugeordnet sein.
- **Invariante D-94 / WP-0.3 (Mmap-Verbot):**
  Memory-Mapping (`memmap2`, `mmap_readonly`) ist in Ring 2 Speicher- und Vektorkernen aus Crash-Sicherheits- und SIGBUS-Gründen untersagt. Gepufferte `Arc<[u8]>`-Puffer über Vfs sind zwingend.

### 9.2 Audit-Befunde (Quantitativ & Qualitativ)

Aus `pattern_diagnostic_workspace.md` (12.311 Fundstellen Gesamt):
- **Verbotene Wortlaute (§6.4):** 124 Vorkommen im Workspace.
  - `contextra/src/lib.rs:33` und `contextra/src/collection_profile.rs:149`: Verwendung des Begriffs `'garantiert'`.
  - `contextra-types/src/error.rs` und Domain-Dateien: 2 Vorkommen verbotener Schlagworte.
- **Memory-Mapping-Verstöße (110 Fundstellen):**
  - Vor allem Reste in Vektor- und Speicherkernen (`contextra-vector/src/diskann/types.rs:12: use memmap2::Mmap;`).

---
## 10. Löschdomänen-Entkopplung & Erasure Leaks (M7 / WP-3.5 / Phase 3)

### 10.1 Spezifikatorische Grundlagen
- **Regel M7 / Paket B31 / Phase 3 Architektur:**
  Strikte Domänentrennung. Die Begriffe und Logiken von DSGVO, DeletionProof, Lösch-Saga, Quorum-Zertifikaten und Audit-Exporten gehören **ausschließlich** in das Crate `contextra-erasure` (sowie Ring 4 Generatoren).
  Kerne wie `contextra-store`, `contextra-engine` oder `contextra-crypto` dürfen keine Rechtsbegriffe enthalten, sondern ausschließlich generische Primitiven (`Purger`, `Tombstone`, `KeyManager`, `revoke_key`).

### 10.2 Audit-Befunde (Quantitativ & Qualitativ)

Aus `erasure_leaks_workspace.md` (181 Befunde, davon 113 Kernel-Verletzungen):
- **Verletzende Crates:**
  - `contextra-crypto` (**66 Verletzungen**): Enthält nach wie vor Module `src/deletion_proof/`, `src/ed25519_proof.rs` und Verweise auf Löschbeweise. Diese müssen nach `contextra-erasure` migriert werden.
  - `contextra-engine` (**25 Verletzungen**): Enthält `contextra_impl/erasure.rs` (618 Zeilen) und 24 direkte `DeletionProof`-Bezüge.
  - `contextra-audit-export` (**20 Verletzungen**): Rechtsbezüge in Daten-Export-Modulen (`bsi_mapping.rs`, `error.rs`).
  - `contextra-store` (**2 Verletzungen**): Letzte Lösch- und Shredding-Begriffe in `kv/segment.rs` und `kv/delete_mode.rs`.
- **Bereits saubere Kern-Crates:**
  `contextra-types` (5 zulässige Referenzen), `contextra-ports` (1 zulässige Referenz).

---
## 11. Error-Topologie & Datenleck-Prävention (INV-LOG-1 / HR-4)

### 11.1 Spezifikatorische Grundlagen
- **INV-LOG-1 / HR-4:** Fehlermeldungen, Display- und Debug-Implementierungen dürfen **niemals sensitive Benutzerdaten, PII, Klartext-Schlüssel oder unzensierte Dateipfade** exponieren.
- Strukturierte Fehlerarchitektur via `thiserror`. Keine unstrukturierten String-Fehler (`Box<dyn Error>` oder formatierte Strings ohne Typisierung).

### 11.2 Audit-Befunde (Quantitativ & Qualitativ)

Aus `error_topology_workspace.md` und `ERROR_TOPOLOGY_REPORT.md`:
- **Fehler-Topologie:** 43 Error-Enums mit insgesamt 312 Varianten und 42 `From<E>` Konvertierungen.
- **Kanonischer Fehlerbaum:**
  - `contextra-types::ContextraError` bildet die Wurzel mit **69 Varianten** (von `Io`, `Crypto`, `Storage`, `WalCorruption`, bis `InvalidInput`).
  - `contextra-crypto::CryptoError` mit 18 Varianten.
  - `contextra-erasure::ErasureError` mit 21 Varianten.
- **Datenleck-Audit:**
  - Grundsätzlich hohe Disziplin bei Display-Implementierungen.
  - Zu überwachen: Pfadangaben in `contextra-types::error.rs:803` (`test_io_error_message_preserved`), um sicherzustellen, dass Pfade in Produktions-Logs anonymisiert bleiben.

---
## 12. Dead Code, verwaiste Symbole & Deprecated Debt (M2 / Ratchet)

### 12.1 Spezifikatorische Grundlagen
- **Regel M2 / Ratchet-Hygiene (`governance/ratchet.toml`):**
  Unbenutzter Code, `#[allow(dead_code)]`-Attribute und veraltete `#[deprecated]` Relikte müssen kontinuierlich reduziert werden. Neue Altlasten sind verboten.

### 12.2 Audit-Befunde (Quantitativ & Qualitativ)

Aus `DEAD_CODE_ANALYSIS.md`, `dead_code_contextra-types.md` und `dead_code_contextra-ports.md`:
- **Untersuchte Symbole im Workspace:** Über 5.000 Deklarationen.
- **Identifizierte Dead-Code Kandidaten (nur 1x im Crate referenziert):**
  - `contextra-erasure`: 66 Kandidaten
  - `contextra-types`: 43 Kandidaten (viele private Hilfsfunktionen in `domain/misc.rs`)
  - `contextra-ports`: 33 Kandidaten
  - `contextra-store`: 32 Kandidaten
  - `contextra-crypto`: 24 Kandidaten
  - `contextra-engine`: 22 Kandidaten
- **Deprecated-Relikte in `contextra-ports`:**
  - `src/erasure_legacy.rs`: 8 `#[deprecated]` Attribute, die unmittelbar entfernt werden können.

---
## 13. Test-Matrix, Testpyramide & Spezifikations-Traceability (T-TRACE-1 / AGENTS.md §8)

### 13.1 Spezifikatorische Grundlagen
- **T-TRACE-1 / SPEC §15.1:** Jeder Testfall muss explizit einer Spezifikations-ID zugeordnet sein (über `/// SPEC: <ID>` oder standardisierte Namensgebung wie `t_...`, `po_...`, `d_...`).
- **AGENTS.md §8 (Die Testpyramide):**
  Jeder kritische Kern erfordert fünf Test-Stufen:
  $$\text{Unit-Tests} \longrightarrow \text{Property-Tests (proptest)} \longrightarrow \text{Loom (Concurrency)} \longrightarrow \text{Chaos/FaultVfs} \longrightarrow \text{Criterion-Benchmarks}$$

### 13.2 Audit-Befunde (Quantitativ & Qualitativ)

Aus `test_matrix_workspace.md` und `TEST_MATRIX_ADVISOR.md`:
- **Gesamtzahl entdeckter Tests:** 5.150 Tests (3.024 Unit, 2.011 Tokio Async, 86 Proptests, 29 Benchmarks).
- **Spezifikations-Traceability (T-TRACE-1):**
  - **Nur 160 von 5.150 Tests (3.1%)** verfügen über ein formales SPEC-Trace!
  - `contextra-erasure` führt mit **66.0% Traceability** (62 von 94 Tests).
  - Fast alle anderen Crates (`contextra`, `store`, `crypto`, `types`) haben eine Traceability von **0.0%**. Dies stellt ein massives Governance-Risiko dar.
- **Pyramiden-Reifegrad pro Crate:**
  - **Vollständig aufgestellt:** `contextra-store` (567 Unit, 17 Proptest, 23 Loom, 7 FaultVfs, 31 Bench), `contextra-crypto` (321 Unit, 7 Proptest, 17 Loom, 6 FaultVfs, 10 Bench), `contextra-checkpoint`.
  - **Lückenhaft (Kein Proptest / Kein Loom / Kein Benchmark):** `contextra-durable-fs` (Kritischer Concurrency-Crate ohne Loom!), `contextra-mcp`, `contextra-agent`, `contextra-privacy`, `contextra-sys`.

---
## 14. AST-Inventar & Kern-Crate-Kontext-Profile

### 14.1 Workspace-weites AST-Inventar

Aus `ast_inventory_workspace.md`:
- **Gescannte Crates:** 37
- **Gescannte Dateien:** 955
- **Gesamtanzahl Rust-Items:** 13.395
  - Structs: 1.253
  - Enums: 278
  - Traits: 141
  - Impl-Blöcke: 1.622
  - Funktionen: 9.393 (davon 1.651 asynchron)
  - Unsafe Items: 43
  - Type-Aliase: 115
  - Konstanten: 593

### 14.2 Kern-Crate Kontext-Profile (Zusammenfassung der Context-Packs)

Aus `context_pack_contextra-*.md`:
1. **`contextra-types` (Ring 0, Klasse [K]):**
   - Rolle: Kanonische Typen, Fehlerbaum (`ContextraError`), RequestContext.
   - Abhängigkeiten: 0 interne Contextra-Crates. Rein externe Basispakete (`thiserror`, `serde`, `blake3`, `ahash`).
   - Lints: `#![forbid(unsafe_code)]`, `#![deny(dead_code)]`.
2. **`contextra-ports` (Ring 0, Klasse [K]):**
   - Rolle: Einzige Vertragsschicht des Baukastens. Alle Austausch-Traits (`Vfs`, `StorageEngine`, `VectorIndex`, `TextIndex`).
   - Abhängigkeiten: Nur `contextra-types`. Keine konkrete Implementierung bekannt.
3. **`contextra-store` (Ring 1, Klasse [K]):**
   - Rolle: LSM-Storage Engine (WAL, Memtable, SSTable, Manifest, Block-Cache).
   - Herausforderungen: Direkte I/O auf `Vfs` umstellen; Löschreste (`kv/segment.rs`) bereinigen; 94 Lock-across-await-Stellen auflösen.
4. **`contextra-engine` (Ring 3, Klasse [K]):**
   - Rolle: Orchestrierung von Collections, Transaktionen, Hybrid-Retrieval und Hintergrundarbeitern.
   - Herausforderungen: Konkrete Typen durch generisches Trait-Bündel `Backends` ersetzen; M3-Leaks eliminieren; Löschlogik nach `contextra-erasure` übertragen.
5. **`contextra-erasure` (Ring 3, Klasse [K]):**
   - Rolle: Eigenständiger kryptografischer Löschkern (Saga, Vault, GateLog, Journal, Ledger, Signierte Berichte).
   - Besonderheit: Null Workspace-Konsumenten (steht kurz vor der Verdrahtung in Phase 3); streng entkoppelt.

---
## 15. Konsolidierte Remediation-Roadmap & Architektur-Empfehlungen

Basierend auf der Synthese aller Diagnosen ergeben sich fünf priorisierte Arbeitspakete für die Entwicklerteams:

### Priorität 1: Durability & Crash Consistency (Sofortmaßnahmen)
1. **Behebung der Swallowed Syncs (Blocker D-55):**
   - In `crates/contextra-crypto/src/revocation_log.rs:529`: `vfs.sync_dir(p)?;` implementieren und Fehler fatal nach oben propagieren.
   - In `crates/contextra-infer-onnx/src/lib.rs:346`: `f.sync_all()?;` absichern.
2. **Vfs-Migration (INV-FSYNC-1):**
   - Schrittweise Ersetzung aller 200 unberechtigten `std::fs` / `tokio::fs` Aufrufe in `contextra-store`, `contextra-vector`, `contextra-checkpoint` und `contextra` durch den `Vfs`-Port.

### Priorität 2: Concurrency & Lock-Safety (Deadlock-Prävention)
1. **Lock-Held-Across-Await Refactoring (M8):**
   - Isolieren der kritischen Abschnitte in `contextra-store` (94 Stellen) und `contextra-engine` (40 Stellen). Mutex-Guards vor `.await`-Aufrufen explizit droppen (`drop(guard);`) oder auf synchrone Berechnungsblöcke beschränken.
2. **Beseitigung synchroner Blockaden (`block_on`):**
   - Entfernen ungeschützter `rt.block_on` in `contextra-store/src/scanner/sstable.rs` und `contextra-checkpoint`.

### Priorität 3: Type Registry & Interface Alignment (K3 / A2)
1. **Kanonisierung von `Timestamp` und `FileId`:**
   - Vereinheitlichung der kollidierenden Typen. `FileId` muss workspace-weit konsistent (bevorzugt `[u8; 16]` aus `contextra-types`) verwendet werden.
   - Entfernen von `types_local.rs` in `contextra-erasure` zugunsten von direkten Importen aus `contextra-ports`.
2. **Fassaden-Leaks (M3) schließen:**
   - Unzulässige Re-Exporte in `contextra-db`, `contextra-engine` und `contextra-router` entfernen.

### Priorität 4: Zero-Panic & Determinismus (P6 / P28)
1. **Panics in Produktion ersetzen:**
   - `panic!` in `contextra-store/src/wal/hmac.rs:217` durch `StorageError::UnsupportedWalVersion` ersetzen.
   - `expect()` in `contextra-store/src/scanner/sstable.rs` durch saubere Fehlerweitergabe ersetzen.
2. **SipHash-Eliminierung (P28):**
   - Ersetzen aller 274 `std::collections::{HashSet, HashMap}` in `contextra-store`, `contextra-engine`, `contextra-graph` und `contextra-vector` durch `BTreeSet`/`BTreeMap` oder `AHashSet`/`AHashMap`.

### Priorität 5: Governance & Test-Traceability (T-TRACE-1 / M2)
1. **SPEC-Traceability ergänzen:**
   - Nachdokumentieren von `/// SPEC: <ID>` für die verbleibenden 4.990 ungetraced Tests.
2. **Dead Code & Deprecated Debt tilgen:**
   - Entfernen von `erasure_legacy.rs` in `contextra-ports` und Bereinigen verwaister Funktionen in `types` und `crypto`.

---
*Dokumentende — Generiert für das Core-Engineering-Team von Contextra.*
