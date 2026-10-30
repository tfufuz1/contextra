# Decision Proposal: Änderungsvorschlag für AGENTS.md und CLAUDE.md §5.4

> **Status-Update 30.10.2026:** Historisch. Abschnitt 5.4 ist in `AGENTS.md` Fassung 4.0 nicht mehr enthalten.

- **Status:** Proposed — erfordert Verfassungs-/Governance-Freigabe
- **Datum:** 2026-10-29

## Kontext
AGENTS.md und CLAUDE.md enthalten in §5.4 Formulierungen bezüglich archivierter Crates, die im Widerspruch zum SPEC 3.0 Baukasten-Modell stehen.

## Formulierungsvorschlag für AGENTS.md / CLAUDE.md §5.4

```markdown
### 5.4 Baukasten-Module [M] und Archivierungs-Kriterien

Baukasten-Module [M] (wie `contextra-graph`, `contextra-adapt`, `contextra-router`, `contextra-cognition`, `contextra-infer-*`)
sind aktive, entkoppelte Module mit isoliertem Vertrag. Sie dürfen vom Kern (`[K]`) nur über definierte Port-Schnittstellen
angewählt werden (INV-CORE-ISOLATION-1).

Eine physikalische Archivierung eines Crates erfolgt ausschließlich nach vollständigem Abbau aller Fan-in-Abhängigkeiten
und formellem ADR-Beschluss der Maintainer.
```

## Hinweis zu CONSTITUTION.md / VETOES.md
Änderungen an CONSTITUTION.md und VETOES.md sind geschützte Verfassungsdokumente. Dieser Vorschlag stellt keine eigenmächtige Verfassungsänderung dar, sondern bereitet die redaktionelle Anpassung für die Maintainer-Freigabe vor.
