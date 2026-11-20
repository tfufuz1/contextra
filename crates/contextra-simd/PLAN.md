# contextra-simd — Implementierungsspezifikation

**Ebene:** E0 (Fundament)
**Status:** Normativ, abgeleitet aus `docs/SPEC.md` v4.1
**Version:** 4.1
**Geltungsbereich:** `crates/contextra-simd`
**Normative Quellen:** SPEC 3.1 (Zeile `contextra-simd`), 3.2 (R3, R5, R7), 3.3, 4.4, 6.3 (REQ-VEC-1, REQ-VEC-1a, REQ-VEC-2), 11.1 (T-RAG-04), 15, ADR-04, ADR-19, ADR-51, ADR-53, ADR-54, G-02, G-03, G-06, G-07, G-13, REM-23, CMP-7, REM-27.

---

## 1. Zweck und Ebene

`contextra-simd` liefert die Distanzkerne für Vektoren: SIMD-Implementierungen mit Laufzeit-Erkennung und eine skalare Referenz. Das Crate kennt keine Metrik-Auswahl, keine Vektorsuche und keine Speicherung. Es ist ein reines Rechenmodul mit einer `unsafe`-Insel nach R5.

Die Entscheidung, welche Metrik ein Index benutzt, trifft `contextra-vector`. Dort liegt auch die Zuordnung „Metrik → Kern“ (REM-23).

### 1.1 Ist-Befund (gemessen am HEAD `f8b44b2`)

| Messgröße | Wert | Befund |
|---|---|---|
| Quelldateien | 21 `.rs`, 6.461 Zeilen (inkl. Tests, Benches, Beispielen) | Kernel-Dateien: `kernels/{scalar,avx2,avx512,neon,mod}.rs`, `dispatch.rs`, `metric.rs`, `contract.rs`, `error.rs`, `compat/mod.rs` |
| Abhängigkeiten | `contextra-types` (optional), `tracing` (optional), `serde` (optional), `bytemuck`, `thiserror` | `contextra-types` verletzt R7 und ist die Ursache von `compat/` (CMP-7). `tracing` gehört nicht hierher |
| Features | `default = ["std"]`, `std`, `serde`, `workspace`, `types-compat`, `simd-avx512` | `workspace` und `types-compat` verletzen R7 und ADR-54. `simd-avx512` ist eine Kernel-Weiche (siehe 5) |
| Öffentliche Funktionen | ca. 40 Funktionen, darunter `*_scalar`, `*_trusted`, `*_f32_bytes`, `*_u8`, `*_parts` | Paarweise Duplikate (z. B. `cosine_distance` und `cosine_distance_f32_bytes`). Ein Kern hat viele Einstiegspunkte |
| `DistanceMetric` | in `metric.rs` definiert, zusätzlich in `contextra-types` und `contextra-vector/src/types.rs` | Dreifach definiert. R4 verletzt |
| Verbraucher außerhalb | 3 Dateien: `vector/src/distance/dispatch.rs`, `vector/tests/simd_differential_test.rs`, `vector/tests/proof_simd_dispatch_legacy.rs` | Der Verbraucher `vector` wird neu geschrieben. Die Schnittstelle richtet sich nach Abschnitt 2 |
| `unsafe` | Ja, in `kernels/avx2.rs`, `kernels/avx512.rs`, `kernels/neon.rs` | Erlaubte Insel nach R5 |
| Tests | 98 bestanden (`differential_tests`, `kernel_reference_tests`, `simd_cross_validation`, weitere) | Grün |

Ergebnis: Der Kern ist technisch in Ordnung. Die Einstiegspunkte sind zahlreich, und die Metrik-Typisierung ist dreifach verteilt.

---

## 2. Vertrag (öffentliche API)

Die API stellt nur Kerne bereit, keine Metriken. Alle Kerne haben eine Länge-Prüfung und liefern `Result`.

```rust
// Fehler. Keine Vektorinhalte im Fehler.
#[non_exhaustive]
pub enum SimdError {
    LengthMismatch { left: usize, right: usize },
    NonFinite,            // NaN oder ±∞ in der Eingabe
    EmptyInput,
}

// Fp32-Kerne (exakte Nachberechnung, REQ-VEC-3)
pub fn dot_f32(a: &[f32], b: &[f32]) -> Result<f32, SimdError>;
pub fn l2_sq_f32(a: &[f32], b: &[f32]) -> Result<f32, SimdError>;
pub fn norm_sq_f32(a: &[f32]) -> Result<f32, SimdError>;

// Asymmetrisch: fp32-Anfrage gegen SQ8-Code (REQ-VEC-1, REQ-VEC-1a)
// Dequantisierung pro Dimension: x = code * scale[i] + offset[i]
pub fn dot_f32_sq8(q: &[f32], code: &[u8], scale: &[f32], offset: &[f32]) -> Result<f32, SimdError>;
pub fn l2_sq_f32_sq8(q: &[f32], code: &[u8], scale: &[f32], offset: &[f32]) -> Result<f32, SimdError>;

// Symmetrisch: SQ8 gegen SQ8 (Navigation im HNSW)
pub fn l2_sq_sq8(a: &[u8], b: &[u8]) -> Result<u32, SimdError>;
pub fn dot_sq8(a: &[u8], b: &[u8]) -> Result<u32, SimdError>;

// Laufzeit-Diagnose ohne Nutzdaten
pub fn active_target() -> Target;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target { Scalar, Avx2, Avx512, Neon }

// Nur Testbau (Feature `test-override`): erzwingt einen Kern für Differentialtests
#[cfg(feature = "test-override")]
pub fn force_target(t: Target) -> Result<(), SimdError>;
pub mod reference { /* skalare Referenzkerne, alle obigen Funktionen */ }
```

**Begründung der Reduktion.** Die Liste oben ersetzt rund 40 öffentliche Funktionen. Die `*_trusted`-Varianten entfallen: Eingaben werden immer geprüft. Die Kosten der Prüfung sind bei Vektoren mit 384 bis 768 Dimensionen gering gegenüber dem Kern. Die `*_parts`-Varianten (Kosinus aus Teilsummen) entfallen. Der Aufrufer berechnet Normen über `norm_sq_f32`.

`DistanceMetric` ist nicht Teil der API. Metrik-Zuordnung liegt in `contextra-vector` (REM-23).

---

## 3. Abhängigkeiten

| Crate | Status | Begründung |
|---|---|---|
| `bytemuck` | behalten | Umwandlung `&[u8]` ↔ `&[f32]` ohne `unsafe` im Aufrufer |
| `thiserror` | behalten | `SimdError` |
| `contextra-types` | **entfernen** (REM-27, R7) | Keine Abhängigkeit auf den Typ-Sammelort |
| `tracing` | **entfernen** | Kein Logging in Kernen |
| `serde` | **entfernen** | Keine Serialisierung von Kernen oder Fehlern |
| `criterion` | **entfernen aus dev-deps** | Benchmarks gehören nach `contextra-bench` (SPEC 3.1) |
| `proptest` | behalten (dev) | Differentialtests |
| `simsimd` | behalten (dev) | Unabhängige Referenz für Differentialtests (nur Test) |
| `serde_json` | **entfernen aus dev-deps** | keine Fixture-Serialisierung nötig, Fixtures als `&'static [f32]` |

Interne Abhängigkeiten: keine (SPEC 3.3, `simd` = `[]`).

---

## 4. Module

```
src/
  lib.rs              # Re-Exports, #![deny(unsafe_op_in_unsafe_fn)], Modul-Dokumentation
  error.rs            # SimdError
  target.rs           # Target, Laufzeit-Erkennung (is_x86_feature_detected!, aarch64 NEON)
  dispatch.rs         # OnceLock<&'static Kernels>, Auswahl beim ersten Aufruf
  validate.rs         # Längen- und NaN-Prüfung
  kernels/
    mod.rs            # Kernel-Tabelle (Struct mit Funktionszeigern)
    scalar.rs         # Referenz, keine unsafe
    avx2.rs           # unsafe, #[target_feature(enable = "avx2,fma")]
    avx512.rs         # unsafe, #[target_feature(enable = "avx512f")]
    neon.rs           # unsafe, aarch64
  reference.rs        # re-export von kernels::scalar
tests/
  differential.rs     # jede SIMD-Variante gegen scalar, Eingaben aus proptest
  reference_values.rs # feste Erwartungswerte aus simsimd
  validation.rs       # Längen, NaN, leere Eingaben
  unsafe_islands.rs   # T-SIMD-06
```

Entfallen: `contract.rs`, `metric.rs`, `compat/mod.rs`, `dispatch.rs`-Teile zu `set_dispatch_override`/`get_dispatch_override` (nur `force_target` bleibt), `tests/building_block_conformance.rs`, `tests/edge_cases_tests.rs`, `tests/golden_distance_oracle.rs`, `tests/quantized_and_mixed_precision_tests.rs`, `tests/simd_cross_validation.rs`, `examples/`, `benches/`.

---

## 5. Entfernungen

| ID | Entfernen | Grund | Ersatz | Verbraucher |
|---|---|---|---|---|
| E-SIMD-01 | `compat/mod.rs` vollständig (`From`-Impls zu `contextra_types`, Tests mit `ContextraError`) | R7, ADR-51, CMP-7 | keine. Die Fehlerabbildung macht `vector` | 0 außerhalb |
| E-SIMD-02 | Features `workspace`, `types-compat`, `serde` | R7, ADR-54 | keine | 0 |
| E-SIMD-03 | Feature `simd-avx512` | ADR-54: keine Feature-Weiche, die einen zweiten Pfad für denselben Kern einschaltet. Die Auswahl erfolgt zur Laufzeit (`is_x86_feature_detected!("avx512f")`) | Laufzeit-Dispatch | 0 |
| E-SIMD-04 | `metric.rs` (`DistanceMetric`) | R4: drei Definitionen im Workspace. Metrik-Zuordnung gehört `vector` | `vector::DistanceMetric` (in Vector-PLAN) | 2 (über `compute_distance`) |
| E-SIMD-05 | `contract.rs` (`DistanceFn`, `MetricDistanceFn`) | ADR-19, REM-14. Trait-Vertrag ohne zweiten Implementierer | Funktionen aus Abschnitt 2 | 0 |
| E-SIMD-06 | Öffentliche Funktionen `*_trusted`, `*_parts`, `*_u8` ohne Längenprüfung, `compute_distance`, `compute_distance_f32_bytes`, `cosine_distance_f32_bytes`, `normalize_inplace`, `validate_vector` (öffentlich) | Duplikate (siehe Abschnitt 2) | Abschnitt 2 | 3 Dateien in `vector` |
| E-SIMD-07 | `set_dispatch_override`, `get_dispatch_override`, `SimdPathOverride` als öffentliche Funktionen | Ersetzt durch `force_target` unter Feature `test-override` | `force_target` | 2 Tests in `vector` |
| E-SIMD-08 | Abhängigkeiten `tracing`, `serde`, `contextra-types` | R7, Abschnitt 3 | keine | 0 |
| E-SIMD-09 | `examples/standalone.rs`, `examples/standalone_simd.rs`, `benches/simd_bench.rs` | SPEC 3.1: Benchmarks in `contextra-bench` | `contextra-bench` | 0 |
| E-SIMD-10 | Tests `building_block_conformance.rs`, `edge_cases_tests.rs`, `golden_distance_oracle.rs`, `quantized_and_mixed_precision_tests.rs`, `simd_cross_validation.rs` | Prüfen entfernte Funktionen oder Duplikate. Relevante Fälle werden in Abschnitt 7 übernommen | Abschnitt 7 | 0 |
| E-SIMD-11 | Erklärende ANCHOR-, LINK-, FILE-CONTEXT-Kommentare | ADR-19, REM-14 | Dokumentation | 0 |

---

## 6. Umbau (je Schritt ein Commit, Workspace grün)

Der einzige Verbraucher ist `contextra-vector`. Bis dessen PLAN umgesetzt ist, kompiliert `vector` gegen die alten Funktionsnamen. Um die Regel „Workspace grün je Commit“ einzuhalten, werden die Verbraucher in `vector` im selben Commit angepasst wie die Entfernung.

| Schritt | Inhalt | Verbraucher-Anpassung | Prüfung |
|---|---|---|---|
| U-SIMD-1 | Features `workspace`, `types-compat`, `serde`, `simd-avx512` entfernen. Abhängigkeiten `contextra-types`, `tracing`, `serde` entfernen. `compat/` löschen (E-SIMD-01 bis E-SIMD-03, E-SIMD-08) | keine außerhalb `vector` | `cargo check -p contextra-simd` |
| U-SIMD-2 | `dispatch.rs` auf `OnceLock` mit Laufzeit-Erkennung umstellen. AVX-512 wird ohne Feature gewählt, wenn die CPU `avx512f` hat. Die Rust-1.89-Stabilisierung der AVX-512-Intrinsics wird beim Umbau am Toolchain verifiziert, bevor der Kern ohne `nightly` kompiliert | keine | `cargo test -p contextra-simd` |
| U-SIMD-3 | Neue API aus Abschnitt 2 einführen (`dot_f32`, `l2_sq_f32`, `norm_sq_f32`, SQ8-Varianten). Scalar-Referenz in `kernels/scalar.rs` mit Längenprüfung | `vector/src/distance/dispatch.rs` ruft die neuen Funktionen auf. Metrik-Zuordnung wechselt nach `vector` (E-SIMD-04) | `cargo check -p contextra-vector` |
| U-SIMD-4 | Alte öffentliche Funktionen löschen (E-SIMD-06, E-SIMD-07) | die beiden `vector`-Tests auf `force_target` umstellen | `grep -rn "compute_distance\|_trusted\|set_dispatch_override" crates` leer außer `vector` nach Umbau |
| U-SIMD-5 | Tests und Beispiele bereinigen (E-SIMD-09, E-SIMD-10). Differentialtests neu nach Abschnitt 7 | keine | `cargo test -p contextra-simd --all-features` |
| U-SIMD-6 | `criterion` und `serde_json` aus dev-deps entfernen | keine | `cargo tree -p contextra-simd --edges dev` |

**Reihenfolge bei der Metrik.** U-SIMD-3 ersetzt `compute_distance` durch die Kern-Funktionen. Die Zuordnung „Metrik → Kern“ wird in `vector` ergänzt, bevor U-SIMD-4 die alte Funktion entfernt. Das verhindert einen Zwischenzustand ohne Metrik-Logik.

---

## 7. Tests

| ID | Prüfung | Ort |
|---|---|---|
| T-SIMD-01 | Jede SIMD-Variante (AVX2, AVX-512, NEON, sofern verfügbar) liefert für 10.000 zufällige Paare denselben Wert wie `reference::*` bis auf relative Toleranz `1e-5` (fp32) bzw. exakt (sq8, u32) | `tests/differential.rs` |
| T-SIMD-02 | Referenzwerte für `dot_f32`, `l2_sq_f32`, `norm_sq_f32` stimmen mit `simsimd` überein (dev-Abhängigkeit) | `tests/reference_values.rs` |
| T-SIMD-03 | `LengthMismatch` bei unterschiedlichen Längen, `EmptyInput` bei leeren Eingaben, `NonFinite` bei NaN und ∞ (jeweils in jedem Kern) | `tests/validation.rs` |
| T-SIMD-04 | Dimensionen 1, 3, 7, 8, 15, 16, 17, 384, 768, 1024 (Restbehandlung der SIMD-Breiten) | `tests/differential.rs` |
| T-SIMD-05 | `force_target` (nur Feature `test-override`) erzwingt den Kern, `active_target()` liefert ihn zurück | `tests/differential.rs` |
| T-SIMD-06 | `unsafe` nur in `kernels/avx2.rs`, `kernels/avx512.rs`, `kernels/neon.rs` (Allowlist) | `tests/unsafe_islands.rs` |
| T-SIMD-07 | `SQ8`-Kerne sind bei denselben Codes deterministisch über alle Targets (Gleichheit der `u32`-Summen) | `tests/differential.rs` |
| T-SIMD-08 | `dot_f32_sq8` mit `scale = 1`, `offset = 0` stimmt mit `dot_f32` über dem Code als fp32 überein | `tests/reference_values.rs` |

---

## 8. Abnahme

| Gate | Prüfung für `contextra-simd` |
|---|---|
| G-01 | `cargo check`, `cargo test`, `cargo test --no-default-features`, `cargo clippy -- -D warnings`, Toolchain 1.89.0 |
| G-02 | `unsafe` nur in den drei Kern-Dateien (T-SIMD-06). `#![deny(unsafe_op_in_unsafe_fn)]` gesetzt |
| G-03 | Keine Workspace-Abhängigkeit in `[dependencies]` |
| G-06 | `default = ["std"]` ist nach R3 als einzige Ausnahme zulässig. Feature `std` ist leer und wird nur für die Fehlertypen mit `std::error::Error` verwendet |
| G-07 | `grep -rn "contextra_types\|compat\|simd-avx512\|types-compat\|workspace" crates/contextra-simd` leer |
| G-13 | keine `unwrap`/`expect`/`panic!` in `src/` außerhalb `#[cfg(test)]` |
| R5 | `#![forbid(unsafe_code)]` in allen Modulen außer `kernels/avx2.rs`, `kernels/avx512.rs`, `kernels/neon.rs`. Diese Ausnahme ist über T-SIMD-06 belegt |

**Reihenfolge im Repository:** Nach dem `sys`-PLAN als vierter Commit. Die Umsetzung U-SIMD-1 bis U-SIMD-6 folgt. `vector` wird in seinem eigenen PLAN im selben Zeitraum angepasst.

---

## Nachtrag N-1: Ziel der Metrik-Definition

**Quelle:** `crates/contextra-types/PLAN.md`, Abschnitt 3.3 und Entscheidung O-TYP-3.

- Zeile 25 (Dreifach-Definition von `DistanceMetric`): Die Definition in `contextra-types` entfällt mit diesem Crate. Die Definition in `contextra-vector/src/types.rs` wandert nach `contextra-ports::vector_index` (`crates/contextra-ports/PLAN.md`, Nachtrag N-1). Die Definition in `metric.rs` dieses Crates entfällt (E-SIMD-04).
- E-SIMD-04: Ziel ist nicht mehr `vector::DistanceMetric`, sondern `contextra-ports::vector_index::DistanceMetric`. Die Distanzfunktionen `distance` und `distance_u8` stehen in `contextra-vector::metric` und rufen die Kernel dieses Crates auf.
- `contextra-simd` erhält keine Metrik-Enum. Die Kern-API bleibt auf Funktionen wie `dot_f32`, `l2_sq_f32` und Varianten beschränkt, wie in Abschnitt „Kern-API“ oben beschrieben. Die Abhängigkeit von `contextra-types` (R7) entfällt damit vollständig.
