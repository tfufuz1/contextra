# Contextra — Dokumentationsinventur, Spezifikationsanalyse und Milestone-Vorschlag

**Stand:** Oktober 2026
**Autor:** Documentation Architect & Release Manager
**Dokument-ID:** `docs/reports/docs-inventory-2026-10.md`
**Bezugs-Spezifikation:** `AGENTS.md` §1, `docs/spec/CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md` (v15)

---

## Executive Summary

Das Verzeichnis `docs/` umfasst aktuell **547 Dateien** mit einer Gesamtgröße von ca. **6,6 MB**. Eine strukturierte Analyse zeigt, dass wesentliche Teile des Bestands historische Entwicklungsstände, doppelte Spezifikationsfassungen und projektinterne Arbeitsberichte darstellen. Mehrere Qualitäts-Gates in `xtask/` sowie GitHub Workflows sind direkt an bestimmte Pfade in `docs/` gekoppelt.

Dieses Dokument liefert:
1. Eine lückenlose Inventur aller 19 Unterverzeichnisse und 18 Root-Dateien in `docs/` mit Risiko- und Nutzenbewertung.
2. Eine Analyse der Spezifikationslandschaft (`docs/spec/`) zur Feststellung der kanonischen Fassung.
3. Einen schrittweisen, risikominimierten Archivierungsplan in maximal 6 Schritten.
4. Einen abgeleiteten Vorschlag für 7 GitHub-Milestones basierend auf den Spezifikationskapiteln §17, §18 und §20.
5. Eine Quantifizierung der Session-Kontextkosten für autonome Agenten (Jules).

---

## 1. Inventar-Tabelle

### 1.1 Unterverzeichnisse von `docs/`

| Pfad | Dateien | Größe | Zweck | Eingehende Referenzen (Repo) | Gate / Generator Read/Write (Beleg) | Empfehlung | Risiko & Begründung |
|---|---|---|---|---|---|---|---|
| `docs/analysis` | 23 | 338,6 KB | Tiefenanalysen und Kampagnenberichte (J-01..J-42, MVCC/Krypto). | 0 (außerhalb `docs/`) | Keine direkten xtask/CI Schreib-Lese-Zugriffe. | behalten | **Niedrig**: Reines Analysedokumentationsarchiv, keine Gate-Abhängigkeiten. |
| `docs/architecture` | 2 | 16,0 KB | Ergänzende Architekturdiagramme und Systembeschreibungen. | 0 (außerhalb `docs/`) | Keine. | zusammenführen | **Niedrig**: Kann ohne Risiko in `docs/ARCHITECTURE.md` konsolidiert werden. |
| `docs/archive` | 163 | 2044,7 KB | Historische Spezifikations- und Planungstände (v1–v4 Master, Entwürfe). | 7 | `xtask/src/gates/check_crate_references.rs:92,102,169` (Ausnahmefilter). | behalten | **Niedrig**: Wird von `check-crate-references` als Archiv ignoriert. |
| `docs/audit` | 1 | 8,1 KB | B21 Löschnachweis-Auditprotokoll. | 21 | `xtask/src/gates/check_crate_references.rs:103` (Ausnahmefilter). | zusammenführen | **Niedrig**: Zusammenführung mit `docs/audits/` bereinigt Namensduplikat. |
| `docs/audits` | 59 | 468,1 KB | Formale Auditberichte (A-01..A-59, Crate-Tiefenaudits). | 21 | `xtask/src/check_audit_duplication.rs:142`, `xtask/src/check_audit_verdict_independence.rs:222`. | behalten | **Mittel**: Wird von `check-audit-duplication` und `check-audit-verdict-independence` gescannt. |
| `docs/ci` | 4 | 17,9 KB | CI/CD Checklisten und Branch-Protection-Protokolle. | 2 | `xtask/src/check_toc_integrity.rs:13` (`docs/ci/BRANCH_PROTECTION_CHECKLIST.md`). | behalten | **Mittel**: `check-toc-integrity` verifiziert Verzeichnisstruktur. |
| `docs/compliance` | 2 | 6,2 KB | DSGVO- und AVV-Compliance-Erläuterungen. | 0 | Keine. | behalten | **Niedrig**: Dokumentatives Nachweisarchiv. |
| `docs/decisions` | 127 | 303,4 KB | Architecture Decision Records (ADR-E01..ADR-108). | 88 | `xtask/src/check_adr_deadlines.rs:1`, `xtask/src/generate_adr.rs:1`, `xtask/src/harness/protected_paths.rs:1`, `xtask/src/harness/gate_weakening.rs:1`, `xtask/src/harness/ratchet.rs:1`, `xtask/src/main.rs:1215`. | behalten | **Hoch**: Protected Path, wird von zahlreichen Governance-Gates und Ratchets strikt verifiziert. |
| `docs/deletion` | 6 | 32,1 KB | Spezifikationen und Nachweise für DSGVO-Löschgarantien. | 12 | `xtask/src/harness/keyspace_registry.rs:7`, `xtask/src/harness/doc_truth_deletion.rs:9`. | behalten | **Hoch**: Von `doc_truth_deletion` und `keyspace_registry` verifiziert. |
| `docs/explain` | 32 | 16,0 KB | Konzept- und Prompter-Erklärungen für KI-Agenten. | 6 | `xtask/src/harness/explain.rs:36` (Harness-Prüfung). | behalten | **Mittel**: Harness prüft Existenz und Struktur. |
| `docs/gates` | 44 | 67.3 KB | Einzelbeschreibungen der Qualitäts- und Architektur-Gates. | 0 | Keine direkten xtask-Scans. | zusammenführen | **Niedrig**: Kann in `xtask/registry.toml` und `docs/ARCHITECTURE.md` integriert werden. |
| `docs/generated` | 2 | 13,5 KB | Automatisch generierte Berichte und Artefakte. | 8 | `xtask/tests/harness_diff_budget_test.rs:105`. | behalten | **Niedrig**: Generiertes Testartefakt. |
| `docs/postmortems` | 2 | 3,3 KB | Vorfalls-Postmortems (PM-0001 und Vorlage). | 13 | `xtask/tests/harness_postmortem_test.rs:41`, `xtask/src/harness/postmortem.rs:1`. | behalten | **Mittel**: Postmortem Harness prüft Formate. |
| `docs/refactor` | 2 | 12,0 KB | Historische Refactoring-Protokolle. | 7 | Keine. | archivieren | **Niedrig**: Reines Übergangsprotokoll, nach `docs/archive/` verschiebbar. |
| `docs/reports` | 5 | 44,5 KB | Status- und Inventarberichte für Management/Release. | 5 | Keine. | behalten | **Niedrig**: Zielort für diesen Inventarbericht. |
| `docs/restore` | 5 | 350,1 KB | Triage-Restaurierungsberichte (T1..T5). | 0 | Keine. | archivieren | **Niedrig**: Abgeschlossene Triage-Berichte, archivierbar nach `docs/archive/restore/`. |
| `docs/spec` | 37 | 1049,5 KB | Normative Spezifikation und Kapiteldateien 00–92. | 18 | `xtask/src/check_toc_integrity.rs:12`, `xtask/src/check_doc_references.rs:13`, `xtask/src/gate_check.rs:93`. | behalten | **Hoch**: Kernspezifikation, verifiziert von `check-doc-references` und `check-toc-integrity`. |
| `docs/spec-sync` | 9 | 9,9 KB | Hilfsskripte und Diff-Dateien für den Spezifikationsabgleich. | 0 | Keine. | archivieren | **Niedrig**: Einmalige Synchronisationswerkzeuge. |
| `docs/verification` | 4 | 37,1 KB | Invarianten-Matrix und Verifier-Spezifikation (`VERIFIER_SPEC.md`). | 3 | Keine direkten Code-Scans. | behalten | **Mittel**: Formale Verifikations-Grundlage. |

### 1.2 Dateien im Root-Verzeichnis von `docs/`

| Datei | Größe | Zweck | Eingehende Referenzen (Repo) | Gate / Generator Read/Write (Beleg) | Empfehlung | Risiko & Begründung |
|---|---|---|---|---|---|---|
| `docs/ARCHITECTURE.md` | 18,1 KB | Normative Systemarchitektur, Ring-Modell und Invarianten. | 12 | `xtask/src/main.rs:1215`, `xtask/src/sync_docs.rs:1` (Lese-/Schreibzugriff für `INVARIANTS_TABLE` und `DAG_TOPOLOGY`). | behalten | **Hoch**: Wird von `cargo xtask sync-docs` automatisch synchronisiert und gelesen. |
| `docs/BENCH-LOG.md` | 13,3 KB | Ausführungsprotokolle historischer Benchmark-Läufe. | 2 | Keine. | zusammenführen | **Niedrig**: Kann mit `BENCHMARKS.md` zusammengeführt werden. |
| `docs/BENCHMARKS.md` | 19,1 KB | Hauptdokument für Benchmark-Methodik und Ergebnisse. | 5 | Keine. | behalten | **Niedrig**: Dokumentation der Benchmark-Ergebnisse. |
| `docs/BENCHMARKS_DELETION_PROOF.md` | 4,4 KB | Spezial-Benchmark für Löschnachweis-Performance. | 2 | Keine. | zusammenführen | **Niedrig**: In `BENCHMARKS.md` integrierbar. |
| `docs/BENCHMARKS_ROUTING_COST.md` | 3,3 KB | Spezial-Benchmark für Router-Latenz und -Kosten. | 0 | Keine. | zusammenführen | **Niedrig**: In `BENCHMARKS.md` integrierbar. |
| `docs/BENCHMARK_PROTOCOL.md` | 7,9 KB | Protokollvorgaben für faire Benchmark-Messungen. | 0 | Keine. | zusammenführen | **Niedrig**: In `BENCHMARKS.md` integrierbar. |
| `docs/CHANGELOG.md` | 107,0 KB | Release-Änderungsprotokoll über alle Versionen. | 7 | `xtask/src/main.rs:1205` (`sync-docs` prüft Sync-Status). | behalten | **Hoch**: `sync-docs` bricht ab, wenn CHANGELOG out-of-sync ist. |
| `docs/CONTEXTRA_SOTA_FORSCHUNGSBERICHT_2025_2026.md` | 36,4 KB | Algorithmentheoretischer Forschungsbericht. | 0 | Keine. | archivieren | **Niedrig**: Hintergrundforschung, archivierbar nach `docs/archive/misc/`. |
| `docs/EXPORT_FORMAT.md` | 3,5 KB | Formatbeschreibung für Compliance-Exporte. | 0 | Keine. | behalten | **Niedrig**: Technische Format-Spezifikation. |
| `docs/FEATURES_SPECIFICATION.md` | 18,0 KB | Crate-übergreifendes Feature-Inventar. | 0 | Keine. | zusammenführen | **Niedrig**: In Spezifikation Teil K integrierbar. |
| `docs/GESAMTSPEZIFIKATION.md` | 21,2 KB | Historische Gesamtspezifikation v2/v4. | 3 | `xtask/src/gates/check_crate_references.rs:104` (Ausnahme-Filter). | archivieren | **Niedrig**: Veraltet, durch v15 ersetzt. |
| `docs/QUALITAETSPRUEFUNG_COMMIT_FUNKTIONEN_SPEZIFIKATION.md` | 27,3 KB | Testspezifikation für Commit-Qualität. | 0 | Keine. | archivieren | **Niedrig**: Historische Prüfungsspezifikation. |
| `docs/SOURCE_OF_TRUTH.md` | 4,2 KB | Crate-Inventar & Crate-Ringe Single Source of Truth. | 4 | `xtask/src/main.rs:1225`, `xtask/src/sync_docs.rs:1` (Schreib- und Prüfzugriff). | behalten | **Hoch**: `cargo xtask sync-docs` verifiziert und schreibt `CRATE_INVENTORY`. |
| `docs/TYPE_REGISTRY.md` | 39,7 KB | Typenregister aller Ring-0/Ring-1 Datenstrukturen. | 3 | `xtask/src/check_type_registry.rs:1` (Prüfung via xtask). | behalten | **Mittel**: Wird von `check-type-registry` genutzt. |
| `docs/bsi-grundschutz-mapping.md` | 2,2 KB | Mapping auf BSI IT-Grundschutz Bausteine. | 1 | Keine. | behalten | **Niedrig**: Compliance-Nachweis. |
| `docs/mutation_score_history.jsonl` | 0,0 KB | Historienprotokoll der Mutation-Testing-Scores. | 4 | `xtask/src/check_mutation_score_gate.rs:55`, `.github/workflows/mutation-testing.yml:45`. | behalten | **Mittel**: Wird von CI und `check-mutation-score-gate` fortgeschrieben. |
| `docs/supply-chain-trust.md` | 3,9 KB | Vertrauensmodell für externe Abhängigkeiten. | 2 | Keine. | behalten | **Niedrig**: Sicherheitsdokumentation. |
| `docs/unwrap_baseline_history.jsonl` | 1,3 KB | Historienprotokoll des unwrap-Ratchet Baselines. | 2 | `.github/workflows/unwrap-baseline-check.yml:33`, `.github/workflows/post-merge-verification.yml:42`. | behalten | **Hoch**: Wird von GitHub Workflows gelesen und fortgeschrieben. |

---

## 2. Spezifikationsanalyse (`docs/spec/`)

### 2.1 Kanonische Fassung der Hauptspezifikation

In `AGENTS.md` §1 sowie `docs/spec/INDEX.md` werden zwei Hauptdateien aufgeführt:
1. `docs/spec/CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md` (364,9 KB)
2. `docs/spec/CONTEXTRA_SPEZIFIKATION_v15.md` (364,9 KB)

**Befund:**
Ein kryptografischer Abgleich zeigt, dass beide Dateien **100 % byte-identische Duplikate** sind:
- **SHA256:** `d2d7ef6f8cc95b79d3b4ab48e32fa138ac685990d1c315e5314bdec70b71bb51` (beide Dateien)
- **Stand:** 29.09.2026 / 30.09.2026 (Systemspezifikation v15)

**Bewertung & Empfehlung:**
`CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md` ist die **alleinige kanonische Hauptspezifikation**.
`CONTEXTRA_SPEZIFIKATION_v15.md` stellt eine redundante Kopie dar. Um Verwirrung und doppelte Maintenance zu vermeiden, sollte `CONTEXTRA_SPEZIFIKATION_v15.md` gelöscht oder als symbolischer Link auf `CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md` geführt werden, verbunden mit der Klarstellung in `AGENTS.md` §1.

### 2.2 Bewertung der Kapiteldateien (`00`–`92`)

In `docs/spec/` existieren neben den Hauptdateien 35 nummerierte Einzelkapitel (`00-meta-workspace-layout.md` bis `92-anhang-d-aenderungsprotokoll-v4.md`, ca. 312 KB gesamt).

**Befund:**
1. **Ursprung:** Alle Kapiteldateien stammen aus der früheren Entwurfsfassung `CONTEXTRA_SPEC_v4_MASTER.md`.
2. **Normativitätsrang:** In `docs/spec/INDEX.md` ist explizit normativ geregelt:
   > *„Hinweis zur Normativität: Die unten aufgeführten nummerierten Kapiteldateien (`00a`…`21`…`92`) sind historische Vorstufen. Bei etwaigen Widersprüchen gilt ausschließlich `CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md` als normative Quelle der Wahrheit.“*
3. **Status:** Die relevanten Inhalte aller Kapitel (inklusive der SOTA-Erweiterungen §21) wurden vollständig in `CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md` integriert.

**Bewertung & Empfehlung:**
Die Kapiteldateien 00–92 sind historischer Natur. Zur Reduzierung der Suchkomplexität und zur Beseitigung verwaister Pfadreferenzen (siehe `cargo xtask check-doc-references`) sollten sie in ein Unterverzeichnis `docs/archive/spec/` verschoben werden.

---

## 3. Archivierungs-Reihenfolge (Execution Plan)

Um Unterbrechungen in CI-Pipelines und Qualitäts-Gates zu vermeiden, wird die Archivierung in **6 kleine, einzeln mergbare Schritte** unterteilt. Jeder Schritt nennt die betroffenen Dateien und die zwingend auszuführenden Gates.

```
+-----------------------------------------------------------------------------------+
| Schritt 1: Konsolidierung der Benchmark-Dokumente im Root                         |
|   Dateien: BENCH-LOG.md, BENCHMARKS_*.md, BENCHMARK_PROTOCOL.md -> BENCHMARKS.md    |
|   Gates: cargo xtask check-doc-references                                         |
+-----------------------------------------------------------------------------------+
                                         |
                                         v
+-----------------------------------------------------------------------------------+
| Schritt 2: Archivierung veralteter Root-Spezifikationen                           |
|   Dateien: GESAMTSPEZIFIKATION.md, SOTA_FORSCHUNGSBERICHT.md -> docs/archive/misc/|
|   Gates: cargo xtask check-doc-references, cargo xtask sync-docs --check          |
+-----------------------------------------------------------------------------------+
                                         |
                                         v
+-----------------------------------------------------------------------------------+
| Schritt 3: Bereinigung von Ordnerduplikaten und Triage-Berichten                  |
|   Dateien: docs/audit/ -> docs/audits/, docs/restore/ -> docs/archive/restore/   |
|   Gates: cargo xtask check-audit-duplication, check-audit-verdict-independence    |
+-----------------------------------------------------------------------------------+
                                         |
                                         v
+-----------------------------------------------------------------------------------+
| Schritt 4: Eliminierung der Namensredundanz Spezifikation v15                     |
|   Dateien: CONTEXTRA_SPEZIFIKATION_v15.md (Symlink/Entfernung), AGENTS.md §1      |
|   Gates: cargo xtask doc-truth, cargo xtask check-doc-references                  |
+-----------------------------------------------------------------------------------+
                                         |
                                         v
+-----------------------------------------------------------------------------------+
| Schritt 5: Archivierung historischer Spec-Kapitel 00–92                           |
|   Dateien: docs/spec/00*..92* -> docs/archive/spec/, Update docs/spec/INDEX.md    |
|   Gates: cargo xtask check-toc-integrity, cargo xtask check-doc-references        |
+-----------------------------------------------------------------------------------+
                                         |
                                         v
+-----------------------------------------------------------------------------------+
| Schritt 6: Konsolidierung von Architecture & Refactor-Protokollen                 |
|   Dateien: docs/architecture/ -> ARCHITECTURE.md, docs/refactor/ -> archive/      |
|   Gates: cargo xtask sync-docs --check, cargo xtask gate-check                    |
+-----------------------------------------------------------------------------------+
```

---

## 4. Milestone-Vorschlag

Aus den verbindlichen Roadmaps in `docs/spec/17-optimierungs-roadmap.md` (§17 Opus-Analyse & SOTA-Algorithmen), `docs/spec/18-gesamtroadmap.md` (§18 Gesamtroadmap) und `docs/spec/20-migrationsplan-und-adr.md` (§20 Strangler-Migrationsplan) werden **7 GitHub-Milestones** abgeleitet.

> **Hinweis:** Alle Meilensteine verzichten auf fiktive Kalenderdaten und definieren stattdessen harte, durch automatisierte Befehle prüfbare Abschlusskriterien.

### Milestone 1: MS-01 — Stufe 0: Korrektheit & Betriebssicherheit
- **Ziel:** Beseitigung aller kritischen Panic-, Bounds- und Recovery-Risiken im LSM/WAL- und Bandit-Hot-Path (§17 Stufe 0).
- **Inhalt:** Opus 0.1 (WAL-Replay-Bounds), Opus 0.2 (Bandit-Dimensionsprüfung), Opus 0.3 (Drift-Bandit-Kopplung), Opus 0.4 (Cloud-Egress-Klassifizierung), Opus 0.5 (Intent-Recovery).
- **Abschlusskriterium (Gate):**
  `cargo test -p contextra-store -p contextra-adapt && cargo run -p xtask -- bench-gate --gate bandit-latency`
- **Reihenfolge:** 1 (Höchste Priorität).

### Milestone 2: MS-02 — Phase 0R/1a/1b: Ring-Modell & Strangler-Refactoring
- **Ziel:** Entkopplung der Kern-Crates, Erstellung der `contextra`-Fassade und Zerlegung von `contextra-core` in `types`/`ports`/`mvcc` (§20 Phase 0R, 1a, 1b).
- **Inhalt:** `contextra`-Fassade, Aufwärtskanten auflösen, `contextra-core` Re-Export Deprecation.
- **Abschlusskriterium (Gate):**
  `cargo xtask check-ring-layering && cargo tree -p contextra-db -e normal | grep -E "candle|ollama" | wc -l | grep -q "^0$"`
- **Reihenfolge:** 2.

### Milestone 3: MS-03 — Phase 1c/2: Unsafe-Isolierung & Sync-Kerne
- **Ziel:** Strikte Isolierung von Unsafe-Code auf die 3 freigegebenen Unsafe-Inseln und Trennung von `StorageRead` (sync) und `StorageWrite` (async) (§20 Phase 1c, Phase 2).
- **Inhalt:** `#![forbid(unsafe_code)]` in Ring 0–3, `contextra-sys`/`simd`/`wire` als Inseln, `ComputePool` für Vektor/Text/Graph.
- **Abschlusskriterium (Gate):**
  `cargo xtask check-unsafe-islands --strict && cargo tree -e normal -p contextra-vector -p contextra-text -p contextra-graph | grep -v tokio`
- **Reihenfolge:** 3.

### Milestone 4: MS-04 — Stufe 1: Hot-Path-Performance & Storage-Optimierung
- **Ziel:** Beseitigung wesentlicher Allokations- und Contention-Engpässe in HNSW, SSTable Zero-Copy und Text-Posting-Formaten (§17 Stufe 1, §18 Stufe 1).
- **Inhalt:** Opus 1.1 (HNSW Adjazenzlisten), Opus 1.2 (Backlink O(1)), Opus 1.4 (SSTable Zero-Copy), Opus 1.6 (MemTable Range-Sharding), Opus 1.9 (Text Posting-Format).
- **Abschlusskriterium (Gate):**
  `cargo run -p xtask -- bench-trend --threshold 0.05`
- **Reihenfolge:** 4.

### Milestone 5: MS-05 — Stufe 1½: SOTA-Algorithmen-Integration
- **Ziel:** Integration der vier normierten SOTA-Algorithmen LeanRAG (S.1), TL-HFD (S.2), DiBud (S.3) und Flow-Corrected Thompson Sampling (S.4) (§17 SOTA, §18 Stufe 1½).
- **Inhalt:** `child_edge_ids`-Schema-Erweiterung, LeanRAG Kaskaden-Aggregation, TL-HFD ShadowMode, DiBud-Streaming-Iteratoren, FC-TS Bandit.
- **Abschlusskriterium (Gate):**
  `cargo test -p contextra-cognition -p contextra-graph -p contextra-rank -p contextra-adapt`
- **Reihenfolge:** 5.

### Milestone 6: MS-06 — Phase 3a/3b & Stufe 2: Konsistenz-Spike, Engine-Zerlegung & Speicher-Kompaktierung
- **Ziel:** Implementierung des WAL-Konsistenzmodells (ADR-N06), Zerlegung von `contextra-db` in Subsysteme und speichereffiziente CSR/Graph-Kompaktierung (§20 Phase 3a, 3b, §17 Stufe 2).
- **Inhalt:** Crash-Injektion im `contextra-testkit`-Fault-VFS (≥ 10⁴ Läufe), CSR-Sentinel-Optimierung, Manifest-Batch-Fsync, `contextra-db` Reduktion auf ≤ 20 `pub fn`.
- **Abschlusskriterium (Gate):**
  `cargo test -p contextra-testkit --test store_wal_crash_harness`
- **Reihenfolge:** 6.

### Milestone 7: MS-07 — Stufe 3: Governance, CI-Powerset & Major-Release-Vorbereitung
- **Ziel:** Vollständige CI Feature-Powerset-Abdeckung, Panic-Inventar-Durchsetzung und Vorbereitung der Major-Release-Freigabe (§17 Stufe 3, §18 Stufe 3, 4).
- **Inhalt:** Feature-Powerset CI Matrix, Panic-Inventar Gate, DocId-128 Cutover Vorbereitung.
- **Abschlusskriterium (Gate):**
  `cargo xtask gate-check --all-phases && cargo xtask check-manifest-completeness`
- **Reihenfolge:** 7.

---

## 5. Session-Kontextkosten (Jules Ambient Load List)

Gemäß `.jules/JULES_CONTEXT.md` und `AGENTS.md` liest der autonome Agent Jules zu Beginn jeder Arbeits-Session den fest geschriebenen permanenten Kontext ein. Nachfolgende Tabelle quantifiziert die Zeichen- und Zeilenmenge pro Datei (`wc -l -m`).

### 5.1 Mandatorischer Ambient Kontext (Ladeliste)

| Datei | Pfad | Zeilen (`wc -l`) | Zeichen (`wc -m`) | Bytes (`wc -c`) | Anteil am Basiskontext |
|---|---|---|---|---|---|
| **Process Instruction** | `.jules/JULES_CONTEXT.md` | 93 | 5.284 | 5.225 | 16,7 % |
| **Session Bootstrap** | `.jules/SESSION_BOOTSTRAP.md` | 114 | 4.457 | 4.398 | 14,1 % |
| **Agent Operational Instructions** | `AGENTS.md` | 61 | 4.733 | 4.692 | 15,0 % |
| **Working State & Open Debts** | `WORKING_STATE.md` | 104 | 10.923 | 10.724 | 34,6 % |
| **ADR Index** | `docs/decisions/README.md` | 49 | 3.596 | 3.565 | 11,4 % |
| **Governance Principles** | `CONSTITUTION.md` | 49 | 3.049 | 3.024 | 9,7 % |
| **Feature Veto Register** | `VETOES.md` | 38 | 1.609 | 1.600 | 5,1 % |
| **Preamble** | `.jules/PREAMBLE.md` | 7 | 801 | 786 | 2,5 % |
| **Gesamt (Permanent Ambient Load)** | — | **515** | **31.597** | **31.021** | **100,0 %** |

*(Hinweis: Bei ca. 4 Zeichen pro Token entspricht der dauerhafte Umgebungskontext ca. 7.900 Tokens pro Session).*

### 5.2 Lokale Crate-Kontextkosten (`crates/*/AGENTS.md`)

Sobald ein Agent in einer spezifischen Crate arbeitet, muss gemäß `JULES_CONTEXT.md` zusätzlich die lokale `AGENTS.md` der Crate geladen werden:

- **Anzahl lokaler Crate-AGENTS.md-Dateien:** 35
- **Gesamtzeilenzahl aller 35 Crate-AGENTS.md:** 2.152 Zeilen
- **Gesamtzeichenzahl aller 35 Crate-AGENTS.md:** 106.991 Zeichen
- **Durchschnittliche Zusatzkosten pro Crate:** 61,5 Zeilen / 3.056 Zeichen (ca. 760 Tokens)

---

## Fazit & Nächste Schritte

1. **Freigabe dieses Inventars:** Durch Merge dieses Berichts steht ein belegtes, konsolidiertes Fundament für die anstehende Dokumentationsbereinigung zur Verfügung.
2. **Umsetzung Schritt 1 der Archivierung:** Erstellung eines ersten isolierten PRs zur Konsolidierung der Root-Benchmark-Dokumente.
3. **Einrichtung der GitHub-Milestones MS-01 bis MS-07:** Übernahme der definierten Meilensteine und Prüfbefehle in das Issue-Tracking-System.
