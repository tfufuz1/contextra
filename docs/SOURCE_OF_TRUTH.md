# Contextra — Quellen der Wahrheit (Single Source of Truth)

<!-- doc-stamp: 2026-11-01 | Spec-Fassung: 2.0.0 (SOLL-Zustand Release 1.0.0) | PLAN: 2026-11-01 -->

> **Normative Grundregel (INV-DOC-SINGLE-SOURCE-1):** Jede Tatsache hat genau eine kanonische Quelle.
> Abgeleitetes wird generiert oder per Doc-Truth-Gate geprüft. Doppelt vergebene IDs oder widersprüchliche Dokumente sind Gate-Fehler.
> Jede Abweichung des Quellcodes oder anderer Dokumente von `docs/SPEC.md` ist ein Defekt.

---

## 1. Rangfolge der Informationsquellen

Bei inhaltlichen Konflikten oder Widersprüchen gilt zwingend folgende Rangfolge:

| Rang | Quelle | Status | Inhalt & Gültigkeit |
|:---:|:---|:---:|:---|
| **1** | **[`docs/SPEC.md`](SPEC.md)** | **normativ** | **Fassung 2.0.0 (SOLL-Zustand für Release 1.0.0).** Einzige normative Quelle für Architektur, Verhalten, Konstanten, Schnittstellen, Testvorgaben, Gates und Invarianten. |
| **2** | **[`PLAN.md`](../PLAN.md)** | **arbeitsleitend** | Begleitdokument zur Spezifikation: Phasen 0–7, Gates G-A bis G-H, G0, verbindliche Arbeitspakete (WP-0.1 bis WP-7.4). |
| **3** | **[`docs/VETOES.md`](VETOES.md)** | **normativ** | Veto-Register: Architekturentscheidungen und harte Verbote (VETO-F02, VETO-F10, VETO-SN01, ADR-001 Vetos). |
| **4** | **[`AGENTS.md`](../AGENTS.md)** | **normativ** | Verbindliche Agenten- und Entwickler-Betriebsanleitung für den gesamten Workspace gemäß ADR-001. |
| **5** | **[`docs/CONSTITUTION.md`](CONSTITUTION.md)** | **normativ** | Dauerhafte Governance-, Qualitäts- und Sicherheitsprinzipien (Zero Panic, P28 Determinismus, WAL First, Fail-Closed, 2 Unsafe-Inseln). |
| **6** | **Produktionscode + grüne Gates** | **faktisch** | Maschinenlesbarer Beleg realer Ausführung (`cargo check`, `cargo test`, Gate-Evidenz unter `evidence/<WP-ID>/`). |

---

## 2. Dokument-Karte

| Dokument | Status | Zweck & Inhalt |
|---|---|---|
| [`docs/SPEC.md`](SPEC.md) | **normativ** | Gesamtspezifikation v2.0.0 (SOLL-Zustand Release 1.0.0, 14 Kern-Crates, Löschpipeline, E1/E2/E3 Messungen, Bericht `ctx-erasure-report/1`). |
| [`PLAN.md`](../PLAN.md) | **arbeitsleitend** | Phasenplan 0–7, Gates G-A bis G-H, Arbeitspakete WP-0.1 bis WP-7.4. |
| [`AGENTS.md`](../AGENTS.md) | **normativ** | Zentrale Arbeitsrichtlinie für Entwickler und Agenten im Workspace-Root. |
| [`docs/ARCHITECTURE.md`](ARCHITECTURE.md) | beschreibend | Darstellung der Schichtenarchitektur (Ringe 0–4), DAG, Abhängigkeitsmatrix und Datenflüsse. |
| [`docs/COMPILATION.md`](COMPILATION.md) | instruktiv | Kompilieranleitung und Workspace-Mitglieder nach SPEC §5.1. |
| [`docs/SECURITY.md`](SECURITY.md) | normativ | Bedrohungsmodell, Vertrauensgrenzen, Pflicht-Verschlüsselung, KDF-Vorgaben (Argon2id $\ge 64\,\text{MiB}$). |
| [`docs/DELETION-DSGVO.md`](DELETION-DSGVO.md) | analytisch | Rechts- und Architektursynthese zu Art. 17/18 DSGVO, Löschbegriff, Vorbehalt und ADR-001 Einordnung. |
| [`docs/CONSTITUTION.md`](CONSTITUTION.md) | normativ | Verfassung: Safety First, Zero Panic, WAL First, P28 Determinismus, 2 Unsafe-Inseln. |
| [`docs/CONTRIBUTING.md`](CONTRIBUTING.md) | informativ | Beitragsrichtlinien, TDD-Workflow, Commit-Konventionen (ein WP = ein PR). |
| [`docs/TYPE_REGISTRY.md`](TYPE_REGISTRY.md) | normativ | Inventar kanonischer Typen nach SPEC §8.1 / §8.6. |
| [`docs/VETOES.md`](VETOES.md) | normativ | Verbindliches Register aller Feature-Vetos. |

---

## 3. Prinzip der Evidenzstufen (INV-EVIDENCE-1)

Aussagen in Dokumenten, Kommentaren oder PRs sind solange reine Absichtserklärungen (**E0**), bis sie maschinell belegt sind:

- **E0 (Behauptung):** Idee, Plan oder unüberprüfte Dokumentation.
- **E1 (Quelltext-Beleg):** Durch Inspektion des Codes an Zeilennummern nachgewiesen.
- **E2 (Gate-Lauf):** Erfolgreicher lokaler Gate- oder Skriptlauf mit archiviertem Protokoll (`evidence/<WP-ID>/`).
- **E3 (CI-Reproduzierbar):** Grüner Lauf in isolierter Umgebung auf unabhängigem Runner.
- **E4 (Produktionsreife):** E3 plus unabhängiges Test-Orakel, Mutationsprüfung (`cargo mutants`) und Nachweis im Erasure Lab.
