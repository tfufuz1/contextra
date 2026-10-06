# Review: Mission P05 — Unsafe-Code, Memory-Mapping & WASM-Sandbox-Isolation

## 1. Zusammenfassung

Die Sicherheits- und Korrektheitsprüfung im Rahmen von **Mission P05** analysierte das gesamte Workspace der air-gap-fähigen Memory-Engine **Contextra**. Im Fokus standen Unsafe-Code-Inseln (`contextra-sys`, `contextra-simd`, `contextra-wire`), Memory-Mapping via `mmap` (`contextra-sys`, `contextra-vector`, `contextra-store`), Wasmtime-Sandbox-Isolation (`contextra-sandbox`) sowie die Python-FFI-Schnittstelle (`contextra-py`).

Der Code weist eine vorbildliche Architektur durch die **Unsafe-Insel-Doktrin** auf: Höhere Abstraktionsebenen (`contextra-vector`, `contextra-store`, `contextra-sandbox`, `contextra-py`) erzwingen ausnahmslos `#![forbid(unsafe_code)]`. Dennoch wurden **2 CRITICAL/HIGH Befunde** identifiziert:
1. Fehlen vertraglicher `// SAFETY:`-Kommentare in `contextra-simd`, was den automatischen Harness-Audit `cargo xtask unsafe-audit` fehlschlagen lässt.
2. Unzureichende Prüfung der Modul-Herkunft/Signatur bei dynamischer WASM-Ausführung in `contextra-sandbox`, was die Ausführung bösartiger WASM-Payloads aus Agenten-Output ermöglicht.

Zusätzlich wurden **2 MEDIUM-Befunde** bezüglich potenzieller mmap-TOCTOU/SIGBUS-Anfälligkeit bei externer Dateitrunkierung und fehlendem Struct-Alignment-Guards dokumentiert.

---

## 2. Geprüfte Dateien

- `crates/contextra-sys/src/mmap.rs` — Low-Level Safe Facade für `memmap2::Mmap` (`mmap_readonly`).
- `crates/contextra-sys/src/mlock.rs` — Speichersperre in RAM (`libc::mlock` / `VirtualLock`).
- `crates/contextra-sys/src/acl_win32.rs` — Restriktive Win32-Dateirechte und Eigentümer-Prüfung.
- `crates/contextra-sys/src/posix.rs` — POSIX Low-Level Syscalls (`reopen_and_dup2`).
- `crates/contextra-sandbox/src/executor.rs` — Wasmtime-Ausführungsmotor mit Fuel- & Wall-Clock-Budget.
- `crates/contextra-sandbox/src/capabilities.rs` — Whitelist-Konfiguration für WASM-Capabilities.
- `crates/contextra-sandbox/src/wasi.rs` — WASI Preview1 Host-Funktionen (stdin, stdout, stderr, clock, random).
- `crates/contextra-sandbox/src/merge.rs` — Pure WASM Merge-Operator Ausführungsumgebung.
- `crates/contextra-simd/src/dispatch.rs` — Dynamic Hardware Feature Detection & SIMD-Dispatch.
- `crates/contextra-simd/src/kernels/avx2.rs` — AVX2/FMA SIMD-Intrinsics.
- `crates/contextra-simd/src/kernels/avx512.rs` — AVX512 SIMD-Intrinsics.
- `crates/contextra-simd/src/kernels/neon.rs` — ARM NEON SIMD-Intrinsics.
- `crates/contextra-wire/src/contextra_generated.rs` — Generierte FlatBuffers Unsafe-Parser-Bindings.
- `crates/contextra-vector/src/diskann/persistence.rs` — DiskANN Index mmap-Persistence und Zero-Copy Slicing.
- `crates/contextra-store/src/wal/replay.rs` — Zero-Copy WAL Replay via mmap.
- `crates/contextra-py/src/lib.rs` — PyO3 FFI Crate Boundary.

---

## 3. Extrahierte Invarianten (Phase A)

- **I-1 (Unsafe-Insel-Doktrin):** Alle `unsafe`-Operationen müssen strikt auf ausgewiesene Unsafe-Inseln (`contextra-sys`, `contextra-simd`, `contextra-wire`) beschränkt sein. Alle anderen Workspace-Crates erzwingen `#![forbid(unsafe_code)]`.
- **I-2 (Unsafe-Begründungspflicht):** Jeder `unsafe fn`, `unsafe impl` und `unsafe {}`-Block MUSS durch einen strukturierten `// SAFETY:`-Kommentar begründet sein, der die einzuhaltenden Aufrufer-Garantien explizit nennt.
- **I-3 (mmap-Lese-Invariante):** `mmap_readonly` garantiert schreibgeschütztes Mapping. Die aufrufende Komponente muss sicherstellen, dass Slice-Zugriffe Grenzen (`offset + len <= mmap.len()`) prüfen.
- **I-4 (WASM Zero-Net-Traffic / Air-Gap):** WASM-Module dürfen unter keinen Umständen Netzwerk- oder Dateisystem-Sockets öffnen. Host-Funktionen wie `host_cloud_query` müssen `allow_cloud_egress` dynamisch erzwingen, WASI-Imports für FS/Net dürfen im Linker nicht registriert werden.
- **I-5 (WASM Ressourcen-Determinismus):** Jedes WASM-Modul muss unter striktem Fuel-Budget (CPU, deterministisch) und Wall-Clock-Timeout (tokio) laufen, um Denial-of-Service (Endlosschleifen) auszuschließen.
- **I-6 (WASM Isolation):** Jede Execution startet mit einer frischen `Store` + `Instance`. Es findet kein Zustandsüberlauf zwischen Aufrufen statt.
- **I-7 (SIMD Feature Guard):** SIMD-Intrinsics (`avx2`, `avx512f`, `neon`) dürfen erst aufgerufen werden, nachdem die CPU-Unterstützung zur Laufzeit positiv geprüft wurde (`is_x86_feature_detected!`).
- **I-8 (FlatBuffers Boundaries):** Unsafe Root-Parsing in `contextra-wire` darf nur über verifizierte Buffer-Grenzen ausgeführt werden.
- **I-9 (Python FFI Exception Safety):** An der Python-FFI-Grenze dürfen keine Rust-Panics entweichen; Fehler müssen kontrolliert als PyO3 exceptions abgebildet werden.

---

## 4. Befunde

### [HIGH] F-01 — Systematisches Fehlen von `// SAFETY:`-Kommentaren in `contextra-simd`

- **Ort:** `crates/contextra-simd/src/dispatch.rs:20`, `crates/contextra-simd/src/kernels/avx2.rs:29`, `crates/contextra-simd/src/kernels/avx512.rs:18`, `crates/contextra-simd/src/kernels/neon.rs:15`
- **Invariante betroffen:** I-2
- **Beleg:**
```rust
#[inline]
pub fn cosine_distance(a: &[f32], b: &[f32]) -> Result<f32, ContextraError> {
    ...
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx512f") {
            return Ok(unsafe { avx512::cosine_distance_avx512(a, b) });
        }
        if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma") {
            return Ok(unsafe { avx2::cosine_distance_avx2(a, b) });
        }
    }
```
- **Angriffs-/Fehlerszenario:**
  1. Der Entwickler verlässt sich darauf, dass `unsafe`-Anweisungen in `contextra-simd` korrekt dokumentiert sind.
  2. In `dispatch.rs`, `avx2.rs`, `avx512.rs` und `neon.rs` fehlen jedoch bei über 300 `unsafe`-Blöcken und `unsafe fn`-Deklarationen die geforderten `// SAFETY:`-Kommentare.
  3. Der automatisierte Harness-Check `cargo xtask unsafe-audit` schlägt fehl (17 Violation Findings, Anstieg der fehlenden Kommentare auf >300).
- **Auswirkung:** Regelverstoß gegen die Doktrin `INV-SYS-UNSAFE-SAFETY-DOC` / I-2. Gate-Failures bei automatisierten CI-Audits; Audit-Unverfolgbarkeit von Zeigerarithmetik und Vector-Unpack-Operationen.
- **Empfehlung:** Ergänzen aller `unsafe fn` und `unsafe {}`-Blöcke in `contextra-simd` um präzise `// SAFETY:`-Kommentare, die den Laufzeit-Feature-Check und die Slicelängen-Gleichheit belegen.

---

### [HIGH] F-02 — Fehlende Modul-Signatur/Allowlist-Validierung bei WASM-Modulausführung

- **Ort:** `crates/contextra-sandbox/src/executor.rs:136`, `crates/contextra-sandbox/src/merge.rs:43`
- **Invariante betroffen:** I-4, I-6
- **Beleg:**
```rust
pub async fn execute(
    &self,
    wasm_bytes: &[u8],
    input: &[u8],
    capabilities: &WasmCapabilities,
    timeout: Duration,
) -> Result<WasmOutput, SandboxError> {
    ...
    if wasm_bytes.len() > capabilities.max_module_size_bytes {
        return Err(SandboxError::InvalidModule(...));
    }
    let module = Module::from_binary(&engine_clone, &wasm_bytes_vec)...
```
- **Angriffs-/Fehlerszenario:**
  1. Ein KI-Agent erzeugt über Prompt Injection beeinflussten Output, der WASM-Bytecode enthält oder referenziert.
  2. Die Anwendung übergibt das WASM-Binary direkt an `WasmExecutor::execute` oder `WasmMergeFunction::new`.
  3. Der `WasmExecutor` prüft lediglich die Dateigröße (`max_module_size_bytes`), validiert aber weder eine kryptografische Ed25519-Signatur noch eine Modul-Allowlist.
  4. Obwohl die WASI-Capabilities das Netzwerk sperren, kann bösartiger WASM-Code durch Exzessiven CPU-Verbrauch, Rechnerlast-Flutungen oder Versuche zur WASM-Engine-Bypass-Exploitation ausgeführt werden.
- **Auswirkung:** Potenzielle Ausführung ungeprüfter/bösartiger Binärdateien aus Agenten-Output (Prompt Injection Remote Code Execution Risk auf Sandbox-Ebene).
- **Empfehlung:** Einführung einer Pflicht zur kryptografischen Modul-Verifikation (z. B. Ed25519-Signaturprüfung der WASM-Bytes gegen einen vertrauenswürdigen Public Key) oder Maintainer-Allowlist-Check vor der Kompilierung in `WasmExecutor`.

---

### [MEDIUM] F-03 — Virtuelle Speicher-Anfälligkeit (SIGBUS) bei externer Dateitrunkierung von mmap-Slices

- **Ort:** `crates/contextra-sys/src/mmap.rs:14`, `crates/contextra-vector/src/diskann/persistence.rs:403`, `crates/contextra-store/src/wal/replay.rs:157`
- **Invariante betroffen:** I-3
- **Beleg:**
```rust
pub fn mmap_readonly(file: &File) -> io::Result<memmap2::Mmap> {
    // SAFETY:
    // 1. `file` is a valid open read-only file descriptor.
    // 2. Read-only mapping prevents data mutation races in Rust address space.
    // 3. Memory pages are managed safely by the kernel page tables.
    unsafe { memmap2::Mmap::map(file) }
}
```
- **Angriffs-/Fehlerszenario:**
  1. Ein Index-File (`.idx`) oder WAL-Segment wird via `contextra_sys::mmap_readonly` gemappt.
  2. Ein externer Prozess oder Admin verkürzt die Datei auf dem Dateisystem während ein Thread in Rust auf das Slice zugreift.
  3. Rust greift auf eine Page zu, die außerhalb der neuen Dateigröße liegt.
  4. Das Betriebssystem sendet ein `SIGBUS`-Signal an den Prozess, was zum sofortigen Absturz führt (Rust Borrow-Checker kann Betriebssystem-Level-Mutationen nicht verhindern).
- **Auswirkung:** Unkontrollierter Prozessabsturz (Crash) durch `SIGBUS` bei Dateisystem-Races.
- **Empfehlung:** Dokumentation und Implementierung von POSIX `SIGBUS`-Handlern/Fallbacks oder Fallback-Mechanismen auf gekapselte Stream-Reader, wo externe Dateimutationen auf ungeschützten Bänden möglich sind.

---

### [MEDIUM] F-04 — Fehlender Struct-Alignment-Guard bei Rohdaten-Casts in On-Disk-Slices

- **Ort:** `crates/contextra-vector/src/diskann/persistence.rs:414`, `crates/contextra-vector/src/diskann/node.rs:50`
- **Invariante betroffen:** I-3, I-8
- **Beleg:**
```rust
let node_data = mmap.get(offset..end_offset)?;
let doc_id_bytes = &node_data[0..8];
let doc_id = u64::from_le_bytes(doc_id_bytes.try_into().unwrap());
```
- **Angriffs-/Fehlerszenario:**
  1. `memmap2::Mmap` garantiert Seiten-Alignment (z. B. 4096 Bytes), aber beliebige Byteslices im Mapping (z. B. Offset 3) sind nicht für Typen wie `u64` oder `f32` ausgerichtet.
  2. Wenn Daten nicht per Kopie (`from_le_bytes`), sondern fälschlicherweise per Pointer-Cast (z. B. `*(ptr as *const u64)`) gelesen würden, tritt auf strikt ausgerichteten Architekturen (wie ARMv7) UB/Alignment-Fault auf.
  3. Aktuell nutzt der Code `from_le_bytes` (Sicher), aber es fehlt ein expliziter Compile-Time/Runtime Alignment Check Guard für zukünftige Direct-Structure-Pointers.
- **Auswirkung:** Wartungsrisiko; potenzieller Alignment-Fault bei unbedachten zukünftigen Unsafe-Casts.
- **Empfehlung:** Ergänzen von `bytemuck` oder `zerocopy` Byte-Alignment-Invariantenprüfungen an allen mmap-Slicing-Grenzflächen.

---

## 5. Optimierungspotenzial (Phase D)

1. **Caching der Hardware-Feature-Erkennung in `contextra-simd`:**
   - *Aktuell:* `is_x86_feature_detected!("avx2")` führt bei jedem Vektordistanzaufruf Atomare/OS-Feature-Abfragen durch.
   - *Vorschlag:* Cachen der CPU-Features in einer `static`-Variablen (`std::sync::OnceLock` oder `lazy_static`), um Overhead im Hot-Path der Vektorsuche einzusparen.
2. **Kompilierungscaching für WASM-Module in `contextra-sandbox`:**
   - *Aktuell:* `WasmExecutor::execute` kompiliert das WASM-Binary bei jedem Aufruf synchron/blocking neu.
   - *Vorschlag:* Einführen eines In-Memory `LruCache<[u8; 32], Module>` basierend auf dem SHA256-Hash des WASM-Codeblocks.

---

## 6. Offene Fragen / nicht verifizierbar ohne Laufzeit-Tests

- **Windows ACL Enforcement unter eingeschränkten Nutzerkonten:** Die Win32 ACL-Restriktion (`acl_win32.rs`) nutzt `SetNamedSecurityInfoW`. Auf Nicht-Admin-Konten kann dies unter bestimmten Gruppenrichtlinien mit `ERROR_ACCESS_DENIED` fehlschlagen, was per Laufzeittest auf Windows-Server-Instanzen verifiziert werden muss.

---

## 7. Jules-Task-Karten

```yaml
id: JULES-P05-01
title: Add missing // SAFETY: documentation comments across contextra-simd
severity: HIGH
files_to_touch:
  - crates/contextra-simd/src/dispatch.rs
  - crates/contextra-simd/src/kernels/avx2.rs
  - crates/contextra-simd/src/kernels/avx512.rs
  - crates/contextra-simd/src/kernels/neon.rs
context: >
  Over 300 unsafe blocks and functions in contextra-simd lack required // SAFETY: comments explaining
  target feature guards, slice bounds, and pointer validity. This causes cargo xtask unsafe-audit to fail
  and violates safety documentation invariant I-2.
acceptance_criteria:
  - Every unsafe fn and unsafe block in contextra-simd has a structured // SAFETY: comment.
  - cargo xtask unsafe-audit passes without missing SAFETY comment errors for contextra-simd.
  - All SIMD unit and integration tests continue to pass.
test_to_add: >
  Run cargo xtask unsafe-audit to ensure zero missing SAFETY comment findings in contextra-simd.
non_goals: >
  Do not alter any SIMD kernel logic or algorithm implementations.
```

```yaml
id: JULES-P05-02
title: Enforce cryptographic module verification or allowlist check in WasmExecutor
severity: HIGH
files_to_touch:
  - crates/contextra-sandbox/src/executor.rs
  - crates/contextra-sandbox/src/capabilities.rs
context: >
  WasmExecutor executes arbitrary WASM binaries without verifying module provenance, signatures, or allowlists.
  Dynamically executing unverified WASM modules generated from agent outputs creates a security risk under prompt injection.
acceptance_criteria:
  - WasmExecutor validates WASM module hashes against an allowed manifest or Ed25519 signature before compilation.
  - Attempts to execute unverified WASM modules fail with SandboxError::InvalidModule.
  - Existing merge function tests pass with updated module verification credentials.
test_to_add: >
  Add unit test test_untrusted_wasm_module_signature_rejected verifying that unsigned WASM modules are rejected prior to execution.
non_goals: >
  Do not change WASI host function capability boundaries or fuel limit mechanics.
```
