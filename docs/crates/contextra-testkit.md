# contextra-testkit — Crate-Spezifikation (Test-Infrastruktur & Simulatoren)

**Ebene:** E3 (Endpunkte / Testwerkzeuge)  
**Status:** Normativ  
**Version:** 1.0  
**Geltungsbereich:** `crates/contextra-testkit`  
**Referenz:** `docs/SPEC.md` §2, §12

---

## 1. Zweck & Systemrolle

`contextra-testkit` stellt die gemeinsame Testinfrastruktur, Mock-Objekte, Invarianten-Prüfer und Fehlersimulatoren für den gesamten Contextra-Workspace bereit:
- **Fehlersimulatoren (Fault Injection):** Gezielte Simulation von I/O-Fehlern, Dateisystem-Voll-Zuständen (`DiskReserve`), Systemabstürzen mitten im Two-Phase-Commit (2PC) und asynchronen Task-Cancellations (Cancel-Safety).
- **Löschketten-Orakel & Positivkontrollen:** Bereitstellung von Canary-Generatoren und deterministischen Scannern zur Validierung der DSGVO-Löschkette (T-DEL-1 bis T-DEL-13).
- **Synthetische Korpus-Generatoren:** Erzeugung von reproduzierbaren Dokumenten, Vektoren und Wissensgraphen mit annotierter Ground-Truth für deterministische Differenztests.
- **Loom- & Concurrency-Harness:** Unterstützung für Concurrency- und Race-Condition-Tests.

---

## 2. Architektur & Test-Module

```
┌─────────────────────────────────────────────────────────────┐
│                      contextra-testkit                      │
│                                                             │
│   ┌─────────────────────────────────────────────────────┐   │
│   │                 FaultInjectionEngine                │   │
│   │   ├── CorruptingVfs (simuliert Dateikorruption)     │   │
│   │   ├── FailingFsync (simuliert Stromausfall)         │   │
│   │   └── Interrupted2PC (bricht nach Phase 1 ab)       │   │
│   └─────────────────────────────────────────────────────┘   │
│                                                             │
│   ┌─────────────────────────────────────────────────────┐   │
│   │                ErasureTestingHarness                │   │
│   │   ├── CanaryOracle (injiziert Byte-Muster)          │   │
│   │   ├── PositivControlCalibrator                      │   │
│   │   └── ResidueScanner (prüft Festplatte auf 0 Reste) │   │
│   └─────────────────────────────────────────────────────┘   │
│                                                             │
│   ┌─────────────────────────────────────────────────────┐   │
│   │                 SyntheticCorpusKit                  │   │
│   │   ├── DeterministicVectorGenerator (Fixed Seed)     │   │
│   │   ├── GermanCorpusSynthesizer (Morphology-Test)     │   │
│   │   └── GroundTruthGraphBuilder                       │   │
│   └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

---

## 3. Test-Utilities für die DSGVO-Löschkette

`contextra-testkit` liefert die standardisierten Test fixtures zur Verifikation der Klassen K, P, D und E:

### 3.1 `CanaryOracle`
Injiziert ein kryptographisch eindeutiges Byte-Muster über die reguläre Schreib-API:
```rust
pub struct CanaryOracle {
    pattern: [u8; 32],
}

impl CanaryOracle {
    pub fn new_random() -> Self;
    pub fn as_bytes(&self) -> &[u8];
    /// Prüft alle Dateien eines Verzeichnisses rekursiv auf Vorkommen des Musters.
    pub fn scan_directory(&self, dir: &Path) -> Result<ScanReport, std::io::Error>;
}
```

### 3.2 Positivkontrolle (T-DEL-7)
Ein Negativtest (keine Funde im Speicher) ist nur gültig, wenn der Scanner nachweislich funktioniert:
1. Der `PositivControlCalibrator` erzeugt eine synthetische Testdatei mit eingebettetem Canary.
2. Der Scanner scannt die Datei und **muss** das Muster finden.
3. Erst wenn die Positivkontrolle bestanden ist, wird der `Scanner<Armed>`-Typ freigeschaltet, der als Beweis in `ErasureLayer::witness` übergeben werden darf.

---

## 4. Richtlinien für Abhängigkeiten (R1 / R2)

1. **Reine Test-Bibliothek:** `contextra-testkit` darf NIEMALS in der `[dependencies]`-Sektion eines Kern-Crates (E0 bis E2) stehen.
2. **Dev-Dependencies:** Ausschließlich unter `[dev-dependencies]` für Integrationstests in `tests/` erlaubt.
3. **`#![forbid(unsafe_code)]`:** Alle Simulatoren und Testwerkzeuge sind speichersicher implementiert.
