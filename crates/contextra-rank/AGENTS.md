# AGENTS.md — `contextra-rank`

## 1. Rolle & Zweck (Baukasten-Prinzip)
`contextra-rank` bietet isolierte Algorithmen für Score-Kalibrierung (Platt Scaling, Isotonische Regression, Conformal Calibration) und Ranking-Anreicherungen (Ring 0).
Er dient der Verfeinerung und Kalibrierung von Konfidenzwerten über mehrere Retrieval-Signale hinweg.
Als wiederverwendbarer Algorithmen-Baustein ist er strikt von Persistenz- und Netzwerk-Details entkoppelt und kann in beliebigen Such-Pipelines genutzt werden.

## 2. Ring, Klasse, Reifegrad & `may_use`
- **Ring:** Ring 0 (Ranking & Kalibrierung)
- **Klasse:** Rand `[R]`
- **Reifegrad:** `stable`
- **`may_use` (Obergrenze):** `contextra-types`, `contextra-ports` (Spec §6.9, `capabilities.toml`).

## 3. Garantien & Architektur-Invarianten
- **REQ-RNK-001:** Deterministischer Tie-Breaker (bei identischen Relevanz-Scores deterministische Sortierung nach `DocId` aufsteigend).
- **REQ-RNK-002:** Konforme Kalibrierung und Platt/Isotonische Skalierung ohne unerwünschte Mutation von Ursprungsscores.
- **Score-Isolation:** Keine Halluzination oder Erfindung von Relevanzwerten; Skalierung erfolgt nach strengen statistischen Modellen.
- **Zero-Panic-Doktrin:** Keine unkontrollierten Panics bei Division durch Null oder leeren Trefferlisten.

## 4. Verbotene Muster
- Kein `unsafe` (`#![forbid(unsafe_code)]` strikt erzwungen).
- **Keine zweite Standardfusion:** RRF-Standardfusion liegt im Kern (`contextra-engine`/`ports`), `rank` bietet reine Kalibrierer und Erweiterungen.
- Keine Hintergrund-Threads oder asynchronen Event-Loops ohne expliziten Port.
- Keine Querabhängigkeiten zu Speicher-Engines oder Inferenz-Servern.

## 5. Schnittstellen & Datenfluss (Ports & Abstraktionen)
- **Implementiert:** `Calibrator`-Port, lokale `ImportanceCalibrator`-Traits für Entkopplung.
- **Bereitgestellt für:** `contextra-engine`, `contextra-router`, `contextra-infer-*`.

## 6. Tests, Sabotage & Qualitätssicherung
- **Tie-Breaker Determinismus-Tests:** Validierung stabiler Sortierreihenfolgen bei Score-Gleichständen.
- **Statistische Kalibrierungstests:** Verifikation der Wahrscheinlichkeitsdichten kalibrierter Scores.
- **Mutations-Testing:** Prüfung auf Monotonie-Verletzungen nach Kalibrierungsdurchläufen.

## 7. Offene Punkte & Technische Schulden
- Keine.
