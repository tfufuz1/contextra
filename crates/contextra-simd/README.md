# contextra-simd

SIMD-beschleunigte Distanzberechnungen und Vektor-Operationen.

Normativitätsklasse: **Kern `[K]`** (Ring 0 Unsafe-Insel, `stable`; §6.1, §6.7, §6.9, `capabilities.toml`).

## Rolle in einem Satz
`contextra-simd` kapselt hardwarenahe SIMD-Vektordistanzberechnungen (AVX2, AVX-512, ARM NEON) und skalare Referenzpfade in einer genau auditierten, sicheren API-Insel mit injizierbarem Dispatcher.

## Ring, Klasse & Reifegrad
- **Ring:** Ring 0 (Unsafe-Insel 1 von 3, `#![deny(unsafe_code)]` ausgenommen)
- **Klasse:** Kern `[K]`
- **Reifegrad:** `stable`
- **Erlaubte Abhängigkeiten (`may_use` Obergrenze):** `contextra-types` (Spec §6.9, `capabilities.toml`)
- **Features:** Minimal, standardmäßig keine Schwergewichte. Additive ISA-Nutzung über Runtime-Feature-Detection.

## Garantien & Invarianten
1. **Design by Contract & Safe Public API:** Sämtliche öffentlichen Methoden (`compute_distance`, `validate_vector`, `Dispatcher`) sind vollkommen sicher und frei von unkontrollierten Panics.
2. **Determinismus & Reduktionsordnung (INV-RING0-DETERMINISM-1):** Float-Reduktionen folgen einer festen Summationsordnung über Vektor-Lanes und deterministische Baumreduktion. Numerische Abweichungen zwischen SIMD- und Skalarpfad liegen strikt innerhalb IEEE-754 ULP-Toleranzen (< 1e-4).
3. **Tail-Handling & Validierung:** Beliebige Vektorlängen werden ohne Vorbedingung `len % lane == 0` unterstützt. Ungültige Gleitkommazahlen (NaN, ±Inf) werden via `validate_vector` Fail-Closed zurückgewiesen.
4. **Injizierbarer Dispatcher:** Ermöglicht zustandslose, thread-sichere Konfiguration (`Dispatcher::auto()`, `Dispatcher::scalar()`, `Dispatcher::new(SimdTarget)`), ohne auf globalen veränderlichen Zustand angewiesen zu sein.

## Einbettungsbeispiel

```rust
use contextra_simd::{compute_distance, validate_vector};
use contextra_simd::dispatch::{Dispatcher, SimdTarget};
use contextra_types::DistanceMetric;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let a = [1.0f32, 2.0, 3.0, 4.0];
    let b = [2.0f32, 3.0, 4.0, 5.0];

    // Vorab-Validierung
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
