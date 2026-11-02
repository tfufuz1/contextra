# `contextra-simd` – Implementierungsspezifikation

|Feld|Wert|
|---|---|
|Zweck-Satz (D-97)|Vektor-Distanzfunktionen (L2, Dot, Cosine) mit SIMD-Kernen und Skalar-Referenz.|
|Ring|1|
|Erlaubte Workspace-Abhängigkeiten|`types` (SPEC 5.3); Verbraucher: `vector` (nur dieses Crate)|
|Stufe / Gates|S2; G-G (Vektorindex), G-C (Panic-Freiheit)|
|Arbeitspakete|WP-8.1 (Vektor-Arena und Kernel-Verdrahtung), WP-0.1 (Ratchet)|
|SPEC-Anker|8.4, D-28, D-29, D-86, AR-8, T-SIMD-1, F-18, 15.1 (Miri, Sanitizer)|

## 1. Ist-Zustand (HEAD `0ede89a26`) [V]

|Punkt|Befund|
|---|---|
|Umfang|2.797 Zeilen `src`: `dispatch.rs` (972), `kernels/avx2.rs` (653), `kernels/avx512.rs` (413), `kernels/scalar.rs` (368), `kernels/neon.rs` (171), `lib.rs` (185), `kernels/mod.rs` (35)|
|u8-VNNI|Treffer `vnni`/`dpbusd` in `src`: **0**. Der Rückbau (Commit auf Basis PR #5520) ist vollzogen; D-28 ist im Quelltext erfüllt. Offen ist die Absicherung per Test und Grep-Gate.|
|`unsafe`|231 Treffer in `src` (SPEC F-18 nannte 278 am älteren HEAD). Aufrufe der Kernel tragen `// SAFETY:`-Kommentare (Stichprobe `dispatch.rs`)|
|Öffentliche API|`validate_vector`, `compute_distance`, `compute_distance_trusted`, `compute_distance_f32_bytes_trusted` (`lib.rs`); Pfadüberschreibung `SimdPathOverride` (`Avx512`, `Neon`, …) in `dispatch.rs`|
|Dispatch|x86_64: AVX-512 (`avx512f`) → AVX2+FMA → Skalar; aarch64: NEON → Skalar; Laufzeiterkennung per `is_x86_feature_detected!`/`is_aarch64_feature_detected!`|
|Manifest|Abhängigkeiten `types`, `thiserror`, `bytemuck`, `tracing`; **Metadatum `ring = "0"` falsch** (Soll 1, F-35 für die `//!`-Köpfe; hier das Manifest-Metadatum)|
|Tests|5 Testdateien: `differential_tests.rs`, `edge_cases_tests.rs`, `quantized_and_mixed_precision_tests.rs`, `simd_cross_validation.rs`, `t_simd_1.rs`; 1 Benchmark `simd_bench.rs`; 13/13 Lib-Tests bestanden **[M]** (3.0.0)|
|Nicht vorhanden|Tests tragen kein `#[ctx_test]` (Attribut fehlt im ganzen Repo, L-6); kein Miri-Lauf; kein Sanitizer-Lauf|

## 2. Verantwortung und Abgrenzung

`simd` berechnet Distanzen zwischen **f32-Vektoren** (L2, Dot, Cosine; `DistanceMetric` aus `types`). Es ist neben `sys` die einzige `unsafe`-Insel (D-29, AR-8): Jeder `unsafe`-Block liegt hier, damit `vector` `#![forbid(unsafe_code)]` tragen kann. `simd` kennt weder Vektorindex noch Arena, weder Schlüssel noch Löschung.

Nicht in `simd`: Quantisierung (SQ8, PQ, binär; entfällt in v1, D-84/D-86), Speicherverwaltung der Arena, Sperren von Speicher (`mlock`/`madvise`; Port `ArenaGuard`, D-88), Threading, Ergebnissortierung.

**Grenze der Löschbarkeit:** `simd` hält **keinen** Zustand und keine Kopien von Vektoren (keine Puffer, keine Caches, keine statischen Tabellen mit Nutzerdaten). Rechenregister und Stack sind die einzige Remanenz; sie fallen unter `ProcessMemory` (Anhang D), nicht unter die Arena-Zeroisierung (D-77, D-86).

## 3. Zielstruktur (Module) [E]

|Modul|Inhalt|
|---|---|
|`lib.rs`|Crate-Kopf (Q-01), Re-Exporte der vier Eintrittsfunktionen|
|`dispatch`|Laufzeit-Auswahl (`SimdPath`), Cache der Auswahl (`OnceLock`), `SimdPathOverride` nur unter `cfg(test)` bzw. als `pub(crate)`|
|`kernels::scalar`|Referenzimplementierung (kein `unsafe`), Quelle der Wahrheit für T-SIMD-1|
|`kernels::avx2`, `kernels::avx512`, `kernels::neon`|f32-Kernel je Metrik; `#[target_feature]`-Funktionen|
|`validate`|`validate_vector` (NaN/Inf, Länge, Dimension ≤ 4096, D-87)|

`dispatch.rs` (972 Zeilen) enthält Tests und Wiederholungen je Metrik und Pfad; Ziel ist eine Tabelle `(Metrik, Pfad) → fn`, nicht je Kombination ein Zweig [E]. Dateien `kernels/mod.rs` bleiben als reine Re-Export-Datei (D-90: eine Modulform).

## 4. Schnittstellen [E]

```rust
pub fn validate_vector(v: &[f32]) -> Result<(), SimdError>;                     // Länge 1..=4096, alle Werte endlich
pub fn distance(metric: DistanceMetric, a: &[f32], b: &[f32]) -> Result<f32, SimdError>; // prüft Längen und Endlichkeit
pub fn distance_prevalidated(metric: DistanceMetric, a: &[f32], b: &[f32]) -> Result<f32, SimdError>; // nur Länge geprüft; Werte vorab per validate_vector (SI-05)
pub fn active_path() -> SimdPath;                                               // für health()/Bericht, rein lesend

#[non_exhaustive] pub enum SimdPath { Avx512, Avx2Fma, Neon, Scalar }
#[non_exhaustive] pub enum SimdError { LengthMismatch, DimensionTooLarge, NonFinite }   // feste Texte (HR-4)
```

Die Ist-Namen `compute_distance*` bleiben bis zur Umstellung von `vector` als `#[deprecated]`-freie Aliase **nicht** erhalten: Es gibt keinen Parallelpfad (P3); `vector` wird im selben WP angepasst. `compute_distance_f32_bytes_trusted` (Bytes-Sicht auf `bytemuck`) entfällt, wenn die Arena f32-Slices liefert (D-86); sonst nur mit benanntem Kernkonsumenten (P8).

## 5. Implementierungsvorgaben

|ID|Vorgabe|Quelle|Test|
|---|---|---|---|
|SI-01|Kein u8-VNNI-, kein `_mm512_dpbusd_epi32`-Pfad, keine u8/i8-Quantisierungskernel. Grep `vnni|dpbusd|dpbusds|u8` auf Kernel-Signaturen = 0.|D-28, D-84|T-SIMD-1, Grep|
|SI-02|Jeder Kernel (AVX2, AVX-512, NEON) hat einen Cross-Check gegen `scalar` über zufällige und Randvektoren; zulässige relative Abweichung ε ≤ 1e-4 (Dot, L2), für Cosine absolute Abweichung ≤ 1e-5 bei normierten Eingaben. Die Grenzen stehen als Konstanten im Test, nicht im Kernel.|D-28, 8.4|T-SIMD-1|
|SI-03|Jede `unsafe`-Operation hat einen `// SAFETY:`-Kommentar mit der geprüften Vorbedingung (Länge, Ausrichtung, CPU-Feature). `#![deny(unsafe_op_in_unsafe_fn)]`. Keine `unsafe fn` in der öffentlichen API (D-29, D-94): Die Eintrittsfunktionen sind sicher und prüfen ihre Vorbedingungen selbst.|D-29, D-94|T-UNSAFE-1|
|SI-04|Der Pfad wird **einmal** beim ersten Aufruf bestimmt und gecacht; der Wert ist über `active_path()` abrufbar. Kein Pfadwechsel zur Laufzeit außer in Tests (`SimdPathOverride` nur `cfg(test)`).|8.4, D-56|T-SIMD-1|
|SI-05|Beide Eintrittsfunktionen sind sicher (keine offene Vorbedingung, D-29): `distance` prüft Länge **und** Endlichkeit (O(n)); `distance_prevalidated` prüft nur die Längengleichheit (O(1)) und überspringt den Endlichkeitsscan, den der Aufrufer (`vector`) beim Einfügen bzw. je Anfrage genau einmal über `validate_vector` erbracht hat. Beide liefern `Err` statt zu lesen oder zu paniken; kein `debug_assert!`/`panic!` (Q-03).|P6, Q-03, D-29|T-SIMD-1, Clippy|
|SI-06|NaN und ±Inf werden durch `validate_vector` abgewiesen (`NonFinite`); die Kernel selbst behandeln sie nicht.|D-87|Property|
|SI-07|Keine heap-Allokation im Distanzpfad, keine `static mut`, kein globaler veränderlicher Zustand außer dem Pfad-Cache.|8.4|Review, Miri (Referenz)|
|SI-08|Der `ring`-Eintrag im Manifest ist `1`. `//!`-Kopf nennt Ring 1 und die Abhängigkeit `types`.|5.1, D-91, F-35|T-ARCH-5|
|SI-09|Ein Kernel für eine Instruktionsfamilie ist nur im Baum, wenn ihn ein Benchmark (BM) oder Test auf der Referenzplattform nachweislich nutzt (P8). AVX-512 bleibt, sofern `simd_bench` einen Gewinn gegenüber AVX2 ausweist; sonst Entfernung.|P8, P9|BM (Mikrobenchmark)|
|SI-10|Keine Panics, kein `unwrap`/`expect`, keine Indexierung per `[]` außerhalb von Kernel-Schleifen, die über `chunks_exact`/`get` begrenzt sind. Ratchet sinkend, Ziel 0.|P6, Q-03|Clippy|

## 6. Rückbau und Migration

|Position|Maßnahme|WP|Nachweis|
|---|---|---|---|
|Manifest `ring = "0"`|auf `1` ändern; Arch-Test liest das Metadatum (Datei 18)|WP-1.5|T-ARCH-5|
|`compute_distance`, `compute_distance_trusted`, `compute_distance_f32_bytes_trusted`|auf die Namen in Abschnitt 4 (`distance`, `distance_prevalidated`) vereinheitlichen; `vector` im selben Commit anpassen|WP-8.1|`cargo check --locked`|
|`SimdPathOverride` im öffentlichen API|auf `cfg(test)`/`pub(crate)` begrenzen|WP-8.1|Grep, T-API-1|
|`quantized_and_mixed_precision_tests.rs`|entfällt, soweit sie u8/i8-Pfade prüfen (D-28, D-86)|WP-8.1|Grep|
|`dispatch.rs` (972 Z.) mit Tests im selben Modul|Tests nach `tests/`, Dispatch als Tabelle|WP-8.1|`tokei` (Budget 5.8)|
|`bytemuck`-Abhängigkeit|nur behalten, wenn die Arena Bytes liefert (D-86); sonst entfernen|WP-8.1|`cargo tree`|

## 7. Arbeitspakete

1. WP-0.1: Test-IDs: `t_simd_1.rs` und `simd_cross_validation.rs` erhalten `#[ctx_test(id = "T-SIMD-1")]` (sobald das Attribut existiert, L-6).
2. WP-8.1 (rot): Cross-Check-Matrix `(Metrik × Pfad × Dimension ∈ {1, 3, 7, 8, 15, 16, 17, 128, 768, 4096})` und Randfälle (leer, Länge 4097, NaN); schlägt rot, wo die API von Abschnitt 4 abweicht.
3. Grün: API vereinheitlichen, `SimdError`, `validate`.
4. Miri läuft auf `scalar` und auf `validate` (SPEC 15.1); die Intrinsics-Kernel sind unter Miri nicht ausführbar und werden mit `cargo-careful` und Sanitizer-Lauf (`asan`) geprüft.
5. Mikrobenchmark `simd_bench` liefert Basislinie je Pfad (P9); AVX-512-Entscheidung nach SI-09.

## 8. Tests und Benchmarks

|ID|Prüft|
|---|---|
|T-SIMD-1|Jeder Kernel gleicht gegen die Skalar-Referenz (SI-02), alle Pfade der Plattform, alle Dimensionen der Matrix|
|T-UNSAFE-1|`unsafe` nur in `sys`/`simd`; keine `unsafe fn` in der öffentlichen API (SI-03)|
|T-ARCH-5|Ring 1 und Abhängigkeiten im Kopf und im Manifest|
|Property|`proptest`: 10⁵ Zufallsvektoren je Metrik; Symmetrie `d(a,b) = d(b,a)` (L2, Dot), `d(a,a) ≈ 0` (L2, Cosine)|
|Miri / `cargo-careful` / `asan`|laut Abschnitt 7 Nr. 4|
|BM (Mikro, `criterion`)|Distanzkern je Pfad und Dimension; Eingang in BM-VEC (Datei 10)|

## 9. Abnahme

Definition of Done (Index Abschnitt 8) und: Grep `vnni|dpbusd` = 0; jede `unsafe`-Stelle mit `SAFETY`; `cargo test -p contextra-simd` auf x86_64 und aarch64 (CI-Matrix, cross-validation-Workflow vorhanden: `simd-cross-validation.yml`); Ratchet-Eintrag `simd` für `unwrap`/`expect`/`panic` = 0.

## 10. Offene Punkte

- **SI-05:** Die SPEC fordert nur Kernel plus Referenz (8.4). Ob `distance_prevalidated` neben `distance` nötig ist, entscheidet WP-8.1 anhand von BM-VEC (Kosten des O(n)-Endlichkeitsscans je Kandidat); ohne messbaren Gewinn entfällt die Variante (P8, P9).
- Zulässige Abweichung für Cosine (SI-02) ist ein Entwurf **[E]**; die SPEC nennt nur „Cross-Check“ (T-SIMD-1). Die Grenze `ε ≤ 1e-4` stammt aus der Altdatei `IMPLEMENTATION_SPEC_PER_CRATE.md` und ist dort ebenfalls nicht aus der SPEC 4.3.0 abgeleitet.
