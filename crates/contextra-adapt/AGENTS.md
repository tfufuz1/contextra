# AGENTS.md — `contextra-adapt`

## 1. Rolle & Zweck (Baukasten-Prinzip)
`contextra-adapt` ist ein autonomer Regelungs- und Adaptions-Baustein (Ring 0) zur dynamischen Systemanpassung (PID Latency Controller, Adaptive Candidate Pool Sizing).
Er modelliert mathematische Feedback-Schleifen zur Laufzeit, um Latenzbudgets deterministisch einzuhalten und Suchpool-Größen adaptiv zu steuern.
Als orthogonaler Algorithmen-Baustein ist er unabhängig von konkreten Storage- oder Vektortechnologien und kann in beliebige Systeme zur Feedback-Regelung integriert werden.

## 2. Ring, Klasse, Reifegrad & `may_use`
- **Ring:** Ring 0 (Regelungstechnik & Modell-Adaption)
- **Klasse:** Rand `[R]`
- **Reifegrad:** `beta`
- **`may_use` (Obergrenze):** `contextra-ports`, `contextra-types` (Spec §6.9, `capabilities.toml`).

## 3. Garantien & Architektur-Invarianten
- **Numerische Stabilität:** Bounds-Checking und Sättigungs-Grenzwerte (`saturating_*`, Clamping) für Integratoren und Differentialglieder des PID-Reglers.
- **Deterministische Latenz-Regelung:** Strikte Konvergenz gegen konfigurierte Latenz-Sollwerte ohne Oszillation.
- **Zero-Panic-Doktrin:** Vollständiger Ausschluss von `unwrap()`, `expect()` oder Panics; fehlerhafte oder extreme Messwerte führen zu sanfter Dämpfung statt Systemabstürzen.

## 4. Verbotene Muster
- Direkte I/O-, Netzwerk- oder Persistenzaufrufe im Regelkern.
- Halten von globalem ungeschütztem Mutable State.
- Querabhängigkeiten zu Speicher- oder Inferenz-Engines (nur über Traits).

## 5. Schnittstellen & Datenfluss (Ports & Abstraktionen)
- **Konsumiert:** `Clock`-Port zur präzisen Zeitdifferenz-Berechnung.
- **Bereitgestellt für:** `contextra-router`, `contextra-engine`, `contextra-mcp` (`PoolController`, `DynamicRerankOrchestrator`).

## 6. Tests, Sabotage & Qualitätssicherung
- **Mathematische Konvergenztests:** Simulation von Lastspitzen und Verifikation der Einschwingzeit des PID-Controllers.
- **Sabotage-Tests:** Injektion extremer Ausreißerwerte und NaN/Inf-Werte zur Überprüfung der numerischen Dämpfung.
- **Property-Based Testing:** Validierung stabiler Regeleigenschaften über Testbereiche.

## 7. Offene Punkte & Technische Schulden
- Keine.
