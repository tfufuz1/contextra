# Contextra — Quellen der Wahrheit (Single Source of Truth)

<!-- doc-stamp: 2026-10-31 | Spec-Fassung: 2.4 | PLAN: 31.10.2026 -->

> **Normative Grundregel (INV-DOC-SINGLE-SOURCE-1):** Jede Tatsache hat genau eine kanonische Quelle.
> Abgeleitetes wird generiert oder per Doc-Truth-Gate geprüft. Doppelt vergebene IDs oder widersprüchliche Dokumente sind Gate-Fehler.

---

## 1. Rangfolge der Informationsquellen

Bei inhaltlichen Konflikten oder Widersprüchen gilt zwingend folgende Rangfolge:

| Rang | Quelle | Status | Inhalt & Gültigkeit |
|:---:|:---|:---:|:---|
| **1** | **[`docs/SPEC.md`](SPEC.md)** | **normativ** | **Fassung 2.4 (Zielzustand v1.0).** Einzige normative Quelle für Architektur, Verhalten, Konstanten, Schnittstellen, Testvorgaben, Gates und Invarianten. |
| **2** | **[`docs/VETOES.md`](VETOES.md)** | **normativ** | Veto-Register: Architekturentscheidungen und harte Verbote (VETO-F02, VETO-F10, VETO-OP03, Single-Node). |
| **3** | **[`capabilities.toml`](../capabilities.toml)** | **normativ** | Single Source of Truth für Ring, Schicht, Klasse (`[K]`, `[R]`, `[E]`), Reifegrad und `may_depend_on` je Crate. |
| **4** | **[`PLAN.md`](../PLAN.md)** | **arbeitsleitend** | Gesamtprüfung des Repositorys, IST-Befunde (B-01..B-12, S-01..S-19), Stufenplan (Stufe 0–3) und Produktzerlegung (EP1–EP7). |
| **5** | **[`docs/CONSTITUTION.md`](CONSTITUTION.md)** | **normativ** | Dauerhafte Governance-, Qualitäts- und Sicherheitsprinzipien (Zero Panic, P28 Determinismus, WAL First, Fail-Closed). |
| **6** | **[`governance/gates.toml`](../governance/gates.toml)** | **normativ** | Definition aller automatisierten Quality Gates und Schwellwerte. |
| **7** | **[`AGENTS.md`](../crates/contextra/AGENTS.md)** | **operativ** | Arbeitsanweisungen und Invarianten für LLM-Agenten und Entwickler. |
| **8** | **Produktionscode + grüne Gates** | **faktisch** | Maschinenlesbarer Beleg realer Ausführung (`cargo check`, `cargo test`). |

---

## 2. Dokument-Karte

| Dokument | Status | Zweck & Inhalt |
|---|---|---|
| [`docs/SPEC.md`](SPEC.md) | **normativ** | Gesamtspezifikation v1.0 (Fassung 2.4, optimierter Löschpfad, 15 Kern-Crates). |
| [`PLAN.md`](../PLAN.md) | **arbeitsleitend** | Statusbericht, Fehlerkatalog (B-01..B-12, S-01..S-19), Stufenplan 0–3. |
| [`README.md`](../README.md) | orientierend | Schnelleinstieg, Produktübersicht, Kernregeln, Schnellstart. |
| [`docs/ARCHITECTURE.md`](ARCHITECTURE.md) | beschreibend | Kompakte Darstellung der Schichtenarchitektur (Ringe 0–4), DAG und Produktfamilie. |
| [`docs/COMPILATION.md`](COMPILATION.md) | instruktiv | Schritt-für-Schritt-Kompilieranleitung und Behebungsleitfaden für Implementierer. |
| [`docs/SECURITY.md`](SECURITY.md) | normativ | Bedrohungsmodell, Vertrauensgrenzen, Sicherheitsanalyse (S-01 bis S-19). |
| [`docs/DELETION-DSGVO.md`](DELETION-DSGVO.md) | analytisch | Synthese zu Art. 17 DSGVO, Crypto-Shredding, Arbeitsspeicher-Purge, Nachweisbarkeit. |
| [`docs/CONSTITUTION.md`](CONSTITUTION.md) | normativ | Verfassung: Safety First, Zero Panic, WAL First, P28 Determinismus. |
| [`docs/CONTRIBUTING.md`](CONTRIBUTING.md) | informativ | Beitragsrichtlinien, PR-Workflow, Commit-Konventionen. |
| [`docs/TYPE_REGISTRY.md`](TYPE_REGISTRY.md) | normativ | Inventar kanonischer Typen und Deduplizierungs-Fahrplan (Regel 11). |
| [`docs/VETOES.md`](VETOES.md) | normativ | Verbindliches Register aller Feature-Vetos. |
| [`capabilities.toml`](../capabilities.toml) | **normativ** | Crate-Klassifikation und erlaubte Kanten (`may_depend_on`). |
| [`governance/README.md`](../governance/README.md) | beschreibend | Dokumentation der Gate-Struktur und Baselines. |

---

## 3. Prinzip der Evidenzstufen (INV-EVIDENCE-1)

Aussagen in Dokumenten, Kommentaren oder PRs sind solange reine Absichtserklärungen (**E0**), bis sie maschinell belegt sind:

- **E0 (Behauptung):** Idee, Plan oder unüberprüfte Dokumentation.
- **E1 (Quelltext-Beleg):** Durch Inspektion des Codes an Zeilennummern nachgewiesen.
- **E2 (Gate-Lauf):** Erfolgreicher lokaler Gate- oder Skriptlauf mit archiviertem Protokoll.
- **E3 (CI-Reproduzierbar):** Grüner Lauf in isolierter Umgebung auf unabhängigem Runner.
- **E4 (Produktionsreife):** E3 plus unabhängiges Test-Orakel, Mutationsprüfung (`cargo mutants`) und formale Prüfung.
