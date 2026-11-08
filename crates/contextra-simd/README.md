# contextra-simd

SIMD-beschleunigte Distanzberechnungen und Vektor-Operationen.

Normativitätsklasse: **Klasse A (Autarker Baustein)** · Ring 0 Unsafe-Insel (`stable`; SPEC §5).

## Rolle in einem Satz
`contextra-simd` kapselt hardwarenahe SIMD-Vektordistanzberechnungen (AVX2, AVX-512, ARM NEON) und skalare Referenzpfade in einer genau auditierten, sicheren API-Insel mit injizierbarem Dispatcher.

## Baustein-Profil
- **Klasse:** A (Keine internen Pflicht-Dependencies; Standard- und Crate-eigene Typen).
- **Externe Dependencies:** `bytemuck`, `thiserror`. Optional: `serde`, `tracing`.
- **Transitional-Features:** `types-compat` (stellt `From`-Impls zu `contextra-types` für Workspace-Kompatibilität bereit).
- **MSRV:** 1.89.0
- **Lizenz:** MIT OR Apache-2.0

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
1. **Design by Contract & Safe Public API:** Sämtliche öffentlichen Methoden (`compute_distance`, `validate_vector`, `Dispatcher`) sind vollkommen sicher und frei von unkontrollierten Panics.
2. **Determinismus & Reduktionsordnung (INV-RING0-DETERMINISM-1):** Float-Reduktionen folgen einer festen Summationsordnung über Vektor-Lanes und deterministische Baumreduktion. Numerische Abweichungen zwischen SIMD- und Skalarpfad liegen strikt innerhalb IEEE-754 ULP-Toleranzen (< 1e-4).
3. **Tail-Handling & Validierung:** Beliebige Vektorlängen werden ohne Vorbedingung `len % lane == 0` unterstützt. Ungültige Gleitkommazahlen (NaN, ±Inf) werden via `validate_vector` Fail-Closed zurückgewiesen.
4. **Injizierbarer Dispatcher:** Ermöglicht zustandslose, thread-sichere Konfiguration (`Dispatcher::auto()`, `Dispatcher::scalar()`, `Dispatcher::new(SimdTarget)`), ohne auf globalen veränderlichen Zustand angewiesen zu sein.
5. **Skalares Orakel:** Differentialtests verifizieren jeden SIMD-Kernel kontinuierlich gegen die deterministische Skalar-Referenz.

## Minimalbeispiel

```rust
use contextra_simd::{compute_distance, validate_vector, DistanceMetric};
use contextra_simd::dispatch::{Dispatcher, SimdTarget};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let a = [1.0f32, 2.0, 3.0, 4.0];
    let b = [2.0f32, 3.0, 4.0, 5.0];

    // Vorab-Validierung gegen NaN / Infinity
    validate_vector(&a)?;
    validate_vector(&b)?;

    // Standard-Berechnung (Runtime-Dispatch auf schnellste verfügbare ISA)
    let dist = compute_distance(&a, &b, DistanceMetric::Cosine)?;
    println!("Cosine distance: {dist}");

    // Injizierbarer Dispatcher (z.B. für Regressionstests gegen Skalarpfad)
    let dispatcher = Dispatcher::new(SimdTarget::Scalar);
    let euc = dispatcher.euclidean_distance(&a, &b)?;
    println!("Scalar Euclidean: {euc}");

    Ok(())
}
```
