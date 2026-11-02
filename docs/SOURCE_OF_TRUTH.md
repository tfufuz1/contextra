# Contextra — Quellen der Wahrheit (Single Source of Truth)

<!-- doc-stamp: 2026-11-02 | Spec-Fassung: 4.0.0 (SOLL-Zustand Release 1.0.0) -->

> **Normative Grundregel (P1, INV-DOC-SINGLE-SOURCE-1):** Jede Tatsache hat genau eine kanonische Quelle.
> Abgeleitetes wird generiert oder per Doc-Truth-Gate geprüft. Widersprüchliche Dokumente sind Defekte.
> **Ausschließlich `docs/SPEC.md` ist die Quelle der Wahrheit.** Jede Abweichung des Quellcodes oder anderer Dokumente von `docs/SPEC.md` ist ein Defekt.

---

## 1. Rangfolge der Informationsquellen

Bei inhaltlichen Konflikten oder Widersprüchen gilt zwingend folgende Rangfolge:

| Rang | Quelle | Status | Inhalt & Gültigkeit |
|:---:|:---|:---:|:---|
| **1** | **[`docs/SPEC.md`](SPEC.md)** | **normativ** | **Fassung 4.0.0 (SOLL-Zustand für Release 1.0.0).** Einzige normative Quelle für Architektur, Verhalten, Konstanten, Schnittstellen, Testvorgaben, Gates, Invarianten, Byte-Formate (Anhang F) und den Umsetzungsplan (§15.4). Es gibt kein Begleitdokument. |
| **2** | **[`docs/VETOES.md`](VETOES.md)** | **abgeleitet normativ** | Veto-Register: Architekturentscheidungen und harte Verbote (VETO-F02, VETO-F10, VETO-SN01, VETO-D01b drei Säulen). |
| **3** | **[`AGENTS.md`](../AGENTS.md)** | **abgeleitet normativ** | Verbindliche Agenten- und Entwickler-Betriebsanleitung für den gesamten Workspace gemäß SPEC 4.0.0. |
| **4** | **[`docs/CONSTITUTION.md`](CONSTITUTION.md)** | **abgeleitet normativ** | Dauerhafte Governance-, Qualitäts- und Sicherheitsprinzipien (Zero Panic, P28 Determinismus, WAL First, Fail-Closed, 2 Unsafe-Inseln). |
| **5** | **Produktionscode + grüne Gates** | **faktisch** | Maschinenlesbarer Beleg realer Ausführung (`cargo check`, `cargo test`, Gate-Evidenz). |

---

## 2. Dokument-Karte

| Dokument | Status | Zweck & Inhalt |
|---|---|---|
| [`docs/SPEC.md`](SPEC.md) | **alleinige normative Quelle** | Gesamtspezifikation v4.0.0 (SOLL-Zustand Release 1.0.0, 14 Kern-Crates, Löschpipeline in drei Säulen gem. D-01/ADR-002, MK1/MK2/MK3 Messungen, Ed25519-Bericht, Anhang F Formate, §15.4 Umsetzungsplan). |
| [`PLAN.md`](../PLAN.md) | verweist auf SPEC | Verweist verbindlich auf SPEC §15.4 (kein eigenständiges Plandokument). |
| [`AGENTS.md`](../AGENTS.md) | abgeleitet normativ | Zentrale Arbeitsrichtlinie für Entwickler und Agenten im Workspace-Root. |
| [`docs/ARCHITECTURE.md`](ARCHITECTURE.md) | beschreibend | Darstellung der Schichtenarchitektur (Ringe 0–4), DAG, 14 Produktions-Crates, Abhängigkeitsmatrix und Datenflüsse. |
| [`docs/COMPILATION.md`](COMPILATION.md) | instruktiv | Kompilieranleitung und Workspace-Mitglieder nach SPEC §5.1, Toolchain nach D-82. |
| [`docs/SECURITY.md`](SECURITY.md) | normativ | Bedrohungsmodell, Vertrauensgrenzen, Pflicht-Verschlüsselung, Tresor/Schlüsselhierarchie (D-79), Dateischlüssel-Vernichtung (D-85), CRA Art. 14 Meldepflichten und Support-Zeitraum (D-67). |
| [`docs/CONSTITUTION.md`](CONSTITUTION.md) | normativ | Verfassung: Safety First, Zero Panic, WAL First, P28 Determinismus, 2 Unsafe-Inseln, Fail-Closed. |
| [`docs/CONTRIBUTING.md`](CONTRIBUTING.md) | informativ | Beitragsrichtlinien, TDD-Workflow, Commit-Konventionen (ein WP = ein PR), Wortlaut-Lint (T-LINT-1). |
| [`docs/TYPE_REGISTRY.md`](TYPE_REGISTRY.md) | normativ | Inventar kanonischer Typen nach SPEC §8.1 / §9.1 / §9.3. |
| [`docs/VETOES.md`](VETOES.md) | normativ | Verbindliches Register aller Feature-Vetos. |

---

## 3. Prinzip der Evidenzstufen (EV0–EV4) und Messklassen (MK1–MK3)

Aussagen in Dokumenten, Kommentaren oder PRs sind solange reine Absichtserklärungen (**EV0**), bis sie maschinell belegt sind (SPEC §0):

- **EV0 (Behauptung):** Idee, Plan oder unüberprüfte Dokumentation.
- **EV1 (Quelltext-Beleg):** Durch Quelltextnachweis an Zeilennummern nachgewiesen.
- **EV2 (Gate-Lauf):** Erfolgreicher lokaler Gate- oder Skriptlauf mit archiviertem Protokoll.
- **EV3 (CI-Reproduzierbar):** Reproduzierbar in CI auf unabhängigem Runner.
- **EV4 (Produktionsreife):** EV3 plus unabhängiges Test-Orakel, Mutationsprüfung und Nachweis im Erasure Lab.

Getrennt davon existieren die **Messklassen der Löschmessung** (SPEC §12.1):
- **MK1 (Struktur):** Strukturelle Prüfung auf Abwesenheit von Ziel-Records in LSM-SSTables, Index-Segmenten und Vektor-Slots.
- **MK2 (Rohscan):** Byteweiser Rohscan über Speicherdateien auf Klartext-Needles.
- **MK3 (Schlüsselprobe):** Verifikation der Schlüsselabwesenheit im Tresor (in Produktion: MK3-Feld; vollständige MK3 im CI-Lab).
