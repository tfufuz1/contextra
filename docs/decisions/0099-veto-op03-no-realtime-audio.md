# ADR-0099: VETO-OP03 — Kein Echtzeit-Audio/STT-Streaming im Kern

- **Status:** Proposed — benötigt Maintainer-Freigabe
- **Datum:** 2026-10-29
- **Autoren:** Architecture Governance Team
- **Relevante Komponenten:** `contextra-core`, `contextra-engine`, `docs/VETOES.md`

## Kontext
Im Veto-Register (`docs/VETOES.md`) ist **VETO-OP03** als unumstößliches Architekturveto definiert. Es betrifft die Behandlung von Audiodaten und Speech-to-Text (STT) Pipelines.

## Entscheidung
1. Im Kern des Contextra-Systems (`contextra-core`, `contextra-engine`, `contextra-db`) ist jegliche Verarbeitung von Echtzeit-Audio-Streaming oder Speech-to-Text (STT) Pipelines strikt verboten (geschlossen).
2. Audiodaten dürfen ausschließlich außerhalb des Kerns in vorgeschalteten Rändern verarbeitet und als extrahierter Text/Embedding an Contextra übergeben werden.

## Konsequenzen
- **Positiv:** Strikte Reduktion der Komplexität und Verringerung der Angriffsfläche im Kern.
- **Negativ:** Audio-Verarbeitung muss vollständig in externen Client-Systemen erfolgen.
