# Architecture Decision Records (ADRs)

Das Verzeichnis `docs/decisions/` ist die einzige kanonische Quelle (Single Source of Truth) für Architecture Decision Records im Contextra-Projekt.

## Struktur & Lifecycle
- Jede Entscheidung wird als `ADR-NNN-slug.md` in diesem Verzeichnis geführt.
- Der Status jeder ADR (z. B. `Proposed`, `Accepted`, `Rejected`, `Deprecated`, `Superseded`) ist im YAML/Markdown-Front-Matter der jeweiligen Datei dokumentiert.

## Befehle
- Mit `just adr list` (geplant) können alle vorhandenen ADRs übersichtlich aufgelistet werden.
