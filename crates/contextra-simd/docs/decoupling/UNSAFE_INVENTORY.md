# Unsafe-Inventar contextra-simd (Ring 0 Unsafe-Insel)

Gemäß SPEC §3.1, JULES_RAHMEN und WP-4: Vollständiges Inventar aller `unsafe`-Konstrukte in `contextra-simd`.

## Übersicht

- **Crate-Status:** Ring 0 Unsafe-Insel (`#![allow(unsafe_code)]`, `#![deny(unsafe_op_in_unsafe_fn)]`).
- **Gesamtzahl `unsafe`-Stellen:** 272 (davon 269 Ausdrücke/Blöcke, 0 ohne `// SAFETY:`-Dokumentation laut `tools/unsafe_boundary_auditor.py`).
- **Öffentliche Schnittstelle:** Vollständig sicher (`pub fn compute_distance`, `pub fn validate_vector`, `pub struct Dispatcher`). Es gibt **keine** `pub unsafe fn` in der öffentlichen Crate-Wurzel.

## Verteilung nach Modulen

| Datei / Modul | Anzahl `unsafe` | Zweck / Natur der Operationen | Sicherheitsinvariante & Verifikation |
|---|---:|---|---|
| `src/dispatch.rs` | 42 | Dynamischer Aufruf von Kernel-Funktionen (`avx2`, `avx512`, `neon`) nach Runtime-Feature-Detection | Vor jedem Aufruf garantiert `is_x86_feature_detected!` bzw. `is_aarch64_feature_detected!`, dass der Befehlssatz auf der Host-CPU vorhanden ist. |
| `src/lib.rs` | 3 | Linter-Attribute (`#![allow(unsafe_code)]`, `#![deny(unsafe_op_in_unsafe_fn)]`) | Crate-Root Konfiguration als isolierte Unsafe-Insel. |
| `src/kernels/avx2.rs` | 108 | 256-Bit Vektor-Loads (`_mm256_loadu_ps`, `_mm256_loadu_si256`), FMA-Operationen (`_mm256_fmadd_ps`), Horizontale Summen (`hsum256`), `get_unchecked` in Restschleifen | Unaligned-Loads verhindern Alignment-Faults (`loadu`). Slice-Längen wurden vor der Schleife geprüft; Tail-Indizes liegen strikt in `0..len`. |
| `src/kernels/avx512.rs` | 94 | 512-Bit Vektor-Loads (`_mm512_loadu_ps`, `_mm512_loadu_si512`), FMA (`_mm512_fmadd_ps`), VNNI-Operationen (`_mm512_dpbusd_epi32`), Horizontale Summen (`hsum512`), `get_unchecked` in Restschleifen | `#[target_feature(enable = "avx512f", ...)]` aktiv; Unaligned-Loads (`loadu`); Restschleifen strikt innerhalb Slice-Grenzen. |
| `src/kernels/neon.rs` | 25 | 128-Bit ARM NEON Loads (`vld1q_f32`, `vld1_u8`), FMA (`vfmaq_f32`), pairwise adds (`vpaddq_f32`), `get_unchecked` in Restschleifen | Target-Feature `neon` aktiv; Unaligned Loads; Grenzen strikt eingehalten. |

## Audit-Details zu Sicherheitsanforderungen

### 1. Speicherausrichtung (Alignment)
Alle SIMD-Vektor-Ladevorgänge verwenden explizit nicht-ausgerichtete Lade-Intrinsics:
- AVX2: `_mm256_loadu_ps(ptr)`, `_mm256_loadu_si256(ptr as *const __m256i)`
- AVX-512: `_mm512_loadu_ps(ptr)`, `_mm512_loadu_si512(ptr as *const __m512i)`
- NEON: `vld1q_f32(ptr)`, `vld1_u8(ptr)`

Dadurch ist kein 32-Byte- oder 64-Byte-Alignment des übergebenen Puffers erforderlich. Unerwartete Segfaults durch unaligned Pointer sind ausgeschlossen (INV-SIMD-ALIGN-1).

### 2. Pufferlängen & Restelemente (Tail-Processing)
- Hauptschleifen iterieren mit Schrittweiten `STEP = 8` (AVX2), `STEP = 16` (AVX-512), `STEP = 4` (NEON) bis `end = len - (len % STEP)`.
- Restelemente `i in end..len` werden deterministisch skalar akkumuliert.
- `get_unchecked(i)` in der Restschleife ist bewiesen sicher, da `end <= i < len` gilt.

### 3. CPU-Feature-Schutz
Alle `#[target_feature]`-behafteten Funktionen sind `pub(crate)` und werden ausschließlich aus `src/dispatch.rs` nach Prüfung aufgerufen:
```rust
if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma") {
    // SAFETY: CPU-Features wurden zur Laufzeit positiv erkannt.
    unsafe { avx2::cosine_distance_avx2(a, b) }
}
```
Eine direkte Ausführung ohne CPU-Unterstützung (SIGILL) ist im gesamten Codebase ausgeschlossen.
