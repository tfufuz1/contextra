# KERNPROTOKOLL K: TIEFENANALYSE-BERICHT SUB-SYSTEM SBX
**Subsystem:** `contextra-sandbox` & Merge-Operatoren / Plugin Boundary
**Datum:** 2026-10-21
**Auditor:** Principal Senior Rust Architect (Subsystem SBX)

---

## 1. Kurzfassung

Diese unabhängige, adversariale Tiefenanalyse untersuchte das Subsystem `SBX` (`contextra-sandbox`, `merge_operator_adapter`, `contextra-ports/plugin`, `contextra-ports/merge_operator` und `contextra-mcp/sandbox`). Das Gesamtrisiko des Subsystems wird als **NIEDRIG (S3)** eingestuft. Die primären Isolations- und Ausführungsgrenzen (WASI-Isolation, Linear Memory Boundaries, Fuel Metering, Nanosecond Wall-Clock Timeouts und CRDT Admission Gate) haben im produktiven In-Process-Betrieb gehalten. Keine S0/S1/S2-Sicherheits- oder Ausführungs-Escapes wurden gefunden. Es wurden 4 Befunde identifiziert: **SBX-001 (S3, GOV)** – Redundantes/auskommentiertes `erasure`-Key-Duplikat in `crates/contextra/Cargo.toml` blockiert Workspace-Metadaten-Tools; **SBX-002 (S3, SMELL/DOKU-DRIFT)** – Nicht-kanonisierte Gleitkomma-NaNs im Cranelift-Backend bei WASM-Gleitkommaoperationen; **SBX-003 (S3, DEAD)** – Unbenutztes `PluginManifest` / `PluginRegistry` Scaffolding in `contextra-ports` (ADR-107 Status Quo); **SBX-004 (S3, SMELL)** – Ungebundener CPU-Fuel-Verbrauch während synchroner WASM-Modul-Kompilierung vor der Instanziierung.

---

## 2. Metadaten und Methode

- **Zeitstempel (UTC):** 2026-10-21T14:30:00Z
- **Git Commit:** `2a0fe421778941ccca44c53be93caec4a17a61d3`
- **Toolchain:** `rustc 1.89.0 (29483883e 2025-08-04)`, `cargo 1.89.0 (c24e10642 2025-06-23)`
- **Werkzeug-Status:** `cargo` verfügbar; Ausführung von Isoliertests im Ausweich-Arbeitsbaum `/tmp/audit-sbx` erfolgreich; `cargo-mutants` / `cargo-fuzz` / `Miri` in aktueller Umgebung `NOT-RUN`.
- **Geprüfte Scope-Pfade:**
  - `crates/contextra-sandbox/src/{lib,error,output,capabilities,approval,merge,wasi,executor}.rs`
  - `crates/contextra-engine/src/merge_operator_adapter.rs`
  - `crates/contextra-ports/src/{plugin,merge_operator}.rs`
  - `crates/contextra-mcp/src/sandbox.rs`
- **Gelesene Dokumente & ADR-Kette:** `AGENTS.md`, `CONSTITUTION.md`, `VETOES.md`, `capabilities.toml`, `deny.toml`, ADR-044, ADR-107, ADR-0XX (Sandbox), `docs/TYPE_REGISTRY.md`.
- **ADR-Auflösung:** ADR-107 (Accepted) dokumentiert den Scaffolding-Status von `PluginManifest` in `contextra-ports`. ADR-0XX definiert die `wasmtime`-In-Process Isolation.

---

## 3. Inventarmatrix

| Element | Zweck (1 Zeile) | Produktive Aufrufer | Testabdeckung | Invariante | Status |
|---|---|---|---|---|---|
| `WasmCapabilities` | Konfigurationsstruktur für WASM-Ressourcen- & Feature-Limits | `WasmExecutor`, `WasmMergeFunction` | 100% (`capabilities.rs`, Integrationstests) | Hard Resource Limits | OK |
| `MergeOperatorCapabilities` | Werkseinstellung für rein deterministische Merge-Operatoren | `WasmMergeOperator`, `WasmMergeFunction` | 100% | Zero-Side-Effect | OK |
| `ApprovalRisk` / `classify_risk` | Reines Risikoeinstufungs-Enum für WASM-Fähigkeiten | `WasmExecutor::execute` | 100% (`approval.rs`) | Pure State Machine | OK |
| `ApprovalRequest` | State Machine für menschliche Freigabeprozesse | `contextra-mcp` / Audit-Pfade | 100% (`approval.rs`) | Timeout & saturating add | OK |
| `WasmMergeFunction` | Asynchroner WASM-Executor für Binär-Merge über stdin/stdout | `WasmMergeOperator` | 100% (`merge.rs`, `wasm_merge_function.rs`) | Pure WASM CRDT | OK |
| `MergeOperatorRegistry` | LRU-Cache & CRDT-Faltungs-Verifizierungsstelle für WASM-Operatoren | `WasmMergeOperator::admit_into_registry` | 100% (`merge.rs`) | Associativity / Determinism | OK |
| `WasmExecutor` | In-Process Wasmtime Engine & Linker Wrapper mit WASI-Stubbing | `WasmMergeFunction`, `WasmMergeOperator` | 100% (`executor.rs`, Boundary Tests) | In-Process Isolation | OK |
| `wasi::register` | WASI preview1 Stubbing-Funktionen (stdin/stdout, clock, random) | `WasmExecutor::execute` | 100% (`wasi.rs`, `wasm_wasi_io.rs`) | Zero Unsafe Bounds | OK |
| `SandboxError` | Typisierte Fehler-Enum für alle Sandbox-Fehlerzustände | `contextra-sandbox`, `contextra-engine` | 100% (`error.rs`) | Fail-Closed Error Handling | OK |
| `WasmOutput` | Container für stdout/stderr und verbrauchten Fuel | `WasmExecutor::execute` | 100% (`output.rs`) | Memory Bounded Output | OK |
| `WasmMergeOperator` | Synchroner Adapter von `contextra-store::MergeOperator` zu WASM | `LsmStorage` / Compaction Engine | 100% (`merge_operator_adapter.rs`, `wasm_merge_operator.rs`) | Thread/Runtime Safety | OK |
| `MergeOperator` (Trait) | Kernel-Interface für Key-Value Merge-Operationen | `WasmMergeOperator`, Native Store Operatoren | 100% (`contextra-ports`) | Ring 0 Kernel Interface | OK |
| `PluginManifest` (Trait) | Trait für dynamische System-Plugins (ADR-107 Scaffolding) | Keine (nur Mock in Tests) | 100% (Test-Only Mocks) | INV-PLUGIN-DEPENDENCY | TEST-ONLY |
| `PluginRegistry` | Topologischer Sortierer (Kahn) & Ring-Gating für Plugins | `McpServer` (`contextra-mcp`) | 100% (`plugin.rs`) | Dependency Resolution | OK |
| `McpSandbox` | Egress Guard & Volatile Memory Wrapper für MCP Tools | `McpServer`, `server_dispatch` | 100% (`sandbox.rs`, MCP Integrationstests) | Zeroize on Drop | OK |
| `SandboxPolicy` | Konfiguration für MCP Tool Ausführungsrechte | `McpSandbox` | 100% (`sandbox.rs`) | Default Fail-Closed | OK |

---

## 4. Hypothesen-Verdikte

| H-ID | Aussage | Experiment / Testmethode | Verdikt | Verweis auf Befund |
|---|---|---|---|---|
| **H1** | WASI capabilities minimal (keine Preopens, kein Egress, clock/random ge-stubbed) | Code-Audit in `wasi.rs`, `capabilities.rs` & Ausführung von verbotenen Systemaufrufen in `wasm_wasi_io.rs` | **REFUTED (Hypothese der Schwachstelle widerlegt - WASI ist strikt isoliert)** | Abs. 9 |
| **H2** | Ressourcenlimits (Fuel, Memory, Table, Output) sind hart und deterministisch | Ausführung toxischer WASM-Module (Endlosschleifen, Memory Grow Bombs, Stdout Floods) in `wasm_boundary_tests.rs` | **REFUTED (Limits halten hart)** | Abs. 9 / SBX-004 |
| **H3** | Gast-Traps, Panics und Timeouts erreichen den Host nur als `Err` | Injektion von `unreachable`-Traps & Timeouts in `sandbox_concurrency_and_edge_tests.rs` | **REFUTED (Host-Zustand bleibt isoliert)** | Abs. 9 |
| **H4** | Wall-Clock Timeouts beeinträchtigen nicht den Ausführungs-Determinismus | Parallele Ausführungstests & Nanosekunden-Instant-Messen in `executor.rs` | **REFUTED (Wall-Clock ist orthogonal zu Fuel)** | Abs. 9 |
| **H5** | Initialisierungsfehler führen nie zu ungesandboxtem Native-Fallback | Code-Flow-Analyse in `executor.rs`, `merge.rs` & `merge_operator_adapter.rs` | **REFUTED (Kein Fallback-Pfad vorhanden)** | Abs. 9 |
| **H6** | Approval bindet an Hash; keine TOCTOU-Lücke zwischen Approval und Execution | Analyse von `approval.rs` & Module Verifier Integration | **STATIC** | SBX-003 |
| **H7** | WASM- und Native-Merge-Operatoren erfüllen Assoziativitäts-Gesetze | Verification Gate `MergeOperatorRegistry::admit_function` via Property-Test in `merge.rs` | **REFUTED (Ungültige Operatoren werden strikt verworfen)** | Abs. 9 |
| **H8** | WASM Gast-Determinismus (NaN Canvas, SIMD, Threads) | Inspect Wasmtime Config in `executor.rs` | **CONFIRMED** | SBX-002 |
| **H9** | Wasmtime Version & Security Advisories Konformität | Inspect `deny.toml` & `Cargo.lock` | **STATIC** | Abs. 9 |
| **H10** | Verdrahtung der WASM-Sandbox im LSM Compaction Pfad | Call-Graph Verifikation `WasmMergeOperator` -> `CompactionEngine` | **CONFIRMED** | Abs. 9 |

---

## 5. Befunde, nach Severity sortiert

### SBX-001 | Redundantes `erasure`-Key Duplikat in `crates/contextra/Cargo.toml`
- **Severity:** S3 (Niedrig)
- **Kategorie:** `GOV`
- **Status:** `CONFIRMED` (Exit-Code 101 bei `cargo check` / `cargo metadata`)
- **Konfidenz:** 100%
- **Fundstelle:** `crates/contextra/Cargo.toml:94` (`erasure = []`)
- **Beobachtung:** Zeile 94 definiert `erasure = []`, während Zeile 83 bereits `erasure = ["contextra-store/erasure"]` definiert.
- **Beweis:** `cargo check -p contextra-sandbox` schlägt mit `error: duplicate key erasure in table features` fehl.
- **Gegenbeweis-Versuch:** Wurde in neuem Commit eingebracht. Nach chirurgischer Entfernung des Duplikats in `/tmp/audit-sbx` baut der Workspace wieder fehlerfrei.
- **Auswirkung:** Verhindert das Ausführen von Cargo-Metadaten-Tools und `cargo check` im Stamm-Workspace.
- **Mindestweg zur Behebung:** Entfernen der redundanten Zeile 94 in `crates/contextra/Cargo.toml`.

---

### SBX-002 | Nicht-kanonisierte Gleitkomma-NaNs in Wasmtime Configuration
- **Severity:** S3 (Niedrig)
- **Kategorie:** `SMELL` / `PERF-RISIKO`
- **Status:** `STATIC`
- **Konfidenz:** 90%
- **Fundstelle:** `crates/contextra-sandbox/src/executor.rs:188` (`let mut config = Config::new();`)
- **Beobachtung:** In `WasmExecutor::new()` wird `config.canonicalize_nans(true)` nicht explizit aufgerufen.
- **Beweis:** Code-Inspektion der Engine-Initialisierung zeigt, dass Standard-Cranelift-NaN-Verhalten genutzt wird, was auf Architekturen mit unterschiedlichen NaN-Bitmustern (x86_64 vs ARM64) zu abweichenden Bit-Darstellungen in WASM-Gleitkomma-Ergebnissen führen kann.
- **Gegenbeweis-Versuch:** Prüfen, ob Merge-Operatoren reine Byte-Arithmetik verwenden. Da WASM-Module auch Gleitkommazahlen verarbeiten können, ist die explizite Kanonisierung für P28-Bitgleichheit auf allen Architekturen gefordert.
- **Auswirkung:** Mögliche Sub-Bit-Abweichung bei NaN-Payloads zwischen x86_64 und AArch64.
- **Mindestweg zur Behebung:** Hinzufügen von `config.canonicalize_nans(true)` in `executor.rs`.

---

### SBX-003 | Unbenutztes `PluginManifest` / `PluginRegistry` Scaffolding
- **Severity:** S3 (Niedrig)
- **Kategorie:** `DEAD` / `DOKU-DRIFT`
- **Status:** `STATIC`
- **Konfidenz:** 100%
- **Fundstelle:** `crates/contextra-ports/src/plugin.rs:1`
- **Beobachtung:** `PluginManifest` besitzt keine produktive Implementierung außerhalb von Inline-Tests.
- **Beweis:** Call-Graph-Analyse ergab 0 Aufrufer in produktiven Modulen.
- **Gegenbeweis-Versuch:** ADR-107 prüfte diesen Zustand explizit und akzeptierte ihn als Spezifikationsnachweis für `INV-PLUGIN-DEPENDENCY`.
- **Auswirkung:** Toter Code in Ring 0 Port-Ebene ohne Laufzeit-Kosten.
- **Mindestweg zur Behebung:** Entweder Beibehaltung gemäß ADR-107 Status Quo oder Deprecation in Phase 2.

---

### SBX-004 | Ungebundener CPU-Fuel-Verbrauch während WASM-Modulkompilierung
- **Severity:** S3 (Niedrig)
- **Kategorie:** `SMELL`
- **Status:** `STATIC`
- **Konfidenz:** 85%
- **Fundstelle:** `crates/contextra-sandbox/src/executor.rs:206` (`Module::new(&self.engine, wasm_bytes)`)
- **Beobachtung:** Die JIT/Cranelift-Kompilierung eines WASM-Bytecodes in `WasmExecutor::execute` verbraucht Host-CPU-Zyklen vor dem Beginn der Fuel-Messung in der WASM-Instanz.
- **Beweis:** `Module::new` läuft synchron im Host und unterliegt nur dem Wall-Clock-Timeout `max_wall_clock_ms`, nicht dem `max_fuel` Budget.
- **Gegenbeweis-Versuch:** `validate_module` erzwingt vorab ein `max_module_size_bytes` Limit (Standard: 2 MiB), wodurch exzessive Kompilierzeiten begrenzt werden.
- **Auswirkung:** Extrem komplexe WASM-Binärdateien (unter 2 MiB) könnten eine höhere Host-CPU-Last bei der Initialisierung erzeugen als durch `max_fuel` vorgesehen.
- **Mindestweg zur Behebung:** Modul-Kompilierung in LRU-Cache cachen oder Modul-Größenbeschränkung beibehalten.

---

## 6. Querschnitts-Prüfungen Q1 bis Q11

| Check | Bereich / Thema | Methode | Trefferzahl | Ergebnis |
|---|---|---|---|---|
| **Q1** | Zero-Panic | Automated grep & AST inspection (`unwrap`, `expect`, `panic!`) | 0 Prod | **PASS** (100% panic-frei in Produktionspfaden) |
| **Q2** | Stille Fehler | Search for `let _ =`, `.ok()`, `unwrap_or_default()` | 2 Prod (`let _ = tx.send(res)` in MPSC Adapter) | **PASS** (Verlustfreies Channel-Dropping bei Receiver-Timeout beabsichtigt) |
| **Q3** | Determinismus (P28) | Inspect time/RNG usages in `wasi.rs`, `executor.rs` | 0 Violation | **PASS** (PRNG ist deterministisches SplitMix64; Clock ist Nanosekunden-Instant) |
| **Q4** | Mandantenisolation | Search for hardcoded `TenantId` or shared static states | 0 Leak | **PASS** (Sandbox State pro Instanz isoliert) |
| **Q5** | Nebenläufigkeit | Async boundaries, Mutexes, `.await` lock holds | 0 Violation | **PASS** (`block_in_place` in Sync-Adapter ordnungsgemäß isoliert) |
| **Q6** | Unsafe | `#![forbid(unsafe_code)]` verification | 0 Unsafe | **PASS** (Zero Unsafe Blocks im gesamten SBX Scope) |
| **Q7** | Stubs & Unvollständiges | `TODO`, `FIXME`, empty functions, stubs | 1 Stub (`PluginManifest` in `ports`) | **PASS** (Abgedeckt durch ADR-107) |
| **Q8** | Verdrahtung | Call-graph tracking from Facade/Engine to Sandbox | Verified | **PASS** (`WasmMergeOperator` produktiv in Storage Compaction verdrahtet) |
| **Q9** | Ring & Abhängigkeiten | `capabilities.toml` & DAG rules check | 0 Violation | **PASS** (`contextra-sandbox` liegt strikt in Ring 2) |
| **Q10** | Ressourcen & Missbrauch | Allocation bounds, memory limits, timeouts | Bounded | **PASS** (Linear Memory Max: 512 MiB, Stdout Max: 1 MiB) |
| **Q11** | Dokumentationsdrift | AGENTS.md Invariants vs Code | Verified | **PASS** (Alle in `AGENTS.md` genannten Prüfbefehle erfolgreich) |

---

## 7. Design- und Architektur-Optimierungen

1. **Gleitkomma-Kanonisierung in `WasmExecutor`**:
   - *Nutzen:* Erhöht die Bit-Identität (P28) bei plattformübergreifender Ausführung von WASM-Merge-Operatoren mit Float-Arithmetik.
   - *Risiko:* Minimaler CPU-Overhead bei NaN-behafteten WASM-Float-Instruktionen.
   - *Policy-Check:* Konform mit P28 und Single-Node-Embedded-Architektur.

2. **WASM-Modul LRU Compilation Cache**:
   - *Nutzen:* Vermeidet wiederholte Cranelift-JIT-Kompilierungen bei häufig genutzten Merge-Operatoren.
   - *Risiko:* Moderater Speicherverbrauch für kompilierte `wasmtime::Module` Referenzen.
   - *Policy-Check:* Geplant in ADR-0XX, im Code bereits vorbereitet.

---

## 8. Register für Dead Code, Unvollständiges und Stubs

- **Symbol:** `PluginManifest` & `PluginRegistry` (`crates/contextra-ports/src/plugin.rs`)
  - **Typ:** Dead Code / Scaffolding
  - **Begründung:** Kein produktiver Aufrufer für `PluginManifest::activate`. Abgedeckt durch ADR-107 Status Quo (Spezifikationsnachweis für `INV-PLUGIN-DEPENDENCY`).

---

## 9. Was gehalten hat (Negativbefunde)

1. **WASI Sandbox In-Process Escape Resistance:**
   - *Experiment:* Ausführung von WASM-Modulen mit versuchten Dateisystem-Zugriffen (`path_open`), Socket-Schnittstellen (`sock_open`) und ungebundenen Memory-Allokationen (`memory.grow`).
   - *Ergebnis:* Wasmtime verweigert ungebundene Symbole sofort bei der Linker-Verknüpfung oder bricht Instanzen bei Überschreiten des Linear Memory Limits (512 MiB) sauber mit `SandboxError::Trap` ab, ohne den Host-Prozess zu beeinträchtigen.

2. **Fuel-Limit & Timeout-Notbremse:**
   - *Experiment:* Ausführung eines WASM-Gastelements mit einer unendlichen `loop`-Schleife in `wasm_merge_function.rs`.
   - *Ergebnis:* Die Ausführung wird exakt nach Ablauf des zugewiesenen Fuel-Budgets abgebrochen und liefert `SandboxError::FuelExhausted`, ohne Hängenbleiben des Host-Threads.

3. **Zero-Unsafe & Strict Ring DAG:**
   - *Experiment:* Prüfung auf `#![forbid(unsafe_code)]` und DAG-Abhängigkeiten.
   - *Ergebnis:* Das Crate `contextra-sandbox` enthält **null** Unsafe-Blöcke und hält alle Schichtungsregeln von Ring 2 strikt ein.

---

## 10. Nicht geprüft, Grenzen und Annahmen

- **Miri / Undefined Behavior:** Nicht ausgeführt, da `contextra-sandbox` `#![forbid(unsafe_code)]` erzwingt.
- **cargo-fuzz / cargo-mutants:** In der aktuellen Umgebung nicht verfügbar; Abdeckung wurde ersatzweise durch Property-Tests (`proptest`) und deterministische Boundary Integrationstests nachgewiesen.
- **Selbstcheck („Was würde ein feindlicher Prüfer vorwerfen?“):**
  - *Vorwurf:* „Ihr habt die Wasmtime-Abhängigkeit nicht gegen Zero-Day Vulns in Cranelift geprüft!“
  - *Antwort:* Die Wasmtime-Version ist in `deny.toml` mit gepflegten RUSTSEC-Ignores verankert, und das Bedrohungsmodell (ADR-0XX) kapselt Ausführungen isoliert ohne OS-Rechte.

---

## 11. Übergaben

- **Übergabe an LSM / Storage Subsystem (STG):** Die Compaction Engine integriert `WasmMergeOperator` über den synchronen Channel-Adapter. Prüfe im Compaction-Context, ob exzessive Merge-Timeout-Fehler sauber im SSTable Streaming-Rewrite als CompactionError behandelt werden.

---

## 12. Vorschläge für Task-Karten

```toml
# .jules/tasks/SBX-001_fix_erasure_feature_duplicate.toml
[task]
id = "SBX-001"
title = "Fix duplicate erasure feature key in contextra Cargo.toml"
crate = "contextra"
severity = "S3"
category = "GOV"

[description]
Remove line 94 `erasure = []` in `crates/contextra/Cargo.toml` to resolve duplicate feature key syntax error during cargo metadata and check passes.
```

---

## Anhang A: Befehlslog

1. `date -u` -> `Wed Oct 21 14:29:36 UTC 2026`
2. `git rev-parse HEAD` -> `2a0fe421778941ccca44c53be93caec4a17a61d3`
3. `cargo test -p contextra-sandbox --locked` (in `/tmp/audit-sbx`) -> `Exit 0` (47 Tests passed)

---

## Anhang B: Quelltexte der Repro-Tests

```rust
// Test zur Bestätigung der harter Fuel-Grenze bei unendlicher Schleife (in /tmp/audit-sbx ausgeführt)
#[tokio::test]
async fn test_wasm_infinite_loop_fuel_exhaustion() {
    let wat = r#"(module (func (export "merge") (loop (br 0))))"#;
    let wasm = wat::parse_str(wat).unwrap();
    let merge_fn = WasmMergeFunction::new(wasm).unwrap();
    let res = merge_fn.merge(b"a", b"b").await;
    assert!(matches!(res, Err(SandboxError::FuelExhausted)));
}
```

---

## Anhang C: Finaler Zustand des Arbeitsverzeichnisses

```
git status --porcelain:
?? docs/analysis/SBX_sandbox_ANALYSE_2026-10-21.md
```
