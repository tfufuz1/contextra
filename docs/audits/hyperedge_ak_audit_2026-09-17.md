# Hyperkanten-Abnahme-Audit (AK-1 bis AK-8)

**Audit-Datum:** 2026-09-17
**Auditor:** Jules (Principal Rust Systems Engineer)
**Ziel-Crates:** `crates/memfuse-graph`, `crates/memfuse-db`
**Referenz-Spezifikation:** `MEMFUSE_FINALE_KONSOLIDIERTE_GESAMTSPEZIFIKATION.md`, §6 & §16.2

---

## Executive Summary

Im Rahmen dieses Audits wurden die Abnahmekriterien **AK-1 bis AK-8** (§16.2) gegen den aktuellen Ist-Zustand der Codebasis im MemFuse-Repository geprüft.

### Grund für die Status-Diskrepanz (🔴 in §19 vs. Teil-Implementierung im Code)
In §19 der Gesamtspezifikation ist der Reifegrad für *N-äre Hyperkanten* mit 🔴 ("Nicht implementiert / Ausstehend") deklariert. Ein Quellcode-Abgleich zeigt jedoch, dass Kernstrukturen (`HyperEdge`, `RoleBinding`, `RoleId`, `RoleInterner`, `relate_n_ary`, `cascade_invalidate_hyperedges_for_superseded_doc`) bereits im Code existieren.

Der Grund für den Spezifikations-Status 🔴 liegt darin, dass wichtige **Abnahmekriterien (AK-1, AK-2, AK-5, AK-6, AK-8)** noch Lücken aufweisen (z.B. fehlende Restart-Persistenz beim Systemstart, fehlende CI-Job-Abhängigkeit und fehlende Abnahmetests), weshalb das Gesamtfeature formal noch nicht abgenommen werden kann.

---

## Ergebnismatrix (AK-1 bis AK-8)

| AK-Nr. | Status | Nachweis-Fundort / Lücken-Beschreibung |
|---|---|---|
| **AK-1** | ❌ | **Lücke:** <br>1. `HyperEdge::validate()` in `crates/memfuse-graph/src/hyperedge.rs:165` prüft nur `len < 2` statt der für AK-1 geforderten ≥3 `RoleBinding`s.<br>2. `CsrGraph::load_from_storage` in `crates/memfuse-graph/src/csr.rs:709-866` lädt beim System-Restart keine Hyperkanten aus dem LSM-Storage (`__graph:hyperedge:`), d.h. Hyperkanten sind aktuell **nicht restart-fest**.<br>3. Die geforderten Test-Dateien `hyperedge_persistence_survives_restart.rs` und `hyperedge_compact_race.rs` fehlen im Repository. |
| **AK-2** | ❌ | **Teilweise erfüllt / Lücke:** <br>`estimate_memory_bytes()` in `crates/memfuse-graph/src/csr.rs:320-353` schließt Hyperkanten ein und `compact_async` in `csr.rs:1610-1622` prüft das Budget.<br>**Lücke:** Die geforderte Test-Datei `hyperedge_memory_budget.rs` existiert nicht im Repository. |
| **AK-3** | ✅ | **Erfüllt:** `Collection::relate_n_ary` in `crates/memfuse-db/src/collection/relate.rs:98-106` sortiert `EntityId`s vor dem Lock-Erwerb kanonisch.<br>Der Loom-Test `test_concurrent_relate_n_ary_deadlock_free_overlapping_participants` in `crates/memfuse-db/tests/loom_relate_n_ary.rs:25-88` beweist Deadlockfreiheit unter nebenläufigen Aufrufen mit unterschiedlich geordneten Teilnehmermengen `{doc-1, doc-2, doc-3}` vs. `{doc-3, doc-2, doc-1}`. |
| **AK-4** | ✅ | **Erfüllt:** `SignalKind` in `crates/memfuse-db/src/fusion.rs:204-214` bleibt unverändert (keine neue Hyperedge-Variante). Hyperkanten werden intern in PathRAG/CSR verarbeitet. Vertragstest in `crates/memfuse-graph/src/path_rag.rs:753-754`. |
| **AK-5** | ❌ | **Lücke:** Zwar existiert `check-flatbuffers-drift` in `.github/workflows/context-gates.yml:23`, aber die explizite CI-Job-Abhängigkeit `hyperedge-schema-merge: needs: [flatbuffers-drift-gate]` ist in den `.github/workflows/*.yml` Workflows nicht definiert. |
| **AK-6** | ❌ | **Lücke:** In `crates/memfuse-graph/src/cascade.rs:114-138` wird der Fan-out bei >1.000 Hyperkanten über `MAX_HYPEREDGE_CASCADE_FANOUT` getrennt (`invalidated` vs. `deferred`).<br>**Lücke:** In `memfuse-db` fehlt der Hintergrund-Worker zur Abarbeitung der `deferred` Hyperkanten-Tombstones, und die Test-Datei `hyperedge_cascade_fanout.rs` fehlt. |
| **AK-7** | ✅ | **Erfüllt:** `CommunityDetectionConfig.hyperedges_included` in `crates/memfuse-graph/src/community.rs:81-89` ist standardmäßig `false` und wird im Report ohne Projektion sichtbar als `false` ausgegeben. Tests in `community.rs:651-700`. |
| **AK-8** | ❌ | **Lücke:** Ein spezifischer Regressions-Test `binary_edge_regression.rs` zur Verifikation, dass N-äre Hyperkanten keine Regressionsauswirkungen auf binäre `relate()`-Benchmarks haben, existiert nicht im Repository. |

---

## Folge-Prompt-Vorschläge (Task-Typ FIX / IMPL)

Falls die identifizierten Lücken geschlossen werden sollen, stehen hier die isolierten Folge-Prompts gemäß dem Jules-Prompt-Template zur Verfügung:

### Folge-Prompt 1: Restart-Persistenz & AK-1 Validierung (Task-Typ FIX)
```markdown
# TASK: Fix Hyperedge Restart Persistence & Minimum Binding Validation (AK-1)

## 1. SCOPE & CRATE-CLAIMING
* **Ziel-Crates:** `crates/memfuse-graph`
* **Command:** `cargo xtask claim --crate memfuse-graph --mode read-write`

## 2. ANFORDERUNGEN
1. Erweitere `CsrGraph::load_from_storage` in `crates/memfuse-graph/src/csr.rs`, sodass beim Start des Systems alle unter `__graph:hyperedge:` (`HYPEREDGE_PREFIX`) abgelegten Hyperkanten deserialisiert und in `inner.hyperedges`, `doc_to_hyperedges` und `hyperedge_index` geladen werden.
2. Stelle sicher, dass `HyperEdge::validate()` in `crates/memfuse-graph/src/hyperedge.rs` bei echten N-ären Hyperkanten mindestens 3 Bindungen (`participants.len() >= 3`) für N-äre Hyperkanten einfordert bzw. die Validierungsregel mit `relate_n_ary` abgleicht.
3. Erstelle die Tests `hyperedge_persistence_survives_restart.rs` und `hyperedge_compact_race.rs` unter `crates/memfuse-graph/tests/` zum Nachweis von AK-1.

## 3. INVARIANTEN & VERIFIKATION
* Keinerlei `.unwrap()` in Produktionscode.
* Führe `cargo test -p memfuse-graph` und `just dag-check` aus.
```

### Folge-Prompt 2: Deferred Cascade Fanout Background Worker (Task-Typ IMPL)
```markdown
# TASK: Implement Deferred Hyperedge Cascade Background Processing (AK-6)

## 1. SCOPE & CRATE-CLAIMING
* **Ziel-Crates:** `crates/memfuse-db`, `crates/memfuse-graph`
* **Command:** `cargo xtask claim --crate memfuse-db --mode read-write`

## 2. ANFORDERUNGEN
1. Binde in `crates/memfuse-db/src/background_workers.rs` einen Worker zur Verarbeitung von deferred Hyperkanten-Tombstones ein, die von `cascade_invalidate_hyperedges_for_superseded_doc` zurückgegeben werden.
2. Erstelle den Integrationstest `crates/memfuse-graph/tests/hyperedge_cascade_fanout.rs`, der >1.000 Hyperkanten provoziert und die Umstellung auf die Hintergrundverarbeitung prüft.

## 3. INVARIANTEN & VERIFIKATION
* Führe `cargo test -p memfuse-graph` und `cargo test -p memfuse-db` aus.
```

### Folge-Prompt 3: CI Drift Gate Dependency & Benchmark Verification (Task-Typ FIX)
```markdown
# TASK: CI Job Dependency & Benchmark Regression Tests (AK-5 & AK-8)

## 1. SCOPE & CRATE-CLAIMING
* **Ziel-Crates:** `.github/workflows`, `benches`
* **Command:** `cargo xtask claim --crate xtask --mode read-write`

## 2. ANFORDERUNGEN
1. Ergänze in `.github/workflows/context-gates.yml` bzw. dem entsprechenden Merge-Workflow den Job `hyperedge-schema-merge` mit der expliziten Abhängigkeit `needs: [flatbuffers-drift-gate]`.
2. Erstelle den Benchmark-/Testpfad `benches/binary_edge_regression.rs` bzw. Integrationstest, um sicherzustellen, dass Hyperkanten-Datenstrukturen keine Performance-Regression auf binäre `relate()`-Operationen verursachen.
```
