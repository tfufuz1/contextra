# Contextra Retrieval Evaluation Benchmark Protocol Specification

*Document Version:* 1.0.0
*Status:* Draft Specification
*Target Frameworks:* LoCoMo, LongMemEval, BEAM Retrieval Suites
*Scope:* Standardized evaluation harness for long-context memory retrieval and multi-signal fusion in Contextra.

---

## 1. Overview & Objective

This document defines the evaluation protocol and execution parameters for benchmarking Contextra's cognitive retrieval capabilities against standard external long-context benchmarks:
1. **LoCoMo**: Long-Context Memory Benchmark evaluating factual recall, temporal reasoning, and multi-turn context retention.
2. **LongMemEval**: Long-Term Memory Evaluation suite assessing dynamic knowledge state changes, including the **Knowledge Update** category (superseded facts and memory invalidation).
3. **BEAM**: Multi-hop associative reasoning benchmark evaluating graph traversal and vector-text-graph signal fusion.

The protocol enforces deterministic execution, explicit LLM-as-a-judge criteria, token budget constraints, and rigorous signal ablation matrices.

---

## 2. Fixed Benchmark Execution Parameters

To ensure exact reproducibility across evaluation runs, all benchmark executions MUST adhere to the following fixed settings:

| Parameter | Specification | Default Value / Setting |
|---|---|---|
| **Answer Generation Model** | Target LLM used to formulate final answers based on retrieved memory context | `gpt-4o-mini` (or local equivalent `llama-3.1-8b-instruct`) |
| **Judge Model (LLM-as-a-Judge)** | Independent evaluator scoring accuracy, hallucination, and recall correctness | `gpt-4o` (temperature = 0.0, seed = 42) |
| **Reranking Engine** | Cross-encoder / reranker applied to merged candidates | `ON` (Cross-Encoder / Cohere Rerank API fallback) & `OFF` (Ablation) |
| **Token Budget per Query** | Max context window tokens allocated for retrieved memory payload | 2,048 tokens / query |
| **Execution Runs & Seeds** | Repetitions for statistical significance | 5 runs across seeds `[42, 123, 456, 789, 101112]` |
| **MVCC Snapshot Mode** | Transactional snapshot isolation level during search | Snapshot-isolated (`read_seq` fixed at query start) |

---

## 3. Signal Ablation Matrix

Retrieval quality MUST be evaluated across four distinct signal configuration tiers to isolate the contribution of each subsystem:

1. **Tier A: Dense Vector Only (`Vector-only`)**
   - HNSW cosine similarity over document chunk embeddings.
   - Weights: Vector = 1.0, BM25 = 0.0, Graph PPR = 0.0, Temporal = 0.0.

2. **Tier B: 3-Signal Hybrid (`Vector + BM25 + Graph PPR`)**
   - Reciprocal Rank Fusion (RRF) combining dense vector search, inverted index BM25 lexical matching, and Personalized PageRank (PPR) over the CSR graph.
   - Standard Fusion Weights: Vector = 0.45, BM25 = 0.35, Graph = 0.20.

3. **Tier C: 4-Signal Hybrid (`Vector + BM25 + Graph PPR + Temporal/Context`)**
   - Full Contextra cognitive fusion incorporating temporal decay and document versioning provenance.
   - Fusion Weights: Vector = 0.40, BM25 = 0.30, Graph = 0.20, Temporal Decay = 0.10.

---

## 4. Benchmark Dataset Protocols

### 4.1 LoCoMo Evaluation Protocol
- **Dataset Focus:** Long conversational dialogs with multi-session factual queries.
- **Metrics:** F1 Score, ROUGE-L, Exact Match (EM), Recall@k ($k \in \{5, 10, 20\}$).
- **Execution Script:** `cargo test -p contextra-bench --test external_benchmarks_test -- test_locomo_suite`

### 4.2 LongMemEval Protocol (Including Knowledge Update)
- **Dataset Focus:** Long-term user interaction logs requiring handling of memory updates, fact overrides, and tombstones.
- **Key Test Category - Knowledge Update:** Evaluates whether retrieved context correctly prioritizes updated facts over stale historical statements without returning invalidated tombstones.
- **Metrics:** Update Accuracy (percentage of queries returning only active facts), Invalidation Precision, Recall@10.
- **Execution Script:** `cargo test -p contextra-bench --test external_benchmarks_test -- test_long_mem_eval_suite`

### 4.3 BEAM Protocol
- **Dataset Focus:** Multi-hop graph traversal and associative entity linking.
- **Metrics:** Traversal Accuracy, Hit@5, MRR (Mean Reciprocal Rank).

---

## 5. Raw Data Storage & Artifact Structure

Evaluation runs generate structured JSON result logs stored in the repository under:
`benchmarks/results/`

### File Naming Convention:
`benchmarks/results/{benchmark_name}_raw_{timestamp}_{commit_hash}.json`

Example:
`benchmarks/results/locomo_raw_20260921_347ef6d.json`

### Required JSON Schema Fields:
```json
{
  "benchmark": "LoCoMo",
  "commit_hash": "347ef6dd86d90fdbc8f6dc0fcfa54ab6bb0c8986",
  "timestamp": "2026-09-21T21:00:00Z",
  "config": {
    "answer_model": "gpt-4o-mini",
    "judge_model": "gpt-4o",
    "reranking": true,
    "token_budget": 2048,
    "seed": 42,
    "signal_tier": "4-Signal"
  },
  "metrics": {
    "recall_at_5": null,
    "recall_at_10": null,
    "f1_score": null,
    "knowledge_update_accuracy": null
  },
  "raw_queries": []
}
```

---

## 6. Known Harness Technical Blocker (Snapshot Isolation & PPR / PathRag)

### Status & Root Cause Analysis
During benchmark harness validation, tests utilizing `PathRag` graph retrieval or Personalized PageRank (PPR) under active MVCC snapshot isolation currently encounter a runtime limitation:
- **Error:** `ContextraError::SnapshotUnsupportedForSignal("PathRag strategy does not support snapshot-isolated retrieval")`
- **Location:** `crates/contextra-db/src/collection/search.rs`
- **Architectural Reason:** PPR graph traversal operates on the live unversioned CSR graph structure (`contextra-graph`), which does not yet maintain sequence-number-aware edge visibility slices for historical MVCC snapshots. Consequently, snapshot isolation explicitly rejects PPR/PathRag queries to prevent stale data leakage.

### Impact on Protocol Execution
Multi-signal benchmark sweeps (Tier B and Tier C) that request snapshot-isolated PathRag retrieval will return `SnapshotUnsupportedForSignal` until graph MVCC edge versioning is implemented. Benchmark execution scripts MUST handle this error gracefully or run graph sweeps in live read mode where snapshot isolation is disengaged.

---

## 7. Execution Commands & Reproduction

To run the evaluation harness locally within the workspace environment, execute the following commands:

```bash
# 1. Run all external benchmark harness integration tests
cargo test -p contextra-bench --test external_benchmarks_test -- --nocapture

# 2. Run LoCoMo benchmark sweep (standalone)
cargo test -p contextra-bench --test external_benchmarks_test -- test_locomo_sweep --nocapture

# 3. Run LongMemEval benchmark sweep (standalone)
cargo test -p contextra-bench --test external_benchmarks_test -- test_long_mem_eval_sweep --nocapture
```

---

## 8. Benchmark Metric Result Templates

*(Note: The tables below serve as empty protocol templates for future evaluation runs. No fabricated or unverified numbers are listed.)*

### 8.1 LoCoMo Retrieval Metrics Template

| Signal Config Tier | Reranker | Recall@5 | Recall@10 | F1 Score | ROUGE-L | Status |
|---|---|---|---|---|---|---|
| Tier A (Vector-only) | OFF | - | - | - | - | `pending` |
| Tier A (Vector-only) | ON | - | - | - | - | `pending` |
| Tier B (3-Signal) | OFF | - | - | - | - | `blocked (SnapshotUnsupportedForSignal)` |
| Tier B (3-Signal) | ON | - | - | - | - | `blocked (SnapshotUnsupportedForSignal)` |
| Tier C (4-Signal) | OFF | - | - | - | - | `pending` |
| Tier C (4-Signal) | ON | - | - | - | - | `pending` |

### 8.2 LongMemEval & Knowledge Update Template

| Signal Config Tier | Knowledge Update Acc (%) | Invalidation Precision | Overall Recall@10 | Status |
|---|---|---|---|---|
| Tier A (Vector-only) | - | - | - | `pending` |
| Tier B (3-Signal) | - | - | - | `blocked (SnapshotUnsupportedForSignal)` |
| Tier C (4-Signal) | - | - | - | `pending` |
