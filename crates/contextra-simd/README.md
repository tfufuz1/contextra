# contextra-simd

SIMD-beschleunigte Distanzberechnungen und Vektor-Operationen.

Normativitätsklasse: **Klasse A (Autarker Baustein)** · Ring 0 Unsafe-Insel (`beta`; SPEC §5, Reifegrad R3).

## Rolle in einem Satz
`contextra-simd` kapselt hardwarenahe SIMD-Vektordistanzberechnungen (AVX2, AVX-512, ARM NEON) und skalare Referenzpfade in einer genau auditierten, sicheren API-Insel mit injizierbarem Dispatcher.

## Baustein-Profil
- **Klasse:** A (Keine internen Pflicht-Dependencies; Standard- und Crate-eigene Typen).
- **Tatsächliche Abhängigkeiten (`cargo tree -p contextra-simd -e normal --no-default-features`):** `thiserror`.
- **Optionale Dependencies:** `serde`, `tracing`, `contextra-types` (nur unter `workspace` / `types-compat` in `src/compat/`).
- **MSRV:** 1.89.0 (`rust-version = "1.89.0"`)
- **Lizenz:** MIT OR Apache-2.0
- **Tier:** Beta (autarker Ring 0 Baustein)

## Dispatch-Tabelle

| Architektur / ISA | Kern-Modul | Vektorbreite | Status | Laufzeit-Prüfung |
|---|---|---|---|---|
| **Scalar (Portable)** | `kernels/scalar.rs` | 32-Bit (f32) | Immer aktiv / Orakel | Keine (universell) |
| **x86_64 AVX2 / FMA** | `kernels/avx2.rs` | 256-Bit (8 Lanes) | Dynamischer Dispatch | `is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma")` |
| **x86_64 AVX-512** | `kernels/avx512.rs` | 512-Bit (16 Lanes) | Dynamischer Dispatch | `is_x86_feature_detected!("avx512f")` (+ `avx512bw`, `avx512vnni`) |
| **AArch64 ARM NEON** | `kernels/neon.rs` | 128-Bit (4 Lanes) | Dynamischer Dispatch | `std::arch::is_aarch64_feature_detected!("neon")` |

## Feature-Tabelle

| Feature | Zweck | Default | Interne Abhängigkeiten |
|---|---|---|---|
| `std` | Standardbibliothek-Integration | Ja | Keine |
| `serde` | Serialisierung für `DistanceMetric` | Nein | Keine (`serde`) |
| `simd-avx512` | Explizites Feature-Flag für AVX-512 Optimierungen | Nein | Keine |
| `workspace` | Kompatibilitätsbrücke zu `contextra-types` in `src/compat/` | Nein | `contextra-types` (optional) |
| `types-compat` | Übergangsalias für `workspace` | Nein | `contextra-types` (über `workspace`) |

## Quickstart (Kompilierbares Minimalbeispiel)

```rust
use contextra_simd::{compute_distance, validate_vector, DistanceMetric};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let a = [1.0f32, 2.0, 3.0, 4.0];
    let b = [5.0f32, 6.0, 7.0, 8.0];

    // Vorab-Validierung gegen NaN / Infinity
    validate_vector(&a)?;
    validate_vector(&b)?;

    // Distanzberechnung mit automatischer Hardwareerkennung
    let cos = compute_distance(&a, &b, DistanceMetric::Cosine)?;
    let euc = compute_distance(&a, &b, DistanceMetric::Euclidean)?;
    let dot = compute_distance(&a, &b, DistanceMetric::DotProduct)?;

    println!("Cosine: {cos:.6}, Euclidean: {euc:.6}, Dot: {dot:.6}");
    Ok(())
}
```

Ausführen via:
```bash
cargo run -p contextra-simd --example standalone --no-default-features
```

## Standalone-Entnahme (Copy-Paste / Export)
Um diesen Baustein völlig unabhängig in ein anderes Projekt zu übernehmen:
```bash
python3 tools/export_crate.py contextra-simd --out /tmp/my-simd-crate
```

## Nicht-Ziele
- Keine Vektor-Indexierungsstrukturen (gehört in Vektorindizes wie HNSW).
- Keine Modell-Inferenz oder Generierung.
- Keine Persistenz, WAL oder Transaktionen.
- Keine Mandanten- oder DSGVO-/Löschlogik.

## Garantien & Invarianten
1. **Design by Contract & Safe Public API:** Sämtliche öffentlichen Methoden (`compute_distance`, `validate_vector`, `Dispatcher`, `MetricDistanceFn`) sind vollkommen sicher und frei von unkontrollierten Panics.
2. **INV-SIMD-DETERMINISM-1 (Determinismus):** Float-Reduktionen folgen einer festen Summationsordnung über Vektor-Lanes und deterministische Baumreduktion. Numerische Abweichungen zwischen SIMD- und Skalarpfad liegen strikt innerhalb der dokumentierten Toleranz (< 1e-5 relativ). Test: `test_inv_simd_determinism_1`.
3. **INV-SIMD-FINITE-1 (Endlichkeit):** Ungültige Gleitkommazahlen (NaN, ±Inf) werden via `validate_vector` Fail-Closed zurückgewiesen. Es wird nie mit NaN/Inf gerechnet. Test: `test_inv_simd_finite_1`.
4. **INV-SIMD-ALIGN-1 (Byte-Puffer-Alignment):** Byte-Puffer müssen eine ganzzahlig durch 4 teilbare Länge besitzen; es werden keine Annahmen über Speicherausrichtung getroffen (Unaligned Loads `loadu`). Test: `test_inv_simd_align_1`.
5. **Injizierbarer Dispatcher:** Ermöglicht zustandslose, thread-sichere Konfiguration (`Dispatcher::auto()`, `Dispatcher::scalar()`, `Dispatcher::new(SimdTarget)`), ohne auf globalen veränderlichen Zustand angewiesen zu sein.
6. **Skalares Orakel:** Differentialtests verifizieren jeden SIMD-Kernel kontinuierlich gegen die deterministische Skalar-Referenz.
