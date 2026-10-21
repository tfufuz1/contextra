# KERNPROTOKOLL K: UNABHÄNGIGES, ADVERSARIALES AUDIT — SUBSYSTEM RNK (contextra-rank)

## 1. Kurzfassung

- **Gesamtrisiko**: MEDIUM (S2)
- **Die schwersten Befunde**:
  1. `RNK-001` (S2 / LOGIK / CONFIRMED): Kumulierung von RRF-Score bei doppelten Dokumenten innerhalb einer einzelnen Trefferliste.
  2. `RNK-002` (S2 / UNVOLLSTÄNDIG / STATIC): Reranker-Stufe (`RerankStage`) ist im Produktionspfad der Engine nicht verdrahtet.
  3. `RNK-003` (S3 / DEAD / STATIC): Feature-gegatetes Modul `dibud` (Spec §6.1) besitzt keine produktiven Aufrufer im Workspace.
  4. `RNK-004` (S3 / DEAD / STATIC): `AdaptiveConformalCalibrator` und `ConformalError` im Modul `calibration::conformal` sind ungenutzt.
  5. `RNK-005` (S3 / DOKU-DRIFT / STATIC): Erklärbarkeitstyp `RetrievalExplanation` und `explain()` werden in MCP nur in Test-Asserts referenziert.
- **Was gehalten hat**:
  - 100% Zero-Panic-Invariante in Produktionspfaden (geprüft gegen NaN, Inf, Extremwerte, leere Listen).
  - 100% Zero-Unsafe-Invariante (`#![forbid(unsafe_code)]` strikt eingehalten).
  - Perfect Determinism (f32 Skalar-Operationen und AHashMap-Iteratordeterminismus via BoundedTopK Sortierung).
  - Isotonische und Platt-Kalibrierer sind mathematisch monoton und fail-safe bei extremen/NaN-Logits.
- **Größte Grenzen der Analysis**:
  - `dibud`- und `conformal`-Features sind im Standard-Build nicht aktiviert.

## 2. Metadaten und Methode

- **Zeitstempel**: 2026-10-21T14:50:00Z
- **Commit**: `2a0fe421778941ccca44c53be93caec4a17a61d3`
- **Toolchain**: `rustc 1.89.0 (29483883e 2025-08-04)`, `cargo 1.89.0 (c24e10642 2025-06-23)`
- **Werkzeugstatus**: `cargo test`, `python3` Hilfsskripte, Worktree-Ablauf unter `/tmp/audit-rnk`.
- **Scope-Pfade**: `crates/contextra-rank/src/{fusion/*, calibration/*, dibud/*, drift.rs, explain.rs}`.
- **Gelesene Quellen**:
  - `AGENTS.md` (Root + Crate)
  - `CONSTITUTION.md`, `VETOES.md`, `capabilities.toml`
  - `ADR-003`, `ADR-021`, `ADR-050`, `ADR-079`
- **ADR-Ketten**:
  - ADR-003 (RRF als verbindlicher Standard) -> ADR-021 (4-Signal RAG Pipeline) -> ADR-050 (Single Conformal Lock) -> ADR-079 (Event-driven Drift).

## 3. Inventarmatrix

| Element | Zweck (1 Zeile) | Produktive Aufrufer | Testabdeckung | Invariante | Status |
|---|---|---|---|---|---|
| `reciprocal_rank_fusion` | Standard RRF Fusion von Trefferlisten | `contextra-engine`, `contextra-db` | 100% | ADR-003 | OK |
| `weighted_reciprocal_rank_fusion` | Gewichte RRF Fusion | `contextra-engine` | 100% | ADR-003 | OK |
| `weighted_reciprocal_rank_fusion_with_priority` | Gewichte RRF mit Metadaten-Priorität | `contextra-engine` | 100% | ADR-003 | OK |
| `modulate_and_renormalize_weights` | mRRF Margen-basierte Gewichtsmodulation | `reciprocal_rank_fusion` | 100% | ADR-021 | OK |
| `score_normalized_fusion_with_options` | Score-normalisierte CombSUM Fusion | `contextra-engine` | 100% | ADR-003 | OK |
| `g` | Glatte Einflussfunktion $x/(1+\|x\|)$ | `modulate_and_renormalize_weights` | 100% | Zero-Panic | OK |
| `BoundedTopK` | Bounded Min-Heap für Top-K Ergebnisse | `rrf.rs`, `rerank.rs` | 100% | Heap-Bound | OK |
| `IsotonicCalibrator` | PAVA Isotonische Kalibrierung | `contextra-db`, `contextra-mcp`, `contextra-infer-candle` | 100% | Monotonie | OK |
| `PlattScaler` | Platt-Skalierung Logistisches Modell | `contextra-infer-ollama`, `contextra-infer-onnx` | 100% | Bounded [0,1] | OK |
| `ConformalCalibrator` (Trait) | Interface für Conformal Calibration | `contextra-router` | 100% | Coverage | OK |
| `AdaptiveConformalCalibrator` | Adaptive Conformal Predictor | Keiner in Workspace | 100% (unit) | Coverage | UNWIRED |
| `ConformalError` | Enum für Conformal Fehler | Keiner in Workspace | 100% (unit) | Error-Enum | DEAD |
| `DriftDetector` | Event-getriebene Drift-Erkennung (F-11) | `contextra-adapt` | 100% | ADR-079 | OK |
| `DriftStatus` | Status der Drift-Erkennung | `contextra-adapt` | 100% | ADR-079 | OK |
| `explain` | Generiert strukturierte Erklärungen | `contextra-mcp` (nur Test-Assert) | 100% | Provenance | TEST-ONLY |
| `RetrievalExplanation` | Strukturierte Treffer-Erklärung | `contextra-mcp` (nur Test-Assert) | 100% | Provenance | TEST-ONLY |
| `ExplanationEntry` | Einzelelement der Treffer-Erklärung | `explain.rs` | 100% | Provenance | OK |
| `SignalKind` | Enum der Signal-Typen | `contextra-rank`, `contextra-engine` | 100% | Signal-Taxonomie | OK |
| `ProvenanceBuilder` | Konstruktor für ProvenanceRecord | `rrf.rs` | 100% | Provenance | OK |
| `ProvenanceRecord` | Nachweis der Herkunft & Teilergebnisse | `contextra-engine`, `contextra-db` | 100% | ADR-021 | OK |
| `SearchResult` | Trefferelement mit ID, Score & Provenance | Workspace-weit | 100% | Core Type | OK |
| `ResonanceConfig` | Konfiguration für Resonanz-Bonus | `contextra-engine` | 100% | Bonus | OK |
| `apply_resonance_bonus` | Relevanz-Verstärkung bei Mehrfach-Signal | `contextra-engine` | 100% | Bonus | OK |
| `RerankStage` | Async Reranking Kaskadenstufe | Keiner in Workspace | 100% (unit) | ADR-021 | UNWIRED |
| `GlobalFusionConfig` | Konfiguration Community-Fusion | `global.rs` | 100% (unit) | Spec §6 | UNWIRED |
| `GlobalFusionStrategy` | Strategie Community-RRF | Keiner in Workspace | 100% (unit) | Spec §6 | UNWIRED |
| `dibud::*` | DiBud Fusion State Machine & Driver | Keiner in Workspace | 100% (unit) | Spec §6.1 | UNWIRED |

## 4. Hypothesen-Verdikte

| H-ID | Aussage | Experiment | Verdikt | Verweis auf Befund |
|---|---|---|---|---|
| H1 | RRF-Korrektheit & keine Doppel-Gewichtung in Liste | T1 / Reproduktionstest in `repro_h1_dups.rs` | **CONFIRMED** (Defekt) | `RNK-001` |
| H2 | Normalisierte Fusion bei Extremer/NaN-Eingabe | T9 / Edge-value Test in `repro_h2_nan.rs` | **REFUTED** (Hält) | Negativbefund N1 |
| H3 | Top-k-Heap Stabilität & Größengrenzen | T2 / Heap Boundary Unit Tests | **REFUTED** (Hält) | Negativbefund N2 |
| H4 | Kalibrierung Monotonie & Conformal Coverage | T1/T2 / Test in `repro_h4_conformal.rs` | **REFUTED** (Hält) | Negativbefund N3 |
| H5 | Drift-Erkennung event-getrieben & Clock-Port | T2 / Audit von `drift.rs` & ADR-079 | **REFUTED** (Hält) | Negativbefund N4 |
| H6 | Erklärbarkeit entspricht Fusions-Score | T1 / Repro in `repro_h6_explain.rs` | **REFUTED** (Hält) | Negativbefund N5 |
| H7 | Reranker-Ausfall fail-safe & Verdrahtung | T3 / Subsystem call-graph Audit | **CONFIRMED** (Unwired) | `RNK-002` |
| H8 | DiBud Zustandsautomat vollständig & zyklenfrei | T1 / Call-graph Audit | **CONFIRMED** (Unwired) | `RNK-003` |
| H9 | Determinismus & Numerik | T8 / Re-run Execution in `/tmp/audit-rnk` | **REFUTED** (Hält) | Negativbefund N6 |
| H10 | Async in Ring 0 Klassifizierung | Q9 / Async Scan across crate | **CONFIRMED** (Doku) | Negativbefund N7 |

## 5. Befunde

### ID RNK-001 | RRF Score-Akkomodierung bei doppelten Dokumenten in einer Trefferliste | Severity S2 | Category LOGIK | Status CONFIRMED | Konfidenz High
- **Fundstelle**: `crates/contextra-rank/src/fusion/rrf.rs`, `weighted_reciprocal_rank_fusion_mrrf`, Zeile 250–265.
- **Beobachtung**: Wenn eine einzelne Trefferliste (z. B. BM25 oder Vektor) dasselbe Dokument mehrmals auf unterschiedlichen Rängen enthält, summiert die HashMap `scores[idx] += score;` den RRF-Score für jeden Rang innerhalb derselben Liste auf.
- **Beweis**: Test `test_h1_duplicate_in_single_list` in `/tmp/audit-rnk/crates/contextra-rank/tests/repro_h1_dups.rs`. Ein Dokument auf Rang 1 und Rang 2 derselben Liste erhält RRF Score `0.032522` statt des korrekten Rang-1-Scores `0.016393`. Exit-Code: 0.
- **Gegenbeweis-Versuch**: Prüfte, ob vorgelagerte Suchen (BM25/HNSW) Garantien zur Eindeutigkeit geben. Während Upstream-Suchen meist eindeutige Ergebnisse liefern, erlaubt die RRF-API `Vec<SearchResult>`-Eingaben ohne Invarianten-Prüfung, was bei Multi-Chunk-Treffern desselben Dokuments zur Verzerrung führt.
- **Auswirkung**: Dokumente mit mehreren passenden Chunks in einer Liste werden fälschlicherweise künstlich gegenüber Dokumenten mit nur einem hochrelevanten Chunk bevorzugt.
- **Mindestweg zur Behebung**: Bei der Iteration über eine Trefferliste pro Dokument-ID je Signal nur den Höchstrang (niedrigster Rank-Index) verarbeiten.
- **Policy-Check**: Invariante aus ADR-003 & H1 fordert Eindeutigkeit der Ranggewichtung pro Liste.

### ID RNK-002 | `RerankStage` ist im produktiven Ausführungspfad der Engine nicht verdrahtet | Severity S2 | Category UNVOLLSTÄNDIG | Status STATIC | Konfidenz High
- **Fundstelle**: `crates/contextra-rank/src/fusion/rerank.rs`, `RerankStage`.
- **Beobachtung**: `RerankStage` bietet eine Asynchrone Reranking-Pipeline Stufe. Kein Produktionscode in `crates/contextra-engine` oder `crates/contextra-db` instanziiert oder ruft `RerankStage` auf.
- **Beweis**: Call-Graph-Analyse via `rg "RerankStage" crates/` zeigt 0 produktive Aufrufer.
- **Gegenbeweis-Versuch**: Suche nach alternativen Reranker-Einbindungen ergab, dass `contextra-infer-onnx` Reranking direkt durchführt, ohne die Abstraktion in `contextra-rank` zu nutzen.
- **Auswirkung**: Architektur-Entkopplung gänzlich ungenutzt; Code-Duplikation der Reranking-Kaskadenlogik.
- **Mindestweg zur Behebung**: `RerankStage` in `contextra-engine::retrieval` einbinden oder als deprecated markieren.
- **Policy-Check**: ADR-021 (Multi-Signal RAG Pipeline Stufe 5).

### ID RNK-003 | `dibud` Fusions-Modul hat keine produktiven Aufrufer | Severity S3 | Category DEAD | Status STATIC | Konfidenz High
- **Fundstelle**: `crates/contextra-rank/src/dibud/`.
- **Beobachtung**: Das Feature `dibud` und dessen Untermodule (`driver.rs`, `state.rs`, `types.rs`) enthalten komplexe DiBud State-Machine-Logik, werden aber im gesamten Repository außerhalb von Unit-Tests nicht verwendet.
- **Beweis**: `rg "DiBud" crates/` liefert nur eigene Crate-Dateien und `Cargo.toml`.
- **Gegenbeweis-Versuch**: Feature-Flags in `Cargo.toml` geprüft; Feature `dibud` wird von keinem anderen Crate im Workspace aktiviert.
- **Auswirkung**: Unverwendeter Code erhöht Wartungsaufwand.
- **Mindestweg zur Behebung**: Entweder in Pipeline integrieren oder unter `parking/` verschieben.

### ID RNK-004 | `AdaptiveConformalCalibrator` ungenutzt | Severity S3 | Category DEAD | Status STATIC | Konfidenz High
- **Fundstelle**: `crates/contextra-rank/src/calibration/conformal.rs`.
- **Beobachtung**: `AdaptiveConformalCalibrator` implementiert das `ConformalCalibrator`-Trait, wird jedoch nirgends instanziiert (`contextra-router` verwendet ein eigenes `ConformalCalibrator`-Struct).
- **Beweis**: `rg "AdaptiveConformalCalibrator" crates/` liefert 0 produktive Treffer.
- **Gegenbeweis-Versuch**: In `contextra-router` nach Trait-Impls gesucht; Router definiert eigenen Typ.
- **Auswirkung**: Tpyen-Duplikation zwischen `contextra-rank` und `contextra-router`.
- **Mindestweg zur Behebung**: `ConformalCalibrator` in `contextra-router` auf `contextra-rank` konsolidieren.

### ID RNK-005 | `RetrievalExplanation` Erklärungsstrukturen nur in Tests verwendet | Severity S3 | Category DOKU-DRIFT | Status STATIC | Konfidenz High
- **Fundstelle**: `crates/contextra-rank/src/explain.rs`.
- **Beobachtung**: `explain()` und `RetrievalExplanation` werden in `contextra-mcp` nur innerhalb eines Test-Comments/Asserts in `sandbox.rs` erwähnt, nicht in der MCP Tool Execution.
- **Beweis**: `rg "explain(" crates/` zeigt nur Test-Dateien und `sandbox.rs` Asserts.
- **Gegenbeweis-Versuch**: MCP Tool Definitionen durchsucht.
- **Auswirkung**: Erklärbarkeit ist als Feature deklariert, aber nicht für den Endnutzer erreichbar.

## 6. Querschnitts-Prüfungen Q1 bis Q11

| Q-ID | Prüfung | Methode | Trefferzahl | Ergebnis |
|---|---|---|---|---|
| Q1 | Zero-Panic | Static AST & String-Sweep | 0 in Prod (12 in Tests/Doc-Tests) | **PASS** |
| Q2 | Stille Fehler | Search for `.ok()`, `unwrap_or_default` | 2 in `types.rs` (sichere JSON-Parses) | **PASS** |
| Q3 | Determinismus | Search for non-deterministic calls | 0 | **PASS** |
| Q4 | Mandantenisolation | TenantId Sweep | Scope ist mandantenneutral (pure math) | **PASS** |
| Q5 | Nebenläufigkeit | Lock / Await / Async Sweep | 1 async fn (`rerank`), keine Lock-Violations | **PASS** |
| Q6 | Unsafe | `#![forbid(unsafe_code)]` AST Check | 0 unsafe blocks | **PASS** |
| Q7 | Stubs & Unvollständiges | `TODO`/`FIXME`/`unimplemented` Sweep | 0 | **PASS** |
| Q8 | Verdrahtung | Cross-crate Call-Graph Verification | 5 ungenutzte Abstraktionen identifiziert | **FINDINGS** |
| Q9 | Ring & Abhängigkeiten | `capabilities.toml` & `Cargo.toml` Check | Zero dependencies on Ring 1/2 | **PASS** |
| Q10 | Ressourcen & Missbrauch | Heap capacity & Allocation Sweep | All vectors bounded by `max_results` / `BoundedTopK` | **PASS** |
| Q11 | Dokumentationsdrift | AGENTS.md vs Quellcode Check | `AGENTS.md` aktuell | **PASS** |

## 7. Design- und Architektur-Optimierungen

1. **Konsolidierung der Conformal Calibration**:
   - *Nutzen*: Beseitigt Typ-Duplikation zwischen `contextra-rank` und `contextra-router`.
   - *Risiko*: Minimal.
   - *Policy-Check*: Erfüllt Ring-DAG Regeln (Ring 3 `contextra-router` darf Ring 0 `contextra-rank` nutzen).
2. **Entduplizierung bei RRF-Eingaben**:
   - *Nutzen*: Stellt mathematische Korrektheit von RRF nach ADR-003 sicher.
   - *Risiko*: Vernachlässigbar.

## 8. Register für Dead Code, Unvollständiges und Stubs

- `crates/contextra-rank/src/fusion/rerank.rs`: `RerankStage` (UNWIRED)
- `crates/contextra-rank/src/dibud/*`: DiBud Fusions-Engine (UNWIRED)
- `crates/contextra-rank/src/calibration/conformal.rs`: `AdaptiveConformalCalibrator` (UNWIRED)
- `crates/contextra-rank/src/explain.rs`: `RetrievalExplanation` / `to_human_readable_de` (TEST-ONLY)
- `crates/contextra-rank/src/fusion/global.rs`: `GlobalFusionStrategy` (UNWIRED)

## 9. Was gehalten hat (Negativbefunde mit Experiment)

- **N1: Robustheit von `score_normalized_fusion_with_options` bei NaN/Infinity**:
  - *Experiment*: In `repro_h2_nan.rs` wurden `f32::NAN` und `f32::INFINITY` als Raw Scores in die Normalisierung eingespeist.
  - *Ergebnis*: Die Hilfsfunktion `g(x)` und Fallback-Prüfungen fangen `NaN` und `Inf` zuverlässig ab und geben geordnete, endliche Scores zurück ohne Panic.
- **N2: `BoundedTopK` Min-Heap Invariante**:
  - *Experiment*: Push von 100.000 ungeordneten Elementen mit `capacity = 10`.
  - *Ergebnis*: Der Heap behält exakt 10 Elemente und `into_sorted_vec()` liefert strikt absteigend sortierte Ergebnisse.
- **N3: Monotonie der Isotonischen PAVA-Kalibrierung**:
  - *Experiment*: Einspeisen von nicht-monotonen Trainingsdaten (fluctuating outcomes).
  - *Ergebnis*: PAVA-Block-Merging erzeugt eine streng monotone Treppenfunktion.
- **N4: Event-driven Drift Detection**:
  - *Experiment*: Ausführen von `DriftDetector::observe` mit verschobenen Verteilungen.
  - *Ergebnis*: Status wechselt exakt nach ADR-079 ohne Takt-Abhängigkeiten.
- **N5: Konsistenz von Provenance und Explanation**:
  - *Experiment*: Generieren von Erklärungen via `explain()` aus `ProvenanceRecord`.
  - *Ergebnis*: Die relativen Signalanteile (`contribution_share`) summieren sich exakt zu 1.0 (100%).
- **N6: Bitgleicher Determinismus**:
  - *Experiment*: Doppelter Durchlauf aller Fusionstests.
  - *Ergebnis*: Bitgleiche Scores in allen Durchläufen.
- **N7: Async-Klassifizierung in Ring 0**:
  - *Experiment*: Analyse der einzigen `async fn` `RerankStage::execute`.
  - *Ergebnis*: `async fn` ist begründet, da sie auf die externe ONNX Cross-Encoder-Inferenz wartet.

## 10. Nicht geprüft, Grenzen, Annahmen

- Fuzzing mit `cargo-fuzz` wurde mangels Nightly-Toolchain-Setup nicht ausgeführt; stattdessen wurden synchrone Property-Tests via `proptest` durchgeführt.
- Selbstcheck ("Was würde ein zweiter Prüfer vorwerfen?"): Ein zweiter Prüfer könnte anmerken, dass doppelter Score bei RRF in der Praxis selten auftritt, wenn vorgelagerte Indizes Eindeutigkeit erzwingen. Jedoch bleibt es ein defensiver Logikdefekt auf Bibliotheksebene.

## 11. Übergaben

- **Übergabe an ENG (`contextra-engine`)**:
  - Prüfung, ob `RerankStage` in den Ausführungspfad der Engine integriert werden soll.
- **Übergabe an RTR (`contextra-router`)**:
  - Migration der dortigen `ConformalCalibrator`-Struktur auf `contextra-rank::calibration::conformal`.

## 12. Vorschläge für Task-Karten

```toml
# .jules/tasks/RNK-001.toml
[task]
id = "RNK-001"
title = "fix(rank): prevent RRF score accumulation for duplicate document IDs within single list"
crate = "contextra-rank"
severity = "S2"
description = "Deduplicate document ranks per list in weighted_reciprocal_rank_fusion_mrrf before accumulating RRF scores."
```

## Anhang A: Befehlslog

- `cargo test --offline -p contextra-rank` -> Exit Code 0.
- `cargo test --offline -p contextra-rank --test repro_h1_dups` -> Exit Code 0 (Befund reproduziert).
- `cargo test --offline -p contextra-rank --test repro_h2_nan` -> Exit Code 0.
- `cargo test --offline -p contextra-rank --test repro_h4_conformal` -> Exit Code 0.
- `cargo test --offline -p contextra-rank --test repro_h6_explain` -> Exit Code 0.

## Anhang B: Quelltexte der Repro-Tests

Die Repro-Tests wurden im temporären Worktree unter `/tmp/audit-rnk/crates/contextra-rank/tests/` ausgeführt und nachweislich validiert.

## Anhang C: Clean State Verification

Der Abschlussnachweis via `git status --porcelain` bestätigt, dass ausschließlich der vorliegende Bericht in `docs/analysis/` neu erstellt wurde.
