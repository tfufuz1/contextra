# AGENTS.md — `contextra-rank`

## 1. Rolle
`contextra-rank` bietet Score-Kalibrierung (Platt, Isotonisch, Conformal) und Ergebnis-Anreicherungen zur Verfeinerung von Retrieval-Rankings.

## 2. Ring, Klasse, Reifegrad & `may_use`
- **Ring:** Ring 0 (Ranking & Kalibrierung)
- **Klasse:** Rand `[R]`
- **Reifegrad:** `experimental` (Anmerkung: In `capabilities.toml` aktuell als `stable` geführt, siehe `docs/status/ABW-L3.md`)
- **`may_use` (Obergrenze):** `contextra-types`, `contextra-ports` (Spec §6.9, `capabilities.toml`)

## 3. Garantien & Anforderungen (§33, §12)
- **REQ-RNK-001:** Deterministischer Tie-Breaker (bei gleichen Scores Sortierung nach DocId aufsteigend).
- **REQ-RNK-002:** Konforme Kalibrierung und Platt/Isotonische Skalierung ohne Mutation der Ursprungsscores.
- **Durchsetzungsmechanismus (§27.6):** `#![forbid(unsafe_code)]`, Ranking Unit-Tests.

## 4. Verbotene Muster
- **Keine Hintergrund-Threads ohne Port.**
- Kein `unsafe` (`#![forbid(unsafe_code)]`).
- **Keine zweite Standardfusion:** RRF-Standardfusion liegt im Kern (`engine`), `rank` bietet nur Kalibrierer/Erweiterungen (§6.10, Punkt 10).

## 5. Implementierte & konsumierte Ports (Anhang A)
- Implementiert: `Calibrator` Port.
- Konsumiert: Port `Metric`.

## 6. Tests & Gates
- **Ranking Tie-Breaker Tests:** Determinismus-Tests für identische Fusion-Scores.

## 7. Offene Punkte
- keine.
