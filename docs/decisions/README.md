# Architecture Decision Records (ADR) Index

Dieses Verzeichnis enthält alle Architecture Decision Records (ADRs) des Contextra-Projekts.

## Status-Definitionen
- **Proposed**: Vorschlag eingereicht, wartet auf Review und Maintainer-Freigabe.
- **Accepted**: Entscheid akzeptiert und normativ gültig.
- **Deprecated**: Durch eine spätere Entscheidung ersetzt oder veraltet.
- **Rejected**: Vorschlag abgelehnt.

## ADR-Index

| ID | Titel | Status | Datum | Relevante Komponenten |
|---|---|---|---|---|
| ADR-0001 | Gemessene Löschung: Scanner-Vertrag v1 | Accepted | 2026-10-12 | `contextra-store`, `contextra-engine` |
| ADR-0002 | Contextra Ingest Adapter Crate | Accepted | 2026-10-24 | `contextra-ingest` |
| ADR-0006 | Release-Regel: Verschlüsselung ab dem ersten Schreibvorgang | Proposed — benötigt Maintainer-Freigabe | 2026-10-29 | `contextra-crypto`, `contextra-store`, `contextra-engine` |
| ADR-0007 | Betroffenen-Erkennung: Contextra erkennt Personen in Freitext nicht | Proposed — benötigt Maintainer-Freigabe | 2026-10-29 | `contextra-engine`, `contextra-crypto`, `contextra-privacy` |
| ADR-0008 | Zweckbindung und Rechtsgrundlage für ErasedSet, Referenz-Hashes und Berichte | Proposed — benötigt Maintainer-Freigabe | 2026-10-29 | `contextra-crypto`, `contextra-engine`, `contextra-avv-generator` |
| ADR-0009 | Integration, Modularisierung und Baukastenaspekte unverdrahteter Workspace-Crates | Proposed — benötigt Maintainer-Freigabe | 2026-10-29 | `contextra-ingest`, `contextra-agent`, `contextra-kvcache` |
| ADR-0077 | HNSW-Löschung per Tombstone + Slot-Nullung (VETO-F02) | Proposed — benötigt Maintainer-Freigabe | 2026-10-29 | `contextra-vector`, `capabilities.toml` |
| ADR-0098 | Determinismus-Ausnahmen für Kryptoschlüssel & Salt | Proposed — benötigt Maintainer-Freigabe | 2026-10-29 | `contextra-store`, `contextra-crypto` |
| ADR-0099 | VETO-OP03: Kein Echtzeit-Audio/STT-Streaming im Kern | Proposed — benötigt Maintainer-Freigabe | 2026-10-29 | `contextra-core`, `contextra-engine` |

---

## Entscheidungsunterlagen & Governance-Analysen

| Dokument | Thema | Status |
|---|---|---|
| [`0003-archiv-crates-befoerderung.md`](0003-archiv-crates-befoerderung.md) | Entscheidungsvorlage Beförderung vs. Abbau archivierter Crates mit Fan-in > 0 | Proposed |
| [`0004-agents-claude-section-5-4-amendment.md`](0004-agents-claude-section-5-4-amendment.md) | Formulierungsentwurf für AGENTS.md / CLAUDE.md §5.4 Änderung | Proposed |
| [`0005-panic-measurement-model.md`](0005-panic-measurement-model.md) | Panic-Messmodell: Analyse der drei Kennzahlen & Empfehlung | Proposed |
