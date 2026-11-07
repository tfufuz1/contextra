# AGENTS.md — `contextra-router`

## 1. Rolle & Zweck (Baukasten-Prinzip)
`contextra-router` ist eine dynamische Routing- und Entscheidungs-Engine (Ring 3) zur adaptiven Modellauswahl (Small Language Models / SLM-Routing) und Drift-Kontrolle.
Er bewertet Modellkapazitäten, Latenzbudgets und Domänen-Affinitäten und wendet `ConformalCalibrator` sowie Lyapunov Drift Watcher an, um Routing-Entscheidungen kontinuierlich nachzuführen.
Als eigenständiger Baustein kapselt er die Auswahl- und Dispatching-Logik und interagiert über lose gekoppelte Ports mit Inferenz- und Speichermodulen.

## 2. Ring, Klasse, Reifegrad & `may_use`
- **Ring:** Ring 3 (Drift Control & Multi-Candidate Search Routing)
- **Klasse:** Erweiterung `[E]`
- **Reifegrad:** `stable`
- **`may_use` (Obergrenze):** `contextra-types`, `contextra-wire`, `contextra-ports`, `contextra-adapt` (Spec §6.9, `capabilities.toml`).

## 3. Garantien & Architektur-Invarianten
- **Strikte Speichersicherheit:** `#![forbid(unsafe_code)]` wird über das gesamte Crate hinweg erzwungen.
- **Budget-Einhaltung:** Ein Modell darf niemals ausgewählt werden, wenn das aktuelle Token-Fenster (`ContextWindow`) dessen Kapazitätsgrenzen übersteigt.
- **Conformal Calibration (ADR-050, ADR-054):** Atomare Updates des Profil-Kalibrierungszustands ohne TOCTOU-Scoring-Rennen.
- **Zero-Panic-Doktrin:** Vollständiger Ausschluss von `unwrap()` und `expect()`; robuste Fallbacks bei Ausfall einzelner Inferenz-Ziele.

## 4. Verbotene Muster
- Direkte Abhängigkeiten zu `contextra-privacy` (Entkopplung über den Standard-Port `EgressPolicy`).
- Verwendung von `contextra-core` (statdessen direkte Nutzung von `contextra-types` und `contextra-ports`).
- Blindes Dispatching ohne vorherige Budget- und Vertrauensbereichs-Prüfung.
- Geschachtelte Locks über Modulgrenzen hinweg.

## 5. Schnittstellen & Datenfluss (Ports & Abstraktionen)
- **Implementiert:** `RouterEngine`, `LyapunovDriftWatcher`, `RoutingStrategy`.
- **Konsumiert:** `Clock`-Port, `EgressPolicy`, `contextra-adapt`.
- **Bereitgestellt für:** `contextra-mcp`, `contextra-agent`.

## 6. Tests, Sabotage & Qualitätssicherung
- **Conformal Calibration Verifikation:** Tests zur Konvergenz der Nicht-Konformitäts-Scores bei Modell-Drift.
- **TOCTOU Race Detection:** Paralleltests mit konkurrierenden Profil-Updates unter extremer Last.
- **Sabotage-Testing:** Gezielte Injektion unerwarteter Modell-Fehlerraten zur Prüfung der Umschaltlogik.

## 7. Offene Punkte & Technische Schulden
- DEBT-04: Vollständige Entkopplung der temporären `contextra-db` Dev-Dependencies.
