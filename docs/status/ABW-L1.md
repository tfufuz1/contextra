# Abweichungsbericht Lane L1 — Root-Dokumente

**Stand:** 2026-10-25
**Bezugs-Spezifikation:** CONTEXTRA Normative Gesamtspezifikation Fassung 1.2 (final)
**Verifizierte Grundlagen:** `capabilities.toml` (Schema 2, Spec v4), Spezifikation 1.2 (§0–§34, Anhang A).

---

## Übersicht der behobenen Abweichungen je Spezifikationsabschnitt

### 1. Spezifikationsbezug und Dokumentenstand (§0, §1)
- **Ist-Zustand (vorbehalten):** `README.md`, `ARCHITECTURE.md` und `SECURITY.md` verwiesen auf „Normative Gesamtspezifikation v1.1“ bzw. „Systemspezifikation v15“.
- **Abweichung:** Veraltete Dokumentenversionierung und falsche Spezifikationsreferenz.
- **Soll-Zustand (§0):** Alle Root-Dokumente verweisen einheitlich auf Spezifikation Fassung 1.2 als einzige Quelle der Soll-Architektur.
- **Status:** Behoben.

### 2. Produktmissionen und M1-Aussagen (§2, §13.1)
- **Ist-Zustand:** `README.md` behauptete M1 (persistenter Prefix-Cache) als aktive/stable Eigenschaft.
- **Abweichung:** Die Bedingungen B1–B3 (§13.1) für die M1-Behauptung (Löschkette, external_kv Generator, Break-even-Gate) sind im Ist-Zustand nicht vollständig maschinell belegt.
- **Soll-Zustand (§2, §13.1, REQ-KVC-001):** M1 wird nicht als Ist-Zustand behauptet, sondern als Rand-Funktion `[R]` (experimental) klassifiziert, solange B1–B3 nicht erbracht sind.
- **Status:** Behoben.

### 3. Invarianten, Vetos & Löschaussagen (§4.3, §20.12, INV-CLAIMS-1)
- **Ist-Zustand:** `README.md`, `CONTRIBUTING.md` und `SECURITY.md` enthielten nicht-konforme Phrasen zur Löschgarantie und rechtlichen Konformität.
- **Abweichung:** Verstoß gegen INV-CLAIMS-1 (§4.3) und Terminology-Vorgaben nach §20.12.
- **Soll-Zustand:** Aussagen zur Löschung nutzen ausschließlich die verbindliche Außenformulierung nach §4.3 und Claim-Taxonomie (§20.12). Keine Pauschalversprechen.
- **Status:** Behoben.

### 4. Ring-Modell, Crate-Inventar, Kantenregeln & Rand-Brücken (§6.0, §6.7, §6.8, §6.10, §6.11)
- **Ist-Zustand:** `ARCHITECTURE.md` hatte teilweise abweichende Crate-Klassifizierungen und veraltete Spezifikationsverweise.
- **Abweichung:** Abweichung von `capabilities.toml` (INV-DOC-SINGLE-SOURCE-1) und fehlende explizite Darstellung der Kantenregeln (§6.10), Rand-Brücken (§6.11), Unsafe-Inseln (§6.7) sowie Kern-Geschlossenheit (§6.8).
- **Soll-Zustand:** Crate-Tabelle entspricht 1:1 `capabilities.toml`. Architektur beschreibt Kantenregeln, Rand-Brücken und genau 3 Unsafe-Inseln (`simd`, `sys`, `wire`).
- **Status:** Behoben.

### 5. Changelog-Dokumentation (§28.5)
- **Ist-Zustand:** `docs/CHANGELOG.md` enthielt keine Aktualisierung bezüglich Spezifikation 1.2.
- **Abweichung:** Fehlende Dokumentation des Spezifikations-Upgrades im Tag-/Review-Bericht.
- **Soll-Zustand:** Eintrag „Spezifikation 1.2 übernommen“ hinzugefügt, ohne historische Einträge zu verfälschen.
- **Status:** Behoben.

### 6. Governance und Sicherheitsmodell (§28, §30)
- **Ist-Zustand:** `CONTRIBUTING.md` und `SECURITY.md` verwiesen nicht explizit auf §28 und §30 der Spezifikation 1.2.
- **Abweichung:** Fehlende normative Querverweise.
- **Soll-Zustand:** `CONTRIBUTING.md` verweist auf Governance (§28), `SECURITY.md` auf das Sicherheitsmodell (§30).
- **Status:** Behoben.

### 7. Quellen-Rangfolge und Gate-Integrität in `AGENTS.md` (§0, §1, §4.1, §5, REQ-QA-010)
- **Ist-Zustand:** `AGENTS.md` definierte keine eindeutige Rangfolge der Informationsquellen.
- **Abweichung:** Verstoß gegen das Gebot der klaren Hierarchie und Missbrauchsgefahr bei Gate-Ausnahmen.
- **Soll-Zustand:** `AGENTS.md` legt fest: Spezifikation 1.2 (§0, §1) > Vetos & Prinzipien (§4.1, §5) > `capabilities.toml` > `AGENTS.md` > Code & grüne Gates. Gates dürfen nicht geschwächt werden (REQ-QA-010).
- **Status:** Behoben.
