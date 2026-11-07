# AGENTS.md — `contextra-cognition`

## 1. Rolle & Zweck (Baukasten-Prinzip)
`contextra-cognition` stellt autonome Hintergrund-Dienste (Sleep Passes, Near-Duplicate Detection, Sliding-Window Clustering) und Kontext-Kompaktierung bereit (Ring 3).
Als entkoppelter Dienst aggregiert und konsolidiert er Langzeit-Gedächtnisstrukturen im Ruhezustand, ohne den Heißpfad der Datenbank zu blockieren.
Er greift über standardisierte Abstraktionen auf Speicher- und Graph-Indizes zu und ist frei von monolithischen Löschkaskaden.

## 2. Ring, Klasse, Reifegrad & `may_use`
- **Ring:** Ring 3 (Memory Consolidation & Synthesis Passes)
- **Klasse:** Erweiterung `[E]`
- **Reifegrad:** `beta`
- **`may_use` (Obergrenze):** `contextra-engine`, `contextra-types`, `contextra-ports`, `contextra-graph` (Spec §6.9, `capabilities.toml`).

## 3. Garantien & Architektur-Invarianten
- **Strikte Speichersicherheit:** `#![forbid(unsafe_code)]` wird an der Crate-Wurzel (`src/lib.rs`) erzwungen.
- **Nicht-blockierender Betrieb:** Hintergrund-Konsolidierungen laufen mit niedriger Priorität und geben Speicherressourcen bei Last sofort frei.
- **Kausalitäts-Treue:** Synthetisierte Chunks und Cluster behalten Verweise auf Ursprungstransaktionen und Quell-Dokumente.
- **Zero-Panic-Doktrin:** Vollständiger Ausschluss von `unwrap()` und `expect()`; Fehler in Hintergrund-Pässen führen niemals zum Absturz des Hauptsystems.

## 4. Verbotene Muster
- Direkte Blockierung des I/O- oder Transaktions-Heißpfads.
- Direkte Abhängigkeiten zu `contextra-core` (statdessen direkte Nutzung von `contextra-types` und `contextra-ports`).
- Monolithische Lösch- oder Nachweislogik im Crate.
- Unbegrenzte Speichernutzung bei großflächigen Graph-Clustering-Algorithmen.

## 5. Schnittstellen & Datenfluss (Ports & Abstraktionen)
- **Konsumiert:** `StorageEngine`, `GraphIndex`, `Clock`-Port.
- **Bereitgestellt für:** `contextra-engine`, `contextra-db`.

## 6. Tests, Sabotage & Qualitätssicherung
- **Konsolidierungs-Integritätstests:** Verifikation, dass Cluster-Synthesen semantisch treu und verlustfrei bleiben.
- **Sabotage-Testing:** Unterbrechung von Hintergrund-Clustering-Durchläufen bei künstlichem Speichermangel.
- **Mutations-Testing:** Prüfung auf Vermeidung von Zyklen und Duplikat-Explosionen.

## 7. Offene Punkte & Technische Schulden
- DEBT-02: Bereinigung historischer Konsolidierungs-Schulden aus frühen Prototypen.
