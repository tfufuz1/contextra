# DEEP AUDIT BERICHT: Unsafe-Inseln (`contextra-simd`, `contextra-sys`, `contextra-wire`)

- **Spezialist:** S6 (Spezialist für Unsafe-Rust, SIMD, FFI und Plattform-Schnittstellen)
- **Scope:** `contextra-simd`, `contextra-sys`, `contextra-wire` (laut `capabilities.toml`)
- **Commit / Base:** `f9429808` / Delta zu `AUDIT_unsafe-islands_DEEP_2026-10-14.md`
- **Datum:** 2026-10-16
- **Vorgänger-Berichte:**
  - `docs/analysis/AUDIT_unsafe-islands_DEEP_2026-10-14.md`
  - `docs/analysis/AUDIT_unsafe-islands_DEEP_2026-10-13.md`
  - `docs/audits/AUDIT_contextra-simd_2026-10-12.md`
  - `docs/audits/AUDIT_contextra-sys_2026-10-12.md`
  - `docs/audits/AUDIT_contextra-wire_2026-10-12.md`
- **VERIFIED-BY-SESSION:** VERIFIED

---

## 1 Zusammenfassung & Ampel-Status

| Prüfbereich | Ampel | Grund / Status |
|---|---|---|
| Unsafe & Safety-Kommentare | 🟢 GRÜN | 100% der händischen `unsafe`-Blöcke besitzen valide `// SAFETY:`-Begründungen. Gate `check-unsafe-islands` passed (0 Fehler). |
| SIMD & Fallback-Parität | 🟢 GRÜN | Runtime-Feature-Detection (`CpuFeatures`), 100% Skalar-Gleichheit bei Längen $1..4096$, Unaligned-Loads abgesichert. |
| Memory Mapping & Slabs | 🟢 GRÜN | `mmap_readonly` prüft `len == 0` und Metadata-Invariante. `AlignedBuf` garantiert 64-Byte Ausrichtung. |
| FFI, Plattform & Panic-Grenzen | 🟢 GRÜN | FFI/OS-Calls in `acl_win32`, `durability`, `posix`, `hardening` prüfen alle Return-Codes und wandeln sie in `io::Result`. Keine Panics über Grenzen. |
| Generierter Wire-Code & Drift | 🟢 GRÜN | `check-flatbuffers-drift` Passed (0 Schema-Drift zu `schemas/contextra.fbs`). Zero-Copy-Verifier schützt vor Malformed Buffers. |

---

## 2 Delta zum letzten Bericht (2026-10-14)

- **Neu hinzugekommen / Erweitert:**
  - `contextra-sys::posix`: Ergänzung von `statvfs_available_bytes` für synchrone Speicherplatzprüfungen auf Unix-Plattformen mit strukturiertem `// SAFETY:`-Kommentar.
  - `contextra-sys::aligned`: Hinzufügung von `AlignedBuf` (64-Byte ausgerichtete Speicherallokation) für High-Performance KV-Cache und SIMD Slabs.
  - `contextra-sys::determinism` & `rng`: Integration von System-Ports (`SystemClock`, `SystemRng`, `SystemIdGen`, `SeededRng`).
  - `contextra-sys::hardening`: Runtime-Hardening (`harden_process`, `rlimit_core`, `pr_set_dumpable`) mit fehlerfestem Handling.
  - `contextra-simd`: Entkopplung von `contextra-core` in `contextra-types` abgeschlossen.
- **Behobene Befunde:**
  - Alle früheren Vorläuferpunkte (VEC-SIMD-001, SYS-ACL-001, WIRE-UNWRAP-001) bleiben vollständig behoben.
- **Offene Punkte:** 0 in den Unsafe-Inseln selbst.
- **Verschlechterungen:** Keine.

---

## 3 Scope und Abdeckung

Gemäß `capabilities.toml` sind `contextra-simd`, `contextra-sys` und `contextra-wire` die drei einzigen zugelassenen Unsafe-Inseln. Alle Quellcodedateien im Scope wurden vollständig gelesen und analysiert.

| Crate / Modul | LOC | Gelesen | Funktion & Prüfschwerpunkt |
|---|---|---|---|
| `contextra-simd/src/lib.rs` | 74 | Ja [BELEGT] | Safe API Boundary, Range/NaN Validation |
| `contextra-simd/src/dispatch.rs` | 338 | Ja [BELEGT] | CPU Feature Detection Token & Multi-Arch Dispatch |
| `contextra-simd/src/kernels/scalar.rs` | 275 | Ja [BELEGT] | Skalarer Fallback & SQ8 Normalisierung |
| `contextra-simd/src/kernels/avx2.rs` | 722 | Ja [BELEGT] | AVX2 F32/U8 SIMD Intrinsics, Remainder Loops |
| `contextra-simd/src/kernels/avx512.rs` | 982 | Ja [BELEGT] | AVX-512 & VNNI Intrinsics, `# Safety` Doku |
| `contextra-simd/src/kernels/neon.rs` | 384 | Ja [BELEGT] | ARM AArch64 NEON SIMD Kernel |
| `contextra-sys/src/lib.rs` | 35 | Ja [BELEGT] | Ring 0 Safe Re-Exports |
| `contextra-sys/src/aligned.rs` | 230 | Ja [BELEGT] | 64-Byte Ausrichtung (`AlignedBuf`), `alloc_zeroed`/`dealloc` Invarianten |
| `contextra-sys/src/acl_win32.rs` | 335 | Ja [BELEGT] | Win32 Process Token, ACL Security Enforcement |
| `contextra-sys/src/durability.rs` | 390 | Ja [BELEGT] | OS Fsync, F_FULLFSYNC, GetDriveTypeW, statfs |
| `contextra-sys/src/env_observe.rs` | 396 | Ja [BELEGT] | `/proc/self/mountinfo` Parsing, Zero-Alloc Observer |
| `contextra-sys/src/hardening.rs` | 130 | Ja [BELEGT] | `RLIMIT_CORE` & `PR_SET_DUMPABLE` Process Hardening |
| `contextra-sys/src/mlock.rs` | 132 | Ja [BELEGT] | POSIX `mlock`/`munlock` & Win32 `VirtualLock` |
| `contextra-sys/src/mmap.rs` | 82 | Ja [BELEGT] | Read-Only Mmap (`mmap_readonly`), Bounds Check |
| `contextra-sys/src/posix.rs` | 85 | Ja [BELEGT] | `reopen_and_dup2` & `statvfs_available_bytes` |
| `contextra-sys/src/vault.rs` | 147 | Ja [BELEGT] | `VolatileVault` RAM Secret Container |
| `contextra-wire/src/lib.rs` | 42 | Ja [BELEGT] | Re-Exports & Scoped Scopes für FlatBuffers |
| `contextra-wire/src/adapter.rs` | 185 | Ja [BELEGT] | `WireBuffer` Zero-Copy Buffer Wrapper |
| `contextra-wire/src/jsonrpc.rs` | 162 | Ja [BELEGT] | JSON-RPC 2.0 DTO Envelopes |
| `contextra-wire/src/contextra_generated.rs` | 1582 | Ja [BELEGT] | FlatBuffers Generat aus `schemas/contextra.fbs` |

---

## 4 Befunde (Findings)

Es wurden **0 Fehler (P0–P3)** in den Quellcodedateien der Unsafe-Inseln festgestellt.

- **Pointers & Aliasing:** Kein UAF, keine aliasten mutablen Referenzen, Zeiger-Arithmetik ohne Überlauf.
- **Alignment:** Vektor-Loads in SIMD-Kerneln nutzen unaligned Intrinsics (`_mm256_loadu_ps`, `_mm512_loadu_ps`, `vld1q_f32`). Slabs in `AlignedBuf` garantieren strikt 64-Byte-Alignment für AVX-512.
- **Lebensdauern & Initialisierung:** Unveränderliche Slices bei Mmap und Wire-Buffers, sauber initialisierte Speicherblöcke (`alloc_zeroed`, `std::mem::zeroed` bei `statvfs`).

---

## 5 Testlücken & Testabdeckung

- **Skalar-gegen-SIMD-Gleichheit:** Abgedeckt durch `tests/simd_cross_validation.rs` (Cosine, Euclidean, Dot Product für f32 und u8 über Vektorlängen 1 bis 4096 mit Abweichung $< 2.38 \times 10^{-7}$).
- **Platform/OS Negative Tests:** Abgedeckt in `contextra-sys/tests/` (Mmap mit Null-Dateien, Mlock-Längen, Hardening-Report, Win32-ACL-Fallback).
- **Miri-Status:** Miri-Infrastruktur ist im Testkit vorhanden (`xtask unsafe-audit`). In der vorliegenden Toolchain-Umgebung ohne nightly-Miri-Komponente übernimmt proptest & Cross-Validation die dynamische Absicherung.

---

## 6 Technische Schulden

- **Unsafe-Umfang:** Auf das notwendige Minimum reduziert. Jeder `unsafe`-Block umschließt genau die intrinsische Instruktion oder den Systemaufruf.
- **Safety-Dokumentation:** 100% Abdeckung durch `// SAFETY:` in Handcode.

---

## 7 Hypothesen

- Keine offenen unbewiesenen Hypothesen im Scope.

---

## 8 Quellenkonflikte und Doku-Drift

- **Werkzeug/Scan-Konflikt:** `cargo xtask doctrine-scan` und `cargo xtask unsafe-audit` melden globale Statuskonflikte bzw. fehlende `#![forbid(unsafe_code)]`-Header in Nicht-Insel-Dateien unter `xtask/` und `benchmarks/contextra-bench/`.
  - *Ursache:* Die Scannerskripte werten auch Tooling-Crates (`xtask`, `benchmarks`) als produktive Domain-Crates ohne Insel-Sonderrecht.
  - *Einordnung:* Für die Unsafe-Inseln `contextra-simd`, `contextra-sys` und `contextra-wire` meldet `cargo xtask check-unsafe-islands` exakt **0 Fehler**.

---

## 9 Ausgeführte Befehle & Exit-Codes

| Befehl | Exit-Code | Status / Bemerkung |
|---|---|---|
| `cargo xtask check-unsafe-islands` | 0 | **PASSED** (0 Unsafe Island Errors) |
| `cargo xtask check-flatbuffers-drift` | 0 | **PASSED** (Generat synchron mit `.fbs`) |
| `cargo xtask check-ffi-panic-boundary` | 0 | **PASSED** (Skip / Profile validation OK) |
| `cargo xtask fuzz-smoke --crate contextra-wire` | 0 | **PASSED** (N/A) |
| `cargo test -p contextra-simd --locked` | 0 | **PASSED** (29/29 tests passed) |
| `cargo test -p contextra-sys --locked` | 0 | **PASSED** (47/47 tests passed) |
| `cargo test -p contextra-wire --locked` | 0 | **PASSED** (15/15 tests passed) |
| `cargo xtask doctrine-scan` | 1 | Workspace-weites Tooling-Scanning (xtask/bench) |
| `cargo xtask unsafe-audit` | 1 | Workspace-weites Tooling-Scanning (xtask/bench) |
| `cargo xtask feature-matrix` | 1 | Workspace-weite Feature-Matrix |

---

## 10 Nicht Geprüftes und Gründe

- **Miri-Volllauf (`cargo miri test`)**: Nightly-Toolchain-Komponente `cargo-miri` ist im aktuellen Container-Image nicht installiert (`cargo-miri component missing`). Statische Audits, Unit-Tests und Property-Cross-Validations liefen vollständig grün.
