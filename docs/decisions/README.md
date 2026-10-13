# Architecture Decision Records (ADRs)

Das Verzeichnis `docs/decisions/` ist die einzige kanonische Quelle (Single Source of Truth) für Architecture Decision Records im Contextra-Projekt.

## Struktur & Lifecycle
- Jede Entscheidung wird als `ADR-NNN-slug.md` <!-- doc-ref-ignore --> in diesem Verzeichnis geführt.
- Der Status jeder ADR (z. B. `Proposed`, `Accepted`, `Rejected`, `Deprecated`, `Superseded`) ist im YAML/Markdown-Front-Matter der jeweiligen Datei dokumentiert.

## Wichtige Architektur-Entscheidungen (Auswahl)
- [ADR-SCOPE-FREEZE-2026-10](./ADR-SCOPE-FREEZE-2026-10.md): Scope Freeze & Verbindliche Crate-Klassifikation (Klasse K: Kern, Klasse R: Rand, Klasse E: Experimentell gemäß Spezifikation §6.0).

## Befehle
- Mit `just adr list` (geplant) können alle vorhandenen ADRs übersichtlich aufgelistet werden.

## Neueste Entscheidungen
- [ADR-SCOPE-FREEZE-2026-10](ADR-SCOPE-FREEZE-2026-10.md): Scope Freeze & Crate-Klassifizierung (Kern, Rand, Experimentell)
