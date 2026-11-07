# AGENTS.md — `contextra-ports`

## 1. Rolle & Zweck (Baukasten-Prinzip)
`contextra-ports` definiert die formale Vertragsschicht (Design by Contract) des gesamten Contextra-Ökosystems (Ring 0).
Es stellt abstrakte Port-Schnittstellen und Trait-Definitionen bereit, über die Speicherkerne, Vektorindizes, Volltextsuchmaschinen, Kryptographie und externe Adapter lose gekoppelt miteinander interagieren.
Als autarker Baustein ermöglicht es die vollständige Austauschbarkeit aller Komponenten und verhindert monolithische Querverdrahtungen.

## 2. Ring, Klasse, Reifegrad & `may_use`
- **Ring:** Ring 0 (Port-Fundament)
- **Klasse:** Kern `[K]`
- **Reifegrad:** `stable`
- **`may_use` (Obergrenze):** `contextra-types` (Spec §6.9, `capabilities.toml`).

## 3. Garantien & Architektur-Invarianten
- **REQ-PRT-001:** Reine Trait-Schnittstellen (Anhang A.1, A.2) ohne konkrete Produktiv-Implementierungen oder Speicherzustände.
- **REQ-PRT-002:** Injektionsfähige Zeit- (`Clock`), Zufalls- (`Rng`, `EntropyRng`) und I/O-Abstraktionen (`Vfs`).
- **Dispatch-Disziplin (D-03, D-118):** Statischer Dispatch (AFIT / `-> impl Future + Send`) auf dem Heißpfad (`StorageEngine`, `VectorIndex`, `TextIndex`); dynamischer Dispatch (`BoxFuture`) auf dem Kaltpfad (`LedgerSink`, `Clock`, `ReadGate`).
- **Lösch-Entkopplung (Purger / Scanner):** Speicherkerne bieten ausschließlich minimale, rechts- und DSGVO-neutrale Bereinigungs-Ports (`Purger`, `LayerScanner`) für `contextra-erasure` an.
- **Zero-Panic-Doktrin:** Trait-Definitionen erzwingen Fehlerbehandlung über typisierte Rückgaben (`Result<T, ContextraError>`).

## 4. Verbotene Muster
- Keine konkreten Port-Implementierungen im Crate (außer Test-Mocks).
- Kein `unsafe` (`#![forbid(unsafe_code)]` strikt erzwungen).
- Keine direkten Systemzeit- oder OS-Zufallsaufrufe (`Instant::now`, `thread_rng` verboten per `clippy.toml`).
- Keine Wildcard-Re-Exports (`pub use ...::*` verboten per AR-10 / T-ARCH-6).
- Keine Lecks von konkreten Engine- oder Speicher-Implementierungstypen in Schnittstellen.

## 5. Definierte Kern-Ports
- **Speicher & Indizes:** `StorageEngine`, `VectorIndex`, `TextIndex`, `GraphIndex`.
- **System & Umwelt:** `Clock`, `Rng`, `EntropyRng`, `IdGen`, `Vfs`.
- **Bereinigung & Sicherheit:** `Purger`, `LayerScanner`, `FileKeyring`, `ReadGate`, `RecordSealer`, `LedgerSink`, `LicenseGate`.

## 6. Tests, Sabotage & Qualitätssicherung
- **Contract-Tests (`ports/contract_tests.rs`):** Validierung der Port-Semantik gegen Mock- und Referenz-Implementierungen.
- **Zero-Trust Conformance:** Prüfung von Port-Verträgen und Fehlermutationen über `contextra-testkit`.
- **Workspace Clippy Gate:** Durchsetzung des Verbots direkter Zeit- und Zufallsprimitiven.

## 7. Offene Punkte & Technische Schulden
- Keine. Trait-Verträge sind versioniert und abwärtskompatibel.
