# contextra-license

Autarke Lizenz- und Aktivierungsprüfung für Contextra Feature-Ringe (§14.6, §15, SPEC v4.0).

Normativitätsklasse: **Baustein Klasse A** (Ring 4, SPEC §2, §5; autark ohne interne Abhängigkeiten).

## Rolle in einem Satz
`contextra-license` prüft kryptografisch signierte Ed25519-Lizenzen zur Autorisierung von Feature-Ringen mit garantierter Non-Destructive Degradation.

---

## 1. Baustein-Eigenschaften (Klasse A, B1–B12)
* **B1/B2 (Autarkie):** Keine internen Pflichtabhängigkeiten in `[dependencies]` oder `[dev-dependencies]`.
* **B3/B4 (API & Fehler):** Alle Schnittstellen basieren auf Standardtypen und Crate-eigenen Typen; ein kanonischer, strukturierter Fehlertyp `LicenseError` (`thiserror`, `#[non_exhaustive]`).
* **B5 (Isolierter Export):** Baut und testet autark aus einem leeren Verzeichnis via `python3 tools/export_crate.py contextra-license`.
* **B6 (Minimale Abhängigkeiten):** Keine schweren Frameworks; reine Krypto (`ed25519-dalek`, `blake3`) und Serialisierung (`serde`, `bincode`).
* **B7 (Keine Fremdfachlogik):** Keine Vermischung mit Speicher-, Vektor- oder Löschlogik.
* **B8 (Vollständig dokumentiert & gebenchmarkt):** `README.md`, `SPEC.md`, Unit-/Integrationstests und Criterion-Benchmarks in `benches/license_benchmark.rs`.
* **B9/B10 (Zero-Panic & Unsafe):** `#![forbid(unsafe_code)]`, striktes Zero-Panic in allen Produktionspfaden.
* **B11 (Determinismus):** Zeitabfragen erfolgen über den Crate-lokalen `Clock`-Trait (kein ungekapseltes `SystemTime::now()` im Kern).
* **B12 (Lizenzierung):** Eigene `LICENSE-MIT` und `LICENSE-APACHE` im Crate enthalten.

---

## 2. Copy-Paste Anleitung & Standalone-Export

Dieses Crate kann mit dem Workspace-Exportwerkzeug isoliert exportiert werden:
```bash
python3 tools/export_crate.py contextra-license --out /tmp/contextra-license-standalone
```

---

## 3. Minimalbeispiel

```rust
use contextra_license::{FeatureRing, LicenseGate, OpenFastGate, SignedLicenseGate};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Offenes Default-Gate (OpenFastGate)
    let fast_gate = OpenFastGate;
    assert!(fast_gate.check_ring(FeatureRing::Fast).is_ok());
    assert!(fast_gate.check_ring(FeatureRing::Sovereign).is_err());

    // 2. Unaktiviertes SignedLicenseGate (Fail-Closed)
    let signed_gate = SignedLicenseGate::no_activation();
    assert!(signed_gate.check_ring(FeatureRing::Fast).is_ok()); // INV-LICENSE-2
    assert!(signed_gate.check_ring(FeatureRing::Sovereign).is_err());

    Ok(())
}
```

---

## 4. Nicht-Ziele
* Kein zentraler Lizenzserver oder Online-Telemetrie (Offline-Only).
* Keine Hard-Locks oder Zerstörung von Nutzerdaten bei Lizenzablauf (Non-Destructive Degradation).
* Keine Kopplung an Monorepo-Kerne oder fremde Schnittstellen-Hubs.
