# ADR-078: Konsolidierung aller NEW_STRATEGY-Dokumente in GESAMTSPEZIFIKATION_v10.0

* **Datum:** 2026-09-08
* **Status:** ✅ Final
* **Kontext:** `docs/NEW_STRATEGY/` enthielt 12 Strategiedokumente (v4.0–v9.0, Governance-Audit, Entscheidungsdokumente, Kritik) mit partiellen Widersprüchen und einer Gesamtgröße von ~6.000 Zeilen. Kein Dokument war alleine normativ.
* **Entscheidung:** Alle Strategiedokumente werden in `docs/GESAMTSPEZIFIKATION_v10.md` synthetisiert. Das neue Dokument ist die einzige normative Wahrheitsquelle. `docs/NEW_STRATEGY/` verbleibt im Repository als Archiv (read-only, kein Jules-Schreibzugriff), wird aber nicht mehr als Referenz in Prompts verwendet. <!-- doc-ref-ignore -->
* **Konsequenzen:**
  1. Neue Jules-Sessions lesen `AGENTS.md` + `GESAMTSPEZIFIKATION_v10.md` statt `docs/NEW_STRATEGY/*.md`. <!-- doc-ref-ignore -->
  2. `GESAMTSPEZIFIKATION_v10.md` wird bei Änderungen an Crate-Topologie oder Feature-Status per `cargo xtask sync-docs` aktualisiert (nicht manuell). <!-- doc-ref-ignore -->
  3. `docs/NEW_STRATEGY/` bekommt eine `README.md`: "ARCHIV — nicht für neue Sessions verwenden. Normative Spezifikation: docs/GESAMTSPEZIFIKATION_v10.md" <!-- doc-ref-ignore -->
* **Alternativen verworfen:** Einzelne Dokumente updaten statt neu synthetisieren (führt zum selben Versions-Drift-Problem, das zur Notwendigkeit dieser ADR geführt hat).

---
