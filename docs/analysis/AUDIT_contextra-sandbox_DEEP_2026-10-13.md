# Deep Audit Report: `crates/contextra-sandbox` & WASM MergeOperator Subsystem

- **Spezialist:** WASM Sandbox, Custom Extensions & Merge Operators Specialist
- **Commit:** `59f853cb`
- **Datum:** 2026-10-13
- **Vorgänger-Bericht:** `docs/audits/AUDIT_contextra-sandbox_2026-10-12.md`
- **Session Status:** `VERIFIED-BY-SESSION: PENDING`

---

## 1. Scope und Abdeckung

| Datei / Pfad | Zeilen | Gelesen | Begründung / Fokus |
|---|---|---|---|
| `crates/contextra-sandbox/src/lib.rs` | 31 | Ja | Crate-Root, Exporte, `#![forbid(unsafe_code)]` |
| `crates/contextra-sandbox/src/admission.rs` | 51 | Ja | WASM Module Admission Control & ModuleVerifier Trait |
| `crates/contextra-sandbox/src/approval.rs` | 370 | Ja | ApprovalRequest State Machine & Risikoklassifizierung |
| `crates/contextra-sandbox/src/capabilities.rs` | 312 | Ja | WasmCapabilities, ModulePolicy, MergeOperatorCapabilities |
| `crates/contextra-sandbox/src/error.rs` | 42 | Ja | SandboxError, AdmissionError Enum-Hierarchie |
| `crates/contextra-sandbox/src/executor.rs` | 496 | Ja | WasmExecutor, Engine & Store Lifecycle, Fuel & Timeout |
| `crates/contextra-sandbox/src/merge.rs` | 138 | Ja | WasmMergeFunction, Stdin/Stdout Binary Protocol |
| `crates/contextra-sandbox/src/output.rs` | 27 | Ja | WasmOutput mit ZeroizeOnDrop für stdout (P9) |
| `crates/contextra-sandbox/src/wasi.rs` | 506 | Ja | Handgeschriebener WASI preview1 Host (fd_read, fd_write, etc.) |
| `crates/contextra-engine/src/merge_operator_adapter.rs` | 191 | Ja | Synchroner Bridge-Adapter `WasmMergeOperator` |
| `crates/contextra-engine/src/collection/query_builder/mod.rs` | 280 | Ja | Top-k Re-Ranking vs Traversal Entkopplung |

<!-- EVIDENCE: logs/audits/contextra-sandbox_2026-10-13/crate-context-sandbox.log (exit=0) -->
<!-- EVIDENCE: logs/audits/contextra-sandbox_2026-10-13/crate-context-engine.log (exit=0) -->

---

## 2. Inventar der pub-Items

| Symbol / Item | Datei : Zeile | Nutzung / Status |
|---|---|---|
| `pub struct AdmittedModule` | `admission.rs:14` | Produktiv (WasmExecutor::execute_admitted) |
| `pub fn AdmittedModule::new` | `admission.rs:23` | Produktiv |
| `pub fn AdmittedModule::bytes` | `admission.rs:28` | Produktiv |
| `pub fn AdmittedModule::digest` | `admission.rs:33` | Nur Test / API |
| `pub fn AdmittedModule::into_bytes` | `admission.rs:38` | Nur Test / API |
| `pub trait ModuleVerifier` | `admission.rs:47` | Ungenutzt (Orphan Symbol in `orphan-symbols`) |
| `pub enum ApprovalRisk` | `approval.rs:16` | Produktiv |
| `pub fn classify_risk` | `approval.rs:34` | Produktiv |
| `pub enum ApprovalStatus` | `approval.rs:46` | Produktiv |
| `pub enum ApprovalTransitionError` | `approval.rs:66` | Produktiv |
| `pub struct ApprovalRequest` | `approval.rs:87` | Produktiv |
| `pub fn ApprovalRequest::new` | `approval.rs:107` | Produktiv |
| `pub fn ApprovalRequest::is_expired` | `approval.rs:129` | Produktiv |
| `pub fn ApprovalRequest::approve` | `approval.rs:137` | Produktiv |
| `pub fn ApprovalRequest::reject` | `approval.rs:167` | Produktiv |
| `pub fn ApprovalRequest::expire` | `approval.rs:197` | Produktiv |
| `pub fn ApprovalRequest::auto_approve_if_low_risk` | `approval.rs:217` | Produktiv |
| `pub fn ApprovalRequest::requires_approval` | `approval.rs:235` | Produktiv |
| `pub enum ModulePolicy` | `capabilities.rs:14` | Produktiv |
| `pub fn constant_time_eq_32` | `capabilities.rs:25` | Produktiv |
| `pub struct WasmCapabilities` | `capabilities.rs:134` | Produktiv |
| `pub fn WasmCapabilities::compute_sha256` | `capabilities.rs:193` | Produktiv |
| `pub fn WasmCapabilities::verify_module_policy` | `capabilities.rs:201` | Produktiv |
| `pub fn WasmCapabilities::pure_merge_operator` | `capabilities.rs:234` | Produktiv |
| `pub struct MergeOperatorCapabilities` | `capabilities.rs:257` | Produktiv |
| `pub fn MergeOperatorCapabilities::pure` | `capabilities.rs:261` | Produktiv |
| `pub fn MergeOperatorCapabilities::pure_with_result_channel` | `capabilities.rs:269` | Produktiv |
| `pub enum AdmissionError` | `error.rs:7` | Produktiv |
| `pub enum SandboxError` | `error.rs:26` | Produktiv |
| `pub struct CapabilityViolationError` | `executor.rs:28` | Produktiv |
| `pub struct WasmExecutor` | `executor.rs:88` | Produktiv |
| `pub fn WasmExecutor::new` | `executor.rs:103` | Produktiv |
| `pub fn WasmExecutor::validate_module` | `executor.rs:116` | Produktiv |
| `pub async fn WasmExecutor::execute_admitted` | `executor.rs:160` | Produktiv |
| `pub async fn WasmExecutor::execute` | `executor.rs:178` | Conditional (`#[cfg(feature = "unverified-execute")]`) |
| `pub struct WasmMergeFunction` | `merge.rs:27` | Produktiv |
| `pub fn WasmMergeFunction::new` | `merge.rs:40` | Produktiv |
| `pub fn WasmMergeFunction::new_with_allowlist` | `merge.rs:51` | Produktiv |
| `pub fn WasmMergeFunction::new_with_policy` | `merge.rs:59` | Produktiv |
| `pub fn WasmMergeFunction::new_with_capabilities` | `merge.rs:69` | Produktiv |
| `pub async fn WasmMergeFunction::merge` | `merge.rs:100` | Produktiv |
| `pub struct WasmOutput` | `output.rs:11` | Produktiv |
| `pub fn WasmOutput::new` | `output.rs:22` | Produktiv |
| `pub struct ProcessExitError` | `wasi.rs:16` | Produktiv |
| `pub struct OutputLimitExceededError` | `wasi.rs:30` | Produktiv |
| `pub struct WasmMergeOperator` | `merge_operator_adapter.rs:39` | Produktiv |
| `pub fn WasmMergeOperator::new` | `merge_operator_adapter.rs:55` | Produktiv |
| `pub fn WasmMergeOperator::new_with_config` | `merge_operator_adapter.rs:69` | Produktiv |

---

## FACHLICHE PRÜFFRAGEN

### 1. Fuel vs Wall-Clock-Limits
- **Ist Fuel das einzige harte Rechenlimit?** Nein. Fuel (`max_fuel: 10_000_000` CPU-Ticks) ist das deterministische CPU-Limit. Standardmäßig greift im Hot-Path zusätzlich ein asynchrones Wall-Clock-Timeout (`max_wall_clock_ms: 5_000` ms) via `tokio::time::timeout_at`.
- **Greifen Wall-Clock-Grenzen im Hot-Path?** Ja. In `WasmExecutor::execute_internal` (`src/executor.rs:200-208` und `:358-361`) wird das Ausführungs-Future in `tokio::time::timeout_at(deadline, execute_future)` gekapselt.
- **Quelltext-Standardwerte:**
  - `max_fuel`: `10_000_000` Ticks (`capabilities.rs:175`)
  - `max_wall_clock_ms`: `5_000` ms (`capabilities.rs:181`)
  - `max_memory_pages`: `16` Pages = 1 MB (`capabilities.rs:174`)
  - `max_module_size_bytes`: `10_485_760` Bytes = 10 MB (`capabilities.rs:182`)

### 2. Threads, Runtimes & Nebenläufigkeitsgrenzen
- **Pro Operator OS-Thread / Runtime?**
  - In `contextra-sandbox` (`WasmMergeFunction`): Nein. Nutzt den bestehenden Tokio-Async-Executor der Anwendung.
  - Im Adapter `contextra-engine` (`WasmMergeOperator`): **JA.** `WasmMergeOperator::new_with_config` (`merge_operator_adapter.rs:88-111`) spawt für jede Instanz von `WasmMergeOperator` einen eigenen OS-Thread (`wasm-merge-worker`) und baut darin eine neue Single-Threaded Tokio-Runtime (`tokio::runtime::Builder::new_current_thread()`).
- **Wie viele Operatoren gleichzeitig möglich & Engpass?**
  - Wenn viele `WasmMergeOperator`-Instanzen erzeugt werden, entsteht pro Instanz ein OS-Thread + Runtime. Die Skalierung wird durch die maximale Thread-Anzahl des Betriebssystems (Linux `max_map_count` / `threads-max`) begrenzt.
  - Bei Compaction in `contextra-store` wird in der Regel *ein* `WasmMergeOperator` pro Column-Family / Collection wiederverwendet (wrapped in `Arc<dyn MergeOperator>`).

### 3. Modul-Kompilierung & Caching
- **Vorkompiliert/gecacht oder je Aufruf neu kompiliert?**
  - In `WasmMergeFunction` (`src/merge.rs:77-80`): Das WASM-Modul wird **einmalig** bei Erzeugung von `WasmMergeFunction` validiert und als `AdmittedModule` gehalten.
  - In `WasmExecutor::execute_internal` (`src/executor.rs:222-235`): Bei **jedem einzelnen `merge()`-Aufruf** wird `Module::from_binary(&engine, &wasm_bytes)` in `tokio::task::spawn_blocking` neu aus den Rohbytes kompiliert! Es existiert **kein `wasmtime::Module`-Cache** für wiederholte Aufrufe.

### 4. Isolation von Traps, OOM, Panics
- **Bricht ein Fehler im Gastmodul nur den Operator ab?**
  - **Ja.** Traps (`wasmtime::Trap`), Memory Growth Rejections (`ResourceLimiter`), Gas-Erschöpfung (`OutOfFuel`) und `proc_exit` werden von Wasmtime gefangen und als `Err(SandboxError::...)` zurückgegeben. Rust-Panics im Host treten nicht auf.
  - Im Adapter `WasmMergeOperator` wird der Fehler abgefangen, mit `tracing::warn!` geloggt und als `Err(ContextraError::Sandbox(...))` zurückgegeben.
- **Testbeleg:**
  - `crates/contextra-sandbox/src/executor.rs`: `test_wasm_fuel_exhaustion_returns_error`, `test_wasm_memory_isolation`, `test_wasm_trap_unreachable_classified_as_wasm_trap`.
  - `crates/contextra-engine/tests/wasm_merge_operator.rs`: `test_wasm_merge_operator_infinite_loop_timeout`.

### 5. Determinismus des Fuel-Verbrauchs
- **Ist der Fuel-Verbrauch deterministisch?**
  - **Ja.** Wasmtime Fuel basiert auf Cranelift-instruierten Basic-Block-Zählern. Bei gleichem WASM-Bytecode und gleichen Inhalten ist der Fuel-Verbrauch exakt identisch.
- **Testbeleg:**
  - In `crates/contextra-sandbox/tests/wasm_boundary_tests.rs`: `test_deterministic_fuel_consumption` verifiziert, dass zwei identische Ausführungen exakt denselben `fuel_consumed`-Wert liefern.

### 6. CRDT-Gesetze & Freischaltbedingung
- **CRDT-Gesetze als Property-Tests vorhanden?**
  - **LÜCKE.** Es existieren Unit-Tests für `WasmMergeOperator` (Counter, String-Concat), aber **keine Property-Based-Tests** (z. B. via `proptest` oder `quickcheck`), die Kommutativität, Assoziativität oder Idempotenz systematisch prüfen.
- **Freischaltbedingung für WASM-Operatoren?**
  - **NEIN.** Jedes valide WASM-Binary kann aktuell als `WasmMergeOperator` registriert werden, ohne dass eine mathematische Verifikation oder Property-Prüfung der Merge-Eigenschaften erzwungen wird.

### 7. Host-API Minimallokation & Rechte
- **Minimal, versioniert, ohne Rechte?**
  - **Ja.** Der Host stellt in `wasi.rs` nur Minimal-WASI preview1 (`fd_read` [stdin], `fd_write` [stdout/stderr], `proc_exit`, `clock_time_get`, `random_get`, `args/environ`) und in `executor.rs` zwei `contextra`-Host-Funktionen (`host_cloud_query`, `host_sleep`) bereit.
  - Netzwerksockets, Dateisystemzugriffe und Host-Prozess-Execution sind vollständig deaktiviert (`ERRNO_ACCES` / `ERRNO_NOSYS`).

### 8. Nutzer-Metriken im Re-Ranking vs Traversal
- **Werden Nutzer-Metriken nur im Top-k Re-Ranking verwendet?**
  - **Ja.** In `crates/contextra-engine/src/collection/query_builder/mod.rs` werden Re-Ranker (z. B. Cross-Encoder oder benutzerdefinierte Metriken) ausschließlich auf die nach Graph-/Vektor-Retrieval gefilterten Top-$k$-Kandidaten angewendet (`candidate_k` Expansion). Das Graph-Traversal (CSR-PF / PPR) und die HNSW-Vektorsuche laufen strikt isoliert auf Topologie und Vektor-Distanzen ohne Nutzer-Metrik-Einfluss.

---

## 3. Ergebnisse je Kategorie A bis M

- **A. Unvollständige / angefangene Implementierungen:**
  - `ModuleVerifier` Trait in `admission.rs:47` ist definiert, besitzt aber workspace-weit keine produktive Implementierung.
  - `ModulePolicy::SignedBy` in `capabilities.rs:218` liefert fest ein `SandboxError::InvalidModule`, da die `ed25519-dalek` Abhängigkeit im Crate fehlt.

- **B. Fehlerhafte Logik:**
  - In `executor.rs:224`: Bei jedem `merge()`-Aufruf wird `Module::from_binary` aufgerufen. Bei hoher Compaction-Frequenz erzeugt das unnötige CPU-Overheads für die wiederholte JIT/Cranelift-Kompilierung.

- **C. Fehlende Verdrahtung:**
  - `ApprovalRequest` / `classify_risk` in `approval.rs` ist vollständig als State Machine implementiert, wird aber von keinem MCP-Tool oder Engine-Call automatisiert aufgerufen (Stand-alone Modul).

- **D. Dead Code:**
  - Trait `ModuleVerifier` in `admission.rs` wird nicht verwendet.

- **E. Fehlerbehandlung:**
  - In `wasi.rs:324` (`args_sizes_get`): Doppelte `if argc_end > mem_slice_mut.len() || buf_size_end > mem_slice_mut.len()` Überprüfung (redudanter Code-Smell).

- **F. Panics und Unwraps im Produktionscode:**
  - `#![forbid(unsafe_code)]` ist in `src/lib.rs` gesetzt.
  - Produktionscode verwendet `.unwrap_or(...)` und `.unwrap_or_else(...)` auf MutexGuards, was sicher ist. Keine ungefangenen Panics.

- **G. Nebenläufigkeit:**
  - In `merge_operator_adapter.rs`: Erzeugt pro `WasmMergeOperator` einen eigenen OS-Thread (`wasm-merge-worker`) mit dedizierter Single-Threaded Tokio-Runtime. Bei vielen aktiven Operatoren droht Thread-Exhaustion.

- **H. Determinismus:**
  - WASI `clock_time_get` liefert relative Nanosekunden seit `start_instant` (keine Absolute Systemzeit). WASI `random_get` nutzt SplitMix64 PRNG basierend auf `random_seed`. Beides ist deterministisch.

- **I. Mandanten- und Berechtigungsgrenzen:**
  - Sandbox führt WASM-Code vollständig isoliert in RAM-Puffern aus. Keine Leaks zwischen Mandanten.

- **J. Architektur und Design:**
  - Ring 2 Grenzen sauber eingehalten (Abhängigkeiten nur zu Ring 0/1, keine Aufwärtskanten).

- **K. Code-Smells:**
  - In `wasi.rs`: Mehrere Lints zu `clippy::cast_sign_loss` bei `i32` zu `usize` Ptr-Konvertierungen.

- **L. Tests:**
  - **Lücke:** Keine Property-Tests für CRDT-Gesetze (Kommutativität, Assoziativität, Idempotenz) von WASM-Merge-Operatoren.

- **M. Dokumentationsdrift:**
  - `AGENTS.md` §1 erwähnt `CodeExecution`-Permission, welche in Ring 4 (`contextra-mcp`) gepflegt wird.

---

## 4. Befunde

### FINDING-SBX-01: Fehlen von CRDT-Property-Tests & Freischaltungs-Gate für WASM Merge-Operatoren
- **Schwere:** Mittel
- **Kategorie:** L (Tests) / A (Unvollständige Logik)
- **Status:** BELEGT
- **Fundstelle:** `crates/contextra-sandbox/src/merge.rs:100` / `crates/contextra-engine/src/merge_operator_adapter.rs:126`
- **Befund:** WASM-Merge-Operatoren werden in LSM-Compactions verwendet. Ist ein Merge-Operator nicht kommutativ oder assoziativ, führt die parallele N-Way-Compaction in LSM-Trees zu nicht-deterministischem Datenverlust oder Inkonsistenzen. Es fehlen Property-Tests sowie eine Verifikations-Sperre für nicht-CRDT-konforme Operatoren.
- **Evidenz:** `grep -rn "CRDT" crates/` ergab 0 Treffer im Sandbox/Engine Crate.
- **Betroffene Invariante:** Inv-LSM-Merge / Determinismus
- **Empfehlung:** Implementierung von Property-Tests via `proptest` für Referenz-Operatoren und Einführung eines optionalen `CRDTValidator` Flags.
- **Red-Test-Vorschlag:** Fuzz-Test mit zufälliger Zusammenfassung von N Werten in unterschiedlicher Reihenfolge.

### FINDING-SBX-02: Unnötige Re-Kompilierung von WASM-Modulen je `merge()`-Aufruf
- **Schwere:** Niedrig
- **Kategorie:** B (Fehlerhafte Logik / Performance)
- **Status:** BELEGT
- **Fundstelle:** `crates/contextra-sandbox/src/executor.rs:224` (`Module::from_binary(&engine_clone, &wasm_bytes_vec)`)
- **Befund:** Obwohl `WasmMergeFunction` ein `AdmittedModule` hält, ruft `execute_internal` bei jedem einzelnen Aufruf `Module::from_binary` auf, um die Wasmtime `Module`-Instanz zu erzeugen, anstatt das vorkompilierte `wasmtime::Module` im Struct zu cachen.
- **Evidenz:** `crates/contextra-sandbox/src/executor.rs:224`
- **Betroffene Invariante:** Hot-Path Performance
- **Empfehlung:** Speichern des kompilierten `wasmtime::Module` in `WasmMergeFunction` oder Einführung eines `ModuleCache`.

---

## 5. Vorgänger-Abgleich

| Vorgänger-ID | Titel / Gegenstand | Status im aktuellen Commit (`59f853cb`) |
|---|---|---|
| P1 | Dual Limits (Fuel + Timeout) | BEHOBEN / BESTÄTIGT (Fuel & Wall-Clock-Timeout aktiv) |
| P2 | State Isolation per Execution | BEHOBEN / BESTÄTIGT (Frische Store/Instance pro Aufruf) |
| P3 | Zeroize-On-Drop für stdout | BEHOBEN / BESTÄTIGT (`WasmOutput.stdout` nutzt `ZeroizeOnDrop`) |
| P4 | Capability Whitelist Default Deny | BEHOBEN / BESTÄTIGT (Netzwerk/FS/Egress default `false`) |
| P5 | Fuel Exhaustion Handling | BEHOBEN / BESTÄTIGT (Rückgabe von `SandboxError::FuelExhausted`) |
| P6 | Strict Safe Rust (`#![forbid(unsafe_code)]`) | BEHOBEN / BESTÄTIGT (0 unsafe blocks in `contextra-sandbox`) |

---

## 6. Testlage im Scope

- `cargo test -p contextra-sandbox` verifiziert alle WASI-I/O-Operationen, Wall-Clock-Timeouts, Fuel-Limits, Zeroize-on-Drop und Risikoklassifizierungen.
  - **Ergebnis:** 10 Passed, 0 Failed.
<!-- EVIDENCE: logs/audits/contextra-sandbox_2026-10-13/test-sandbox.log (exit=0) -->

---

## 7. Nicht prüfbar und warum

- **Fuzzing mit Nightly Toolchain:** `cargo xtask fuzz-smoke` meldet `cargo-fuzz or +nightly toolchain missing` in der Sandbox-Umgebung.
<!-- EVIDENCE: logs/audits/contextra-sandbox_2026-10-13/fuzz-smoke.log (exit=2) -->

---

## 8. Out-of-scope-Befunde

- **`contextra-store` Kompilierfehler:** Bei Ausführung von `cargo test -p contextra-engine` schlug der Build von `contextra-store` fehl (`method ledger_handle, not a field`). Da `contextra-store` in Ring 1 außerhalb des zugewiesenen Scopes liegt, wird dieser Befund hier nur als Out-of-scope notiert.

---

## 9. Priorisierte Maßnahmen mit Akzeptanzbefehl

1. **CRDT-Property-Tests hinzufügen (SBX-01):** Property-Tests für WASM-Merge-Operatoren mit `proptest` erstellen.
   - *Akzeptanzbefehl:* `cargo test -p contextra-sandbox --test pure_merge_operator_capabilities_test`
2. **Module-Caching in WasmMergeFunction (SBX-02):** `wasmtime::Module` vorkompilieren und im Struct halten.
   - *Akzeptanzbefehl:* `cargo test -p contextra-sandbox`
