# Contextra — Quellen der Wahrheit

<!-- doc-stamp: 2026-11-08 | Spec-Fassung: 4.1 -->

> **Normative Grundregel (INV-DOC-SINGLE-SOURCE-1):** Jede Tatsache hat genau eine kanonische Quelle.
> Abgeleitetes wird generiert oder per Doc-Truth-Gate geprüft. Doppelt vergebene IDs sind Gate-Fehler.

---

## Rangfolge der Informationsquellen

Bei Konflikten gilt zwingend:

| Rang | Quelle | Inhalt |
|------|--------|--------|
| 1 | **`docs/SPEC.md`** (Fassung 4.1) | Einzige normative Quelle: Baukasten-Architektur, Verhalten, Invarianten, Crate-Soll-Zustände (§10–§14), Gates G1–G10, Entscheidungsregister E-01 bis E-12 |
| 2 | **`docs/VETOES.md`** | Veto-Register (VETO-F02, VETO-F10, VETO-OP03, VETO-ADR) |
| 3 | **`capabilities.toml`** | Single Source of Truth für Ring, Klasse (A/B/C/T), Reifegrad und `may_depend_on` je Crate |
| 4 | **`AGENTS.md`** | Agenten-Betriebsanleitung, Baukasten-Vertrag (M1–M11), Arbeitsregeln |
| 5 | **Produktionscode + grüne Quality Gates** | Tatsächlicher Codezustand und Verifikationsergebnisse |

---

## Dokument-Karte

| Dokument | Typ | Inhalt |
|----------|-----|--------|
| `docs/SPEC.md` | **normativ** | Gesamtspezifikation des Baukastens (Fassung 4.1, Stand 2026-11-08) |
| `docs/VETOES.md` | **normativ** | Veto-Register für unzulässige Features und Architekturbrüche |
| `docs/CONSTITUTION.md` | **normativ** | Grundlegende Architektur- und Governance-Prinzipien |
| `capabilities.toml` | **normativ (Single Source)** | Ring, Klasse (A/B/C/T), Reifegrad, `may_depend_on` je Crate |
| `AGENTS.md` | **normativ** | Operative Agenten-Betriebsanleitung und Baukasten-Vertrag |
| `PLAN.md` | Arbeitsunterlage | Implementierungs- und Refaktorisierungsplan zur Baukasten-Transformation |
| `docs/ARCHITECTURE.md` | Lese-Hilfsmittel | Kompakte Darstellung von Architektur und Baukasten-Struktur (nicht normativ) |
| `docs/TYPE_REGISTRY.md` | Arbeitsunterlage | Typ-Registry und Kollisionsüberwachung gemäß B3/B4/E-02 |
| `docs/SECURITY.md` | informativ | Sicherheits- und Bedrohungsmodell für Bausteine und Speicherkern |
| `docs/CONTRIBUTING.md` | informativ | Beitragsrichtlinien und Entwicklungsabläufe |
| `docs/CHANGELOG.md` | historisch | Chronologisches Änderungsprotokoll des Repositories |
| `docs/decisions/` | normativ | Architektur-Entscheidungs-Aufzeichnungen (ADRs) |
| `governance/gates.toml` | normativ | Gate-Definitionen |
| `governance/ratchet.toml` | normativ | Ratchet-Baselines |

---

## Veraltete / archivierte Dokumente

> **Diese Dokumente sind OBSOLET und dürfen NICHT mehr als Referenz verwendet werden.**
> Sie wurden durch `docs/SPEC.md` Fassung 4.1 ersetzt oder in `docs/archive/` verschoben.

- Frühere Spezifikationsfassungen (Fassung 2.3, 3.0, 4.0-Reduktionsfassung) → ersetzt durch `docs/SPEC.md` Fassung 4.1
- `docs/archive/SPEC_v3_oekosystem.md` → archivierte Plattform-Spezifikation v3.0
- `docs/archive/MODULAR_DECOUPLING_AUDIT.md` → historische Entkopplungsanalyse
- `docs/archive/ZIELARCHITEKTUR_BAUKASTEN.md`, `docs/archive/BAUKASTEN_TRANSFORMATION_ABSCHLUSSBERICHT.md` → archiviert
- Veraltete Monolith-Spezifikationen (`docs/CONTEXTRA_SPEZIFIKATION.md`, `docs/GESAMTSPEZIFIKATION.md`, `docs/FEATURES_SPECIFICATION.md`) → obsolet

---

*Gate `doc-truth` prüft: jede normative ID einmal definiert, Dokumente mit Stempel versehen.*
