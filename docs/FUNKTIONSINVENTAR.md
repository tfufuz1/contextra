# Contextra — Funktionsinventar pro Crate

> Vollständige, automatisch aus dem Quellcode (`crates/*/src/**/*.rs` sowie `benchmarks/` & `xtask*`) extrahierte Übersicht aller Crates, öffentlichen Typen und Funktionen.
> *Hinweis:* Produktive Funktionen (`pub`, `pub(crate)`, `pub(super)`, privat) sind vollständig mit Datei und Zeilennummer aufgelistet; Testfunktionen (`#[test]`, `#[cfg(test)]`) sind in den Summen ausgewiesen.

**Workspace-Crates (`crates/`):** 35 Crates · 681 Quelldateien · 1069 öffentliche Typen · **5446 produktive Funktionen** (3167 `pub`, 2279 intern) · 2438 Testfunktionen.

---

## 1. Gesamtübersicht (Crates)

| Crate | Pfad | Dateien | Pub-Typen | `pub fn` | Intern `fn` | Produktiv gesamt | Tests |
|---|---|---:|---:|---:|---:|---:|---:|
| [`contextra`](#contextra) | `crates/contextra` | 6 | 13 | 51 | 20 | 71 | 6 |
| [`contextra-adapt`](#contextra-adapt) | `crates/contextra-adapt` | 13 | 45 | 106 | 45 | 151 | 78 |
| [`contextra-agent`](#contextra-agent) | `crates/contextra-agent` | 13 | 35 | 88 | 63 | 151 | 40 |
| [`contextra-audit-export`](#contextra-audit-export) | `crates/contextra-audit-export` | 5 | 9 | 15 | 4 | 19 | 2 |
| [`contextra-avv-generator`](#contextra-avv-generator) | `crates/contextra-avv-generator` | 3 | 4 | 16 | 1 | 17 | 0 |
| [`contextra-checkpoint`](#contextra-checkpoint) | `crates/contextra-checkpoint` | 11 | 18 | 101 | 79 | 180 | 92 |
| [`contextra-cognition`](#contextra-cognition) | `crates/contextra-cognition` | 19 | 36 | 76 | 50 | 126 | 69 |
| [`contextra-core`](#contextra-core) | `crates/contextra-core` | 1 | 0 | 0 | 0 | 0 | 0 |
| [`contextra-crypto`](#contextra-crypto) | `crates/contextra-crypto` | 18 | 49 | 168 | 45 | 213 | 151 |
| [`contextra-db`](#contextra-db) | `crates/contextra-db` | 4 | 11 | 16 | 5 | 21 | 53 |
| [`contextra-durable-fs`](#contextra-durable-fs) | `crates/contextra-durable-fs` | 7 | 24 | 44 | 49 | 93 | 0 |
| [`contextra-engine`](#contextra-engine) | `crates/contextra-engine` | 83 | 65 | 322 | 195 | 517 | 184 |
| [`contextra-graph`](#contextra-graph) | `crates/contextra-graph` | 54 | 93 | 257 | 162 | 419 | 238 |
| [`contextra-infer-candle`](#contextra-infer-candle) | `crates/contextra-infer-candle` | 15 | 23 | 65 | 53 | 118 | 58 |
| [`contextra-infer-ollama`](#contextra-infer-ollama) | `crates/contextra-infer-ollama` | 15 | 26 | 60 | 25 | 85 | 113 |
| [`contextra-infer-onnx`](#contextra-infer-onnx) | `crates/contextra-infer-onnx` | 6 | 6 | 25 | 18 | 43 | 34 |
| [`contextra-ingest`](#contextra-ingest) | `crates/contextra-ingest` | 78 | 85 | 151 | 314 | 465 | 65 |
| [`contextra-kvcache`](#contextra-kvcache) | `crates/contextra-kvcache` | 9 | 27 | 113 | 54 | 167 | 49 |
| [`contextra-license`](#contextra-license) | `crates/contextra-license` | 2 | 3 | 20 | 5 | 25 | 0 |
| [`contextra-mcp`](#contextra-mcp) | `crates/contextra-mcp` | 25 | 27 | 76 | 39 | 115 | 117 |
| [`contextra-mvcc`](#contextra-mvcc) | `crates/contextra-mvcc` | 5 | 14 | 109 | 40 | 149 | 46 |
| [`contextra-ports`](#contextra-ports) | `crates/contextra-ports` | 21 | 64 | 27 | 172 | 199 | 134 |
| [`contextra-privacy`](#contextra-privacy) | `crates/contextra-privacy` | 12 | 42 | 88 | 61 | 149 | 68 |
| [`contextra-py`](#contextra-py) | `crates/contextra-py` | 16 | 8 | 60 | 11 | 71 | 33 |
| [`contextra-rank`](#contextra-rank) | `crates/contextra-rank` | 20 | 32 | 96 | 33 | 129 | 9 |
| [`contextra-router`](#contextra-router) | `crates/contextra-router` | 28 | 24 | 59 | 42 | 101 | 111 |
| [`contextra-sandbox`](#contextra-sandbox) | `crates/contextra-sandbox` | 8 | 14 | 18 | 9 | 27 | 21 |
| [`contextra-simd`](#contextra-simd) | `crates/contextra-simd` | 7 | 2 | 51 | 0 | 51 | 0 |
| [`contextra-store`](#contextra-store) | `crates/contextra-store` | 70 | 73 | 286 | 321 | 607 | 246 |
| [`contextra-sys`](#contextra-sys) | `crates/contextra-sys` | 8 | 4 | 27 | 7 | 34 | 15 |
| [`contextra-testkit`](#contextra-testkit) | `crates/contextra-testkit` | 5 | 6 | 30 | 21 | 51 | 8 |
| [`contextra-text`](#contextra-text) | `crates/contextra-text` | 24 | 33 | 74 | 117 | 191 | 139 |
| [`contextra-types`](#contextra-types) | `crates/contextra-types` | 23 | 60 | 165 | 70 | 235 | 125 |
| [`contextra-vector`](#contextra-vector) | `crates/contextra-vector` | 41 | 49 | 186 | 111 | 297 | 108 |
| [`contextra-wire`](#contextra-wire) | `crates/contextra-wire` | 6 | 45 | 121 | 38 | 159 | 26 |
| **Summe (`crates/`)** | — | **681** | **1069** | **3167** | **2279** | **5446** | **2438** |

### Werkzeuge & Benchmarks (außerhalb `crates/`)

| Paket | Pfad | Dateien | Pub-Typen | `pub fn` | Intern `fn` | Produktiv gesamt | Tests |
|---|---|---:|---:|---:|---:|---:|---:|
| [`contextra-bench`](#contextra-bench) | `benchmarks/contextra-bench` | 11 | 34 | 26 | 38 | 64 | 23 |
| [`xtask`](#xtask) | `xtask` | 148 | 222 | 590 | 293 | 883 | 232 |
| [`xtask-heavy`](#xtask-heavy) | `xtask-heavy` | 3 | 0 | 5 | 3 | 8 | 4 |

---

## 2. Detailinventar pro Crate

### contextra

> Contextra — Embedded hybrid-search for AI agents (Facade)

- **Pfad:** `crates/contextra` | **Dateien:** 6 | **Produktiv-Funktionen:** 71 (`51` `pub`, `20` intern) | **Tests:** 6
- **Öffentliche Typen (13)**: `MemoryId`, `RelationId`, `Memory`, `AgentMemory`, `ContextraBuilder`, `LsmTuning`, `CollectionProfile`, `CollectionProfileError`, `DeploymentTier`, `PerformanceProfile`, `ResolvedProfileConfig`, `PerformanceProfileError`, `LlmQueryRewriter`

#### `crates/contextra/src/agent_memory.rs` (21)

- ***(Modulfunktionen)***: `next_fallback_id` (`priv`, L24)
- **MemoryId**: `new` (`pub`, L37), `as_str` (`pub`, L42)
- **From<String> for MemoryId**: `from` (`priv`, L48)
- **From<**: `from` (`priv`, L54)
- **AsRef<str> for MemoryId**: `as_ref` (`priv`, L60)
- **fmt::Display for MemoryId**: `fmt` (`priv`, L66)
- **fmt::Display for RelationId**: `fmt` (`priv`, L78)
- **From<SearchResult> for Memory**: `from` (`priv`, L101)
- **From<Arc<Contextra>> for AgentMemory**: `from` (`priv`, L123)
- **From<Contextra> for AgentMemory**: `from` (`priv`, L129)
- **AgentMemory**: `new` (`pub`, L136), `new_arc` (`pub`, L141), `new_with_id_gen` (`pub`, L149), `with_id_gen` (`pub`, L157), `engine` (`pub`, L163), `remember` (`pub`, L181), `recall` (`pub`, L203), `forget` (`pub`, L211), `relate` (`pub`, L218), `relate_n_ary` (`pub`, L238)

#### `crates/contextra/src/builder.rs` (25)

- **std::fmt::Debug for ContextraBuilder**: `fmt` (`priv`, L31)
- **Clone for ContextraBuilder**: `clone` (`priv`, L49)
- **ContextraBuilder**: `new` (`pub`, L65), `from_config` (`pub`, L82), `from_profile` (`pub`, L97), `from_tier` (`pub`, L102), `from_signed_license` (`pub`, L107), `with_storage_path` (`pub`, L116), `with_max_elements` (`pub`, L122), `with_distance_metric` (`pub`, L128), `with_encryption_passphrase` (`pub`, L134), `with_embedding_backend` (`pub`, L140), `with_consolidation` (`pub`, L146), `with_embedder` (`pub`, L153), `with_license_gate` (`pub`, L159), `with_signed_license` (`pub`, L166), `with_performance_profile` (`pub`, L190), `with_collection_profile` (`pub`, L199), `with_config` (`pub`, L209), `storage_path` (`pub`, L228), `config` (`pub`, L233), `license_gate` (`pub`, L238), `performance_profile` (`pub`, L243), `build` (`pub`, L248)
- **Default for ContextraBuilder**: `default` (`priv`, L303)

#### `crates/contextra/src/collection_profile.rs` (3)

- **CollectionProfile**: `validate` (`pub`, L44), `validate_with_license` (`pub`, L53)
- **DeploymentTier**: `resolve` (`pub`, L74)

#### `crates/contextra/src/lib.rs` (3)

- ***(Modulfunktionen)***: `builder` (`pub`, L41), `open` (`pub`, L46), `open_with_config` (`pub`, L51)

#### `crates/contextra/src/performance_profile.rs` (4)

- **Default for PerformanceProfile**: `default` (`priv`, L18)
- **PerformanceProfile**: `resolve` (`pub`, L42)
- **ResolvedProfileConfig**: `enforce_license` (`pub`, L67), `validate` (`pub`, L75)

#### `crates/contextra/src/query_rewriter.rs` (15)

- **LlmQueryRewriter**: `new_with_limits` (`pub`, L22), `new` (`pub`, L43), `with_max_subqueries` (`pub`, L48), `with_max_context_results` (`pub`, L54), `with_max_snippet_chars` (`pub`, L60), `max_subqueries` (`pub`, L66), `max_context_results` (`pub`, L71), `max_snippet_chars` (`pub`, L76)
- **QueryRewriter for LlmQueryRewriter**: `rewrite` (`priv`, L82)
- ***(Modulfunktionen)***: `truncate_str_chars` (`priv`, L102), `extract_snippet` (`priv`, L109), `build_prompt` (`priv`, L125), `clean_line` (`priv`, L153), `strip_code_fences` (`priv`, L177), `parse_response` (`priv`, L186)

### contextra-adapt

> Adaptive controllers, bandits, and PID regulators for Contextra

- **Pfad:** `crates/contextra-adapt` | **Dateien:** 13 | **Produktiv-Funktionen:** 151 (`106` `pub`, `45` intern) | **Tests:** 78
- **Öffentliche Typen (45)**: `BanditError`, `AlignedF32Vec`, `BanditImplementation`, `SketchMatrix`, `BanditPolicy`, `BanditProfileState`, `DecaySignalInputs`, `DecayControllerConfig`, `AdaptiveDecayController`, `DriftSignal`, `DriftDetector`, `CatoniDriftDetector`, `EnsembleDriftWatcher`, `DriftPolicyBridge`, `FcTsError`, `FcTsRng`, `SplitMix64`, `FcTsConfig`, `Ring3Token`, `FlowCorrectedThompsonBandit`, `DiagonalApproximationBandit`, `OffPolicyCompatibility`, `OffPolicyError`, `FcTsSamplingDistribution`, `FcTsArmSet`, `BanditShadowReport`, `BanditDefaultFlipGate`, `DefaultBanditFlipGate`, `RerankPidController`, `RerankDeadline`, `DriftReason`, `LyapunovResult`, `LyapunovDriftWatcher`, `RandomizedLoggingPolicy`, `OffPolicyStats`, `OffPolicyEvaluator`, `AntiWindupController`, `PidController`, `PidLatencyController`, `LatencyBudgetGuard` *(+5 weitere)*

#### `crates/contextra-adapt/src/bandit.rs` (28)

- **From<BanditError> for contextra_types::ContextraError**: `from` (`priv`, L41)
- **std::ops::Deref for AlignedF32Vec**: `deref` (`priv`, L81)
- **std::ops::DerefMut for AlignedF32Vec**: `deref_mut` (`priv`, L87)
- **AlignedF32Vec**: `new` (`pub`, L94), `zeros` (`pub`, L99)
- **SketchMatrix**: `from_tenant_seed` (`pub`, L141), `derive_seed` (`pub`, L152), `from_seed` (`pub`, L163), `project` (`pub`, L193)
- **trait BanditPolicy**: `apply_drift_penalty` (`priv`, L205)
- ***(Modulfunktionen)***: `default_gamma` (`priv`, L208), `default_drift_gamma` (`priv`, L212), `default_drift_decay_window` (`priv`, L216), `default_alpha_base` (`priv`, L220), `default_alpha_max_multiplier` (`priv`, L224), `dot_product_f32` (`priv`, L233), `fused_row_update` (`priv`, L282)
- **BanditPolicy for BanditProfileState**: `apply_drift_penalty` (`priv`, L378)
- **BanditProfileState**: `cold_start` (`pub`, L396), `ensure_sketched_state` (`pub`, L424), `init_tenant_sketch` (`pub`, L470), `ensure_inv_a` (`priv`, L480), `ensure_work_buf` (`priv`, L492), `expected_dim` (`pub`, L500), `score` (`pub`, L512), `update` (`pub`, L644), `on_drift_detected` (`pub`, L786), `observe_and_adapt_drift` (`pub`, L791)

#### `crates/contextra-adapt/src/decay_controller.rs` (8)

- **Default for DecaySignalInputs**: `default` (`priv`, L34)
- **Default for DecayControllerConfig**: `default` (`priv`, L56)
- **AdaptiveDecayController**: `new` (`pub`, L73), `with_defaults` (`pub`, L78), `system_temperature` (`pub`, L84), `effective_half_life` (`pub`, L91), `effective_score` (`pub`, L98), `should_evict` (`pub`, L114)

#### `crates/contextra-adapt/src/drift.rs` (15)

- **trait DriftDetector**: `observe` (`priv`, L21), `is_drifting` (`priv`, L23)
- **Default for CatoniDriftDetector**: `default` (`priv`, L46)
- **CatoniDriftDetector**: `new` (`pub`, L53), `reset` (`pub`, L64)
- **DriftDetector for CatoniDriftDetector**: `observe` (`priv`, L71), `is_drifting` (`priv`, L92)
- **Default for EnsembleDriftWatcher**: `default` (`priv`, L110)
- **EnsembleDriftWatcher**: `new` (`pub`, L120), `observe_and_decide` (`pub`, L125), `observe_and_react_policy` (`pub`, L133), `set_baseline` (`pub`, L150)
- **Default for DriftPolicyBridge**: `default` (`priv`, L176)
- **DriftPolicyBridge**: `new` (`pub`, L188), `observe_and_react` (`pub`, L200)

#### `crates/contextra-adapt/src/flow_thompson.rs` (39)

- **From<FcTsError> for contextra_types::ContextraError**: `from` (`priv`, L30)
- **trait FcTsRng**: `next_u64` (`priv`, L50), `next_f32` (`priv`, L53), `next_standard_normal` (`priv`, L59)
- **SplitMix64**: `new` (`pub`, L77)
- **FcTsRng for SplitMix64**: `next_u64` (`priv`, L83)
- **Default for FcTsConfig**: `default` (`priv`, L112)
- **FcTsConfig**: `validate` (`pub`, L127)
- ***(Modulfunktionen)***: `ring3_background_task_token` (`pub`, L176)
- **std::fmt::Debug for FlowCorrectedThompsonBandit**: `fmt` (`priv`, L219)
- **FlowCorrectedThompsonBandit**: `new` (`pub`, L236), `with_shadow_sink` (`pub`, L268), `shadow_sink` (`pub`, L277), `config` (`pub`, L282), `mu` (`pub`, L287), `drift_rate` (`pub`, L292), `inv_a` (`pub`, L297), `update_with_flow` (`pub`, L312), `sample_score` (`pub`, L425), `baseline_score` (`pub`, L456), `select_arm` (`pub`, L482), `recompute_drift_rate_from_window` (`pub`, L509), `recompute_drift_rate_in_background` (`pub`, L626)
- **DiagonalApproximationBandit**: `new` (`pub`, L641), `from_pairs` (`pub`, L648), `propensity` (`pub`, L657)
- **trait OffPolicyCompatibility**: `verify_positivity` (`priv`, L668)
- **From<OffPolicyError> for contextra_types::ContextraError**: `from` (`priv`, L683)
- **FcTsSamplingDistribution**: `new` (`pub`, L697), `from_pairs` (`pub`, L704), `probability` (`pub`, L713)
- **OffPolicyCompatibility for FcTsSamplingDistribution**: `verify_positivity` (`priv`, L719)
- **FcTsArmSet**: `with_shadow_sink` (`pub`, L744), `select_arm` (`pub`, L759), `update_arm` (`pub`, L807), `recompute_drift_rates_in_background` (`pub`, L825)
- **OffPolicyCompatibility for FcTsArmSet**: `verify_positivity` (`priv`, L834)
- **trait BanditDefaultFlipGate**: `should_flip` (`priv`, L883)
- **BanditDefaultFlipGate for DefaultBanditFlipGate**: `should_flip` (`priv`, L891)

#### `crates/contextra-adapt/src/homeostat.rs` (13)

- **Default for RerankPidController**: `default` (`priv`, L26)
- **RerankPidController**: `new` (`pub`, L34), `kp` (`pub`, L51), `ki` (`pub`, L56), `kd` (`pub`, L61), `target_p95_latency_ms` (`pub`, L66), `k_pool` (`pub`, L71), `k_min` (`pub`, L78), `k_max` (`pub`, L83), `update` (`pub`, L88)
- ***(Modulfunktionen)***: `pid_regulated_candidate_pool` (`pub`, L112)
- **RerankDeadline**: `new` (`pub`, L134), `deadline_exceeded` (`pub`, L139)

#### `crates/contextra-adapt/src/lyapunov.rs` (10)

- **LyapunovResult**: `is_drift_detected` (`pub`, L77), `apply_drift_if_detected` (`pub`, L84)
- **Default for LyapunovDriftWatcher**: `default` (`priv`, L114)
- **LyapunovDriftWatcher**: `new` (`pub`, L121), `observe_score` (`pub`, L134), `analyze` (`pub`, L139), `latest_result` (`pub`, L146), `status_str` (`pub`, L151), `set_baseline` (`pub`, L168), `update` (`pub`, L176)

#### `crates/contextra-adapt/src/offpolicy.rs` (10)

- **RandomizedLoggingPolicy**: `new` (`pub`, L31), `compute_propensities` (`pub`, L39), `select_action` (`pub`, L53)
- **OffPolicyEvaluator**: `new` (`pub`, L94), `observe` (`pub`, L112), `estimate` (`pub`, L136), `cumulative_ips` (`pub`, L141), `samples` (`pub`, L146), `discarded` (`pub`, L151), `stats` (`pub`, L156)

#### `crates/contextra-adapt/src/pid.rs` (7)

- **trait AntiWindupController**: `update_with_anti_windup` (`priv`, L49)
- **Default for PidController**: `default` (`priv`, L85)
- **PidController**: `new` (`pub`, L99), `update` (`pub`, L125), `current_pool_size` (`pub`, L157), `reset` (`pub`, L162)
- **AntiWindupController for PidController**: `update_with_anti_windup` (`priv`, L170)

#### `crates/contextra-adapt/src/pid_latency_controller.rs` (11)

- **Default for PidLatencyController**: `default` (`priv`, L53)
- **PidLatencyController**: `new` (`pub`, L60), `with_params` (`pub`, L78), `observe_and_adjust` (`pub`, L100), `scale_pool_size` (`pub`, L106), `compute_adjustment` (`pub`, L122), `reset` (`pub`, L160)
- **LatencyBudgetGuard**: `new` (`pub`, L175), `elapsed_ms` (`pub`, L190), `is_exceeded` (`pub`, L195), `budget_ms` (`pub`, L200)

#### `crates/contextra-adapt/src/rie_greedy.rs` (8)

- **From<RieGreedyError> for contextra_types::ContextraError**: `from` (`priv`, L30)
- **RieGreedyProfile**: `new` (`pub`, L67), `mu_hat` (`pub`, L96), `predict` (`pub`, L114), `update` (`pub`, L142), `trace_precision_inv` (`pub`, L215), `uncertainty` (`pub`, L225), `confidence_score` (`pub`, L234)

#### `crates/contextra-adapt/src/shadow_mode.rs` (2)

- **trait ShadowSink**: `record` (`priv`, L23)
- **ShadowSink<T> for TracingShadowSink**: `record` (`priv`, L32)

### contextra-agent

> Persistent agent workflow engine for Contextra — checkpoint/execute/audit loop

- **Pfad:** `crates/contextra-agent` | **Dateien:** 13 | **Produktiv-Funktionen:** 151 (`88` `pub`, `63` intern) | **Tests:** 40
- **Öffentliche Typen (35)**: `AuditEntry`, `MigrationStats`, `AuditLog`, `InMemoryStorageEngine`, `ContextEditAuditRecord`, `ContextEditAuditSink`, `NoopContextEditAuditSink`, `CountingContextEditAuditSink`, `ScratchpadCacheInvalidator`, `NoopScratchpadCacheInvalidator`, `CountingScratchpadCacheInvalidator`, `PinnedRegionId`, `ScratchpadEditOp`, `ScratchpadCheckpoint`, `ClmScratchpad`, `AgentStatus`, `AgentContext`, `AgentEngine`, `DeadLetterQueue`, `EventLoopExitReason`, `OrchestratorEngine`, `BackgroundEvent`, `EventSource`, `PollingDocumentEventSource`, `VecEventSource`, `GoalCondition`, `NodeId`, `NodeType`, `AgentNode`, `WorkflowEdge`, `StateGraph`, `StepDeadLetter`, `DeadLetterReason`, `StepResult`, `AgentTool`

#### `crates/contextra-agent/src/audit.rs` (24)

- ***(Modulfunktionen)***: `validate_audit_payload_and_error` (`pub`, L50), `migrate_legacy_audit_entries` (`pub`, L145)
- **AuditLog<S>**: `new` (`pub`, L86), `append_to` (`pub`, L91), `append` (`pub`, L107), `migrate_legacy_entries` (`pub`, L112), `replay_task` (`pub`, L117)
- **InMemoryStorageEngine**: `new` (`pub`, L566)
- **StorageEngine for InMemoryStorageEngine**: `get` (`priv`, L575), `get_at_seq` (`priv`, L584), `put` (`priv`, L592), `put_if_absent` (`priv`, L607), `delete` (`priv`, L626), `commit` (`priv`, L640), `rollback` (`priv`, L644), `rollback_to_tx` (`priv`, L648), `flush` (`priv`, L652), `stats` (`priv`, L656), `last_seq_no` (`priv`, L666), `last_tx_id` (`priv`, L670), `pin_checkpoint` (`priv`, L674), `unpin_checkpoint` (`priv`, L678), `scan_prefix` (`priv`, L682), `scan` (`priv`, L699)

#### `crates/contextra-agent/src/budget.rs` (1)

- ***(Modulfunktionen)***: `reserve_tokens` (`pub`, L17)

#### `crates/contextra-agent/src/clm_scratchpad.rs` (29)

- **trait ContextEditAuditSink**: `record_edit` (`priv`, L41)
- **ContextEditAuditSink for NoopContextEditAuditSink**: `record_edit` (`priv`, L49)
- **CountingContextEditAuditSink**: `new` (`pub`, L59), `count` (`pub`, L65)
- **ContextEditAuditSink for CountingContextEditAuditSink**: `record_edit` (`priv`, L71)
- **trait ScratchpadCacheInvalidator**: `invalidate_scratchpad_scope` (`priv`, L78)
- **ScratchpadCacheInvalidator for NoopScratchpadCacheInvalidator**: `invalidate_scratchpad_scope` (`priv`, L86)
- **CountingScratchpadCacheInvalidator**: `new` (`pub`, L102), `count` (`pub`, L108)
- **ScratchpadCacheInvalidator for CountingScratchpadCacheInvalidator**: `invalidate_scratchpad_scope` (`priv`, L114)
- **PinnedRegionId**: `new` (`pub`, L132), `as_str` (`pub`, L137)
- ***(Modulfunktionen)***: `clone_chunk` (`priv`, L201)
- **ClmScratchpad**: `new_scoped` (`pub`, L216), `new` (`pub`, L237), `with_tenant_id` (`pub`, L256), `with_clock` (`pub`, L262), `with_audit_sink` (`pub`, L268), `with_cache_invalidator` (`pub`, L274), `with_pinned_region` (`pub`, L283), `with_created_at_tx` (`pub`, L291), `task_id` (`pub`, L297), `tenant_id` (`pub`, L302), `current_subgoal_index` (`pub`, L307), `created_at_tx` (`pub`, L312), `pinned_regions` (`pub`, L317), `apply_edit` (`pub`, L328), `checkpoint_and_reset` (`pub`, L464), `total_bytes` (`pub`, L501)

#### `crates/contextra-agent/src/context.rs` (15)

- ***(Modulfunktionen)***: `validate_task_id` (`pub`, L27), `validate_node_id` (`pub`, L49)
- **AgentContext**: `try_new` (`pub`, L110), `with_clm_scratchpad` (`pub`, L151), `set_cache_directive` (`pub`, L157), `get_cache_directive` (`pub`, L162), `set_step_cache_directive` (`pub`, L167), `directive_for_node` (`pub`, L173), `eval_directive_for_node` (`pub(crate)`, L177), `attach_event` (`pub`, L191), `next_retry_step_count` (`pub`, L202), `try_attach_event` (`pub`, L207)
- **AgentEngine**: `new` (`pub`, L230), `directive_for_step` (`pub`, L241), `run` (`pub`, L246)

#### `crates/contextra-agent/src/dlq.rs` (13)

- ***(Modulfunktionen)***: `extract_tx_id` (`priv`, L19), `check_json_for_tx_id` (`priv`, L24)
- **DeadLetterQueue**: `new` (`pub`, L51), `with_metrics_sink` (`pub`, L59), `new_with_metrics` (`pub`, L64), `drain_uncommitted` (`pub`, L73), `report_depth` (`priv`, L88), `push` (`pub`, L93), `drain` (`pub`, L118), `list` (`pub`, L153), `remove` (`pub`, L171), `is_already_committed` (`pub`, L198), `allocate_tx` (`pub`, L234)

#### `crates/contextra-agent/src/engine.rs` (27)

- **OrchestratorEngine**: `try_new` (`pub`, L45), `with_clock` (`pub`, L56), `with_metrics_sink` (`pub`, L62), `now_secs` (`priv`, L69), `now_millis` (`priv`, L73), `try_from_db` (`pub`, L78), `from_db` (`pub`, L84), `new` (`pub`, L89)
- **CheckpointRegistry for FallbackRegistry**: `save_checkpoint` (`priv`, L92), `load_checkpoint` (`priv`, L98), `list_checkpoints` (`priv`, L104)
- **contextra_ports::Checkpoint for FallbackRegistry**: `take_snapshot` (`priv`, L111), `restore` (`priv`, L123)
- ***(Modulfunktionen)***: `try_register_tool` (`pub`, L143), `recover_orphans` (`pub`, L167), `run` (`pub`, L173), `run_internal` (`priv`, L182), `replay_from` (`pub`, L539), `checkpoint` (`pub`, L647), `run_event_loop` (`pub`, L677), `commit_step` (`priv`, L717), `audit_log` (`priv`, L740), `audit_log_failure` (`priv`, L756), `persist_final_state` (`priv`, L771), `resolve_outgoing_edge` (`priv`, L785), `resolve_next_node` (`priv`, L810), `evaluate_decision` (`priv`, L821)

#### `crates/contextra-agent/src/engine/condition.rs` (3)

- ***(Modulfunktionen)***: `get_context_value` (`priv`, L3), `value_matches` (`priv`, L61), `evaluate_condition_expr` (`pub`, L94)

#### `crates/contextra-agent/src/engine/tests.rs` (5)

- ***(Modulfunktionen)***: `create_dummy_context` (`priv`, L8)
- **AgentTool for MockTool**: `name` (`priv`, L126), `execute` (`priv`, L130)
- **AgentTool for GoalMatchingTool**: `name` (`priv`, L195), `execute` (`priv`, L199)

#### `crates/contextra-agent/src/event_source.rs` (15)

- **BackgroundEvent**: `try_new` (`pub`, L35), `new` (`pub`, L66)
- **trait EventSource**: `next_event` (`priv`, L92), `is_exhausted` (`priv`, L96), `wait_for_event` (`priv`, L104)
- **PollingDocumentEventSource<S>**: `new` (`pub`, L124), `new_from_seq` (`pub`, L129), `with_capacity` (`pub`, L139), `with_last_seen_seq` (`pub`, L152), `poll_interval` (`pub`, L157)
- **EventSource for PollingDocumentEventSource<S>**: `next_event` (`priv`, L163), `wait_for_event` (`priv`, L221)
- **VecEventSource**: `try_new` (`pub`, L235)
- **EventSource for VecEventSource**: `next_event` (`priv`, L250), `is_exhausted` (`priv`, L254)

#### `crates/contextra-agent/src/goal_condition.rs` (4)

- **GoalCondition**: `evaluate` (`pub`, L62), `describe` (`pub`, L101)
- ***(Modulfunktionen)***: `resolve_json_path` (`priv`, L139), `parse_legacy_condition_string` (`pub`, L167)

#### `crates/contextra-agent/src/graph.rs` (9)

- **WorkflowEdge**: `has_condition` (`pub`, L56), `evaluate_condition` (`pub`, L74)
- **StateGraph**: `new` (`pub`, L108), `try_add_node` (`pub`, L114), `add_node` (`pub`, L175), `try_add_edge` (`pub`, L189), `try_add_edge_with_goal` (`pub`, L227), `add_edge` (`pub`, L249), `get_node` (`pub`, L255)

#### `crates/contextra-agent/src/step.rs` (6)

- **trait AgentTool**: `name` (`priv`, L64), `estimated_cost` (`priv`, L69), `execute` (`priv`, L73), `timeout_ms` (`priv`, L81), `is_retriable` (`priv`, L86), `max_retries` (`priv`, L91)

### contextra-audit-export

> GDPR Article 30 Processing Register export generator for Contextra

- **Pfad:** `crates/contextra-audit-export` | **Dateien:** 5 | **Produktiv-Funktionen:** 19 (`15` `pub`, `4` intern) | **Tests:** 2
- **Öffentliche Typen (9)**: `BsiMappingEntry`, `AuditExportError`, `EgressEventSummary`, `DeletionProofSummary`, `ProcessingRegisterEntry`, `Art30ProcessingRegisterEntry`, `ProcessingRegisterSource`, `AuditRegisterExporter`, `InMemoryProcessingRegisterSource`

#### `crates/contextra-audit-export/src/bsi_mapping.rs` (3)

- ***(Modulfunktionen)***: `bsi_mapping_table` (`pub`, L36), `sanitize_markdown_cell` (`priv`, L66), `render_bsi_mapping_markdown` (`pub`, L76)

#### `crates/contextra-audit-export/src/lib.rs` (9)

- **trait ProcessingRegisterSource**: `collect_entries` (`priv`, L102)
- ***(Modulfunktionen)***: `render_register_json` (`pub`, L109)
- **AuditRegisterExporter<S>**: `new` (`pub`, L123), `source` (`pub`, L128), `source_mut` (`pub`, L133), `export_json` (`pub`, L152), `export_markdown` (`pub`, L158)
- **AuditRegisterExporter<testkit::InMemoryProcessingRegisterSource>**: `with_sample_data_for` (`pub`, L140), `add_entry` (`pub`, L145)

#### `crates/contextra-audit-export/src/markdown_template.rs` (3)

- ***(Modulfunktionen)***: `sanitize_markdown_cell` (`priv`, L6), `render_register_markdown` (`pub`, L24), `render_art30_register_markdown` (`pub`, L67)

#### `crates/contextra-audit-export/src/testkit.rs` (4)

- **InMemoryProcessingRegisterSource**: `new` (`pub`, L17), `with_sample_data_for` (`pub`, L22), `add_entry` (`pub`, L64)
- **ProcessingRegisterSource for InMemoryProcessingRegisterSource**: `collect_entries` (`priv`, L70)

### contextra-avv-generator

> AVV (Auftragsverarbeitungsvertrag) template generator referencing technical guarantees for Contextra

- **Pfad:** `crates/contextra-avv-generator` | **Dateien:** 3 | **Produktiv-Funktionen:** 17 (`16` `pub`, `1` intern) | **Tests:** 0
- **Öffentliche Typen (4)**: `AvvGeneratorError`, `AvvContext`, `AvvContextBuilder`, `TechnicalMeasure`

#### `crates/contextra-avv-generator/src/lib.rs` (15)

- **AvvContext**: `new` (`pub`, L35), `default_technical_measures` (`pub`, L51), `builder` (`pub`, L57), `render` (`pub`, L66)
- **AvvContextBuilder**: `new` (`pub`, L79), `with_technical_measures` (`pub`, L90), `with_default_technical_measures` (`pub`, L96), `add_technical_measure` (`pub`, L102), `with_subprocessors` (`pub`, L108), `add_subprocessor` (`pub`, L114), `with_deletion_sla_days` (`pub`, L120), `build` (`pub`, L126), `render` (`pub`, L131)
- ***(Modulfunktionen)***: `render_avv_markdown` (`pub`, L148), `default_technical_measures` (`pub`, L153)

#### `crates/contextra-avv-generator/src/template.rs` (2)

- ***(Modulfunktionen)***: `render` (`pub`, L8), `format_technical_measure` (`priv`, L152)

### contextra-checkpoint

> Backup and snapshot management for Contextra storage

- **Pfad:** `crates/contextra-checkpoint` | **Dateien:** 11 | **Produktiv-Funktionen:** 180 (`101` `pub`, `79` intern) | **Tests:** 92
- **Öffentliche Typen (18)**: `PinGuard`, `CheckpointGuard`, `LoomReadDir`, `LoomDirEntry`, `LoomMetadata`, `HardlinkCloneResult`, `CheckpointHardlinkCloner`, `DefaultHardlinkCloner`, `CheckpointManifest`, `CheckpointMeta`, `StateCheckpoint`, `PinId`, `OrphanRegistry`, `PinnedSeqNoOrphan`, `OrphanState`, `InstanceOrphanRegistry`, `CheckpointRegistry`, `PersistentCheckpointStore`

#### `crates/contextra-checkpoint/src/guard.rs` (22)

- **PinGuard<S>**: `pin` (`pub`, L20), `pin_with_clock` (`pub`, L29), `clock` (`pub`, L44), `defuse` (`pub`, L49), `unpin` (`pub`, L54)
- **Drop for PinGuard<S>**: `drop` (`priv`, L66)
- **CheckpointGuard<S>**: `new` (`pub`, L102), `with_clock` (`pub`, L107), `with_registry` (`pub`, L127), `with_registry_and_counter` (`pub`, L145), `with_registry_counter_and_clock` (`pub`, L163), `with_metrics_sink` (`pub`, L183), `with_metrics_sink_opt` (`pub`, L188), `clock` (`pub`, L196), `for_agent_step` (`pub`, L201), `for_agent_step_with_registry` (`pub`, L206), `for_agent_step_with_clock` (`pub`, L225), `checkpoint` (`pub`, L242), `commit` (`pub`, L248), `rollback` (`pub`, L278), `rollback_blocking` (`pub`, L315)
- **Drop for CheckpointGuard<S>**: `drop` (`priv`, L361)

#### `crates/contextra-checkpoint/src/guard/tests.rs` (16)

- **MockStorage**: `new` (`priv`, L13)
- **StorageEngine for MockStorage**: `get` (`priv`, L23), `get_at_seq` (`priv`, L26), `put` (`priv`, L33), `delete` (`priv`, L44), `commit` (`priv`, L50), `rollback` (`priv`, L53), `rollback_to_tx` (`priv`, L56), `last_seq_no` (`priv`, L62), `last_tx_id` (`priv`, L65), `flush` (`priv`, L68), `stats` (`priv`, L71), `pin_checkpoint` (`priv`, L80), `unpin_checkpoint` (`priv`, L86), `scan_prefix` (`priv`, L92), `scan` (`priv`, L105)

#### `crates/contextra-checkpoint/src/hardlink_cloner.rs` (11)

- ***(Modulfunktionen)***: `create_dir_all` (`pub`, L35), `read_dir` (`pub`, L39), `metadata` (`pub`, L43), `remove_file` (`pub`, L50), `hard_link` (`pub`, L54), `is_cross_device_error` (`priv`, L196)
- **LoomReadDir**: `next_entry` (`pub`, L63)
- **LoomDirEntry**: `path` (`pub`, L70)
- **trait CheckpointHardlinkCloner**: `clone_sstable_hardlinks` (`priv`, L104)
- **DefaultHardlinkCloner**: `new` (`pub`, L119)
- **CheckpointHardlinkCloner for DefaultHardlinkCloner**: `clone_sstable_hardlinks` (`priv`, L125)

#### `crates/contextra-checkpoint/src/manifest.rs` (2)

- **CheckpointManifest**: `new` (`pub`, L17), `verify` (`pub`, L37)

#### `crates/contextra-checkpoint/src/meta.rs` (2)

- ***(Modulfunktionen)***: `validate_identifier` (`pub(crate)`, L6)
- **CheckpointMeta**: `into_workflow_state` (`pub`, L33)

#### `crates/contextra-checkpoint/src/orphan.rs` (45)

- ***(Modulfunktionen)***: `warn_deprecated_global_orphan_path` (`priv`, L14), `global_orphan_registry` (`pub`, L28), `default_data_dir` (`priv`, L41), `monotonic_timestamp_ms_with` (`pub`, L46), `monotonic_timestamp_ms` (`pub`, L51), `clock_timestamp_ms` (`pub(crate)`, L56), `orphan_pin_file_path` (`priv`, L60), `orphan_file_path` (`priv`, L119), `register_pinned_seq_no_orphan` (`pub`, L138), `register_orphaned_checkpoint` (`pub`, L458), `get_orphaned_checkpoints` (`pub`, L471), `get_orphaned_checkpoints_for_namespace` (`pub`, L477), `clear_orphaned_checkpoint` (`pub`, L490), `clear_all_orphaned_checkpoints` (`pub`, L498), `await_pending_rollbacks` (`pub`, L501), `pending_rollback_count` (`pub`, L504), `orphaned_checkpoint_count` (`pub`, L510)
- **Default for OrphanRegistry**: `default` (`priv`, L78)
- **OrphanRegistry**: `new` (`pub`, L85), `register_orphan` (`pub`, L92), `get_orphans` (`pub`, L97), `clear_all` (`pub`, L106), `recover_and_clean` (`pub`, L111)
- **OrphanState**: `persist_sync` (`pub`, L155), `load_sync` (`pub`, L163)
- **std::fmt::Debug for InstanceOrphanRegistry**: `fmt` (`priv`, L187)
- **Default for InstanceOrphanRegistry**: `default` (`priv`, L199)
- **InstanceOrphanRegistry**: `new` (`pub`, L205), `with_clock` (`pub`, L210), `set_clock` (`pub`, L215), `clock` (`pub`, L219), `load_sync` (`pub`, L223), `persist_sync` (`pub`, L247), `register_orphan` (`pub`, L270), `register_orphan_sync` (`pub`, L288), `recover_and_clean` (`pub`, L303), `register_checkpoint_sync` (`pub`, L331), `flush_orphan_registry` (`pub`, L346), `drain_orphan_pins` (`pub`, L372), `get_orphan_pins` (`pub`, L388), `clear_orphan_pin` (`pub`, L392), `drain_orphaned_checkpoints` (`pub`, L408), `get_orphaned_checkpoints` (`pub`, L424), `clear_orphaned_checkpoint` (`pub`, L428), `clear_all` (`pub`, L440)

#### `crates/contextra-checkpoint/src/store.rs` (66)

- **trait CheckpointRegistry**: `save_checkpoint` (`priv`, L22), `load_checkpoint` (`priv`, L23), `list_checkpoints` (`priv`, L24), `orphan_registry` (`priv`, L26), `register_orphan` (`priv`, L30), `create_hardlink_clone` (`priv`, L38), `recover_orphaned_pins` (`priv`, L53), `recover_orphaned_checkpoints` (`priv`, L57), `flush_orphan_registry` (`priv`, L61)
- **CheckpointIndex**: `insert` (`priv`, L83), `remove_by_name` (`priv`, L92), `remove_by_seq` (`priv`, L100), `get_by_name` (`priv`, L111), `get_by_seq` (`priv`, L117), `clear` (`priv`, L121)
- ***(Modulfunktionen)***: `persist_hwm_internal` (`priv`, L135), `create_named_checkpoint` (`priv`, L877), `restore_named_checkpoint` (`priv`, L889), `drop_named_checkpoint` (`priv`, L893), `list_named_checkpoints` (`priv`, L897), `take_snapshot` (`priv`, L905), `restore` (`priv`, L915)
- **PersistentCheckpointStore<S>**: `open` (`pub`, L187), `open_with_orphan_registry` (`pub`, L195), `new` (`pub`, L290), `new_with_orphan_registry` (`pub`, L297), `allocate_tx` (`pub`, L360), `skipped_rollback_count` (`pub`, L382), `checkpoint_guard_skipped_rollback_count` (`pub`, L386), `checkpoint_counter` (`pub`, L390), `with_clock` (`pub`, L398), `clock` (`pub`, L406), `set_metrics_sink` (`pub`, L410), `with_metrics_sink` (`pub`, L414), `monotonic_timestamp_ms` (`pub`, L419), `create_guard` (`pub`, L429), `create_checkpoint` (`pub`, L447), `drop_checkpoint` (`pub`, L537), `save_checkpoint_internal` (`priv`, L576), `get_checkpoint_internal` (`priv`, L602), `list_checkpoints` (`pub`, L628), `get_checkpoint` (`pub`, L661), `restore_checkpoint` (`pub`, L672), `orphan_registry` (`pub`, L700), `flush_orphan_registry` (`pub`, L704), `shutdown` (`pub`, L712), `close` (`pub`, L717), `register_pinned_seq_no_orphan` (`pub`, L721), `register_orphaned_checkpoint` (`pub`, L725), `get_orphaned_checkpoints` (`pub`, L729), `clear_orphaned_checkpoint` (`pub`, L733), `clear_all_orphaned_checkpoints` (`pub`, L737), `recover_orphaned_pins` (`pub`, L743), `drain_orphaned_checkpoints` (`pub`, L759), `drain_orphan_pins` (`pub`, L764), `recover_orphaned_checkpoints` (`pub`, L770), `create_hardlink_clone` (`pub`, L801)
- **CheckpointRegistry for PersistentCheckpointStore<S>**: `save_checkpoint` (`priv`, L816), `load_checkpoint` (`priv`, L823), `list_checkpoints` (`priv`, L834), `orphan_registry` (`priv`, L838), `register_orphan` (`priv`, L842), `create_hardlink_clone` (`priv`, L846), `recover_orphaned_pins` (`priv`, L859), `recover_orphaned_checkpoints` (`priv`, L863), `flush_orphan_registry` (`priv`, L867)

#### `crates/contextra-checkpoint/src/store/tests.rs` (16)

- **MockStorage**: `new` (`priv`, L17)
- **StorageEngine for MockStorage**: `get` (`priv`, L28), `get_at_seq` (`priv`, L31), `put` (`priv`, L38), `delete` (`priv`, L54), `commit` (`priv`, L60), `rollback` (`priv`, L63), `rollback_to_tx` (`priv`, L66), `last_seq_no` (`priv`, L72), `last_tx_id` (`priv`, L75), `flush` (`priv`, L78), `stats` (`priv`, L81), `pin_checkpoint` (`priv`, L90), `unpin_checkpoint` (`priv`, L96), `scan_prefix` (`priv`, L102), `scan` (`priv`, L115)

### contextra-cognition

> Contextra — Memory consolidation, compaction, and context management

- **Pfad:** `crates/contextra-cognition` | **Dateien:** 19 | **Produktiv-Funktionen:** 126 (`76` `pub`, `50` intern) | **Tests:** 69
- **Öffentliche Typen (36)**: `AggregationConfig`, `AggregationPhaseResult`, `ConsolidationPipelineResult`, `AggregationNode`, `AggregationEdge`, `SuperEdgeDraft`, `AlphaNode`, `SuperEdgeSink`, `ConsolidationLockGuard`, `GroundingValidatorAdapter`, `ConsolidationEngine`, `ConsolidationNodesGuard`, `ContextManager`, `SpatialFence`, `ContextCompactor`, `ConsolidationSession`, `CompactionStrategy`, `CompactedContext`, `StatusToken`, `CsrGraphSuperEdgeSink`, `LeanRagInputs`, `PercolationConfig`, `MaintenanceConfig`, `MaintenanceScheduler`, `ActiveSessionGuard`, `ConsolidationConfig`, `TurnSegment`, `ConsolidationPhaseResult`, `SynthesisConfig`, `CommunityStabilityTracker`, `MetaChunk`, `SynthesisPhaseResult`, `PreCondensedInput`, `SegmentSynthesisResult`, `SynthesisPhaseResult`, `SynthesizedChunk`

#### `crates/contextra-cognition/src/aggregation_phase.rs` (12)

- **Default for AggregationConfig**: `default` (`priv`, L41)
- **AggregationConfig**: `validate` (`pub`, L59)
- **trait SuperEdgeSink**: `tombstone_edge` (`priv`, L178), `write_super_edge` (`priv`, L180), `commit` (`priv`, L182)
- ***(Modulfunktionen)***: `check_compaction_budget` (`pub`, L186), `compute_entity_community_hash` (`pub`, L224), `run_aggregation_pass` (`pub`, L238), `run_deterministic_diagonal_gmm` (`priv`, L528)
- **SplitMix64**: `new` (`priv`, L203), `next_u64` (`priv`, L207), `next_f64` (`priv`, L215)

#### `crates/contextra-cognition/src/consolidation_executor.rs` (21)

- **ConsolidationLockGuard**: `try_acquire` (`pub`, L42)
- **Drop for ConsolidationLockGuard**: `drop` (`priv`, L55)
- ***(Modulfunktionen)***: `execute_consolidation_pass` (`pub`, L63), `execute_background_consolidation` (`pub`, L219), `execute_background_consolidation_with_rich_validator` (`pub`, L248), `execute_leanrag_aggregation_stage` (`pub`, L387), `execute_sleep_cycle` (`pub`, L403), `start_consolidation_worker` (`pub`, L424), `start_consolidation_worker_full` (`pub`, L441), `start_consolidation_reaper` (`pub`, L466)
- **ResponseGroundingValidator for GroundingValidatorAdapter<**: `score_grounding` (`priv`, L170)
- **ConsolidationEngine<S**: `new` (`pub`, L498), `with_llm` (`pub`, L520), `with_leanrag` (`pub`, L526), `with_validator` (`pub`, L532), `with_rich_validator` (`pub`, L538), `start` (`pub`, L544), `start_worker` (`pub`, L551), `start_worker_full` (`pub`, L569), `run` (`pub`, L599), `run_cycle` (`pub`, L632)

#### `crates/contextra-cognition/src/consolidation_locks.rs` (3)

- **ConsolidationNodesGuard<**: `try_acquire` (`pub`, L36), `collection` (`pub`, L48), `cascade_invalidate_edges_ordered` (`pub`, L57)

#### `crates/contextra-cognition/src/context.rs` (11)

- **Default for ContextManager**: `default` (`priv`, L34)
- **ContextManager**: `new` (`pub`, L41), `with_defaults` (`pub`, L49), `set_relevance_threshold` (`pub`, L54), `with_relevance_threshold` (`pub`, L59), `with_threshold` (`pub`, L65), `relevance_threshold` (`pub`, L71), `prepare_context` (`pub`, L78), `estimate_tokens` (`pub`, L152)
- **SpatialFence**: `new` (`pub`, L238), `matches` (`pub`, L246)

#### `crates/contextra-cognition/src/context_compaction/cleanup.rs` (1)

- ***(Modulfunktionen)***: `cleanup_orphaned_consolidation_intents` (`pub`, L10)

#### `crates/contextra-cognition/src/context_compaction/compactor.rs` (9)

- **std::fmt::Debug for ContextCompactor**: `fmt` (`priv`, L21)
- **ContextCompactor**: `new` (`pub`, L32), `with_rng` (`pub`, L42), `compact` (`pub`, L51), `consolidate_via_llm` (`pub`, L121), `is_tool_output` (`priv`, L207), `generate_status_token` (`priv`, L215), `consolidate_with_retry` (`pub`, L237), `consolidate_with_retry_with_rng` (`pub`, L262)

#### `crates/contextra-cognition/src/context_compaction/session.rs` (7)

- **ConsolidationSession<**: `start` (`pub`, L31), `validate_occ` (`pub`, L67), `refresh` (`pub`, L93), `abort` (`pub`, L135), `execute` (`pub`, L147), `commit` (`pub`, L216), `commit_ref` (`pub`, L228)

#### `crates/contextra-cognition/src/context_compaction/tests.rs` (20)

- ***(Modulfunktionen)***: `make_chunk` (`priv`, L13)
- **LlmTextGenerator for UnreachableLlmGenerator**: `generate` (`priv`, L129)
- **LlmTextGenerator for MockLlmGenerator**: `generate` (`priv`, L183)
- **StorageEngine for FaultyDeleteStorage**: `get` (`priv`, L248), `get_at_seq` (`priv`, L252), `put` (`priv`, L260), `put_if_absent` (`priv`, L264), `delete` (`priv`, L273), `commit` (`priv`, L284), `rollback` (`priv`, L288), `rollback_to_tx` (`priv`, L292), `flush` (`priv`, L296), `stats` (`priv`, L300), `last_seq_no` (`priv`, L304), `last_tx_id` (`priv`, L308), `pin_checkpoint` (`priv`, L312), `unpin_checkpoint` (`priv`, L316), `scan_prefix` (`priv`, L320), `scan_prefix_at` (`priv`, L327), `scan` (`priv`, L335)

#### `crates/contextra-cognition/src/graph_sink.rs` (4)

- **CsrGraphSuperEdgeSink<**: `new` (`pub`, L27)
- **SuperEdgeSink for CsrGraphSuperEdgeSink<**: `tombstone_edge` (`priv`, L38), `write_super_edge` (`priv`, L49), `commit` (`priv`, L75)

#### `crates/contextra-cognition/src/leanrag_input.rs` (1)

- ***(Modulfunktionen)***: `build_leanrag_inputs` (`pub`, L42)

#### `crates/contextra-cognition/src/maintenance_config.rs` (2)

- **Default for PercolationConfig**: `default` (`priv`, L21)
- **Default for MaintenanceConfig**: `default` (`priv`, L76)

#### `crates/contextra-cognition/src/maintenance_scheduler.rs` (16)

- **MaintenanceScheduler<S**: `new` (`pub`, L34), `with_clock` (`pub`, L51), `new_with_edge_reinforcement` (`pub`, L58), `with_edge_reinforcement_buffer` (`pub`, L69), `active_agent_sessions` (`pub`, L78), `increment_active_sessions` (`pub`, L83), `decrement_active_sessions` (`pub`, L88), `acquire_session_guard` (`pub`, L95), `start` (`pub`, L100), `run_tick` (`pub`, L133)
- ***(Modulfunktionen)***: `write_tick_intent` (`priv`, L307), `write_tick_intent_with_clock` (`priv`, L313), `complete_tick_intent` (`priv`, L331), `complete_tick_intent_with_clock` (`priv`, L337), `drop` (`priv`, L369)
- **ActiveSessionGuard<**: `new` (`priv`, L360)

#### `crates/contextra-cognition/src/memory_consolidation.rs` (15)

- **Default for ConsolidationConfig**: `default` (`priv`, L46)
- ***(Modulfunktionen)***: `cosine_similarity` (`pub(crate)`, L84), `group_turns_into_segments` (`pub`, L144), `detect_near_duplicates` (`pub`, L235), `run_consolidation_pass` (`pub`, L268), `compute_community_hash` (`pub`, L393), `run_structural_synthesis_pass` (`pub`, L409), `intake_pre_condensed_inputs` (`pub`, L565), `compact_segment_via_context_compactor` (`pub`, L689)
- **WorkingSegment**: `new` (`priv`, L109), `add_turn` (`priv`, L116)
- **Default for SynthesisConfig**: `default` (`priv`, L331)
- **CommunityStabilityTracker**: `new` (`pub`, L348), `observe` (`pub`, L355), `reset_if_absent` (`pub`, L362)

#### `crates/contextra-cognition/src/semantic_aggregation_facade.rs` (1)

- ***(Modulfunktionen)***: `consolidate_semantic_hyperedges` (`pub`, L40)

#### `crates/contextra-cognition/src/synthesis_phase.rs` (1)

- ***(Modulfunktionen)***: `run_synthesis_pass` (`pub`, L44)

#### `crates/contextra-cognition/src/transitivity_veto.rs` (2)

- ***(Modulfunktionen)***: `validate_transitivity_veto` (`pub`, L24), `filter_candidates_with_transitivity_veto` (`pub`, L59)

### contextra-core

> Deprecated Strangler Facade re-exporting Ring-0 types, traits, MVCC, and wire IPC for Contextra

- **Pfad:** `crates/contextra-core` | **Dateien:** 1 | **Produktiv-Funktionen:** 0 (`0` `pub`, `0` intern) | **Tests:** 0

*Keine direkt definierten produktiven Funktionen (reine Re-Export-Fassade oder Typdefinitionen).*

### contextra-crypto

> Encryption at Rest and KV-Cache Security utilities for Contextra

- **Pfad:** `crates/contextra-crypto` | **Dateien:** 18 | **Produktiv-Funktionen:** 213 (`168` `pub`, `45` intern) | **Tests:** 151
- **Öffentliche Typen (49)**: `VolatileEncryptionKey`, `DataClass`, `AuditChainEntry`, `AuditChainAnchor`, `AuditChainHeadSignature`, `EncryptedCommitmentSalt`, `AuditChain`, `KeyManager`, `GraphRepairAttestation`, `DeletionProofKeyPair`, `VerificationKey`, `LayerCleanupProof`, `DeletionLayer`, `ExcludedScope`, `DeletionScope`, `DeletionProof`, `Missing`, `Cleaned`, `DeletionProofBuilder`, `DeletionProofError`, `DeletionProofKeyPair`, `SignatureVersion`, `Result`, `CryptoError`, `KdfParams`, `KdfHeader`, `DerivedKey`, `EncryptedKvLayer`, `KvCipher`, `KvSegmentCipher`, `EvictionWorker`, `EncryptedSegmentPayload`, `KvSegment`, `SpillHandler`, `TenantIsolatedKvStore`, `SubKey`, `GroupKek`, `RecordDek`, `EncryptedRecordPayload`, `GroupRevocationListener` *(+9 weitere)*

#### `crates/contextra-crypto/src/anti_tamper.rs` (6)

- **VolatileEncryptionKey**: `new` (`pub`, L23), `emergency_wipe` (`pub`, L31), `as_bytes` (`pub`, L38), `inspect_key_bytes_for_test` (`pub`, L45)
- **std::fmt::Debug for VolatileEncryptionKey**: `fmt` (`priv`, L51)
- **PartialEq for VolatileEncryptionKey**: `eq` (`priv`, L61)

#### `crates/contextra-crypto/src/audit_chain.rs` (17)

- **AuditChainEntry**: `compute_hash` (`pub`, L67)
- **EncryptedCommitmentSalt**: `encrypt` (`pub`, L140), `decrypt` (`pub`, L155)
- ***(Modulfunktionen)***: `compute_record_commitment` (`pub`, L161)
- **AuditChain**: `new` (`pub`, L178), `append` (`pub`, L187), `len` (`pub`, L231), `is_empty` (`pub`, L236), `entries` (`pub`, L241), `head_hash` (`pub`, L246), `sign_head` (`pub`, L251), `verify_head_signature` (`pub`, L282), `verify_chain` (`pub`, L302), `anchor` (`pub`, L344), `verify_chain_against_anchor` (`pub`, L357), `create_signed_anchor` (`pub`, L375), `verify_signed_anchor` (`pub`, L387)

#### `crates/contextra-crypto/src/crypto.rs` (21)

- **std::fmt::Debug for KeyManager**: `fmt` (`priv`, L56)
- **KeyManager**: `try_new` (`pub`, L71), `try_new_with_kdf` (`pub`, L113), `try_new_argon2id` (`pub`, L137), `try_new_random_salt` (`pub`, L144), `derive_file_key` (`pub`, L154), `derive_segment_key_for_id` (`pub`, L178), `derive_segment_key` (`pub`, L192), `derive_segment_key_bytes` (`pub`, L197), `derive_kv_key` (`pub`, L229), `derive_kv_key_scoped` (`pub`, L242), `cipher_for` (`pub`, L300), `cipher_for_scoped` (`pub`, L307), `integrity_key` (`pub`, L324), `derive_deletion_proof_key` (`pub`, L334), `create_deletion_proof` (`pub`, L346), `cipher` (`priv`, L367), `encrypt_auto_nonce` (`pub`, L381), `decrypt_auto_nonce` (`pub`, L412), `emergency_wipe` (`pub`, L421), `inspect_key_bytes_for_test` (`pub`, L429)

#### `crates/contextra-crypto/src/deletion_proof.rs` (28)

- ***(Modulfunktionen)***: `hash_deleted_keys_length_prefixed` (`pub`, L52), `default_signature_version` (`priv`, L303), `compute_wal_delete_receipt` (`pub`, L789), `verify_wal_delete_receipt` (`pub`, L801), `compute_hmac_sha256` (`priv`, L812)
- **DeletionProofKeyPair**: `generate` (`pub`, L71), `verifying_key_bytes` (`pub`, L82), `signing_key` (`pub`, L87)
- **From<**: `from` (`priv`, L102), `from` (`priv`, L108), `from` (`priv`, L114), `from` (`priv`, L120)
- **LayerCleanupProof**: `new_after_verified_empty` (`pub`, L145), `verify_and_create` (`pub`, L172), `layer` (`pub`, L191)
- **DeletionProof**: `create` (`pub`, L314), `create_with_wal_receipt` (`pub`, L334), `create_v3` (`pub`, L397), `create_v3_with_audit_position` (`pub`, L422), `create_with_wal_receipt_v3` (`pub`, L449), `create_full_v3` (`pub`, L476), `signature_version_typed` (`pub`, L543), `verify` (`pub`, L552), `construct_v3_payload` (`priv`, L637), `verify_external` (`pub`, L716), `export_for_audit` (`pub`, L743), `tenant_id` (`pub`, L764), `verify_wal_receipt` (`pub`, L773)

#### `crates/contextra-crypto/src/deletion_proof_typestate.rs` (10)

- **DeletionProofBuilder<Missing**: `new` (`pub`, L91), `with_lsm_memtable_cleanup` (`pub`, L121)
- **DeletionProofBuilder<Lsm**: `with_wal_chain_receipt` (`pub`, L113), `with_sstable_all_levels_cleanup` (`pub`, L146), `with_hnsw_index_cleanup` (`pub`, L171), `with_wal_all_segments_cleanup` (`pub`, L196), `with_csr_graph_cleanup` (`pub`, L221), `with_kv_cache_segments_cleanup` (`pub`, L246), `with_embedding_cache_cleanup` (`pub`, L271)
- **DeletionProofBuilder<Cleaned**: `finish` (`pub`, L298)

#### `crates/contextra-crypto/src/ed25519_proof.rs` (9)

- **DeletionProofKeyPair**: `generate` (`pub`, L45), `verifying_key_bytes` (`pub`, L57), `signing_key` (`pub`, L62), `verifying_key` (`pub`, L67)
- ***(Modulfunktionen)***: `sign_deletion_proof_v3` (`pub`, L75), `verify_deletion_proof_v3` (`pub`, L90)
- **SignatureVersion**: `as_u8` (`pub`, L125)
- **From<SignatureVersion> for u8**: `from` (`priv`, L131)
- **TryFrom<u8> for SignatureVersion**: `try_from` (`priv`, L139)

#### `crates/contextra-crypto/src/error.rs` (2)

- **CryptoError**: `wal_corruption` (`pub`, L64)
- **From<CryptoError> for contextra_types::ContextraError**: `from` (`priv`, L73)

#### `crates/contextra-crypto/src/kdf.rs` (8)

- **Default for KdfParams**: `default` (`priv`, L54)
- **KdfParams**: `new` (`pub`, L65), `new_for_test` (`pub`, L90)
- **KdfHeader**: `generate_default` (`pub`, L110), `new` (`pub`, L122), `to_bytes` (`pub`, L148), `from_bytes` (`pub`, L165)
- ***(Modulfunktionen)***: `derive_key_argon2id` (`pub`, L256)

#### `crates/contextra-crypto/src/kv_cipher.rs` (13)

- **trait KvCipher**: `seal` (`priv`, L61), `open` (`priv`, L63)
- **KvCipher for KeyManager**: `seal` (`priv`, L67), `open` (`priv`, L77)
- **KvSegmentCipher**: `new` (`pub`, L108), `with_revocation_log` (`pub`, L116), `registry` (`pub`, L122), `encrypt` (`pub`, L130), `decrypt` (`pub`, L156), `encrypt_with_version` (`pub`, L171), `decrypt_with_version` (`pub`, L193)
- **KvCipher for KvSegmentCipher**: `seal` (`priv`, L212), `open` (`priv`, L216)

#### `crates/contextra-crypto/src/kv_segment/eviction_worker.rs` (5)

- **EvictionWorker**: `spawn` (`pub`, L37), `trigger_eviction` (`pub`, L68), `shutdown` (`pub`, L76)
- **Drop for EvictionWorker**: `drop` (`priv`, L85)
- ***(Modulfunktionen)***: `emergency_wipe` (`pub`, L93)

#### `crates/contextra-crypto/src/kv_segment/segment.rs` (9)

- **KvSegment**: `new` (`pub`, L60), `new_with_metadata` (`pub`, L76), `new_encrypted` (`pub`, L99), `decrypt_data` (`pub`, L133), `to_spill_bytes` (`pub`, L164), `as_bytes` (`pub`, L175), `len` (`pub`, L180), `is_empty` (`pub`, L185)
- **std::fmt::Debug for KvSegment**: `fmt` (`priv`, L191)

#### `crates/contextra-crypto/src/kv_segment/store.rs` (28)

- **TenantState**: `new` (`priv`, L29), `insert_returning_evicted` (`priv`, L38), `get_bytes` (`priv`, L50), `get_segment_ref_mut` (`priv`, L56), `remove` (`priv`, L61), `pop_lru` (`priv`, L68), `segment_ids` (`priv`, L75), `len` (`priv`, L79), `is_empty` (`priv`, L83)
- **Shard**: `new` (`priv`, L96)
- **TenantIsolatedKvStore**: `new` (`pub`, L127), `with_shard_count` (`pub`, L133), `with_capacity` (`pub`, L155), `set_spill_handler` (`pub`, L163), `shard_idx` (`priv`, L169), `insert_segment` (`pub`, L174), `on_rollback` (`pub`, L197), `remove_segments_for_rollback` (`pub`, L206), `remove_segment` (`pub`, L234), `get_segment_bytes` (`pub`, L239), `insert_encrypted_segment` (`pub`, L250), `get_segments` (`pub`, L273), `get_decrypted_segment` (`pub`, L285), `get_tenant_segment_len` (`pub`, L303), `evict_lru_fair` (`pub`, L382), `evict_lru_fair_internal` (`priv`, L401), `clear_all` (`pub`, L496)
- **Default for TenantIsolatedKvStore**: `default` (`priv`, L512)

#### `crates/contextra-crypto/src/kv_shredding.rs` (26)

- **Clone for SubKey**: `clone` (`priv`, L32)
- **std::fmt::Debug for SubKey**: `fmt` (`priv`, L38)
- **std::fmt::Debug for GroupKek**: `fmt` (`priv`, L50)
- **std::fmt::Debug for RecordDek**: `fmt` (`priv`, L62)
- ***(Modulfunktionen)***: `derive_subkey` (`pub`, L103)
- **trait GroupRevocationListener**: `on_group_revoked` (`priv`, L113)
- **std::fmt::Debug for KeyRegistry**: `fmt` (`priv`, L127)
- **KeyRegistry**: `new` (`pub`, L140), `new_in_memory` (`pub`, L151), `with_revocation_log` (`pub`, L164), `with_revocation_listener` (`pub`, L170), `get_wrapped_kek` (`pub`, L176), `get_wrapped_dek` (`pub`, L186), `is_group_revoked` (`pub`, L200), `was_group_ever_registered` (`pub`, L214), `get_or_derive` (`pub`, L224), `revoke_group` (`pub`, L279), `revoke_subkey` (`pub`, L316), `revoke_record` (`pub`, L322), `is_group_active` (`pub`, L347), `is_key_active` (`pub`, L357), `is_record_active` (`pub`, L362), `encrypt_record` (`pub`, L377), `decrypt_record` (`pub`, L432), `encrypt_with_group` (`pub`, L489), `decrypt_with_group` (`pub`, L517)

#### `crates/contextra-crypto/src/revocation_log.rs` (13)

- **RevocationTarget**: `canonical_bytes` (`pub`, L54)
- ***(Modulfunktionen)***: `serialize` (`pub`, L85), `deserialize` (`pub`, L92), `compute_entry_hash` (`pub`, L104)
- **RevocationLog**: `new_in_memory` (`pub`, L130), `open_or_create` (`pub`, L150), `verify_integrity` (`pub`, L193), `verify_chain_entries` (`priv`, L199), `append` (`pub`, L240), `is_revoked` (`pub`, L337), `len` (`pub`, L343), `is_empty` (`pub`, L348)
- **std::fmt::Debug for RevocationLog**: `fmt` (`priv`, L354)

#### `crates/contextra-crypto/src/wal_completeness.rs` (1)

- ***(Modulfunktionen)***: `verify_wal_chain_completeness` (`pub`, L36)

#### `crates/contextra-crypto/src/wal_crypto.rs` (17)

- **trait KmsProvider**: `get_key` (`priv`, L20)
- **EncryptedWal**: `new` (`pub`, L51), `encrypt_chunk` (`pub`, L71), `decrypt_chunk` (`pub`, L87)
- **WalHmac**: `new` (`pub`, L128), `update` (`pub`, L144), `finalize` (`pub`, L148)
- **IntegrityVerifier**: `new` (`pub`, L198), `set_last_hmac` (`pub`, L212), `last_hmac_snapshot` (`pub`, L217), `set_last_seq_no` (`pub`, L222), `last_seq_no_snapshot` (`pub`, L227), `handoff_from` (`pub`, L233), `verify_and_update_v3` (`pub`, L239), `verify_and_update_v2` (`pub`, L327), `skip_hmac_verify_legacy` (`pub`, L396), `verify_and_update` (`pub`, L402)

### contextra-db

> Contextra — Embedded hybrid-search for AI agents

- **Pfad:** `crates/contextra-db` | **Dateien:** 4 | **Produktiv-Funktionen:** 21 (`16` `pub`, `5` intern) | **Tests:** 53
- **Öffentliche Typen (11)**: `MultiStepConfig`, `MultiStepResult`, `MultiStepEngine`, `SignalModality`, `VaultChunk`, `VaultConfig`, `PurgeReceipt`, `CommitReceipt`, `VaultError`, `VolatileContextVault`, `VaultChunkMetadata`

#### `crates/contextra-db/src/multistep.rs` (7)

- **Default for MultiStepConfig**: `default` (`priv`, L36)
- **MultiStepEngine<S>**: `new` (`pub`, L73), `with_clock` (`pub`, L84), `search` (`pub`, L94), `search_with_embedder` (`pub`, L106), `quality_sufficient` (`priv`, L350)
- ***(Modulfunktionen)***: `deduplicate_search_results` (`priv`, L359)

#### `crates/contextra-db/src/volatile_vault.rs` (14)

- **VaultChunk**: `new` (`pub`, L50), `with_label` (`pub`, L60), `size_bytes` (`pub`, L66)
- **Default for VaultConfig**: `default` (`priv`, L83)
- **VolatileContextVault**: `open` (`pub`, L148), `ingest` (`pub`, L171), `purge` (`pub`, L213), `current_size_bytes` (`pub`, L248), `len` (`pub`, L253), `is_empty` (`pub`, L257), `is_memory_locked` (`pub`, L266), `preview_metadata` (`pub`, L272), `drain_for_commit` (`pub`, L290)
- **Drop for VolatileContextVault**: `drop` (`priv`, L312)

### contextra-durable-fs

> Crash-sichere Dateisystem-Operationen und synchrones Disk-Flushing für Contextra

- **Pfad:** `crates/contextra-durable-fs` | **Dateien:** 7 | **Produktiv-Funktionen:** 93 (`44` `pub`, `49` intern) | **Tests:** 0
- **Öffentliche Typen (24)**: `DurableFsError`, `Error`, `Result`, `FaultInjector`, `NoFault`, `InjectedFault`, `FaultInjectingVfs`, `HealthState`, `HealthEvent`, `AtomicHealthState`, `JournalSyncPolicy`, `RecordRecoveryPolicy`, `JournalRecord`, `AppendOnlyJournal`, `FsClass`, `PoisonedHandle`, `DurableHandle`, `LockMode`, `DirLock`, `FlushKind`, `Vfs`, `StdVfs`, `RealFs`, `TmpFileGuard`

#### `crates/contextra-durable-fs/src/error.rs` (2)

- **DurableFsError**: `kind` (`pub`, L46)
- **From<DurableFsError> for std::io::Error**: `from` (`priv`, L59)

#### `crates/contextra-durable-fs/src/fault_injector.rs` (28)

- **trait FaultInjector**: `on_write` (`priv`, L12), `on_fsync` (`priv`, L17), `on_rename` (`priv`, L22), `on_dir_fsync` (`priv`, L27)
- **FaultInjector for NoFault**: `on_write` (`priv`, L38), `on_fsync` (`priv`, L43), `on_rename` (`priv`, L48), `on_dir_fsync` (`priv`, L53)
- **FaultInjector**: `on_write` (`priv`, L60), `on_fsync` (`priv`, L65), `on_rename` (`priv`, L70), `on_dir_fsync` (`priv`, L75)
- **FaultInjectingVfs<V>**: `new` (`pub`, L107), `inject_write_disk_full` (`pub`, L120), `inject_fsync_failure` (`pub`, L125), `inject_rename_failure` (`pub`, L130), `inject_dir_fsync_failure` (`pub`, L135), `simulate_crash` (`pub`, L141)
- **Vfs for FaultInjectingVfs<V>**: `read` (`priv`, L161), `write_atomic` (`priv`, L165), `append` (`priv`, L203), `sync_file` (`priv`, L227), `sync_dir` (`priv`, L244), `rename` (`priv`, L253), `list` (`priv`, L268), `exists` (`priv`, L272), `len` (`priv`, L276), `create_dir_all` (`priv`, L280)

#### `crates/contextra-durable-fs/src/health.rs` (8)

- **HealthState**: `new` (`pub`, L33), `transition` (`pub`, L39), `allows_writes` (`pub`, L60), `is_poisoned` (`pub`, L66)
- **Default for AtomicHealthState**: `default` (`priv`, L91)
- **AtomicHealthState**: `new` (`pub`, L99), `load` (`pub`, L107), `record_event` (`pub`, L117)

#### `crates/contextra-durable-fs/src/journal.rs` (10)

- ***(Modulfunktionen)***: `calculate_crc32` (`pub`, L45)
- **AppendOnlyJournal**: `open` (`pub`, L82), `open_without_recovery` (`pub`, L91), `open_with_policy` (`pub`, L116), `path` (`pub`, L131), `append` (`pub`, L136), `flush` (`pub`, L179), `recover` (`pub`, L194), `handle_recovery_corruption` (`priv`, L277), `health_state` (`pub`, L300)

#### `crates/contextra-durable-fs/src/lib.rs` (23)

- **FsClass**: `is_supported_local` (`pub`, L61)
- ***(Modulfunktionen)***: `flush_file` (`pub`, L67), `flush_dir` (`pub`, L72), `is_cloud_sync_path` (`priv`, L76), `fs_class` (`pub`, L90), `rename_durable` (`pub`, L105), `sync_dir` (`pub`, L311), `sync_dir_with_injector` (`pub`, L316), `atomic_replace` (`pub`, L322), `atomic_replace_with_injector` (`pub`, L327)
- **DurableHandle<NoFault>**: `new` (`pub`, L130)
- **DurableHandle<F>**: `new_with_injector` (`pub`, L137), `new_with_path_and_injector` (`pub`, L147), `is_poisoned` (`pub`, L158), `flush` (`pub`, L163), `get_ref` (`pub`, L189), `get_mut` (`pub`, L198), `poison` (`pub`, L207)
- **Write for DurableHandle<F>**: `write` (`priv`, L213), `flush` (`priv`, L228)
- **DirLock**: `acquire` (`pub`, L256), `mode` (`pub`, L295)
- **Drop for DirLock**: `drop` (`priv`, L301)

#### `crates/contextra-durable-fs/src/platform.rs` (2)

- ***(Modulfunktionen)***: `sync_file` (`pub`, L23), `sync_dir_path` (`pub`, L37)

#### `crates/contextra-durable-fs/src/vfs.rs` (20)

- **trait Vfs**: `read` (`priv`, L14), `write_atomic` (`priv`, L17), `append` (`priv`, L20), `sync_file` (`priv`, L23), `sync_dir` (`priv`, L26), `rename` (`priv`, L29), `list` (`priv`, L32), `exists` (`priv`, L35), `len` (`priv`, L40), `create_dir_all` (`priv`, L47)
- **Vfs for StdVfs**: `read` (`priv`, L60), `write_atomic` (`priv`, L64), `append` (`priv`, L68), `sync_file` (`priv`, L78), `sync_dir` (`priv`, L83), `rename` (`priv`, L87), `list` (`priv`, L91)
- ***(Modulfunktionen)***: `map_io_err` (`priv`, L105), `generate_tmp_path` (`pub(crate)`, L125)
- **Drop for TmpFileGuard<**: `drop` (`priv`, L119)

### contextra-engine

> Contextra — Core storage, index, and transaction orchestrator engine

- **Pfad:** `crates/contextra-engine` | **Dateien:** 83 | **Produktiv-Funktionen:** 517 (`322` `pub`, `195` intern) | **Tests:** 184
- **Öffentliche Typen (65)**: `OrphanCleanupBackoffConfig`, `OrphanCleanupIndex`, `DeferredHyperedgeQueue`, `CodeFence`, `MarkdownBlock`, `ChunkerConfig`, `MarkdownChunker`, `RawSection`, `ExtractorMode`, `EntityExtractionConfig`, `NoopLlmGenerator`, `AutoExtractionConfig`, `KeyGuard`, `KvKeyLocks`, `PercolationResult`, `StoredDocument`, `StoredDocumentMeta`, `Collection`, `HybridQueryBuilder`, `ScopeConstraint`, `SearchStrategy`, `SignalWeights`, `CheckpointPinGuard`, `ExportDocumentV1`, `ExportCollectionV1`, `ExportMemoryV1`, `ExportRelationV1`, `OpenIeExtractor`, `RuleBasedExtractor`, `ExtractedTriple`, `EntityExtractionConfig`, `FilterOp`, `MetadataFilter`, `ImportSummary`, `AccessCounterAttentionExporter`, `DeletionLayer`, `DeletionScope`, `LayerCleanupProof`, `ExcludedScope`, `DeletionProof` *(+25 weitere)*

#### `crates/contextra-engine/src/background_workers/config.rs` (6)

- **Default for OrphanCleanupBackoffConfig**: `default` (`priv`, L19)
- ***(Modulfunktionen)***: `calculate_rebuild_cooldown` (`pub`, L29)
- **trait OrphanCleanupIndex**: `check_connectivity` (`priv`, L46), `rebuild` (`priv`, L47)
- **OrphanCleanupIndex for contextra_vector::hnsw::HnswIndex**: `check_connectivity` (`priv`, L53), `rebuild` (`priv`, L57)

#### `crates/contextra-engine/src/background_workers/expiry_workers.rs` (2)

- ***(Modulfunktionen)***: `start_expiry_cleanup_worker` (`pub`, L18), `start_decay_cleanup_worker` (`pub`, L68)

#### `crates/contextra-engine/src/background_workers/hyperedge_worker.rs` (4)

- **DeferredHyperedgeQueue**: `new` (`pub`, L25), `enqueue` (`pub`, L32), `drain_up_to` (`pub`, L38)
- ***(Modulfunktionen)***: `start_hyperedge_cascade_deferred_worker` (`pub`, L46)

#### `crates/contextra-engine/src/background_workers/orphan_workers.rs` (2)

- ***(Modulfunktionen)***: `start_orphan_cleanup_worker_with_config` (`pub`, L16), `start_orphan_cleanup_worker` (`pub`, L151)

#### `crates/contextra-engine/src/background_workers/tests.rs` (2)

- **OrphanCleanupIndex for MockDegradedHnswIndex**: `check_connectivity` (`priv`, L48), `rebuild` (`priv`, L58)

#### `crates/contextra-engine/src/chunker/blocks.rs` (7)

- ***(Modulfunktionen)***: `parse_opening_fence` (`pub(crate)`, L26), `is_closing_fence` (`pub(crate)`, L50), `parse_markdown_blocks` (`pub(crate)`, L119), `is_table_row` (`pub(crate)`, L215), `is_table_delimiter_row` (`pub(crate)`, L225)
- **MarkdownBlock**: `to_text` (`pub(crate)`, L80), `estimate_tokens` (`pub(crate)`, L112)

#### `crates/contextra-engine/src/chunker/config.rs` (1)

- **Default for ChunkerConfig**: `default` (`priv`, L29)

#### `crates/contextra-engine/src/chunker/estimate.rs` (1)

- ***(Modulfunktionen)***: `estimate_tokens` (`pub`, L20)

#### `crates/contextra-engine/src/chunker/mod.rs` (7)

- ***(Modulfunktionen)***: `scan_lines` (`priv`, L49), `is_setext_h1` (`priv`, L122), `is_setext_h2` (`priv`, L127), `parts_first` (`priv`, L394)
- **MarkdownChunker**: `new` (`pub`, L134), `with_defaults` (`pub`, L139), `chunk` (`pub`, L144)

#### `crates/contextra-engine/src/chunker/split.rs` (1)

- ***(Modulfunktionen)***: `split_raw_section` (`pub(crate)`, L21)

#### `crates/contextra-engine/src/chunker/windows.rs` (2)

- ***(Modulfunktionen)***: `chunk_text_with_overlap` (`pub`, L16), `chunk_text` (`pub`, L59)

#### `crates/contextra-engine/src/collection/crud/auto_extraction.rs` (16)

- **Default for ExtractorMode**: `default` (`priv`, L25)
- **Default for EntityExtractionConfig**: `default` (`priv`, L40)
- **LlmTextGenerator for NoopLlmGenerator**: `generate` (`priv`, L54)
- ***(Modulfunktionen)***: `recommended_mode_for_regulated` (`pub`, L69), `auto_extract_and_relate` (`pub(crate)`, L228)
- **AutoExtractionConfig**: `new` (`pub`, L91), `for_regulated` (`pub`, L100), `with_mode` (`pub`, L105), `is_enabled` (`pub`, L118)
- **Default for AutoExtractionConfig**: `default` (`priv`, L127)
- **Collection<S**: `with_auto_extraction` (`pub`, L158), `set_auto_extraction` (`pub`, L168), `set_auto_extraction_with_mode` (`pub`, L177), `set_auto_extraction_config` (`pub`, L190), `auto_extraction_config` (`pub`, L196), `auto_extraction_config_with_extractor` (`pub`, L207)

#### `crates/contextra-engine/src/collection/crud/delete.rs` (2)

- **Collection<S**: `delete` (`pub`, L8), `delete_op` (`pub`, L34)

#### `crates/contextra-engine/src/collection/crud/insert.rs` (12)

- **Collection<S**: `insert_text_only` (`pub`, L12), `upsert_text_only` (`pub`, L60), `insert_with_ttl` (`pub`, L108), `insert_typed` (`pub`, L139), `insert` (`pub`, L176), `insert_inner_unlocked` (`pub(crate)`, L195), `check_doc_id_collision` (`pub(crate)`, L238), `check_doc_id_collision_tracked` (`pub(crate)`, L261), `insert_op` (`pub`, L288), `insert_many` (`pub`, L355), `upsert` (`pub`, L431), `upsert_many` (`pub`, L477)

#### `crates/contextra-engine/src/collection/crud/internal.rs` (4)

- ***(Modulfunktionen)***: `validate_doc_id` (`pub(crate)`, L5), `validate_embedding` (`pub(crate)`, L24)
- **Collection<S**: `apply_insert_backpressure` (`pub(crate)`, L45), `lock_keys_sorted` (`pub(crate)`, L66)

#### `crates/contextra-engine/src/collection/crud/kv.rs` (3)

- **Collection<S**: `put_kv` (`pub`, L9), `put_kv_if_absent` (`pub`, L24), `get_kv` (`pub`, L52)

#### `crates/contextra-engine/src/collection/crud/links.rs` (2)

- **Collection<S**: `link_memories` (`pub`, L8), `get_links` (`pub`, L197)

#### `crates/contextra-engine/src/collection/crud/read.rs` (8)

- **Collection<S**: `snapshot_seq` (`pub`, L21), `get` (`pub`, L27), `get_at_snapshot` (`pub`, L33), `get_doc_tx` (`pub`, L53), `scan_prefix` (`pub`, L79), `scan` (`pub`, L165), `get_tracked` (`pub`, L271), `get_at_seq_tracked` (`pub`, L282)

#### `crates/contextra-engine/src/collection/crud/update.rs` (2)

- **Collection<S**: `update` (`pub`, L11), `update_op` (`pub`, L44)

#### `crates/contextra-engine/src/collection/kv_lock.rs` (9)

- **KvKeyLocks**: `new` (`pub`, L22), `shard_idx` (`pub`, L31), `lock_shard` (`pub`, L35), `lock_for` (`pub`, L40), `lock_many_sorted` (`pub`, L61)
- ***(Modulfunktionen)***: `sorted_unique_shards` (`pub`, L79)
- **Default for KvKeyLocks**: `default` (`priv`, L97)
- **LoomKvKeyLocks**: `new` (`priv`, L150), `acquire_multi_sorted` (`priv`, L160)

#### `crates/contextra-engine/src/collection/maintenance.rs` (13)

- **Collection<S**: `evict_decayed_chunks` (`pub`, L21), `repair` (`pub`, L142), `repair_internal` (`priv`, L160), `stats` (`pub`, L424), `reap_expired_documents` (`pub`, L431), `trigger_expiry_cleanup` (`pub`, L525), `update_document_importance` (`pub`, L646), `run_community_detection` (`pub`, L721), `run_community_detection_with_config` (`pub`, L729), `get_community` (`pub`, L756), `get_communities_batch` (`pub`, L774), `drop_collection` (`pub`, L813), `run_percolation_check` (`pub`, L850)

#### `crates/contextra-engine/src/collection/mod.rs` (62)

- **Default for CollectionConfig**: `default` (`priv`, L62)
- **From<**: `from` (`priv`, L85)
- ***(Modulfunktionen)***: `parse_importance_score` (`pub`, L95), `compute_default_importance` (`pub`, L115), `ensure_importance_metadata` (`pub`, L132), `extract_effective_importance` (`pub`, L202), `extract_text` (`pub(super)`, L223)
- **KvLifecycleHooks for CryptoKvStoreAdapter**: `on_rollback` (`priv`, L270), `remove_doc_segments` (`priv`, L274), `purge_tenant` (`priv`, L280)
- **Clone for Collection<S**: `clone` (`priv`, L326)
- **Collection<S**: `with_hnsw` (`pub`, L359), `new` (`pub`, L390), `deferred_hyperedge_queue` (`pub`, L438), `with_metrics_sink` (`pub`, L443), `set_metrics_sink` (`pub`, L449), `metrics_sink` (`pub`, L454), `with_clock` (`pub`, L459), `set_clock` (`pub`, L465), `clock` (`pub`, L470), `with_config` (`pub`, L475), `set_config` (`pub`, L481), `config` (`pub`, L486), `with_pressure_receiver` (`pub`, L491), `set_pressure_receiver` (`pub`, L500), `consolidation_in_progress` (`pub`, L508), `graph_index` (`pub`, L513), `with_kv_hooks` (`pub`, L518), `set_kv_hooks` (`pub`, L524), `kv_hooks` (`pub`, L529), `tenant_id` (`pub`, L535), `with_kv_store` (`pub`, L550), `set_kv_store` (`pub`, L559), `set_kv_store` (`pub`, L566), `with_kv_store` (`pub`, L571), `kv_store` (`pub`, L578), `kv_store` (`pub`, L584), `with_embedder` (`pub`, L590), `set_embedder` (`pub`, L600), `vector_index` (`pub`, L607), `namespaced_key` (`pub`, L613), `storage` (`pub`, L646), `user_key_prefix` (`pub`, L651), `name` (`pub`, L657), `len` (`pub`, L663), `dimension` (`pub`, L669), `is_empty` (`pub`, L675), `load_index` (`pub`, L681), `migrate_doc_keys_v1` (`pub`, L715), `load_text_stats` (`pub`, L752), `set_community_detection_trigger_threshold` (`pub`, L757), `with_community_detection_trigger_threshold` (`pub`, L763), `community_detection_trigger_threshold` (`pub`, L769), `mutations_since_community_detection` (`pub`, L775), `write_kv_segment` (`pub`, L781), `read_kv_segment` (`pub`, L792), `delete_kv_segment` (`pub`, L801), `generate_kv_deletion_proof` (`pub`, L810), `tenant_collection_prefix` (`pub`, L819), `tenant_encode_graph_key` (`pub`, L828), `point_lookup_metrics` (`pub`, L838), `check_and_trigger_community_detection` (`pub(super)`, L847)

#### `crates/contextra-engine/src/collection/query_builder/builder.rs` (31)

- **HybridQueryBuilder<**: `new` (`pub`, L55), `text` (`pub`, L87), `embedding` (`pub`, L93), `vector` (`pub`, L99), `k` (`pub`, L104), `weights` (`pub`, L110), `fusion_weights` (`pub`, L121), `fusion_strategy` (`pub`, L127), `filter` (`pub`, L133), `metadata_filter` (`pub`, L140), `strategy` (`pub`, L148), `anchor_entities` (`pub`, L154), `anchors` (`pub`, L160), `same_community_as` (`pub`, L165), `memory_type_filter` (`pub`, L171), `include_provenance` (`pub`, L177), `include_superseded` (`pub`, L183), `memory_types` (`pub`, L189), `filter_fn` (`pub`, L194), `reranker` (`pub`, L200), `rerank_pool_multiplier` (`pub`, L209), `rerank_pool_max` (`pub`, L217), `pid_controller` (`pub`, L224), `seq` (`pub`, L233), `as_of` (`pub`, L239), `query_timestamp` (`pub`, L245), `current_tx` (`pub`, L251), `on_signal_failure` (`pub`, L260), `with_metrics_sink` (`pub`, L266), `query_config` (`pub`, L272)
- **Collection<S**: `query` (`pub`, L302)

#### `crates/contextra-engine/src/collection/query_builder/builder_exec.rs` (3)

- **HybridQueryBuilder<**: `execute` (`pub`, L10), `execute_with_report` (`pub`, L32)
- ***(Modulfunktionen)***: `execute_with_scope` (`pub`, L304)

#### `crates/contextra-engine/src/collection/query_builder/scope.rs` (5)

- **ScopeConstraint**: `new` (`pub`, L23), `from_allowed_ids` (`pub`, L31)
- **Default for ScopeConstraint**: `default` (`priv`, L41)
- **HybridQueryBuilder<**: `scope` (`pub`, L51), `scope_ids` (`pub`, L57)

#### `crates/contextra-engine/src/collection/query_builder/strategy.rs` (4)

- **SearchStrategy**: `to_graph_strategy` (`pub`, L28), `to_fusion_strategy` (`pub`, L50), `to_signal_fusion_strategies` (`pub`, L59)
- **From<GraphTraversalStrategy> for SearchStrategy**: `from` (`priv`, L65)

#### `crates/contextra-engine/src/collection/query_builder/tests.rs` (7)

- **MockReranker**: `passthrough` (`priv`, L19), `with_delay` (`priv`, L26)
- **Reranker for SlowPartialMockReranker**: `rerank` (`priv`, L40), `rerank_deadline_ms` (`priv`, L68)
- **Reranker for MockReranker**: `rerank` (`priv`, L74), `rerank_deadline_ms` (`priv`, L97)
- ***(Modulfunktionen)***: `create_test_collection` (`priv`, L102)

#### `crates/contextra-engine/src/collection/query_builder/weights.rs` (3)

- **SignalWeights**: `new` (`pub`, L16)
- **From<SignalWeights> for FusionWeights**: `from` (`priv`, L27)
- **From<FusionWeights> for SignalWeights**: `from` (`priv`, L33)

#### `crates/contextra-engine/src/collection/relate.rs` (4)

- **Collection<S**: `relate_with_provenance` (`pub`, L9), `relate` (`pub`, L80), `relate_bidirectional` (`pub`, L86), `relate_n_ary` (`pub`, L100)

#### `crates/contextra-engine/src/collection/search/basic.rs` (4)

- **Collection<S**: `search` (`pub`, L16), `search_with_filter` (`pub`, L34), `search_with_filter_expr` (`pub`, L51), `search_text` (`pub`, L97)

#### `crates/contextra-engine/src/collection/search/checkpoint.rs` (6)

- **CheckpointPinGuard<**: `new` (`pub`, L23), `new_at_latest` (`pub`, L34), `release` (`pub`, L48)
- **Drop for CheckpointPinGuard<**: `drop` (`priv`, L56)
- ***(Modulfunktionen)***: `with_pinned_checkpoint` (`pub`, L69), `with_pinned_checkpoint_at_latest` (`pub`, L87)

#### `crates/contextra-engine/src/collection/search/filtered.rs` (5)

- **Collection<S**: `estimate_filter_selectivity` (`pub(super)`, L20), `get_matching_doc_ids_at` (`pub(super)`, L78), `get_matching_doc_ids_for_query_at` (`pub(super)`, L112), `search_filtered` (`pub`, L170), `search_filtered_at` (`pub`, L187)

#### `crates/contextra-engine/src/collection/search/hybrid/mod.rs` (7)

- **Collection<S**: `hybrid_search` (`pub`, L16), `hybrid_search_reranked` (`pub`, L36), `hybrid_search_with_weights` (`pub`, L59), `hybrid_search_with_strategy` (`pub`, L85), `apply_community_boost_post_fusion` (`pub(super)`, L273), `traverse_links` (`pub`, L325), `filter_by_importance` (`pub`, L364)

#### `crates/contextra-engine/src/collection/search/hybrid/query.rs` (3)

- **Collection<S**: `hybrid_search_with_query` (`pub`, L21), `hybrid_search_with_query_at` (`pub`, L35), `hybrid_search_with_query_and_report_at` (`pub`, L48)

#### `crates/contextra-engine/src/collection/search/hydrate.rs` (2)

- **Collection<S**: `hydrate_from_scored_at` (`pub(super)`, L9), `hydrate_from_tuples_at` (`pub(super)`, L56)

#### `crates/contextra-engine/src/collection/tests/insert_tests.rs` (17)

- **StorageEngine for FailingRollbackMockStorage**: `get` (`priv`, L238), `get_at_seq` (`priv`, L241), `put` (`priv`, L248), `put_if_absent` (`priv`, L251), `delete` (`priv`, L259), `commit` (`priv`, L262), `rollback` (`priv`, L265), `rollback_to_tx` (`priv`, L272), `flush` (`priv`, L275), `stats` (`priv`, L278), `last_seq_no` (`priv`, L287), `last_tx_id` (`priv`, L290), `pin_checkpoint` (`priv`, L293), `unpin_checkpoint` (`priv`, L296), `scan` (`priv`, L299), `scan_prefix` (`priv`, L307), `scan_prefix_bounded` (`priv`, L313)

#### `crates/contextra-engine/src/collection/tests/link_tests.rs` (30)

- **StorageEngine for FailOnStorageCommit**: `get` (`priv`, L67), `get_at_seq` (`priv`, L70), `put` (`priv`, L77), `delete` (`priv`, L80), `commit` (`priv`, L83), `rollback` (`priv`, L90), `rollback_to_tx` (`priv`, L93), `flush` (`priv`, L96), `stats` (`priv`, L99), `last_seq_no` (`priv`, L108), `last_tx_id` (`priv`, L111), `pin_checkpoint` (`priv`, L114), `unpin_checkpoint` (`priv`, L117), `scan_prefix` (`priv`, L120), `scan` (`priv`, L126)
- **StorageEngine for FailOnPutStorage**: `get` (`priv`, L182), `get_at_seq` (`priv`, L185), `put` (`priv`, L192), `delete` (`priv`, L203), `commit` (`priv`, L206), `rollback` (`priv`, L209), `rollback_to_tx` (`priv`, L212), `flush` (`priv`, L215), `stats` (`priv`, L218), `last_seq_no` (`priv`, L227), `last_tx_id` (`priv`, L230), `pin_checkpoint` (`priv`, L233), `unpin_checkpoint` (`priv`, L236), `scan_prefix` (`priv`, L239), `scan` (`priv`, L245)

#### `crates/contextra-engine/src/collection/tests/misc_tests.rs` (1)

- **TextEmbeddingEngine for FakeEmbedder**: `embed` (`priv`, L13)

#### `crates/contextra-engine/src/collection/tests/read_tests.rs` (16)

- **StorageEngine for BoundedScanMockStorage**: `get` (`priv`, L18), `get_at_seq` (`priv`, L21), `put` (`priv`, L28), `delete` (`priv`, L31), `commit` (`priv`, L34), `rollback` (`priv`, L37), `rollback_to_tx` (`priv`, L40), `flush` (`priv`, L43), `stats` (`priv`, L46), `last_seq_no` (`priv`, L55), `last_tx_id` (`priv`, L58), `pin_checkpoint` (`priv`, L61), `unpin_checkpoint` (`priv`, L64), `scan_prefix` (`priv`, L67), `scan_prefix_bounded` (`priv`, L75), `scan` (`priv`, L110)

#### `crates/contextra-engine/src/collection/tests/search_tests.rs` (16)

- **StorageEngine for PinTrackingFailingStorage**: `get` (`priv`, L63), `get_at_seq` (`priv`, L66), `put` (`priv`, L73), `delete` (`priv`, L76), `commit` (`priv`, L79), `rollback` (`priv`, L82), `rollback_to_tx` (`priv`, L85), `flush` (`priv`, L88), `stats` (`priv`, L91), `last_seq_no` (`priv`, L100), `last_tx_id` (`priv`, L103), `pin_checkpoint` (`priv`, L106), `unpin_checkpoint` (`priv`, L113), `scan_prefix` (`priv`, L120), `scan_prefix_at` (`priv`, L126), `scan` (`priv`, L137)

#### `crates/contextra-engine/src/collection/tx.rs` (3)

- **Collection<S**: `next_tx` (`pub`, L12), `allocate_tx` (`pub`, L19), `begin_transaction` (`pub`, L31)

#### `crates/contextra-engine/src/contextra_impl/collections.rs` (9)

- **Contextra**: `collection_for_tenant` (`pub`, L8), `collection_with_language_and_tenant` (`pub`, L19), `collection` (`pub`, L138), `collection_with_language` (`pub`, L148), `allocate_tx` (`pub`, L258), `list_collections` (`pub`, L269), `list_collections_for_tenant` (`pub`, L294), `drop_collection` (`pub`, L324), `purge_tenant` (`pub`, L447)

#### `crates/contextra-engine/src/contextra_impl/crud.rs` (32)

- **Contextra**: `default_col` (`pub(crate)`, L8), `put_kv` (`pub`, L13), `get_kv` (`pub`, L18), `insert` (`pub`, L23), `insert_typed` (`pub`, L31), `upsert` (`pub`, L46), `insert_many` (`pub`, L54), `upsert_many` (`pub`, L59), `get` (`pub`, L64), `get_at_snapshot` (`pub`, L69), `last_committed_seq` (`pub`, L74), `create_snapshot` (`pub`, L79), `update` (`pub`, L84), `search` (`pub`, L93), `search_with_filter` (`pub`, L109), `search_with_filter_expr` (`pub`, L125), `insert_text_only` (`pub`, L140), `upsert_text_only` (`pub`, L153), `search_text` (`pub`, L167), `search_filtered` (`pub`, L179), `hybrid_search` (`pub`, L194), `hybrid_search_reranked` (`pub`, L211), `hybrid_search_with_weights` (`pub`, L232), `hybrid_search_with_strategy` (`pub`, L253), `hybrid_search_with_query` (`pub`, L278), `delete` (`pub`, L290), `relate` (`pub`, L295), `relate_bidirectional` (`pub`, L301), `scan_prefix` (`pub`, L307), `len` (`pub`, L316), `is_empty` (`pub`, L321), `scan` (`pub`, L326)

#### `crates/contextra-engine/src/contextra_impl/diagnostics.rs` (3)

- **Contextra**: `stats` (`pub`, L7), `flush` (`pub`, L52), `orphan_registry` (`pub`, L57)

#### `crates/contextra-engine/src/contextra_impl/lifecycle.rs` (14)

- **Contextra**: `open` (`pub`, L12), `open_with_config` (`pub`, L27), `open_authorized` (`pub`, L42), `init_embedding_backend` (`priv`, L205), `initialize_collections` (`priv`, L237), `repair_on_open` (`pub`, L254), `repair_on_open_internal` (`priv`, L267), `scan_pending_intents` (`priv`, L427), `shutdown` (`pub`, L472), `wait_shutdown` (`pub`, L476), `close` (`pub`, L493), `with_embedder` (`pub`, L501), `set_embedder` (`pub`, L516)
- **Drop for Contextra**: `drop` (`priv`, L486)

#### `crates/contextra-engine/src/contextra_impl/wiring.rs` (5)

- **Contextra**: `set_metrics_sink` (`pub`, L6), `metrics_sink` (`pub`, L11), `set_router` (`pub`, L15), `set_calibrator` (`pub`, L19), `set_pid_controller` (`pub`, L26)

#### `crates/contextra-engine/src/export.rs` (2)

- **Collection<S**: `export_memories` (`pub`, L82)
- **Contextra**: `export_memories` (`pub`, L372)

#### `crates/contextra-engine/src/extraction/mod.rs` (5)

- **EntityExtractor for OpenIeExtractor**: `extract_triples` (`priv`, L39)
- **EntityExtractor for RuleBasedExtractor**: `extract_triples` (`priv`, L57)
- ***(Modulfunktionen)***: `create_extractor` (`pub`, L68), `extract_triples_for_mode` (`pub`, L76), `extract_triples_for_config` (`pub`, L87)

#### `crates/contextra-engine/src/extraction/open_ie.rs` (3)

- ***(Modulfunktionen)***: `chunk_text_simple` (`priv`, L14), `sanitize_json_response` (`priv`, L48), `extract_triples` (`pub`, L73)

#### `crates/contextra-engine/src/extraction/rule_based.rs` (5)

- ***(Modulfunktionen)***: `is_protected_compound` (`priv`, L66), `clean_entity_token` (`priv`, L81), `extract_entity_candidate` (`priv`, L86)
- **RuleBasedExtractor**: `new` (`pub`, L122), `extract` (`pub`, L127)

#### `crates/contextra-engine/src/extraction/tests.rs` (2)

- **MockLlmGenerator**: `new` (`priv`, L19)
- **LlmTextGenerator for MockLlmGenerator**: `generate` (`priv`, L28)

#### `crates/contextra-engine/src/extraction/types.rs` (1)

- **Default for EntityExtractionConfig**: `default` (`priv`, L37)

#### `crates/contextra-engine/src/filter.rs` (3)

- ***(Modulfunktionen)***: `extract_memory_type` (`pub`, L13)
- **MetadataFilter**: `matches` (`pub`, L77)
- **TryFrom<MetadataFilter> for FilterExpr**: `try_from` (`priv`, L90)

#### `crates/contextra-engine/src/fusion.rs` (3)

- ***(Modulfunktionen)***: `fuse_search_results_with_strategy` (`pub`, L16), `fuse_search_results_with_signal_strategies` (`pub`, L35), `fuse_signals` (`pub`, L54)

#### `crates/contextra-engine/src/import.rs` (2)

- **Collection<S**: `import_memories` (`pub`, L28)
- **Contextra**: `import_memories` (`pub`, L105)

#### `crates/contextra-engine/src/kv_cache_integration/mod.rs` (7)

- **AccessCounterAttentionExporter**: `new` (`pub`, L26), `record_access` (`pub`, L31), `get_access_count` (`pub`, L37)
- **AttentionExporter for AccessCounterAttentionExporter**: `export_attention_weights` (`priv`, L47)
- ***(Modulfunktionen)***: `build_eviction_worker` (`pub`, L61), `build_eviction_worker_with_exporter` (`pub`, L68), `start_kv_cache_eviction` (`pub`, L79)

#### `crates/contextra-engine/src/lib.rs` (33)

- **LayerCleanupProof**: `new_after_verified_empty` (`pub`, L53), `verify_and_create` (`pub`, L69)
- ***(Modulfunktionen)***: `default_signature_version` (`priv`, L118), `default_tx_id` (`priv`, L122), `construct_hmac_payload` (`priv`, L157), `register_consolidation_launcher` (`pub`, L426)
- **TenantIsolatedKvStore**: `new` (`pub`, L130), `purge_tenant_segments` (`pub`, L134), `on_rollback` (`pub`, L141), `remove_tenant_segment` (`pub`, L148)
- **DeletionProof**: `create` (`pub`, L188), `tenant_id` (`pub`, L247), `verify` (`pub`, L255), `export_for_audit` (`pub`, L302)
- **Default for CommunityDetectionConfig**: `default` (`priv`, L476)
- **Default for EmbeddingBackend**: `default` (`priv`, L493)
- **ContextraConfig**: `with_consolidation_launcher` (`pub`, L558), `with_tenant_policy` (`pub`, L563), `with_metrics_sink` (`pub`, L568)
- **std::fmt::Debug for ContextraConfig**: `fmt` (`priv`, L576)
- **Default for ContextraConfig**: `default` (`priv`, L616)
- **Contextra**: `inner_storage` (`pub`, L684), `config` (`pub`, L689), `license_gate` (`pub`, L694), `kv_eviction_attention_source` (`pub`, L699), `set_kv_hooks` (`pub`, L709), `kv_hooks` (`pub`, L714)
- **trait SandboxBridge**: `db_search` (`priv`, L721), `db_insert` (`priv`, L726), `db_get` (`priv`, L731)
- **SandboxBridge for Contextra**: `db_search` (`priv`, L739), `db_insert` (`priv`, L769), `db_get` (`priv`, L783)

#### `crates/contextra-engine/src/merge_operator_adapter.rs` (6)

- **std::fmt::Debug for WasmMergeOperator**: `fmt` (`priv`, L46)
- **WasmMergeOperator**: `new` (`pub`, L55), `new_with_config` (`pub`, L69)
- **MergeOperator for WasmMergeOperator**: `merge` (`priv`, L127)
- **Drop for WasmMergeOperator**: `drop` (`priv`, L172)
- ***(Modulfunktionen)***: `map_sandbox_err` (`priv`, L181)

#### `crates/contextra-engine/src/metrics_names.rs` (3)

- ***(Modulfunktionen)***: `record_histogram_safe` (`pub`, L27), `record_gauge_safe` (`pub`, L39), `record_counter_safe` (`pub`, L51)

#### `crates/contextra-engine/src/temporal_filter.rs` (7)

- **ValidityWindow**: `is_empty` (`pub`, L36), `is_valid_bitemporal` (`pub`, L46)
- ***(Modulfunktionen)***: `parse_u64_val` (`priv`, L83), `parse_i64_val` (`priv`, L100), `extract_validity_window` (`pub`, L113), `apply_temporal_validity_filter` (`pub`, L159), `apply_temporal_validity_filter_at` (`pub`, L179)

#### `crates/contextra-engine/src/transaction/cleanup.rs` (1)

- ***(Modulfunktionen)***: `cleanup_orphaned_consolidation_intents` (`pub`, L7)

#### `crates/contextra-engine/src/transaction/compensating_actions.rs` (14)

- **trait CompensatingAction**: `execute` (`priv`, L9)
- **CommitLedger**: `new` (`pub`, L19), `push` (`pub`, L25), `execute_rollback` (`pub`, L29)
- **CompensateLsmAction<S**: `new` (`pub`, L47)
- **CompensatingAction for CompensateLsmAction<S**: `execute` (`priv`, L65)
- **CompensateGraphAction<S**: `new` (`pub`, L199)
- **CompensatingAction for CompensateGraphAction<S**: `execute` (`priv`, L208)
- **CompensateHnswAction<S**: `new` (`pub`, L245)
- **CompensatingAction for CompensateHnswAction<S**: `execute` (`priv`, L259)
- **CompensateTextAction<S**: `new` (`pub`, L305)
- **CompensatingAction for CompensateTextAction<S**: `execute` (`priv`, L314)
- **RollbackStagedAction<S**: `new` (`pub`, L350)
- **CompensatingAction for RollbackStagedAction<S**: `execute` (`priv`, L356)

#### `crates/contextra-engine/src/transaction/db_transaction.rs` (11)

- **DbTransaction<S**: `new` (`pub`, L28), `record_keys` (`pub`, L47), `record_keys_with_old_values` (`pub`, L52), `stage_text_insert` (`pub`, L79), `stage_text_delete` (`pub`, L87), `stage_graph_entity` (`pub`, L95), `stage_graph_edge` (`pub`, L103), `stage_graph_entity_delete` (`pub`, L111), `stage_graph_edge_delete` (`pub`, L119), `stage_hyperedge` (`pub`, L127)
- **Drop for DbTransaction<S**: `drop` (`priv`, L137)

#### `crates/contextra-engine/src/transaction/db_transaction_commit.rs` (4)

- **DbTransaction<S**: `commit_text_staged` (`pub(super)`, L14), `commit_graph_staged` (`pub(super)`, L47), `commit` (`pub`, L120), `commit_internal` (`priv`, L138)

#### `crates/contextra-engine/src/transaction/db_transaction_rollback.rs` (6)

- **DbTransaction<S**: `trigger_kv_store_rollback` (`pub(super)`, L10), `compensate_hnsw` (`pub(super)`, L29), `compensate_text` (`pub(super)`, L52), `compensate_lsm` (`pub(super)`, L77), `rollback_internal` (`pub(super)`, L191), `rollback` (`pub`, L218)

#### `crates/contextra-engine/src/transaction/tests.rs` (1)

- ***(Modulfunktionen)***: `create_test_collection` (`priv`, L12)

### contextra-graph

> CSR-Graph for entity-relation traversal (Signal 3 in 4-Signal Fusion)

- **Pfad:** `crates/contextra-graph` | **Dateien:** 54 | **Produktiv-Funktionen:** 419 (`257` `pub`, `162` intern) | **Tests:** 238
- **Öffentliche Typen (93)**: `ApprhError`, `ApprhGateMonitorConfig`, `ApprhObservationRecord`, `ApprhGateMonitorSnapshot`, `ApprhGateMonitor`, `ApprhParams`, `ApprhMode`, `ApprhSelectorConfig`, `ApprhSelector`, `ApprhFlipGate`, `DefaultApprhFlipGate`, `ApprhShadowComparison`, `ArcSlice`, `CascadeInvalidationReport`, `CascadeStatus`, `DeletionProof`, `CascadeTicket`, `GraphGarbageCollection`, `HyperedgeCascadeReport`, `CascadeReport`, `CommunityDetectionConfig`, `VirtualHyperedgeNode`, `StarExpansionIterator`, `CommunityAssignment`, `EdgeId`, `ConflictPattern`, `EdgeAssertion`, `ContradictionDetector`, `ExactPredicateConflictDetector`, `ConsistencyEnforcer`, `EdgePayload`, `CsrGraph`, `GraphInner`, `MemoryEstimate`, `InnerWriteGuard`, `EdgeType`, `Edge`, `PersistedEdgePayload`, `CsrGraphConfig`, `InternalIndex` *(+53 weitere)*

#### `crates/contextra-graph/src/apprh/diffusion.rs` (1)

- ***(Modulfunktionen)***: `run_apprh_push` (`pub`, L22)

#### `crates/contextra-graph/src/apprh/error.rs` (1)

- **From<ApprhError> for contextra_types::ContextraError**: `from` (`priv`, L22)

#### `crates/contextra-graph/src/apprh/gate_monitor.rs` (11)

- **Default for ApprhGateMonitorConfig**: `default` (`priv`, L21)
- **ApprhGateMonitorConfig**: `validate` (`pub`, L34)
- **ApprhGateMonitor**: `new` (`pub`, L110), `with_gate` (`pub`, L118), `observe` (`pub`, L139), `production_ready` (`pub`, L169), `snapshot` (`pub`, L175), `reset` (`pub`, L216), `config` (`pub`, L224)
- **Default for ApprhGateMonitor**: `default` (`priv`, L230)
- **std::fmt::Debug for ApprhGateMonitor**: `fmt` (`priv`, L246)

#### `crates/contextra-graph/src/apprh/mod.rs` (6)

- ***(Modulfunktionen)***: `apprh_local` (`pub`, L34), `forward_push_apprh` (`pub`, L49)
- **CsrGraph**: `apprh_diffusion` (`pub`, L79), `evaluate_apprh_diffusion` (`pub`, L88), `apprh_shadow_compare` (`pub`, L97), `evaluate_apprh_shadow` (`pub`, L108)

#### `crates/contextra-graph/src/apprh/params.rs` (2)

- **Default for ApprhParams**: `default` (`priv`, L21)
- **ApprhParams**: `validate` (`pub`, L38)

#### `crates/contextra-graph/src/apprh/selector.rs` (10)

- **Default for ApprhSelectorConfig**: `default` (`priv`, L42)
- **ApprhSelector**: `new` (`pub`, L68), `with_default_monitor` (`pub`, L81), `mode` (`pub`, L87), `set_mode` (`pub`, L96), `promote_to_production` (`pub`, L111), `demote_to_shadow` (`pub`, L116), `monitor` (`pub`, L121), `evaluate_and_run` (`pub`, L129)
- **std::fmt::Debug for ApprhSelector**: `fmt` (`priv`, L197)

#### `crates/contextra-graph/src/apprh/shadow.rs` (4)

- **trait ApprhFlipGate**: `should_flip` (`priv`, L14)
- **ApprhFlipGate for DefaultApprhFlipGate**: `should_flip` (`priv`, L22)
- ***(Modulfunktionen)***: `shadow_compare_forward_push_vs_apprh` (`pub`, L50), `normalize_and_sort` (`pub(crate)`, L115)

#### `crates/contextra-graph/src/arc_slice.rs` (13)

- **ArcSlice<T>**: `new` (`pub`, L20), `from_vec` (`pub`, L31), `try_slice` (`pub`, L38), `slice` (`pub`, L65), `as_arc` (`pub`, L75), `offset` (`pub`, L81), `len` (`pub`, L87), `is_empty` (`pub`, L93), `as_slice` (`pub`, L99)
- **Deref for ArcSlice<T>**: `deref` (`priv`, L108)
- **From<Arc<**: `from` (`priv`, L115)
- **From<Vec<T>> for ArcSlice<T>**: `from` (`priv`, L122)
- **IntoIterator**: `into_iter` (`priv`, L132)

#### `crates/contextra-graph/src/cascade.rs` (7)

- ***(Modulfunktionen)***: `cascade_invalidate_edges_for_superseded_doc` (`pub`, L31), `topological_sort_hyperedge_closure` (`pub`, L169), `cascade_invalidate_hyperedges_for_superseded_doc` (`pub`, L243), `dfs_upward` (`priv`, L265), `verify_doc_hyperedge_deletion_completeness` (`pub`, L361)
- **trait GraphGarbageCollection**: `sweep_orphans` (`priv`, L102)
- **GraphGarbageCollection for CsrGraph**: `sweep_orphans` (`priv`, L106)

#### `crates/contextra-graph/src/community.rs` (10)

- **VirtualHyperedgeNode**: `new` (`pub`, L48)
- **StarExpansionIterator**: `new` (`pub`, L71), `next_with_weight` (`pub`, L90)
- **Iterator for StarExpansionIterator**: `next` (`priv`, L115)
- **Default for CommunityDetectionConfig**: `default` (`priv`, L122)
- **SimpleRng**: `new` (`priv`, L149), `next_u64` (`priv`, L159), `gen_range` (`priv`, L168), `shuffle` (`priv`, L175)
- ***(Modulfunktionen)***: `detect_communities` (`pub`, L204)

#### `crates/contextra-graph/src/community/tests.rs` (2)

- **tracing_subscriber::Layer<S> for LogCaptureLayer**: `on_event` (`priv`, L249)
- **tracing::field::Visit for StringVisitor**: `record_debug` (`priv`, L262)

#### `crates/contextra-graph/src/consistency_enforcement.rs` (13)

- **EdgeAssertion**: `pattern_hash` (`pub`, L55)
- **trait ContradictionDetector**: `conflicts` (`priv`, L67)
- **ContradictionDetector for ExactPredicateConflictDetector**: `conflicts` (`priv`, L77)
- **ConsistencyEnforcer**: `new` (`pub`, L96), `check_before_insert` (`pub`, L105), `detect_contradiction` (`pub`, L113), `record_contradiction` (`pub`, L126), `is_suppressed` (`pub`, L153), `active_patterns` (`pub`, L160), `get_pattern` (`pub`, L165), `suppression_threshold` (`pub`, L170), `suggest_tombstone_candidates` (`pub`, L181)
- **Default for ConsistencyEnforcer**: `default` (`priv`, L190)

#### `crates/contextra-graph/src/csr/graph_index.rs` (20)

- **GraphIndexExt for CsrGraph**: `remove_entity` (`priv`, L19)
- **Default for CsrGraph**: `default` (`priv`, L125)
- **GraphIndex for CsrGraph**: `add_entity` (`priv`, L131), `add_edge` (`priv`, L164), `personalized_page_rank` (`priv`, L256), `personalized_page_rank_at` (`priv`, L281), `path_rag_at` (`priv`, L318), `traverse_at` (`priv`, L327), `traverse_at_time` (`priv`, L339), `traverse_at_bitemporal` (`priv`, L351), `traverse` (`priv`, L508), `commit` (`priv`, L648), `remove_edge` (`priv`, L765), `add_bidirectional` (`priv`, L801), `neighbors` (`priv`, L817), `rollback` (`priv`, L821), `rollback_to_tx` (`priv`, L831), `last_tx_id` (`priv`, L839), `len` (`priv`, L843), `stats` (`priv`, L847)

#### `crates/contextra-graph/src/csr/graph_persist.rs` (4)

- **CsrGraph**: `persist_entity` (`pub`, L23), `persist_edge` (`pub`, L36), `delete_edge_persistence` (`pub`, L57), `load_from_storage` (`pub`, L75)

#### `crates/contextra-graph/src/csr/graph_read.rs` (15)

- **CsrGraph**: `compact` (`pub`, L12), `compact_async` (`pub`, L37), `set_communities_batch` (`pub`, L79), `get_communities_batch` (`pub`, L88), `neighbors` (`pub`, L135), `pagerank` (`pub`, L186), `get_deleted_node_indices` (`pub`, L268), `deleted_view` (`pub`, L286), `personalized_page_rank_with_context_async` (`pub`, L291), `entity_count` (`pub`, L312), `entity_exists` (`pub`, L321), `is_entity_deleted` (`pub`, L332), `edge_count` (`pub`, L341), `remove_entity` (`pub`, L349), `path_rag_at` (`pub`, L357)

#### `crates/contextra-graph/src/csr/graph_write.rs` (57)

- **CsrGraph**: `new` (`pub`, L42), `with_config` (`pub`, L47), `intern_role` (`pub`, L67), `resolve_role_string` (`pub`, L72), `contains_role` (`pub`, L77), `contains_role_id` (`pub`, L82), `push_cooccurrence_signal` (`pub`, L88), `push_traversal_signal` (`pub`, L95), `reinforcement_signal_counts` (`pub`, L102), `flush_reinforcement_buffer` (`pub`, L111), `with_storage` (`pub`, L119), `with_config_and_storage` (`pub`, L124), `with_consistency_enforcer` (`pub`, L134), `active_conflict_patterns` (`pub`, L142), `detect_contradiction` (`pub`, L152), `get_conflict_pattern` (`pub`, L167), `is_conflict_suppressed` (`pub`, L177), `suggest_tombstone_candidates_for_pattern` (`pub`, L187), `tombstone_edge` (`pub`, L200), `remove_doc` (`pub`, L220), `inner_read` (`pub`, L272), `inner_write` (`pub(crate)`, L276), `set_storage` (`pub`, L284), `storage` (`pub`, L289), `enqueue_cascade_deferred` (`pub`, L294), `pending_cascade_queue_len` (`pub`, L306), `process_cascade_queue` (`pub`, L314), `dfs_upward` (`priv`, L336), `estimate_memory` (`pub`, L390), `estimate_memory_bytes` (`pub`, L396), `estimate_compaction_peak_bytes` (`pub`, L403), `get_source_doc_id` (`pub`, L409), `source_doc_id_at` (`pub`, L415), `tombstone_edges_direct` (`pub(crate)`, L453), `relate_n_ary` (`pub`, L507), `relate_n_ary_and_persist` (`pub`, L530), `persist_hyperedge` (`pub`, L547), `insert_hyperedge_direct` (`pub`, L574), `parent_hyperedges_of` (`pub`, L605), `hyperedges_for_doc` (`pub`, L630), `max_hyperedge_id` (`pub`, L650), `get_hyperedge` (`pub`, L661), `hyperedges_for_entity` (`pub`, L674), `tombstone_hyperedge` (`pub`, L701), `commit_super_edge_batch` (`pub`, L758), `commit_super_edge_batch_async` (`pub`, L854), `edges_for_doc` (`pub`, L869), `insert_hyperedge` (`pub`, L880), `insert_entity_direct` (`pub`, L885), `add_edge` (`pub`, L897), `insert_edge_auto` (`pub`, L1001), `insert_edge_direct` (`pub`, L1033), `insert_edge_direct_with_validity` (`pub`, L1049), `insert_edge_direct_with_bitemporal_validity` (`pub`, L1074), `load_entity_direct` (`pub`, L1101), `load_edge_direct` (`pub`, L1114)
- **GraphCollectionMutation for CsrGraph**: `relate_n_ary` (`priv`, L1167)

#### `crates/contextra-graph/src/csr/inner.rs` (28)

- **MemoryEstimate**: `total_bytes` (`pub`, L91)
- ***(Modulfunktionen)***: `table_bytes` (`priv`, L97), `vec_bytes` (`priv`, L102), `sentinel_entity` (`pub(crate)`, L107)
- **Default for GraphInner**: `default` (`priv`, L112)
- **GraphInner**: `new` (`pub`, L118), `entity_at` (`pub(crate)`, L152), `tx_valid_from_at` (`pub(crate)`, L157), `tx_valid_to_at` (`pub(crate)`, L165), `business_valid_from_at` (`pub(crate)`, L173), `business_valid_to_at` (`pub(crate)`, L181), `source_doc_id_at` (`pub(crate)`, L189), `estimate_memory` (`pub(crate)`, L196), `estimate_memory_bytes` (`pub(crate)`, L403), `estimate_compaction_peak_bytes` (`pub(crate)`, L407), `hyperedges_for_entity` (`pub(crate)`, L417), `insert_hyperedge` (`pub(crate)`, L428), `get_or_create_index` (`pub`, L445), `add_to_out_weight_sum` (`pub(crate)`, L461), `recompute_node_out_weight_sum` (`pub(crate)`, L473), `compact` (`pub`, L519), `find_edge_mut_by_entities` (`pub(crate)`, L651), `all_entity_ids` (`pub(crate)`, L708), `outgoing_edges_mut` (`pub(crate)`, L712), `sync_edge_reinforcement_weights` (`pub(crate)`, L747)
- **std::ops::Deref for InnerWriteGuard<**: `deref` (`priv`, L799)
- **std::ops::DerefMut for InnerWriteGuard<**: `deref_mut` (`priv`, L805)
- **Drop for InnerWriteGuard<**: `drop` (`priv`, L811)

#### `crates/contextra-graph/src/csr/path_graph.rs` (12)

- **crate::path_rag::PathGraph for CsrGraph**: `neighbors_with_weights` (`priv`, L8), `predecessors_with_weights` (`priv`, L56), `hyperedges_for_entity` (`priv`, L107), `get_hyperedge` (`priv`, L111)
- **crate::path_rag::PathGraph**: `neighbors_with_weights` (`priv`, L120), `predecessors_with_weights` (`priv`, L123), `hyperedges_for_entity` (`priv`, L126), `get_hyperedge` (`priv`, L129)
- **crate::path_rag::PathGraph for Arc<CsrGraph>**: `neighbors_with_weights` (`priv`, L138), `predecessors_with_weights` (`priv`, L141), `hyperedges_for_entity` (`priv`, L144), `get_hyperedge` (`priv`, L147)

#### `crates/contextra-graph/src/csr/tests/basic_tests.rs` (1)

- ***(Modulfunktionen)***: `setup_test_graph` (`pub(super)`, L8)

#### `crates/contextra-graph/src/csr/tests/bitemporal_tests.rs` (1)

- ***(Modulfunktionen)***: `consume_ppr_inner` (`priv`, L208)

#### `crates/contextra-graph/src/csr/tests/graph_index_tests.rs` (3)

- **tracing_subscriber::Layer<S> for LogCaptureLayer**: `on_event` (`priv`, L160)
- **tracing::field::Visit for StringVisitor**: `record_debug` (`priv`, L173)
- ***(Modulfunktionen)***: `verify_bitemporal_assertions` (`priv`, L348)

#### `crates/contextra-graph/src/csr/types.rs` (2)

- **Edge**: `new` (`pub`, L30)
- **Default for CsrGraphConfig**: `default` (`priv`, L79)

#### `crates/contextra-graph/src/csr/visibility.rs` (4)

- ***(Modulfunktionen)***: `is_suspicious_tx_id` (`pub`, L31), `is_edge_visible` (`pub`, L38), `is_edge_visible_business` (`pub`, L48), `is_edge_visible_bitemporal` (`pub`, L62)

#### `crates/contextra-graph/src/edge_reinforcement.rs` (5)

- **Default for EdgeReinforcementConfig**: `default` (`priv`, L31)
- ***(Modulfunktionen)***: `compute_edge_weight` (`pub`, L142), `apply_cooccurrence_reinforcement` (`pub`, L148), `apply_traversal_reinforcement` (`pub`, L159), `apply_weight_normalization` (`pub`, L173)

#### `crates/contextra-graph/src/edge_reinforcement_buffer.rs` (6)

- **EdgeReinforcementBuffer**: `new` (`pub`, L57), `push_cooccurrence` (`pub`, L63), `push_traversal` (`pub`, L72), `cooccurrence_count` (`pub`, L81), `traversal_count` (`pub`, L86), `flush_to_graph` (`pub`, L94)

#### `crates/contextra-graph/src/entity_extraction.rs` (8)

- **ExtractedRelation**: `new` (`pub`, L35)
- **trait EntityExtractor**: `extract` (`priv`, L53)
- **RuleBasedEntityExtractor**: `new` (`pub`, L81), `clean_entity_name` (`priv`, L127), `find_entity_candidate` (`priv`, L134)
- **Default for RuleBasedEntityExtractor**: `default` (`priv`, L165)
- **EntityExtractor for RuleBasedEntityExtractor**: `extract` (`priv`, L171)
- ***(Modulfunktionen)***: `ingest_extracted_relations` (`pub`, L235)

#### `crates/contextra-graph/src/error.rs` (1)

- **From<GraphMutationError> for ContextraError**: `from` (`priv`, L55)

#### `crates/contextra-graph/src/hyperedge.rs` (37)

- ***(Modulfunktionen)***: `star_weight` (`pub`, L27), `sort_dedup_entities` (`pub`, L356)
- **HyperEdgeId**: `new` (`pub`, L45), `inner` (`pub`, L51)
- **From<u64> for HyperEdgeId**: `from` (`priv`, L57)
- **std::fmt::Display for HyperEdgeId**: `fmt` (`priv`, L63)
- **RoleId**: `new` (`pub`, L78), `inner` (`pub`, L84)
- **From<u32> for RoleId**: `from` (`priv`, L90)
- **std::fmt::Display for RoleId**: `fmt` (`priv`, L96)
- **RoleBinding**: `new` (`pub`, L112)
- **HyperEdge**: `new` (`pub`, L171), `with_tx_validity` (`pub`, L192), `with_business_validity` (`pub`, L199), `with_source_doc_id` (`pub`, L210), `with_child_edge_ids` (`pub`, L218), `validate` (`pub`, L237), `serialize` (`pub`, L260), `deserialize` (`pub`, L267), `view` (`pub`, L289)
- **HyperEdgeView<**: `new` (`pub`, L303), `slice_participants` (`pub`, L311), `primary_participants` (`pub`, L319)
- **Deref for HyperEdgeView<**: `deref` (`priv`, L329)
- **Default for RoleInterner**: `default` (`priv`, L346)
- **ConsolidationNodesGuard**: `acquire` (`pub`, L379), `try_acquire` (`pub`, L388), `entities` (`pub`, L395), `with_entities` (`pub`, L400)
- **RoleInterner**: `new` (`pub`, L410), `get_or_intern` (`pub`, L422), `resolve` (`pub`, L446), `resolve_string` (`pub`, L454), `contains_role` (`pub`, L459), `contains_id` (`pub`, L464), `len` (`pub`, L469), `is_empty` (`pub`, L474)

#### `crates/contextra-graph/src/hyperedge_suggest.rs` (3)

- **From<HyperedgeSuggestError> for GraphMutationError**: `from` (`priv`, L48)
- ***(Modulfunktionen)***: `compute_co_occurrence_candidates` (`pub`, L62), `validate_candidates_with_llm` (`pub`, L113)

#### `crates/contextra-graph/src/lib.rs` (1)

- **trait GraphIndexExt**: `remove_entity` (`priv`, L123)

#### `crates/contextra-graph/src/path_rag/k_path.rs` (7)

- **Default for KPathConfig**: `default` (`priv`, L24)
- **trait KPathDiffusion**: `find_k_paths` (`priv`, L47)
- **PartialEq for HeapItem**: `eq` (`priv`, L62)
- **Ord for HeapItem**: `cmp` (`priv`, L70)
- **PartialOrd for HeapItem**: `partial_cmp` (`priv`, L79)
- ***(Modulfunktionen)***: `is_simple_path` (`priv`, L84)
- **KPathDiffusion for PathRAGEngine<G>**: `find_k_paths` (`priv`, L90)

#### `crates/contextra-graph/src/path_rag/mod.rs` (20)

- **trait PathGraph**: `neighbors_with_weights` (`priv`, L46), `predecessors_with_weights` (`priv`, L47), `hyperedges_for_entity` (`priv`, L50), `get_hyperedge` (`priv`, L55), `hyperedge_participants` (`priv`, L60)
- **Default for PprParams**: `default` (`priv`, L80)
- ***(Modulfunktionen)***: `forward_push_ppr` (`pub`, L92)
- **Default for PathRAGConfig**: `default` (`priv`, L177)
- **PathRAGEngine<G>**: `new` (`pub`, L193), `with_defaults` (`pub`, L205), `with_config` (`pub`, L209), `max_hops` (`pub`, L213), `sufficiency_threshold` (`pub`, L217), `get_expanded_neighbors` (`priv`, L223), `get_expanded_predecessors` (`priv`, L234), `expand_with_hyperedges` (`priv`, L243), `find_path` (`pub`, L266), `find_all_paths` (`pub`, L397), `sufficiency_check` (`pub`, L426), `to_rrf_signal` (`pub`, L435)

#### `crates/contextra-graph/src/path_rag/snapshot.rs` (5)

- **SnapshotPathGraph<**: `new` (`pub`, L27)
- **PathGraph for SnapshotPathGraph<**: `neighbors_with_weights` (`priv`, L33), `predecessors_with_weights` (`priv`, L108), `hyperedges_for_entity` (`priv`, L182), `get_hyperedge` (`priv`, L206)

#### `crates/contextra-graph/src/path_rag/tests.rs` (8)

- **TestGraph**: `new` (`priv`, L9)
- **PathGraph for TestGraph**: `neighbors_with_weights` (`priv`, L19), `predecessors_with_weights` (`priv`, L22)
- **TestGraphWithHyperedges**: `new` (`priv`, L184)
- **PathGraph for TestGraphWithHyperedges**: `neighbors_with_weights` (`priv`, L209), `predecessors_with_weights` (`priv`, L212), `hyperedges_for_entity` (`priv`, L228), `get_hyperedge` (`priv`, L231)

#### `crates/contextra-graph/src/percolation.rs` (5)

- **Default for PercolationConfig**: `default` (`priv`, L22)
- ***(Modulfunktionen)***: `compute_percolation_health` (`pub`, L34), `should_trigger_rebonding` (`pub`, L46), `cosine_similarity` (`priv`, L51), `find_rebonding_candidates` (`pub`, L71)

#### `crates/contextra-graph/src/ppr.rs` (16)

- **DeletedView**: `from_nodes` (`pub(crate)`, L35), `contains` (`pub`, L49), `len` (`pub`, L55), `is_empty` (`pub`, L61)
- **PprContext**: `new` (`pub`, L80), `prepare` (`priv`, L85)
- ***(Modulfunktionen)***: `compute_ppr` (`pub(crate)`, L100), `compute_ppr_with_apprh_selector` (`pub`, L112), `select_auto_ppr_algorithm` (`pub(crate)`, L158), `compute_ppr_with_context` (`pub(crate)`, L175), `forward_push_ppr` (`pub(crate)`, L449), `compute_ppr_dense` (`pub(crate)`, L766)
- **crate::path_rag::PathGraph for GraphInner**: `neighbors_with_weights` (`priv`, L334), `predecessors_with_weights` (`priv`, L381), `hyperedges_for_entity` (`priv`, L431), `get_hyperedge` (`priv`, L438)

#### `crates/contextra-graph/src/ppr/cost.rs` (9)

- **GraphStats**: `new` (`pub`, L20)
- **PprCostEstimate**: `cheapest_strategy` (`pub`, L47)
- **PprCostCalibrator**: `new` (`pub`, L76), `record` (`pub`, L81), `sample_count` (`pub`, L87), `recommended_multiplier` (`pub`, L95), `evaluate_recommended_multiplier` (`pub`, L100), `clear` (`pub`, L125)
- ***(Modulfunktionen)***: `estimate_ppr_cost` (`pub`, L135)

#### `crates/contextra-graph/src/ppr/shadow_hook.rs` (2)

- ***(Modulfunktionen)***: `log_tl_hfd_vs_baseline_discrepancy` (`pub`, L9), `log_tl_hfd_discrepancy_tracing` (`pub`, L49)

#### `crates/contextra-graph/src/ppr/snapshot.rs` (1)

- ***(Modulfunktionen)***: `compute_ppr_at` (`pub(crate)`, L26)

#### `crates/contextra-graph/src/ppr/tests/algo_tests.rs` (2)

- **tracing_subscriber::Layer<S> for LogCaptureLayer**: `on_event` (`priv`, L814)
- **tracing::field::Visit for StringVisitor**: `record_debug` (`priv`, L828)

#### `crates/contextra-graph/src/ppr/tests/context_tests.rs` (2)

- **tracing_subscriber::Layer<S> for LogCaptureLayer**: `on_event` (`priv`, L9)
- **tracing::field::Visit for StringVisitor**: `record_debug` (`priv`, L21)

#### `crates/contextra-graph/src/ppr_stream.rs` (10)

- **PprCandidateStream<**: `new` (`pub`, L33), `with_batch_size` (`pub`, L46), `with_exclude_seeds` (`pub`, L52), `next_batch` (`pub`, L59), `next_entity` (`pub`, L75), `next_batch_docs` (`pub`, L87), `yielded` (`pub`, L108), `is_exhausted` (`pub`, L113), `max_depth` (`pub`, L121), `ensure_calculated` (`priv`, L125)

#### `crates/contextra-graph/src/provenance.rs` (7)

- **EdgeProvenance**: `new` (`pub`, L27)
- **DocEdgeIndex**: `new` (`pub`, L45), `record` (`pub`, L53), `record_provenance` (`pub`, L59), `edges_for_doc` (`pub`, L82), `provenance_for_edge` (`pub`, L91), `remove_doc` (`pub`, L107)

#### `crates/contextra-graph/src/session_dag.rs` (25)

- **NodesGuard<**: `nodes` (`pub`, L36), `edges` (`pub`, L42), `edges_write` (`pub`, L46), `active_head` (`pub`, L50), `active_head_write` (`pub`, L54)
- **NodesWriteGuard<**: `nodes` (`pub`, L67), `nodes_mut` (`pub`, L71), `edges` (`pub`, L75), `edges_write` (`pub`, L79), `active_head` (`pub`, L83), `active_head_write` (`pub`, L87)
- **SessionBranchTree**: `new` (`pub`, L151), `lock_nodes` (`pub`, L173), `lock_nodes_write` (`pub`, L182), `branch_from_internal` (`priv`, L191), `append_step` (`pub`, L242), `branch_from` (`pub`, L272), `set_active_head` (`pub`, L296), `path_to_head` (`pub`, L311), `children_of` (`pub`, L351), `active_head` (`pub`, L362), `node_count` (`pub`, L369), `get_node` (`pub`, L374), `save` (`pub`, L379), `load` (`pub`, L427)

#### `crates/contextra-graph/src/tl_hfd/diffusion.rs` (1)

- ***(Modulfunktionen)***: `run_diffusion` (`pub`, L27)

#### `crates/contextra-graph/src/tl_hfd/error.rs` (1)

- **From<TlHfdError> for contextra_types::ContextraError**: `from` (`priv`, L18)

#### `crates/contextra-graph/src/tl_hfd/lovasz.rs` (2)

- ***(Modulfunktionen)***: `truncate_participants` (`pub`, L13), `compute_lovasz_extension` (`pub`, L54)

#### `crates/contextra-graph/src/tl_hfd/mod.rs` (3)

- ***(Modulfunktionen)***: `tl_hfd_local` (`pub`, L30)
- **CsrGraph**: `thresholded_local_hfd` (`pub`, L46), `evaluate_thresholded_local_hfd` (`pub`, L55)

#### `crates/contextra-graph/src/tl_hfd/params.rs` (2)

- **Default for TlHfdParams**: `default` (`priv`, L26)
- **TlHfdParams**: `validate` (`pub`, L46)

#### `crates/contextra-graph/src/tl_hfd/shadow.rs` (3)

- **trait DefaultFlipGate**: `should_flip` (`priv`, L36)
- ***(Modulfunktionen)***: `shadow_compare_forward_push_vs_tl_hfd` (`pub`, L72), `normalize_and_sort` (`priv`, L137)

### contextra-infer-candle

> Native Candle GGUF ML inference backend for Contextra

- **Pfad:** `crates/contextra-infer-candle` | **Dateien:** 15 | **Produktiv-Funktionen:** 118 (`65` `pub`, `53` intern) | **Tests:** 58
- **Öffentliche Typen (23)**: `CandleAttentionExporter`, `CandleEmbedInner`, `CandleEmbedClient`, `BertEmbedModel`, `DefaultCandleEmbedModel`, `GaspConfig`, `GaspValidator`, `GgufMetadata`, `CandleModelInner`, `CandleLlmClient`, `QuantizedLlamaModel`, `DefaultCandleLlmModel`, `PrefixSeed`, `PrefixRun`, `KvPrefixContext`, `KvCacheKey`, `KvBridgeAdapter`, `LayerKv`, `KvState`, `QuantizedMatMul`, `LayerWeights`, `ModelWeights`, `CandleQuantization`

#### `crates/contextra-infer-candle/src/attention_exporter.rs` (8)

- **Default for CandleAttentionExporter**: `default` (`priv`, L30)
- **CandleAttentionExporter**: `new` (`pub`, L37), `record_attention_weights` (`pub`, L46), `record_layer_head_scores` (`pub`, L68), `remove_request` (`pub`, L86), `tracked_request_count` (`pub`, L94), `clear` (`pub`, L99)
- **AttentionExporter for CandleAttentionExporter**: `export_attention_weights` (`priv`, L107)

#### `crates/contextra-infer-candle/src/embedding.rs` (12)

- **trait CandleEmbedInner**: `embed` (`priv`, L24), `dim` (`priv`, L32)
- **CandleEmbedClient**: `new` (`pub`, L55), `with_max_concurrent_embeddings` (`pub`, L76), `fingerprint` (`pub`, L84), `from_dir` (`pub`, L89), `from_dir_with_concurrency` (`pub`, L189)
- **BertEmbedModel**: `load` (`pub`, L207)
- **CandleEmbedInner for BertEmbedModel**: `embed` (`priv`, L238), `dim` (`priv`, L305)
- **CandleEmbedInner for DefaultCandleEmbedModel**: `embed` (`priv`, L317), `dim` (`priv`, L350)

#### `crates/contextra-infer-candle/src/embedding_provider.rs` (4)

- **EmbeddingProvider for CandleEmbedClient**: `provider_name` (`priv`, L19), `embedding_dim` (`priv`, L23), `embed` (`priv`, L28), `embed_batch` (`priv`, L56)

#### `crates/contextra-infer-candle/src/gasp.rs` (14)

- **Default for GaspConfig**: `default` (`priv`, L45)
- ***(Modulfunktionen)***: `contains_at_word_boundary` (`priv`, L64)
- **GaspValidator**: `new` (`pub`, L103), `with_config` (`pub`, L108), `set_threshold` (`pub`, L121), `record_external_feedback` (`pub`, L132), `threshold` (`pub`, L139), `config` (`pub`, L144), `refresh_config` (`pub`, L150), `observation_count` (`pub`, L158), `compute_raw_grounding_score` (`pub`, L170)
- **Default for GaspValidator**: `default` (`priv`, L263)
- **ResponseGroundingValidator for GaspValidator**: `score_grounding` (`priv`, L269)
- **GroundingValidator for GaspValidator**: `validate_grounding` (`priv`, L288)

#### `crates/contextra-infer-candle/src/gguf_loader.rs` (1)

- ***(Modulfunktionen)***: `parse_gguf_metadata` (`pub`, L25)

#### `crates/contextra-infer-candle/src/inference/core.rs` (31)

- **trait CandleModelInner**: `generate` (`priv`, L37), `generate_stream` (`priv`, L46), `last_attention_scores` (`priv`, L59), `kv_layout` (`priv`, L65), `generate_stream_with_prefix` (`priv`, L76)
- **CandleLlmClient**: `new` (`pub`, L137), `prefill_count` (`pub`, L166), `prefill_skip_count` (`pub`, L171), `prefill_skipped_tokens` (`pub`, L178), `tracked_request_count` (`pub`, L184), `with_prefix_store` (`pub`, L193), `from_dir_with_prefix_store` (`pub`, L200), `with_attention_exporter` (`pub`, L215), `with_id_gen` (`pub`, L224), `generate_with_prefix_store` (`pub`, L231), `with_max_concurrent_inferences` (`pub`, L354), `with_kv_bridge` (`pub`, L363), `fingerprint` (`pub`, L369), `swap_model` (`pub`, L376), `from_dir` (`pub`, L405)
- **QuantizedLlamaModel**: `load` (`pub`, L523)
- **CandleModelInner for QuantizedLlamaModel**: `kv_layout` (`priv`, L558), `generate` (`priv`, L583), `generate_stream` (`priv`, L597), `generate_stream_with_prefix` (`priv`, L697)
- **CandleModelInner for DefaultCandleLlmModel**: `generate` (`priv`, L872), `generate_stream` (`priv`, L881)
- **LlmTextGenerator for CandleLlmClient**: `generate` (`priv`, L905), `generate_with_context` (`priv`, L944), `generate_with_context` (`priv`, L997)
- **LlmTextGeneratorStreaming for CandleLlmClient**: `generate_stream` (`priv`, L1023)

#### `crates/contextra-infer-candle/src/inference/prefix_reuse.rs` (3)

- **fmt::Debug for KvPrefixContext**: `fmt` (`priv`, L43)
- ***(Modulfunktionen)***: `build_prefix_key` (`pub`, L51), `seed_from_hit` (`pub`, L77)

#### `crates/contextra-infer-candle/src/inference/tests.rs` (2)

- **CandleModelInner for MockCandleModel**: `generate` (`priv`, L16)
- **CandleModelInner for SlowCandleModel**: `generate` (`priv`, L483)

#### `crates/contextra-infer-candle/src/kv_bridge.rs` (11)

- **KvCacheKey**: `new` (`pub`, L40)
- **KvBridgeAdapter**: `new` (`pub`, L64), `with_lsm_fallback` (`pub`, L74), `consult_segment` (`pub`, L120), `consultation_count` (`pub`, L132), `validate_payload` (`priv`, L137), `try_get_cached_segment` (`pub`, L175), `try_get_cached_segment_async` (`pub`, L198), `try_get_cached_segment_async_bytes` (`pub`, L246), `store_segment` (`pub`, L257)
- **std::fmt::Debug for KvBridgeAdapter**: `fmt` (`priv`, L294)

#### `crates/contextra-infer-candle/src/kv_state.rs` (13)

- **LayerKv**: `new` (`pub`, L26)
- ***(Modulfunktionen)***: `tensor_to_payload` (`priv`, L59), `payload_to_tensor` (`priv`, L101)
- **KvState**: `new` (`pub`, L148), `is_empty` (`pub`, L153), `pos` (`pub`, L158), `layer_count` (`pub`, L163), `tensor_seq_dim` (`priv`, L167), `truncate` (`pub`, L185), `export_block` (`pub`, L209), `import_block_internal` (`priv`, L255), `import_block` (`pub`, L304), `import_block_at` (`pub`, L309)

#### `crates/contextra-infer-candle/src/model/quantized_llama.rs` (16)

- ***(Modulfunktionen)***: `repeat_kv` (`priv`, L17), `precomput_freqs_cis` (`priv`, L35), `build_causal_mask` (`priv`, L67)
- **QuantizedMatMul**: `from_qtensor` (`priv`, L88), `forward` (`priv`, L95)
- **Mlp**: `forward` (`priv`, L111)
- **MlpOrMoe**: `forward` (`priv`, L128)
- **LayerWeights**: `apply_rotary_emb` (`priv`, L159), `forward_attn` (`priv`, L184)
- **ModelWeights**: `from_gguf` (`pub`, L291), `mask` (`priv`, L424), `export_kv_state` (`pub`, L441), `import_kv_state` (`pub`, L462), `truncate_kv_cache` (`pub`, L479), `clear_kv_cache` (`pub`, L504), `forward` (`pub`, L511)

#### `crates/contextra-infer-candle/src/model_registry.rs` (3)

- **fmt::Display for CandleQuantization**: `fmt` (`priv`, L26)
- **FromStr for CandleQuantization**: `from_str` (`priv`, L38)
- ***(Modulfunktionen)***: `compute_fingerprint` (`pub`, L56)

### contextra-infer-ollama

- **Pfad:** `crates/contextra-infer-ollama` | **Dateien:** 15 | **Produktiv-Funktionen:** 85 (`60` `pub`, `25` intern) | **Tests:** 113
- **Öffentliche Typen (26)**: `MonotonicClock`, `SystemMonotonicClock`, `BreakerStatus`, `CircuitOpenError`, `CircuitBreaker`, `OllamaConfig`, `OllamaClient`, `ChatRequest`, `ChatMessage`, `ChatStreamChunk`, `ChatMessageResponse`, `GenerateRequest`, `GenerateResponse`, `EmbedRequest`, `EmbedResponse`, `BatchEmbedRequest`, `BatchEmbedResponse`, `ContextPrefixConfig`, `ContextPrefixEngine`, `ContextPrefixer`, `OllamaEmbedder`, `Confidence`, `ImportanceAssessment`, `ModelInfo`, `OllamaQueryRewriterConfig`, `OllamaQueryRewriter`

#### `crates/contextra-infer-ollama/src/client/circuit_breaker.rs` (9)

- **trait MonotonicClock**: `monotonic_nanos` (`priv`, L21)
- **MonotonicClock for SystemMonotonicClock**: `monotonic_nanos` (`priv`, L31)
- **fmt::Display for CircuitOpenError**: `fmt` (`priv`, L56)
- **CircuitBreaker**: `new` (`pub`, L84), `status` (`pub`, L97), `check` (`pub`, L108), `record_success` (`pub`, L120), `record_failure` (`pub`, L128), `advance_locked` (`priv`, L150)

#### `crates/contextra-infer-ollama/src/client/config.rs` (1)

- **Default for OllamaConfig**: `default` (`priv`, L35)

#### `crates/contextra-infer-ollama/src/client/core.rs` (23)

- ***(Modulfunktionen)***: `retry_jitter` (`pub(crate)`, L23)
- **std::fmt::Debug for OllamaClient**: `fmt` (`priv`, L36)
- **OllamaClient**: `new` (`pub`, L47), `with_config` (`pub`, L56), `with_rng` (`pub`, L85), `is_available` (`pub`, L91), `check_availability` (`pub`, L96), `with_defaults` (`pub`, L124), `base_url` (`pub`, L128), `config` (`pub`, L132), `embed_batch` (`pub`, L143), `try_embed_batch` (`pub`, L190), `list_models` (`pub`, L249), `is_model_available` (`pub`, L287), `generate_text` (`pub`, L316), `generate_text_stream` (`pub`, L353), `try_generate_text` (`pub`, L493), `generate` (`pub`, L552), `try_generate` (`pub`, L589), `ensure_model_available` (`pub`, L641), `embed` (`pub`, L652), `try_embed` (`pub`, L691), `chat_with_rag_streaming` (`pub`, L741)

#### `crates/contextra-infer-ollama/src/client/errors.rs` (3)

- ***(Modulfunktionen)***: `is_transient_network_error` (`pub`, L4), `classify_reqwest_error` (`pub`, L9), `is_transient_error` (`pub`, L32)

#### `crates/contextra-infer-ollama/src/client/tests.rs` (1)

- ***(Modulfunktionen)***: `verify_xml_structure` (`priv`, L213)

#### `crates/contextra-infer-ollama/src/client/trait_impls.rs` (4)

- **contextra_ports::LlmTextGenerator for OllamaClient**: `generate` (`priv`, L4)
- **contextra_ports::LlmTextGeneratorStreaming for OllamaClient**: `generate_stream` (`priv`, L13)
- **contextra_ports::SegmentSynthesizer for OllamaClient**: `synthesize_segment` (`priv`, L42), `model_id` (`priv`, L57)

#### `crates/contextra-infer-ollama/src/client/validation.rs` (5)

- ***(Modulfunktionen)***: `xml_escape` (`pub`, L5), `build_rag_prompt` (`pub`, L19), `validate_text_length` (`pub`, L29), `validate_batch_size` (`pub`, L40), `validate_model_name` (`pub`, L50)

#### `crates/contextra-infer-ollama/src/context_prefixer.rs` (7)

- **Default for ContextPrefixConfig**: `default` (`priv`, L36)
- **ContextPrefixEngine**: `new` (`pub`, L65), `generate_prefix` (`pub`, L77), `generate_prefix_batch` (`pub`, L124)
- ***(Modulfunktionen)***: `compute_max_prefix_chars` (`priv`, L141), `truncate_chars` (`pub`, L145), `truncate_prefix` (`pub`, L156)

#### `crates/contextra-infer-ollama/src/embedding.rs` (11)

- **OllamaEmbedder**: `new` (`pub`, L23), `with_config` (`pub`, L40), `with_defaults` (`pub`, L51), `config` (`pub`, L55), `with_concurrency` (`pub`, L59), `with_expected_dimension` (`pub`, L64), `model` (`pub`, L69)
- **EmbeddingProvider for OllamaEmbedder**: `provider_name` (`priv`, L75), `embed` (`priv`, L79), `embedding_dim` (`priv`, L109), `embed_batch` (`priv`, L113)

#### `crates/contextra-infer-ollama/src/importance.rs` (10)

- ***(Modulfunktionen)***: `score_importance_batch` (`pub`, L27), `get_score_regex` (`priv`, L142), `score_importance` (`pub`, L163), `score_importance_with_calibrator` (`pub`, L174), `record_importance_outcome` (`pub`, L229), `parse_importance_score_response` (`pub`, L240)
- **ImportanceAssessment**: `new` (`pub`, L107), `with_provenance` (`pub`, L117), `value` (`pub`, L132), `is_parsed` (`pub`, L137)

#### `crates/contextra-infer-ollama/src/model_info.rs` (3)

- **OllamaClient**: `validate_model_available` (`pub`, L23), `show_model` (`pub`, L35)
- ***(Modulfunktionen)***: `known_dimension` (`pub`, L105)

#### `crates/contextra-infer-ollama/src/query_rewriter.rs` (8)

- **Default for OllamaQueryRewriterConfig**: `default` (`priv`, L29)
- **OllamaQueryRewriter**: `new` (`pub`, L46), `config` (`pub`, L51)
- **QueryRewriter for OllamaQueryRewriter**: `rewrite_structured` (`priv`, L57)
- ***(Modulfunktionen)***: `extract_snippet` (`priv`, L85), `truncate_chars` (`priv`, L101), `build_rewrite_prompt` (`priv`, L108), `parse_rewrite_response` (`priv`, L154)

### contextra-infer-onnx

- **Pfad:** `crates/contextra-infer-onnx` | **Dateien:** 6 | **Produktiv-Funktionen:** 43 (`25` `pub`, `18` intern) | **Tests:** 34
- **Öffentliche Typen (6)**: `TextEmbedderConfig`, `OnnxEmbedder`, `TextEmbedder`, `RerankConfig`, `CrossEncoderReranker`, `OnnxReranker`

#### `crates/contextra-infer-onnx/src/lib.rs` (19)

- ***(Modulfunktionen)***: `default_model_cache_dir` (`pub`, L44), `ensure_onnx_model_download` (`pub`, L58), `download_onnx_file` (`priv`, L125), `ensure_onnx_model_download` (`pub`, L181)
- **Default for TextEmbedderConfig**: `default` (`priv`, L216)
- **TextEmbedderConfig**: `validate` (`pub`, L230)
- **std::fmt::Debug for TextEmbedder**: `fmt` (`priv`, L274)
- **EmbeddingProvider for TextEmbedder**: `provider_name` (`priv`, L284), `embed` (`priv`, L288), `embedding_dim` (`priv`, L317), `embed_batch` (`priv`, L321)
- **TextEmbedder**: `from_path` (`pub`, L374), `new` (`pub`, L385), `load` (`pub`, L392), `load_with_config` (`pub`, L397), `session_load_count` (`pub`, L447), `with_expected_dimension` (`pub`, L453), `embed_async` (`pub`, L462), `run_inference` (`priv`, L501)

#### `crates/contextra-infer-onnx/src/reranker/config.rs` (2)

- **Default for RerankConfig**: `default` (`priv`, L38)
- **RerankConfig**: `with_calibration` (`pub`, L54)

#### `crates/contextra-infer-onnx/src/reranker/cross_encoder.rs` (16)

- **CrossEncoderReranker**: `record_implicit_feedback` (`pub`, L45), `is_calibrated` (`pub`, L55), `calibration_observation_count` (`pub`, L60), `record_outcome` (`pub`, L66), `calibrate` (`pub`, L76), `invalidate_calibration` (`pub`, L81), `fitted_calibration` (`pub`, L88), `passthrough` (`pub`, L93), `passthrough_with_config` (`pub`, L98), `config` (`pub`, L112), `with_calibration` (`pub`, L117), `new` (`pub`, L124), `rerank` (`pub`, L153)
- **Reranker for CrossEncoderReranker**: `rerank` (`priv`, L194), `record_implicit_feedback` (`priv`, L202), `rerank_deadline_ms` (`priv`, L206)

#### `crates/contextra-infer-onnx/src/reranker/onnx.rs` (5)

- **OnnxReranker**: `new` (`pub(super)`, L43), `rerank` (`pub(super)`, L79), `score_pairs_blocking` (`priv`, L132), `score_batch` (`priv`, L156), `extract_scores_from_tensor_calibrated` (`priv`, L285)

#### `crates/contextra-infer-onnx/src/reranker/tests.rs` (1)

- ***(Modulfunktionen)***: `compute_ece` (`priv`, L395)

### contextra-ingest

> Document format extractors and structure-aware chunking adapter crate

- **Pfad:** `crates/contextra-ingest` | **Dateien:** 78 | **Produktiv-Funktionen:** 465 (`151` `pub`, `314` intern) | **Tests:** 65
- **Öffentliche Typen (85)**: `CanonicalText`, `ChunkingConfig`, `ProseOptions`, `TabularMode`, `TabularOptions`, `PagedOptions`, `StructuredOptions`, `EmailOptions`, `EmailChunker`, `FallbackChunker`, `ChunkDraft`, `IrChunker`, `PagedChunker`, `ParsedBlockKind`, `ParsedListItem`, `ParsedMarkdownBlock`, `ProseChunker`, `SentenceSpan`, `SplitContext`, `StructuredChunker`, `ProseChunker`, `TabularChunker`, `DocId`, `ContextChunkLink`, `ContextChunk`, `Confidence`, `DetectHints`, `Detection`, `LimitKind`, `IngestError`, `DocumentExtractor`, `ResourceBudget`, `NestGuard`, `ExtractContext`, `FormatId`, `ChunkStrategy`, `DocumentParseResult`, `DocxExtractor`, `ExtractedNote`, `NumMap` *(+45 weitere)*

#### `crates/contextra-ingest/src/canonical.rs` (4)

- ***(Modulfunktionen)***: `sanitize_text` (`pub`, L34), `render_block` (`priv`, L110)
- **DocumentIr**: `canonical` (`pub`, L82), `canonical_hash` (`pub`, L102)

#### `crates/contextra-ingest/src/chunking/common.rs` (5)

- ***(Modulfunktionen)***: `token_count` (`pub`, L26), `estimate_tokens` (`pub`, L32), `window_split` (`pub`, L100), `chunk_text_with_overlap` (`pub`, L120), `finalize` (`pub`, L165)

#### `crates/contextra-ingest/src/chunking/config.rs` (7)

- **Default for ProseOptions**: `default` (`priv`, L49)
- **Default for TabularOptions**: `default` (`priv`, L80)
- **Default for PagedOptions**: `default` (`priv`, L101)
- **Default for StructuredOptions**: `default` (`priv`, L120)
- **Default for EmailOptions**: `default` (`priv`, L138)
- **Default for ChunkingConfig**: `default` (`priv`, L147)
- **ChunkingConfig**: `fingerprint` (`pub`, L167)

#### `crates/contextra-ingest/src/chunking/email/mod.rs` (8)

- **IrChunker for EmailChunker**: `strategy` (`priv`, L31), `draft` (`priv`, L35)
- ***(Modulfunktionen)***: `parse_email_messages` (`priv`, L181), `emit_body_chunks` (`priv`, L295), `finalize_body_batch` (`priv`, L328), `emit_oversize_body_windows` (`priv`, L378), `build_short_header` (`priv`, L427), `determine_attachment_strategy` (`priv`, L449)

#### `crates/contextra-ingest/src/chunking/fallback.rs` (2)

- **IrChunker for FallbackChunker**: `strategy` (`priv`, L28), `draft` (`priv`, L32)

#### `crates/contextra-ingest/src/chunking/meta.rs` (2)

- ***(Modulfunktionen)***: `build_chunk_metadata` (`pub`, L22), `encode` (`pub`, L117)

#### `crates/contextra-ingest/src/chunking/mod.rs` (4)

- **trait IrChunker**: `strategy` (`priv`, L63), `draft` (`priv`, L64)
- ***(Modulfunktionen)***: `draft_document` (`pub`, L75), `draft_block_range` (`pub`, L96)

#### `crates/contextra-ingest/src/chunking/paged/mod.rs` (3)

- **IrChunker for PagedChunker**: `strategy` (`priv`, L29), `draft` (`priv`, L33)
- ***(Modulfunktionen)***: `draft_paged` (`pub`, L46)

#### `crates/contextra-ingest/src/chunking/paged/page.rs` (17)

- ***(Modulfunktionen)***: `process_paged_document` (`pub`, L39), `collect_initial_groups` (`priv`, L88), `build_breadcrumb` (`priv`, L209), `apply_cross_page_join` (`priv`, L238), `should_join_pages` (`priv`, L285), `is_sentence_terminal` (`priv`, L344), `get_block_text` (`priv`, L348), `is_group_empty` (`priv`, L364), `apply_merge_small_pages` (`priv`, L380), `calc_group_tokens` (`priv`, L436), `generate_drafts_for_group` (`priv`, L448), `make_locator` (`priv`, L528), `get_span_and_text` (`priv`, L540), `split_large_group` (`priv`, L552), `split_table_block` (`priv`, L655), `split_paragraph_block` (`priv`, L754), `find_sub_span` (`priv`, L791)

#### `crates/contextra-ingest/src/chunking/prose/markdown_parse.rs` (18)

- ***(Modulfunktionen)***: `parse_markdown_blocks` (`pub`, L88), `collect_lines` (`priv`, L203), `try_parse_yaml_front_matter` (`priv`, L231), `try_parse_fenced_code` (`priv`, L265), `parse_atx_heading` (`priv`, L347), `try_parse_setext_heading` (`priv`, L382), `try_parse_table` (`priv`, L424), `is_table_row` (`priv`, L478), `is_table_delimiter_row` (`priv`, L486), `try_parse_note` (`priv`, L491), `is_thematic_break` (`priv`, L537), `parse_blockquote` (`priv`, L550), `is_list_item_start` (`priv`, L596), `parse_list` (`priv`, L610), `is_indented_code` (`priv`, L680), `parse_indented_code` (`priv`, L684), `parse_html_block` (`priv`, L728), `parse_paragraph` (`priv`, L769)

#### `crates/contextra-ingest/src/chunking/prose/merge.rs` (4)

- ***(Modulfunktionen)***: `merge_small_chunks` (`pub`, L21), `attach_headings` (`priv`, L87), `is_heading_only` (`priv`, L148), `is_same_main_section` (`priv`, L153)

#### `crates/contextra-ingest/src/chunking/prose/mod.rs` (4)

- **IrChunker for ProseChunker**: `strategy` (`priv`, L40), `draft` (`priv`, L44)
- ***(Modulfunktionen)***: `draft_prose` (`pub`, L91), `collect_intermediate_blocks` (`priv`, L347)

#### `crates/contextra-ingest/src/chunking/prose/sentence.rs` (2)

- ***(Modulfunktionen)***: `split_sentences` (`pub`, L35), `ends_with_abbreviation` (`priv`, L90)

#### `crates/contextra-ingest/src/chunking/prose/split.rs` (5)

- ***(Modulfunktionen)***: `split_oversized_code` (`pub`, L38), `split_oversized_table` (`pub`, L125), `split_oversized_list` (`pub`, L221), `split_oversized_paragraph` (`pub`, L292), `calculate_sentence_overlap` (`priv`, L411)

#### `crates/contextra-ingest/src/chunking/structured/mod.rs` (12)

- **IrChunker for StructuredChunker**: `strategy` (`priv`, L30), `draft` (`priv`, L34)
- ***(Modulfunktionen)***: `group_tokens` (`priv`, L127), `build_groups` (`priv`, L137), `split_or_collect_group` (`priv`, L159), `build_groups_with_original_indices` (`priv`, L203), `emit_batch_draft` (`priv`, L231), `emit_oversize_record_windows` (`priv`, L317), `parse_path_segments` (`priv`, L365), `get_path_prefix` (`priv`, L418), `format_segments` (`priv`, L436), `compute_batch_breadcrumb` (`priv`, L449)

#### `crates/contextra-ingest/src/chunking/stubs.rs` (4)

- **IrChunker for ProseChunker**: `strategy` (`priv`, L31), `draft` (`priv`, L35)
- **IrChunker for TabularChunker**: `strategy` (`priv`, L54), `draft` (`priv`, L58)

#### `crates/contextra-ingest/src/chunking/tabular/helpers.rs` (2)

- ***(Modulfunktionen)***: `col_to_letter` (`pub`, L17), `format_a1_range` (`pub`, L37)

#### `crates/contextra-ingest/src/chunking/tabular/mod.rs` (8)

- ***(Modulfunktionen)***: `draft_tabular` (`pub`, L43), `process_table` (`priv`, L99), `format_row_group` (`priv`, L369), `handle_oversized_row` (`priv`, L450), `build_draft` (`priv`, L592), `compute_row_byte_spans` (`priv`, L639), `get_span_for_rows` (`priv`, L684), `maybe_merge_mini_sheets` (`priv`, L705)

#### `crates/contextra-ingest/src/chunking/types.rs` (4)

- **DocId**: `new` (`pub`, L22), `as_u64` (`pub`, L28)
- **fmt::Display for DocId**: `fmt` (`priv`, L34)
- **From<u64> for DocId**: `from` (`priv`, L40)

#### `crates/contextra-ingest/src/detect/mod.rs` (20)

- ***(Modulfunktionen)***: `detect_reader` (`pub`, L57), `detect` (`pub`, L76), `resolve_hint_format` (`priv`, L144), `detect_content_format` (`priv`, L168), `is_pdf` (`priv`, L232), `is_rtf` (`priv`, L244), `is_zip_container` (`priv`, L248), `is_ole_container` (`priv`, L252), `is_gzip` (`priv`, L256), `byte_at` (`priv`, L261), `inspect_zip_container` (`priv`, L269), `inspect_ole_container` (`priv`, L364), `contains_slice` (`priv`, L402), `detect_text_heuristics` (`priv`, L410), `decode_utf16_le` (`priv`, L548), `decode_utf16_be` (`priv`, L566), `is_markdown` (`priv`, L584), `is_eml_header` (`priv`, L610), `is_text_format` (`priv`, L632), `is_ole_format` (`priv`, L652)

#### `crates/contextra-ingest/src/extractor.rs` (11)

- **trait DocumentExtractor**: `formats` (`priv`, L26), `extract` (`priv`, L29)
- **ResourceBudget**: `new` (`pub`, L51), `charge_decompressed` (`pub`, L64), `charge_xml_events` (`pub`, L79), `charge_text` (`pub`, L94), `charge_cells` (`pub`, L109), `charge_pages` (`pub`, L124), `enter_nested` (`pub`, L139)
- **Drop for NestGuard<**: `drop` (`priv`, L161)
- **ExtractContext<**: `extract_nested` (`pub`, L177)

#### `crates/contextra-ingest/src/format.rs` (5)

- **FormatId**: `name` (`pub`, L68), `extensions` (`pub`, L104), `mime` (`pub`, L140), `from_extension` (`pub`, L180), `default_strategy` (`pub`, L218)

#### `crates/contextra-ingest/src/formats/docx/document.rs` (4)

- ***(Modulfunktionen)***: `parse_document_xml` (`pub`, L52), `append_run_text` (`priv`, L416), `emit_paragraph` (`priv`, L447), `emit_table` (`priv`, L504)

#### `crates/contextra-ingest/src/formats/docx/mod.rs` (6)

- **DocxExtractor**: `new` (`pub`, L34)
- **DocumentExtractor for DocxExtractor**: `formats` (`priv`, L42), `extract` (`priv`, L46)
- ***(Modulfunktionen)***: `parse_core_properties` (`priv`, L263), `get_parent_dir` (`priv`, L305), `get_filename` (`priv`, L312)

#### `crates/contextra-ingest/src/formats/docx/notes.rs` (3)

- ***(Modulfunktionen)***: `parse_footnotes_or_endnotes` (`pub`, L31), `parse_comments` (`pub`, L112), `parse_header_footer` (`pub`, L166)

#### `crates/contextra-ingest/src/formats/docx/numbering.rs` (2)

- **NumMap**: `parse` (`pub`, L35), `is_ordered` (`pub`, L137)

#### `crates/contextra-ingest/src/formats/docx/styles.rs` (3)

- **StyleMap**: `parse` (`pub`, L42), `heading_level` (`pub`, L149)
- ***(Modulfunktionen)***: `match_heading_string` (`pub`, L190)

#### `crates/contextra-ingest/src/formats/email/eml.rs` (14)

- ***(Modulfunktionen)***: `extract_eml` (`pub`, L30), `extract_message_headers` (`priv`, L80), `extract_addrs` (`priv`, L113), `push_addr` (`priv`, L134), `sanitize_header_str` (`priv`, L150), `extract_body` (`priv`, L156), `extract_part_text` (`priv`, L188), `process_body_lines` (`priv`, L227), `build_body_blocks` (`priv`, L242), `strip_quote_prefix` (`priv`, L277), `flush_paragraph` (`priv`, L286), `flush_quote` (`priv`, L301), `extract_attachments` (`priv`, L316), `is_inline_image` (`priv`, L383)

#### `crates/contextra-ingest/src/formats/email/html.rs` (7)

- ***(Modulfunktionen)***: `strip_html` (`pub`, L23), `match_tag_start` (`priv`, L68), `match_block_tag` (`priv`, L90), `skip_tag` (`priv`, L113), `skip_until_end_tag` (`priv`, L124), `decode_entity` (`priv`, L157), `utf8_char_len` (`priv`, L190)

#### `crates/contextra-ingest/src/formats/email/mbox.rs` (4)

- ***(Modulfunktionen)***: `extract_mbox` (`pub`, L24), `split_mbox_messages` (`priv`, L101), `starts_with_from` (`priv`, L145), `unescape_mbox_from` (`priv`, L149)

#### `crates/contextra-ingest/src/formats/email/mod.rs` (3)

- **EmailExtractor**: `new` (`pub`, L30)
- **DocumentExtractor for EmailExtractor**: `formats` (`priv`, L38), `extract` (`priv`, L42)

#### `crates/contextra-ingest/src/formats/odf/content.rs` (8)

- ***(Modulfunktionen)***: `parse_content` (`pub`, L27), `append_to_current_text` (`priv`, L390), `parse_attr_u8` (`priv`, L403), `parse_attr_u64` (`priv`, L411), `get_attr_value` (`priv`, L419), `check_hidden_attr` (`priv`, L434), `check_condition_attr` (`priv`, L451), `check_ordered_attr` (`priv`, L463)

#### `crates/contextra-ingest/src/formats/odf/meta.rs` (1)

- ***(Modulfunktionen)***: `parse_meta` (`pub`, L24)

#### `crates/contextra-ingest/src/formats/odf/mod.rs` (7)

- **OdfExtractor**: `new` (`pub`, L31)
- **DocumentExtractor for OdfExtractor**: `formats` (`priv`, L39), `extract` (`priv`, L43)
- ***(Modulfunktionen)***: `is_flat_odf` (`priv`, L72), `extract_flat_odf` (`priv`, L79), `extract_zip_odf` (`priv`, L105), `check_manifest_encrypted` (`priv`, L195)

#### `crates/contextra-ingest/src/formats/odf/presentation.rs` (3)

- ***(Modulfunktionen)***: `parse_presentation` (`pub`, L24), `is_page_hidden` (`priv`, L201), `get_attr_value` (`priv`, L218)

#### `crates/contextra-ingest/src/formats/ole/cfb_reader.rs` (7)

- **SafeCfbReader<**: `open` (`pub`, L34), `exists` (`pub`, L45), `is_stream` (`pub`, L50), `is_storage` (`pub`, L55), `read_stream` (`pub`, L60), `list_children` (`pub`, L105), `walk_all` (`pub`, L133)

#### `crates/contextra-ingest/src/formats/ole/doc.rs` (11)

- ***(Modulfunktionen)***: `read_u16_le` (`priv`, L43), `read_u32_le` (`priv`, L49), `extract_doc` (`pub`, L69), `parse_clx` (`priv`, L154), `parse_plc_pcd` (`priv`, L182), `decode_pieces` (`priv`, L228), `windows_1252_to_char` (`priv`, L264), `process_cp_ranges` (`priv`, L298), `emit_doc_paragraphs` (`priv`, L334), `push_doc_block` (`priv`, L383), `fallback_heuristics` (`priv`, L397)

#### `crates/contextra-ingest/src/formats/ole/mod.rs` (3)

- **OleExtractor**: `new` (`pub`, L32)
- **DocumentExtractor for OleExtractor**: `formats` (`priv`, L40), `extract` (`priv`, L44)

#### `crates/contextra-ingest/src/formats/ole/msg.rs` (11)

- ***(Modulfunktionen)***: `extract_msg` (`pub`, L47), `extract_msg_in_storage` (`priv`, L55), `read_property_string` (`priv`, L139), `read_property_u32` (`priv`, L184), `read_property_filetime` (`priv`, L211), `filetime_to_iso8601` (`priv`, L255), `is_leap_year` (`priv`, L314), `parse_recipients` (`priv`, L319), `parse_attachments` (`priv`, L368), `strip_html_coarse` (`priv`, L497), `starts_with_chars` (`priv`, L554)

#### `crates/contextra-ingest/src/formats/ole/ppt.rs` (6)

- **RecordHeader**: `is_container` (`priv`, L51)
- ***(Modulfunktionen)***: `read_record_header` (`priv`, L57), `extract_ppt` (`pub`, L90), `parse_ppt_records` (`priv`, L111), `parse_ppt_atom` (`priv`, L174), `process_extracted_text` (`priv`, L214)

#### `crates/contextra-ingest/src/formats/pdf/mod.rs` (5)

- **PdfExtractor**: `new` (`pub`, L43)
- **DocumentExtractor for PdfExtractor**: `formats` (`priv`, L51), `extract` (`priv`, L55)
- ***(Modulfunktionen)***: `extract_metadata` (`priv`, L291), `extract_outlines` (`priv`, L317)

#### `crates/contextra-ingest/src/formats/pdf/text_processing.rs` (5)

- ***(Modulfunktionen)***: `clean_page_text` (`pub`, L20), `join_end_of_line_hyphens` (`pub`, L41), `reconstruct_paragraphs` (`pub`, L99), `normalize_line_digits` (`pub`, L138), `analyze_headers_footers` (`pub`, L159)

#### `crates/contextra-ingest/src/formats/pptx/mod.rs` (4)

- **PptxExtractor**: `new` (`pub`, L33)
- **DocumentExtractor for PptxExtractor**: `formats` (`priv`, L41), `extract` (`priv`, L45)
- ***(Modulfunktionen)***: `extract_core_metadata` (`priv`, L100)

#### `crates/contextra-ingest/src/formats/pptx/presentation.rs` (1)

- ***(Modulfunktionen)***: `parse_presentation_slides` (`pub`, L31)

#### `crates/contextra-ingest/src/formats/pptx/slide.rs` (14)

- ***(Modulfunktionen)***: `parse_slide_content` (`pub`, L34), `get_rels_path` (`priv`, L142), `find_sp_tree` (`priv`, L150), `extract_slide_title` (`priv`, L167), `find_title_in_container` (`priv`, L172), `is_placeholder_type` (`priv`, L194), `find_tbl_in_graphic_frame` (`priv`, L215), `process_shape_container` (`priv`, L225), `process_tx_body` (`priv`, L324), `parse_paragraph` (`priv`, L356), `process_table` (`priv`, L397), `collect_tx_body_text` (`priv`, L451), `extract_notes_text` (`priv`, L465), `extract_comments_text` (`priv`, L486)

#### `crates/contextra-ingest/src/formats/sheets/csv_fmt.rs` (7)

- ***(Modulfunktionen)***: `extract_csv_tsv` (`pub`, L25), `decode_text` (`priv`, L98), `detect_delimiter` (`priv`, L123), `count_delimiter_outside_quotes` (`priv`, L169), `build_tables_from_rows` (`pub`, L186), `process_and_push_table` (`priv`, L213), `determine_header_rows` (`pub`, L289)

#### `crates/contextra-ingest/src/formats/sheets/excel.rs` (14)

- ***(Modulfunktionen)***: `extract_excel` (`pub`, L27), `map_calamine_error` (`priv`, L66), `extract_xlsx` (`priv`, L80), `extract_xlsb` (`priv`, L125), `extract_ods` (`priv`, L165), `extract_xls` (`priv`, L194), `create_ir` (`priv`, L228), `should_process_sheet` (`priv`, L237), `check_range_size` (`priv`, L267), `apply_merge_cells` (`priv`, L284), `process_range` (`priv`, L297), `format_cell_data` (`priv`, L317), `excel_serial_to_iso8601` (`priv`, L332), `format_cell_error` (`priv`, L366)

#### `crates/contextra-ingest/src/formats/sheets/mod.rs` (3)

- **SheetsExtractor**: `new` (`pub`, L29)
- **DocumentExtractor for SheetsExtractor**: `formats` (`priv`, L44), `extract` (`priv`, L48)

#### `crates/contextra-ingest/src/formats/structured/json.rs` (18)

- **serde::de::DeserializeSeed<**: `deserialize` (`priv`, L46)
- **Visitor<**: `expecting` (`priv`, L57), `visit_bool` (`priv`, L61), `visit_i64` (`priv`, L65), `visit_u64` (`priv`, L69), `visit_f64` (`priv`, L73), `visit_str` (`priv`, L81), `visit_string` (`priv`, L88), `visit_none` (`priv`, L92), `visit_some` (`priv`, L96), `visit_unit` (`priv`, L103), `visit_seq` (`priv`, L107), `visit_map` (`priv`, L128)
- **JsonValue**: `parse` (`pub`, L152)
- ***(Modulfunktionen)***: `extract_json` (`pub`, L178), `extract_jsonl` (`pub`, L203), `emit_records` (`priv`, L262), `push_record` (`priv`, L312)

#### `crates/contextra-ingest/src/formats/structured/mod.rs` (3)

- **StructuredExtractor**: `new` (`pub`, L33)
- **DocumentExtractor for StructuredExtractor**: `formats` (`priv`, L47), `extract` (`priv`, L51)

#### `crates/contextra-ingest/src/formats/structured/path.rs` (4)

- ***(Modulfunktionen)***: `needs_quoting` (`pub`, L20), `escape_json_string` (`pub`, L37), `append_key` (`pub`, L61), `append_index` (`pub`, L77)

#### `crates/contextra-ingest/src/formats/structured/toml.rs` (3)

- ***(Modulfunktionen)***: `extract_toml` (`pub`, L25), `emit_toml_value` (`priv`, L54), `push_record` (`priv`, L111)

#### `crates/contextra-ingest/src/formats/structured/xml.rs` (4)

- ***(Modulfunktionen)***: `extract_xml` (`pub`, L40), `emit_element` (`priv`, L199), `emit_element_at_path` (`priv`, L250), `push_record` (`priv`, L299)

#### `crates/contextra-ingest/src/formats/structured/yaml.rs` (4)

- ***(Modulfunktionen)***: `extract_yaml` (`pub`, L26), `emit_yaml_node` (`priv`, L85), `yaml_key_to_string` (`priv`, L171), `push_record` (`priv`, L182)

#### `crates/contextra-ingest/src/formats/text/asciidoc.rs` (2)

- ***(Modulfunktionen)***: `parse_asciidoc` (`pub`, L22), `strip_admonition` (`priv`, L186)

#### `crates/contextra-ingest/src/formats/text/encoding.rs` (6)

- ***(Modulfunktionen)***: `decode_text` (`pub`, L22), `decode_raw_bytes` (`priv`, L45), `decode_utf16_le` (`priv`, L111), `decode_utf16_be` (`priv`, L125), `decode_utf32_le` (`priv`, L139), `decode_utf32_be` (`priv`, L156)

#### `crates/contextra-ingest/src/formats/text/markdown.rs` (2)

- ***(Modulfunktionen)***: `parse_markdown` (`pub`, L21), `extract_title` (`priv`, L37)

#### `crates/contextra-ingest/src/formats/text/mod.rs` (3)

- **TextExtractor**: `new` (`pub`, L34)
- **DocumentExtractor for TextExtractor**: `formats` (`priv`, L49), `extract` (`priv`, L53)

#### `crates/contextra-ingest/src/formats/text/rst.rs` (5)

- ***(Modulfunktionen)***: `parse_rst` (`pub`, L22), `is_adornment_line` (`priv`, L173), `strip_ordered_prefix` (`priv`, L187), `parse_rst_indented_block` (`priv`, L203), `parse_rst_table` (`priv`, L230)

#### `crates/contextra-ingest/src/formats/text/rtf.rs` (4)

- **Default for RtfGroupState**: `default` (`priv`, L33)
- ***(Modulfunktionen)***: `parse_rtf` (`pub`, L50), `push_char` (`priv`, L347), `decode_cp1252` (`priv`, L366)

#### `crates/contextra-ingest/src/formats/text/tex.rs` (5)

- ***(Modulfunktionen)***: `parse_tex` (`pub`, L30), `strip_tex_comments` (`priv`, L122), `parse_tex_heading` (`priv`, L149), `clean_tex_line` (`priv`, L172), `parse_tex_tabular` (`priv`, L241)

#### `crates/contextra-ingest/src/formats/text/txt.rs` (1)

- ***(Modulfunktionen)***: `parse_txt` (`pub`, L21)

#### `crates/contextra-ingest/src/formats/web/encoding.rs` (2)

- ***(Modulfunktionen)***: `decode_html_bytes` (`pub`, L30), `extract_meta_charset` (`priv`, L65)

#### `crates/contextra-ingest/src/formats/web/epub.rs` (8)

- ***(Modulfunktionen)***: `extract_epub` (`pub`, L29), `check_epub_encryption` (`priv`, L123), `get_container_opf_path` (`priv`, L169), `parse_opf` (`priv`, L219), `parse_ncx_titles` (`priv`, L318), `read_zip_entry` (`priv`, L366), `get_parent_dir` (`priv`, L386), `resolve_relative_path` (`priv`, L395)

#### `crates/contextra-ingest/src/formats/web/html.rs` (38)

- **DomArena**: `new` (`pub`, L67), `check_node_depth` (`priv`, L82)
- **TreeSink for DomArena**: `finish` (`priv`, L107), `parse_error` (`priv`, L111), `get_document` (`priv`, L113), `set_quirks_mode` (`priv`, L117), `same_node` (`priv`, L121), `elem_name` (`priv`, L125), `get_template_contents` (`priv`, L135), `append` (`priv`, L139), `append_based_on_parent_node` (`priv`, L166), `append_before_sibling` (`priv`, L180), `append_doctype_to_document` (`priv`, L187), `add_attrs_if_missing` (`priv`, L201), `remove_from_parent` (`priv`, L216), `reparent_children` (`priv`, L224), `is_mathml_annotation_xml_integration_point` (`priv`, L232), `create_element` (`priv`, L236), `create_comment` (`priv`, L252), `create_pi` (`priv`, L260)
- ***(Modulfunktionen)***: `destroy_tree_iteratively` (`pub`, L270), `extract_html` (`pub`, L282), `extract_from_arena` (`priv`, L320), `scan_arena_structure` (`priv`, L334), `collect_node_text` (`priv`, L399), `collapse_whitespace` (`priv`, L417), `is_hidden_element` (`priv`, L762), `extract_language` (`priv`, L782), `is_header_tr` (`priv`, L811), `extract_tr_cells` (`priv`, L827), `flatten_table_grid` (`priv`, L875)
- **HtmlBlockBuilder<**: `new` (`priv`, L469), `traverse` (`priv`, L487), `handle_element_enter` (`priv`, L595), `handle_element_exit` (`priv`, L638), `switch_block` (`priv`, L653), `flush_text_block` (`priv`, L659), `process_table` (`priv`, L698)

#### `crates/contextra-ingest/src/formats/web/mod.rs` (3)

- **WebExtractor**: `new` (`pub`, L30)
- **DocumentExtractor for WebExtractor**: `formats` (`priv`, L38), `extract` (`priv`, L42)

#### `crates/contextra-ingest/src/fuzz_support.rs` (1)

- ***(Modulfunktionen)***: `run` (`pub`, L24)

#### `crates/contextra-ingest/src/ir.rs` (3)

- **DocumentIr**: `new` (`pub`, L177), `push` (`pub`, L187), `append_nested` (`pub`, L192)

#### `crates/contextra-ingest/src/limits.rs` (2)

- **Default for IngestLimits**: `default` (`priv`, L50)
- **Default for ExtractPolicy**: `default` (`priv`, L85)

#### `crates/contextra-ingest/src/pipeline.rs` (1)

- ***(Modulfunktionen)***: `ingest` (`pub`, L56)

#### `crates/contextra-ingest/src/registry.rs` (4)

- **ExtractorRegistry**: `new_empty` (`pub`, L33), `register` (`pub`, L40), `with_defaults` (`pub`, L48), `extract` (`pub`, L88)

#### `crates/contextra-ingest/src/safe/archive.rs` (10)

- **SafeZipReader<**: `new` (`pub`, L30), `read_file` (`pub`, L41), `contains_file` (`pub`, L89), `file_names` (`pub`, L94)
- **SafeArchive<**: `new` (`pub`, L113), `read_entry` (`pub`, L133), `has_entry` (`pub`, L203), `list_entries` (`pub`, L221)
- ***(Modulfunktionen)***: `validate_archive_entry_path` (`pub`, L233), `validate_zip_archive` (`pub`, L272)

#### `crates/contextra-ingest/src/safe/opc.rs` (12)

- **OpcRels**: `parse` (`pub`, L41), `target_for_id` (`pub`, L65), `get_target` (`pub`, L70), `target_for_type` (`pub`, L75), `find_targets_by_type_suffix` (`pub`, L85)
- **OpcPackage<**: `open` (`pub`, L104), `has_part` (`pub`, L110), `read_part` (`pub`, L115), `parse_relationships` (`pub`, L124), `main_document_part` (`pub`, L180), `resolve_target` (`pub`, L204)
- ***(Modulfunktionen)***: `resolve_opc_target` (`pub`, L210)

#### `crates/contextra-ingest/src/safe/xml.rs` (10)

- **SafeXmlReader<**: `new` (`pub`, L33), `next_event` (`pub`, L47)
- **XmlElement**: `local_name` (`pub`, L117), `attr` (`pub`, L125), `child_elements` (`pub`, L142), `find_children` (`pub`, L153), `find_first_child` (`pub`, L166), `collect_text` (`pub`, L178), `collect_text_into` (`priv`, L184)
- ***(Modulfunktionen)***: `parse_xml` (`pub`, L195)

### contextra-kvcache

> Ring 1 Prefix-Radix tree, KV-Block cache, tenant-isolated memory store and tiering for Contextra

- **Pfad:** `crates/contextra-kvcache` | **Dateien:** 9 | **Produktiv-Funktionen:** 167 (`113` `pub`, `54` intern) | **Tests:** 49
- **Öffentliche Typen (27)**: `AttentionScoreSource`, `NullAttentionScoreSource`, `EvictionWorker`, `TenantPrefixKvStore`, `KiviQuantizeConfig`, `KvTensorView`, `KiviBlockMeta`, `KiviQuantizedBlock`, `KvReusePolicy`, `KvBlockGuard`, `PrefixMatch`, `PrefixRadixTree`, `KvSegmentRef`, `KvLookupResult`, `SemanticEmbedder`, `SemanticCacheConfig`, `ContentAddressedKvStore`, `ScratchpadCacheScope`, `ScratchpadInvalidator`, `PrefixStoreScratchpadInvalidator`, `KvSegmentContent`, `ShreddableSegmentKey`, `Tier2EncryptedSegment`, `EncryptedSegmentPayload`, `KvSegment`, `SpillHandler`, `TenantIsolatedKvStore`

#### `crates/contextra-kvcache/src/attention_score.rs` (8)

- **trait AttentionScoreSource**: `importance_score` (`priv`, L22), `importance_score_for_tenant` (`priv`, L27), `register_segment` (`priv`, L32), `unregister_segment` (`priv`, L35)
- **AttentionScoreSource for NullAttentionScoreSource**: `importance_score` (`priv`, L43)
- ***(Modulfunktionen)***: `rank_for_eviction` (`pub`, L53), `rank_for_eviction_weighted` (`pub`, L66), `rank_for_eviction_weighted_for_tenant` (`pub`, L75)

#### `crates/contextra-kvcache/src/eviction_worker.rs` (15)

- **ExporterBackedScoreSource**: `new` (`priv`, L41)
- **AttentionScoreSource for ExporterBackedScoreSource**: `importance_score` (`priv`, L50), `importance_score_for_tenant` (`priv`, L54), `register_segment` (`priv`, L71), `unregister_segment` (`priv`, L77)
- **EvictionWorker**: `spawn` (`pub`, L93), `with_attention_exporter` (`pub`, L139), `register_segment_request` (`pub`, L148), `unregister_segment_request` (`pub`, L159), `attention_source` (`pub`, L164), `trigger_eviction` (`pub`, L169), `notify_block_released` (`pub`, L177), `shutdown` (`pub`, L185)
- **Drop for EvictionWorker**: `drop` (`priv`, L194)
- ***(Modulfunktionen)***: `emergency_wipe` (`pub`, L201)

#### `crates/contextra-kvcache/src/prefix_store.rs` (16)

- **Partition**: `new` (`priv`, L37)
- **std::fmt::Debug for Partition**: `fmt` (`priv`, L47)
- **TenantPrefixKvStore**: `new` (`pub`, L70), `with_capacity_and_policy` (`pub`, L81), `with_byte_budget_per_tenant` (`pub`, L88), `with_reuse_policy` (`pub`, L94), `lookup` (`pub`, L100), `insert` (`pub`, L122), `evict` (`pub`, L183), `invalidate_prefix_scope_keys` (`pub`, L197), `evict_to_budget_internal` (`priv`, L253)
- **Default for TenantPrefixKvStore**: `default` (`priv`, L310)
- **std::fmt::Debug for TenantPrefixKvStore**: `fmt` (`priv`, L318)
- **KvPrefixStore for TenantPrefixKvStore**: `lookup` (`priv`, L329), `insert` (`priv`, L333), `evict` (`priv`, L343)

#### `crates/contextra-kvcache/src/quantize_kivi.rs` (10)

- **Default for KiviQuantizeConfig**: `default` (`priv`, L21)
- **KvTensorView**: `new` (`pub`, L44), `to_bytes` (`pub`, L78), `from_bytes` (`pub`, L83)
- ***(Modulfunktionen)***: `kivi_quantize` (`pub`, L144), `kivi_dequantize` (`pub`, L333), `unpack_kivi_block` (`pub`, L480), `pack_kivi_block` (`pub`, L527), `compress_bytes` (`pub`, L606), `decompress_bytes` (`pub`, L639)

#### `crates/contextra-kvcache/src/radix.rs` (35)

- **Default for KvReusePolicy**: `default` (`priv`, L34)
- **KvReusePolicy**: `should_reuse` (`pub`, L41)
- **KvBlockGuard**: `new` (`pub`, L61), `block_id` (`pub`, L77), `tenant_id` (`pub`, L82), `active_refs` (`pub`, L87)
- **Drop for KvBlockGuard**: `drop` (`priv`, L93)
- **std::fmt::Debug for KvBlockGuard**: `fmt` (`priv`, L109)
- **RadixNode**: `new` (`priv`, L137), `lcp_len` (`priv`, L145)
- **PrefixRadixTree**: `new` (`pub`, L159), `tenant_id` (`pub`, L168), `insert` (`pub`, L173), `insert_into_vec` (`priv`, L189), `find_longest_prefix` (`pub`, L243), `remove` (`pub`, L297), `remove_from_vec` (`priv`, L304), `len` (`pub`, L327), `is_empty` (`pub`, L332), `clear` (`pub`, L337)
- **KvSegmentRef**: `new` (`pub`, L589), `matched_len` (`pub`, L598)
- **trait SemanticEmbedder**: `embed` (`priv`, L623)
- **SemanticCacheConfig**: `new` (`pub`, L636)
- **ContentAddressedKvStore**: `new` (`pub`, L658), `with_reuse_policy` (`pub`, L670), `with_semantic_config` (`pub`, L676), `position_index` (`pub`, L682), `hash_tokens` (`pub`, L687), `insert` (`pub`, L696), `lookup` (`pub`, L738), `remove` (`pub`, L799), `len` (`pub`, L821), `is_empty` (`pub`, L826)
- ***(Modulfunktionen)***: `cosine_similarity` (`priv`, L832)

#### `crates/contextra-kvcache/src/scratchpad_invalidation.rs` (5)

- **ScratchpadCacheScope**: `new` (`pub`, L29)
- **trait ScratchpadInvalidator**: `invalidate_scope` (`priv`, L43)
- **PrefixStoreScratchpadInvalidator<**: `new` (`pub`, L56), `with_keys` (`pub`, L64)
- **ScratchpadInvalidator for PrefixStoreScratchpadInvalidator<**: `invalidate_scope` (`priv`, L71)

#### `crates/contextra-kvcache/src/segment.rs` (25)

- **ShreddableSegmentKey**: `try_new` (`pub`, L45), `try_new_random` (`pub`, L54), `encrypt` (`pub`, L63), `decrypt` (`pub`, L75), `shred` (`pub`, L88), `is_shredded` (`pub`, L95)
- **std::fmt::Debug for ShreddableSegmentKey**: `fmt` (`priv`, L101)
- **Tier2EncryptedSegment**: `new` (`pub`, L119), `new_with_random_key` (`pub`, L136), `read_and_decrypt` (`pub`, L147), `to_kv_segment` (`pub`, L152)
- **KvSegment**: `new` (`pub`, L198), `new_with_metadata` (`pub`, L217), `new_quantized` (`pub`, L243), `write_quantized` (`pub`, L258), `read_dequantized` (`pub`, L284), `new_encrypted` (`pub`, L303), `active_refs` (`pub`, L339), `acquire_guard` (`pub`, L344), `decrypt_data` (`pub`, L355), `to_spill_bytes` (`pub`, L391), `as_bytes` (`pub`, L402), `len` (`pub`, L407), `is_empty` (`pub`, L412)
- **std::fmt::Debug for KvSegment**: `fmt` (`priv`, L418)

#### `crates/contextra-kvcache/src/store.rs` (53)

- **TenantState**: `new` (`priv`, L45), `insert_returning_evicted_with_directive` (`priv`, L57), `get_bytes` (`priv`, L117), `get_segment_ref_mut` (`priv`, L127), `acquire_guard` (`priv`, L132), `remove` (`priv`, L142), `release_step` (`priv`, L154), `pop_eviction_candidate` (`priv`, L172), `pop_lru` (`priv`, L242), `insert_token_sequence` (`priv`, L247), `find_prefix_match` (`priv`, L256), `segment_ids` (`priv`, L268), `len` (`priv`, L272), `is_empty` (`priv`, L276)
- **Shard**: `new` (`priv`, L288)
- **TenantIsolatedKvStore**: `new` (`pub`, L311), `with_shard_count` (`pub`, L316), `set_attention_source` (`pub`, L341), `attention_source` (`pub`, L346), `register_segment_request` (`pub`, L351), `unregister_segment_request` (`pub`, L362), `insert_segment_for_request` (`pub`, L368), `with_capacity` (`pub`, L379), `set_spill_handler` (`pub`, L387), `shard_idx` (`priv`, L393), `insert_segment_with_directive` (`pub`, L398), `insert_segment` (`pub`, L430), `release_step` (`pub`, L435), `insert_token_sequence` (`pub`, L447), `find_prefix_match` (`pub`, L463), `acquire_block_guard` (`pub`, L477), `pin_block` (`pub`, L490), `on_rollback` (`pub`, L495), `remove_segments_for_rollback` (`pub`, L500), `remove_segment` (`pub`, L535), `remove_doc_segments` (`pub`, L540), `purge_tenant` (`pub`, L546), `get_segment_bytes` (`pub`, L566), `insert_encrypted_segment` (`pub`, L577), `get_segments` (`pub`, L599), `get_decrypted_segment` (`pub`, L611), `insert_quantized_segment` (`pub`, L630), `get_dequantized_segment` (`pub`, L645), `get_tenant_segment_len` (`pub`, L663), `evict_fair` (`pub`, L676), `evict_weighted_fair` (`pub`, L681), `evict_lru_fair` (`pub`, L698), `evict_fair_internal` (`priv`, L716), `clear_all` (`pub`, L812)
- **contextra_ports::KvLifecycleHooks for TenantIsolatedKvStore**: `on_rollback` (`priv`, L830), `remove_doc_segments` (`priv`, L834), `purge_tenant` (`priv`, L838)
- **Default for TenantIsolatedKvStore**: `default` (`priv`, L844)

### contextra-license

> License and activation enforcement gate layer for Contextra feature rings

- **Pfad:** `crates/contextra-license` | **Dateien:** 2 | **Produktiv-Funktionen:** 25 (`20` `pub`, `5` intern) | **Tests:** 0
- **Öffentliche Typen (3)**: `SignedActivation`, `LicensePayload`, `SignedLicenseGate`

#### `crates/contextra-license/src/signed_gate.rs` (25)

- ***(Modulfunktionen)***: `constant_time_eq_32` (`priv`, L15), `serialize` (`pub`, L26), `deserialize` (`pub`, L33), `derive_local_installation_id_hash` (`pub`, L104)
- **SignedActivation**: `payload_bytes` (`pub`, L61), `create_signed` (`pub`, L70), `verify_signature` (`pub`, L87)
- **From<LegacyLicensePayload> for LicensePayload**: `from` (`priv`, L153)
- **std::fmt::Debug for SignedLicenseGate**: `fmt` (`priv`, L176)
- **Default for SignedLicenseGate**: `default` (`priv`, L187)
- **SignedLicenseGate**: `no_activation` (`pub`, L194), `unactivated_local` (`pub`, L205), `from_activation` (`pub`, L210), `from_activation_params` (`pub`, L215), `from_activation_with_clock` (`pub`, L232), `from_signed_payload` (`pub`, L251), `from_signed_payload_with_clock` (`pub`, L268), `with_local_installation_id` (`pub`, L307), `with_auto_installation_id` (`pub`, L313), `license_payload` (`pub`, L319), `activation` (`pub`, L324), `verifying_key` (`pub`, L329), `create_test_signed_payload` (`pub`, L334), `for_testing` (`pub`, L357)
- **LicenseGate for SignedLicenseGate**: `check_ring` (`priv`, L370)

### contextra-mcp

- **Pfad:** `crates/contextra-mcp` | **Dateien:** 25 | **Produktiv-Funktionen:** 115 (`76` `pub`, `39` intern) | **Tests:** 117
- **Öffentliche Typen (27)**: `ResolvedProviderSetting`, `EmbeddingConfig`, `LlmConfig`, `RouterConfig`, `MockLlmGenerator`, `ExplainRequest`, `ExplainResponse`, `PluginStatusResponse`, `PluginStatusEntry`, `SecurityAuditRecord`, `SecurityAuditLogger`, `NormalizedPattern`, `PromptInjectionGuard`, `QuarantinePolicy`, `PromptInjectionConfig`, `ResolvedProofKey`, `ProofKeyError`, `ProofKeySource`, `ProofKeyResolution`, `McpError`, `RoutingHandle`, `ToolDefinition`, `ToolCategory`, `SandboxPolicy`, `VolatileToolResult`, `McpSandbox`, `McpServer`

#### `crates/contextra-mcp/src/bin/contextra-mcp-server.rs` (1)

- ***(Modulfunktionen)***: `main` (`priv`, L13)

#### `crates/contextra-mcp/src/config.rs` (11)

- ***(Modulfunktionen)***: `resolve_provider_setting` (`pub(crate)`, L24), `create_embedding_provider` (`pub`, L128), `create_llm_text_generator` (`pub`, L275)
- **Default for EmbeddingConfig**: `default` (`priv`, L64)
- **EmbeddingConfig**: `from_env` (`pub`, L77), `build_provider` (`pub`, L116)
- **Default for LlmConfig**: `default` (`priv`, L217)
- **LlmConfig**: `from_env` (`pub`, L229), `build_generator` (`pub`, L264)
- **RouterConfig**: `from_env` (`pub`, L347)
- **LlmTextGenerator for MockLlmGenerator**: `generate` (`priv`, L386)

#### `crates/contextra-mcp/src/egress_gateway.rs` (2)

- **From<EgressError> for McpError**: `from` (`priv`, L12)
- **contextra_privacy::InjectionDetector for PromptInjectionGuard**: `detect` (`priv`, L26)

#### `crates/contextra-mcp/src/explain.rs` (2)

- ***(Modulfunktionen)***: `default_collection` (`priv`, L18)
- **McpServer**: `handle_explain` (`pub(crate)`, L40)

#### `crates/contextra-mcp/src/io.rs` (1)

- ***(Modulfunktionen)***: `read_line_bounded` (`pub`, L15)

#### `crates/contextra-mcp/src/plugin_status.rs` (2)

- **From<contextra_ports::plugin::PluginCapability> for PluginStatusEntry**: `from` (`priv`, L25)
- ***(Modulfunktionen)***: `handle_plugin_status` (`pub`, L36)

#### `crates/contextra-mcp/src/prompt_injection/audit.rs` (3)

- **SecurityAuditLogger**: `new` (`pub`, L26), `log_event` (`pub`, L33), `get_recorded_events` (`pub`, L61)

#### `crates/contextra-mcp/src/prompt_injection/guard.rs` (26)

- **std::fmt::Debug for PromptInjectionGuard**: `fmt` (`priv`, L43)
- **Default for PromptInjectionGuard**: `default` (`priv`, L55)
- **PromptInjectionGuard**: `new` (`pub`, L66), `with_clock` (`pub`, L96), `clock` (`pub`, L101), `get_timestamp` (`priv`, L105), `default_patterns` (`pub`, L121), `policy` (`pub`, L174), `patterns` (`pub`, L178), `audit_logger` (`pub`, L182), `load_from_file` (`pub`, L187), `from_env` (`pub`, L218), `is_zero_width` (`pub`, L253), `skeletonize_char` (`pub`, L269), `normalize_text` (`pub`, L408), `decode_leetspeak_view` (`pub`, L416), `decode_html_entities_view` (`pub`, L432), `is_markdown_formatting_char` (`pub`, L481), `strip_markdown_inline_formatting_view` (`pub`, L486), `collapse_whitespace` (`pub`, L510), `strip_whitespace` (`pub`, L528), `decode_base64` (`pub`, L533), `extract_base64_candidates` (`pub`, L571), `detect_recursive` (`pub`, L602), `detect` (`pub`, L641), `process_result` (`pub`, L688)

#### `crates/contextra-mcp/src/prompt_injection/policy.rs` (2)

- **QuarantinePolicy**: `from_env` (`pub`, L33)
- ***(Modulfunktionen)***: `default_redaction_placeholder` (`pub`, L59)

#### `crates/contextra-mcp/src/prompt_injection/tests.rs` (2)

- **contextra_ports::Clock for FixedTestClock**: `now_unix_nanos` (`priv`, L358), `monotonic_nanos` (`priv`, L362)

#### `crates/contextra-mcp/src/proof_key.rs` (2)

- ***(Modulfunktionen)***: `resolve_deletion_proof_key` (`pub(crate)`, L46), `deletion_proof_key_from_env` (`pub`, L87)

#### `crates/contextra-mcp/src/proof_key_env.rs` (3)

- **fmt::Debug for ProofKeyResolution**: `fmt` (`priv`, L29)
- ***(Modulfunktionen)***: `resolve_proof_key` (`pub`, L44), `resolve_proof_key_from_env` (`pub`, L93)

#### `crates/contextra-mcp/src/protocol.rs` (10)

- **McpError**: `parse_error` (`pub`, L29), `invalid_request` (`pub`, L33), `method_not_found` (`pub`, L37), `invalid_params` (`pub`, L41), `internal_error` (`pub`, L45), `code` (`pub`, L49)
- **From<contextra_types::ContextraError> for McpError**: `from` (`priv`, L61)
- **From<String> for McpError**: `from` (`priv`, L71)
- **From<**: `from` (`priv`, L77)
- ***(Modulfunktionen)***: `response_from_error` (`pub`, L85)

#### `crates/contextra-mcp/src/routing.rs` (6)

- **contextra::router::ports_local::HybridSearchProvider for CollectionAdapter**: `search_hybrid` (`priv`, L9)
- **contextra::router::ports_local::CommunityResolver for CommunityAdapter**: `get_community` (`priv`, L43)
- **contextra::router::ports_local::ContextPreparer for ContextPreparerAdapter**: `prepare_context` (`priv`, L54)
- **contextra_ports::DriftStatusProvider for RouterDriftAdapter**: `overall_drift_status` (`priv`, L100)
- ***(Modulfunktionen)***: `setup_routing` (`pub`, L113), `setup_kv_bridge` (`pub`, L160)

#### `crates/contextra-mcp/src/sandbox.rs` (14)

- **Default for SandboxPolicy**: `default` (`priv`, L295)
- **VolatileToolResult**: `encrypt` (`pub`, L319), `decrypt` (`pub`, L330)
- **McpSandbox**: `new` (`pub`, L359), `policy` (`pub`, L376), `validate_tool_call` (`pub`, L390), `try_classify_method` (`pub`, L434), `classify_method` (`pub`, L452), `execute_with_timeout` (`pub`, L457), `store_volatile` (`pub`, L476), `get_volatile` (`pub`, L503)
- **Drop for McpSandbox**: `drop` (`priv`, L515)
- ***(Modulfunktionen)***: `generate_sandbox_crypto_material` (`priv`, L524), `encode` (`pub`, L537)

#### `crates/contextra-mcp/src/server.rs` (12)

- **McpServer**: `new` (`pub`, L33), `from_env` (`pub`, L41), `with_write_permission` (`pub`, L65), `with_sandbox` (`pub`, L82), `with_kv_bridge` (`pub`, L105), `with_injection_guard` (`pub`, L113), `with_egress_classifier` (`pub`, L118), `with_egress_guard` (`pub`, L123), `with_routing` (`pub`, L128), `with_plugin_registry` (`pub`, L133), `get_volatile_output` (`pub`, L142), `run_stdio` (`pub`, L154)

#### `crates/contextra-mcp/src/server_dispatch.rs` (2)

- **McpServer**: `handle_value` (`pub`, L9), `handle` (`pub`, L102)

#### `crates/contextra-mcp/src/server_tools.rs` (1)

- **McpServer**: `call_tool` (`pub(crate)`, L15)

#### `crates/contextra-mcp/src/tests.rs` (8)

- **contextra_ports::EmbeddingProvider for MockEmbedder**: `provider_name` (`priv`, L18), `embed` (`priv`, L22), `embedding_dim` (`priv`, L29), `embed_batch` (`priv`, L33)
- ***(Modulfunktionen)***: `create_mock_server` (`priv`, L41), `create_mock_server_with_write` (`priv`, L45), `create_mock_server_with_write_and_egress` (`priv`, L49), `make_request` (`priv`, L71)

#### `crates/contextra-mcp/src/tools_crud.rs` (3)

- **McpServer**: `handle_create_collection` (`pub(crate)`, L10), `handle_drop_collection` (`pub(crate)`, L91), `handle_delete` (`pub(crate)`, L147)

#### `crates/contextra-mcp/src/validation.rs` (2)

- ***(Modulfunktionen)***: `is_write_allowed_by_env` (`pub`, L4), `validate_collection_name` (`pub`, L16)

### contextra-mvcc

> Multi-Version Concurrency Control (MVCC), sequence log, and transaction buffer for Contextra

- **Pfad:** `crates/contextra-mvcc` | **Dateien:** 5 | **Produktiv-Funktionen:** 149 (`109` `pub`, `40` intern) | **Tests:** 46
- **Öffentliche Typen (14)**: `SeqLogEntry`, `SeqLogChange`, `SequenceLog`, `SnapshotRegistry`, `SnapshotGuard`, `ReadSet`, `SsiValidator`, `PruningBlockerInfo`, `SequenceLogSsiValidator`, `SequenceLogSsiValidatorBuilder`, `TxBufferConfig`, `IndexOp`, `StagedOpSize`, `TxBuffer`

#### `crates/contextra-mvcc/src/seq_log.rs` (23)

- **SeqLogEntry**: `is_visible` (`pub`, L41)
- **SeqLogChange**: `seq` (`pub`, L70), `doc_id` (`pub`, L77)
- **Default for SequenceLog**: `default` (`priv`, L95)
- **SequenceLog**: `new` (`pub`, L102), `with_max_pin_duration` (`pub`, L113), `set_max_pin_duration` (`pub`, L119), `max_pin_duration` (`pub`, L124), `pin_snapshot` (`pub`, L129), `pin_snapshot_at` (`pub`, L134), `unpin_snapshot` (`pub`, L140), `expired_pins_at` (`pub`, L155), `expired_pins` (`pub`, L170), `min_retention_seq_at` (`pub`, L178), `min_retention_seq` (`pub`, L203), `deletion_seq` (`pub`, L216), `record_insert` (`pub`, L221), `record_delete` (`pub`, L239), `is_visible` (`pub`, L252), `compact` (`pub`, L272), `len` (`pub`, L291), `is_empty` (`pub`, L296), `changes_since` (`pub`, L302)

#### `crates/contextra-mvcc/src/snapshot.rs` (15)

- **Default for SnapshotRegistry**: `default` (`priv`, L46)
- **SnapshotRegistry**: `new` (`pub`, L53), `register` (`pub`, L62), `register_at` (`pub`, L67), `min_active_seqno` (`pub`, L91), `pin` (`pub`, L96), `pin_at` (`pub`, L101), `unpin` (`pub`, L109), `release` (`pub(crate)`, L113), `release_at` (`pub(crate)`, L117), `longest_active_pin_at` (`pub`, L141), `longest_active_pin` (`pub`, L163), `update_min` (`priv`, L167)
- **SnapshotGuard**: `seq_no` (`pub`, L189)
- **Drop for SnapshotGuard**: `drop` (`priv`, L195)

#### `crates/contextra-mvcc/src/ssi.rs` (58)

- **ReadSet**: `new` (`pub`, L53), `record_read` (`pub`, L65), `record_prefix` (`pub`, L87), `register_read` (`pub`, L106), `get` (`pub`, L111), `is_empty` (`pub`, L116), `len` (`pub`, L121), `clear` (`pub`, L126), `min_snapshot_seq` (`pub`, L134), `iter` (`pub`, L139), `prefixes_iter` (`pub`, L144), `keys_map` (`pub`, L149), `prefixes_map` (`pub`, L154)
- **trait SsiValidator**: `validate` (`priv`, L174)
- ***(Modulfunktionen)***: `coarsen_keys_to_prefixes` (`priv`, L224), `derive_doc_id_for_key` (`priv`, L1040)
- **CommittedWrites**: `tracked_count` (`priv`, L265), `coarsened_prefix_count` (`priv`, L269), `insert` (`priv`, L279), `coarsen_oldest_buckets` (`priv`, L302), `merge_adjacent_buckets` (`priv`, L331), `remove_from_seq` (`priv`, L387), `prune_through` (`priv`, L431), `len` (`priv`, L461), `get` (`priv`, L465)
- **std::fmt::Debug for SequenceLogSsiValidator**: `fmt` (`priv`, L496)
- **Default for SequenceLogSsiValidator**: `default` (`priv`, L510)
- **std::fmt::Debug for SequenceLogSsiValidatorBuilder**: `fmt` (`priv`, L528)
- **SequenceLogSsiValidatorBuilder**: `new` (`pub`, L541), `with_max_tracked_keys` (`pub`, L546), `with_sequence_log` (`pub`, L552), `with_snapshot_registry` (`pub`, L558), `with_max_pin_duration` (`pub`, L564), `with_metrics_sink` (`pub`, L570), `build` (`pub`, L579)
- **SequenceLogSsiValidator**: `builder` (`pub`, L604), `new` (`pub`, L609), `new_with_bounds` (`pub`, L614), `with_sequence_log` (`pub`, L627), `with_snapshot_registry` (`pub`, L640), `set_snapshot_registry` (`pub`, L646), `with_metrics_sink` (`pub`, L657), `set_metrics_sink` (`pub`, L666), `with_max_pin_duration` (`pub`, L674), `prune_through` (`pub`, L690), `forget_from` (`pub`, L706), `pruned_through` (`pub`, L713), `tracked_commit_keys` (`pub`, L718), `max_tracked_keys` (`pub`, L723), `check_and_coarsen_if_needed` (`priv`, L728), `diagnose_pruning_blocker` (`pub`, L787), `diagnose_pruning_blocker_at` (`pub`, L792), `diagnose_pruning_blocker_internal` (`priv`, L797), `record_commit_key` (`pub`, L840), `record_commit_keys` (`pub`, L845), `validate_internal` (`priv`, L859), `validate_and_record` (`pub`, L1015)
- **SsiValidator for SequenceLogSsiValidator**: `validate` (`priv`, L1035)

#### `crates/contextra-mvcc/src/tx_buffer.rs` (53)

- **Default for TxBufferConfig**: `default` (`priv`, L76)
- **IndexOp<T>**: `doc_id` (`pub`, L109), `estimated_bytes` (`pub`, L161)
- **trait StagedOpSize**: `staged_bytes` (`priv`, L120)
- **StagedOpSize for Vec<T>**: `staged_bytes` (`priv`, L124)
- **StagedOpSize**: `staged_bytes` (`priv`, L130)
- **StagedOpSize for String**: `staged_bytes` (`priv`, L136)
- **StagedOpSize for usize**: `staged_bytes` (`priv`, L142)
- **StagedOpSize for u64**: `staged_bytes` (`priv`, L148)
- **StagedOpSize for u8**: `staged_bytes` (`priv`, L154)
- **TxShard<T>**: `new` (`priv`, L179)
- **KeyShard**: `new` (`priv`, L195)
- **TxBuffer<T>**: `new` (`pub`, L224), `new_with_config` (`pub`, L232), `new_with_config_ext` (`pub`, L241), `staged_bytes` (`pub`, L263), `shard_idx` (`priv`, L268), `key_shard_idx` (`priv`, L274), `has_tx` (`pub`, L282), `begin_at` (`pub`, L289), `begin` (`pub`, L301), `register_read` (`pub`, L306), `register_prefix_read` (`pub`, L317), `try_register_read` (`pub`, L334), `record_read` (`pub`, L358), `min_read_snapshot` (`pub`, L375), `get_read_set` (`pub`, L389), `read_set` (`pub`, L396), `stage` (`pub`, L401), `stage_bounded` (`pub`, L409), `stage_many` (`pub`, L454), `validate_pending_ops` (`pub`, L504), `try_drain` (`pub`, L523), `drain` (`pub`, L532), `discard` (`pub`, L550), `len` (`pub`, L567), `is_empty` (`pub`, L574), `reap_orphans_at` (`pub`, L584), `reap_orphans` (`pub`, L594), `reap_orphans_bounded_at` (`pub`, L603), `reap_orphans_bounded` (`pub`, L642), `get_ops` (`pub`, L647), `tx_timeout` (`pub`, L655)
- **TxBuffer<**: `track_key_stage` (`priv`, L661), `untrack_key_ops_kv` (`priv`, L671), `stage_kv_if_absent` (`pub`, L696), `stage_kv` (`pub`, L720), `try_drain_kv` (`pub`, L750), `drain_kv` (`pub`, L759), `discard_kv` (`pub`, L768), `is_key_staged_for_tx` (`pub`, L780), `staged_status` (`pub`, L798)
- **Default for TxBuffer<T>**: `default` (`priv`, L827)

### contextra-ports

> Canonical dyn-compatible port traits for Contextra subsystems

- **Pfad:** `crates/contextra-ports` | **Dateien:** 21 | **Produktiv-Funktionen:** 199 (`27` `pub`, `172` intern) | **Tests:** 134
- **Öffentliche Typen (64)**: `RequestId`, `AttentionExporter`, `Checkpoint`, `CheckpointCoordinator`, `Clock`, `SystemClock`, `ContextSegment`, `TextEmbeddingEngine`, `EmbeddingError`, `EmbeddingProvider`, `LlmTextGenerator`, `LlmTextGeneratorStreaming`, `MockEmbedder`, `RoleId`, `RoleBinding`, `HyperEdgeId`, `GraphMutationError`, `GraphCollectionMutation`, `GraphIndex`, `GraphIndexStats`, `CommunityResolver`, `IdGen`, `SequentialIdGen`, `KvLayout`, `RopeConfig`, `PrefixKey`, `KvBlock`, `KvPrefixHit`, `KvPrefixStore`, `KvLifecycleHooks`, `KvBridgeStorage`, `BoxFuture`, `BoxStream`, `VectorDeleteMode`, `FeatureRing`, `AuthorizedRing`, `LicenseError`, `LicenseGate`, `OpenFastGate`, `GroundingAssessment` *(+24 weitere)*

#### `crates/contextra-ports/src/attention.rs` (1)

- **trait AttentionExporter**: `export_attention_weights` (`priv`, L15)

#### `crates/contextra-ports/src/checkpoint.rs` (6)

- **trait Checkpoint**: `take_snapshot` (`priv`, L11), `restore` (`priv`, L14)
- **trait CheckpointCoordinator**: `create_named_checkpoint` (`priv`, L31), `restore_named_checkpoint` (`priv`, L41), `drop_named_checkpoint` (`priv`, L47), `list_named_checkpoints` (`priv`, L50)

#### `crates/contextra-ports/src/clock.rs` (8)

- **trait Clock**: `now_unix_nanos` (`priv`, L12), `monotonic_nanos` (`priv`, L15)
- **SystemClock**: `new` (`pub`, L26)
- **Default for SystemClock**: `default` (`priv`, L34)
- **Clock for SystemClock**: `now_unix_nanos` (`priv`, L40), `monotonic_nanos` (`priv`, L47)
- **Clock for Arc<T>**: `now_unix_nanos` (`priv`, L55), `monotonic_nanos` (`priv`, L59)

#### `crates/contextra-ports/src/embedding.rs` (21)

- **ContextSegment<**: `new` (`pub`, L20), `with_offset` (`pub`, L30), `with_fingerprint` (`pub`, L35), `with_rope_offset` (`pub`, L41)
- **trait TextEmbeddingEngine**: `embed` (`priv`, L50), `embed_batch` (`priv`, L54)
- **trait EmbeddingProvider**: `provider_name` (`priv`, L90), `embed` (`priv`, L93), `embedding_dim` (`priv`, L96), `embed_batch` (`priv`, L99)
- **trait LlmTextGenerator**: `generate` (`priv`, L116), `generate_with_context` (`priv`, L121)
- **trait LlmTextGeneratorStreaming**: `generate_stream` (`priv`, L141)
- **TextEmbeddingEngine for T**: `embed` (`priv`, L150), `embed_batch` (`priv`, L158)
- **MockEmbedder**: `new` (`pub`, L180), `with_fixed_output` (`pub`, L188), `fixed_unit` (`pub`, L197)
- **EmbeddingProvider for MockEmbedder**: `provider_name` (`priv`, L203), `embed` (`priv`, L207), `embedding_dim` (`priv`, L216)

#### `crates/contextra-ports/src/graph.rs` (11)

- **RoleId**: `new` (`pub`, L19), `inner` (`pub`, L25)
- **From<u32> for RoleId**: `from` (`priv`, L31)
- **std::fmt::Display for RoleId**: `fmt` (`priv`, L37)
- **RoleBinding**: `new` (`pub`, L53)
- **HyperEdgeId**: `new` (`pub`, L68), `inner` (`pub`, L74)
- **From<u64> for HyperEdgeId**: `from` (`priv`, L80)
- **std::fmt::Display for HyperEdgeId**: `fmt` (`priv`, L86)
- **From<GraphMutationError> for ContextraError**: `from` (`priv`, L136)
- **trait GraphCollectionMutation**: `relate_n_ary` (`priv`, L147)

#### `crates/contextra-ports/src/graph_index.rs` (22)

- **trait GraphIndex**: `traverse` (`priv`, L53), `neighbors` (`priv`, L60), `remove_edge` (`priv`, L68), `add_bidirectional` (`priv`, L81), `multi_traverse` (`priv`, L97), `multi_traverse_at` (`priv`, L121), `path_rag_at` (`priv`, L154), `traverse_at` (`priv`, L175), `personalized_page_rank_at` (`priv`, L195), `traverse_at_time` (`priv`, L215), `traverse_at_bitemporal` (`priv`, L234), `personalized_page_rank` (`priv`, L260), `add_entity` (`priv`, L274), `add_edge` (`priv`, L277), `commit` (`priv`, L280), `rollback` (`priv`, L283), `rollback_to_tx` (`priv`, L286), `last_tx_id` (`priv`, L289), `len` (`priv`, L292), `is_empty` (`priv`, L295), `stats` (`priv`, L300)
- **trait CommunityResolver**: `get_community` (`priv`, L318)

#### `crates/contextra-ports/src/id_gen.rs` (5)

- **trait IdGen**: `next_id` (`priv`, L9)
- **SequentialIdGen**: `new` (`pub`, L22)
- **Default for SequentialIdGen**: `default` (`priv`, L30)
- **IdGen for SequentialIdGen**: `next_id` (`priv`, L36)
- **IdGen for Arc<T>**: `next_id` (`priv`, L42)

#### `crates/contextra-ports/src/kv.rs` (14)

- **std::hash::Hash for RopeConfig**: `hash` (`priv`, L34)
- **std::hash::Hash for PrefixKey**: `hash` (`priv`, L56)
- ***(Modulfunktionen)***: `serialize_bytes` (`priv`, L66), `deserialize_bytes` (`priv`, L73)
- **serde::de::Visitor<**: `expecting` (`priv`, L81), `visit_bytes` (`priv`, L85), `visit_byte_buf` (`priv`, L92), `visit_seq` (`priv`, L99)
- **trait KvPrefixStore**: `lookup` (`priv`, L141), `insert` (`priv`, L144), `evict` (`priv`, L154)
- **trait KvLifecycleHooks**: `on_rollback` (`priv`, L163), `remove_doc_segments` (`priv`, L166), `purge_tenant` (`priv`, L169)

#### `crates/contextra-ports/src/kv_bridge_port.rs` (3)

- **trait KvBridgeStorage**: `get` (`priv`, L17), `put` (`priv`, L20), `commit` (`priv`, L23)

#### `crates/contextra-ports/src/license.rs` (6)

- **Default for VectorDeleteMode**: `default` (`priv`, L14)
- **AuthorizedRing**: `new` (`pub(crate)`, L42), `ring` (`pub`, L47)
- **trait LicenseGate**: `check_ring` (`priv`, L72), `authorize` (`priv`, L75)
- **LicenseGate for OpenFastGate**: `check_ring` (`priv`, L86)

#### `crates/contextra-ports/src/lifecycle.rs` (3)

- **trait GroundingValidator**: `validate_grounding` (`priv`, L31)
- **trait ResponseGroundingValidator**: `score_grounding` (`priv`, L42)
- **trait ContextPreparer**: `prepare_context` (`priv`, L49)

#### `crates/contextra-ports/src/metrics.rs` (12)

- **trait MetricsSink**: `record_counter` (`priv`, L17), `record_gauge` (`priv`, L20), `record_histogram` (`priv`, L23)
- **MetricsSink for NoopMetricsSink**: `record_counter` (`priv`, L31), `record_gauge` (`priv`, L33), `record_histogram` (`priv`, L35)
- **TestMetricsSink**: `new` (`pub`, L78), `events` (`pub`, L83), `clear` (`pub`, L91)
- **MetricsSink for TestMetricsSink**: `record_counter` (`priv`, L100), `record_gauge` (`priv`, L115), `record_histogram` (`priv`, L130)

#### `crates/contextra-ports/src/observability.rs` (1)

- **trait DriftStatusProvider**: `overall_drift_status` (`priv`, L13)

#### `crates/contextra-ports/src/plugin.rs` (14)

- **PluginCapability**: `new` (`pub`, L24)
- **trait PluginManifest**: `name` (`priv`, L42), `requires` (`priv`, L45), `conflicts` (`priv`, L50), `feature_ring_required` (`priv`, L55), `capability` (`priv`, L60), `activate` (`priv`, L63)
- **From<PluginError> for contextra_types::ContextraError**: `from` (`priv`, L83)
- **PluginRegistry**: `new` (`pub`, L97), `is_active` (`pub`, L106), `snapshot` (`pub`, L111), `current_feature_ring` (`pub`, L116), `activate` (`pub`, L127), `activate_all` (`pub`, L138)

#### `crates/contextra-ports/src/query_rewriter.rs` (2)

- **trait QueryRewriter**: `rewrite` (`priv`, L11), `rewrite_structured` (`priv`, L32)

#### `crates/contextra-ports/src/reranker.rs` (3)

- **trait Reranker**: `rerank` (`priv`, L9), `record_implicit_feedback` (`priv`, L16), `rerank_deadline_ms` (`priv`, L19)

#### `crates/contextra-ports/src/rng.rs` (11)

- **trait Rng**: `next_u64` (`priv`, L14), `fill_bytes` (`priv`, L17), `next_unit_f64` (`priv`, L21)
- **SeededRng**: `new` (`pub`, L42), `from_entropy` (`pub`, L59)
- **Rng for SeededRng**: `next_u64` (`priv`, L76), `fill_bytes` (`priv`, L87), `next_unit_f64` (`priv`, L97)
- **Rng for Arc<T>**: `next_u64` (`priv`, L105), `fill_bytes` (`priv`, L109), `next_unit_f64` (`priv`, L113)

#### `crates/contextra-ports/src/storage.rs` (26)

- **trait StorageEngine**: `get` (`priv`, L38), `get_at_seq` (`priv`, L41), `get_tracked` (`priv`, L48), `get_at_seq_tracked` (`priv`, L61), `scan_prefix_tracked` (`priv`, L76), `supports_ssi_tracking` (`priv`, L87), `put` (`priv`, L92), `put_if_absent` (`priv`, L95), `put_batch` (`priv`, L110), `delete` (`priv`, L124), `delete_many` (`priv`, L127), `delete_prefix` (`priv`, L140), `commit` (`priv`, L153), `rollback` (`priv`, L156), `rollback_to_tx` (`priv`, L159), `flush` (`priv`, L162), `stats` (`priv`, L165), `last_seq_no` (`priv`, L168), `last_tx_id` (`priv`, L171), `pin_checkpoint` (`priv`, L174), `unpin_checkpoint` (`priv`, L177), `scan_prefix` (`priv`, L181), `scan_prefix_bounded` (`priv`, L188), `scan_prefix_at` (`priv`, L222), `scan` (`priv`, L237), `scan_bounded` (`priv`, L246)

#### `crates/contextra-ports/src/text_index.rs` (13)

- **trait SegmentSynthesizer**: `synthesize_segment` (`priv`, L19), `model_id` (`priv`, L24)
- **trait TextIndex**: `search` (`priv`, L44), `search_at` (`priv`, L56), `insert` (`priv`, L72), `delete` (`priv`, L75), `commit` (`priv`, L78), `rollback` (`priv`, L81), `rollback_to_tx` (`priv`, L84), `last_tx_id` (`priv`, L87), `len` (`priv`, L90), `is_empty` (`priv`, L93), `stats` (`priv`, L98)

#### `crates/contextra-ports/src/vector_index.rs` (17)

- **trait VectorIndex**: `insert` (`priv`, L40), `all_doc_ids` (`priv`, L48), `insert_batch` (`priv`, L53), `search` (`priv`, L67), `search_at` (`priv`, L79), `search_filtered` (`priv`, L107), `delete` (`priv`, L125), `commit` (`priv`, L128), `rollback` (`priv`, L131), `rollback_to_tx` (`priv`, L134), `last_tx_id` (`priv`, L137), `len` (`priv`, L140), `is_empty` (`priv`, L143), `stats` (`priv`, L148), `is_rebuild_required` (`priv`, L151), `trigger_rebuild_async` (`priv`, L156)
- **trait HybridSearchProvider**: `search_hybrid` (`priv`, L163)

### contextra-privacy

> Cloud Egress Security, DLP & Exfiltration Protection for Contextra (Ring 3)

- **Pfad:** `crates/contextra-privacy` | **Dateien:** 12 | **Produktiv-Funktionen:** 149 (`88` `pub`, `61` intern) | **Tests:** 68
- **Öffentliche Typen (42)**: `EgressClassifierTrace`, `AvvContext`, `SessionId`, `BulkExfiltrationOutcome`, `BulkExfiltrationDetector`, `ContextEditKind`, `ContextEditAuditError`, `ContextEditInput`, `ContextEditAuditRecord`, `DefaultEgressClassifier`, `EgressGuardCheck`, `InjectionDetector`, `CloudResponseRehydrator`, `CloudQueryRequest`, `CloudQueryResponse`, `TextSearchResult`, `TextSearchEngine`, `EgressGuard`, `EntityRecognizer`, `NoOpRecognizer`, `SurrogateVault`, `BoxFuture`, `PolicyCategory`, `BlockReason`, `EgressClassification`, `PatternValidator`, `CompiledPattern`, `EgressVaultError`, `EgressVault`, `EgressClassifier`, `PatternRecognizer`, `EntityMatch`, `Gazetteer`, `RuleBasedEntityRecognizer`, `EgressError`, `Unsanitized`, `Sanitized`, `GuardedPayload`, `ProcessorRole`, `TenantId` *(+2 weitere)*

#### `crates/contextra-privacy/src/audit_trace.rs` (4)

- ***(Modulfunktionen)***: `extract_rule_id` (`pub`, L10), `compute_audit_trace` (`pub`, L29)
- **trait EgressClassifierTrace**: `classify_with_trace` (`priv`, L54)
- **EgressClassifierTrace for T**: `classify_with_trace` (`priv`, L62)

#### `crates/contextra-privacy/src/avv_generator.rs` (1)

- ***(Modulfunktionen)***: `generate_avv_draft` (`pub`, L30)

#### `crates/contextra-privacy/src/bulk_exfiltration_detector.rs` (9)

- **From<String> for SessionId**: `from` (`priv`, L19)
- **From<**: `from` (`priv`, L25)
- **SlidingWindowCounter**: `new` (`priv`, L51), `prune_and_sum` (`priv`, L62), `record_and_check` (`priv`, L83)
- **std::fmt::Debug for BulkExfiltrationDetector**: `fmt` (`priv`, L120)
- **BulkExfiltrationDetector**: `new` (`pub`, L130), `record_and_check` (`pub`, L142), `record_and_check_at` (`priv`, L151)

#### `crates/contextra-privacy/src/context_edit_audit.rs` (5)

- ***(Modulfunktionen)***: `compute_record_hash` (`pub`, L79), `build_context_edit_audit_record` (`pub`, L131), `verify_audit_chain` (`pub`, L156), `render_audit_line` (`pub`, L190), `hex_encode_32` (`priv`, L208)

#### `crates/contextra-privacy/src/egress_gateway.rs` (22)

- **trait EgressGuardCheck**: `check` (`priv`, L27), `check_scoped` (`priv`, L30)
- **trait InjectionDetector**: `detect` (`priv`, L46)
- ***(Modulfunktionen)***: `match_surrogate_token` (`priv`, L53), `extract_surrogate_tokens` (`pub`, L81), `pii_vault_forces_crypto_shred` (`pub`, L180), `resolve_effective_kv_delete_mode` (`pub`, L187), `handle_cloud_query` (`pub`, L221), `handle_cloud_query_scoped` (`pub`, L229), `handle_cloud_query_scoped_with_guard` (`pub`, L245), `handle_cloud_query_scoped_with_bulk_detector` (`pub`, L266), `handle_cloud_query_with_guard` (`pub`, L317), `check_bulk_exfiltration` (`pub`, L353), `handle_cloud_query_with_bulk_detector` (`pub`, L370), `process_cloud_response` (`pub`, L385), `guard_and_sanitize_payload` (`pub`, L405), `guard_and_sanitize_payload_scoped` (`pub`, L426), `evaluate_classification_result` (`priv`, L453)
- **CloudResponseRehydrator**: `new` (`pub`, L110), `from_vault` (`pub`, L118), `scoped_to_request` (`pub`, L130), `rehydrate` (`pub`, L141)

#### `crates/contextra-privacy/src/egress_guard.rs` (11)

- **trait TextSearchEngine**: `search_text` (`priv`, L22)
- **EgressGuard**: `new` (`pub`, L52), `with_timeout` (`pub`, L62), `threshold` (`pub`, L68), `min_bytes` (`pub`, L73), `timeout` (`pub`, L78), `check_scoped` (`pub`, L85), `check` (`pub`, L109)
- **crate::egress_gateway::EgressGuardCheck for EgressGuard**: `check` (`priv`, L161), `check_scoped` (`priv`, L165)
- **EgressClassifier for EgressGuard**: `classify` (`priv`, L175)

#### `crates/contextra-privacy/src/egress_vault.rs` (43)

- ***(Modulfunktionen)***: `is_luhn_valid` (`pub`, L19), `is_iban_valid` (`pub`, L45), `normalize_payload` (`pub`, L89), `is_credit_card_pattern` (`priv`, L121), `is_iban_pattern` (`priv`, L126), `validate_luhn` (`pub`, L301), `validate_iban_mod97` (`pub`, L323), `normalize_payload_for_dlp` (`pub`, L349), `eval_text` (`priv`, L379), `classify_layer1` (`pub`, L502), `classify_layer1_arc` (`pub`, L521)
- **trait EntityRecognizer**: `recognize` (`priv`, L137)
- **EntityRecognizer for NoOpRecognizer**: `recognize` (`priv`, L145)
- **std::fmt::Debug for SurrogateVault**: `fmt` (`priv`, L159)
- **SurrogateVault**: `new` (`pub`, L169), `random_salt` (`pub`, L177), `generate_surrogate` (`pub`, L196), `get_entity` (`pub`, L221), `len` (`pub`, L229), `is_empty` (`pub`, L234)
- **Drop for SurrogateVault**: `drop` (`priv`, L240)
- **BlockReason**: `category` (`pub`, L282)
- **CompiledPattern**: `new` (`pub`, L435), `with_validator` (`pub`, L463), `is_valid_match` (`pub`, L469)
- **EgressVault**: `default_patterns` (`pub`, L586), `try_default` (`pub`, L603), `new` (`pub`, L609), `with_timeout` (`pub`, L633), `with_surrogate_vault` (`pub`, L639), `patterns` (`pub`, L645), `policy_category` (`pub`, L650), `timeout` (`pub`, L655), `surrogate_vault` (`pub`, L660), `get_entity` (`pub`, L665), `classify_scoped` (`pub`, L671), `sanitize_and_vault_scoped` (`pub`, L684), `sanitize_and_vault` (`pub`, L702)
- **trait EgressClassifier**: `classify` (`priv`, L773), `classify_scoped` (`priv`, L776)
- **Default for EgressVault**: `default` (`priv`, L791)
- **EgressClassifier for EgressVault**: `classify` (`priv`, L809), `classify_scoped` (`priv`, L819)

#### `crates/contextra-privacy/src/entity_recognizer.rs` (41)

- **Gazetteer**: `new` (`pub`, L55), `add_term` (`pub`, L63), `is_empty` (`pub`, L72), `len` (`pub`, L77)
- **Default for RuleBasedEntityRecognizer**: `default` (`priv`, L93)
- **RuleBasedEntityRecognizer**: `new` (`pub`, L104), `with_gazetteer` (`pub`, L110), `with_max_input_length` (`pub`, L119), `max_input_length` (`pub`, L126), `add_gazetteer_term` (`pub`, L132), `recognize` (`pub`, L139), `recognize_detailed` (`pub`, L144), `recognize_bounded` (`pub`, L152), `recognize_detailed_bounded` (`pub`, L167), `scan_entities_internal` (`priv`, L181)
- **EntityRecognizer for RuleBasedEntityRecognizer**: `recognize` (`priv`, L232)
- **Normalizer**: `normalize` (`priv`, L251), `len` (`priv`, L284), `get_char` (`priv`, L288), `char_span_to_byte_range` (`priv`, L293), `as_string` (`priv`, L302)
- ***(Modulfunktionen)***: `is_zero_width_or_control` (`priv`, L309), `fold_homoglyph` (`priv`, L327), `detect_emails` (`priv`, L400), `is_email_local_char` (`priv`, L473), `is_email_domain_char` (`priv`, L477), `detect_ibans` (`priv`, L484), `get_iban_country_length` (`priv`, L549), `validate_iban_checksum` (`priv`, L571), `detect_credit_cards` (`priv`, L607), `validate_luhn_checksum` (`priv`, L659), `detect_steuer_ids` (`priv`, L681), `validate_german_steuer_id` (`priv`, L741), `detect_phone_numbers` (`priv`, L796), `detect_ip_addresses` (`priv`, L864), `detect_bics` (`priv`, L992), `is_german_city_start_char` (`priv`, L1068), `scan_city_name_after_plz` (`priv`, L1073), `detect_plz_and_license_plates` (`priv`, L1193), `detect_gazetteer` (`priv`, L1326), `deduplicate_matches` (`priv`, L1371)

#### `crates/contextra-privacy/src/error.rs` (5)

- **From<crate::egress_vault::EgressVaultError> for EgressError**: `from` (`priv`, L22)
- **EgressError**: `invalid_params` (`pub`, L47), `policy_violation` (`pub`, L51), `internal_error` (`pub`, L55), `capacity_exceeded` (`pub`, L62)

#### `crates/contextra-privacy/src/guarded_payload.rs` (4)

- **GuardedPayload<S>**: `session_id` (`pub`, L29)
- **GuardedPayload<Unsanitized>**: `new` (`pub`, L36)
- **GuardedPayload<Sanitized>**: `from_sanitized` (`pub`, L47), `into_inner` (`pub`, L56)

#### `crates/contextra-privacy/src/processing_registry.rs` (4)

- **ProcessorRole**: `as_str` (`pub`, L24)
- ***(Modulfunktionen)***: `generate_registry` (`pub`, L68), `sanitize_markdown_cell` (`priv`, L75), `render_markdown` (`pub`, L86)

### contextra-py

> Python bindings for Contextra using PyO3

- **Pfad:** `crates/contextra-py` | **Dateien:** 16 | **Produktiv-Funktionen:** 71 (`60` `pub`, `11` intern) | **Tests:** 33
- **Öffentliche Typen (8)**: `PyCollection`, `PyContextra`, `PyDbStats`, `PyDocument`, `PyRuntimeState`, `PySearchResult`, `PyStorageStats`, `PyVectorIndexStats`

#### `crates/contextra-py/src/bindings/collection.rs` (6)

- **PyCollection**: `new` (`pub`, L25), `is_poisoned` (`pub`, L42), `_trigger_panic_for_test` (`pub`, L52), `stats` (`pub`, L57), `len` (`pub`, L71), `is_empty` (`pub`, L77)

#### `crates/contextra-py/src/bindings/common.rs` (16)

- ***(Modulfunktionen)***: `dict_to_json` (`pub`, L39), `opt_dict_to_json` (`pub`, L44), `validate_label` (`pub`, L54), `validate_id` (`pub`, L76), `validate_collection_name` (`pub`, L98), `validate_db_path` (`pub`, L120), `validate_query_text` (`pub`, L135), `validate_batch_size` (`pub`, L157), `validate_vector` (`pub`, L171), `validate_id_obj` (`pub`, L184), `check_subinterpreter_guard` (`pub`, L216), `run_blocking_ffi` (`pub`, L246), `json_to_py` (`pub`, L282), `doc_to_py` (`pub`, L289), `results_to_py` (`pub`, L301), `contextra_err` (`pub`, L321)

#### `crates/contextra-py/src/bindings/crud_macros.rs` (14)

- ***(Modulfunktionen)***: `insert` (`pub`, L14), `get` (`pub`, L33), `update` (`pub`, L49), `upsert` (`pub`, L69), `delete` (`pub`, L88), `search` (`pub`, L100), `search_fb` (`pub`, L126), `hybrid_search` (`pub`, L184), `hybrid_search_fb` (`pub`, L230), `relate` (`pub`, L305), `scan_prefix` (`pub`, L325), `scan` (`pub`, L349), `insert_many` (`pub`, L393), `upsert_many` (`pub`, L424)

#### `crates/contextra-py/src/bindings/db.rs` (14)

- **PyContextra**: `new` (`pub`, L29), `worker_threads` (`pub`, L48), `is_poisoned` (`pub`, L54), `_trigger_panic_for_test` (`pub`, L64), `__enter__` (`priv`, L71), `__exit__` (`priv`, L77), `collection` (`pub`, L95), `list_collections` (`pub`, L111), `drop_collection` (`pub`, L129), `flush` (`pub`, L157), `stats` (`pub`, L165), `len` (`pub`, L191), `is_empty` (`pub`, L199)
- ***(Modulfunktionen)***: `resolve_proof_key` (`priv`, L212)

#### `crates/contextra-py/src/bindings/db_stats.rs` (1)

- **PyDbStats**: `__repr__` (`priv`, L28)

#### `crates/contextra-py/src/bindings/document.rs` (1)

- **PyDocument**: `__repr__` (`priv`, L14)

#### `crates/contextra-py/src/bindings/functions.rs` (3)

- ***(Modulfunktionen)***: `open` (`pub`, L22), `trigger_panic_helper` (`pub(crate)`, L89), `_trigger_panic_for_test` (`pub`, L114)

#### `crates/contextra-py/src/bindings/hyperedge.rs` (2)

- ***(Modulfunktionen)***: `validate_hyperedge_args` (`pub`, L7)
- **PyCollection**: `relate_n_ary` (`pub`, L43)

#### `crates/contextra-py/src/bindings/runtime_state.rs` (2)

- ***(Modulfunktionen)***: `parse_worker_threads_env` (`pub`, L11), `get_runtime` (`pub`, L38)

#### `crates/contextra-py/src/bindings/search_result.rs` (1)

- **PySearchResult**: `__repr__` (`priv`, L16)

#### `crates/contextra-py/src/bindings/storage_stats.rs` (1)

- **PyStorageStats**: `__repr__` (`priv`, L17)

#### `crates/contextra-py/src/bindings/vector_index_stats.rs` (1)

- **PyVectorIndexStats**: `__repr__` (`priv`, L17)

#### `crates/contextra-py/src/kv_links.rs` (8)

- ***(Modulfunktionen)***: `parse_link_relation` (`pub`, L10), `relation_as_str` (`priv`, L27)
- **PyCollection**: `put_kv` (`pub`, L39), `put_kv_if_absent` (`pub`, L55), `get_kv` (`pub`, L71), `relate_bidirectional` (`pub`, L89), `link_memories` (`pub`, L111), `get_links` (`pub`, L131)

#### `crates/contextra-py/src/lib.rs` (1)

- ***(Modulfunktionen)***: `_contextra` (`priv`, L33)

### contextra-rank

> 4-Signal Fusion, Isotonic & Platt Calibration, and Drift Detection for Contextra Cognitive OS

- **Pfad:** `crates/contextra-rank` | **Dateien:** 20 | **Produktiv-Funktionen:** 129 (`96` `pub`, `33` intern) | **Tests:** 9
- **Öffentliche Typen (32)**: `ConformalError`, `ConformalCalibrator`, `AdaptiveConformalCalibrator`, `IsotonicCalibrator`, `PlattScaler`, `DiBudStep`, `DiBudFusionState`, `BudgetedChannel`, `FusionBudget`, `DiBudOutcome`, `DriftStatus`, `DriftDetector`, `SignalKind`, `ExplanationEntry`, `RetrievalExplanation`, `GlobalFusionConfig`, `GlobalFusionStrategy`, `ProvenanceBuilder`, `ResonanceConfig`, `SignalCalibrationContext`, `SignalKey`, `FusedEntry`, `SignalKind`, `MetadataMergePriority`, `BoundedTopK`, `TopKCandidate`, `ProvenanceRecord`, `SignalContribution`, `SearchResult`, `FusedScore`, `Signal`, `SearchReport`

#### `crates/contextra-rank/src/calibration/conformal.rs` (12)

- **trait ConformalCalibrator**: `update` (`priv`, L45), `threshold` (`priv`, L52)
- **AdaptiveConformalCalibrator**: `new` (`pub`, L88), `bounded` (`pub`, L97), `with_params` (`pub`, L107), `with_bounds` (`pub`, L135), `alpha` (`pub`, L156), `learning_rate` (`pub`, L161), `observation_count` (`pub`, L166), `invalidate_on_config_change` (`pub`, L171)
- **ConformalCalibrator for AdaptiveConformalCalibrator**: `update` (`priv`, L188), `threshold` (`priv`, L225)

#### `crates/contextra-rank/src/calibration/isotonic.rs` (17)

- **fmt::Debug for IsotonicCalibrator**: `fmt` (`priv`, L33)
- **IsotonicCalibrator**: `new` (`pub`, L54), `with_clock` (`pub`, L70), `with_defaults` (`pub`, L76), `record_outcome` (`pub`, L81), `observation_count` (`pub`, L91), `is_calibrated` (`pub`, L96), `last_calibration_at` (`pub`, L101), `cached_ece` (`pub`, L106), `calibrated_probability` (`pub`, L111), `force_rebuild` (`pub`, L125), `maybe_rebuild_on_ece` (`pub`, L136), `invalidate_on_config_change` (`pub`, L150), `rebuild_model` (`priv`, L167), `lookup_isotonic` (`priv`, L224), `expected_calibration_error` (`pub`, L247), `calculate_ece` (`priv`, L257)

#### `crates/contextra-rank/src/calibration/platt.rs` (12)

- **Default for PlattScaler**: `default` (`priv`, L14)
- **PlattScaler**: `new` (`pub`, L21), `identity` (`pub`, L30), `is_identity` (`pub`, L39), `is_fitted` (`pub`, L44), `calibration_context` (`pub`, L49), `params` (`pub`, L58), `predict` (`pub`, L63), `apply` (`pub`, L68), `transform` (`pub`, L73), `invalidate_on_config_change` (`pub`, L86), `fit` (`pub`, L101)

#### `crates/contextra-rank/src/dibud/driver.rs` (2)

- ***(Modulfunktionen)***: `fuse_exact_prefix` (`pub`, L10), `fuse_exact_prefix_async` (`pub`, L42)

#### `crates/contextra-rank/src/dibud/state.rs` (16)

- **Default for DiBudFusionState**: `default` (`priv`, L39)
- **DiBudFusionState**: `new` (`pub`, L46), `channel_depths` (`pub`, L58), `exhausted` (`pub`, L63), `accesses` (`pub`, L68), `provenance_of` (`pub`, L73), `explain_doc` (`pub`, L78), `feed` (`pub`, L85), `next_request` (`pub`, L122), `compute_f_minus` (`priv`, L179), `compute_f_plus` (`priv`, L194), `compute_f_plus_unseen` (`priv`, L207), `has_unobserved_unexhausted_channels` (`priv`, L219), `can_overtake` (`priv`, L230), `certify_prefix` (`priv`, L250), `update_provenance` (`priv`, L295)

#### `crates/contextra-rank/src/dibud/types.rs` (5)

- **BudgetedChannel**: `index` (`pub`, L26), `from_index` (`pub`, L32), `all` (`pub`, L42)
- **Default for FusionBudget**: `default` (`priv`, L63)
- **FusionBudget**: `validate` (`pub`, L79)

#### `crates/contextra-rank/src/drift.rs` (7)

- **DriftStatus**: `as_str` (`pub`, L26)
- **DriftDetector**: `new` (`pub`, L58), `with_defaults` (`pub`, L68), `set_baseline` (`pub`, L73), `observe` (`pub`, L80), `analyze` (`pub`, L94), `overall_drift_status` (`pub`, L119)

#### `crates/contextra-rank/src/explain.rs` (5)

- **SignalKind**: `name_de` (`pub`, L25)
- **fmt::Display for SignalKind**: `fmt` (`priv`, L36)
- **RetrievalExplanation**: `to_human_readable_de` (`pub`, L74)
- **fmt::Display for RetrievalExplanation**: `fmt` (`priv`, L120)
- ***(Modulfunktionen)***: `explain` (`pub`, L130)

#### `crates/contextra-rank/src/fusion/global.rs` (4)

- **Default for GlobalFusionConfig**: `default` (`priv`, L26)
- **GlobalFusionStrategy**: `new` (`pub`, L43), `config` (`pub`, L48), `fuse` (`pub`, L61)

#### `crates/contextra-rank/src/fusion/normalized.rs` (2)

- ***(Modulfunktionen)***: `score_normalized_fusion_with_options` (`pub`, L9), `weights_to_signal_factors` (`pub`, L276)

#### `crates/contextra-rank/src/fusion/provenance.rs` (13)

- **ProvenanceBuilder**: `new` (`pub`, L25), `vector` (`pub`, L45), `bm25` (`pub`, L53), `graph` (`pub`, L61), `rerank_score` (`pub`, L69), `source_collection` (`pub`, L75), `index_type` (`pub`, L81), `expected_total` (`pub`, L87), `build` (`pub`, L93), `explain_human_readable` (`pub`, L113)
- **ProvenanceRecord**: `explain_human_readable` (`pub`, L120), `explain_human_readable_with_params` (`pub`, L125)
- ***(Modulfunktionen)***: `build_provenance` (`pub(super)`, L157)

#### `crates/contextra-rank/src/fusion/resonance.rs` (3)

- **Default for ResonanceConfig**: `default` (`priv`, L15)
- ***(Modulfunktionen)***: `fuse_signals` (`pub`, L24), `apply_resonance_bonus` (`pub`, L32)

#### `crates/contextra-rank/src/fusion/rrf.rs` (10)

- ***(Modulfunktionen)***: `g` (`pub`, L16), `reciprocal_rank_fusion` (`pub`, L24), `merge_metadata_ref` (`pub(super)`, L35), `weighted_reciprocal_rank_fusion` (`pub`, L72), `weighted_reciprocal_rank_fusion_with_priority` (`pub`, L86), `modulate_and_renormalize_weights` (`pub`, L98), `weighted_reciprocal_rank_fusion_with_options` (`pub`, L175), `weighted_reciprocal_rank_fusion_mrrf` (`pub`, L194), `fuse_search_results_with_signal_strategies` (`pub`, L544), `fuse_search_results_with_strategy` (`pub`, L954)

#### `crates/contextra-rank/src/fusion/signal.rs` (7)

- **SignalCalibrationContext**: `new` (`pub`, L18), `should_use_calibrated_scores` (`pub`, L33)
- **SignalKey<**: `from_name` (`pub(super)`, L45), `as_str` (`pub(super)`, L55)
- **SignalKind**: `from_name` (`pub`, L94), `as_str` (`pub`, L121)
- **MetadataMergePriority**: `signal_rank` (`pub`, L147)

#### `crates/contextra-rank/src/fusion/topk.rs` (10)

- ***(Modulfunktionen)***: `cmp_scores` (`pub(super)`, L1)
- **BoundedTopK<T>**: `new` (`pub`, L17), `capacity` (`pub`, L26), `len` (`pub`, L31), `is_empty` (`pub`, L36), `push` (`pub`, L41), `into_sorted_vec` (`pub`, L56)
- **PartialEq for TopKCandidate<**: `eq` (`priv`, L68)
- **Ord for TopKCandidate<**: `cmp` (`priv`, L76)
- **PartialOrd for TopKCandidate<**: `partial_cmp` (`priv`, L82)

#### `crates/contextra-rank/src/fusion/types.rs` (4)

- ***(Modulfunktionen)***: `estimate_tokens` (`pub(super)`, L5)
- **TryFrom<SearchResult> for ContextChunk**: `try_from` (`priv`, L17)
- **ProvenanceRecord**: `synthesized_from` (`pub`, L89)
- **std::fmt::Display for Signal**: `fmt` (`priv`, L145)

### contextra-router

- **Pfad:** `crates/contextra-router` | **Dateien:** 28 | **Produktiv-Funktionen:** 101 (`59` `pub`, `42` intern) | **Tests:** 111
- **Öffentliche Typen (24)**: `ArmRegistry`, `ArmRegistryError`, `FcTsDispatchError`, `DecisionId`, `DecisionIdGenerator`, `RoutingOutcome`, `QuantizationLevel`, `SlmProfile`, `ConformalCalibrator`, `ProfileCalibrationState`, `BanditExploration`, `ConfidenceMetrics`, `RoutingDecision`, `RouterState`, `DefaultRouterEngine`, `RouterEngine`, `ProfileScoring`, `RoutingStrategy`, `MockStorageEngine`, `CollectionAdapter`, `TestContextPreparer`, `LogCaptureLayer`, `StringVisitor`, `Transport`

#### `crates/contextra-router/src/arm_registry.rs` (7)

- **Default for ArmRegistry**: `default` (`priv`, L21)
- **ArmRegistry**: `strategy_for` (`pub`, L40), `arm_for` (`pub`, L52), `resolve_strategy` (`pub`, L65), `contains_strategy` (`pub`, L76), `active_strategies` (`pub`, L81), `create_fc_ts_arm_set` (`pub`, L89)

#### `crates/contextra-router/src/bandit_regret_tests.rs` (4)

- **SimpleRng**: `new` (`priv`, L14), `next_u32` (`priv`, L20), `next_f32` (`priv`, L29), `next_vec_f32` (`priv`, L33)

#### `crates/contextra-router/src/dispatch.rs` (3)

- ***(Modulfunktionen)***: `split_endpoint` (`priv`, L20), `dispatch_to_slm` (`pub`, L61), `route_and_dispatch_to_slm` (`pub`, L171)

#### `crates/contextra-router/src/fc_ts_dispatch.rs` (2)

- ***(Modulfunktionen)***: `select_profile_fc_ts` (`pub`, L17), `deterministic_fc_ts_rng` (`pub`, L36)

#### `crates/contextra-router/src/outcome.rs` (6)

- **DecisionId**: `from_raw` (`pub`, L19), `inner` (`pub`, L24)
- **DecisionIdGenerator**: `new` (`pub`, L37), `next` (`pub`, L44)
- **Default for DecisionIdGenerator**: `default` (`priv`, L50)
- **RoutingOutcome**: `non_conformity_score` (`pub`, L70)

#### `crates/contextra-router/src/profile.rs` (24)

- **SlmProfile**: `new` (`pub`, L64), `with_fingerprint` (`pub`, L87), `with_resource_cost_estimate` (`pub`, L93), `estimated_cost` (`pub`, L100), `validate` (`pub`, L109), `try_new` (`pub`, L134), `try_new_with_cost` (`pub`, L153)
- **Default for ConformalCalibrator**: `default` (`priv`, L202)
- **ConformalCalibrator**: `new` (`pub`, L215), `update` (`pub`, L231), `empirical_error_rate` (`pub`, L254), `reset_window` (`pub`, L262)
- **Default for ProfileCalibrationState**: `default` (`priv`, L290)
- **ProfileCalibrationState**: `new` (`pub`, L303), `check_and_invalidate_fingerprint` (`pub`, L316), `is_calibrated` (`pub`, L338), `average_confidence` (`pub`, L349), `empirical_error_rate` (`pub`, L357), `reset_window` (`pub`, L363), `is_healthy` (`pub`, L368), `recalibrate_conformal` (`pub`, L376), `reset` (`pub`, L389)
- ***(Modulfunktionen)***: `serialize` (`pub`, L402), `deserialize` (`pub`, L407)

#### `crates/contextra-router/src/router/dispatch_core.rs` (4)

- **RouterEngine**: `route` (`pub`, L11), `select_profile_cascade` (`pub(crate)`, L293), `select_profile_bandit` (`pub(crate)`, L472), `select_profile_fc_ts_dispatch` (`pub(crate)`, L585)

#### `crates/contextra-router/src/router/lifecycle.rs` (8)

- **RouterEngine**: `new` (`pub`, L9), `with_initial_decision_id` (`pub`, L70), `with_routing_strategy` (`pub`, L77), `try_new` (`pub`, L89), `update_profiles` (`pub`, L116), `try_update_profiles` (`pub`, L123), `apply_profile_update` (`priv`, L131), `profiles` (`pub`, L164)

#### `crates/contextra-router/src/router/mod.rs` (6)

- **BanditExploration**: `new` (`pub`, L53), `next_sample` (`pub`, L65)
- ***(Modulfunktionen)***: `score_profile` (`pub(crate)`, L191), `compute_profile_score` (`pub(crate)`, L238), `compute_profile_scores` (`pub(crate)`, L246)
- **LocalDriftStatusProvider for RouterEngine**: `overall_drift_status` (`priv`, L323)

#### `crates/contextra-router/src/router/outcomes.rs` (10)

- **RouterEngine**: `bandit_decision_propensity` (`pub`, L10), `calibration_stats` (`pub`, L15), `reset_calibration` (`pub`, L20), `drift_status` (`pub`, L37), `overall_drift_status` (`pub`, L47), `set_lyapunov_baseline` (`pub`, L72), `evict_stale_decisions` (`pub(super)`, L86), `record_outcome` (`pub`, L113), `pending_decision_count` (`pub`, L201), `reset_all_calibration` (`pub`, L206)

#### `crates/contextra-router/src/serde_helpers.rs` (2)

- ***(Modulfunktionen)***: `serialize` (`pub`, L13), `deserialize` (`pub`, L19)

#### `crates/contextra-router/src/tests/fixtures.rs` (23)

- **StorageEngine for MockStorageEngine**: `get` (`priv`, L10), `get_at_seq` (`priv`, L13), `put` (`priv`, L20), `delete` (`priv`, L23), `commit` (`priv`, L26), `rollback` (`priv`, L29), `rollback_to_tx` (`priv`, L32), `flush` (`priv`, L35), `stats` (`priv`, L38), `last_seq_no` (`priv`, L47), `last_tx_id` (`priv`, L50), `pin_checkpoint` (`priv`, L53), `unpin_checkpoint` (`priv`, L56), `scan_prefix` (`priv`, L59), `scan` (`priv`, L62)
- **CollectionAdapter<S>**: `new` (`pub(crate)`, L77)
- **crate::ports_local::HybridSearchProvider for CollectionAdapter<S>**: `search_hybrid` (`priv`, L83)
- **crate::ports_local::CommunityResolver for CollectionAdapter<S>**: `get_community` (`priv`, L112)
- **crate::ports_local::ContextPreparer for TestContextPreparer**: `prepare_context` (`priv`, L120)
- ***(Modulfunktionen)***: `create_test_router` (`pub(crate)`, L132), `try_create_test_router` (`pub(crate)`, L148)
- **tracing_subscriber::Layer<S> for LogCaptureLayer**: `on_event` (`priv`, L168)
- **tracing::field::Visit for StringVisitor**: `record_debug` (`priv`, L183)

#### `crates/contextra-router/src/transport.rs` (2)

- **Transport**: `is_cloud` (`pub`, L24)
- **Default for Transport**: `default` (`priv`, L34)

### contextra-sandbox

> WASM Execution Boundary for Contextra MCP CodeExecution Permission

- **Pfad:** `crates/contextra-sandbox` | **Dateien:** 8 | **Produktiv-Funktionen:** 27 (`18` `pub`, `9` intern) | **Tests:** 21
- **Öffentliche Typen (14)**: `ApprovalRisk`, `ApprovalStatus`, `ApprovalTransitionError`, `ApprovalRequest`, `WasmCapabilities`, `MergeOperatorCapabilities`, `SandboxError`, `CapabilityViolationError`, `SandboxState`, `WasmExecutor`, `WasmMergeFunction`, `WasmOutput`, `ProcessExitError`, `OutputLimitExceededError`

#### `crates/contextra-sandbox/src/approval.rs` (9)

- ***(Modulfunktionen)***: `classify_risk` (`pub`, L34), `summarize_capabilities_diff` (`priv`, L241)
- **ApprovalRequest**: `new` (`pub`, L107), `is_expired` (`pub`, L129), `approve` (`pub`, L137), `reject` (`pub`, L167), `expire` (`pub`, L197), `auto_approve_if_low_risk` (`pub`, L217), `requires_approval` (`pub`, L235)

#### `crates/contextra-sandbox/src/capabilities.rs` (4)

- **Default for WasmCapabilities**: `default` (`priv`, L50)
- **WasmCapabilities**: `pure_merge_operator` (`pub`, L84)
- **MergeOperatorCapabilities**: `pure` (`pub`, L110), `pure_with_result_channel` (`pub`, L118)

#### `crates/contextra-sandbox/src/executor.rs` (7)

- **std::fmt::Display for CapabilityViolationError**: `fmt` (`priv`, L32)
- **wasmtime::ResourceLimiter for SandboxState**: `memory_growing` (`priv`, L56), `table_growing` (`priv`, L70)
- **std::fmt::Debug for WasmExecutor**: `fmt` (`priv`, L92)
- **WasmExecutor**: `new` (`pub`, L102), `validate_module` (`pub`, L115), `execute` (`pub`, L145)

#### `crates/contextra-sandbox/src/merge.rs` (3)

- **WasmMergeFunction**: `new` (`pub`, L41), `new_with_capabilities` (`pub`, L49), `merge` (`pub`, L77)

#### `crates/contextra-sandbox/src/output.rs` (1)

- **WasmOutput**: `new` (`pub`, L22)

#### `crates/contextra-sandbox/src/wasi.rs` (3)

- **std::fmt::Display for ProcessExitError**: `fmt` (`priv`, L21)
- **std::fmt::Display for OutputLimitExceededError**: `fmt` (`priv`, L36)
- ***(Modulfunktionen)***: `register` (`pub(crate)`, L55)

### contextra-simd

> Ring 0 SIMD distance kernels and runtime dispatch for Contextra (Unsafe Island)

- **Pfad:** `crates/contextra-simd` | **Dateien:** 7 | **Produktiv-Funktionen:** 51 (`51` `pub`, `0` intern) | **Tests:** 0
- **Öffentliche Typen (2)**: `CosineSimilarityPartsU8`, `CosineSimilarityPartsF32U8`

#### `crates/contextra-simd/src/dispatch.rs` (9)

- ***(Modulfunktionen)***: `cosine_distance` (`pub`, L9), `euclidean_distance` (`pub`, L38), `dot_product_distance` (`pub`, L67), `cosine_distance_f32_bytes` (`pub`, L96), `euclidean_distance_f32_bytes` (`pub`, L118), `dot_product_distance_f32_bytes` (`pub`, L140), `dot_product_u8` (`pub`, L162), `euclidean_distance_sq_u8` (`pub`, L187), `cosine_similarity_parts_u8` (`pub`, L209)

#### `crates/contextra-simd/src/kernels/avx2.rs` (11)

- ***(Modulfunktionen)***: `cosine_distance_avx2` (`pub`, L13), `euclidean_distance_avx2` (`pub`, L57), `dot_product_avx2` (`pub`, L87), `cosine_distance_f32_bytes_avx2` (`pub`, L115), `euclidean_distance_f32_bytes_avx2` (`pub`, L164), `dot_product_f32_bytes_avx2` (`pub`, L200), `hsum256_ps_avx` (`pub`, L234), `dot_product_u8_avx2` (`pub`, L246), `euclidean_distance_sq_u8_avx2` (`pub`, L284), `cosine_similarity_parts_u8_avx2` (`pub`, L325), `hsum256_epi32_avx2` (`pub`, L376)

#### `crates/contextra-simd/src/kernels/avx512.rs` (11)

- ***(Modulfunktionen)***: `cosine_distance_avx512` (`pub`, L13), `euclidean_distance_avx512` (`pub`, L57), `dot_product_avx512` (`pub`, L87), `cosine_distance_f32_bytes_avx512` (`pub`, L115), `euclidean_distance_f32_bytes_avx512` (`pub`, L164), `dot_product_f32_bytes_avx512` (`pub`, L200), `hsum512_ps_avx` (`pub`, L234), `dot_product_u8_avx512vnni` (`pub`, L247), `euclidean_distance_sq_u8_avx512` (`pub`, L275), `cosine_similarity_parts_u8_avx512` (`pub`, L316), `hsum512_epi32_avx512` (`pub`, L358)

#### `crates/contextra-simd/src/kernels/neon.rs` (3)

- ***(Modulfunktionen)***: `cosine_distance_neon` (`pub`, L11), `euclidean_distance_neon` (`pub`, L55), `dot_product_neon` (`pub`, L85)

#### `crates/contextra-simd/src/kernels/scalar.rs` (13)

- ***(Modulfunktionen)***: `cosine_distance_scalar` (`pub`, L20), `euclidean_distance_scalar` (`pub`, L41), `dot_product_scalar` (`pub`, L51), `cosine_distance_f32_bytes_scalar` (`pub`, L60), `euclidean_distance_f32_bytes_scalar` (`pub`, L87), `dot_product_f32_bytes_scalar` (`pub`, L103), `dot_product_u8_scalar` (`pub`, L118), `euclidean_distance_sq_u8_scalar` (`pub`, L128), `cosine_similarity_parts_u8_scalar` (`pub`, L139), `dot_product_f32_u8` (`pub`, L161), `euclidean_distance_sq_f32_u8` (`pub`, L171), `cosine_similarity_parts_f32_u8` (`pub`, L183), `normalize_inplace` (`pub`, L204)

#### `crates/contextra-simd/src/lib.rs` (4)

- ***(Modulfunktionen)***: `validate_vector` (`pub`, L33), `compute_distance` (`pub`, L43), `compute_distance_trusted` (`pub`, L54), `compute_distance_f32_bytes_trusted` (`pub`, L68)

### contextra-store

> LSM-Tree storage engine for Contextra

- **Pfad:** `crates/contextra-store` | **Dateien:** 70 | **Produktiv-Funktionen:** 607 (`286` `pub`, `321` intern) | **Tests:** 246
- **Öffentliche Typen (73)**: `WorkloadMetrics`, `WorkloadMetricsSnapshot`, `CompactionStrategy`, `AdaptiveCompactionPlan`, `AdaptiveCompactionPlanner`, `CostBasedAdaptivePlanner`, `CompactionConfig`, `CompactionEngine`, `MergeOperator`, `InMemoryStorageEngine`, `KvDeleteMode`, `KvSegmentConfig`, `KvSegmentPayload`, `KvSegmentManager`, `DurabilityMode`, `DurabilityConfigError`, `LsmConfig`, `StorageHealth`, `LsmStorage`, `GroupCommitRequest`, `WalQueueGuard`, `PendingCommitQueue`, `CommitGuard`, `LsmState`, `WriteOrigin`, `WalEntryRef`, `CommitContext`, `CommittedBatch`, `WalObserver`, `ObserverRegistry`, `AsyncObserverAdapter`, `SstableScanMode`, `Manifest`, `ManifestEntry`, `MemTable`, `SieveCacheBackend`, `BlockCacheBackend`, `LruBlockCacheBackend`, `QuickCacheBlockCacheBackend`, `BlockCacheShard` *(+33 weitere)*

#### `crates/contextra-store/src/compaction/adaptive.rs` (14)

- **WorkloadMetrics**: `new` (`pub`, L25), `record_read` (`pub`, L35), `record_write` (`pub`, L41), `snapshot` (`pub`, L47)
- **Default for WorkloadMetrics**: `default` (`priv`, L57)
- **PartialEq for AdaptiveCompactionPlan**: `eq` (`priv`, L90)
- **std::fmt::Debug for AdaptiveCompactionPlan**: `fmt` (`priv`, L105)
- **trait AdaptiveCompactionPlanner**: `plan_compaction` (`priv`, L117)
- **CostBasedAdaptivePlanner**: `new` (`pub`, L138), `validate_tombstone_safety` (`pub`, L149)
- **Default for CostBasedAdaptivePlanner**: `default` (`priv`, L175)
- **AdaptiveCompactionPlanner for CostBasedAdaptivePlanner**: `plan_compaction` (`priv`, L181)
- ***(Modulfunktionen)***: `compute_key_overlap_score` (`pub(super)`, L250), `select_stcs_candidates` (`pub(super)`, L268)

#### `crates/contextra-store/src/compaction/config.rs` (1)

- **Default for CompactionConfig**: `default` (`priv`, L75)

#### `crates/contextra-store/src/compaction/engine.rs` (19)

- **CompactionEngine**: `new` (`pub`, L32), `with_merge_operator` (`pub`, L68), `with_clock` (`pub`, L74), `with_pressure_rx` (`pub`, L80), `with_adaptive_planner` (`pub`, L90), `record_read_op` (`pub`, L97), `record_write_op` (`pub`, L103), `maybe_compact` (`pub`, L112), `maybe_compact_with_cancel` (`pub`, L122), `select_compaction_candidates` (`pub`, L354), `estimate_compaction_peak_bytes` (`pub(super)`, L369), `merge_sstables` (`pub`, L389), `merge_sstables_with_cancel` (`pub`, L407), `merge_sstables_inner` (`priv`, L440)
- **PartialEq for HeapItem**: `eq` (`priv`, L457)
- **PartialOrd for HeapItem**: `partial_cmp` (`priv`, L467)
- **Ord for HeapItem**: `cmp` (`priv`, L473)
- ***(Modulfunktionen)***: `generate_sst_path` (`pub`, L771), `run_loop` (`pub`, L783)

#### `crates/contextra-store/src/compaction/merge_operator.rs` (1)

- **trait MergeOperator**: `merge` (`priv`, L15)

#### `crates/contextra-store/src/compaction/retention.rs` (1)

- ***(Modulfunktionen)***: `retain_key_versions` (`pub`, L12)

#### `crates/contextra-store/src/compaction/tests.rs` (1)

- ***(Modulfunktionen)***: `create_test_sstable` (`pub(super)`, L10)

#### `crates/contextra-store/src/compaction/tests/advanced.rs` (3)

- ***(Modulfunktionen)***: `test_compaction_stress_and_gc` (`priv`, L4), `test_compaction_pressure_awareness` (`priv`, L383), `create_padded_sst` (`priv`, L842)

#### `crates/contextra-store/src/compaction/tests/basic.rs` (1)

- ***(Modulfunktionen)***: `create_test_sstable` (`priv`, L6)

#### `crates/contextra-store/src/engine/memory_only.rs` (17)

- **Default for InMemoryStorageEngine**: `default` (`priv`, L40)
- **InMemoryStorageEngine**: `new` (`pub`, L52)
- **StorageEngine for InMemoryStorageEngine**: `get` (`priv`, L58), `get_at_seq` (`priv`, L70), `put` (`priv`, L84), `delete` (`priv`, L95), `commit` (`priv`, L106), `rollback` (`priv`, L137), `rollback_to_tx` (`priv`, L145), `flush` (`priv`, L169), `stats` (`priv`, L173), `last_seq_no` (`priv`, L194), `last_tx_id` (`priv`, L198), `pin_checkpoint` (`priv`, L202), `unpin_checkpoint` (`priv`, L206), `scan_prefix` (`priv`, L210), `scan` (`priv`, L232)

#### `crates/contextra-store/src/kv/delete_mode.rs` (1)

- **Default for KvDeleteMode**: `default` (`priv`, L19)

#### `crates/contextra-store/src/kv/segment.rs` (6)

- **KvSegmentManager**: `new` (`pub`, L44), `delete_mode` (`pub`, L57), `write_segment` (`pub`, L66), `read_segment` (`pub`, L108), `delete_segment` (`pub`, L148), `generate_deletion_proof` (`pub`, L161)

#### `crates/contextra-store/src/lsm/commit.rs` (9)

- **LsmStorage**: `notify_commit_observers` (`pub(super)`, L9), `cleanup_intent_locks_where` (`pub`, L27), `clear_intent_locks_for_tx` (`pub`, L35), `clear_intent_locks_above_tx` (`pub`, L40), `cleanup_intent_locks_for_tx` (`pub(super)`, L44), `cleanup_intent_locks_for_txs` (`pub(super)`, L48), `apply_backpressure` (`pub(super)`, L53), `advance_visibility` (`pub(super)`, L65), `apply_mem_updates` (`pub(super)`, L99)

#### `crates/contextra-store/src/lsm/config.rs` (4)

- **Default for DurabilityMode**: `default` (`priv`, L15)
- **From<DurabilityConfigError> for contextra_core::ContextraError**: `from` (`priv`, L31)
- **DurabilityMode**: `validate_against_features` (`pub`, L37)
- **Default for LsmConfig**: `default` (`priv`, L100)

#### `crates/contextra-store/src/lsm/engine.rs` (38)

- **Drop for LsmStorage**: `drop` (`priv`, L66)
- **LsmStorage**: `open` (`pub`, L73), `open_with_merge_operator` (`pub`, L79), `pressure_receiver` (`pub`, L88), `shutdown` (`pub`, L95), `wait_shutdown` (`pub`, L101), `close` (`pub`, L107), `spawn_tracked` (`pub`, L120), `simulate_wal_append_failure_for_test` (`pub`, L128), `simulate_wal_append_failure_for_tx_for_test` (`pub`, L134), `restore_wal_file_handle_for_test` (`pub`, L140), `budget_tracking_drift_bytes` (`pub`, L147), `register_observer` (`pub`, L157), `deregister_observer` (`pub`, L162), `set_max_observer_latency` (`pub`, L167), `max_observer_latency` (`pub`, L172), `metrics_sink` (`pub`, L177), `set_metrics_sink` (`pub`, L182), `with_metrics_sink` (`pub`, L187), `set_clock` (`pub`, L193), `get_tracked` (`pub`, L207), `get_at_seq_tracked` (`pub`, L220), `scan_prefix_tracked` (`pub`, L233), `last_applied_seq` (`pub`, L242), `dropped_count_for` (`pub`, L248), `clear_circuit_breaker` (`pub`, L253), `health` (`pub`, L258), `request_flush` (`pub`, L267), `has_observers` (`pub`, L272), `next_seq_no_for_test` (`pub`, L277), `sstables_for_test` (`pub`, L282), `block_cache_for_test` (`pub`, L287), `budget_for_test` (`pub`, L292), `manifest_for_test` (`pub`, L297), `maybe_compact_for_test` (`pub`, L302), `has_pending_legacy_wal_migration_path` (`pub`, L313), `has_pending_legacy_wal_migration` (`pub`, L365), `migrate_legacy_wal_keys` (`pub`, L377)

#### `crates/contextra-store/src/lsm/flush.rs` (2)

- **LsmStorage**: `force_flush` (`pub`, L6), `maybe_compact` (`pub`, L14)

#### `crates/contextra-store/src/lsm/group_commit.rs` (3)

- ***(Modulfunktionen)***: `execute_group_commit_append` (`pub(super)`, L7)
- **WalQueueGuard**: `new` (`pub(super)`, L60)
- **Drop for WalQueueGuard**: `drop` (`priv`, L67)

#### `crates/contextra-store/src/lsm/observer.rs` (28)

- **WalEntryRef<**: `new` (`pub`, L31), `op` (`pub`, L36), `seq_no` (`pub`, L41), `checksum` (`pub`, L46), `prev_hmac` (`pub`, L51), `tx_id` (`pub`, L56)
- **trait WalObserver**: `requires_durability` (`priv`, L81), `on_commit` (`priv`, L91)
- **BoundedObserverWorker**: `new` (`priv`, L121), `is_circuit_breaker_open` (`priv`, L168)
- **Default for ObserverRegistry**: `default` (`priv`, L185)
- **AsyncObserverAdapter**: `new` (`pub`, L202)
- **WalObserver for AsyncObserverAdapter**: `requires_durability` (`priv`, L224), `on_commit` (`priv`, L228)
- **ObserverRegistry**: `is_empty` (`pub`, L237), `new` (`pub`, L242), `register_observer` (`pub`, L247), `deregister_observer` (`pub`, L254), `set_max_observer_latency` (`pub`, L270), `max_observer_latency` (`pub`, L275), `set_clock` (`pub`, L280), `dropped_count` (`pub`, L285), `dropped_count_for` (`pub`, L296), `is_circuit_breaker_open` (`pub`, L306), `is_any_circuit_breaker_open` (`pub`, L319), `clear_circuit_breaker` (`pub`, L328), `notify_with_context` (`pub`, L343), `notify` (`pub`, L467)

#### `crates/contextra-store/src/lsm/ops.rs` (25)

- **StorageEngine for LsmStorage**: `get` (`priv`, L9), `get_at_seq` (`priv`, L13), `get_tracked` (`priv`, L21), `get_at_seq_tracked` (`priv`, L29), `scan_prefix` (`priv`, L38), `scan_prefix_tracked` (`priv`, L45), `supports_ssi_tracking` (`priv`, L53), `scan_prefix_bounded` (`priv`, L57), `scan_prefix_at` (`priv`, L66), `scan_bounded` (`priv`, L74), `scan` (`priv`, L84), `put` (`priv`, L93), `put_if_absent` (`priv`, L97), `delete_many` (`priv`, L106), `delete_prefix` (`priv`, L110), `delete` (`priv`, L114), `commit` (`priv`, L118), `rollback` (`priv`, L122), `rollback_to_tx` (`priv`, L126), `pin_checkpoint` (`priv`, L130), `unpin_checkpoint` (`priv`, L134), `flush` (`priv`, L138), `stats` (`priv`, L142), `last_seq_no` (`priv`, L146), `last_tx_id` (`priv`, L150)

#### `crates/contextra-store/src/lsm/ops/compaction.rs` (1)

- ***(Modulfunktionen)***: `flush` (`pub(super)`, L15)

#### `crates/contextra-store/src/lsm/ops/maintenance.rs` (7)

- ***(Modulfunktionen)***: `rollback` (`pub(super)`, L5), `rollback_to_tx` (`pub(super)`, L11), `pin_checkpoint` (`pub(super)`, L15), `unpin_checkpoint` (`pub(super)`, L20), `stats` (`pub(super)`, L25), `last_seq_no` (`pub(super)`, L46), `last_tx_id` (`pub(super)`, L50)

#### `crates/contextra-store/src/lsm/ops/read.rs` (10)

- ***(Modulfunktionen)***: `get_tracked` (`pub(crate)`, L8), `get_at_seq_tracked` (`pub(crate)`, L32), `scan_prefix_tracked` (`pub(crate)`, L47), `get` (`pub(super)`, L65), `get_at_seq` (`pub(super)`, L78), `scan_prefix` (`pub(super)`, L148), `scan_prefix_bounded` (`pub(super)`, L155), `scan_prefix_at` (`pub(super)`, L218), `scan_bounded` (`pub(super)`, L243), `scan` (`pub(super)`, L300)

#### `crates/contextra-store/src/lsm/ops/write.rs` (11)

- ***(Modulfunktionen)***: `put` (`pub(super)`, L15), `put_if_absent` (`pub(super)`, L64), `delete` (`pub(super)`, L131), `delete_many` (`pub(super)`, L151), `delete_prefix` (`pub(super)`, L192), `commit` (`pub(super)`, L220), `commit_internal` (`priv`, L237)
- **PutIfAbsentLockGuard<**: `disarm` (`priv`, L44)
- **Drop for PutIfAbsentLockGuard<**: `drop` (`priv`, L50)
- **Drop for IntentLockGuard<**: `drop` (`priv`, L247)
- **Drop for LeaderCancelGuard<**: `drop` (`priv`, L530)

#### `crates/contextra-store/src/lsm/recovery.rs` (9)

- ***(Modulfunktionen)***: `next_temp_file_counter` (`pub(crate)`, L21), `generate_crypto_salt` (`pub(crate)`, L28), `write_salt_atomically` (`pub(crate)`, L35), `directory_is_pristine` (`pub(super)`, L115)
- **LsmStorage**: `new` (`pub`, L145), `new_with_merge_operator` (`pub`, L149), `rollback_to_tx` (`pub`, L689), `rollback_to_tx_locked` (`pub(super)`, L697), `prune_memtable_above_tx` (`priv`, L739)

#### `crates/contextra-store/src/lsm/scan.rs` (4)

- ***(Modulfunktionen)***: `upper_bound_for_prefix` (`priv`, L11), `check_in_range` (`pub(super)`, L23)
- **LsmStorage**: `point_lookup_metrics` (`pub`, L43), `collect_visible_entries` (`pub(super)`, L81)

#### `crates/contextra-store/src/lsm/tests.rs` (1)

- ***(Modulfunktionen)***: `test_storage` (`pub(crate)`, L8)

#### `crates/contextra-store/src/lsm/tests/mvcc_tests.rs` (1)

- ***(Modulfunktionen)***: `test_storage` (`priv`, L4)

#### `crates/contextra-store/src/lsm/validate.rs` (4)

- ***(Modulfunktionen)***: `validate_key` (`pub`, L4), `derive_doc_id` (`pub`, L19), `derive_doc_id` (`pub`, L27), `validate_value` (`pub`, L34)

#### `crates/contextra-store/src/manifest/core.rs` (10)

- **std::fmt::Debug for Manifest**: `fmt` (`priv`, L28)
- **Manifest**: `open` (`pub`, L37), `open_with_dir_lock` (`pub(crate)`, L42), `append` (`pub`, L100), `append_batch` (`pub`, L105), `load` (`pub`, L145), `reconstruct_valid_sstables` (`pub`, L277), `reconstruct_dead_sstables` (`pub`, L330), `extract_high_water_mark` (`pub`, L370), `path` (`pub`, L379)

#### `crates/contextra-store/src/manifest/entry.rs` (2)

- **ManifestEntry**: `to_bytes` (`pub`, L44), `from_bytes` (`pub`, L113)

#### `crates/contextra-store/src/manifest/recovery.rs` (1)

- ***(Modulfunktionen)***: `is_valid_tail_truncation_candidate` (`pub(super)`, L9)

#### `crates/contextra-store/src/manifest/rollover.rs` (2)

- **Manifest**: `rollover` (`pub`, L24), `maybe_rollover` (`pub`, L41)

#### `crates/contextra-store/src/memtable.rs` (24)

- ***(Modulfunktionen)***: `is_tombstone` (`pub`, L46), `is_valid_range_bounds` (`pub`, L52), `matching_reserved_prefix` (`priv`, L90), `hash_remainder` (`priv`, L104)
- **MemTable**: `new` (`pub`, L131), `shard_for` (`pub`, L157), `shard_range_for_prefix` (`priv`, L171), `shard_range_for_bounds` (`priv`, L188), `put` (`pub`, L220), `tx_range` (`pub`, L267), `rollback` (`pub`, L275), `get` (`pub`, L321), `get_at_seq` (`pub`, L331), `saturating_sub_size` (`priv`, L372), `size` (`pub`, L381), `is_empty` (`pub`, L390), `iter` (`pub`, L396), `scan_prefix_into` (`pub`, L419), `scan_prefix_into_matching` (`pub`, L430), `scan_range_into` (`pub`, L489), `scan_range_into_matching` (`pub`, L501), `iter_latest` (`pub`, L556), `shard_entry_counts` (`pub`, L574)
- **Default for MemTable**: `default` (`priv`, L580)

#### `crates/contextra-store/src/sstable/block_cache.rs` (32)

- **BlockCacheBackend for SieveCacheBackend**: `new` (`priv`, L45), `get` (`priv`, L59), `insert` (`priv`, L70), `len` (`priv`, L128)
- **trait BlockCacheBackend**: `new` (`priv`, L135), `get` (`priv`, L140), `insert` (`priv`, L143), `len` (`priv`, L146), `is_empty` (`priv`, L149), `contains` (`priv`, L154)
- **BlockCacheBackend for LruBlockCacheBackend**: `new` (`priv`, L171), `get` (`priv`, L183), `insert` (`priv`, L188), `len` (`priv`, L206), `contains` (`priv`, L211)
- **quick_cache::Weighter<**: `weight` (`priv`, L222)
- **BlockCacheBackend for QuickCacheBlockCacheBackend**: `new` (`priv`, L235), `get` (`priv`, L247), `insert` (`priv`, L252), `len` (`priv`, L257), `contains` (`priv`, L262)
- **BlockCache**: `new` (`pub`, L279), `new_with_shards` (`pub`, L283), `shard_idx` (`pub`, L298), `shard` (`priv`, L304), `get` (`pub`, L310), `insert` (`pub`, L315), `len` (`pub`, L320), `is_empty` (`pub`, L325), `contains` (`pub`, L330)
- ***(Modulfunktionen)***: `create_block_cache` (`pub`, L344), `create_block_cache_with_shards` (`pub`, L349)

#### `crates/contextra-store/src/sstable/block_search.rs` (6)

- ***(Modulfunktionen)***: `get_entry_at_index` (`pub`, L4), `get_entry_key_at_index` (`pub`, L74), `binary_search_first_index_in_block` (`pub`, L93), `binary_search_index_in_block` (`pub`, L125), `binary_search_entry_in_block` (`pub`, L169), `block_binary_search` (`pub`, L185)

#### `crates/contextra-store/src/sstable/bloom.rs` (6)

- **BloomFilter**: `new` (`pub`, L24), `insert` (`pub`, L48), `may_contain` (`pub`, L59), `hash_pair` (`pub`, L75), `to_bytes` (`pub`, L88), `from_bytes` (`pub`, L101)

#### `crates/contextra-store/src/sstable/builder.rs` (15)

- **BlockBuilder**: `new` (`pub`, L22), `new_with_version` (`pub`, L26), `update_bloom` (`priv`, L37), `add` (`pub`, L54), `current_size` (`pub`, L108), `is_empty` (`pub`, L132), `build` (`pub`, L137)
- **SstableBuilder**: `create` (`pub`, L221), `create_with_version` (`pub`, L226), `current_size` (`pub`, L233), `create_with_key_manager` (`pub`, L238), `set_format_version` (`pub`, L279), `add` (`pub`, L285), `flush_block` (`priv`, L346), `finish` (`pub`, L403)

#### `crates/contextra-store/src/sstable/io.rs` (2)

- ***(Modulfunktionen)***: `pread_exact` (`pub(super)`, L1), `pread_exact` (`pub(super)`, L11)

#### `crates/contextra-store/src/sstable/reader.rs` (17)

- ***(Modulfunktionen)***: `parse_block_trailer` (`pub(super)`, L17), `may_contain_key_in_block_bloom` (`pub(super)`, L135)
- **SstableReader**: `first_key` (`pub`, L238), `last_key` (`pub`, L242), `min_tx_id` (`pub`, L246), `max_tx_id` (`pub`, L250), `file_path` (`pub`, L254), `open` (`pub`, L259), `open_with_key_manager` (`pub`, L263), `read_block_at_file` (`priv`, L674), `get_block` (`pub(super)`, L737), `get_at` (`pub`, L756), `get` (`pub`, L900), `lookup_metrics` (`pub`, L905), `metadata` (`pub`, L960), `stream` (`pub`, L965), `iter` (`pub`, L977)

#### `crates/contextra-store/src/sstable/reader_ext.rs` (2)

- **SstableReader**: `scan_prefix` (`pub`, L8), `scan_range` (`pub`, L154)

#### `crates/contextra-store/src/sstable/stream.rs` (2)

- **SstableStream**: `next` (`pub`, L16), `next_entry` (`pub`, L130)

#### `crates/contextra-store/src/system_pressure.rs` (4)

- **SystemPressureMonitor**: `new` (`pub`, L66), `compute_pressure` (`pub`, L82), `run` (`pub`, L114), `run_with_metrics_sink` (`pub`, L131)

#### `crates/contextra-store/src/tenant_codec.rs` (117)

- **TenantKeyCodec**: `new` (`pub`, L32), `encode_chunk_key` (`pub`, L43), `encode_graph_key` (`pub`, L56), `scan_prefix` (`pub`, L69), `collection_prefix` (`pub`, L75), `decode_tenant_id` (`pub`, L82)
- **trait StorageEngineHandle**: `get_handle` (`priv`, L97), `get_at_seq_handle` (`priv`, L98), `get_tracked_handle` (`priv`, L103), `get_at_seq_tracked_handle` (`priv`, L108), `scan_prefix_tracked_handle` (`priv`, L115), `supports_ssi_tracking_handle` (`priv`, L120), `put_handle` (`priv`, L121), `put_if_absent_handle` (`priv`, L127), `put_batch_handle` (`priv`, L133), `delete_handle` (`priv`, L138), `delete_many_handle` (`priv`, L139), `delete_prefix_handle` (`priv`, L144), `commit_handle` (`priv`, L149), `rollback_handle` (`priv`, L150), `rollback_to_tx_handle` (`priv`, L151), `flush_handle` (`priv`, L152), `stats_handle` (`priv`, L153), `last_seq_no_handle` (`priv`, L154), `last_tx_id_handle` (`priv`, L155), `pin_checkpoint_handle` (`priv`, L156), `unpin_checkpoint_handle` (`priv`, L157), `scan_prefix_handle` (`priv`, L159), `scan_prefix_bounded_handle` (`priv`, L164), `scan_prefix_at_handle` (`priv`, L171), `scan_handle` (`priv`, L177), `scan_bounded_handle` (`priv`, L184)
- **StorageEngineHandle for LsmStorage**: `get_handle` (`priv`, L194), `get_at_seq_handle` (`priv`, L197), `get_tracked_handle` (`priv`, L204), `get_at_seq_tracked_handle` (`priv`, L211), `scan_prefix_tracked_handle` (`priv`, L219), `supports_ssi_tracking_handle` (`priv`, L226), `put_handle` (`priv`, L229), `put_if_absent_handle` (`priv`, L237), `put_batch_handle` (`priv`, L245), `delete_handle` (`priv`, L252), `delete_many_handle` (`priv`, L255), `delete_prefix_handle` (`priv`, L262), `commit_handle` (`priv`, L269), `rollback_handle` (`priv`, L272), `rollback_to_tx_handle` (`priv`, L275), `flush_handle` (`priv`, L278), `stats_handle` (`priv`, L281), `last_seq_no_handle` (`priv`, L284), `last_tx_id_handle` (`priv`, L287), `pin_checkpoint_handle` (`priv`, L290), `unpin_checkpoint_handle` (`priv`, L293), `scan_prefix_handle` (`priv`, L296), `scan_prefix_bounded_handle` (`priv`, L302), `scan_prefix_at_handle` (`priv`, L310), `scan_handle` (`priv`, L317), `scan_bounded_handle` (`priv`, L325)
- **StorageEngineHandle for Arc<S>**: `get_handle` (`priv`, L337), `get_at_seq_handle` (`priv`, L340), `get_tracked_handle` (`priv`, L347), `get_at_seq_tracked_handle` (`priv`, L354), `scan_prefix_tracked_handle` (`priv`, L362), `supports_ssi_tracking_handle` (`priv`, L369), `put_handle` (`priv`, L372), `put_if_absent_handle` (`priv`, L380), `put_batch_handle` (`priv`, L388), `delete_handle` (`priv`, L395), `delete_many_handle` (`priv`, L398), `delete_prefix_handle` (`priv`, L405), `commit_handle` (`priv`, L412), `rollback_handle` (`priv`, L415), `rollback_to_tx_handle` (`priv`, L418), `flush_handle` (`priv`, L421), `stats_handle` (`priv`, L424), `last_seq_no_handle` (`priv`, L427), `last_tx_id_handle` (`priv`, L430), `pin_checkpoint_handle` (`priv`, L433), `unpin_checkpoint_handle` (`priv`, L436), `scan_prefix_handle` (`priv`, L439), `scan_prefix_bounded_handle` (`priv`, L445), `scan_prefix_at_handle` (`priv`, L453), `scan_handle` (`priv`, L460), `scan_bounded_handle` (`priv`, L468)
- **TenantScopedStorage<S>**: `new` (`pub`, L487), `tenant_id` (`pub`, L495), `make_key` (`priv`, L500), `strip_key` (`priv`, L509), `filter_strip_batch` (`priv`, L518)
- ***(Modulfunktionen)***: `upper_bound_for_prefix` (`priv`, L533), `bound_as_ref` (`priv`, L544)
- **StorageEngine for TenantScopedStorage<S>**: `get` (`priv`, L553), `get_at_seq` (`priv`, L560), `get_tracked` (`priv`, L571), `get_at_seq_tracked` (`priv`, L582), `scan_prefix_tracked` (`priv`, L596), `supports_ssi_tracking` (`priv`, L611), `put` (`priv`, L615), `put_if_absent` (`priv`, L622), `put_batch` (`priv`, L636), `delete` (`priv`, L650), `delete_many` (`priv`, L657), `delete_prefix` (`priv`, L664), `commit` (`priv`, L673), `rollback` (`priv`, L677), `rollback_to_tx` (`priv`, L681), `flush` (`priv`, L685), `stats` (`priv`, L689), `last_seq_no` (`priv`, L693), `last_tx_id` (`priv`, L697), `pin_checkpoint` (`priv`, L701), `unpin_checkpoint` (`priv`, L705), `scan_prefix` (`priv`, L709), `scan_prefix_bounded` (`priv`, L720), `scan_prefix_at` (`priv`, L739), `scan` (`priv`, L754), `scan_bounded` (`priv`, L792)

#### `crates/contextra-store/src/util.rs` (3)

- **DirLock**: `acquire` (`pub(crate)`, L22)
- ***(Modulfunktionen)***: `decide_on_lock_error` (`priv`, L44), `fsync_parent_dir` (`pub(crate)`, L106)

#### `crates/contextra-store/src/wal/encode.rs` (14)

- **WalOp**: `tx_id` (`pub`, L45)
- **PreparedBatch**: `is_empty` (`pub`, L76), `len` (`pub`, L81), `entries` (`pub`, L86), `into_inner` (`pub`, L91), `extend` (`pub`, L96)
- **WalEntry**: `tx_id` (`pub`, L115), `try_new` (`pub`, L122), `compute_checksum` (`pub`, L138), `compute_checksum_v2` (`pub`, L148), `compute_checksum_v3` (`pub`, L185), `to_bytes` (`pub`, L231), `from_bytes_classified` (`pub`, L296), `from_bytes` (`pub`, L450)

#### `crates/contextra-store/src/wal/flusher.rs` (4)

- **std::fmt::Debug for WalCommand**: `fmt` (`priv`, L41)
- **Default for WalFlusherConfig**: `default` (`priv`, L96)
- ***(Modulfunktionen)***: `encode_encrypted_rewrite_chunks` (`priv`, L104)
- **Wal**: `enable_flusher_with_config` (`pub(crate)`, L154)

#### `crates/contextra-store/src/wal/hmac.rs` (23)

- ***(Modulfunktionen)***: `warn_legacy_key_use_once` (`pub(crate)`, L38), `deobfuscate_legacy_integrity_key` (`pub(crate)`, L75), `deobfuscate_legacy_integrity_key` (`pub(crate)`, L91), `legacy_integrity_key` (`pub(crate)`, L97), `legacy_integrity_key` (`pub(crate)`, L103), `migrate_legacy_key` (`pub`, L124), `migrate_legacy_key` (`pub`, L153)
- **LegacyKeyStatus**: `is_legacy` (`pub`, L60), `is_standard` (`pub`, L65)
- **Wal**: `prepare_batch` (`pub`, L161), `legacy_key_status` (`pub`, L187), `get_integrity_key` (`pub(crate)`, L199), `integrity_key_for_test` (`pub`, L217), `legacy_integrity_key_for_test` (`pub`, L223), `load_or_create_integrity_key` (`pub(crate)`, L234), `read_key_file` (`priv`, L247), `migration_marker_path` (`pub(crate)`, L338), `has_migration_marker` (`pub(crate)`, L342), `calculate_retirement_marker_hash` (`pub(crate)`, L347), `write_migration_marker_atomically` (`pub(crate)`, L368), `write_migration_marker_with_key_atomically` (`pub(crate)`, L373), `load_or_create_wal_uuid` (`pub(crate)`, L445), `read_uuid_file` (`priv`, L452)

#### `crates/contextra-store/src/wal/io.rs` (16)

- ***(Modulfunktionen)***: `do_scan_entries_with_callback` (`pub(crate)`, L19), `set_restrictive_file_acl` (`pub(crate)`, L1011)
- **Wal**: `prepare_append_payload` (`pub(crate)`, L524), `append_batch` (`pub`, L688), `enqueue_append_batch_locked` (`pub`, L700), `append_batch_locked` (`pub`, L739), `try_append_batch` (`pub`, L753), `try_enqueue_append_batch_locked` (`pub`, L758), `try_append_batch_locked` (`pub`, L803), `scan_entries_with_callback` (`pub`, L819), `rewrite_as_v3` (`pub(crate)`, L861), `truncate` (`pub`, L903), `rotate_and_seal` (`pub`, L936), `find_tx_offset` (`pub`, L962), `last_hmac_snapshot` (`pub`, L995), `arm_truncate_failure_for_test` (`pub`, L1005)

#### `crates/contextra-store/src/wal/mod.rs` (61)

- ***(Modulfunktionen)***: `read` (`pub`, L31), `write` (`pub`, L36), `remove_file` (`pub`, L42), `rename` (`pub`, L46), `copy` (`pub`, L50), `hard_link` (`pub`, L54), `try_exists` (`pub`, L58), `set_permissions` (`pub`, L63), `metadata` (`pub`, L70)
- **LoomMetadata**: `len` (`pub`, L78), `permissions` (`pub`, L82)
- **OpenOptions**: `new` (`pub`, L122), `read` (`pub`, L125), `write` (`pub`, L128), `create` (`pub`, L131), `create_new` (`pub`, L134), `append` (`pub`, L137), `mode` (`pub`, L140), `open` (`pub`, L143)
- **LoomFile**: `new` (`pub`, L155), `open` (`pub`, L161), `write_all` (`pub`, L164), `flush` (`pub`, L168), `sync_all` (`pub`, L171), `set_len` (`pub`, L174), `seek` (`pub`, L178), `metadata` (`pub`, L190)
- **tokio::io::AsyncRead for LoomFile**: `poll_read` (`priv`, L196)
- **tokio::io::AsyncWrite for LoomFile**: `poll_write` (`priv`, L213), `poll_flush` (`priv`, L221), `poll_shutdown` (`priv`, L227)
- **tokio::io::AsyncSeek for LoomFile**: `start_seek` (`priv`, L236), `poll_complete` (`priv`, L251)
- **KeyManager**: `try_new` (`pub`, L274), `derive_file_key` (`pub`, L277), `encrypt_auto_nonce` (`pub`, L280), `decrypt_auto_nonce` (`pub`, L283), `integrity_key` (`pub`, L286)
- **Default for WalConfig**: `default` (`priv`, L325)
- **WalConfig**: `with_legacy_fallback` (`pub`, L336)
- **Drop for Wal**: `drop` (`priv`, L360)
- **std::fmt::Debug for Wal**: `fmt` (`priv`, L373)
- **Wal**: `open` (`pub`, L382), `migrate_legacy_wal` (`pub`, L399), `open_for_legacy_migration` (`pub`, L443), `rekey_from_legacy` (`pub(crate)`, L471), `open_read_only` (`pub`, L518), `open_read_only_with_config` (`pub`, L533), `open_with_key_manager` (`pub`, L586), `open_with_config` (`pub`, L601), `close` (`pub`, L784), `size` (`pub`, L811), `path` (`pub`, L815), `is_sealed` (`pub`, L819), `is_poisoned` (`pub`, L823), `allow_legacy_fallback_for_test` (`pub`, L827), `legacy_key_used_for_test` (`pub`, L832), `was_legacy_rekeyed` (`pub`, L837), `truncate_uncommitted_tail` (`pub`, L857), `recover_from_poison` (`pub`, L879), `open_loom` (`pub`, L933)

#### `crates/contextra-store/src/wal/replay.rs` (17)

- **trait ReplayProgressSink**: `on_entry_replayed` (`priv`, L14)
- **ReplayProgressSink for NoopReplayProgressSink**: `on_entry_replayed` (`priv`, L19)
- **Wal**: `handle_wal_entry_parse_error` (`pub(crate)`, L23), `replay` (`pub`, L59), `replay_with_sink` (`pub`, L64), `replay_stream` (`pub`, L78), `replay_from` (`pub`, L94), `replay_with_size_and_version` (`pub(crate)`, L104), `replay_mmap_with_size_and_version` (`priv`, L112), `replay_mmap` (`pub`, L133), `replay_mmap_with_sink` (`pub`, L140), `parse_mmap_slice` (`pub(crate)`, L181), `parse_mmap_slice_with_sink` (`pub(crate)`, L190), `scan_entries_mmap` (`pub`, L213), `verify_entry_snapshot` (`priv`, L266), `scan_entries_from_slice` (`priv`, L328)
- ***(Modulfunktionen)***: `recover_from_bak_if_present` (`pub(crate)`, L725)

#### `crates/contextra-store/src/wal/tests/hmac_tests.rs` (1)

- ***(Modulfunktionen)***: `compute_v3_hmac_reference_independent` (`priv`, L12)

#### `crates/contextra-store/src/wal/tests/replay_tests.rs` (4)

- ***(Modulfunktionen)***: `create_5_entry_wal_unencrypted` (`priv`, L951), `read_unencrypted_raw_entries` (`priv`, L977), `read_encrypted_raw_chunks` (`priv`, L1000), `write_valid_wal_fuzz` (`priv`, L1284)

### contextra-sys

> Low-level unsafe system abstractions and FFI island for Contextra (Ring 0)

- **Pfad:** `crates/contextra-sys` | **Dateien:** 8 | **Produktiv-Funktionen:** 34 (`27` `pub`, `7` intern) | **Tests:** 15
- **Öffentliche Typen (4)**: `DurableFile`, `Error`, `Result`, `LockedRegions`

#### `crates/contextra-sys/src/acl_win32.rs` (6)

- ***(Modulfunktionen)***: `set_restrictive_file_acl` (`pub`, L13), `verify_file_acl_owner_only` (`pub`, L154), `verify_file_acl_owner_only` (`pub`, L315), `set_restrictive_file_acl` (`pub`, L321)
- **Drop for TokenGuard**: `drop` (`priv`, L43)
- **Drop for SecDescGuard**: `drop` (`priv`, L199)

#### `crates/contextra-sys/src/durability.rs` (12)

- ***(Modulfunktionen)***: `raw_os_sync_file` (`priv`, L65), `raw_os_sync_file` (`priv`, L135), `raw_os_sync_file` (`priv`, L155), `durable_flush_file` (`pub`, L163), `durable_flush_dir` (`pub`, L169), `write_all_durable` (`pub`, L194)
- **DurableFile**: `new` (`pub`, L227), `is_poisoned` (`pub`, L235), `get_ref` (`pub`, L240), `write_all` (`pub`, L249), `sync_all` (`pub`, L257), `sync_data` (`pub`, L271)

#### `crates/contextra-sys/src/error.rs` (1)

- **From<Error> for std::io::Error**: `from` (`priv`, L57)

#### `crates/contextra-sys/src/mlock.rs` (6)

- ***(Modulfunktionen)***: `mem_lock` (`pub`, L6), `mem_unlock` (`pub`, L31), `mem_lock` (`pub`, L46), `mem_unlock` (`pub`, L73), `mem_lock` (`pub`, L88), `mem_unlock` (`pub`, L103)

#### `crates/contextra-sys/src/mmap.rs` (1)

- ***(Modulfunktionen)***: `mmap_readonly` (`pub`, L9)

#### `crates/contextra-sys/src/posix.rs` (2)

- ***(Modulfunktionen)***: `reopen_and_dup2` (`pub`, L9), `reopen_and_dup2` (`pub`, L38)

#### `crates/contextra-sys/src/vault.rs` (6)

- **LockedRegions**: `new` (`pub`, L21), `lock_slice` (`pub`, L34), `unlock_all` (`pub`, L54), `len` (`pub`, L61), `is_empty` (`pub`, L66)
- **Drop for LockedRegions**: `drop` (`priv`, L72)

### contextra-testkit

> Deterministic test utilities, ManualClock, InMemoryStorageEngine, and FaultVfs for Contextra

- **Pfad:** `crates/contextra-testkit` | **Dateien:** 5 | **Produktiv-Funktionen:** 51 (`30` `pub`, `21` intern) | **Tests:** 8
- **Öffentliche Typen (6)**: `FaultConfig`, `FaultVfs`, `InMemoryStorageEngine`, `ManualClock`, `RefOp`, `ReferenceModel`

#### `crates/contextra-testkit/src/fault_vfs.rs` (12)

- **FaultVfs**: `new` (`pub`, L40), `set_config` (`pub`, L45), `trigger_crash` (`pub`, L51), `reset_counters` (`pub`, L56), `check_write_fault` (`priv`, L64), `check_read_fault` (`priv`, L85), `write_file` (`pub`, L106), `read_file` (`pub`, L113), `sync` (`pub`, L123), `write_count` (`pub`, L141), `read_count` (`pub`, L146), `sync_count` (`pub`, L151)

#### `crates/contextra-testkit/src/in_memory_store.rs` (17)

- **Default for InMemoryStorageEngine**: `default` (`priv`, L34)
- **InMemoryStorageEngine**: `new` (`pub`, L46)
- **StorageEngine for InMemoryStorageEngine**: `get` (`priv`, L52), `get_at_seq` (`priv`, L59), `put` (`priv`, L71), `delete` (`priv`, L82), `commit` (`priv`, L93), `rollback` (`priv`, L119), `rollback_to_tx` (`priv`, L127), `flush` (`priv`, L135), `stats` (`priv`, L139), `last_seq_no` (`priv`, L151), `last_tx_id` (`priv`, L155), `pin_checkpoint` (`priv`, L159), `unpin_checkpoint` (`priv`, L163), `scan_prefix` (`priv`, L167), `scan` (`priv`, L185)

#### `crates/contextra-testkit/src/manual_clock.rs` (9)

- **Default for ManualClock**: `default` (`priv`, L19)
- **ManualClock**: `new` (`pub`, L28), `advance` (`pub`, L35), `set_nanos` (`pub`, L42), `now_nanos` (`pub`, L47), `now_secs_f64` (`pub`, L52), `now_system_time` (`pub`, L57)
- **Clock for ManualClock**: `now_unix_nanos` (`priv`, L63), `monotonic_nanos` (`priv`, L67)

#### `crates/contextra-testkit/src/reference_model.rs` (13)

- **ReferenceModel**: `new` (`pub`, L29), `put` (`pub`, L38), `delete` (`pub`, L46), `rollback` (`pub`, L52), `commit` (`pub`, L59), `commit_batch` (`pub`, L64), `snapshot_seq` (`pub`, L94), `get_at` (`pub`, L101), `scan_prefix_at` (`pub`, L113), `scan_range_at` (`pub`, L131), `get_latest` (`pub`, L157), `scan_prefix_latest` (`pub`, L162), `snapshot_map_at` (`pub`, L167)

### contextra-text

> Contextra — Text processing and BM25 search for Hybrid Search

- **Pfad:** `crates/contextra-text` | **Dateien:** 24 | **Produktiv-Funktionen:** 191 (`74` `pub`, `117` intern) | **Tests:** 139
- **Öffentliche Typen (33)**: `BM25`, `FieldId`, `FieldWeight`, `BM25F`, `LegalDomainVocabulary`, `MedicalDomainVocabulary`, `DomainVocabulary`, `ParsedVocabulary`, `InvertedIndex`, `BM25MorphIndex`, `MockStorage`, `Language`, `TextIndexMetadata`, `StagedStatsChange`, `Bm25Scorer`, `MorphologicalTokenizer`, `GermanCompoundSplitter`, `TokenReductionMetrics`, `PassthroughTokenizer`, `TrieNode`, `Trie`, `DocIdRaw`, `DocIdRaw`, `PostingBlockInfo`, `Posting`, `PostingList`, `ResidentPostingIndex`, `Bm25CandidateStream`, `Tokenizer`, `DefaultTokenizer`, `GermanMorphTokenizer`, `BoundedTopK`, `WandSearchResult`

#### `crates/contextra-text/src/bm25.rs` (9)

- **BM25**: `new` (`pub`, L34), `score_term` (`pub`, L47)
- **Default for BM25**: `default` (`priv`, L54)
- ***(Modulfunktionen)***: `score_term` (`pub`, L70), `score_term_with_params` (`pub`, L75), `score_term_bm25f` (`pub`, L200)
- **FieldWeight**: `new` (`pub`, L127)
- **BM25F**: `new` (`pub`, L162), `score_term` (`pub`, L178)

#### `crates/contextra-text/src/domain/legal_de.rs` (6)

- ***(Modulfunktionen)***: `get_parsed_legal` (`priv`, L13), `get_legal_compound_stems` (`pub`, L35), `get_legal_protected_terms` (`pub`, L39)
- **DomainVocabulary for LegalDomainVocabulary**: `compound_stems` (`priv`, L21), `protected_terms` (`priv`, L25), `domain_id` (`priv`, L29)

#### `crates/contextra-text/src/domain/medical_de.rs` (6)

- ***(Modulfunktionen)***: `get_parsed_medical` (`priv`, L13), `get_medical_compound_stems` (`pub`, L35), `get_medical_protected_terms` (`pub`, L39)
- **DomainVocabulary for MedicalDomainVocabulary**: `compound_stems` (`priv`, L21), `protected_terms` (`priv`, L25), `domain_id` (`priv`, L29)

#### `crates/contextra-text/src/domain/mod.rs` (4)

- **trait DomainVocabulary**: `compound_stems` (`priv`, L19), `protected_terms` (`priv`, L22), `domain_id` (`priv`, L24)
- ***(Modulfunktionen)***: `parse_domain_resource` (`pub(crate)`, L32)

#### `crates/contextra-text/src/inverted/index_struct.rs` (30)

- **Clone for InvertedIndex<S>**: `clone` (`priv`, L31)
- **InvertedIndex<S>**: `new_with_language` (`pub`, L61), `new` (`pub`, L88), `with_tokenizer` (`pub`, L93), `load_stats` (`pub`, L99), `key` (`pub(crate)`, L112), `key_with_id` (`pub(crate)`, L119), `key_with_term_doc` (`pub(crate)`, L129), `key_term_prefix` (`pub(crate)`, L141), `key_batch_posting_list` (`pub(crate)`, L150), `ensure_term_loaded` (`pub(crate)`, L159), `key_tombstone` (`pub(crate)`, L227), `upsert_document` (`pub`, L240), `resolve_tombstones` (`pub`, L330), `delete_document` (`pub`, L407), `stage_stats_change` (`priv`, L461), `commit_stats` (`pub(crate)`, L475), `rollback_stats` (`pub(crate)`, L527), `search_bm25` (`pub`, L536), `search_bm25_at` (`pub`, L547)
- **TextIndex for InvertedIndex<S>**: `search` (`priv`, L600), `search_at` (`priv`, L612), `insert` (`priv`, L620), `delete` (`priv`, L624), `commit` (`priv`, L628), `rollback` (`priv`, L633), `rollback_to_tx` (`priv`, L638), `last_tx_id` (`priv`, L642), `len` (`priv`, L646), `stats` (`priv`, L650)

#### `crates/contextra-text/src/inverted/morph_index.rs` (12)

- **BM25MorphIndex<S>**: `new` (`pub`, L14), `tokenizer` (`pub`, L29)
- **TextIndex for BM25MorphIndex<S>**: `search` (`priv`, L35), `search_at` (`priv`, L46), `insert` (`priv`, L50), `delete` (`priv`, L54), `commit` (`priv`, L58), `rollback` (`priv`, L62), `rollback_to_tx` (`priv`, L66), `last_tx_id` (`priv`, L70), `len` (`priv`, L74), `stats` (`priv`, L78)

#### `crates/contextra-text/src/inverted/tests/mock.rs` (17)

- **MockStorage**: `new` (`pub(crate)`, L17)
- **StorageEngine for MockStorage**: `get` (`priv`, L27), `put` (`priv`, L30), `delete` (`priv`, L48), `commit` (`priv`, L71), `rollback` (`priv`, L77), `rollback_to_tx` (`priv`, L92), `get_at_seq` (`priv`, L95), `last_seq_no` (`priv`, L117), `last_tx_id` (`priv`, L126), `flush` (`priv`, L129), `stats` (`priv`, L132), `pin_checkpoint` (`priv`, L141), `unpin_checkpoint` (`priv`, L144), `scan` (`priv`, L147), `scan_prefix` (`priv`, L155), `scan_prefix_at` (`priv`, L161)

#### `crates/contextra-text/src/inverted/tests/search_tests.rs` (17)

- **CountingStorage**: `new` (`priv`, L570)
- **StorageEngine for CountingStorage**: `get` (`priv`, L579), `put` (`priv`, L582), `delete` (`priv`, L585), `commit` (`priv`, L588), `rollback` (`priv`, L591), `rollback_to_tx` (`priv`, L594), `get_at_seq` (`priv`, L597), `last_seq_no` (`priv`, L605), `last_tx_id` (`priv`, L608), `flush` (`priv`, L611), `stats` (`priv`, L614), `pin_checkpoint` (`priv`, L617), `unpin_checkpoint` (`priv`, L620), `scan` (`priv`, L623), `scan_prefix` (`priv`, L631), `scan_prefix_at` (`priv`, L637)

#### `crates/contextra-text/src/inverted/types.rs` (1)

- **Language**: `from_iso` (`pub`, L26)

#### `crates/contextra-text/src/lib.rs` (11)

- **Bm25Scorer<S>**: `new` (`pub`, L47)
- **TextIndex for Bm25Scorer<S>**: `search` (`priv`, L55), `search_at` (`priv`, L62), `insert` (`priv`, L66), `delete` (`priv`, L70), `commit` (`priv`, L74), `rollback` (`priv`, L78), `rollback_to_tx` (`priv`, L82), `last_tx_id` (`priv`, L86), `len` (`priv`, L90), `stats` (`priv`, L94)

#### `crates/contextra-text/src/morphology/compound_splitter.rs` (13)

- **trait MorphologicalTokenizer**: `decompose` (`priv`, L25), `language` (`priv`, L28)
- **GermanCompoundSplitter**: `new` (`pub`, L70), `with_min_length` (`pub`, L75), `with_dictionary` (`pub`, L90), `with_domains` (`pub`, L100), `new_with_all_domains` (`pub`, L110), `extend_vocabulary` (`pub`, L117), `min_component_len` (`pub`, L127), `is_valid_component` (`priv`, L132)
- **Default for GermanCompoundSplitter**: `default` (`priv`, L168)
- **MorphologicalTokenizer for GermanCompoundSplitter**: `decompose` (`priv`, L181), `language` (`priv`, L281)

#### `crates/contextra-text/src/morphology/metrics.rs` (1)

- **TokenReductionMetrics**: `expansion_ratio` (`pub`, L16)

#### `crates/contextra-text/src/morphology/passthrough.rs` (3)

- **PassthroughTokenizer**: `new` (`pub`, L10)
- **MorphologicalTokenizer for PassthroughTokenizer**: `decompose` (`priv`, L16), `language` (`priv`, L20)

#### `crates/contextra-text/src/morphology/stopwords.rs` (3)

- ***(Modulfunktionen)***: `get_german_stopwords` (`pub`, L9), `is_german_stopword` (`pub`, L27), `normalize_umlauts` (`pub`, L44)

#### `crates/contextra-text/src/morphology/trie.rs` (4)

- **Trie**: `new` (`pub`, L18), `insert` (`pub`, L23), `contains` (`pub`, L32), `starts_with` (`pub`, L45)

#### `crates/contextra-text/src/posting_list.rs` (22)

- **Posting**: `new` (`pub`, L52), `doc_id` (`pub`, L61)
- ***(Modulfunktionen)***: `compute_blocks` (`priv`, L67), `encode_varint` (`priv`, L257), `decode_varint` (`priv`, L265)
- **PostingList**: `new` (`pub`, L105), `empty` (`pub`, L113), `as_slice` (`pub`, L122), `blocks` (`pub`, L128), `len` (`pub`, L134), `is_empty` (`pub`, L140), `upsert` (`pub`, L145), `remove` (`pub`, L163), `encode_compact` (`pub`, L176), `decode_compact` (`pub`, L204)
- **ResidentPostingIndex**: `new` (`pub`, L293), `get` (`pub`, L300), `insert_list` (`pub`, L305), `upsert_posting` (`pub`, L310), `remove_posting_from_terms` (`pub`, L321), `remove_terms` (`pub`, L326), `clear` (`pub`, L341)

#### `crates/contextra-text/src/stream.rs` (8)

- **Bm25CandidateStream<**: `new` (`pub`, L41), `with_batch_size` (`pub`, L60), `next_batch` (`pub`, L67), `next_doc` (`pub`, L85), `yielded` (`pub`, L99), `is_exhausted` (`pub`, L104), `max_depth` (`pub`, L109), `refill` (`priv`, L113)

#### `crates/contextra-text/src/tokenizer.rs` (11)

- ***(Modulfunktionen)***: `get_protected_regex` (`priv`, L21), `get_shared_splitter` (`priv`, L35), `clean_protected_match` (`priv`, L41), `segment_text` (`priv`, L59), `get_stopwords` (`priv`, L94), `tokenize` (`pub`, L221)
- **trait Tokenizer**: `tokenize` (`priv`, L112)
- **Tokenizer for DefaultTokenizer**: `tokenize` (`priv`, L119)
- **GermanMorphTokenizer**: `new` (`pub`, L149)
- **Default for GermanMorphTokenizer**: `default` (`priv`, L157)
- **Tokenizer for GermanMorphTokenizer**: `tokenize` (`priv`, L163)

#### `crates/contextra-text/src/wand.rs` (13)

- **PartialEq for Bm25Candidate**: `eq` (`priv`, L18)
- **Ord for Bm25Candidate**: `cmp` (`priv`, L26)
- **PartialOrd for Bm25Candidate**: `partial_cmp` (`priv`, L37)
- **BoundedTopK**: `new` (`pub`, L48), `push` (`pub`, L56), `min_threshold` (`pub`, L71), `into_sorted_vec` (`pub`, L79)
- **TermCursor**: `new` (`priv`, L101), `current_posting` (`priv`, L132), `current_doc_id` (`priv`, L137), `block_max_score_at_doc_id` (`priv`, L142), `advance_to` (`priv`, L180)
- ***(Modulfunktionen)***: `block_max_wand_search` (`pub`, L220)

### contextra-types

> Canonical domain types, IDs, budgets, filters, and error types for Contextra

- **Pfad:** `crates/contextra-types` | **Dateien:** 23 | **Produktiv-Funktionen:** 235 (`165` `pub`, `70` intern) | **Tests:** 125
- **Öffentliche Typen (60)**: `HnswDeletionError`, `StepId`, `CacheDirective`, `Result`, `ContextraError`, `ContextraErrorDto`, `ModelFingerprint`, `RetrievalStrategy`, `DocIdWidth`, `ManifestSchemaVersion`, `TenantScoped`, `TenantScopeViolation`, `TombstoneSemanticsCheck`, `SeqBitTombstone`, `BudgetStrategy`, `TokenBudget`, `Reservation`, `ResourceBudget`, `ResourceTracker`, `DistanceMetric`, `Embedding`, `ScoredDocument`, `RerankResult`, `TenantId`, `CollectionId`, `DocId`, `DocId`, `EntityId`, `TxId`, `WorkflowState`, `LinkRelation`, `MemoryLink`, `Entity`, `Edge`, `MemoryType`, `TlHfdParams`, `PprAlgorithm`, `PprConfig`, `ConfigFingerprint`, `AutoExtractionMode` *(+20 weitere)*

#### `crates/contextra-types/src/error.rs` (23)

- **StepId**: `new` (`pub`, L46), `inner` (`pub`, L51)
- **std::fmt::Display for StepId**: `fmt` (`priv`, L57)
- **From<u64> for StepId**: `from` (`priv`, L63)
- **Default for CacheDirective**: `default` (`priv`, L88)
- **ContextraError**: `limit_exceeded` (`pub`, L361), `model_load` (`pub`, L369), `snapshot_unsupported_for_signal` (`pub`, L377), `orphaned_vector_reference` (`pub`, L382), `capability_unsupported` (`pub`, L393), `invalid_input` (`pub`, L403), `wal_corruption` (`pub`, L408), `checksum_mismatch` (`pub`, L416), `kv_quantization` (`pub`, L424), `from_kind_and_message` (`pub`, L429), `durability_config` (`pub`, L449), `check_durability` (`pub`, L454), `check_vector_reference` (`pub`, L463), `pin_budget_exceeded` (`pub`, L476), `cross_device_link` (`pub`, L481), `wal_truncation_detected` (`pub`, L492), `is_occ_conflict` (`pub`, L500)
- **From<std::array::TryFromSliceError> for ContextraError**: `from` (`priv`, L506)

#### `crates/contextra-types/src/error_dto.rs` (7)

- **ContextraErrorDto**: `new` (`pub`, L30), `with_details` (`pub`, L39)
- **std::fmt::Display for ContextraErrorDto**: `fmt` (`priv`, L53)
- **From<ContextraError> for ContextraErrorDto**: `from` (`priv`, L61)
- **From<String> for ContextraErrorDto**: `from` (`priv`, L67)
- **From<**: `from` (`priv`, L77), `from` (`priv`, L87)

#### `crates/contextra-types/src/model_fingerprint.rs` (1)

- **ModelFingerprint**: `new` (`pub`, L23)

#### `crates/contextra-types/src/retrieval_strategy.rs` (1)

- **fmt::Display for RetrievalStrategy**: `fmt` (`priv`, L31)

#### `crates/contextra-types/src/schema.rs` (10)

- **DocIdWidth**: `current` (`pub`, L18), `bytes_len` (`pub`, L27), `current_bytes_len` (`pub`, L35)
- **ManifestSchemaVersion**: `doc_id_width` (`pub`, L55), `as_u8` (`pub`, L63), `from_u8` (`pub`, L68), `from_u8_checked` (`pub`, L79), `doc_id_bytes_len` (`pub`, L89), `is_compatible_with_current_build` (`pub`, L94), `validate_build_compatibility` (`pub`, L102)

#### `crates/contextra-types/src/tenant_scope.rs` (4)

- **TenantScoped<T>**: `new` (`pub`, L19), `tenant_id` (`pub`, L25), `into_inner_checked` (`pub`, L31), `map` (`pub`, L43)

#### `crates/contextra-types/src/tombstone.rs` (4)

- **trait TombstoneSemanticsCheck**: `is_tombstone` (`priv`, L27), `make_tombstone` (`priv`, L30)
- **TombstoneSemanticsCheck for SeqBitTombstone**: `is_tombstone` (`priv`, L43), `make_tombstone` (`priv`, L51)

#### `crates/contextra-types/src/types/budget.rs` (26)

- **Clone for TokenBudget**: `clone` (`priv`, L49)
- **PartialEq for TokenBudget**: `eq` (`priv`, L60)
- **Serialize for TokenBudget**: `serialize` (`priv`, L69)
- **Deserialize<**: `deserialize` (`priv`, L92)
- **TokenBudget**: `new` (`pub`, L124), `for_model` (`pub`, L139), `with_reserved` (`pub`, L165), `with_strategy` (`pub`, L171), `effective_limit` (`pub`, L179), `available` (`pub`, L188), `consume` (`pub`, L195), `reserve` (`pub`, L211), `try_reserve` (`pub`, L223), `refund` (`pub`, L236), `consumed` (`pub`, L251)
- **Reservation<**: `settle` (`pub`, L262)
- **Drop for Reservation<**: `drop` (`priv`, L270)
- **Default for TokenBudget**: `default` (`priv`, L289)
- **Default for ResourceBudget**: `default` (`priv`, L302)
- **Default for ResourceTracker**: `default` (`priv`, L319)
- **ResourceTracker**: `new` (`pub`, L326), `consume_memory` (`pub`, L338), `release_memory` (`pub`, L367), `memory_used` (`pub`, L391), `budget` (`pub`, L396), `has_memory_capacity` (`pub`, L403)

#### `crates/contextra-types/src/types/domain/document.rs` (8)

- **DistanceMetric**: `compute` (`pub`, L20), `compute_u8` (`pub`, L103)
- **Embedding**: `new` (`pub`, L161), `dim` (`pub`, L167), `as_slice` (`pub`, L173), `l2_norm` (`pub`, L178), `normalize` (`pub`, L187)
- **ScoredDocument**: `new` (`pub`, L209)

#### `crates/contextra-types/src/types/domain/ids.rs` (45)

- **TenantId**: `new` (`pub`, L30), `try_new` (`pub`, L35), `inner` (`pub`, L47), `as_u64` (`pub`, L53), `is_system` (`pub`, L59)
- **Default for TenantId**: `default` (`priv`, L65)
- **TryFrom<u64> for TenantId**: `try_from` (`priv`, L73)
- **std::fmt::Display for TenantId**: `fmt` (`priv`, L79)
- **CollectionId**: `new` (`pub`, L95), `try_new` (`pub`, L100), `inner` (`pub`, L112), `as_u64` (`pub`, L119)
- **From<u64> for CollectionId**: `from` (`priv`, L125)
- **std::fmt::Display for CollectionId**: `fmt` (`priv`, L131)
- **DocId**: `new` (`pub`, L166), `inner` (`pub`, L172), `as_u64` (`pub`, L179), `from_key` (`pub`, L195), `new` (`pub`, L214), `inner` (`pub`, L220), `as_u64` (`pub`, L227), `from_key` (`pub`, L237)
- **From<u64> for DocId**: `from` (`priv`, L248)
- **From<u128> for DocId**: `from` (`priv`, L255)
- **std::fmt::Display for DocId**: `fmt` (`priv`, L261)
- **EntityId**: `new` (`pub`, L274), `inner` (`pub`, L280), `as_u64` (`pub`, L287), `as_bytes` (`pub`, L292), `from_doc_id` (`pub`, L298), `from_key` (`pub`, L311)
- **From<u64> for EntityId**: `from` (`priv`, L317)
- ***(Modulfunktionen)***: `hash_key_u64` (`priv`, L323), `hash_key_u128` (`priv`, L332)
- **From<**: `from` (`priv`, L342)
- **From<String> for EntityId**: `from` (`priv`, L352)
- **std::fmt::Display for EntityId**: `fmt` (`priv`, L358)
- **TxId**: `new` (`pub`, L400), `inner` (`pub`, L406), `as_u64` (`pub`, L413), `internal` (`pub`, L419), `internal_with_offset` (`pub`, L425), `try_from_internal_offset` (`pub`, L438), `is_valid_origin` (`pub`, L456)
- **std::fmt::Display for TxId**: `fmt` (`priv`, L468)

#### `crates/contextra-types/src/types/domain/misc.rs` (32)

- ***(Modulfunktionen)***: `serialize` (`pub`, L9), `deserialize` (`pub`, L23), `default_tl_hfd_sigma` (`priv`, L368), `default_tl_hfd_delta` (`priv`, L371), `default_tl_hfd_gamma` (`priv`, L374), `default_tl_hfd_max_iterations` (`priv`, L377), `default_tl_hfd_max_top_k_expansion` (`priv`, L380), `default_tl_hfd_max_hyperedge_sort_size` (`priv`, L383), `default_warn_on_non_convergence` (`priv`, L441)
- **Entity**: `new` (`pub`, L113), `try_new` (`pub`, L126)
- **Edge**: `new` (`pub`, L182), `with_source_doc_id` (`pub`, L197), `try_new` (`pub`, L206), `with_weight` (`pub`, L227), `with_tx_validity` (`pub`, L233), `with_business_validity` (`pub`, L238), `with_validity` (`pub`, L245)
- **MemoryType**: `default_decay` (`pub`, L295), `default_ttl_tx` (`pub`, L312), `as_str` (`pub`, L320), `as_metadata_key` (`pub`, L325)
- **std::fmt::Display for MemoryType**: `fmt` (`priv`, L336)
- **Default for TlHfdParams**: `default` (`priv`, L388)
- **Default for PprConfig**: `default` (`priv`, L446)
- **ConfigFingerprint**: `new` (`pub`, L486), `with_threshold` (`pub`, L507), `threshold` (`pub`, L514), `temperature` (`pub`, L520)
- **Default for ConfigFingerprint**: `default` (`priv`, L526)
- **AutoExtractionMode**: `is_enabled` (`pub`, L544)
- **Default for AutoExtractionMode**: `default` (`priv`, L550)

#### `crates/contextra-types/src/types/domain/tests/misc_tests.rs` (6)

- ***(Modulfunktionen)***: `prop_docid_serialization` (`priv`, L167), `doc_id_from_key_deterministic` (`priv`, L174), `doc_id_from_key_never_panics` (`priv`, L180), `doc_id_empty_key_is_err` (`priv`, L184), `prop_txid_serialization` (`priv`, L188), `prop_entityid_serialization` (`priv`, L195)

#### `crates/contextra-types/src/types/filter.rs` (2)

- ***(Modulfunktionen)***: `compare_values` (`priv`, L88)
- **FilterExpr**: `evaluate` (`pub`, L106)

#### `crates/contextra-types/src/types/importance.rs` (7)

- **ImportanceScore**: `new` (`pub`, L27), `value` (`pub`, L36)
- **Default for ImportanceScore**: `default` (`priv`, L42)
- **DecayFunction**: `decay_factor` (`pub`, L84), `decay_factor_wallclock` (`pub`, L137)
- **MemoryImportance**: `new` (`pub`, L178), `effective_score` (`pub`, L190)

#### `crates/contextra-types/src/types/math.rs` (7)

- **SaturatingU8**: `new` (`pub`, L13), `get` (`pub`, L19)
- **From<u8> for SaturatingU8**: `from` (`priv`, L26)
- **From<SaturatingU8> for u8**: `from` (`priv`, L33)
- **From<usize> for SaturatingU8**: `from` (`priv`, L41)
- **From<u32> for SaturatingU8**: `from` (`priv`, L50)
- **From<u64> for SaturatingU8**: `from` (`priv`, L58)

#### `crates/contextra-types/src/types/saos.rs` (49)

- **Default for SignalFusionStrategies**: `default` (`priv`, L40)
- **From<FusionStrategy> for SignalFusionStrategies**: `from` (`priv`, L50)
- **SignalFusionStrategies**: `uniform` (`pub`, L61), `strategy_for_signal` (`pub`, L66)
- **Deserialize<**: `deserialize` (`priv`, L77)
- **Default for GraphTraversalStrategy**: `default` (`priv`, L130)
- **Default for FusionWeights**: `default` (`priv`, L146)
- **FusionWeights**: `new` (`pub`, L158), `vector` (`pub`, L191), `text` (`pub`, L196), `graph` (`pub`, L201), `metadata` (`pub`, L207)
- **ContextChunk**: `combined_text_owned` (`pub`, L240), `combined_token_count` (`pub`, L255), `has_context_prefix` (`pub`, L263)
- **std::fmt::Display for ContextWindow**: `fmt` (`priv`, L282)
- **std::fmt::Display for Signal**: `fmt` (`priv`, L305)
- **QueryRewriteOutput**: `new` (`pub`, L351), `text_only` (`pub`, L364), `is_empty` (`pub`, L373), `into_query_builder` (`pub`, L383)
- **HybridQuery**: `builder` (`pub`, L462), `text_search` (`pub`, L467), `vector_search` (`pub`, L472), `graph_search` (`pub`, L477), `with_fusion` (`pub`, L488), `filtered` (`pub`, L499), `community_boosted` (`pub`, L504), `memory_filtered` (`pub`, L511), `with_provenance_and_superseded` (`pub`, L518), `rerank_pool_config` (`pub`, L529), `with_failure_policy` (`pub`, L537)
- **HybridQueryBuilder**: `new` (`pub`, L569), `with_text_query` (`pub`, L574), `with_vector_query` (`pub`, L580), `with_graph_start_node` (`pub`, L586), `with_graph_strategy` (`pub`, L592), `with_fusion_weights` (`pub`, L598), `with_fusion_strategy` (`pub`, L604), `with_filter` (`pub`, L610), `with_same_community_as` (`pub`, L616), `with_memory_type_filter` (`pub`, L622), `with_include_superseded` (`pub`, L628), `with_include_provenance` (`pub`, L634), `with_rerank_pool_multiplier` (`pub`, L640), `with_rerank_pool_max` (`pub`, L646), `with_on_signal_failure` (`pub`, L652), `with_k` (`pub`, L658), `build` (`pub`, L667)

#### `crates/contextra-types/src/types/saturating.rs` (3)

- **SaturatingU16**: `new` (`pub`, L15), `get` (`pub`, L21)
- **From<usize> for SaturatingU16**: `from` (`priv`, L28)

### contextra-vector

> HNSW vector index with SIMD distance computation for Contextra

- **Pfad:** `crates/contextra-vector` | **Dateien:** 41 | **Produktiv-Funktionen:** 297 (`186` `pub`, `111` intern) | **Tests:** 108
- **Öffentliche Typen (49)**: `FilteredIndex`, `AcornError`, `NaiveReferenceIndex`, `VectorCandidateStream`, `TaskHandle`, `ComputePool`, `TombstoneSet`, `TombstoneSet`, `DiskAnnFallbackPolicy`, `DiskAnnConfig`, `CachedNode`, `VectorData`, `DiskAnnHeader`, `DiskAnnFooter`, `DiskAnnIndex`, `DiskAnnIndexInner`, `SearchCandidate`, `AdaptiveEfPolicy`, `AdaptiveEfStats`, `AdaptiveEfStateMachine`, `BacklinkKey`, `BacklinkTable`, `HnswArena`, `PreparedInsert`, `NeighborBacklink`, `BatchContext`, `SearchContext`, `HnswConfig`, `HnswConfigBuilder`, `DeletionStats`, `GhostFreeVectorIndex`, `Sq8Bias`, `VectorData`, `HnswNode`, `Candidate`, `RebuildGuard`, `SnapshotPinGuard`, `HnswIndex`, `HnswHotCore`, `HnswColdCore` *(+9 weitere)*

#### `crates/contextra-vector/src/acorn/gamma_augmentation.rs` (1)

- ***(Modulfunktionen)***: `compute_gamma_edge_budget` (`pub`, L29)

#### `crates/contextra-vector/src/acorn/mod.rs` (2)

- **trait FilteredIndex**: `search_knn_acorn` (`priv`, L19)
- **std::fmt::Display for AcornError**: `fmt` (`priv`, L53)

#### `crates/contextra-vector/src/acorn/naive_reference.rs` (4)

- **NaiveReferenceIndex**: `new` (`pub`, L27), `from_vectors` (`pub`, L38), `insert` (`pub`, L46)
- **FilteredIndex for NaiveReferenceIndex**: `search_knn_acorn` (`priv`, L54)

#### `crates/contextra-vector/src/candidate_stream.rs` (9)

- **VectorCandidateStream<**: `new` (`pub`, L53), `with_batch_size` (`pub`, L68), `next_batch` (`pub`, L75), `next_doc` (`pub`, L93), `yielded` (`pub`, L107), `fetched_depth` (`pub`, L112), `is_exhausted` (`pub`, L117), `max_depth` (`pub`, L122), `refill` (`priv`, L126)

#### `crates/contextra-vector/src/compute_pool.rs` (8)

- **TaskHandle<T>**: `join` (`pub`, L20)
- **ComputePool**: `new` (`pub`, L41), `max_workers` (`pub`, L99), `active_jobs` (`pub`, L104), `execute` (`pub`, L109), `spawn` (`pub`, L120)
- **Default for ComputePool**: `default` (`priv`, L136)
- **std::fmt::Debug for ComputePool**: `fmt` (`priv`, L146)

#### `crates/contextra-vector/src/diskann/build.rs` (18)

- **TombstoneSet**: `new` (`pub(crate)`, L25), `insert` (`pub(crate)`, L31), `contains` (`pub(crate)`, L35), `len` (`pub(crate)`, L39), `is_empty` (`pub(crate)`, L44), `clear` (`pub(crate)`, L48)
- **DiskAnnIndex**: `prune_in_memory` (`pub(crate)`, L56), `append_to_tombstone_wal` (`pub(crate)`, L107), `read_tombstone_wal` (`pub(crate)`, L140), `append_to_pending_wal` (`pub(crate)`, L239), `read_pending_wal` (`pub(crate)`, L293), `recover_pending_delta_sync` (`pub`, L411), `prune_streaming` (`pub(crate)`, L441), `write_incremental_to_file_sync` (`pub(crate)`, L499), `build_to_path_sync` (`pub`, L605), `build` (`pub`, L687), `build_sync` (`pub`, L691), `verify_graph_integrity_debug` (`pub(crate)`, L720)

#### `crates/contextra-vector/src/diskann/config.rs` (1)

- **Default for DiskAnnConfig**: `default` (`priv`, L49)

#### `crates/contextra-vector/src/diskann/filtered.rs` (1)

- **DiskAnnIndex**: `search_filtered_internal` (`pub(crate)`, L21)

#### `crates/contextra-vector/src/diskann/format.rs` (5)

- ***(Modulfunktionen)***: `compute_adaptive_flush_threshold` (`pub(crate)`, L27)
- **DiskAnnHeader**: `to_bytes` (`pub(crate)`, L51), `try_from_bytes` (`pub(crate)`, L67)
- **DiskAnnFooter**: `to_bytes` (`pub(crate)`, L164), `try_from_bytes` (`pub(crate)`, L172)

#### `crates/contextra-vector/src/diskann/persistence.rs` (10)

- **DiskAnnIndex**: `trigger_background_persist_delta` (`pub`, L18)
- **Drop for FlushGuard**: `drop` (`priv`, L24)
- ***(Modulfunktionen)***: `persist_delta` (`pub`, L36), `persist_delta_sync` (`pub`, L40), `load_all_vectors_from_mmap` (`pub(crate)`, L98), `write_to_path_sync` (`pub(crate)`, L129), `init_hnsw_fallback` (`pub(crate)`, L232), `load` (`pub`, L245), `load_sync` (`pub`, L249), `load_node` (`pub(crate)`, L466)

#### `crates/contextra-vector/src/diskann/predicate_augmented.rs` (1)

- **DiskAnnIndex**: `search_predicate_augmented` (`pub`, L25)

#### `crates/contextra-vector/src/diskann/search.rs` (7)

- **DiskAnnIndex**: `get_dist_to_query` (`pub(crate)`, L16), `search_in_memory` (`pub(crate)`, L32), `get_dist_mixed` (`pub(crate)`, L111), `get_vec_mixed` (`pub(crate)`, L130), `search_streaming` (`pub(crate)`, L153), `search_internal` (`pub`, L225), `search_blocking` (`pub(crate)`, L289)

#### `crates/contextra-vector/src/diskann/types.rs` (7)

- **Clone for DiskAnnIndex**: `clone` (`priv`, L51)
- **PartialEq for SearchCandidate**: `eq` (`priv`, L169)
- **PartialOrd for SearchCandidate**: `partial_cmp` (`priv`, L175)
- **Ord for SearchCandidate**: `cmp` (`priv`, L180)
- **DiskAnnIndex**: `try_new` (`pub`, L186), `len_sync` (`pub`, L281), `check_quantizer_drift` (`pub(crate)`, L285)

#### `crates/contextra-vector/src/diskann/vector_index_impl.rs` (11)

- **VectorIndex for DiskAnnIndex**: `insert` (`priv`, L16), `search` (`priv`, L50), `search_filtered` (`priv`, L54), `delete` (`priv`, L64), `commit` (`priv`, L102), `rollback` (`priv`, L110), `rollback_to_tx` (`priv`, L118), `all_doc_ids` (`priv`, L127), `last_tx_id` (`priv`, L148), `len` (`priv`, L152), `stats` (`priv`, L160)

#### `crates/contextra-vector/src/hnsw/acorn_filtered.rs` (3)

- **PartialOrd for CandidateWithDoc**: `partial_cmp` (`priv`, L26)
- **Ord for CandidateWithDoc**: `cmp` (`priv`, L32)
- **FilteredIndex for HnswIndex**: `search_knn_acorn` (`priv`, L42)

#### `crates/contextra-vector/src/hnsw/adaptive_ef.rs` (9)

- **AdaptiveEfPolicy**: `new` (`pub`, L25), `validate` (`pub`, L67)
- **Default for AdaptiveEfPolicy**: `default` (`priv`, L99)
- **AdaptiveEfStateMachine**: `new` (`pub`, L136), `current_ef` (`pub`, L151), `round_count` (`pub`, L156), `is_terminated` (`pub`, L161), `stats` (`pub`, L166), `step` (`pub`, L178)

#### `crates/contextra-vector/src/hnsw/arena.rs` (13)

- **BacklinkTable**: `new` (`pub`, L28), `get` (`pub`, L36), `insert` (`pub`, L42), `clear` (`pub`, L47)
- **Default for HnswArena**: `default` (`priv`, L65)
- **HnswArena**: `new` (`pub`, L72), `layer_offset` (`pub`, L86), `align_capacity` (`pub`, L96), `allocate_node` (`pub`, L102), `get_ram_node_connections` (`pub`, L183), `update_backlink` (`pub`, L220), `try_relink_pruned_neighbors` (`pub`, L260), `free_node` (`pub`, L297)

#### `crates/contextra-vector/src/hnsw/batch.rs` (2)

- **BatchContext**: `new` (`pub`, L38)
- ***(Modulfunktionen)***: `get_neighbor_conns_in_batch` (`pub(super)`, L48)

#### `crates/contextra-vector/src/hnsw/config.rs` (18)

- **Default for HnswConfig**: `default` (`priv`, L41)
- ***(Modulfunktionen)***: `validate_vector` (`pub(super)`, L64)
- **HnswConfig**: `validate` (`pub`, L70)
- **HnswConfigBuilder**: `new` (`pub`, L114), `max_elements` (`pub`, L124), `m` (`pub`, L130), `ef_construction` (`pub`, L136), `ef_search` (`pub`, L142), `distance_metric` (`pub`, L148), `quantize` (`pub`, L154), `quantizer_recalibration_sample_size` (`pub`, L160), `quantizer_drift_threshold` (`pub`, L166), `rebuild_threshold` (`pub`, L172), `backoff_base_delay` (`pub`, L178), `backoff_max_delay` (`pub`, L184), `backoff_alert_threshold` (`pub`, L190), `partial_rebuild_config` (`pub`, L197), `build` (`pub`, L206)

#### `crates/contextra-vector/src/hnsw/core_insert.rs` (7)

- **HnswIndexCore**: `resolve_connections` (`pub(super)`, L13), `compute_insert` (`pub`, L60), `compute_insert_with_context` (`pub`, L65), `apply_insert` (`pub`, L366), `do_insert` (`pub(super)`, L432), `connectivity_score` (`pub`, L438), `check_connectivity` (`pub`, L454)

#### `crates/contextra-vector/src/hnsw/core_rebuild.rs` (23)

- **HnswIndexCore**: `rebuild_status` (`pub`, L14), `wait_for_rebuild` (`pub`, L22), `wait_for_rebuild_with_timeout` (`pub`, L27), `random_layer` (`pub(super)`, L44), `compute_distance_with_data` (`priv`, L51), `compute_distance_with_mmap` (`priv`, L81), `compute_symmetric_distance` (`priv`, L114), `resolve_dist` (`pub(super)`, L134), `resolve_doc_id` (`pub(super)`, L167), `resolve_vector_data_with_batch` (`priv`, L190), `compute_symmetric_distance_hybrid_with_batch` (`pub(super)`, L241), `select_neighbors_heuristic_with_batch` (`pub(super)`, L267), `do_delete` (`pub(super)`, L347), `deleted_ratio` (`pub`, L441), `rebuild_count` (`pub`, L445), `visited_dead_nodes` (`pub`, L449), `is_rebuild_required` (`pub`, L453), `rebuild` (`pub`, L467), `rebuild_sync` (`pub`, L471), `rebuild_region` (`pub`, L578), `rebuild_region_sync` (`pub`, L582), `rebuild_phase1_snapshot_and_build` (`priv`, L683), `rebuild_phase2_merge_and_swap` (`priv`, L807)

#### `crates/contextra-vector/src/hnsw/core_search.rs` (6)

- **HnswIndexCore**: `search_layer` (`pub(super)`, L15), `search_layer_with_context` (`pub(super)`, L26)
- **HnswIndex**: `search_filtered_internal` (`pub(super)`, L168), `search_adaptive` (`pub`, L430), `search_adaptive_filtered` (`pub`, L440), `search_adaptive_with_stats` (`pub`, L454)

#### `crates/contextra-vector/src/hnsw/deletion.rs` (3)

- **trait GhostFreeVectorIndex**: `remove_with_graph_repair` (`priv`, L33)
- **GhostFreeVectorIndex for HnswIndex**: `remove_with_graph_repair` (`priv`, L37)
- **HnswIndex**: `verify_no_ghost_pointers` (`priv`, L278)

#### `crates/contextra-vector/src/hnsw/sq8_bias.rs` (3)

- **Default for Sq8Bias**: `default` (`priv`, L21)
- **Sq8Bias**: `new` (`pub`, L32), `calibrate` (`pub`, L41)

#### `crates/contextra-vector/src/hnsw/tests.rs` (4)

- ***(Modulfunktionen)***: `test_config` (`priv`, L6)
- **contextra_ports::MetricsSink for TestMetricsSink**: `record_counter` (`priv`, L234), `record_gauge` (`priv`, L244), `record_histogram` (`priv`, L246)

#### `crates/contextra-vector/src/hnsw/types.rs` (44)

- **PartialEq for Candidate**: `eq` (`priv`, L46)
- **PartialOrd for Candidate**: `partial_cmp` (`priv`, L53)
- **Ord for Candidate**: `cmp` (`priv`, L59)
- **Drop for RebuildGuard<**: `drop` (`priv`, L67)
- **SnapshotPinGuard<**: `new` (`pub(super)`, L75)
- **Drop for SnapshotPinGuard<**: `drop` (`priv`, L82)
- **HnswIndexCore**: `set_layer_seed` (`pub`, L113), `metrics_sink` (`pub`, L118), `set_metrics_sink` (`pub`, L123)
- **HnswHotCore**: `get_entry_point` (`pub`, L130), `set_entry_point` (`pub`, L140), `get_ram_entry_point` (`pub`, L149), `set_ram_entry_point` (`pub`, L159), `get_ram_node_connections` (`pub`, L167)
- **HnswIndex**: `try_new_with_rng` (`pub`, L214), `try_new` (`pub`, L274), `new` (`pub`, L283), `quantizer` (`pub`, L353), `sq8_bias` (`pub`, L358), `metrics_sink` (`pub`, L363), `set_metrics_sink` (`pub`, L368), `with_metrics_sink` (`pub`, L373), `set_layer_seed` (`pub`, L379), `deleted_ratio` (`pub`, L383), `rebuild_count` (`pub`, L387), `visited_dead_nodes` (`pub`, L391), `connectivity_score` (`pub`, L395), `check_connectivity` (`pub`, L399), `is_rebuild_required` (`pub`, L403), `rebuild` (`pub`, L407), `rebuild_region` (`pub`, L411), `check_and_trigger_partial_rebuild` (`pub`, L416), `compact_seq_log` (`pub`, L449), `pin_snapshot` (`pub`, L453), `unpin_snapshot` (`pub`, L457), `all_doc_ids_from_map` (`pub`, L461), `all_doc_ids_and_layers` (`pub`, L471), `trigger_rebuild_async` (`pub`, L484), `rebuild_status` (`pub`, L496), `wait_for_rebuild` (`pub`, L500), `wait_for_rebuild_with_timeout` (`pub`, L504), `load_mmap` (`pub`, L508), `load_mmap_from_instance` (`pub(super)`, L513), `save` (`pub`, L609)

#### `crates/contextra-vector/src/hnsw/vector_index_impl.rs` (14)

- **VectorIndex for HnswIndex**: `insert` (`priv`, L15), `search` (`priv`, L42), `search_filtered` (`priv`, L221), `delete` (`priv`, L230), `commit` (`priv`, L247), `search_at` (`priv`, L384), `rollback` (`priv`, L392), `rollback_to_tx` (`priv`, L403), `last_tx_id` (`priv`, L471), `all_doc_ids` (`priv`, L475), `len` (`priv`, L508), `is_rebuild_required` (`priv`, L525), `trigger_rebuild_async` (`priv`, L529), `stats` (`priv`, L533)

#### `crates/contextra-vector/src/partial_rebuild.rs` (6)

- **Default for PartialRebuildConfig**: `default` (`priv`, L49)
- **TraversalTracker**: `new` (`pub`, L67), `record_traversal` (`pub`, L75), `find_oversaturated_regions` (`pub`, L87), `hot_path_nodes` (`pub`, L127)
- ***(Modulfunktionen)***: `should_trigger_partial_rebuild` (`pub`, L148)

#### `crates/contextra-vector/src/persistence/header.rs` (26)

- **HnswHeader**: `new` (`pub`, L39), `new_v2` (`pub`, L70), `new_v2_with_bias` (`pub`, L105), `magic` (`pub`, L143), `version` (`pub`, L147), `m` (`pub`, L151), `metric` (`pub`, L155), `quantized` (`pub`, L159), `q_min` (`pub`, L163), `q_max` (`pub`, L167), `nodes_offset` (`pub`, L171), `connections_offset` (`pub`, L175), `set_connections_offset` (`pub`, L179), `node_count` (`pub`, L183), `last_tx_id` (`pub`, L187), `quant_calibration_offset` (`pub`, L191), `quant_calibration_len` (`pub`, L195), `sq8_bias_mean` (`pub`, L199), `sq8_bias_variance` (`pub`, L203), `sq8_bias` (`pub`, L207), `dimension` (`pub`, L215), `entry_point` (`pub`, L219), `is_quantized` (`pub`, L223), `q_range` (`pub`, L227), `try_from_bytes` (`pub`, L231), `to_bytes` (`pub`, L416)

#### `crates/contextra-vector/src/persistence/mmap.rs` (5)

- **MmapIndex**: `open` (`pub`, L18), `open_async` (`pub`, L88), `get_node_record` (`pub`, L92), `get_vector` (`pub`, L110), `get_connections` (`pub`, L129)

#### `crates/contextra-vector/src/persistence/node.rs` (2)

- **NodeRecord**: `from_bytes` (`pub`, L25), `to_bytes` (`pub`, L72)

#### `crates/contextra-vector/src/quantize/mod.rs` (18)

- **Clone for ScalarQuantizer**: `clone` (`priv`, L39)
- **ScalarQuantizer**: `try_train` (`pub`, L61), `try_train_with_percentiles` (`pub`, L67), `train_with_percentiles` (`pub`, L171), `train` (`pub`, L194), `mins` (`pub`, L207), `maxes` (`pub`, L212), `scales` (`pub`, L217), `inv_scales` (`pub`, L222), `dimension` (`pub`, L227), `calculate_vector_drift` (`pub`, L238), `check_drift` (`pub`, L257), `drift_ratio` (`pub`, L266), `is_rebuild_required` (`pub`, L278), `quantize` (`pub`, L287), `dequantize` (`pub`, L357), `asymmetric_dist` (`pub`, L390), `symmetric_dist` (`pub`, L449)

#### `crates/contextra-vector/src/quantize_rabitq.rs` (6)

- **RaBitQQuantizer**: `try_train` (`pub`, L43), `dimension` (`pub`, L75), `code_length` (`pub`, L80), `quantize` (`pub`, L90), `asymmetric_distance` (`pub`, L157)
- ***(Modulfunktionen)***: `generate_orthogonal_matrix` (`priv`, L229)

### contextra-wire

> Ring 0 Unsafe Island: Auto-generated FlatBuffers IPC code and zero-copy adapters for Contextra

- **Pfad:** `crates/contextra-wire` | **Dateien:** 6 | **Produktiv-Funktionen:** 159 (`121` `pub`, `38` intern) | **Tests:** 26
- **Öffentliche Typen (45)**: `Unverified`, `Verified`, `VerifiableRoot`, `WireBuffer`, `EmbeddingOffset`, `Embedding`, `EmbeddingArgs`, `EmbeddingBuilder`, `ScoredDocumentOffset`, `ScoredDocument`, `ScoredDocumentArgs`, `ScoredDocumentBuilder`, `SearchResponseOffset`, `SearchResponse`, `SearchResponseArgs`, `SearchResponseBuilder`, `VectorIndexUpdateOffset`, `VectorIndexUpdate`, `VectorIndexUpdateArgs`, `VectorIndexUpdateBuilder`, `RoleIdOffset`, `RoleId`, `RoleIdArgs`, `RoleIdBuilder`, `HyperEdgeIdOffset`, `HyperEdgeId`, `HyperEdgeIdArgs`, `HyperEdgeIdBuilder`, `RoleBindingOffset`, `RoleBinding`, `RoleBindingArgs`, `RoleBindingBuilder`, `HyperEdgeOffset`, `HyperEdge`, `HyperEdgeArgs`, `HyperEdgeBuilder`, `JsonRpcConfig`, `JsonRpcPayload`, `JsonRpcRequest`, `JsonRpcResponse` *(+5 weitere)*

#### `crates/contextra-wire/src/adapter.rs` (9)

- **WireBuffer<Unverified>**: `new` (`pub`, L113), `verify` (`pub`, L131)
- **WireBuffer<Verified<T>>**: `root` (`pub`, L157), `root_with_config` (`pub`, L168)
- **WireBuffer<State>**: `as_slice` (`pub`, L187), `into_inner` (`pub`, L198), `len` (`pub`, L204), `is_empty` (`pub`, L210)
- **AsRef<**: `as_ref` (`priv`, L219)

#### `crates/contextra-wire/src/contextra_generated.rs` (126)

- **flatbuffers::Follow<**: `follow` (`priv`, L39), `follow` (`priv`, L171), `follow` (`priv`, L320), `follow` (`priv`, L452), `follow` (`priv`, L584), `follow` (`priv`, L682), `follow` (`priv`, L780), `follow` (`priv`, L895)
- **Embedding<**: `init_from_table` (`pub`, L51), `create` (`pub`, L55), `data` (`pub`, L68), `quantized_u8` (`pub`, L75), `metric` (`pub`, L82)
- **flatbuffers::Verifiable for Embedding<**: `run_verifier` (`priv`, L92)
- **Default for EmbeddingArgs<**: `default` (`priv`, L111)
- **EmbeddingBuilder<**: `add_data` (`pub`, L126), `add_quantized_u8` (`pub`, L130), `add_metric` (`pub`, L134), `new` (`pub`, L138), `finish` (`pub`, L146)
- **core::fmt::Debug for Embedding<**: `fmt` (`priv`, L153)
- **ScoredDocument<**: `init_from_table` (`pub`, L184), `create` (`pub`, L188), `id` (`pub`, L202), `score` (`pub`, L209), `metadata` (`pub`, L216), `embedding` (`pub`, L223)
- **flatbuffers::Verifiable for ScoredDocument<**: `run_verifier` (`priv`, L233)
- **Default for ScoredDocumentArgs<**: `default` (`priv`, L254)
- **ScoredDocumentBuilder<**: `add_id` (`pub`, L270), `add_score` (`pub`, L274), `add_metadata` (`pub`, L278), `add_embedding` (`pub`, L282), `new` (`pub`, L286), `finish` (`pub`, L294)
- **core::fmt::Debug for ScoredDocument<**: `fmt` (`priv`, L301)
- **SearchResponse<**: `init_from_table` (`pub`, L332), `create` (`pub`, L336), `results` (`pub`, L349), `total_hits` (`pub`, L356), `processing_time_ms` (`pub`, L363)
- **flatbuffers::Verifiable for SearchResponse<**: `run_verifier` (`priv`, L373)
- **Default for SearchResponseArgs<**: `default` (`priv`, L392)
- **SearchResponseBuilder<**: `add_results` (`pub`, L407), `add_total_hits` (`pub`, L411), `add_processing_time_ms` (`pub`, L415), `new` (`pub`, L419), `finish` (`pub`, L427)
- **core::fmt::Debug for SearchResponse<**: `fmt` (`priv`, L434)
- **VectorIndexUpdate<**: `init_from_table` (`pub`, L464), `create` (`pub`, L468), `id` (`pub`, L481), `embedding` (`pub`, L488), `metadata` (`pub`, L495)
- **flatbuffers::Verifiable for VectorIndexUpdate<**: `run_verifier` (`priv`, L505)
- **Default for VectorIndexUpdateArgs<**: `default` (`priv`, L524)
- **VectorIndexUpdateBuilder<**: `add_id` (`pub`, L539), `add_embedding` (`pub`, L543), `add_metadata` (`pub`, L547), `new` (`pub`, L551), `finish` (`pub`, L559)
- **core::fmt::Debug for VectorIndexUpdate<**: `fmt` (`priv`, L566)
- **RoleId<**: `init_from_table` (`pub`, L594), `create` (`pub`, L598), `id` (`pub`, L609)
- **flatbuffers::Verifiable for RoleId<**: `run_verifier` (`priv`, L619)
- **Default for RoleIdArgs**: `default` (`priv`, L634)
- **RoleIdBuilder<**: `add_id` (`pub`, L647), `new` (`pub`, L651), `finish` (`pub`, L659)
- **core::fmt::Debug for RoleId<**: `fmt` (`priv`, L666)
- **HyperEdgeId<**: `init_from_table` (`pub`, L692), `create` (`pub`, L696), `id` (`pub`, L707)
- **flatbuffers::Verifiable for HyperEdgeId<**: `run_verifier` (`priv`, L717)
- **Default for HyperEdgeIdArgs**: `default` (`priv`, L732)
- **HyperEdgeIdBuilder<**: `add_id` (`pub`, L745), `new` (`pub`, L749), `finish` (`pub`, L757)
- **core::fmt::Debug for HyperEdgeId<**: `fmt` (`priv`, L764)
- **RoleBinding<**: `init_from_table` (`pub`, L791), `create` (`pub`, L795), `role` (`pub`, L807), `entity` (`pub`, L814)
- **flatbuffers::Verifiable for RoleBinding<**: `run_verifier` (`priv`, L824)
- **Default for RoleBindingArgs**: `default` (`priv`, L841)
- **RoleBindingBuilder<**: `add_role` (`pub`, L855), `add_entity` (`pub`, L859), `new` (`pub`, L863), `finish` (`pub`, L871)
- **core::fmt::Debug for RoleBinding<**: `fmt` (`priv`, L878)
- **HyperEdge<**: `init_from_table` (`pub`, L914), `create` (`pub`, L918), `id` (`pub`, L938), `predicate` (`pub`, L945), `participants` (`pub`, L952), `weight` (`pub`, L959), `tx_valid_from` (`pub`, L966), `tx_valid_to` (`pub`, L973), `business_valid_from` (`pub`, L980), `business_valid_to` (`pub`, L987), `source_doc_id` (`pub`, L994), `child_edge_ids` (`pub`, L1001)
- **flatbuffers::Verifiable for HyperEdge<**: `run_verifier` (`priv`, L1011)
- **Default for HyperEdgeArgs<**: `default` (`priv`, L1044)
- **HyperEdgeBuilder<**: `add_id` (`pub`, L1066), `add_predicate` (`pub`, L1070), `add_participants` (`pub`, L1074), `add_weight` (`pub`, L1078), `add_tx_valid_from` (`pub`, L1082), `add_tx_valid_to` (`pub`, L1086), `add_business_valid_from` (`pub`, L1090), `add_business_valid_to` (`pub`, L1094), `add_source_doc_id` (`pub`, L1098), `add_child_edge_ids` (`pub`, L1102), `new` (`pub`, L1106), `finish` (`pub`, L1114)
- **core::fmt::Debug for HyperEdge<**: `fmt` (`priv`, L1121)
- ***(Modulfunktionen)***: `root_as_search_response` (`pub`, L1143), `size_prefixed_root_as_search_response` (`pub`, L1153), `root_as_search_response_with_opts` (`pub`, L1163), `size_prefixed_root_as_search_response_with_opts` (`pub`, L1176), `root_as_search_response_unchecked` (`pub`, L1186), `size_prefixed_root_as_search_response_unchecked` (`pub`, L1194), `finish_search_response_buffer` (`pub`, L1199), `finish_size_prefixed_search_response_buffer` (`pub`, L1206)

#### `crates/contextra-wire/src/jsonrpc.rs` (13)

- **Default for JsonRpcConfig**: `default` (`priv`, L51)
- **JsonRpcRequest**: `validate` (`pub`, L106)
- **std::fmt::Display for JsonRpcError**: `fmt` (`priv`, L163)
- **JsonRpcError**: `parse_error` (`pub`, L174), `invalid_request` (`pub`, L185), `method_not_found` (`pub`, L196), `invalid_params` (`pub`, L207), `internal_error` (`pub`, L218)
- **JsonRpcResponse**: `ok` (`pub`, L236), `err` (`pub`, L254)
- ***(Modulfunktionen)***: `parse_jsonrpc_payload` (`pub`, L272), `parse_jsonrpc_payload_with_config` (`pub`, L294), `validate_json_structure_and_limits` (`priv`, L368)

#### `crates/contextra-wire/src/verification.rs` (11)

- **fmt::Display for FlatBufferError**: `fmt` (`priv`, L50)
- **Default for FlatBufferConfig**: `default` (`priv`, L88)
- ***(Modulfunktionen)***: `verify_flatbuffer` (`pub`, L111), `verify_and_parse_search_response` (`pub`, L146), `verify_and_parse_embedding` (`pub`, L160), `verify_and_parse_scored_document` (`pub`, L174), `verify_and_parse_vector_index_update` (`pub`, L188), `verify_and_parse_role_id` (`pub`, L202), `verify_and_parse_hyper_edge_id` (`pub`, L216), `verify_and_parse_role_binding` (`pub`, L230), `verify_and_parse_hyper_edge` (`pub`, L244)

---

## 3. Detailinventar: Werkzeuge & Benchmarks

### contextra-bench

> Contextra — Reproducible Benchmark Harness for Retrieval Accuracy

- **Pfad:** `benchmarks/contextra-bench` | **Dateien:** 11 | **Produktiv-Funktionen:** 64 (`26` `pub`, `38` intern) | **Tests:** 23
- **Öffentliche Typen (34)**: `AnnBenchResult`, `BeirDocument`, `BeirQuery`, `BeirMetrics`, `IntervalResult`, `ScaleBenchmarkOutput`, `LongMemEvalMetricsSummary`, `LocomoMetricsSummary`, `CombinedMetrics`, `MetricComparisonResult`, `LocomoQuestionCategory`, `LocomoCase`, `LocomoReport`, `LocomoConversationIndex`, `LocomoDataset`, `RawLocomoTurn`, `BoxFuture`, `ScoredChunk`, `LongMemEvalQuestionType`, `LongMemEvalCase`, `LongMemEvalReport`, `SessionTurn`, `Session`, `MultiSessionScenario`, `RegressionSuite`, `RegressionReport`, `GroundTruthQuery`, `SyntheticDocument`, `BenchmarkMetrics`, `ScenarioComparison`, `BenchmarkReport`, `PathRagSweepMetric`, `PathRagSweepReport`, `RegressionGateResult`

#### `benchmarks/contextra-bench/src/ann_benchmarks.rs` (8)

- ***(Modulfunktionen)***: `generate_synthetic_vectors` (`pub`, L35), `euclidean_distance_sq` (`priv`, L50), `cosine_distance` (`priv`, L60), `dot_product_distance` (`priv`, L75), `compute_ground_truth` (`pub`, L80), `recall_at_k` (`pub`, L112), `run_ann_benchmark` (`pub`, L136), `run_ann_benchmark_synthetic_ci` (`pub`, L249)

#### `benchmarks/contextra-bench/src/beir_eval.rs` (6)

- ***(Modulfunktionen)***: `load_beir_corpus` (`pub`, L43), `load_beir_queries` (`pub`, L81), `ndcg_at_k` (`pub`, L185), `map_at_k` (`pub`, L214), `recall_at_k` (`pub`, L239), `run_beir_eval` (`pub`, L261)

#### `benchmarks/contextra-bench/src/bin/compare_baseline.rs` (2)

- ***(Modulfunktionen)***: `print_usage` (`priv`, L11), `main` (`priv`, L20)

#### `benchmarks/contextra-bench/src/bin/scale_100k.rs` (9)

- **SimplePrng**: `new` (`priv`, L39), `next_u64` (`priv`, L43), `next_f32` (`priv`, L51), `next_vector` (`priv`, L56)
- ***(Modulfunktionen)***: `read_rss_bytes` (`priv`, L69), `parse_cli_args` (`priv`, L85), `write_output` (`priv`, L132), `run_benchmark` (`priv`, L142), `main` (`priv`, L295)

#### `benchmarks/contextra-bench/src/compare.rs` (3)

- ***(Modulfunktionen)***: `compare_metrics` (`pub`, L41), `compare_single_metric` (`priv`, L109), `compare_metrics_files` (`pub`, L195)

#### `benchmarks/contextra-bench/src/locomo.rs` (6)

- **LocomoQuestionCategory**: `from_u8` (`pub`, L25)
- **std::fmt::Display for LocomoQuestionCategory**: `fmt` (`priv`, L38)
- **std::ops::Deref for LocomoDataset**: `deref` (`priv`, L85)
- ***(Modulfunktionen)***: `extract_turns` (`priv`, L129), `load_locomo_dataset` (`pub`, L144), `run_locomo_eval` (`pub`, L240)

#### `benchmarks/contextra-bench/src/long_mem_eval.rs` (12)

- **std::fmt::Display for LongMemEvalQuestionType**: `fmt` (`priv`, L40)
- **LongMemEvalCase**: `answer_str` (`pub`, L68), `expected_answer` (`pub`, L80)
- ***(Modulfunktionen)***: `check_regression` (`pub`, L143), `pad_vector` (`priv`, L191), `load_from_jsonl` (`pub`, L1125), `parse_question_type` (`priv`, L1195), `json_val_to_string` (`priv`, L1215), `parse_raw_item` (`priv`, L1236), `run_long_mem_eval` (`pub`, L1304)
- **RegressionSuite**: `baseline` (`pub`, L201), `run_against_collection` (`pub`, L992)

#### `benchmarks/contextra-bench/src/main.rs` (14)

- ***(Modulfunktionen)***: `pad_vector` (`priv`, L74), `build_evaluation_corpus` (`priv`, L82), `create_synthetic_corpus` (`priv`, L448), `calculate_metrics` (`priv`, L535), `run_scenario_a` (`priv`, L589), `run_scenario_b` (`priv`, L701), `run_scenario_b` (`priv`, L800), `generate_markdown_summary` (`priv`, L811), `update_current_metrics_long_mem_eval` (`priv`, L896), `update_current_metrics_locomo` (`priv`, L923), `run_long_mem_eval_cmd` (`priv`, L952), `run_locomo_cmd` (`priv`, L1042), `run_pathrag_sweep_cmd` (`priv`, L1193), `main` (`priv`, L1276)

#### `benchmarks/contextra-bench/src/path_rag_sweep.rs` (3)

- ***(Modulfunktionen)***: `pad_vector` (`priv`, L14), `run_pathrag_sweep_long_mem_eval` (`pub`, L39), `run_pathrag_sweep_locomo` (`pub`, L198)

#### `benchmarks/contextra-bench/src/regression_gate.rs` (1)

- ***(Modulfunktionen)***: `run_regression_gate` (`pub`, L26)

### xtask

- **Pfad:** `xtask` | **Dateien:** 148 | **Produktiv-Funktionen:** 883 (`590` `pub`, `293` intern) | **Tests:** 232
- **Öffentliche Typen (222)**: `ArtifactHeaderError`, `ArtifactHeader`, `GapSeverity`, `AuditFixGap`, `BenchCompileResult`, `Dataset`, `DownloadTarget`, `AdrDeadlineEntry`, `AdrDeadlineCheckResult`, `FreshnessViolation`, `AuditSection`, `DuplicationMatch`, `AuditEvidenceViolation`, `EvidenceMarker`, `VerdictEntry`, `VerdictValidationResult`, `CommitDiffStats`, `CrateCoverageResult`, `DocRefViolation`, `PrimitiveDuplicate`, `PrimitiveOccurrence`, `CommitEntry`, `DuplicateHit`, `BranchDiffEntry`, `DuplicateSymbol`, `CrossFileDuplicate`, `FreshnessError`, `Violation`, `ModuleReachabilityResult`, `MutationScoreThreshold`, `MutationGateReport`, `MutationGateError`, `Violation`, `ViolationReason`, `PlaceholderViolation`, `RecallHistoryEntry`, `Violation`, `AsyncException`, `AsyncPurityViolation`, `RingConsistencyFinding` *(+182 weitere)*

#### `xtask/src/agent_lifecycle/branch_overlap.rs` (13)

- ***(Modulfunktionen)***: `run_check_branch_overlap` (`pub`, L21), `run_check_branch_overlap_with_args` (`pub`, L27), `find_root_dir_local` (`priv`, L184), `determine_active_card_scope` (`priv`, L198), `find_card_id_from_git_log` (`priv`, L215), `find_card_id_from_pr_body` (`priv`, L229), `find_card_id_from_session_md` (`priv`, L234), `find_card_id_from_claims_json` (`priv`, L250), `parse_ls_remote_branches` (`priv`, L268), `git_rev_parse` (`priv`, L283), `git_rev_parse_branch` (`priv`, L296), `git_merge_base` (`priv`, L314), `git_diff_name_only` (`priv`, L332)

#### `xtask/src/agent_lifecycle/prune_branches.rs` (1)

- ***(Modulfunktionen)***: `run_prune_branches` (`pub`, L8)

#### `xtask/src/artifact_header.rs` (4)

- **fmt::Display for ArtifactHeaderError**: `fmt` (`priv`, L21)
- **ArtifactHeader**: `capture` (`pub`, L44), `render_markdown_frontmatter` (`pub`, L89), `render_json` (`pub`, L96)

#### `xtask/src/audit_integrity_check.rs` (5)

- ***(Modulfunktionen)***: `parse_iso_datetime` (`priv`, L25), `extract_verdict` (`priv`, L41), `derive_crate_name` (`priv`, L50), `run_audit_integrity_check_impl` (`pub`, L66), `run_audit_integrity_check` (`pub`, L248)

#### `xtask/src/bench_compile.rs` (3)

- ***(Modulfunktionen)***: `discover_benches` (`pub`, L15), `build_bench_command_args` (`pub`, L60), `run_bench_compile` (`pub`, L87)

#### `xtask/src/bench_download.rs` (3)

- ***(Modulfunktionen)***: `get_dataset_targets` (`pub`, L28), `build_curl_args` (`pub`, L72), `run_bench_download` (`pub`, L82)

#### `xtask/src/bin/update_registry.rs` (1)

- ***(Modulfunktionen)***: `main` (`priv`, L3)

#### `xtask/src/check_action_pinning.rs` (1)

- ***(Modulfunktionen)***: `run_check_action_pinning` (`pub`, L10)

#### `xtask/src/check_adr_deadlines.rs` (6)

- **AdrDeadlineEntry**: `title_summary` (`pub`, L32)
- ***(Modulfunktionen)***: `parse_adr_deadlines` (`pub`, L37), `check_adr_deadlines_at` (`pub`, L123), `check_adr_deadlines_with_root_and_date` (`pub`, L176), `check_adr_deadlines_with_root` (`pub`, L238), `check_adr_deadlines` (`pub`, L243)

#### `xtask/src/check_agents_freshness.rs` (3)

- ***(Modulfunktionen)***: `get_changed_files_from_git` (`pub`, L17), `check_agents_freshness_for_files` (`pub`, L52), `run_check_agents_freshness` (`pub`, L105)

#### `xtask/src/check_agents_integrity.rs` (2)

- ***(Modulfunktionen)***: `run_check_agents_integrity` (`pub`, L6), `is_type_or_crate_implemented` (`priv`, L89)

#### `xtask/src/check_audit_duplication.rs` (7)

- ***(Modulfunktionen)***: `parse_audit_sections` (`pub`, L24), `normalize_sentence` (`pub`, L61), `extract_sentence_set` (`pub`, L72), `calculate_jaccard_similarity` (`pub`, L89), `check_sections_in_file` (`pub`, L102), `find_audit_files` (`pub`, L141), `run_check_audit_duplication` (`pub`, L165)

#### `xtask/src/check_audit_tool_evidence.rs` (3)

- ***(Modulfunktionen)***: `parse_evidence_markers` (`pub`, L26), `extract_audit_timestamp` (`pub`, L53), `run_check_audit_tool_evidence` (`pub`, L71)

#### `xtask/src/check_audit_verdict_independence.rs` (6)

- ***(Modulfunktionen)***: `parse_iso_timestamp` (`pub`, L36), `parse_verdict_line` (`pub`, L54), `is_actionable_verdict` (`pub`, L113), `validate_verdict_entry` (`pub`, L122), `find_audit_files` (`pub`, L223), `run_check_audit_verdict_independence` (`pub`, L248)

#### `xtask/src/check_commit_diff_integrity.rs` (9)

- **CommitDiffStats**: `total_lines_changed` (`pub`, L46)
- ***(Modulfunktionen)***: `get_commit_diff_stats` (`pub`, L52), `parse_git_stat_output` (`pub`, L78), `get_commit_message` (`pub`, L121), `strip_conventional_commit_prefix` (`priv`, L141), `count_claim_points` (`pub`, L171), `check_single_commit` (`pub`, L239), `resolve_commit_range` (`pub`, L272), `run_check_commit_diff_integrity` (`pub`, L301)

#### `xtask/src/check_commit_messages.rs` (2)

- ***(Modulfunktionen)***: `validate_commit_message` (`pub`, L17), `check_commit_messages` (`pub`, L64)

#### `xtask/src/check_compile.rs` (1)

- ***(Modulfunktionen)***: `run_check_compile` (`pub`, L10)

#### `xtask/src/check_coverage_gate.rs` (3)

- ***(Modulfunktionen)***: `get_threshold_for_crate` (`pub`, L40), `run_check_coverage_gate` (`pub`, L56), `run_check_coverage_gate_file` (`pub`, L60)

#### `xtask/src/check_doc_references.rs` (5)

- ***(Modulfunktionen)***: `extract_path_candidates` (`pub`, L25), `resolve_path_reference` (`pub`, L106), `check_doc_references_in_content` (`pub`, L168), `check_doc_references_for_files` (`pub`, L193), `run_check_doc_references` (`pub`, L226)

#### `xtask/src/check_duplicate_core_primitives.rs` (3)

- ***(Modulfunktionen)***: `scan_duplicate_core_primitives` (`pub`, L27), `find_root_dir` (`priv`, L148), `run_check_duplicate_core_primitives` (`pub`, L184)

#### `xtask/src/check_duplicate_intent.rs` (19)

- ***(Modulfunktionen)***: `normalize_subject` (`pub`, L17), `parse_conventional_commit` (`pub`, L32), `get_trigrams` (`pub`, L51), `normalized_similarity` (`pub`, L66), `parse_override_exception` (`pub`, L90), `parse_git_log_output` (`pub`, L96), `get_historical_commits` (`pub`, L121), `get_remote_branch_commits` (`pub`, L135), `get_current_head_sha` (`pub`, L155), `fetch_open_prs_from_github` (`pub`, L165), `get_new_commits` (`pub`, L228), `evaluate_duplicate_hits` (`pub`, L266), `parse_diff_file_path` (`priv`, L306), `extract_changed_files_and_symbols` (`pub`, L332), `compute_symbol_overlap` (`pub`, L386), `get_remote_branch_diffs` (`pub`, L414), `fetch_open_pr_diffs_from_github` (`pub`, L491), `get_current_diff` (`pub`, L602), `check_duplicate_intent` (`pub`, L640)

#### `xtask/src/check_duplicate_symbols.rs` (6)

- ***(Modulfunktionen)***: `scan_file_for_duplicate_symbols` (`pub`, L36), `scan_crate_for_cross_module_duplicate_symbols` (`pub`, L198), `scan_crate_for_cross_module_duplicate_fns` (`pub`, L380), `find_crate_roots_to_scan` (`priv`, L390), `check_duplicate_symbols` (`pub`, L432), `check_duplicate_symbols_with_options` (`pub`, L437)

#### `xtask/src/check_duplicate_symbols_cross_file.rs` (5)

- ***(Modulfunktionen)***: `get_pub_item_info` (`priv`, L25), `extract_cfg_attr` (`priv`, L45), `scan_directory_for_cross_file_duplicates` (`pub`, L59), `scan_workspace_for_cross_file_duplicates` (`pub`, L173), `run` (`pub`, L218)

#### `xtask/src/check_ffi_panic_boundary.rs` (3)

- ***(Modulfunktionen)***: `panic_abort_active` (`pub`, L16), `check_file` (`pub`, L41), `run_check_ffi_panic_boundary` (`pub`, L76)

#### `xtask/src/check_flatbuffers_drift.rs` (4)

- ***(Modulfunktionen)***: `find_or_fetch_flatc` (`priv`, L12), `check_flatbuffers_drift` (`pub`, L78), `regenerate_flatbuffers` (`pub`, L153), `normalize_code` (`priv`, L195)

#### `xtask/src/check_jules_context_freshness.rs` (3)

- ***(Modulfunktionen)***: `check_file_freshness` (`pub`, L25), `get_last_crates_change_date` (`pub`, L66), `run_check_jules_context_freshness` (`pub`, L93)

#### `xtask/src/check_manifest_completeness.rs` (4)

- ***(Modulfunktionen)***: `extract_workspace_member_crates` (`pub`, L23), `extract_capabilities_crates` (`pub`, L100), `validate_manifest_completeness` (`pub`, L133), `run` (`pub`, L189)

#### `xtask/src/check_max_results_unbound.rs` (2)

- ***(Modulfunktionen)***: `run_check_max_results_unbound` (`pub`, L13), `run_check_max_results_unbound_with_options` (`pub`, L17)

#### `xtask/src/check_module_reachability.rs` (6)

- ***(Modulfunktionen)***: `strip_comments_and_strings` (`priv`, L30), `extract_mod_declarations` (`priv`, L189), `resolve_mod_file` (`priv`, L236), `find_crate_roots` (`priv`, L292), `check_crate_reachability` (`pub`, L322), `run_check_module_reachability` (`pub`, L478)

#### `xtask/src/check_mutation_score_gate.rs` (4)

- ***(Modulfunktionen)***: `get_mutation_threshold_for_crate` (`pub`, L13), `check_gate` (`pub`, L88), `run_check_mutation_score_gate` (`pub`, L168)
- **fmt::Display for MutationGateError**: `fmt` (`priv`, L50)

#### `xtask/src/check_nan_validation_in_hot_loop.rs` (2)

- ***(Modulfunktionen)***: `run_check_nan_validation_in_hot_loop` (`pub`, L13), `run_check_nan_validation_in_hot_loop_with_options` (`pub`, L17)

#### `xtask/src/check_orphan_modules.rs` (1)

- ***(Modulfunktionen)***: `run_check_orphan_modules` (`pub`, L10)

#### `xtask/src/check_phantom_files.rs` (4)

- ***(Modulfunktionen)***: `extract_claimed_rs_files` (`priv`, L9), `get_actual_diff_files` (`priv`, L15), `get_recent_commit_messages` (`priv`, L28), `run_check_phantom_files` (`pub`, L51)

#### `xtask/src/check_placeholder_refs.rs` (4)

- ***(Modulfunktionen)***: `collect_target_files` (`pub`, L27), `check_placeholder_refs_in_content` (`pub`, L69), `check_placeholder_refs` (`pub`, L155), `run` (`pub`, L175)

#### `xtask/src/check_recall_stability.rs` (4)

- ***(Modulfunktionen)***: `run_check_recall_stability` (`pub`, L22), `check_recall_stability_from_file` (`pub`, L32), `parse_history_entries` (`pub`, L66), `evaluate_recall_stability` (`pub`, L85)

#### `xtask/src/check_result_dropped_on_io.rs` (3)

- ***(Modulfunktionen)***: `run_check_result_dropped_on_io` (`pub`, L13), `strip_comments_and_strings` (`priv`, L19), `run_check_result_dropped_on_io_with_options` (`pub`, L177)

#### `xtask/src/check_ring0_async_purity.rs` (3)

- ***(Modulfunktionen)***: `load_allowlist` (`pub`, L45), `check_cargo_toml_dependencies` (`pub`, L80), `run_check_ring0_async_purity` (`pub`, L111)

#### `xtask/src/check_ring_capabilities_consistency.rs` (3)

- ***(Modulfunktionen)***: `find_root_dir` (`priv`, L27), `run_check_ring_capabilities_consistency` (`pub`, L49), `run_check_ring_capabilities_consistency_in_root` (`pub`, L54)

#### `xtask/src/check_ring_layering.rs` (12)

- **Ring**: `name` (`pub`, L26), `from_str` (`pub`, L37)
- ***(Modulfunktionen)***: `ring0_crate_order` (`pub`, L52), `ring3_crate_order` (`pub`, L65), `find_allowlist_entry` (`pub`, L112), `get_crate_ring` (`pub`, L164), `get_ring_map_from_metadata_json` (`pub`, L169), `get_workspace_ring_map` (`pub`, L198), `check_ring_layering_from_metadata_json` (`pub`, L215), `run_check_ring_layering` (`pub`, L426), `run_check_ring_layering_full` (`pub`, L430), `run_check_ring_layering_with_options` (`pub`, L434)

#### `xtask/src/check_stale_tags.rs` (6)

- ***(Modulfunktionen)***: `scan_audit_tags_in_root` (`pub`, L43), `get_line_last_commit_date` (`pub`, L119), `parse_iso_datetime` (`pub`, L145), `parse_until_date` (`pub`, L161), `check_stale_tags_impl` (`pub`, L166), `run_check_stale_tags` (`pub`, L363)

#### `xtask/src/check_toc_integrity.rs` (3)

- ***(Modulfunktionen)***: `slugify` (`pub`, L23), `check_toc_integrity_content` (`pub`, L42), `run_check_toc_integrity` (`pub`, L143)

#### `xtask/src/check_toctou_trait_defaults.rs` (2)

- ***(Modulfunktionen)***: `run_check_toctou_trait_defaults` (`pub`, L12), `run_check_toctou_trait_defaults_with_options` (`pub`, L16)

#### `xtask/src/check_type_registry.rs` (3)

- ***(Modulfunktionen)***: `run_check_type_registry` (`pub`, L33), `check_type_registry` (`pub`, L65), `extract_crate_from_path` (`priv`, L177)

#### `xtask/src/check_unsafe_islands.rs` (5)

- ***(Modulfunktionen)***: `strip_comments_and_strings` (`priv`, L44), `extract_crate_name_from_path` (`pub`, L181), `scan_file_for_unsafe` (`pub`, L197), `run_check_unsafe_islands_at` (`pub`, L272), `run_check_unsafe_islands` (`pub`, L351)

#### `xtask/src/check_vetoes.rs` (10)

- **VetoEntry**: `reason_summary` (`pub`, L34), `is_file_affected` (`pub`, L48)
- ***(Modulfunktionen)***: `parse_vetoes` (`pub`, L69), `check_conditional_review_deadlines_scoped` (`pub`, L180), `parse_adr_status` (`priv`, L245), `is_adr_status_accepted` (`priv`, L272), `validate_adr_on_base_commit` (`pub`, L284), `check_veto_date_extensions` (`pub`, L325), `check_vetoes_with_root_and_opts` (`pub`, L369), `check_vetoes` (`pub`, L509)

#### `xtask/src/check_workflow_commands.rs` (4)

- ***(Modulfunktionen)***: `extract_valid_subcommands` (`pub`, L11), `extract_harness_subcommands` (`pub`, L42), `parse_workflow_invocations` (`pub`, L74), `run_check_workflow_commands` (`pub`, L99)

#### `xtask/src/claim.rs` (12)

- ***(Modulfunktionen)***: `get_max_active_claims` (`pub`, L21), `default_active` (`priv`, L45), `expire_stale_claims` (`pub`, L126), `write_session_snapshot` (`pub`, L165), `run_release_local` (`pub`, L203), `run_claim` (`pub`, L242), `run_claim_local` (`priv`, L255), `run_claim_github` (`pub`, L368)
- **ClaimsDatabase**: `load` (`pub`, L55), `save` (`pub`, L66), `find_active_claim` (`pub`, L83), `count_active_claims` (`pub`, L102)

#### `xtask/src/cli/audit.rs` (8)

- ***(Modulfunktionen)***: `run_debt_audit` (`pub`, L6), `run_shell_commit_audit` (`pub`, L14), `run_audit_integrity_check` (`pub`, L45), `run_hotspot_report` (`pub`, L84), `run_tag_health` (`pub`, L123), `run_commit_health` (`pub`, L144), `run_panic_inventory` (`pub`, L166), `run_security_scan` (`pub`, L185)

#### `xtask/src/cli/bench.rs` (4)

- ***(Modulfunktionen)***: `run_bench_trend` (`pub`, L6), `run_bench_gate` (`pub`, L13), `run_bench_compile` (`pub`, L29), `run_bench_download` (`pub`, L47)

#### `xtask/src/cli/checks.rs` (49)

- ***(Modulfunktionen)***: `run_check_commit_diff_integrity` (`pub`, L6), `run_check_toc_integrity` (`pub`, L15), `run_check_agents_freshness` (`pub`, L24), `run_check_manifest_completeness` (`pub`, L32), `run_check_crate_references` (`pub`, L40), `run_check_unsafe_islands` (`pub`, L48), `run_check_ring_layering` (`pub`, L64), `run_check_ring_layering_full` (`pub`, L80), `run_check_duplicate_core_primitives` (`pub`, L96), `run_check_ring0_async_purity` (`pub`, L111), `run_check_module_reachability` (`pub`, L126), `run_check_audit_tool_evidence` (`pub`, L156), `run_check_audit_verdict_independence` (`pub`, L183), `run_check_ffi_panic_boundary` (`pub`, L190), `run_lint_unsafe_slices` (`pub`, L200), `run_check_mutation_score_gate` (`pub`, L207), `run_mutation_score_record` (`pub`, L217), `run_check_compile` (`pub`, L227), `run_check_phantom_files` (`pub`, L234), `run_check_audit_duplication` (`pub`, L241), `run_check_recall_stability` (`pub`, L249), `run_check_doc_references` (`pub`, L257), `run_check_duplicate_intent` (`pub`, L265), `run_check_commit_messages` (`pub`, L273), `run_check_duplicate_symbols_cross_file` (`pub`, L281), `run_check_duplicate_symbols` (`pub`, L289), `run_check_workflow_commands` (`pub`, L326), `run_check_placeholder_refs` (`pub`, L334), `run_check_bandit_latency_budget` (`pub`, L342), `run_check_action_pinning` (`pub`, L366), `run_check_flatbuffers_drift` (`pub`, L389), `run_regenerate_flatbuffers` (`pub`, L397), `run_check_adr_deadlines` (`pub`, L405), `run_check_vetoes` (`pub`, L413), `run_validate_tags` (`pub`, L421), `run_check_review_coverage` (`pub`, L431), `run_check_agents_integrity` (`pub`, L440), `run_check_consistency` (`pub`, L448), `run_check_dag` (`pub`, L456), `run_check_type_registry` (`pub`, L463), `run_check_stale_tags` (`pub`, L478), `run_check_max_results_unbound` (`pub`, L492), `run_check_toctou_defaults` (`pub`, L520), `run_check_nan_hot_loop` (`pub`, L551), `run_check_result_dropped_io` (`pub`, L579), `run_check_coverage_gate` (`pub`, L607), `run_check_marker_drift` (`pub`, L646), `run_check_ring_capabilities_consistency` (`pub`, L656), `run_check_veto_deadlines` (`pub`, L679)

#### `xtask/src/cli/docs.rs` (5)

- ***(Modulfunktionen)***: `run_sync_docs` (`pub`, L6), `run_gen_arch_docs` (`pub`, L15), `run_gen_feature_catalog` (`pub`, L24), `run_generate_adr` (`pub`, L32), `run_consolidate_adrs` (`pub`, L51)

#### `xtask/src/cli/jules.rs` (11)

- ***(Modulfunktionen)***: `run_generate_diagnostics` (`pub`, L6), `run_jules_preflight` (`pub`, L14), `run_validate_pr_checklist` (`pub`, L23), `run_claim` (`pub`, L31), `run_pre_push` (`pub`, L39), `run_jules_submit_gate` (`pub`, L47), `run_gate_check` (`pub`, L66), `run_generate_markers` (`pub`, L132), `run_context_pack` (`pub`, L142), `run_session_init` (`pub`, L158), `run_crate_context` (`pub`, L175)

#### `xtask/src/cli/misc.rs` (13)

- ***(Modulfunktionen)***: `run_reproducible_build` (`pub`, L6), `run_prune_branches` (`pub`, L14), `run_forensic_test` (`pub`, L21), `run_gen_sbom` (`pub`, L33), `run_run_community_detection` (`pub`, L40), `run_context_tags` (`pub`, L48), `run_post_merge_report` (`pub`, L55), `run_migrate_docid_128` (`pub`, L64), `run_feature_matrix` (`pub`, L72), `run_workspace_verify` (`pub`, L95), `run_loom_run` (`pub`, L143), `run_py_test` (`pub`, L162), `run_env_validate` (`pub`, L181)

#### `xtask/src/cli/tools.rs` (5)

- ***(Modulfunktionen)***: `run_generate_diagnostics` (`pub`, L6), `run_validate_pr_checklist` (`pub`, L14), `run_gate_check` (`pub`, L22), `run_generate_markers` (`pub`, L88), `run_crate_context` (`pub`, L98)

#### `xtask/src/commit_health.rs` (2)

- ***(Modulfunktionen)***: `run_commit_health_impl` (`pub`, L5), `run_commit_health` (`pub`, L172)

#### `xtask/src/context_pack.rs` (2)

- ***(Modulfunktionen)***: `scan_critical_tags` (`pub`, L24), `run_context_pack` (`pub`, L70)

#### `xtask/src/crate_context.rs` (6)

- ***(Modulfunktionen)***: `count_crate_loc` (`priv`, L25), `extract_declared_contextra_deps` (`priv`, L44), `scan_crate_ai_tags` (`priv`, L66), `read_agents_md_excerpt` (`priv`, L96), `parse_capabilities_info` (`priv`, L107), `run_crate_context` (`pub`, L135)

#### `xtask/src/env_validate.rs` (3)

- ***(Modulfunktionen)***: `read_required_rustc_version` (`priv`, L20), `parse_semver` (`priv`, L39), `run_env_validate` (`pub`, L60)

#### `xtask/src/feature_matrix.rs` (5)

- **FeatureDefault**: `as_str` (`pub`, L21)
- ***(Modulfunktionen)***: `find_root_dir` (`priv`, L58), `scan_crate_features` (`pub`, L80), `run_feature_matrix` (`pub`, L164), `run_feature_matrix_in_root` (`pub`, L172)

#### `xtask/src/gate_check.rs` (8)

- **GateLevel**: `from_str` (`pub`, L14), `number` (`pub`, L28)
- **std::fmt::Display for GateCheckError**: `fmt` (`priv`, L69)
- ***(Modulfunktionen)***: `run` (`pub`, L116), `execute_cmd` (`priv`, L135), `run_gate_zero_for_crate` (`priv`, L172), `run_gate_zero` (`priv`, L224), `run_gate_one` (`priv`, L241)

#### `xtask/src/gates/check_crate_references.rs` (7)

- ***(Modulfunktionen)***: `get_active_workspace_members` (`pub`, L24), `extract_crate_references` (`pub`, L63), `should_check_file` (`pub`, L95), `check_crate_references_in_content` (`pub`, L124), `scan_and_check_workspace` (`pub`, L154), `run_check_crate_references` (`pub`, L201), `run` (`pub`, L235)

#### `xtask/src/gates/debt_audit.rs` (20)

- ***(Modulfunktionen)***: `meta_evals_to_test` (`priv`, L21), `is_test_attribute` (`pub`, L44), `has_test_attribute` (`pub`, L65), `has_inline_suppression` (`pub`, L70), `is_test_or_generated_file` (`pub`, L81), `get_item_attrs` (`priv`, L229), `check_unwraps_in_file` (`pub`, L250), `scan_unwraps_in_workspace` (`pub`, L265), `check_unsafe_code` (`pub`, L294), `check_stdfs_in_workspace` (`pub`, L365), `collect_test_lines` (`priv`, L412), `walk_item` (`priv`, L415), `run_debt_audit` (`pub`, L444), `prod_only_fn` (`priv`, L647)
- **DebtAstVisitor<**: `new` (`priv`, L98), `check_file` (`priv`, L106), `check_item` (`priv`, L112), `check_block` (`priv`, L184)
- **Visit<**: `visit_expr_method_call` (`priv`, L193), `visit_item` (`priv`, L216)

#### `xtask/src/gen_arch_docs.rs` (4)

- **RingSortKey**: `from_str` (`priv`, L55), `display_name` (`priv`, L66)
- ***(Modulfunktionen)***: `generate_arch_docs_content` (`pub`, L78), `run_gen_arch_docs` (`pub`, L196)

#### `xtask/src/gen_feature_catalog.rs` (1)

- ***(Modulfunktionen)***: `run_gen_feature_catalog` (`pub`, L8)

#### `xtask/src/gen_prompter_data.rs` (5)

- ***(Modulfunktionen)***: `determine_size_class` (`priv`, L89), `count_crate_tests` (`priv`, L101), `get_git_head_short` (`priv`, L124), `derive_status_from_working_state` (`priv`, L136), `run` (`pub`, L165)

#### `xtask/src/generate_adr.rs` (4)

- ***(Modulfunktionen)***: `consolidate_decisions` (`pub`, L15), `run_generate_adr` (`pub`, L105), `generate_slug` (`priv`, L219), `get_today_iso` (`priv`, L240)

#### `xtask/src/generate_diagnostics.rs` (1)

- ***(Modulfunktionen)***: `run_generate_diagnostics` (`pub`, L19)

#### `xtask/src/generate_markers.rs` (6)

- ***(Modulfunktionen)***: `find_root_dir` (`pub`, L7), `render_marker_table` (`pub`, L83), `load_capabilities_from_file` (`pub`, L124), `run_generate_markers` (`pub`, L141), `run_check_marker_drift` (`pub`, L171), `normalize_text` (`priv`, L220)

#### `xtask/src/generators/sbom.rs` (1)

- ***(Modulfunktionen)***: `run_gen_sbom` (`pub`, L8)

#### `xtask/src/harness/blast_radius.rs` (6)

- ***(Modulfunktionen)***: `run_blast_radius` (`pub`, L8), `blast_radius_find_repo_root` (`priv`, L236), `blast_radius_get_changed_files_worktree` (`pub`, L251), `blast_radius_get_affected_crates` (`pub`, L324), `blast_radius_has_cycle` (`priv`, L413), `blast_radius_dfs_cycle` (`priv`, L425)

#### `xtask/src/harness/can_merge.rs` (1)

- ***(Modulfunktionen)***: `run_can_merge` (`pub`, L7)

#### `xtask/src/harness/claim_guard.rs` (3)

- ***(Modulfunktionen)***: `run_claim_guard` (`pub`, L6), `find_root_dir` (`priv`, L83), `get_changed_files` (`priv`, L92)

#### `xtask/src/harness/claims_in_pr.rs` (10)

- ***(Modulfunktionen)***: `claims_in_pr_required_headers` (`pub`, L29), `claims_in_pr_is_placeholder` (`pub`, L41), `claims_in_pr_parse_sections` (`pub`, L63), `claims_in_pr_extract_listed_files` (`pub`, L87), `run_claims_in_pr` (`pub`, L124), `claims_in_pr_emit` (`priv`, L352), `claims_in_pr_default_root` (`priv`, L375), `claims_in_pr_git_merge_base` (`priv`, L388), `claims_in_pr_get_changed_files` (`priv`, L410), `claims_in_pr_diff_has_new_test` (`priv`, L433)

#### `xtask/src/harness/codeowners_coverage.rs` (8)

- ***(Modulfunktionen)***: `run_codeowners_coverage` (`pub`, L39), `glob_to_regex_pattern` (`priv`, L185), `matches_glob_pattern` (`priv`, L216), `filtered_walkdir` (`priv`, L234), `is_valid_codeowners_pattern` (`priv`, L248), `codeowners_matches_glob` (`priv`, L271), `default_root` (`priv`, L319), `emit` (`priv`, L332)

#### `xtask/src/harness/dependency_gate.rs` (4)

- ***(Modulfunktionen)***: `run_dependency_gate` (`pub`, L6), `dependency_gate_find_repo_root` (`priv`, L110), `dependency_gate_get_changed_files` (`priv`, L125), `dependency_gate_get_commit_message` (`priv`, L143)

#### `xtask/src/harness/dependency_graph_audit.rs` (5)

- ***(Modulfunktionen)***: `build_graph_from_crates` (`pub`, L36), `detect_cycles` (`pub`, L121), `ring_to_u32` (`pub`, L187), `detect_ring_jumps` (`pub`, L198), `run_dependency_graph_audit` (`pub`, L253)

#### `xtask/src/harness/determinism_check.rs` (6)

- **DeterminismCheckAllowEntry**: `description` (`pub`, L34)
- ***(Modulfunktionen)***: `determinism_check_parse_args` (`pub`, L56), `determinism_check_get_ring01_crates` (`pub`, L106), `determinism_check_is_test_or_bench` (`pub`, L137), `determinism_check_scan_file` (`pub`, L153), `run_determinism_check` (`pub`, L252)

#### `xtask/src/harness/diff_budget.rs` (10)

- ***(Modulfunktionen)***: `run_diff_budget` (`pub`, L41), `diff_budget_format_split_recommendation` (`priv`, L194), `diff_budget_emit` (`priv`, L212), `diff_budget_default_root` (`priv`, L231), `diff_budget_git_merge_base` (`priv`, L244), `diff_budget_git_log_trailers` (`priv`, L266), `diff_budget_find_card_id` (`priv`, L283), `diff_budget_resolve_card_path` (`priv`, L288), `diff_budget_is_excluded` (`pub`, L297), `diff_budget_get_numstat` (`pub`, L319)

#### `xtask/src/harness/doc_truth.rs` (22)

- ***(Modulfunktionen)***: `run_doc_truth` (`pub`, L30), `doc_truth_find_root` (`pub`, L115), `doc_truth_target_files` (`priv`, L131), `doc_truth_check_paths_pub` (`pub`, L165), `doc_truth_is_path_candidate` (`priv`, L212), `doc_truth_path_exists` (`priv`, L240), `doc_truth_check_crates_pub` (`pub`, L280), `doc_truth_get_workspace_crates` (`priv`, L315), `doc_truth_check_commands_pub` (`pub`, L344), `doc_truth_get_xtask_commands` (`priv`, L395), `doc_truth_check_unsafe_islands_pub` (`pub`, L436), `doc_truth_get_capabilities_unsafe_islands` (`priv`, L494), `doc_truth_get_agents_unsafe_islands` (`priv`, L516), `doc_truth_get_workspace_crate_paths` (`priv`, L534), `doc_truth_get_actual_unsafe_crates` (`priv`, L553), `doc_truth_file_has_unsafe` (`priv`, L583), `doc_truth_is_cfg_test` (`priv`, L644)
- **Visit<**: `visit_item_mod` (`priv`, L604), `visit_item_fn` (`priv`, L613), `visit_item_impl` (`priv`, L625), `visit_item_trait` (`priv`, L632), `visit_expr_unsafe` (`priv`, L639)

#### `xtask/src/harness/doctrine_scan.rs` (1)

- ***(Modulfunktionen)***: `run_doctrine_scan` (`pub`, L30)

#### `xtask/src/harness/env_attest.rs` (5)

- ***(Modulfunktionen)***: `env_attest_read_toolchain_pin` (`pub`, L50), `env_attest_read_required_tools` (`pub`, L59), `env_attest_check_cmd` (`pub`, L68), `env_attest_execute` (`pub`, L97), `run_env_attest` (`pub`, L267)

#### `xtask/src/harness/explain.rs` (2)

- ***(Modulfunktionen)***: `run_explain` (`pub`, L7), `explain_find_repo_root` (`priv`, L266)

#### `xtask/src/harness/fast_diff_lint.rs` (5)

- ***(Modulfunktionen)***: `parse_unified_diff` (`pub`, L38), `extract_crate_name` (`priv`, L115), `is_test_context` (`priv`, L120), `lint_file` (`priv`, L146), `run_fast_diff_lint` (`pub`, L283)

#### `xtask/src/harness/flake_report.rs` (2)

- ***(Modulfunktionen)***: `run_flake_report` (`pub`, L8), `flake_report_find_repo_root` (`priv`, L165)

#### `xtask/src/harness/fuzz_smoke.rs` (2)

- ***(Modulfunktionen)***: `fuzz_smoke_get_touched_fuzz_crates` (`pub`, L26), `run_fuzz_smoke` (`pub`, L70)

#### `xtask/src/harness/gate_arch.rs` (7)

- ***(Modulfunktionen)***: `run_gate_arch` (`pub`, L25), `execute_xtask_subcmd` (`priv`, L216), `get_git_merge_base` (`priv`, L276), `get_changed_files` (`priv`, L291), `file_contains_unsafe` (`priv`, L324), `detect_repo_root` (`priv`, L332), `parse_summary_from_output` (`priv`, L345)

#### `xtask/src/harness/gate_guard.rs` (5)

- ***(Modulfunktionen)***: `run_gate_guard` (`pub`, L24), `execute_xtask_subcmd` (`priv`, L172), `get_git_merge_base` (`priv`, L232), `detect_repo_root` (`priv`, L247), `parse_summary_from_output` (`priv`, L260)

#### `xtask/src/harness/gate_integrity.rs` (7)

- ***(Modulfunktionen)***: `run_gate_integrity` (`pub`, L25), `execute_xtask_subcmd` (`priv`, L219), `get_git_merge_base` (`priv`, L279), `get_changed_files` (`priv`, L294), `file_contains_pattern` (`priv`, L327), `detect_repo_root` (`priv`, L335), `parse_summary_from_output` (`priv`, L348)

#### `xtask/src/harness/gate_weakening.rs` (5)

- ***(Modulfunktionen)***: `gate_weakening_parse_args` (`pub`, L16), `gate_weakening_get_commits` (`pub`, L87), `gate_weakening_get_commit_trailers` (`pub`, L113), `gate_weakening_analyze_diff` (`pub`, L145), `run_gate_weakening` (`pub`, L497)

#### `xtask/src/harness/gates_catalog_sync.rs` (7)

- ***(Modulfunktionen)***: `run_gates_catalog_sync` (`pub`, L59), `parse_harness_phases` (`priv`, L256), `parse_verdict_required` (`priv`, L318), `normalize_cmd_name` (`priv`, L364), `check_justfile_and_workflows_read_only` (`priv`, L372), `default_root` (`priv`, L389), `emit` (`priv`, L402)

#### `xtask/src/harness/jules.rs` (5)

- ***(Modulfunktionen)***: `jules_facade_find_root` (`pub`, L46), `jules_facade_get_session_owner` (`pub`, L58), `jules_facade_run_step` (`pub`, L71), `jules_facade_execute_phase` (`pub`, L180), `run_jules` (`pub`, L538)

#### `xtask/src/harness/jules_dispatch.rs` (8)

- **trait JulesHttpTransport**: `post` (`priv`, L8)
- **JulesHttpTransport for CurlHttpTransport**: `post` (`priv`, L14)
- ***(Modulfunktionen)***: `jules_dispatch_redact_key` (`pub`, L43), `jules_dispatch_find_root` (`pub`, L51), `jules_dispatch_build_payload` (`pub`, L63), `task_card_lint_file_local` (`priv`, L113), `jules_dispatch_execute` (`pub`, L136), `run_jules_dispatch` (`pub`, L232)

#### `xtask/src/harness/ledger.rs` (5)

- ***(Modulfunktionen)***: `ledger_compute_tree_hash` (`pub`, L35), `ledger_get_head_commit` (`pub`, L90), `ledger_write` (`pub`, L106), `ledger_verify` (`pub`, L176), `run_ledger` (`pub`, L200)

#### `xtask/src/harness/lock_audit.rs` (4)

- ***(Modulfunktionen)***: `lock_audit_get_touched_crates` (`pub`, L26), `lock_audit_scan_file_fallback` (`pub`, L51), `lock_audit_check_agents_md` (`pub`, L86), `run_lock_audit` (`pub`, L125)

#### `xtask/src/harness/loop_guard.rs` (11)

- ***(Modulfunktionen)***: `loop_guard_normalize_error` (`pub`, L35), `loop_guard_hash_error` (`pub`, L79), `loop_guard_get_state_path` (`pub`, L85), `loop_guard_load_state` (`pub`, L89), `loop_guard_save_state` (`pub`, L98), `loop_guard_record_text` (`pub`, L108), `loop_guard_record` (`pub`, L151), `loop_guard_check` (`pub`, L166), `loop_guard_stop` (`pub`, L190), `loop_guard_reset` (`pub`, L237), `run_loop_guard` (`pub`, L250)

#### `xtask/src/harness/mod.rs` (3)

- ***(Modulfunktionen)***: `dispatch_with_builtin` (`pub`, L7), `run_harness_list` (`priv`, L15), `run_harness_help` (`priv`, L46)

#### `xtask/src/harness/mutants_diff.rs` (3)

- ***(Modulfunktionen)***: `mutants_diff_get_tier1_touched_crates` (`pub`, L26), `mutants_diff_get_expected_score` (`pub`, L69), `run_mutants_diff` (`pub`, L91)

#### `xtask/src/harness/orphan_symbols.rs` (11)

- **DeclVisitor<**: `new` (`priv`, L62), `is_vis_public_or_restricted` (`priv`, L72), `collect_items` (`priv`, L76)
- **IdentVisitor**: `new` (`priv`, L185), `record_ident` (`priv`, L193), `count_tokens` (`priv`, L205)
- **Visit<**: `visit_ident` (`priv`, L219), `visit_macro` (`priv`, L224)
- ***(Modulfunktionen)***: `orphan_symbols_default_root` (`priv`, L230), `scan_text_references` (`priv`, L245), `run_orphan_symbols` (`pub`, L288)

#### `xtask/src/harness/plan_lint.rs` (6)

- **trait PlanLintHttpTransport**: `approve_plan` (`priv`, L18)
- **PlanLintHttpTransport for PlanLintCurlTransport**: `approve_plan` (`priv`, L30)
- ***(Modulfunktionen)***: `plan_lint_find_root` (`pub`, L78), `plan_lint_check_plan_text` (`pub`, L90), `plan_lint_process_session` (`pub`, L248), `run_plan_lint` (`pub`, L389)

#### `xtask/src/harness/postmortem.rs` (2)

- ***(Modulfunktionen)***: `run_postmortem` (`pub`, L7), `postmortem_find_repo_root` (`priv`, L263)

#### `xtask/src/harness/protected_paths.rs` (11)

- ***(Modulfunktionen)***: `protected_paths_glob_to_regex` (`pub`, L22), `protected_paths_parse_args` (`pub`, L61), `check_shallow_repository` (`pub`, L134), `check_commit_valid` (`pub`, L153), `protected_paths_get_changed_files` (`pub`, L170), `protected_paths_get_commits` (`pub`, L228), `protected_paths_get_commit_trailers` (`pub`, L254), `parse_adr_status` (`pub`, L286), `is_status_accepted` (`pub`, L313), `validate_adr_on_base_commit` (`pub`, L325), `run_protected_paths` (`pub`, L396)

#### `xtask/src/harness/ratchet.rs` (8)

- ***(Modulfunktionen)***: `ratchet_parse_args` (`pub`, L21), `ratchet_count_legacy_unwrap` (`pub`, L79), `ratchet_measure_repo` (`pub`, L119), `run_ratchet` (`pub`, L256)
- **syn::visit::Visit<**: `visit_item_mod` (`priv`, L182), `visit_attribute` (`priv`, L196), `visit_expr_method_call` (`priv`, L206), `visit_expr_unsafe` (`priv`, L215)

#### `xtask/src/harness/registry_check.rs` (9)

- ***(Modulfunktionen)***: `run_registry_check` (`pub`, L37), `registry_check_parse_args` (`priv`, L213), `registry_check_detect_root` (`priv`, L250), `registry_check_load_registry` (`priv`, L269), `registry_check_extract_main_commands` (`priv`, L323), `registry_check_extract_justfile_commands` (`priv`, L345), `registry_check_extract_workflow_commands` (`priv`, L361), `registry_check_extract_harness_commands` (`priv`, L390), `registry_check_finish` (`priv`, L409)

#### `xtask/src/harness/repo_doctor.rs` (6)

- ***(Modulfunktionen)***: `run_repo_doctor` (`pub`, L23), `find_repo_root` (`priv`, L130), `run_git_status_check` (`priv`, L145), `run_workspace_verify_check` (`priv`, L235), `run_env_check` (`priv`, L287), `run_commit_health_check` (`priv`, L334)

#### `xtask/src/harness/risk.rs` (3)

- ***(Modulfunktionen)***: `run_risk` (`pub`, L17), `risk_find_repo_root` (`priv`, L182), `risk_get_changed_files` (`priv`, L197)

#### `xtask/src/harness/scan_common.rs` (14)

- ***(Modulfunktionen)***: `classify_file` (`pub`, L35), `is_generated` (`priv`, L49), `is_test_file_path` (`priv`, L57), `normalize_path` (`priv`, L82), `has_cfg_test_attr` (`pub`, L87), `get_module_dir` (`pub`, L134), `resolve_external_test_modules` (`pub`, L153), `find_cfg_test_mods_in_items` (`priv`, L162), `run_scan_common` (`pub`, L196)
- **TestItemStripper**: `count_test_fns_in_item` (`priv`, L224), `should_remove_item` (`priv`, L242), `filter_items` (`priv`, L273)
- **VisitMut for TestItemStripper**: `visit_file_mut` (`priv`, L287), `visit_item_mod_mut` (`priv`, L291)

#### `xtask/src/harness/scope_guard.rs` (10)

- ***(Modulfunktionen)***: `scope_guard_glob_to_regex` (`pub`, L36), `scope_guard_matches_glob` (`pub`, L70), `scope_guard_find_card_id` (`pub`, L79), `scope_guard_resolve_card_path` (`pub`, L84), `run_scope_guard` (`pub`, L93), `scope_guard_emit` (`priv`, L285), `scope_guard_default_root` (`priv`, L304), `scope_guard_git_merge_base` (`priv`, L317), `scope_guard_git_log_trailers` (`priv`, L339), `scope_guard_get_changed_files` (`priv`, L356)

#### `xtask/src/harness/search.rs` (1)

- ***(Modulfunktionen)***: `run_search` (`pub`, L17)

#### `xtask/src/harness/session_report.rs` (3)

- ***(Modulfunktionen)***: `session_report_get_diff_files` (`pub`, L36), `session_report_generate` (`pub`, L55), `run_session_report` (`pub`, L187)

#### `xtask/src/harness/stub_impls.rs` (12)

- **SeverityLevel**: `as_str` (`pub`, L18)
- **Visit<**: `visit_macro` (`priv`, L52), `visit_item_mod` (`priv`, L149), `visit_item_impl` (`priv`, L160)
- ***(Modulfunktionen)***: `find_stub_macro` (`priv`, L70), `is_non_unit_return` (`priv`, L76), `is_cfg_test` (`priv`, L89), `match_trivial_expr` (`priv`, L100), `find_root_dir` (`priv`, L265), `collect_rs_files` (`priv`, L287), `run_stub_impls` (`pub`, L341), `run_stub_impls_with_writer` (`pub`, L345)

#### `xtask/src/harness/symbol_exists.rs` (15)

- ***(Modulfunktionen)***: `symbol_exists_lev_distance` (`pub`, L51), `symbol_exists_normalize_crate_name` (`pub`, L74), `symbol_exists_find_crate_path` (`pub`, L78), `run_symbol_exists` (`pub`, L104), `join_mod_path` (`priv`, L616), `format_vis` (`priv`, L624), `symbol_exists_emit` (`priv`, L632), `symbol_exists_default_root` (`priv`, L651)
- **SymbolExistsCrateFinder**: `new` (`pub`, L247), `scan_crate` (`pub`, L257), `parse_module_file` (`pub`, L333), `register_item` (`priv`, L363), `register_use_tree` (`priv`, L561), `lookup_symbol` (`pub`, L593), `get_similar_candidates` (`pub`, L600)

#### `xtask/src/harness/task_card.rs` (7)

- **TaskCard**: `get_crate` (`pub`, L37)
- ***(Modulfunktionen)***: `task_card_find_root` (`pub`, L57), `task_card_get_protected_paths` (`pub`, L69), `task_card_run_new` (`pub`, L81), `task_card_lint_file` (`pub`, L138), `task_card_lint_pr` (`pub`, L410), `run_task_card` (`pub`, L468)

#### `xtask/src/harness/test_integrity.rs` (16)

- **Visit<**: `visit_macro` (`priv`, L37), `visit_expr_call` (`priv`, L55), `visit_expr_method_call` (`priv`, L67), `visit_expr` (`priv`, L75)
- ***(Modulfunktionen)***: `test_integrity_is_assertion_helper` (`pub`, L83), `test_integrity_is_test_attr` (`pub`, L91), `test_integrity_has_should_panic` (`pub`, L102), `test_integrity_has_ignore_attr` (`pub`, L111), `run_test_integrity` (`pub`, L118), `test_integrity_check_fn` (`pub`, L285), `test_integrity_emit` (`priv`, L377), `test_integrity_default_root` (`priv`, L396), `test_integrity_git_merge_base` (`priv`, L409), `test_integrity_get_changed_files` (`priv`, L431), `test_integrity_get_deleted_files` (`priv`, L454), `test_integrity_get_added_lines_for_file` (`priv`, L477)

#### `xtask/src/harness/unsafe_audit.rs` (4)

- **UnsafeCrateInfo**: `name_ref` (`pub`, L38)
- ***(Modulfunktionen)***: `unsafe_audit_get_islands_and_crates` (`pub`, L43), `unsafe_audit_scan_file` (`pub`, L85), `run_unsafe_audit` (`pub`, L174)

#### `xtask/src/harness/unwired_ports.rs` (11)

- **CodeVisitor<**: `new` (`priv`, L62)
- **Visit<**: `visit_item_trait` (`priv`, L74), `visit_item_impl` (`priv`, L85), `visit_item_struct` (`priv`, L102), `visit_item_enum` (`priv`, L110), `visit_item_type` (`priv`, L118), `visit_item_union` (`priv`, L126)
- ***(Modulfunktionen)***: `extract_type_name` (`priv`, L135), `find_workspace_crates` (`priv`, L162), `collect_rs_files` (`priv`, L236), `run_unwired_ports` (`pub`, L253)

#### `xtask/src/harness/verdict.rs` (2)

- ***(Modulfunktionen)***: `run_verdict` (`pub`, L56), `verdict_find_artifact` (`priv`, L366)

#### `xtask/src/harness/wal_replay_verify.rs` (2)

- ***(Modulfunktionen)***: `run_wal_replay_verify` (`pub`, L24), `wal_replay_is_applicable` (`pub`, L369)

#### `xtask/src/hotspot_report.rs` (2)

- ***(Modulfunktionen)***: `run_hotspot_report_impl` (`pub`, L22), `run_hotspot_report` (`pub`, L132)

#### `xtask/src/init_audit_fix.rs` (1)

- ***(Modulfunktionen)***: `run_init_audit_fix` (`pub`, L9)

#### `xtask/src/jules_preflight.rs` (4)

- ***(Modulfunktionen)***: `get_changed_rs_files_from_git_diff` (`priv`, L19), `derive_crate_from_changed_files` (`priv`, L55), `check_no_active_claim_conflict` (`pub`, L68), `run_jules_preflight` (`pub`, L183)

#### `xtask/src/jules_submit_gate.rs` (11)

- ***(Modulfunktionen)***: `default_blocking` (`priv`, L20), `get_ledger_path` (`pub`, L33), `submit_gate_tree_hash` (`pub`, L38), `submit_gate_tree_hash_in_dir` (`pub`, L42), `iso8601_now` (`pub`, L91), `write_ledger` (`pub`, L95), `read_ledger` (`pub`, L105), `verify_ledger_matches_worktree` (`pub`, L111), `run_full_gate_chain` (`pub`, L142), `run_pre_push` (`pub`, L358), `run_jules_submit_gate` (`pub`, L369)

#### `xtask/src/lib.rs` (50)

- ***(Modulfunktionen)***: `find_root_dir` (`pub`, L104), `find_root_dir_uncached` (`priv`, L125), `chrono_or_today_with_tags` (`pub`, L175), `chrono_or_today` (`pub`, L213), `load_capabilities_manifest` (`pub`, L245), `scan_tags` (`pub`, L293), `calculate_crate_loc` (`pub`, L405), `compute_crate_layers` (`pub`, L431), `get_layer` (`priv`, L441), `get_workspace_crates` (`pub`, L504), `get_workspace_crates_from_root` (`pub`, L522), `render_updated_markdown` (`pub`, L652), `update_markdown_section` (`pub`, L707), `generate_session_continuity_section` (`pub`, L724), `generate_session_continuity_section_from_root` (`pub`, L729), `tokenize_title` (`pub`, L749), `jaccard_similarity` (`pub`, L758), `extract_scope` (`pub`, L774), `generate_full_working_state` (`pub`, L785), `generate_full_working_state_from_root` (`pub`, L790), `generate_invariants_table_section` (`pub`, L837), `generate_changelog` (`pub`, L852), `generate_ai_tags_section` (`pub`, L881), `generate_dag_topology_section` (`pub`, L901), `format_loc` (`pub`, L927), `generate_crate_inventory_section` (`pub`, L935), `severity_weight` (`pub`, L957), `run_sync_docs` (`pub`, L967), `check_readme_crate_count` (`pub`, L997), `check_crate_agents` (`pub`, L1007), `check_no_orphan_adr_files` (`pub`, L1019), `check_adr_consistency_dir` (`pub`, L1042), `check_adr_consistency` (`pub`, L1066), `get_changed_rs_files_from_git_diff` (`pub`, L1080), `get_git_file_last_modified` (`pub`, L1100), `levenshtein_distance` (`pub`, L1116), `split_into_sentences` (`pub`, L1160), `sentences_are_similar` (`pub`, L1196), `check_sentence_overlap` (`pub`, L1208), `run_check_consistency` (`pub`, L1223), `validate_working_state_tag_date` (`pub`, L1252), `validate_tags_items` (`pub`, L1266), `run_validate_tags` (`pub`, L1277), `check_dag_layer_violations` (`pub`, L1299), `run_check_dag` (`pub`, L1322), `run_check_review_coverage` (`pub`, L1360), `extract_crate_name` (`pub`, L1472), `parse_context_tag_args` (`pub`, L1487), `filter_tags` (`pub`, L1501), `run_context_tags` (`pub`, L1541)

#### `xtask/src/lint_unsafe_slice_bounds.rs` (13)

- **SimdFnVisitor<**: `new` (`priv`, L32), `check_item_fn` (`priv`, L39)
- **Visit<**: `visit_item_fn` (`priv`, L103), `visit_expr_method_call` (`priv`, L117), `visit_expr_call` (`priv`, L133), `visit_macro` (`priv`, L145), `visit_expr_if` (`priv`, L159), `visit_expr_binary` (`priv`, L167)
- ***(Modulfunktionen)***: `is_simd_load_intrinsic` (`priv`, L175), `extract_slice_elem_type` (`priv`, L182), `expr_contains_length_check` (`priv`, L191), `lint_code_str` (`pub`, L196), `run_lint_unsafe_slice_bounds` (`pub`, L204)

#### `xtask/src/loom_run.rs` (2)

- ***(Modulfunktionen)***: `discover_loom_tests` (`pub`, L22), `run_loom` (`pub`, L62)

#### `xtask/src/main.rs` (1)

- ***(Modulfunktionen)***: `main` (`priv`, L7)

#### `xtask/src/migrate_docid_128.rs` (4)

- ***(Modulfunktionen)***: `default_v1_version` (`priv`, L48), `derive_doc_id_128_hex` (`pub`, L53), `run_migration` (`pub`, L70), `run_cli` (`pub`, L137)

#### `xtask/src/panic_inventory.rs` (7)

- **PanicKind**: `as_str` (`pub`, L22)
- ***(Modulfunktionen)***: `find_root_dir` (`priv`, L42), `load_baseline` (`priv`, L64), `scan_rs_file` (`priv`, L84), `collect_rs_files` (`priv`, L167), `run_panic_inventory` (`pub`, L194), `run_panic_inventory_in_root` (`pub`, L203)

#### `xtask/src/post_merge_report.rs` (1)

- ***(Modulfunktionen)***: `run_post_merge_report` (`pub`, L12)

#### `xtask/src/proof/bench_trend.rs` (1)

- ***(Modulfunktionen)***: `run_bench_trend` (`pub`, L6)

#### `xtask/src/proof/forensic_test.rs` (1)

- ***(Modulfunktionen)***: `run_forensic_test` (`pub`, L8)

#### `xtask/src/py_test.rs` (4)

- ***(Modulfunktionen)***: `build_py_import_check_cmd` (`pub`, L5), `build_maturin_develop_cmd` (`pub`, L16), `build_pytest_cmd` (`pub`, L25), `run_py_test` (`pub`, L42)

#### `xtask/src/record_mutation_score.rs` (1)

- ***(Modulfunktionen)***: `run_record_mutation_score` (`pub`, L20)

#### `xtask/src/reproducible_build.rs` (5)

- ***(Modulfunktionen)***: `run_reproducible_build_check` (`pub`, L14), `execute_build` (`pub`, L112), `collect_executable_hashes` (`pub`, L143), `is_executable_file` (`pub`, L164), `compute_sha256` (`pub`, L199)

#### `xtask/src/security_scan.rs` (6)

- **SecurityPattern**: `as_str` (`pub`, L20)
- ***(Modulfunktionen)***: `find_root_dir` (`priv`, L37), `collect_rs_files` (`priv`, L59), `scan_rs_file` (`priv`, L92), `run_security_scan` (`pub`, L216), `run_security_scan_in_root` (`pub`, L224)

#### `xtask/src/session_history.rs` (10)

- **Default for RotationConfig**: `default` (`priv`, L32)
- **SessionHistoryEntry**: `new` (`pub`, L62), `with_crate` (`pub`, L74), `with_issue` (`pub`, L80), `with_details` (`pub`, L86)
- ***(Modulfunktionen)***: `find_root_dir` (`pub`, L93), `default_session_history_path` (`pub`, L108), `append_entry` (`pub`, L114), `append_entry_with_config` (`pub`, L121), `rotate_logs` (`priv`, L148)

#### `xtask/src/session_init.rs` (6)

- ***(Modulfunktionen)***: `generate_session_hash` (`pub`, L19), `count_open_blockers` (`priv`, L24), `get_last_commit` (`priv`, L50), `get_last_merge` (`priv`, L61), `get_prompter_distance` (`priv`, L73), `run_session_init` (`pub`, L113)

#### `xtask/src/shell_commit_audit.rs` (4)

- ***(Modulfunktionen)***: `parse_stat_line` (`priv`, L20), `ensure_unshallow` (`priv`, L45), `run_shell_commit_audit_impl` (`pub`, L56), `run_shell_commit_audit` (`pub`, L178)

#### `xtask/src/tag_health.rs` (3)

- ***(Modulfunktionen)***: `parse_iso_datetime` (`priv`, L25), `run_tag_health_impl` (`pub`, L41), `run_tag_health` (`pub`, L166)

#### `xtask/src/validate_pr_checklist.rs` (6)

- **CheckStatus**: `icon` (`priv`, L25)
- ***(Modulfunktionen)***: `run_validate_pr_checklist` (`pub`, L38), `get_git_diff` (`priv`, L318), `get_diff_files` (`priv`, L342), `check_unsafe_without_safety` (`priv`, L359), `check_silent_io_in_diff` (`priv`, L410)

#### `xtask/src/veto_deadline_gate.rs` (6)

- **VetoDeadlineEntry**: `is_file_affected` (`pub`, L22)
- ***(Modulfunktionen)***: `parse_veto_deadlines` (`pub`, L43), `check_veto_deadlines_from_content_scoped` (`pub`, L134), `check_veto_deadlines_from_content` (`pub`, L227), `get_git_changed_files` (`priv`, L234), `run_check_veto_deadlines` (`pub`, L266)

#### `xtask/src/workspace_index.rs` (17)

- **WorkspaceIndex**: `build` (`pub`, L72), `build_from_root` (`pub`, L77)
- ***(Modulfunktionen)***: `extract_crate_name` (`priv`, L225), `analyze_file` (`priv`, L237), `scan_text_refs` (`priv`, L430), `fetch_cargo_deps` (`priv`, L472)
- **AstIndexVisitor<**: `record_ident` (`priv`, L281), `line_of_span` (`priv`, L291)
- **Visit<**: `visit_ident` (`priv`, L297), `visit_item_fn` (`priv`, L302), `visit_item_struct` (`priv`, L316), `visit_item_enum` (`priv`, L330), `visit_item_trait` (`priv`, L344), `visit_item_type` (`priv`, L358), `visit_item_const` (`priv`, L372), `visit_item_impl` (`priv`, L386), `visit_item_macro` (`priv`, L419)

#### `xtask/src/workspace_verify.rs` (11)

- **Default for WorkspaceVerifyConfig**: `default` (`priv`, L32)
- ***(Modulfunktionen)***: `find_root_dir` (`priv`, L120), `get_bottom_up_crate_order_from_metadata` (`pub`, L142), `compute_depth` (`priv`, L164), `get_bottom_up_crate_order` (`pub`, L201), `parse_cargo_json_diagnostics` (`priv`, L219), `run_workspace_verify` (`pub`, L257), `run_workspace_verify_in_root` (`pub`, L265), `write_summary_md` (`priv`, L419), `write_diagnostics_jsonl` (`priv`, L445), `write_fehlerbericht_md` (`priv`, L459)

#### `xtask/src/ws_cache.rs` (3)

- ***(Modulfunktionen)***: `workspace_rs_files` (`pub`, L12), `workspace_rs_files_from_root` (`pub`, L18), `read_cached` (`pub`, L59)

### xtask-heavy

- **Pfad:** `xtask-heavy` | **Dateien:** 3 | **Produktiv-Funktionen:** 8 (`5` `pub`, `3` intern) | **Tests:** 4

#### `xtask-heavy/src/bench_gate.rs` (1)

- ***(Modulfunktionen)***: `run_bench_gate` (`pub`, L9)

#### `xtask-heavy/src/check_bandit_latency_budget.rs` (6)

- ***(Modulfunktionen)***: `get_benchmark_runs` (`priv`, L24), `get_benchmark_iterations` (`priv`, L33), `calculate_p95` (`pub`, L42), `calculate_median` (`pub`, L54), `evaluate_p95_runs` (`pub`, L69), `check_bandit_latency_budget` (`pub`, L97)

#### `xtask-heavy/src/main.rs` (1)

- ***(Modulfunktionen)***: `main` (`priv`, L4)
