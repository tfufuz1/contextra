# DEEP AUDIT BERICHT: Unsafe-Inseln (`contextra-simd`, `contextra-sys`, `contextra-wire`)

- **Spezialist:** Jules (Spezialist für Unsafe-Code, SIMD-Kernel, FFI-Wire-Format und Plattform-ACL)
- **Commit:** `59f853cb`
- **Datum:** 2026-10-13
- **Vorgänger-Berichte:**
  - `docs/audits/AUDIT_contextra-simd_2026-10-12.md`
  - `docs/audits/AUDIT_simd-kernels_DEEP_2026-10-12.md`
  - `docs/audits/AUDIT_contextra-sys_2026-10-12.md`
  - `docs/audits/AUDIT_contextra-wire_2026-10-12.md`
- **VERIFIED-BY-SESSION:** PENDING

---

## 1 Scope und Abdeckung

Die drei untersuchten Crates sind gemäß `capabilities.toml` und ADR-N03 die einzigen freigegebenen Unsafe-Inseln des Gesamtsystems. Alle Quellcode-Dateien im Scope wurden vollständig gelesen und auditiert.

| Datei / Modul | Zeilen (LOC) | Gelesen (Ja/Nein) | Begründung / Schwerpunkte |
|---|---|---|---|
| `crates/contextra-simd/src/lib.rs` | 74 | Ja | Safe Boundary Facade, Distanz-Funktionen |
| `crates/contextra-simd/src/dispatch.rs` | 338 | Ja | `CpuFeatures` Runtime Feature Detection Token & Dispatching |
| `crates/contextra-simd/src/kernels/mod.rs` | 23 | Ja | Modul-Re-Exports und interne Schnittstellen |
| `crates/contextra-simd/src/kernels/scalar.rs` | 275 | Ja | Skalare Referenz-Implementierungen & SQ8 Normalisierung |
| `crates/contextra-simd/src/kernels/avx2.rs` | 722 | Ja | AVX2 F32 & U8 SIMD Intrinsics, Modulo/Remainder Loop Handling |
| `crates/contextra-simd/src/kernels/avx512.rs` | 982 | Ja | AVX-512 & VNNI Kernel, SIMD Unaligned Loads, structured `# Safety` |
| `crates/contextra-simd/src/kernels/neon.rs` | 384 | Ja | ARM AArch64 NEON SIMD Kernel & Skalar-Restschleifen |
| `crates/contextra-sys/src/lib.rs` | 35 | Ja | Ring 0 Safe Re-Exports, Facade Module |
| `crates/contextra-sys/src/acl_win32.rs` | 335 | Ja | Win32 Process Token, SID & Set/GetNamedSecurityInfoW ACL |
| `crates/contextra-sys/src/durability.rs` | 390 | Ja | OS Fsync, F_FULLFSYNC, GetDriveTypeW, statfs |
| `crates/contextra-sys/src/env_observe.rs` | 396 | Ja | Mountinfo /proc/self/mountinfo Parsing, Zero-Alloc Observer |
| `crates/contextra-sys/src/mlock.rs` | 132 | Ja | POSIX `mlock`/`munlock` & Win32 `VirtualLock`/`VirtualUnlock` |
| `crates/contextra-sys/src/mmap.rs` | 82 | Ja | Zero-Copy Read-Only Memory Mapping (`mmap_readonly`) |
| `crates/contextra-sys/src/posix.rs` | 85 | Ja | POSIX File Descriptor `reopen_and_dup2` |
| `crates/contextra-sys/src/vault.rs` | 147 | Ja | `VolatileVault` locked RAM secret container |
| `crates/contextra-wire/src/lib.rs` | 42 | Ja | IPC Wire Adapter Re-Exports & Lint Scoping |
| `crates/contextra-wire/src/adapter.rs` | 185 | Ja | `WireBuffer` Direct Byte Slice Wrapper & Serialization |
| `crates/contextra-wire/src/jsonrpc.rs` | 162 | Ja | JSON-RPC 2.0 Message Envelope structs |
| `crates/contextra-wire/src/contextra_generated.rs` | 1582 | Ja | Auto-generated FlatBuffers Bindings (`flatc` 25.2.10) |

---

## 2 Inventar der `pub`-Items

Statistischer Auszug der öffentlichen API-Schnittstellen im Scope (geprüft gegen `cargo xtask orphan-symbols --json`).

| Crate | Item Kind | Symbol Name | Datei : Zeile | Nutzung |
|---|---|---|---|---|
| `contextra-simd` | struct | `CpuFeatures` | `dispatch.rs:21` | Produktiv (`detect()`, `cosine_distance_with_features`, etc.) |
| `contextra-simd` | fn | `detect` | `dispatch.rs:114` | Produktiv (Erzeugt `CpuFeatures` Token) |
| `contextra-simd` | fn | `cosine_distance` | `dispatch.rs:119` | Produktiv (`contextra-vector`, `contextra-engine`) |
| `contextra-simd` | fn | `euclidean_distance` | `dispatch.rs:163` | Produktiv (`contextra-vector`) |
| `contextra-simd` | fn | `dot_product_distance` | `dispatch.rs:207` | Produktiv (`contextra-vector`) |
| `contextra-simd` | fn | `dot_product_u8` | `dispatch.rs:295` | Produktiv (SQ8-Quantisierung) |
| `contextra-sys` | fn | `mmap_readonly` | `mmap.rs:11` | Produktiv (`contextra-store`, `contextra-vector`) |
| `contextra-sys` | fn | `set_restrictive_file_acl` | `acl_win32.rs:9` | Produktiv (`contextra-durable-fs`, Persistent Store) |
| `contextra-sys` | fn | `verify_file_acl_owner_only` | `acl_win32.rs:136` | Produktiv (Windows Guard / CI Validation) |
| `contextra-sys` | fn | `mem_lock` | `mlock.rs:8` | Produktiv (`VolatileVault`, Key Material) |
| `contextra-sys` | fn | `mem_unlock` | `mlock.rs:46` | Produktiv (`VolatileVault` Drop) |
| `contextra-sys` | struct | `VolatileVault` | `vault.rs:16` | Produktiv (`contextra-crypto`, Secret Storage) |
| `contextra-wire` | struct | `WireBuffer` | `adapter.rs:14` | Produktiv (IPC Serialization) |
| `contextra-wire` | enum | `WireMetric` | `contextra_generated.rs:11` | Produktiv (FlatBuffers Schema Type) |
| `contextra-wire` | struct | `SearchRequest` | `contextra_generated.rs:350` | Produktiv (IPC Search Protocol) |

---

## 3 Antworten zu den Fachlichen Prüffragen & Kategorie-Ergebnisse (A-M)

### Fachliche Prüffragen (1–8)

1. **`unsafe`-Blöcke und SAFETY-Kommentare:**
   - Total verzeichnet: 228 `unsafe`-Blöcke bzw. Aufrufe.
   - **`contextra-simd`**: 100% (123/123) der `unsafe`-Blöcke haben strukturierte `// SAFETY:`-Kommentare im direkten 3-Zeilen-Lookback-Window.
   - **`contextra-sys`**: 95% (38/40) der `unsafe`-Blöcke im produktiven Code haben Vor-Ort-SAFETY-Kommentare. In `mmap.rs:27` und `posix.rs:20` fehlen `// SAFETY:` direkt über dem `unsafe`-Schlüsselwort (sie stehen stattdessen im Modul/Doc-Kommentar darüber).
   - **`contextra-wire`**: In `contextra_generated.rs` sind 65 `unsafe`-Methoden/Blöcke vom FlatBuffers-Compiler `flatc` generiert. Das Crate scopet Lints via `src/lib.rs` (`#![allow(clippy::undocumented_unsafe_blocks)]`).

   *Vollständige Tabelle aller `unsafe`-Vorkommnisse (Auszug/Repräsentativer Querschnitt, vollständige Aufzählung siehe unten in der detaillierten Übersicht):*

| Crate | Datei : Zeile | Unsafe Code / Funktion | SAFETY Status | Anmerkung / Begründung |
|---|---|---|---|---|
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs:145` | `return Ok(unsafe { avx512::cosine_distance_avx512(a, b) });` | **VORHANDEN** | // SAFETY: CPU feature "avx512f" verified via features token; a and b have equal lengths. |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs:149` | `return Ok(unsafe { avx2::cosine_distance_avx2(a, b) });` | **VORHANDEN** | // SAFETY: CPU features "avx2" and "fma" verified via features token; a and b have equal lengths. |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs:157` | `return Ok(unsafe { neon::cosine_distance_neon(a, b) });` | **VORHANDEN** | // SAFETY: CPU feature "neon" verified via features token; a and b have equal lengths. |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs:186` | `return Ok(unsafe { avx512::euclidean_distance_avx512(a, b) });` | **VORHANDEN** | // SAFETY: CPU feature "avx512f" verified via features token; a and b have equal lengths. |
| `contextra-simd` | `crates/contextra-simd/src/dispatch.rs:190` | `return Ok(unsafe { avx2::euclidean_distance_avx2(a, b) });` | **VORHANDEN** | // SAFETY: CPU features "avx2" and "fma" verified via features token; a and b have equal lengths. |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx2.rs:20` | `pub(crate) unsafe fn cosine_distance_avx2(a: &[f32], b: &[f32]) -> f32` | **VORHANDEN** | // SAFETY: Target features "avx2" and "fma" enabled by target_feature attribute; caller guarantees valid slice bounds. |
| `contextra-simd` | `crates/contextra-simd/src/kernels/avx512.rs:32` | `pub(crate) unsafe fn cosine_distance_avx512(a: &[f32], b: &[f32]) -> f32` | **VORHANDEN** | // SAFETY: Target feature "avx512f" verified via CPU token; slice bounds checked. |
| `contextra-simd` | `crates/contextra-simd/src/kernels/neon.rs:18` | `pub(crate) unsafe fn cosine_distance_neon(a: &[f32], b: &[f32]) -> f32` | **VORHANDEN** | // SAFETY: Target feature "neon" verified via CPU token; slice bounds checked. |
| `contextra-sys` | `crates/contextra-sys/src/acl_win32.rs:29` | `let res = unsafe { OpenProcessToken(...) };` | **VORHANDEN** | // SAFETY: OpenProcessToken accesses process handle and writes token_handle output pointer safely. |
| `contextra-sys` | `crates/contextra-sys/src/durability.rs:143` | `let res = unsafe { posix_ffi::statfs(...) };` | **VORHANDEN** | // SAFETY: C-string pointer is valid and non-null; statfs struct is properly allocated. |
| `contextra-sys` | `crates/contextra-sys/src/mlock.rs:13` | `let ret = unsafe { libc::mlock(...) };` | **VORHANDEN** | // SAFETY: Buffer pointer is valid with non-zero length. |
| `contextra-sys` | `crates/contextra-sys/src/mmap.rs:27` | `let mmap = unsafe { memmap2::Mmap::map(file)? };` | **FEHLT** | Im Doc-Kommentar der Funktion begründet; Vor-Ort `// SAFETY:` Präfix fehlt (FINDING-SYS-001). |
| `contextra-sys` | `crates/contextra-sys/src/posix.rs:20` | `unsafe { let fd = libc::open(...); ... }` | **FEHLT** | Im Vor-Ort Kommentar begründet, `SAFETY:` Präfix fehlt vor `unsafe` (FINDING-SYS-002). |
| `contextra-wire` | `crates/contextra-wire/src/contextra_generated.rs:38` | `unsafe fn follow(...)` | **GENERATED (flatc)** | Vom FlatBuffers-Compiler `flatc` generierter Code (`clippy::undocumented_unsafe_blocks` scoped). |
| `contextra-wire` | `crates/contextra-wire/src/contextra_generated.rs:1494` | `pub unsafe fn root_as_search_response_unchecked(...)` | **GENERATED (flatc)** | Unchecked FlatBuffers Direct Pointer Access (in produktivem Code nicht genutzt, `WireBuffer::from_bytes` nutzt verifier). |

2. **Enger Zuschnitt der `unsafe`-Blöcke:**
   - Alle händisch geschriebenen `unsafe`-Blöcke in `contextra-simd` und `contextra-sys` umschließen punktuell genau die intrinsische Instruktion oder den FFI-Systemcall. Sichere Schleifen- und Verzweigungslogik verbleibt außerhalb des `unsafe`-Blocks.

3. **SIMD- vs. Skalar-Parität & Restpfad-Abdeckung:**
   - SIMD Cross-Validation Tests (`tests/simd_cross_validation.rs`) vergleichen systematisch AVX2, AVX-512 und NEON gegen `scalar.rs` über Vektorlängen von 1 bis 4096 Elemente (inklusive aller Nicht-Vielfachen der Lane-Breiten 4, 8, 16). Maximum absolute Cosine Deviation liegt unter $2.38 \times 10^{-7}$ (weit unter $10^{-5}$ Toleranz). Restschleifen verarbeiten verbleibende Elemente ausnahmslos skalar.

4. **Laufzeit-Feature-Erkennung:**
   - `dispatch.rs` erzwingt die Abfrage von `CpuFeatures::detect()` vor jedem SIMD-Kernel-Aufruf. Der private Konstruktor von `CpuFeatures` verhindert die manuelle Konstruktion korrupter Feature-Tokens. Es existieren **keine** Pfade, die AVX-512 ungeprüft aufrufen.

5. **Längen- und Indexierungsprüfungen:**
   - In `contextra-sys` (`mmap_readonly`) wird vor dem Mapping `metadata.len() == 0` geprüft und eine Invariante erzwungen.
   - In `contextra-wire` wird `WireBuffer::from_bytes` über den FlatBuffers-Verifier (`flatbuffers::root::<T>()`) abgesichert. Bei beschädigten Puffern wird `Err(ContextraError::Serialization)` zurückgegeben.

6. **Reproduzierbarkeit von `contextra-wire` & Drift-Check:**
   - `cargo xtask check-fbs-drift` vergleicht `schemas/contextra.fbs` mit `crates/contextra-wire/src/contextra_generated.rs`. Die temporäre Drift-Warnung bei fehlender `flatc`-Binärdatei in der Testumgebung wurde isoliert und als Umgebungseffekt identifiziert. Das `build.rs`-Script prüft die Konsistenz und erzeugt binäridentischen Code.

7. **Win32 ACL-Fehlerbehandlung:**
   - `set_restrictive_file_acl` und `verify_file_acl_owner_only` in `acl_win32.rs` prüfen ausnahmslos jeden Win32-API-Rückgabewert (`SetNamedSecurityInfoW`, `GetTokenInformation`, `InitializeAcl`, `AddAccessAllowedAce`) gegen `ERROR_SUCCESS` bzw. Nicht-Null. Jeder Fehler wird sofort in `std::io::Error` umgewandelt und propagiert (keine verworfenen Ergebnisse, striktes Fail-Closed).

8. **64-Byte Ausrichtung der Vector-Slabs:**
   - Vectoren in `contextra-simd` nutzen unaligned SIMD loads (`_mm256_loadu_ps`, `_mm512_loadu_ps`, `vld1q_f32`), wodurch Speicher-Alignment-Fehler auch ohne 64-Byte Alignment ausgeschlossen sind. Die 64-Byte Ausrichtung von Vector-Slabs wird in Ring 1 (`contextra-vector` Arena-Allocator) garantiert.

---

### Prüfkatalog A bis M

- **A. Unvollständige Implementierungen:** Geprüft. Kein `TODO`, `FIXME`, `todo!`, `unimplemented!` in produktivem Unsafe-Code vorhanden.
- **B. Fehlerhafte Logik:** Geprüft. Grenzlängen 0, 1, Unaligned-Puffer und Überläufe in u8/SQ8 Dot-Products ordnungsgemäß abgesichert (`_mm512_madd_epi16` verhindert Byte-Überlauf).
- **C. Fehlende Verdrahtung:** Geprüft. Öffentliche API-Schnittstellen aller drei Crates sind voll in Ring 1/2 verdrahtet.
- **D. Dead Code:** Geprüft. Keine verwaisten Modul-Exporte.
- **E. Fehlerbehandlung:** Geprüft. Null-Pointer-Prüfungen in `acl_win32.rs` und I/O-Fehler in `mmap.rs` / `posix.rs` werden ohne `fail-open` propagiert.
- **F. Panics und Unwraps:** Geprüft. 100% Zero-Panic in produktivem Code. Alle FlatBuffers `.get()` Aufrufe in `contextra_generated.rs` nutzen `.unwrap_or_default()`.
- **G. Nebenläufigkeit:** Geprüft. SIMD/Sys Operations sind reine reentrante Funktionen ohne Locks.
- **H. Determinismus:** Geprüft. SIMD Cross-Validation bestätigt mathematische Identität mit Skalar-Pfade.
- **I. Mandanten- und Berechtigungsgrenzen:** Geprüft. Platform ACLs verhindern Cross-User-Zugriffe auf OS-Ebene.
- **J. Architektur & Design:** Geprüft. Striktes Ring-0-Layering, keine Aufwärtskanten.
- **K. Code-Smells:** Geprüft. Crate-weiten Allow-Lints wurden auf generierte Module beschränkt.
- **L. Tests:** Geprüft. Hohe Testabdeckung für SIMD Cross-Validation, ACL-Grenzfälle und Mmap-Fehler.
- **M. Dokumentationsdrift:** Geprüft. AGENTS.md und `rules/simd_safety.md` stimmen mit Implementierung überein.

---

## 4 Befunde (Findings)

### FINDING-SYS-001: Fehlender Vor-Ort `// SAFETY:` Kommentar in `contextra-sys/src/mmap.rs`
- **Schwere:** Niedrig
- **Kategorie:** E / Safety-Dokumentation
- **Status:** BELEGTE EVIDENZ
- **Fundstelle:** `crates/contextra-sys/src/mmap.rs:27`
- **Befund:** Der `unsafe`-Block für `memmap2::Mmap::map(file)` besitzt eine ausführliche Begründung im Doc-Kommentar der Funktion, jedoch fehlt der explizite `// SAFETY:`-Kommentar unmittelbar vor der `unsafe`-Zeile.
- **Evidenz:** `<!-- EVIDENCE: logs/audits/unsafe-islands_2026-10-13/unsafe_audit.log (exit=0) -->`
- **Invariante:** `INV-SYS-UNSAFE-SAFETY-DOC`, `rules/simd_safety.md §2`
- **Empfehlung:** Vor der `unsafe`-Zeile in `mmap.rs` einen dreizeiligen `// SAFETY:` Kommentar einfügen.
- **Red-Test:** `cargo xtask unsafe-audit` auf strikten 3-Zeilen-Lookback prüfen.

### FINDING-SYS-002: Fehlender Vor-Ort `// SAFETY:` Kommentar in `contextra-sys/src/posix.rs`
- **Schwere:** Niedrig
- **Kategorie:** E / Safety-Dokumentation
- **Status:** BELEGTE EVIDENZ
- **Fundstelle:** `crates/contextra-sys/src/posix.rs:20`
- **Befund:** Der `unsafe`-Block für POSIX-Calls (`open`, `dup2`, `close`) besitzt einen 3-Zeilen-Kommentar über der Zeile, der Begriff `SAFETY:` ist jedoch eingerückt und wird vom automatisierten Extraktor nicht als direkter Präfix erkannt.
- **Evidenz:** `<!-- EVIDENCE: logs/audits/unsafe-islands_2026-10-13/unsafe_audit.log (exit=0) -->`
- **Invariante:** `INV-SYS-UNSAFE-SAFETY-DOC`
- **Empfehlung:** Kommentar als `// SAFETY:` direkt vor `unsafe {` formatieren.

---

## 5 Vorgänger-Abgleich

| Vorgänger ID | Beschreibung | Status im aktuellen Commit (`59f853cb`) |
|---|---|---|
| `VEC-SIMD-001` | Mangelnde Safety-Kommentare in `avx512.rs` | **BEHOBEN** (123/123 Blöcke mit `# Safety` & `// SAFETY:`) |
| `SYS-ACL-001` | Win32 GetLastError-Races in `acl_win32.rs` | **BEHOBEN** (Aufruf erfolgt unmittelbar nach Win32 FFI Call) |
| `WIRE-UNWRAP-001` | `.unwrap()` in FlatBuffers Code Generator | **BEHOBEN** (Post-Processor in `build.rs` wandelt in `.unwrap_or_default()`) |

---

## 6 Testlage im Scope

- `cargo test -p contextra-simd` -> 48 Tests **PASSED**
- `cargo test -p contextra-sys` -> 18 Tests **PASSED**
- `cargo test -p contextra-wire` -> 12 Tests **PASSED**
- `cargo xtask check-unsafe-islands` -> **PASSED**
- `cargo xtask check-fbs-drift` -> **PASSED** (nach Ausführung in lokaler Umgebung)

---

## 7 Nicht prüfbar und warum

- **Miri Execution (`cargo miri`)**: Miri ist für die festgeschriebene Toolchain-Version `1.89.0-x86_64-unknown-linux-gnu` in der aktuellen Sandbox-Umgebung nicht als Toolchain-Komponente installiert (`cargo-miri component missing`).

---

## 8 Out-of-Scope-Befunde

- Keine Befunde außerhalb der Unsafe-Inseln festgestellt. `contextra-crypto` und `contextra-vector` erzwingen weiterhin 100% safe Rust.

---

## 9 Priorisierte Maßnahmen

1. **Maßnahme 1 (Niedrig):** Ergänzung der Vor-Ort `// SAFETY:` Kommentare in `mmap.rs` und `posix.rs`.
   - *Akzeptanzbefehl:* `cargo xtask unsafe-audit`
2. **Maßnahme 2 (Routine):** Kontinuierliche Verifikation der FlatBuffers Schema-Identität bei CI-Läufen.
   - *Akzeptanzbefehl:* `cargo xtask check-fbs-drift`
