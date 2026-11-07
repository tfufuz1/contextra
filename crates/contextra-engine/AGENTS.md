# AGENTS.md — `contextra-engine`

## 1. Rolle & Zweck (Baukasten-Prinzip)
`contextra-engine` dient als generische Orchestrierungs- und Ausführungsschicht (Ring 3) für asynchrone Transaktionsabwicklung, Multi-Index-Koordination und Background-Maintenance.
Als entkoppelter Baustein bindet er Speicher-, Vektor-, Text- und Graph-Subsysteme nicht über konkrete Implementierungen, sondern ausschließlich über den generischen `Backends`-Trait und standardisierte Ports aus `contextra-ports`.
Transaktions-Workflows werden über zustandsbehaftete Typestates und das agnostic State-Machine-Muster deterministisch geführt.

## 2. Ring, Klasse, Reifegrad & `may_use`
- **Ring:** Ring 3 (Compute Pool & Storage Execution Layer)
- **Klasse:** Kern `[K]`
- **Reifegrad:** `stable`
- **`may_use` (Obergrenze):** `contextra-types`, `contextra-ports`, `contextra-sys`, `contextra-mvcc`, `contextra-crypto`, `contextra-store`, `contextra-vector`, `contextra-text`, `contextra-checkpoint`, `contextra-rank`, `contextra-graph` (Spec §6.9, `capabilities.toml`).

## 3. Garantien & Architektur-Invarianten
- **Strikte Speichersicherheit:** `#![forbid(unsafe_code)]` wird an der Crate-Wurzel (`src/lib.rs`) strikt durchgesetzt.
- **Deterministische TxId-Allokation:** Monoton steigende Transaktions-IDs ohne Kausalitätsbrüche.
- **Zustandsmaschinen-Disziplin:** Typestate-gestützte Workflow-Transitions (`WorkflowStep`, `EngineServiceContainer`) verhindern ungültige Zwischenzustände.
- **Zero-Panic-Doktrin:** Vollständiger Ausschluss von `unwrap()` und `expect()`; alle Fehler propagieren als `ContextraError` via `?`.
- **Löschketten-Entkopplung:** Keine internen Deletion-Sagas oder Proof-Generatoren; Löschungen werden an `contextra-erasure` delegiert.

## 4. Verbotene Muster
- Jegliche Verwendung von `unsafe`.
- Feste Verdrahtung auf konkrete Index- oder Store-Typen anstelle von `contextra-ports`-Abstraktionen.
- Blockieren des async Event-Loops durch rechenintensive oder synchrone I/O-Tasks.
- Umgehen der `DbTransaction`-Transaktionsklammern.

## 5. Schnittstellen & Datenfluss (Ports & Abstraktionen)
- **Implementiert:** Orchestrierungs-Dienste, `Collection`-Verwaltung, `DbTransaction`.
- **Konsumiert:** `StorageEngine`, `VectorIndex`, `TextIndex`, `GraphIndex`, `Clock`-Port.
- **Bereitgestellt für:** `contextra-db`, `contextra-cognition`, `contextra-ingest`.

## 6. Tests, Sabotage & Qualitätssicherung
- **State-Machine Concurrency Tests:** Parallele Transaktionsdurchläufe mit Commit- und Abort-Rennen.
- **Sabotage-Testing:** Gezielte Unterbrechung von Workflows zur Prüfung der deterministischen Rollback-Garantie.
- **Mutations-Testing:** Prüfung auf Datenintegrität bei simulierten Backend-Timeouts.

## 7. Offene Punkte & Technische Schulden
- DEBT-04: Vollständige Monomorphisierung aller Heißpfade über den `Backends`-Trait.
