# Audit-Bericht: `contextra-db`

**Datum:** 2026-10-04
**Auditor:** Principal Senior Rust Architect (Jules)
**Crate:** `contextra-db` (Layer 3 Strangler Shell / Re-export Facade)
**Status / Ring:** Ring 3 / Layer 3 — `#![forbid(unsafe_code)]`
**Claim Ticket:** `full-audit`

---

## Executive Summary

Das Crate `contextra-db` dient als primäre High-Level-API und Strangler-Shell für Contextra. Es re-exportiert und orchestriert Compute- und Storage-Komponenten aus `contextra-engine` (Layer 3) sowie Kognitions- und Konsolidierungslogik aus `contextra-cognition` (Layer 3).

Sämtliche kritischen Sicherheits- und Architektur-Invarianten wurden im Rahmen des Audits geprüft. `contextra-db` hält `#![forbid(unsafe_code)]` strikt ein.

---

## (1) AGT-DB-001-Nachweis (TxId-Erzeugung)

**Invariante:** `TxId` **MUSS IMMER** über atomic counter `next_tx` via `collection.allocate_tx().await` bezogen werden. Verwendung von `SystemTime::now()` oder `Instant::now()` außerhalb von Testfächern ist unzulässig.

* **Audit-Befund:** `grep -rn "SystemTime::now()\|Instant::now()\|UNIX_EPOCH" crates/contextra-db/src/ | grep -v test` lieferte ein leeres Ergebnis.
* **EVIDENCE-Marker:** `EVIDENCE-AGT-DB-001: ZERO_TIMESTAMP_CALLS_IN_SRC`
* **Ergebnis:** Konform. Die Transaktions-ID-Erzeugung erfolgt ausschließlich deterministisch über den atomaren `next_tx`-Zähler der `Collection`.

---

## (2) Parallelitäts-Analyse der 4-Signal-Fusion

**Invariante / Erwartung:** Bei einer hybriden Multi-Signal-Suche sollten die Retrieval-Signale (Vector, Text, Graph) möglichst asynchron-parallel abgefragt werden.

* **Audit-Befund:**
  Die Re-Export-Pipeline greift in `contextra-engine::collection::search::hybrid::mod.rs` auf `hybrid_search_with_strategy` zu. Innerhalb des `with_pinned_checkpoint_at_latest`-Snapshots werden die Signale sequentiell nacheinander ausgeführt:
  1. `self.search_filtered_at(...)` (Vector HNSW Signal) via `.await`
  2. `self.text_index.search_at(...)` + Hydration (Text BM25 Signal) via `.await`
  3. `self.graph_index.multi_traverse_at(...)` / PPR / PathRag + Hydration (Graph Signal) via `.await`
* **EVIDENCE-Marker:** `EVIDENCE-4SIGNAL-PARALLELISM: SEQUENTIAL_AWAIT_CHAIN`
* **Beurteilung:** **Performance-Befund (Nicht-Kritisch / Optimization Target)**. Die sequentielle Ausführung gewährleistet zwar perfekte Snapshot-Isolierung unter demselben Pinned Checkpoint Sequence Number, erzeugt jedoch bei Latenz-kritischen Anfragen eine kaskadierte Latenz der Einzelkomponenten. Eine zukünftige Refaktorisierung auf `tokio::join!` innerhalb der Pinned-Snapshot-Grenzziehung wird empfohlen.

---

## (3) Markdown-Chunker-Pflicht (Agenten-Wissen)

**Invariante:** Agenten-Wissen in Markdown-Form darf nicht als unstruktureller Riesenstring an die Vector-Embedding-Engine übergeben werden.

* **Audit-Befund:** `MarkdownChunker` in `chunker.rs` (re-exportiert via `contextra_engine::chunker`) teilt Dokumente strukturell an Heading-Befehlen (`#`, `##`) in semantische Einheiten auf und vererbt Parent-Headings.
* **Beurteilung:** Konform. Das API-Design von `contextra-db` stellt den Chunker für Agenten-Pipelines bereit und dokumentiert die Chunker-Pflicht in `AGENTS.md`.

---

## (4) Bug-Proof-Test-Ergebnisse (Search-Result-Bound)

**Test-Befehl:** `cargo test -p contextra-db --test search_result_bound`
* **Ergebnis:** 4/4 Tests erfolgreich bestanden.
  * `proof_search_never_exceeds_k` — OK
  * `proof_search_with_k_zero_returns_empty` — OK
  * `proof_search_returns_nonzero_results_for_matching_query` — OK
  * `proof_usize_max_removed_from_search_path` — OK
* **EVIDENCE-Marker:** `EVIDENCE-SEARCH-BOUND: 4_PASSED_0_FAILED`

---

## (5) Proptest-Search-Invarianten

**Test-Befehl:** `PROPTEST_CASES=20 cargo test -p contextra-db --test proptest_search_invariants`
* **Ergebnis:** 5/5 Property-Tests erfolgreich bestanden.
  * `prop_rrf_bounded_by_k` — OK
  * `prop_e2e_search_bounded_by_k` — OK
  * `prop_bm25_score_monotonic_in_tf` — OK
  * `prop_rrf_scores_sorted_descending` — OK
  * `prop_rrf_empty_inputs_never_panic` — OK
* **EVIDENCE-Marker:** `EVIDENCE-PROPTEST: 5_PASSED_0_FAILED`

---

## (6) Volatile-Vault-Sicherheit (`volatile_vault.rs`)

**Invariante:** Ephemerer RAM-Puffer für sensitive MCP-Tool-Ergebnisse / Safe-Modus (Gated via Feature `volatile-vault`). No Disk Fallback, Zeroize-on-Drop, mlock Best-Effort.

* **Audit-Befund:**
  * `VaultChunk` ist annotiert mit `#[derive(ZeroizeOnDrop)]`.
  * `VolatileContextVault::purge()` führt explizit `chunk.content.zeroize()` vor `munlock()` und Deallokation aus.
  * `Drop`-Implementierung stellt sicher, dass auch ohne explizites `purge()` alle Chunks im RAM überschrieben werden.
  * RAM-Fixierung erfolgt via `contextra_sys::LockedRegions` (`mlock` / `VirtualLock`) zur Vermeidung von OS-Swap-Auslagerung.
  * Kapazitätsgrenze (`max_capacity_bytes`, Default 512 MB) bricht bei Überschreitung mit `VaultError::CapacityExceeded` ab — absolut kein Disk-/WAL-Fallback (INV-VAULT-3).
* **EVIDENCE-Marker:** `EVIDENCE-VOLATILE-VAULT: ZEROIZE_ON_DROP_AND_MLOCK_CONFIRMED`
* **Ergebnis:** Konform.

---

## (7) Multi-Step Query-Rewrite Loop-Limit (`multistep.rs`)

**Invariante:** Schutz vor unendlichen LLM-Abfrage-Schleifen im iterativen RAG (o-series Pattern).

* **Audit-Befund:**
  * Iterationsschleife ist fest limitiert durch `MultiStepConfig.max_rounds` (Default: 3, Schleifenkopf `for _round in 2..=self.config.max_rounds`).
  * Jede Runde wird zusätzlich durch einen PID-regulierten `LatencyBudgetGuard` überwacht (`budget_guard.is_exceeded()`), welcher bei Budgetüberschreitung die Erweiterung sofort abbricht.
  * LLM-Rewriter-Aufrufe (`rewriter.rewrite_structured(...)`) sind zeitlich via `tokio::time::timeout` an das verbleibende Latenzbudget gebunden.
* **EVIDENCE-Marker:** `EVIDENCE-MULTISTEP-BOUND: MAX_ROUNDS_AND_LATENCY_GUARD_CONFIRMED`
* **Ergebnis:** Konform.

---

## VERDICT

```text
================================================================================
AUDIT VERDICT: PASSED (WITH PERFORMANCE RECOMMENDATION)
CRATE: contextra-db
RING: 3 (Layer 3 Strangler Shell / Re-export Facade)
UNSAFE CODE: FORBIDDEN (#![forbid(unsafe_code)])

SUMMARY OF EVIDENCE:
- EVIDENCE-AGT-DB-001: ZERO_TIMESTAMP_CALLS_IN_SRC
- EVIDENCE-4SIGNAL-PARALLELISM: SEQUENTIAL_AWAIT_CHAIN
- EVIDENCE-SEARCH-BOUND: 4_PASSED_0_FAILED
- EVIDENCE-PROPTEST: 5_PASSED_0_FAILED
- EVIDENCE-VOLATILE-VAULT: ZEROIZE_ON_DROP_AND_MLOCK_CONFIRMED
- EVIDENCE-MULTISTEP-BOUND: MAX_ROUNDS_AND_LATENCY_GUARD_CONFIRMED

RECOMMENDATIONS:
1. Optimize 4-signal retrieval in contextra-engine by fanning out vector, text,
   and graph signal queries via tokio::join! under the same pinned checkpoint sequence.
================================================================================
```
