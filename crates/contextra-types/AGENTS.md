# AGENTS.md — `contextra-types`

## 1. Rolle & Zweck (Baukasten-Prinzip)
`contextra-types` bildet das fundamentale, I/O-freie und deterministische Typen-Fundament (Ring 0) des Contextra-Baukastensystems.
Es definiert produkt- und speicherunabhängige Domänentypen, Newtypes (`DocId`, `TxId`, `TenantId`), Identifikatoren und die universelle Fehlerrepräsentation (`ContextraError`).
Als autonomer Baustein enthält es keinerlei Geschäfts- oder Speicherdurchführungslogik und ist ohne externe Runtime-Abhängigkeiten in beliebige Fremdprojekte einbettbar.

## 2. Ring, Klasse, Reifegrad & `may_use`
- **Ring:** Ring 0 (Typen-Kern, I/O-frei, deterministisch)
- **Klasse:** Kern `[K]`
- **Reifegrad:** `stable`
- **`may_use` (Obergrenze):** `(keine)` (Spec §6.9, `capabilities.toml`) — Null Workspace-Abhängigkeiten.

## 3. Garantien & Architektur-Invarianten
- **REQ-TYP-001:** Lossless DTO-Konvertierung für `ContextraError` (`ContextraErrorDto`).
- **REQ-TYP-002:** Breite-agnostische `DocId`-Instanziierung (`DocId::new`).
- **Zero-Panic-Doktrin:** Vollständiges Verbot von unkontrolliertem `unwrap()`, `expect()` oder `panic!` in produktiven Codepfaden. Alle Konstruktoren und Konverter garantieren Fehlerbehandlung über typisierte `Result`-Rückgaben.
- **Reinheit & Determinismus (T-ARCH-1):** Kein I/O, keine Threads, kein Async, keine Systemzeit, keine Zufallszahlen.
- **Entkopplung von Lösch- und DSGVO-Logik:** Typen bleiben strikt frei von multi-level Deletion-Proofs oder DSGVO-spezifischen Begriffen; diese verbleiben exklusiv in `contextra-erasure`.

## 4. Verbotene Muster
- Kein `unsafe` (`#![forbid(unsafe_code)]` strikt erzwungen).
- Keine Asynchronität, kein Tokio, keine Netzwerk- oder Dateisystem-Imports.
- Keine Abhängigkeiten zu anderen Workspace-Crates (DAG-Wurzel).
- Keine globalen Zustände oder geteilte veränderliche Strukturen (Shared Mutable State).

## 5. Schnittstellen & Datenfluss (Ports & Abstraktionen)
- Konsumiert/Implementiert keine Port-Traits direkt; stellt reine Datenstrukturen (`DocId`, `TxId`, `TenantId`, `ContextraError`, `ScoredDocument`, `DistanceMetric`) als universelle Währung für alle Ringe bereit.

## 6. Tests, Sabotage & Qualitätssicherung
- **Unit- & Property-Testing:** Umfassende Randfall- und Property-Tests für ID-Generierung, Serialisierung und DTO-Konvertierungen.
- **Fuzzing (REQ-QA-003):** Serde- und Deserialisierungs-Fuzzing für DTOs.
- **Mutations-Testing (REQ-QA-005):** Ziel-Mutation-Score >= 85 % zur Verhinderung stiller Regressionen.

## 7. Offene Punkte & Technische Schulden
- Keine offenen Architektur-Schulden (P-01 bis P-14 bleiben juristische Prüfpunkte gemäß §20.13).
