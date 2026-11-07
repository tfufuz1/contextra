# AGENTS.md — `contextra-testkit`

## 1. Rolle & Zweck (Baukasten-Prinzip)
`contextra-testkit` stellt ein umfassendes Test- und Verifikations-Harness (Tooling) für das gesamte Contextra-Baukastensystem bereit.
Es enthält Test-Fixtures, In-Memory Referenz-Implementierungen, Fault-Injection-Mocks (`FailpointVfs`), Chaos-Matrizen und Zero-Trust Conformance-Suiten zur Validierung von Port-Verträgen.
Als reines Tooling-Crate ist es von Produktionscode entkoppelt und dient ausschließlich der Härtung und Sabotage-Prüfung aller Komponenten.

## 2. Ring, Klasse, Reifegrad & `may_use`
- **Ring:** Tooling
- **Klasse:** Werkzeug `[W]`
- **Reifegrad:** `stable`
- **`may_use` (Obergrenze):** `contextra-types`, `contextra-ports` (Spec §6.9, `capabilities.toml`).

## 3. Garantien & Architektur-Invarianten
- **Zero-Trust Conformance:** Validierung sämtlicher Port-Verträge (`StorageEngine`, `VectorIndex`, `TextIndex`, `Clock`, `Vfs` etc.) gegen fehlerhafte Mutanten; 100 % Erkennungsrate fehlerhafter Implementierungen.
- **Deterministische Fehlerinjektion:** Präzise steuerbare Fehler-Szenarien (I/O-Timeouts, Corruption, ENOSPC).
- **Keine Produktiv-Abhängigkeiten:** `contextra-testkit` darf NIEMALS in Produktiv-Abhängigkeiten von Ring 0-3 Crates auftauchen (nur als `dev-dependencies`).

## 4. Verbotene Muster
- Import von `contextra-testkit` als reguläre Abhängigkeit in Bibliotheken.
- Flaky Tests durch unkontrollierte echte Systemzeit oder echte Zufallsquellen (nur injizierte `Clock`/`Rng`).
- Verwendung von `contextra-core`.

## 5. Schnittstellen & Datenfluss (Ports & Abstraktionen)
- **Implementiert:** Test-Mocks für sämtliche Ports aus `contextra-ports`.
- **Bereitgestellt für:** Unit- und Integrationstests aller Workspace-Crates.

## 6. Tests, Sabotage & Qualitätssicherung
- **Conformance Self-Tests:** `conformance_selftest` und `tk_fault_vfs_conformance` beweisen, dass mutierte Fehler deterministisch erkannt werden.
- **Chaos-Matrix:** Multi-Threaded Stress-Tests unter extremen Fehlerinjektionen.

## 7. Offene Punkte & Technische Schulden
- Keine.
