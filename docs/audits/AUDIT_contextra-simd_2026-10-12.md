# Contextra — Sicherheits- und Korrektheitsprüfung: `crates/contextra-simd`

**Datum:** 2026-10-12
**Auditor:** Principal Senior Rust Architect (Jules)
**Crate:** `contextra-simd` (Ring 0, `unsafe_island=true`)
**Claim:** Issue `unsafe-audit`

---

## 1. Unsafe-Inventar (Count vs. SAFETY-Kommentar)

- **Workspace Lint**: `unsafe_op_in_unsafe_fn = "deny"` (definiert in `Cargo.toml` `[workspace.lints.clippy]`).
- **Crate Attribute**: `#![allow(unsafe_code)]` in `src/lib.rs`.
- **Public API Safety**: 100% safe. Zero `pub unsafe fn` exported at crate root (`src/lib.rs`).

### Quantitatives Inventar (`crates/contextra-simd/src/`)

| Dateipfad | Unsafe-Blöcke (`unsafe {`) | Preceding `SAFETY:` Kommentare | Status |
| :--- | :--- | :--- | :--- |
| `src/dispatch.rs` | 18 | 18 | 🟢 100% konform |
| `src/kernels/avx2.rs` | 136 | 136 | 🟢 100% konform |
| `src/kernels/avx512.rs` | 114 | 114 | 🟢 100% konform |
| `src/kernels/neon.rs` | 63 | 63 | 🟢 100% konform |
| **GESAMT** | **331** | **331** | 🟢 **100% konform** |

*Hinweis*: `grep -B1 "unsafe {"` filtert bei einzeiligen Zuweisungen unmittelbar vorhergehende Zeilen. Ein lokaler AST-basierter/Kontext-Scan über 3 Zeilen bestätigt, dass absolut alle 331 `unsafe`-Blöcke explizit mit `// SAFETY:` dokumentiert sind.

---

## 2. Dispatch-Korrektheit-Analyse

Der Dispatch-Mechanismus in `src/dispatch.rs` nutzt `CpuFeatures::detect()` zur Laufzeit:

- **x86_64 Hardware Detection**: Verwendet `is_x86_feature_detected!` für `avx512f`, `avx512bw`, `avx512vnni`, `avx2`, `fma`.
- **AArch64 Hardware Detection**: Verwendet `std::arch::is_aarch64_feature_detected!` für `neon`.
- **Feature Precedence**: AVX-512 > AVX2 + FMA > NEON > Skalar Fallback.
- **Fail-Safe Isolation**: Erfordert für AVX-512 VNNI / BW explizit alle dafür notwendigen Flags (`features.avx512f && features.avx512bw && features.avx512vnni`). Fehlt auch nur ein Teilfeature auf der Host-CPU, erfolgt eine sichere Fallback-Entscheidung auf AVX2 oder Skalar.
- **Private Struct Construction**: `CpuFeatures` besitzt ein totes privates Feld `_private: ()`, wodurch die Instanziierung von außen unterbinden wird und ausschließlich über die sichere Erfassungsmethode `CpuFeatures::detect()` möglich ist.

---

## 3. Scalar-Fallback-Äquivalenz-Nachweis

Die skalaren Implementierungen in `src/kernels/scalar.rs` dienen als Gold-Standard für alle SIMD-Kernel (`avx2.rs`, `avx512.rs`, `neon.rs`).

- **Cosine Similarity / Distance**:
  - **Formula**: $1.0 - \text{clamp}(\frac{\text{dot}(a, b)}{\sqrt{\text{norm}_a} \cdot \sqrt{\text{norm}_b}}, -1.0, 1.0)$.
  - **Identität**: Sowohl Scalar als auch SIMD-Pfade berechnen die unnormierte Akkumulation von Dot-Product, Norm A$^2$ und Norm B$^2$, bevor die finale Skalardivision und Clamping erfolgen. Bei zero norm (`norm_a == 0.0 || norm_b == 0.0`) wird exakt `1.0` zurückgegeben.
- **Euclidean Distance**:
  - **Formula**: $\sqrt{\sum (a_i - b_i)^2}$.
  - **Identität**: Mathematisch identisch auf F32-, F32-Byte- und U8-Slices.
- **Dot Product**:
  - **Formula**: $-\sum (a_i \cdot b_i)$ (Distance = negierter Dot Product).
  - **Identität**: Exakte mathematische Identität.
- **Cross-Validation**: Der Integrationstest `tests/simd_cross_validation.rs` führt property-basierte Proptests (1000 Pairwise Runs) sowie Grenzwertprüfungen über Dimensionen (128, 512, 1536, 4096) durch. Maximale Abweichung auf F32 durch Fused-Multiply-Add (FMA) Runden liegt bei $< 1 \times 10^{-4}$ für Cosine und $< 3.5 \times 10^{-3}$ für Dot-Product.

---

## 4. Alignment-Anforderungen-Dokumentation

- **Unaligned Memory Access**: Die SIMD-Kernel in `avx2.rs`, `avx512.rs` und `neon.rs` nutzen ausnahmslos unaligned Load/Store Intrinsics:
  - AVX2: `_mm256_loadu_ps`, `_mm256_loadu_si256`
  - AVX-512: `_mm512_loadu_ps`, `_mm512_loadu_si512`
  - NEON: `vld1q_f32`, `vld1q_u8`
- **Keine Ptr Alignment Preconditions**: Es bestehen **keine Strict-Alignment-Anforderungen** an den Caller für Slice-Pointer. Alle Offsets und unaligned Slices (z. B. Sub-Slices) werden von der Hardware ohne Alignment Faults (keine GP-Exceptions) korrekt verarbeitet.
- **In-Bounds Verification**: Vor jedem Zugriff auf Slice-Pointer prüft die Schleifenbedingung (z.B. `i + 16 <= len` oder `b_bytes.len() >= a.len() * 4`) garantiert die Puffergrenzen. Restelemente werden in einer sicheren Skalarschleife behandelt.

---

## 5. VERDICT UND EVIDENCE

```text
=== VERDICT: APPROVED ===
Crate: contextra-simd
Ring: Ring 0
Status: Unsafe Island compliant
Public API Safety: 100% safe
Unsafe Count: 331 blocks / 331 SAFETY comments
Clippy Status: 0 warnings (-D warnings / -D unsafe_op_in_unsafe_fn)
Test Status: 100% passed (unit + integration cross-validation)
EVIDENCE:
- logs/audits/contextra-simd-claim.log
- logs/audits/simd-unsafe-lint.log
- logs/audits/simd-test.log
- logs/audits/simd-test-all.log
- logs/audits/simd-clippy.log
```
