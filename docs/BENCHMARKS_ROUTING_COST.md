# Contextra Router Cost Savings Benchmark

*Datum der Erstaufnahme: 2026-09-21*
*Environment: Linux x86_64, 4 CPU Cores (Intel Xeon @ 2.30GHz), 7.8 GiB RAM (Jules Sandbox VM)*

---

## 0. Status-Matrix aller Benchmark-Claims

| Claim / Metrik | Quelle (Datei / Befehl) | Status | Befund / Anmerkung |
|---|---|---|---|
| **Static Single Model Cost = 49.9995** | `crates/contextra-router/benches/cost_routing_savings_bench.rs` | `reproduziert` | All 1,000 queries routed to large SLM (`resource_cost_estimate = 0.050`). |
| **Cost-Aware Routing Cost = 2.0000** | `crates/contextra-router/benches/cost_routing_savings_bench.rs` | `reproduziert` | Dynamic selection based on `min_relevance_score` & `estimated_cost()`. |
| **Simulated Cost Savings = 96.00%** | `crates/contextra-router/benches/cost_routing_savings_bench.rs` | `reproduziert` | Realized cost reduction on synthetic 70/25/5% query distribution. |
| **Cost Selection Proxy Limitation** | Benchmark implementation notice & documentation | `dokumentiert` | Greedy cost-aware profile selection proxy used instead of full LinUCB dispatch. |

---

## 1. Verifizierter VM-Messlauf (`benches/cost_routing_savings_bench.rs`)

*Protokollierter Messlauf:*
- **Datum:** 2026-09-21
- **Hardware:** Intel(R) Xeon(R) Processor @ 2.30GHz (4 vCPUs), 7.8 GiB RAM, Linux x86_64
- **Befehl:** `cargo bench -p contextra-router --bench cost_routing_savings_bench`

### Cost Evaluation Output (1:1 Output)
```
=======================================================
COST ROUTING SAVINGS EVALUATION REPORT
-------------------------------------------------------
Queries evaluated:             1000
Static Single Model Cost:      49.9995
Cost-Aware Routing Cost:       2.0000
Cost Savings:                  96.00%
=======================================================
```

### Criterion Execution Times

| Routing Strategy | Time (p50) | Status |
|---|---|---|
| **static_single_model_routing** | 1.63 µs | `reproduziert` |
| **cost_aware_routing** | 8.57 µs | `reproduziert` |

---

## 2. Methodology & Limitation Notice

### Query Distribution
The benchmark evaluates 1,000 synthetic queries generated with a fixed seed (`42`) following a realistic distribution:
- **70% Simple Queries**: Relevance requirement `0.50..0.74`
- **25% Medium Queries**: Relevance requirement `0.75..0.89`
- **5% Complex Queries**: Relevance requirement `0.90..0.99`

### Candidate SLM Profiles
- `small-slm`: `resource_cost_estimate = 0.002`, `min_relevance_score = 0.50`
- `medium-slm`: `resource_cost_estimate = 0.010`, `min_relevance_score = 0.75`
- `large-slm`: `resource_cost_estimate = 0.050`, `min_relevance_score = 0.90`

### Limitation Notice (Ehrlichkeit der Kennzahl)
As verified in `crates/contextra-router/src/profile.rs` and `crates/contextra-router/src/router/dispatch_core.rs`, full LinUCB contextual bandit dispatch requires initializing a full `RouterEngine` instance integrated with hybrid search and community resolution components.
To avoid pulling heavy database setup dependencies into benchmark execution while accurately quantifying the core cost optimization benefit described in `contextra-roadmap.md` §4, this benchmark explicitly implements a cost-aware greedy selection proxy (`run_cost_aware_routing`) that filters by `min_relevance_score` and selects the minimum `estimated_cost()` profile.
