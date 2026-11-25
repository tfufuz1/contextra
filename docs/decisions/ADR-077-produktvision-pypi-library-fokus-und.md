# ADR-077: Produktvision PyPI-Library Fokus und Tauri Deprecation

* **Status:** Umgesetzt (physisch entfernt am 2026-09-12)
* **Datum:** 2026-09-08 (Umsetzung: 2026-09-12)
* **Target Path:** crates/contextra-tauri
* **Kontext / Auslöser:** Zielarchitektur v8.0 §6 & Entscheidungsdokumentation v1.0. Das Projekt führte zuvor drei unentschiedene Produktvisionen parallel (PyPI-Library, Desktop-Enterprise-App, Voice-Assistant).

## Entscheidung
1. **Verbindliche Fokussierung auf Option 1: PyPI-Library (Position A/B, ADR-007-Richtung)**. Contextra wird primär als hochperformante, kryptographisch isolierte Embedded AI Memory Library für Python (`contextra-py`) und Rust entwickelt.
2. **ADR-018 (Doppelstrategie) wird explizit durch diese ADR abgelöst (`superseded`)**.
3. **`contextra-tauri` wird als `deprecated` eingestuft** und im Rahmen des Crate-Konsolidierungs-Fahrplans physisch aus dem Repository entfernt.

## Begründung
- Die Entwicklungsdynamik (Schwarm-Entwicklung, Solo-Architekt) erfordert maximale Fokussierung auf die Kernstärke: kaskadierende Retrieval-Qualität und kryptographische Mandantenisolation.
- Eine Desktop-Enterprise-App bindet erhebliche Ressourcen in UI/Desktop-Packaging (Tauri/GTK), ohne direkten Beitrag zur Inferenz- und Gedächtnisleistung.

## Konsequenzen
- `contextra-py` bildet die primäre FFI-Grenzschicht.
- `contextra-tauri` wird in Folgeschritten aus der Cargo-Workspace-Topologie entfernt.
- Doku-Artefakte und README/Architecture-Guides werden entsprechend aktualisiert.

---

## Review-Log (Vetoes F-02 & OP-03)

### Veto F-02: Partielles HNSW-Rewiring / Teilgraph-Rebuilding
* **Status:** AUSSTEHEND — Frist 2026-10-07
* **Verantwortliche Rolle:** Projektleiter (gemäß stabilization_plan.md §5.4)
* **Betroffener Code:**
  - `crates/contextra-vector/src/hnsw/` (`core_rebuild.rs`, `config.rs`, `types.rs`, `vector_index_impl.rs`)
  - `crates/contextra-vector/src/partial_rebuild.rs`
* **Reviewer-Signatur:** `[AUSSTEHEND]`
* **Review-Datum:** `[AUSSTEHEND]`
* **Inhaltlicher Review-Befund / Entscheidung:** `[Inhaltliche Entscheidung über Aufhebung/Bestätigung ausstehend]`

### Veto OP-03: Realtime-Audio / Voice / Speech-to-Text / Jarvis
* **Status:** AUSSTEHEND — Frist 2026-10-07
* **Verantwortliche Rolle:** Projektleiter (gemäß stabilization_plan.md §5.4)
* **Betroffener Code:**
  - Kein betroffener Code im aktuellen Workspace gefunden (Hinweis: `crates/contextra-db/src/volatile_vault.rs` enthält lediglich das Metadaten-Enum `SignalModality::AudioTranscript`, jedoch keine Audio-Streaming-, Speech-to-Text-, Voice- oder Jarvis-Engine).
* **Reviewer-Signatur:** `[AUSSTEHEND]`
* **Review-Datum:** `[AUSSTEHEND]`
* **Inhaltlicher Review-Befund / Entscheidung:** `[Inhaltliche Entscheidung über Aufhebung/Bestätigung ausstehend]`
