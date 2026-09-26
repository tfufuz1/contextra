# Contextra Spezifikation — Index und Kapitelübersicht

Dieses Verzeichnis `docs/spec/` enthält die vollständige, verbindliche Gesamtspezifikation **Contextra — Finale Produktspezifikation (Synthese)** (`CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md`), sowie die vorausgegangenen Kapiteldateien.

> **Hinweis zur Normativität**: Die unten aufgeführten nummerierten Kapiteldateien (`00a`…`21`…`92`) sind historische Vorstufen der Synthese. Bei etwaigen Widersprüchen gilt ausschließlich `CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md` als normative Quelle der Wahrheit.

## Hauptspezifikation

- **[`CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md`](CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md)**: Normative Mikrofeingranulare Schnittstellen- und Implementierungsspezifikation des Zielprodukts (Synthese aller Vorkapitel).

## Hinweis zur Namenskollision 'Anhang E'

In externen SOLL-Spezifikationsdokumenten (z. B. "Contextra Master-Spezifikation v6") wird der Name "Anhang E" für zwei unterschiedliche Dinge verwendet:
1. Referenztabellen für Trait-/Fehler-/Abhängigkeitskataloge vs.
2. "Teil E — Autonome Intelligenzschicht" / CIAI.

Für dieses Repository gilt ausschließlich E.1–E.13 der CIAI-Vollspezifikation als "Teil E". Der Referenztabellen-Anhang ist in diesem externen Dokument nicht vorhanden und muss bei Bedarf aus Code + `capabilities.toml` + `AGENTS.md` neu erzeugt werden.

## Kapitelindex

| Dateiname | Kapitelüberschrift (Originalwortlaut) | Grober Ring-/Crate-Bezug |
|---|---|---|
| `CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md` | Contextra — Finale Produktspezifikation (Synthese) | Global / Ring 0–4 / Master |
| `00a-titel-und-einleitung.md` | Contextra — Finale Konsolidierte Gesamtspezifikation (Fassung 4 · SOTA-Algorithmen-Integration & Architekten-Review) | Global / Workspace |
| `00b-teil-a-stabilisierungsauftrag.md` | ## Teil A — Stabilisierungsauftrag: Ground Truth, Reifegrade, Gates | Global / Ring 0–4 |
| `00c-teil-a2-zielarchitektur-v2.md` | ## Teil A2 — Zielarchitektur v2 (Ring-Modell), verbindlich ab sofort | Ring 0–4 / Crate-Graph |
| `00d-teil-a3-aktueller-umsetzungsstand.md` | ## Teil A3 — Aktueller Umsetzungsstand & priorisierte Restarbeit (Fassung 3, Quelle der Wahrheit) | Global / Workspace |
| `00e-teil-a4-architekten-review-sota.md` | ## Teil A4 — Architekten-Review: Machbarkeit & Optimierungspotenzial der SOTA-Algorithmen (Fassung 4) | `contextra-graph`, `contextra-rank`, `contextra-adapt`, `contextra-cognition` |
| `00-meta-workspace-layout.md` | ## 0. Meta: Workspace-Layout und Build-Konfiguration | Workspace / Tooling / `xtask` |
| `01-kernthese-und-leitprinzip.md` | ## 1. Kernthese und Leitprinzip | Global / Ring 0–4 |
| `02-produktvision-und-nicht-ziele.md` | ## 2. Produktvision, Alleinstellungsmerkmale und Nicht-Ziele | Global / Ring 0–4 |
| `03-architekturprinzipien.md` | ## 3. Architekturprinzipien P1–P30 | Global / Ring 0–4 |
| `04-crate-graph.md` | ## 4. Systemarchitektur: der Crate-Graph (Ring-Modell, vormals Crate-DAG) | Ring 0–4 Crate-Graph |
| `05-speicherschicht.md` | ## 5. Speicherschicht: LSM-Tree, WAL und Block-Cache | Ring 1 (`contextra-store`, `contextra-kvcache`) |
| `06-wissensgraph-datenmodell-a.md` | ## 6. Wissensgraph-Datenmodell: binäre Kanten und n-äre Hyperkanten (Teil A: §6.1–§6.4) | Ring 0 (`contextra-graph`) |
| `06-wissensgraph-datenmodell-b.md` | ## 6. Wissensgraph-Datenmodell: binäre Kanten und n-äre Hyperkanten (Teil B: §6.5–§6.7) | Ring 0 (`contextra-graph`) |
| `07-retrieval-pipeline.md` | ## 7. Retrieval-Pipeline: 4-Signal-Fusion und ihre Algorithmen | Ring 0/1/3 (`contextra-text`, `contextra-graph`, `contextra-vector`, `contextra-rank`) |
| `08-contextual-bandit-routing.md` | ## 8. Contextual-Bandit-Routing | Ring 0/3 (`contextra-adapt`, `contextra-router`) |
| `09-inferenz-kv-cache-ipc.md` | ## 9. Inferenz, KV-Cache v2 und Zero-Copy-IPC | Ring 0/1/3 (`contextra-kvcache`, `contextra-wire`, `contextra-infer-candle`) |
| `10-sicherheits-und-datenschutzmodell.md` | ## 10. Sicherheits- und Datenschutzmodell | Ring 0/2 (`contextra-crypto`, `contextra-sandbox`) |
| `11-betriebsmodi.md` | ## 11. Betriebsmodi | Global / Embedded & Server |
| `12-flatbuffers-schema.md` | ## 12. FlatBuffers-Schema (vollständig, `schemas/contextra.fbs`) | Ring 0 (`contextra-wire`) |
| `13-fehlertaxonomie.md` | ## 13. Fehlertaxonomie (crateübergreifend) | Ring 0 (`contextra-core`, `contextra-types`) |
| `14-feature-flag-politik.md` | ## 14. Feature-Flag-Politik: Produktions-Default vs. Opt-in | Workspace / Cargo Features |
| `15-test-und-ci-spezifikation.md` | ## 15. Test- und CI-Spezifikation | Tooling / CI (`.github/workflows`) |
| `16-abnahmekriterien.md` | ## 16. Vollständige Abnahmekriterien | Global / Akzeptanztests |
| `17-optimierungs-roadmap.md` | ## 17. Priorisierte Optimierungs-Roadmap (Opus-Analyse) | Global / Roadmap |
| `18-gesamtroadmap.md` | ## 18. Gesamtroadmap | Global / Roadmap |
| `19-rueckverfolgbarkeitsmatrix.md` | ## 19. Rückverfolgbarkeitsmatrix | Global / Traceability |
| `20-migrationsplan-und-adr.md` | ## 20. Migrationsplan v2 und ADR-Übersicht (neu) | Workspace / `docs/decisions/` |
| `21-sota-algorithmen-erweiterung-a.md` | ## 21. Normative SOTA-Algorithmen-Erweiterung (Teil A: §21.1–§21.2 TL-HFD, DiBud) | Ring 0/1 (`contextra-graph`, `contextra-rank`) |
| `21-sota-algorithmen-erweiterung-b.md` | ## 21. Normative SOTA-Algorithmen-Erweiterung (Teil B: §21.3–§21.4 FC-TS, LeanRAG) | Ring 0/3 (`contextra-adapt`, `contextra-cognition`) |
| `90-anhang-b-begruendungen-a.md` | # Anhang B — Begründungen, Ist-Zustand, Literatur (Teil A: §B.5.1–§B.5.2) | Ring 0/3 (`contextra-graph`, `contextra-router`) |
| `90-anhang-b-begruendungen-b.md` | # Anhang B — Begründungen, Ist-Zustand, Literatur (Teil B: §B.5.3–§B.7) | Ring 0/1 (`contextra-store`, `contextra-vector`) |
| `91-anhang-c-aenderungsprotokoll-v2-1.md` | # Anhang C — Änderungsprotokoll Fassung 2.1 und Prüfnachweise | Meta / Dokumentenhistorie |
| `92-anhang-d-aenderungsprotokoll-v4.md` | # Anhang D — Änderungsprotokoll Fassung 4 und Prüfnachweise | Meta / Dokumentenhistorie |
