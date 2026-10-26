# Contextra — Quellen der Wahrheit

<!-- doc-stamp: 2026-10-26 | Spec-Fassung: 2.3 -->

> **Normative Grundregel (INV-DOC-SINGLE-SOURCE-1):** Jede Tatsache hat genau eine kanonische Quelle.
> Abgeleitetes wird generiert oder per Doc-Truth-Gate geprüft. Doppelt vergebene IDs sind Gate-Fehler.

---

## Rangfolge der Informationsquellen

Bei Konflikten gilt zwingend:

| Rang | Quelle | Inhalt |
|------|--------|--------|
| 1 | **`docs/SPEC.md`** (Fassung 2.3) | Einzige normative Quelle: Architektur, Verhalten, Konstanten, Tests, Gates, Plan |
| 2 | **`docs/VETOES.md`** | Veto-Register (VETO-F02, VETO-F10, VETO-OP03, Single-Node) |
| 3 | **`capabilities.toml`** | Ring, Klasse, Reifegrad, `may_depend_on` je Crate |
| 4 | **`AGENTS.md`** | Agenten-Betriebsanleitung, WP-Status, IST-Befunde |
| 5 | **Produktionscode + grüne Quality Gates** | REQ-QA-010 |

---

## Dokument-Karte

| Dokument | Typ | Inhalt |
|----------|-----|--------|
| `docs/SPEC.md` | **normativ** | Gesamtspezifikation v1.0 (Fassung 2.3) |
| `docs/PLAN.md` | Arbeitsunterlage | Implementierungsplan IST→SOLL, WP 0.1–9.4 |
| `docs/ARCHITECTURE.md` | Lese-Hilfsmittel | Kompakte Darstellung von SPEC §4 (nicht normativ) |
| `docs/VETOES.md` | normativ | Veto-Register |
| `docs/CONSTITUTION.md` | normativ | Grundlegende Governance-Prinzipien |
| `docs/CONTRIBUTING.md` | informativ | Beitragsrichtlinien |
| `docs/SECURITY.md` | informativ | Sicherheits-Reporting |
| `docs/CHANGELOG.md` | historisch | Änderungsprotokoll |
| `capabilities.toml` | **normativ (Single Source)** | Ring, Klasse, Reifegrad je Crate |
| `AGENTS.md` | normativ | Agenten-Betriebsanleitung |
| `.agents/jules/SESSION_BOOTSTRAP.md` | normativ für Jules | Jules Session-Ablauf |
| `docs/decisions/` | normativ | ADR-Index |
| `governance/gates.toml` | normativ | Gate-Definitionen |
| `governance/ratchet.toml` | normativ | Ratchet-Baselines |

---

## Veraltete / archivierte Dokumente

> **Diese Dokumente sind OBSOLET und dürfen NICHT mehr als Referenz verwendet werden.**
> Sie wurden durch `docs/SPEC.md` Fassung 2.3 ersetzt.

- `docs/CONTEXTRA_SPEZIFIKATION.md` → ersetzt durch `docs/SPEC.md`
- `docs/GESAMTSPEZIFIKATION.md` → ersetzt durch `docs/SPEC.md`  
- `docs/FEATURES_SPECIFICATION.md` → ersetzt durch `docs/SPEC.md`
- `docs/CONTEXTRA_IMPLEMENTIERUNGSPLAN.md` → ersetzt durch `docs/PLAN.md`
- `docs/CONTEXTRA_META_PLAN.md` → ersetzt durch `docs/PLAN.md`
- `docs/spec/CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md` → ersetzt durch `docs/SPEC.md`
- Root-Berichte (`PLAN.md` im Root, `JULES_PROMPTS.md`, `FINDINGS_REPORT.md` etc.) → archiviert in `archiv/v0`

---

*Gate `doc-truth` prüft: jede normative ID einmal definiert, Dokumente mit Stempel versehen.*
