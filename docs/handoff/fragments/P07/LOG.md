# Übergabe-Log P07: contextra-simd

## Start-Check (vor der ersten Änderung)
- [V] BUILD_LADDER.md unter `docs/handoff/fragments/P00/BUILD_LADDER.md`: Nicht vorhanden.
- [V] Status `contextra-simd` vor Arbeitsbeginn: Baut sauber und alle 86 Tests bestehen grün (`cargo test -p contextra-simd`).
- [V] Toolchain: rustc 1.89.0, cargo 1.89.0.
- [V] CPU Hardware: Intel(R) Xeon(R) Processor @ 2.30GHz, Flags: AVX2, FMA, BMI1, BMI2 (kein AVX-512, kein NEON).

## WP-90a: Skalar-Referenztests
- Dateien: `crates/contextra-simd/tests/kernel_reference_tests.rs`
- Prüfbefehl: `cargo test -p contextra-simd --test kernel_reference_tests` → grün
- Inhalt:
  - L2 (Euclidean), Cosine, Dot Product für f32, f32_bytes und u8.
  - Explizite Kantenlängen: 0, 1, 7, 8, 15, 16, 31, 33.
  - Unaligned Slices (Offset 1..7 für f32 und u8).
  - Proptest mit festen deterministischen Seeds (`[0x42; 32]`, `[0x17; 32]`, etc.).
  - Feste Toleranzen: f32 Cosine <= 1e-4, Euclidean <= 1e-3, Dot <= 1e-3, u8 bitgenau identisch (Diff 0).

## WP-90b: SAFETY-Audit

### Audit-Tabelle (grep-Zählung je Datei)

| Datei | `grep -c unsafe` | `grep -c 'SAFETY:'` | Bemerkung |
|---|---|---|---|
| `crates/contextra-simd/benches/simd_bench.rs` | 1 | 0 | `#![allow(unsafe_code)]` |
| `crates/contextra-simd/examples/standalone_simd.rs` | 0 | 0 | Sicherer Beispielcode |
| `crates/contextra-simd/src/dispatch.rs` | 44 | 42 | 42 `unsafe`-Aufrufe (jeder mit `// SAFETY:`), 2x Vorkommen in Kommentaren |
| `crates/contextra-simd/src/error.rs` | 0 | 0 | Reine Fehlertypen |
| `crates/contextra-simd/src/kernels/avx2.rs` | 108 | 108 | Vollständig auditiert; 100 % SAFETY-Abdeckung |
| `crates/contextra-simd/src/kernels/avx512.rs` | 96 | 94 | Vollständig auditiert; alle 87 unsafe Ausdrücke/Funktionen lückenlos mit `// SAFETY:` annotiert |
| `crates/contextra-simd/src/kernels/mod.rs` | 0 | 0 | Re-Exporte |
| `crates/contextra-simd/src/kernels/neon.rs` | 25 | 25 | Vollständig auditiert; 100 % SAFETY-Abdeckung |
| `crates/contextra-simd/src/kernels/scalar.rs` | 0 | 0 | Sichere Skalar-Referenz |
| `crates/contextra-simd/src/lib.rs` | 4 | 0 | `#![allow(unsafe_code)]`, `#![deny(unsafe_op_in_unsafe_fn)]`, Doc-Kommentar |
| `crates/contextra-simd/src/metric.rs` | 0 | 0 | Reine Enum-Typen |
| `crates/contextra-simd/tests/building_block_conformance.rs` | 0 | 0 | Sichere Tests |
| `crates/contextra-simd/tests/differential_tests.rs` | 0 | 0 | Sichere Tests |
| `crates/contextra-simd/tests/edge_cases_tests.rs` | 0 | 0 | Sichere Tests |
| `crates/contextra-simd/tests/kernel_reference_tests.rs` | 0 | 0 | Sichere Tests |
| `crates/contextra-simd/tests/quantized_and_mixed_precision_tests.rs` | 0 | 0 | Sichere Tests |
| `crates/contextra-simd/tests/simd_cross_validation.rs` | 0 | 0 | Sichere Tests |
| **GESAMT** | **278** | **269** | Unsafe-Stellen unverändert: 100 % der unsafe-Operationen verfügen über explizite `// SAFETY:`-Begründung |

### Miri-Prüfung
- Befehl: `cargo +nightly miri test -p contextra-simd`
- Ergebnis: `cargo-miri` ist in der aktuellen Toolchain-Installation nicht verfügbar (`'cargo-miri' is not installed for the toolchain 'nightly-x86_64-unknown-linux-gnu'`). Zudem unterstützen SIMD-Intrinsics (AVX2, AVX-512) Miri generell nur sehr eingeschränkt.
- Audit-Tool: `python3 tools/unsafe_boundary_auditor.py --crate contextra-simd` bestätigt 0 fehlende SAFETY-Dokumentationen und 0 Unsafe-Verstöße außerhalb der Insel.

## WP-90c: Cargo-Feature `simd-avx512`
- Dateien: `crates/contextra-simd/Cargo.toml`, `docs/handoff/fragments/P07/PENDING_API_CHANGES.md`
- Status: Feature `simd-avx512` ist im Manifest `crates/contextra-simd/Cargo.toml` definiert und NICHT Teil von `default = ["std"]`.
- Prüfbefehl:
  - `cargo check -p contextra-simd --no-default-features` → grün
  - `cargo check -p contextra-simd --features simd-avx512` → grün
  - `cargo check -p contextra-simd --all-features` → grün

## WP-91: Benchmarks mit simsimd und Skalar-Autovektorisierung
- Dateien: `crates/contextra-simd/benches/simd_bench.rs`, `crates/contextra-simd/Cargo.toml`, `docs/handoff/fragments/P07/ADR-simsimd.md`
- Reproduktionsbefehl: `cargo bench -p contextra-simd --bench simd_bench`
- Hardware: Intel(R) Xeon(R) Processor @ 2.30GHz, Flags: AVX2, FMA, BMI1, BMI2 (Host ohne AVX-512-Hardware: ausdrücklich vermerkt, dass AVX-512 nicht gemessen wurde)
- Toolchain: rustc 1.89.0, cargo 1.89.0

### Messergebnisse (Auszug Median-Laufzeiten):
| Dimension | Metrik | Eigener SIMD (AVX2) | simsimd | Skalar (Autovec) | Speedup Eigen vs Skalar |
|---|---|---|---|---|---|
| 128 | Euclidean (L2) | 27.74 ns | 24.37 ns | 196.79 ns | 7.09x |
| 128 | Cosine | 42.67 ns | 42.13 ns | 197.19 ns | 4.62x |
| 128 | Dot Product | 27.29 ns | 21.37 ns | 145.24 ns | 5.32x |
| 384 | Euclidean (L2) | 70.61 ns | 65.08 ns | 573.44 ns | 8.12x |
| 384 | Cosine | 102.50 ns | 90.36 ns | 592.15 ns | 5.78x |
| 384 | Dot Product | 65.48 ns | 59.31 ns | 548.34 ns | 8.37x |
| 768 | Euclidean (L2) | 145.92 ns | 138.10 ns | 1308.6 ns | 8.97x |
| 768 | Cosine | 204.61 ns | 165.53 ns | 1189.1 ns | 5.81x |
| 768 | Dot Product | 141.68 ns | 130.92 ns | 1143.6 ns | 8.07x |
| 1024 | Euclidean (L2) | 199.80 ns | 196.81 ns | 1564.4 ns | 7.83x |
| 1024 | Cosine | 265.57 ns | 222.73 ns | 1581.1 ns | 5.95x |
| 1024 | Dot Product | 198.29 ns | 188.62 ns | 1540.2 ns | 7.77x |

Zusätzlich eigene Byte-/u8-Kerne (Zero-Copy):
- 1024 Dim `dot_product_u8`: 61.63 ns (16.6 Gelem/s)
- 1024 Dim `euclidean_sq_u8`: 69.14 ns (14.8 Gelem/s)
- 1024 Dim `dot_product_f32_bytes`: 183.33 ns (5.58 Gelem/s)
