# contextra-testkit — Implementierungsspezifikation (Soll-Zustand)

**Ebene:** E3 (Endpunkte / Testwerkzeuge)  
**Kern-Closure:** `ports`, `types`, `ids`, `env`, `erasure`  
**Status:** Normativ (Soll-Zustand für Entwickler)  
**Version:** 2.0  
**Geltungsbereich:** `crates/contextra-testkit`  
**Referenzdokumente:** `docs/SPEC.md` §2, §12 (Testkatalog); `ANALYSIS-E0-CONTRADICTIONS.md`; `ALTLASTEN-AUDIT.md` (AP-21)  

---

## 1. Zweck & Architekturgarantien

`contextra-testkit` stellt die gemeinsame Testinfrastruktur, In-Memory-Testdoppel, Fehlersimulatoren und Konformitätssuiten für das Contextra-System bereit:
- **In-Memory Port-Testdoppel:** Schnelle RAM-Implementierungen aller Port-Traits (`InMemoryStore`, `InMemoryVector`, `InMemoryText`, `InMemoryGraph`).
- **Fehlersimulatoren (Fault Injection):** Gezielte Injektion von I/O-Fehlern, Disk-Voll-Zuständen (`DiskReserveLow`) und Handle-Poisoning zur Verifikation von Crash-Konsistenz.
- **Canary-Orakel & Positivkontrolle (T-DEL-7):** Erzeugung kryptographisch eindeutiger Canaries zur Kalibrierung des `Scanner<Armed>` im DSGVO-Testkatalog (T-DEL-1 bis T-DEL-13).
- **Deterministische Korpora:** Reproduzierbare synthetische Dokumente und Vektoren für Regressions- und Differenzialtests.

### Beseitigung von Altlasten & Richtlinien (R2, R3, AP-21)
1. **R2-Abhängigkeitsregel:** `contextra-testkit` ist eine reine E3-Komponente. Kern-Crates der Ebenen E0 und E1 dürfen `contextra-testkit` **weder in dependencies noch in dev-dependencies** einbinden.
2. **Leere Default-Features (R3):** `default = []` (Entfernung von `default = ["testkit", "erasure-conformance"]`).
3. **Altlasten-Bereinigung (`AP-21`):** In `src/lib.rs` (Z. 27–28) und `src/conformance/routing_ports.rs` werden alle Referenzen auf `contextra-agent`, `contextra-cognition` und `contextra-router` gelöscht.

### Architekturgarantien (R1–R6, INV-3, INV-4)
1. **`#![forbid(unsafe_code)]`:** 100 % speichersicher.
2. **Zero-Panic-Policy:** Test-Frameworks und Mocks schlagen kontrolliert mit Fehlern fehl; keine unkontrollierten Panics in Helfern.

---

## 2. Architektur & Test-Module

```
┌─────────────────────────────────────────────────────────────┐
│                      contextra-testkit                      │
│                                                             │
│   ┌─────────────────────────────────────────────────────┐   │
│   │                 FaultInjectionEngine                │   │
│   │   ├── FaultVfs (simuliert Dateisystem-Fehler)       │   │
│   │   └── PoisonSimulator (triggert INV-FSYNC-1)        │   │
│   └─────────────────────────────────────────────────────┘   │
│                                                             │
│   ┌─────────────────────────────────────────────────────┐   │
│   │                 InMemoryTestDoubles                 │   │
│   │   ├── InMemoryStorageEngine                         │   │
│   │   ├── InMemoryVectorIndex                           │   │
│   │   ├── InMemoryTextIndex                             │   │
│   │   └── InMemoryGraph                                 │   │
│   └─────────────────────────────────────────────────────┘   │
│                                                             │
│   ┌─────────────────────────────────────────────────────┐   │
│   │                ErasureTestingHarness                │   │
│   │   ├── CanaryGenerator (T-DEL-1 bis T-DEL-13)        │   │
│   │   └── PositiveControlArming (Scanner<Armed>)        │   │
│   └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

---

## 3. Test-Harness für die Löschkette (T-DEL-1 bis T-DEL-13)

```rust
use std::path::Path;

pub struct CanaryPattern {
    pub bytes: [u8; 32],
}

impl CanaryPattern {
    pub fn generate() -> Self {
        use rand::RngCore;
        let mut b = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut b);
        Self { bytes: b }
    }
}

pub struct CanaryScanner;

impl CanaryScanner {
    /// Scannt ein Dateisystem-Verzeichnis rekursiv nach dem Canary-Muster.
    /// Liefert die Anzahl gefundener Vorkommen im Klartext.
    pub fn scan_dir(dir: &Path, canary: &CanaryPattern) -> Result<u64, std::io::Error> {
        let mut count = 0;
        for entry in walkdir::WalkDir::new(dir) {
            let entry = entry?;
            if entry.file_type().is_file() {
                let data = std::fs::read(entry.path())?;
                if data.windows(32).any(|w| w == canary.bytes) {
                    count += 1;
                }
            }
        }
        Ok(count)
    }
}
```

---

## 4. `Cargo.toml` Spezifikation

```toml
[package]
name = "contextra-testkit"
version.workspace = true
edition.workspace = true
license.workspace = true
rust-version.workspace = true
description = "Deterministic test utilities, in-memory port doubles, and fault injection for Contextra"

[lints]
workspace = true

[package.metadata.contextra]
ring = "tooling"
level = 3

[dependencies]
contextra-ids = { workspace = true }
contextra-env = { workspace = true }
contextra-types = { workspace = true }
contextra-ports = { workspace = true }
contextra-erasure = { workspace = true }
bytes = { workspace = true }
parking_lot = { workspace = true }
tracing = { workspace = true }
walkdir = "2.4"
rand = { workspace = true }

[dev-dependencies]
tokio = { workspace = true, features = ["macros", "rt"] }
tempfile = { workspace = true }

[features]
default = []
testkit = []
erasure-conformance = []
```

---

## 5. Implementierungs-Arbeitspakete für Entwickler

1. **`Cargo.toml` Bereinigung:**
   - Default-Features leeren (`default = []`, R3).
2. **Altlasten-Bereinigung (`AP-21`):**
   - In `src/lib.rs` (Z. 27–28) Referenzen auf `contextra-agent` und `contextra-cognition` entfernen.
   - In `src/conformance/routing_ports.rs` Bezüge zu `contextra-router` bereinigen.
3. **Verifikation:**
   - `cargo test -p contextra-testkit`
