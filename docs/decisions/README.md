# Architecture Decision Records (ADRs)

Das Verzeichnis `docs/decisions/` ist die einzige kanonische Quelle (Single Source of Truth) für Architecture Decision Records im Contextra-Projekt.

## Struktur & Lifecycle
- Jede Entscheidung wird als `ADR-NNN-slug.md` <!-- doc-ref-ignore --> in diesem Verzeichnis geführt.
- Der Status jeder ADR (z. B. `Proposed`, `Accepted`, `Rejected`, `Deprecated`, `Superseded`) ist im YAML/Markdown-Front-Matter der jeweiligen Datei dokumentiert.

## Wiring-Richtlinie & Definition von "verdrahtet"
- **Definition "verdrahtet":** Ein Symbol oder Subsystem gilt nur dann als *verdrahtet*, wenn:
  1. mindestens ein **Produktionsaufrufer** in einem anderen Modul oder Crate existiert UND
  2. ein **Integrationstest/Verdrahtungstest** den echten Produktionspfad durchläuft.
  Re-Exporte, Selbstreferenzen, Test-only-Aufrufer oder Fuzz-Aufrufer zählen *nicht* als Verdrahtung.
- **Verbot von Scheinverdrahtung / hohlen Wiring-Typen:**
  Ein `<Name>Wiring`-Typ ohne Inhalt (z. B. leeres Struct, `build(_deps: ())` ohne Funktion) oder ohne echten Produktionskonsumenten ist als Scheinverdrahtung unzulässig. Jeder Wiring-Einstieg muss echte Abhängigkeiten injizieren und im Composition Root aufgerufen werden.

## Wichtige Architektur-Entscheidungen (Auswahl)
- [ADR-SCOPE-FREEZE-2026-10](./ADR-SCOPE-FREEZE-2026-10.md): Scope Freeze & Verbindliche Crate-Klassifikation (Klasse K: Kern, Klasse R: Rand, Klasse E: Experimentell gemäß Spezifikation §6.0).

## Befehle
- Mit `just adr list` (geplant) können alle vorhandenen ADRs übersichtlich aufgelistet werden.

## Neueste Entscheidungen
- [ADR-SCOPE-FREEZE-2026-10](ADR-SCOPE-FREEZE-2026-10.md): Scope Freeze & Crate-Klassifizierung (Kern, Rand, Experimentell)

## Vorgeschlagene ADR-T Entwürfe (In Arbeit / Proposed)
- [ADR-T01-P29-kein-globaler-veraenderlicher-zustand](ADR-T01-P29-kein-globaler-veraenderlicher-zustand.md): Prinzip P29 — Kein globaler veränderlicher Zustand (Entwurf)
- [ADR-T02-INV-KMS-1-kms-provider-verdrahtung](ADR-T02-INV-KMS-1-kms-provider-verdrahtung.md): INV-KMS-1 — Strikte KMS-Provider-Verdrahtung (Entwurf)
- [ADR-T03-INV-DELETION-WIRE-1-loeschpfad-verdrahtung](ADR-T03-INV-DELETION-WIRE-1-loeschpfad-verdrahtung.md): INV-DELETION-WIRE-1 — Durchgängige Löschpfad-Verdrahtung (Entwurf)
- [ADR-T04-INV-SSI-ACTIVE-1-ssi-durchsetzung](ADR-T04-INV-SSI-ACTIVE-1-ssi-durchsetzung.md): INV-SSI-ACTIVE-1 — Strikte SSI-Durchsetzung (Entwurf)
- [ADR-T05-INV-DOC-SINGLE-SOURCE-1-doc-single-source](ADR-T05-INV-DOC-SINGLE-SOURCE-1-doc-single-source.md): INV-DOC-SINGLE-SOURCE-1 — Doc Single Source of Truth (Entwurf)
- [ADR-T06-INV-RING0-DETERMINISM-1-ring0-determinismus](ADR-T06-INV-RING0-DETERMINISM-1-ring0-determinismus.md): INV-RING0-DETERMINISM-1 — Ring-0 Determinismus (Entwurf)
