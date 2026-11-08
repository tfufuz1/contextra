# Anti-Pattern & Invariant Violation Diagnostic Report

> **Werkzeug:** `pattern_diagnostic.py`  
> **Erstellungszeit:** `2026-11-07T23:05:42.298377+00:00`  
> **Status/Befunde:** `12311 Befunde erfasst`  

## Beschreibung
Comprehensive audit of Zero-Panic (unwrap/expect), Determinism (Clock/Rng/Hash), Memory Mapping (mmap), Direct I/O, and §6.4 Forbidden Wording invariants.

## Metriken-Übersicht
| Metrik | Wert |
|---|---|
| `total_findings` | `12311` |
| `production_violations` | `1638` |
| `test_violations` | `10673` |
| `category_zero_panic` | `11603` |
| `category_determinism` | `169` |
| `category_memory_mapping` | `110` |
| `category_direct_io` | `305` |
| `category_forbidden_wording` | `124` |

## Kategorien-Übersicht
| Kategorie | Funde Gesamt | Beschreibung |
|---|---|---|
| `ZERO_PANIC` | 11603 | `unwrap()`, `expect()`, `panic!()` in Code |
| `DETERMINISM` | 169 | Zeit/Zufall/Hasher ohne Ports (P28) |
| `MEMORY_MAPPING` | 110 | Verbotene mmap-Nutzung (D-94) |
| `DIRECT_IO` | 305 | Direkte std::fs Umgehung von Vfs |
| `FORBIDDEN_WORDING` | 124 | Garantieversprechen / DSGVO-Labels (§6.4) |

## Detaillierte Befunde
| Crate | Datei | Zeile | Art / Beschreibung |
|---|---|---|---|
| `contextra` | `crates/contextra/src/builder.rs` | `498` | Invocation of panic!() macro |
| `contextra` | `crates/contextra/src/builder.rs` | `516` | std::fs usage bypasses Vfs port |
| `contextra` | `crates/contextra/src/builder.rs` | `527` | std::fs usage bypasses Vfs port |
| `contextra` | `crates/contextra/src/builder.rs` | `537` | std::fs usage bypasses Vfs port |
| `contextra` | `crates/contextra/src/builder.rs` | `546` | std::fs usage bypasses Vfs port |
| `contextra` | `crates/contextra/src/collection_profile.rs` | `149` | Forbidden wording (§6.4): 'garantiert' |
| `contextra` | `crates/contextra/src/lib.rs` | `33` | Forbidden wording (§6.4): 'garantiert' |
| `contextra` | `crates/contextra/src/lib.rs` | `187` | std::fs usage bypasses Vfs port |
| `contextra` | `crates/contextra/src/lib.rs` | `205` | std::fs usage bypasses Vfs port |
| `contextra` | `crates/contextra/tests/agent_memory_facade.rs` | `37` | Call to .expect() in code |
| `contextra` | `crates/contextra/tests/agent_memory_facade.rs` | `45` | Call to .expect() in code |
| `contextra` | `crates/contextra/tests/agent_memory_facade.rs` | `57` | Call to .expect() in code |
| `contextra` | `crates/contextra/tests/agent_memory_facade.rs` | `62` | Call to .expect() in code |
| `contextra` | `crates/contextra/tests/agent_memory_facade.rs` | `70` | Call to .expect() in code |
| `contextra` | `crates/contextra/tests/agent_memory_facade.rs` | `83` | Call to .expect() in code |
| `contextra` | `crates/contextra/tests/agent_memory_facade.rs` | `86` | Call to .expect() in code |
| `contextra` | `crates/contextra/tests/agent_memory_facade.rs` | `92` | Call to .expect() in code |
| `contextra` | `crates/contextra/tests/agent_memory_hyperedges.rs` | `31` | Call to .expect() in code |
| `contextra` | `crates/contextra/tests/agent_memory_hyperedges.rs` | `39` | Call to .expect() in code |
| `contextra` | `crates/contextra/tests/agent_memory_hyperedges.rs` | `53` | Call to .expect() in code |
| `contextra` | `crates/contextra/tests/agent_memory_hyperedges.rs` | `57` | Call to .expect() in code |
| `contextra` | `crates/contextra/tests/agent_memory_hyperedges.rs` | `61` | Call to .expect() in code |
| `contextra` | `crates/contextra/tests/agent_memory_hyperedges.rs` | `67` | Call to .expect() in code |
| `contextra` | `crates/contextra/tests/agent_memory_hyperedges.rs` | `73` | Call to .expect() in code |
| `contextra` | `crates/contextra/tests/agent_memory_hyperedges.rs` | `87` | Call to .expect() in code |
| `contextra` | `crates/contextra/tests/agent_memory_hyperedges.rs` | `107` | Call to .expect() in code |
| `contextra` | `crates/contextra/tests/agent_memory_hyperedges.rs` | `108` | Call to .expect() in code |
| `contextra` | `crates/contextra/tests/agent_memory_hyperedges.rs` | `142` | Call to .expect() in code |
| `contextra` | `crates/contextra/tests/agent_memory_hyperedges.rs` | `164` | Call to .expect() in code |
| `contextra` | `crates/contextra/tests/agent_memory_hyperedges.rs` | `165` | Call to .expect() in code |
| `contextra` | `crates/contextra/tests/agent_memory_hyperedges.rs` | `171` | Call to .expect() in code |
| `contextra` | `crates/contextra/tests/deletion_chain_e2e.rs` | `96` | Call to .expect() in code |
| `contextra` | `crates/contextra/tests/deletion_chain_e2e.rs` | `171` | Call to .expect() in code |
| `contextra` | `crates/contextra/tests/deletion_chain_e2e.rs` | `224` | Invocation of panic!() macro |
| `contextra` | `crates/contextra/tests/deletion_chain_e2e.rs` | `242` | Call to .unwrap() in code |
| `contextra` | `crates/contextra/tests/deletion_chain_e2e.rs` | `259` | Call to .expect() in code |
| `contextra` | `crates/contextra/tests/deletion_chain_e2e.rs` | `280` | Call to .expect() in code |
| `contextra` | `crates/contextra/tests/deletion_chain_e2e.rs` | `326` | Call to .expect() in code |
| `contextra` | `crates/contextra/tests/deletion_chain_e2e.rs` | `343` | Call to .expect() in code |
| `contextra` | `crates/contextra/tests/deletion_chain_e2e.rs` | `393` | Invocation of panic!() macro |
| `contextra` | `crates/contextra/tests/deletion_chain_e2e.rs` | `400` | Call to .unwrap() in code |
| `contextra` | `crates/contextra/tests/deletion_chain_e2e.rs` | `437` | Call to .expect() in code |
| `contextra` | `crates/contextra/tests/deletion_chain_e2e.rs` | `453` | Call to .expect() in code |
| `contextra` | `crates/contextra/tests/deletion_chain_e2e.rs` | `505` | Call to .expect() in code |
| `contextra` | `crates/contextra/tests/deletion_chain_e2e.rs` | `545` | Call to .expect() in code |
| `contextra` | `crates/contextra/tests/deletion_e2e.rs` | `43` | Call to .expect() in code |
| `contextra` | `crates/contextra/tests/deletion_e2e.rs` | `59` | Call to .expect() in code |
| `contextra` | `crates/contextra/tests/deletion_e2e.rs` | `91` | Call to .expect() in code |
| `contextra` | `crates/contextra/tests/encryption_builder_test.rs` | `56` | Invocation of panic!() macro |
| `contextra` | `crates/contextra/tests/encryption_builder_test.rs` | `68` | std::fs usage bypasses Vfs port |
| `contextra` | `crates/contextra/tests/encryption_builder_test.rs` | `79` | std::fs usage bypasses Vfs port |
| `contextra` | `crates/contextra/tests/encryption_builder_test.rs` | `91` | Call to .unwrap() in code |
| `contextra` | `crates/contextra/tests/encryption_builder_test.rs` | `108` | std::fs usage bypasses Vfs port |
| `contextra` | `crates/contextra/tests/encryption_builder_test.rs` | `119` | Call to .unwrap() in code |
| `contextra` | `crates/contextra/tests/encryption_builder_test.rs` | `126` | std::fs usage bypasses Vfs port |
| `contextra` | `crates/contextra/tests/encryption_builder_test.rs` | `136` | std::fs usage bypasses Vfs port |
| `contextra` | `crates/contextra/tests/encryption_builder_test.rs` | `146` | std::fs usage bypasses Vfs port |
| `contextra` | `crates/contextra/tests/license_gate_always_enforced.rs` | `22` | SystemTime::now() violates P28 (use Clock port) |
| `contextra` | `crates/contextra/tests/license_gate_always_enforced.rs` | `27` | std::fs usage bypasses Vfs port |
| `contextra` | `crates/contextra/tests/license_gate_always_enforced.rs` | `29` | Call to .unwrap() in code |
| `contextra` | `crates/contextra/tests/license_gate_always_enforced.rs` | `32` | std::fs usage bypasses Vfs port |
| `contextra` | `crates/contextra/tests/license_gate_always_enforced.rs` | `42` | SystemTime::now() violates P28 (use Clock port) |
| `contextra` | `crates/contextra/tests/license_gate_always_enforced.rs` | `47` | std::fs usage bypasses Vfs port |
| `contextra` | `crates/contextra/tests/license_gate_always_enforced.rs` | `56` | Call to .unwrap() in code |
| `contextra` | `crates/contextra/tests/license_gate_always_enforced.rs` | `64` | Invocation of panic!() macro |
| `contextra` | `crates/contextra/tests/license_gate_always_enforced.rs` | `67` | std::fs usage bypasses Vfs port |
| `contextra` | `crates/contextra/tests/license_gate_always_enforced.rs` | `77` | SystemTime::now() violates P28 (use Clock port) |
| `contextra` | `crates/contextra/tests/license_gate_always_enforced.rs` | `82` | std::fs usage bypasses Vfs port |
| `contextra` | `crates/contextra/tests/license_gate_always_enforced.rs` | `96` | Call to .expect() in code |
| `contextra` | `crates/contextra/tests/license_gate_always_enforced.rs` | `105` | std::fs usage bypasses Vfs port |
| `contextra` | `crates/contextra/tests/license_gate_always_enforced.rs` | `115` | SystemTime::now() violates P28 (use Clock port) |
| `contextra` | `crates/contextra/tests/license_gate_always_enforced.rs` | `120` | std::fs usage bypasses Vfs port |
| `contextra` | `crates/contextra/tests/license_gate_always_enforced.rs` | `137` | Call to .unwrap() in code |
| `contextra` | `crates/contextra/tests/license_gate_always_enforced.rs` | `145` | Invocation of panic!() macro |
| `contextra` | `crates/contextra/tests/license_gate_always_enforced.rs` | `148` | std::fs usage bypasses Vfs port |
| `contextra` | `crates/contextra/tests/license_gate_always_enforced.rs` | `158` | SystemTime::now() violates P28 (use Clock port) |
| `contextra` | `crates/contextra/tests/license_gate_always_enforced.rs` | `163` | std::fs usage bypasses Vfs port |
| `contextra` | `crates/contextra/tests/license_gate_always_enforced.rs` | `180` | Call to .unwrap() in code |
| `contextra` | `crates/contextra/tests/license_gate_always_enforced.rs` | `188` | Invocation of panic!() macro |
| `contextra` | `crates/contextra/tests/license_gate_always_enforced.rs` | `191` | std::fs usage bypasses Vfs port |
| `contextra` | `crates/contextra/tests/license_gate_always_enforced.rs` | `201` | SystemTime::now() violates P28 (use Clock port) |
| `contextra` | `crates/contextra/tests/license_gate_always_enforced.rs` | `206` | std::fs usage bypasses Vfs port |
| `contextra` | `crates/contextra/tests/license_gate_always_enforced.rs` | `229` | std::fs usage bypasses Vfs port |
| `contextra` | `crates/contextra/tests/license_gate_enforcement.rs` | `26` | SystemTime::now() violates P28 (use Clock port) |
| `contextra` | `crates/contextra/tests/license_gate_enforcement.rs` | `31` | std::fs usage bypasses Vfs port |
| `contextra` | `crates/contextra/tests/license_gate_enforcement.rs` | `40` | Call to .unwrap() in code |
| `contextra` | `crates/contextra/tests/license_gate_enforcement.rs` | `49` | Invocation of panic!() macro |
| `contextra` | `crates/contextra/tests/license_gate_enforcement.rs` | `52` | std::fs usage bypasses Vfs port |
| `contextra` | `crates/contextra/tests/license_gate_enforcement.rs` | `62` | SystemTime::now() violates P28 (use Clock port) |
| `contextra` | `crates/contextra/tests/license_gate_enforcement.rs` | `67` | std::fs usage bypasses Vfs port |
| `contextra` | `crates/contextra/tests/license_gate_enforcement.rs` | `77` | Call to .unwrap() in code |
| `contextra` | `crates/contextra/tests/license_gate_enforcement.rs` | `86` | Invocation of panic!() macro |
| `contextra` | `crates/contextra/tests/license_gate_enforcement.rs` | `89` | std::fs usage bypasses Vfs port |
| `contextra` | `crates/contextra/tests/license_gate_enforcement.rs` | `99` | SystemTime::now() violates P28 (use Clock port) |
| `contextra` | `crates/contextra/tests/license_gate_enforcement.rs` | `104` | std::fs usage bypasses Vfs port |
| `contextra` | `crates/contextra/tests/license_gate_enforcement.rs` | `116` | Call to .unwrap() in code |
| `contextra` | `crates/contextra/tests/license_gate_enforcement.rs` | `117` | Call to .unwrap() in code |
| `contextra` | `crates/contextra/tests/license_gate_enforcement.rs` | `119` | std::fs usage bypasses Vfs port |
| `contextra` | `crates/contextra/tests/license_gate_enforcement.rs` | `139` | SystemTime::now() violates P28 (use Clock port) |
| `contextra` | `crates/contextra/tests/license_gate_enforcement.rs` | `144` | std::fs usage bypasses Vfs port |
| `contextra` | `crates/contextra/tests/license_gate_enforcement.rs` | `158` | Call to .unwrap() in code |
| `contextra` | `crates/contextra/tests/license_gate_enforcement.rs` | `159` | Call to .unwrap() in code |
| `contextra` | `crates/contextra/tests/license_gate_enforcement.rs` | `161` | std::fs usage bypasses Vfs port |
| `contextra` | `crates/contextra/tests/review_stub_proof.rs` | `24` | Call to .unwrap() in code |
| `contextra` | `crates/contextra/tests/review_stub_proof.rs` | `55` | Call to .unwrap() in code |
| `contextra` | `crates/contextra/tests/review_stub_proof.rs` | `74` | Call to .expect() in code |
| `contextra` | `crates/contextra/tests/review_stub_proof.rs` | `78` | Call to .unwrap() in code |
| `contextra` | `crates/contextra/tests/tenant_isolation_facade.rs` | `18` | Call to .expect() in code |
| `contextra` | `crates/contextra/tests/tenant_isolation_facade.rs` | `59` | Call to .expect() in code |
| `contextra` | `crates/contextra/tests/tenant_isolation_facade.rs` | `60` | Call to .expect() in code |
| `contextra` | `crates/contextra/tests/tenant_isolation_facade.rs` | `136` | Call to .expect() in code |
| `contextra` | `crates/contextra/tests/vertical_path_ssi_under_write_load.rs` | `83` | Call to .unwrap() in code |
| `contextra` | `crates/contextra/tests/vertical_path_ssi_under_write_load.rs` | `101` | Instant::now() violates P28 (use Clock port) |
| `contextra` | `crates/contextra/tests/vertical_path_ssi_under_write_load.rs` | `244` | Call to .unwrap() in code |
| `contextra` | `crates/contextra/tests/vertical_path_ssi_under_write_load.rs` | `289` | Invocation of panic!() macro |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `883` | Call to .expect() in code |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `886` | Call to .expect() in code |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `889` | Call to .expect() in code |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `904` | Call to .expect() in code |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `920` | Call to .expect() in code |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `927` | Call to .expect() in code |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `957` | Instant::now() violates P28 (use Clock port) |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `1031` | Call to .expect() in code |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `1042` | Call to .expect() in code |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `1054` | Call to .expect() in code |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `1055` | Call to .expect() in code |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `1099` | Invocation of panic!() macro |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `1120` | Call to .expect() in code |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `1123` | Call to .expect() in code |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `1141` | Call to .expect() in code |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `1144` | Call to .expect() in code |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `1176` | Call to .expect() in code |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `1188` | Invocation of panic!() macro |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `1204` | Invocation of panic!() macro |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `1235` | Invocation of panic!() macro |
| `contextra-adapt` | `crates/contextra-adapt/src/lyapunov.rs` | `323` | Invocation of panic!() macro |
| `contextra-adapt` | `crates/contextra-adapt/src/lyapunov.rs` | `353` | Invocation of panic!() macro |
| `contextra-adapt` | `crates/contextra-adapt/src/lyapunov.rs` | `407` | Invocation of panic!() macro |
| `contextra-adapt` | `crates/contextra-adapt/src/lyapunov.rs` | `427` | Invocation of panic!() macro |
| `contextra-adapt` | `crates/contextra-adapt/src/lyapunov.rs` | `452` | Call to .expect() in code |
| `contextra-adapt` | `crates/contextra-adapt/src/lyapunov.rs` | `483` | Call to .expect() in code |
| `contextra-adapt` | `crates/contextra-adapt/src/lyapunov.rs` | `547` | Call to .expect() in code |
| `contextra-adapt` | `crates/contextra-adapt/src/offpolicy.rs` | `182` | Invocation of panic!() macro |
| `contextra-adapt` | `crates/contextra-adapt/src/pid.rs` | `315` | Invocation of panic!() macro |
| `contextra-adapt` | `crates/contextra-adapt/src/pid.rs` | `343` | Invocation of panic!() macro |
| `contextra-adapt` | `crates/contextra-adapt/src/pid.rs` | `397` | Invocation of panic!() macro |
| `contextra-adapt` | `crates/contextra-adapt/src/pid_latency_controller.rs` | `24` | Forbidden wording (§6.4): '100 %' |
| `contextra-adapt` | `crates/contextra-adapt/src/shadow_mode.rs` | `112` | Invocation of panic!() macro |
| `contextra-adapt` | `crates/contextra-adapt/src/shadow_mode.rs` | `123` | Invocation of panic!() macro |
| `contextra-adapt` | `crates/contextra-adapt/tests/bandit_sherman_morrison_long_run_stability.rs` | `49` | Invocation of panic!() macro |
| `contextra-adapt` | `crates/contextra-adapt/tests/bandit_sherman_morrison_long_run_stability.rs` | `102` | Invocation of panic!() macro |
| `contextra-adapt` | `crates/contextra-adapt/tests/bandit_sherman_morrison_long_run_stability.rs` | `215` | Call to .expect() in code |
| `contextra-adapt` | `crates/contextra-adapt/tests/bandit_sherman_morrison_long_run_stability.rs` | `223` | Invocation of panic!() macro |
| `contextra-adapt` | `crates/contextra-adapt/tests/bandit_sherman_morrison_long_run_stability.rs` | `228` | Call to .expect() in code |
| `contextra-adapt` | `crates/contextra-adapt/tests/controller_properties_and_convergence.rs` | `186` | Call to .expect() in code |
| `contextra-adapt` | `crates/contextra-adapt/tests/controller_properties_and_convergence.rs` | `187` | Call to .expect() in code |
| `contextra-adapt` | `crates/contextra-adapt/tests/controller_properties_and_convergence.rs` | `203` | Call to .expect() in code |
| `contextra-adapt` | `crates/contextra-adapt/tests/controller_properties_and_convergence.rs` | `205` | Call to .expect() in code |
| `contextra-adapt` | `crates/contextra-adapt/tests/controller_properties_and_convergence.rs` | `233` | Call to .expect() in code |
| `contextra-adapt` | `crates/contextra-adapt/tests/controller_properties_and_convergence.rs` | `234` | Call to .expect() in code |
| `contextra-adapt` | `crates/contextra-adapt/tests/controller_properties_and_convergence.rs` | `237` | Call to .expect() in code |
| `contextra-adapt` | `crates/contextra-adapt/tests/controller_properties_and_convergence.rs` | `238` | Call to .expect() in code |
| `contextra-adapt` | `crates/contextra-adapt/tests/controller_properties_and_convergence.rs` | `282` | Call to .unwrap() in code |
| `contextra-adapt` | `crates/contextra-adapt/tests/controller_properties_and_convergence.rs` | `292` | Call to .expect() in code |
| `contextra-adapt` | `crates/contextra-adapt/tests/controller_properties_and_convergence.rs` | `293` | Call to .expect() in code |
| `contextra-adapt` | `crates/contextra-adapt/tests/controller_properties_and_convergence.rs` | `313` | Call to .expect() in code |
| `contextra-adapt` | `crates/contextra-adapt/tests/controller_properties_and_convergence.rs` | `316` | Call to .expect() in code |
| `contextra-adapt` | `crates/contextra-adapt/tests/controller_properties_and_convergence.rs` | `324` | Call to .unwrap() in code |
| `contextra-adapt` | `crates/contextra-adapt/tests/fc_ts_off_policy_positivity.rs` | `90` | Invocation of panic!() macro |
| `contextra-adapt` | `crates/contextra-adapt/tests/pid_stability.rs` | `132` | Call to .expect() in code |
| `contextra-adapt` | `crates/contextra-adapt/tests/proptest_sherman_morrison_matrix_inverse_differential.rs` | `211` | Call to .expect() in code |
| `contextra-adapt` | `crates/contextra-adapt/tests/proptest_sherman_morrison_matrix_inverse_differential.rs` | `221` | Call to .expect() in code |
| `contextra-adapt` | `crates/contextra-adapt/tests/sketched_projection_config_validation.rs` | `86` | Call to .expect() in code |
| `contextra-adapt` | `crates/contextra-adapt/tests/sketched_projection_config_validation.rs` | `96` | Invocation of panic!() macro |
| `contextra-adapt` | `crates/contextra-adapt/tests/sketched_projection_config_validation.rs` | `106` | Invocation of panic!() macro |
| `contextra-adapt` | `crates/contextra-adapt/tests/sketched_projection_determinism.rs` | `19` | Call to .expect() in code |
| `contextra-adapt` | `crates/contextra-adapt/tests/sketched_projection_determinism.rs` | `23` | Call to .expect() in code |
| `contextra-adapt` | `crates/contextra-adapt/tests/sketched_projection_determinism.rs` | `42` | Call to .expect() in code |
| `contextra-adapt` | `crates/contextra-adapt/tests/sketched_projection_determinism.rs` | `72` | Call to .expect() in code |
| `contextra-adapt` | `crates/contextra-adapt/tests/sketched_projection_determinism.rs` | `75` | Call to .expect() in code |
| `contextra-adapt` | `crates/contextra-adapt/tests/sketched_projection_determinism.rs` | `77` | Call to .expect() in code |
| `contextra-adapt` | `crates/contextra-adapt/tests/sketched_projection_determinism.rs` | `85` | Call to .expect() in code |
| `contextra-adapt` | `crates/contextra-adapt/tests/sketched_projection_determinism.rs` | `93` | Call to .expect() in code |
| `contextra-adapt` | `crates/contextra-adapt/tests/sketched_projection_determinism.rs` | `96` | Call to .expect() in code |
| `contextra-adapt` | `crates/contextra-adapt/tests/sketched_projection_determinism.rs` | `107` | Call to .expect() in code |
| `contextra-adapt` | `crates/contextra-adapt/tests/sketched_projection_determinism.rs` | `108` | Call to .expect() in code |
| `contextra-adapt` | `crates/contextra-adapt/tests/sketched_projection_determinism.rs` | `119` | Call to .expect() in code |
| `contextra-adapt` | `crates/contextra-adapt/tests/sketched_projection_determinism.rs` | `120` | Call to .expect() in code |
| `contextra-adapt` | `crates/contextra-adapt/tests/sketched_projection_vs_sherman_morrison.rs` | `103` | Forbidden wording (§6.4): 'garantiert' |
| `contextra-adapt` | `crates/contextra-adapt/tests/sketched_projection_vs_sherman_morrison.rs` | `78` | Call to .expect() in code |
| `contextra-adapt` | `crates/contextra-adapt/tests/sketched_projection_vs_sherman_morrison.rs` | `81` | Call to .expect() in code |
| `contextra-adapt` | `crates/contextra-adapt/tests/sketched_projection_vs_sherman_morrison.rs` | `89` | Call to .expect() in code |
| `contextra-adapt` | `crates/contextra-adapt/tests/sketched_projection_vs_sherman_morrison.rs` | `90` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/src/audit.rs` | `241` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/src/audit.rs` | `277` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/src/audit.rs` | `278` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/src/audit.rs` | `280` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/src/audit.rs` | `291` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/src/audit.rs` | `317` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/src/audit.rs` | `329` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/src/budget.rs` | `34` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/src/budget.rs` | `48` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/src/budget.rs` | `76` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/src/budget.rs` | `79` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/src/clm_scratchpad.rs` | `544` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/src/context.rs` | `506` | Instant::now() violates P28 (use Clock port) |
| `contextra-agent` | `crates/contextra-agent/src/context.rs` | `517` | Instant::now() violates P28 (use Clock port) |
| `contextra-agent` | `crates/contextra-agent/src/dlq.rs` | `342` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/src/dlq.rs` | `356` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/src/dlq.rs` | `371` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/src/dlq.rs` | `386` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/src/graph.rs` | `56` | Forbidden wording (§6.4): '100 %' |
| `contextra-agent` | `crates/contextra-agent/src/engine/tests.rs` | `11` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/src/engine/tests.rs` | `21` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/src/engine/tests.rs` | `23` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/src/engine/tests.rs` | `31` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/src/engine/tests.rs` | `85` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/src/engine/tests.rs` | `111` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/src/engine/tests.rs` | `116` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/src/engine/tests.rs` | `151` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/src/engine/tests.rs` | `240` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/src/engine/tests.rs` | `247` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/src/engine/tests.rs` | `250` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/src/engine/tests.rs` | `255` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/src/engine/tests.rs` | `258` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/src/engine/tests.rs` | `261` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/src/engine/tests.rs` | `265` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/src/engine/tests.rs` | `268` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/src/engine/tests.rs` | `274` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/src/tools/tests.rs` | `43` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/src/tools/tests.rs` | `53` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/src/tools/tests.rs` | `55` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/src/tools/tests.rs` | `63` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/src/tools/tests.rs` | `102` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/src/tools/tests.rs` | `133` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/src/tools/tests.rs` | `140` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/src/tools/tests.rs` | `170` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/src/tools/tests.rs` | `176` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/agent_recovery.rs` | `66` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/agent_recovery.rs` | `76` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/agent_recovery.rs` | `84` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/agent_recovery.rs` | `92` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/agent_recovery.rs` | `97` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/agent_recovery.rs` | `100` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/agent_recovery.rs` | `101` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/agent_recovery.rs` | `103` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/agent_recovery.rs` | `104` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/agent_recovery.rs` | `106` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/agent_recovery.rs` | `113` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/agent_recovery.rs` | `122` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/agent_recovery.rs` | `130` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/agent_recovery.rs` | `135` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/agent_recovery.rs` | `138` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/agent_recovery.rs` | `141` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/agent_recovery.rs` | `142` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/agent_recovery.rs` | `143` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/agent_recovery.rs` | `145` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/agent_recovery.rs` | `146` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/agent_recovery.rs` | `149` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/agent_recovery.rs` | `157` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/agent_recovery.rs` | `167` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/agent_recovery.rs` | `175` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/agent_recovery.rs` | `180` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/agent_recovery.rs` | `183` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/agent_recovery.rs` | `184` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/agent_recovery.rs` | `186` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/agent_recovery.rs` | `187` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/agent_recovery.rs` | `199` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/agent_recovery.rs` | `210` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/agent_recovery.rs` | `218` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/agent_recovery.rs` | `223` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/agent_recovery.rs` | `226` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/agent_recovery.rs` | `227` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/agent_recovery.rs` | `229` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/agent_recovery.rs` | `230` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/agent_recovery.rs` | `232` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/agent_recovery.rs` | `239` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/agent_recovery.rs` | `250` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/agent_recovery.rs` | `258` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/agent_recovery.rs` | `263` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/agent_recovery.rs` | `266` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/agent_recovery.rs` | `267` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/agent_recovery.rs` | `269` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/agent_recovery.rs` | `270` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/agent_recovery.rs` | `282` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/agent_recovery.rs` | `291` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/agent_recovery.rs` | `299` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/agent_recovery.rs` | `307` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/agent_recovery.rs` | `313` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/agent_recovery.rs` | `316` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/agent_recovery.rs` | `319` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/agent_recovery.rs` | `321` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/agent_recovery.rs` | `322` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/agent_recovery.rs` | `324` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/agent_recovery.rs` | `326` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/agent_recovery.rs` | `327` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/agent_recovery.rs` | `332` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/agent_recovery.rs` | `336` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/agent_recovery.rs` | `349` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/agent_recovery.rs` | `367` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/agent_recovery.rs` | `376` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/agent_recovery.rs` | `388` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/agent_recovery.rs` | `404` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/agent_recovery.rs` | `406` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/agent_recovery.rs` | `414` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/agent_recovery.rs` | `419` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/agent_recovery.rs` | `422` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/agent_recovery.rs` | `423` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/agent_recovery.rs` | `426` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/audit_isolation_test.rs` | `24` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/audit_isolation_test.rs` | `76` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/audit_isolation_test.rs` | `150` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/audit_isolation_test.rs` | `293` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/audit_isolation_test.rs` | `311` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/audit_isolation_test.rs` | `392` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/audit_isolation_test.rs` | `395` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/audit_isolation_test.rs` | `407` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/boundary_validation_tests.rs` | `115` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/boundary_validation_tests.rs` | `119` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/boundary_validation_tests.rs` | `120` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/boundary_validation_tests.rs` | `150` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/budget_race_test.rs` | `68` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/budget_race_test.rs` | `77` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/budget_race_test.rs` | `300` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/budget_race_test.rs` | `305` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/budget_race_test.rs` | `307` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/budget_race_test.rs` | `315` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/budget_race_test.rs` | `317` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/clm_scratchpad_reset_flow.rs` | `37` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/clm_scratchpad_reset_flow.rs` | `63` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/clm_scratchpad_reset_flow.rs` | `73` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/clm_scratchpad_reset_flow.rs` | `80` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/clock_tests.rs` | `88` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/clock_tests.rs` | `154` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/clock_tests.rs` | `175` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/contract_tests.rs` | `77` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/contract_tests.rs` | `87` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/contract_tests.rs` | `90` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/contract_tests.rs` | `91` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/contract_tests.rs` | `94` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/contract_tests.rs` | `103` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/contract_tests.rs` | `105` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/contract_tests.rs` | `107` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/contract_tests.rs` | `108` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/contract_tests.rs` | `110` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/contract_tests.rs` | `111` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/contract_tests.rs` | `112` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/contract_tests.rs` | `123` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/contract_tests.rs` | `126` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/contract_tests.rs` | `129` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/contract_tests.rs` | `136` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/contract_tests.rs` | `162` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/contract_tests.rs` | `165` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/contract_tests.rs` | `170` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/contract_tests.rs` | `177` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/contract_tests.rs` | `188` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/contract_tests.rs` | `191` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/contract_tests.rs` | `194` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/contract_tests.rs` | `195` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/contract_tests.rs` | `199` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/contract_tests.rs` | `228` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/contract_tests.rs` | `231` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/crash_recovery_injection.rs` | `70` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/crash_recovery_injection.rs` | `76` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/crash_recovery_injection.rs` | `78` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/crash_recovery_injection.rs` | `95` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/crash_recovery_injection.rs` | `98` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/crash_recovery_injection.rs` | `101` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/crash_recovery_injection.rs` | `104` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/crash_recovery_injection.rs` | `108` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/crash_recovery_injection.rs` | `111` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/crash_recovery_injection.rs` | `112` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/crash_recovery_injection.rs` | `122` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/crash_recovery_injection.rs` | `124` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/crash_recovery_injection.rs` | `125` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/crash_recovery_injection.rs` | `126` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/crash_recovery_injection.rs` | `129` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/crash_recovery_injection.rs` | `144` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/crash_recovery_injection.rs` | `154` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/crash_recovery_injection.rs` | `159` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/crash_recovery_injection.rs` | `168` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/crash_recovery_injection.rs` | `170` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/crash_recovery_injection.rs` | `184` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/crash_recovery_injection.rs` | `185` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/crash_recovery_injection.rs` | `188` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/crash_recovery_injection.rs` | `202` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/crash_recovery_injection.rs` | `205` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/crash_recovery_injection.rs` | `207` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/dlq_clock_determinism_test.rs` | `98` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/dlq_clock_determinism_test.rs` | `109` | Invocation of panic!() macro |
| `contextra-agent` | `crates/contextra-agent/tests/dlq_tests.rs` | `75` | Invocation of panic!() macro |
| `contextra-agent` | `crates/contextra-agent/tests/dlq_tests.rs` | `78` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/dlq_tests.rs` | `89` | Invocation of panic!() macro |
| `contextra-agent` | `crates/contextra-agent/tests/dlq_tests.rs` | `338` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/e2e_integration.rs` | `51` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/e2e_integration.rs` | `61` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/e2e_integration.rs` | `65` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/e2e_integration.rs` | `66` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/e2e_integration.rs` | `84` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/e2e_integration.rs` | `87` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/e2e_integration.rs` | `90` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/e2e_integration.rs` | `92` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/e2e_integration.rs` | `93` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/e2e_integration.rs` | `95` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/e2e_integration.rs` | `98` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/e2e_integration.rs` | `100` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/e2e_integration.rs` | `109` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/e2e_integration.rs` | `118` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/e2e_integration.rs` | `123` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/e2e_integration.rs` | `126` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/e2e_integration.rs` | `132` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/e2e_integration.rs` | `133` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/e2e_integration.rs` | `134` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/e2e_integration.rs` | `137` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/e2e_integration.rs` | `138` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/e2e_integration.rs` | `141` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/e2e_integration.rs` | `142` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/e2e_integration.rs` | `147` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/e2e_integration.rs` | `151` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/e2e_integration.rs` | `153` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/e2e_integration.rs` | `154` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/e2e_integration.rs` | `155` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/e2e_integration.rs` | `156` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/e2e_integration.rs` | `161` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/e2e_integration.rs` | `173` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/e2e_integration.rs` | `184` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/e2e_integration.rs` | `192` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/e2e_integration.rs` | `200` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/e2e_integration.rs` | `203` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/e2e_integration.rs` | `204` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/e2e_integration.rs` | `211` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/event_loop_integration.rs` | `60` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/event_loop_integration.rs` | `70` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/event_loop_integration.rs` | `74` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/event_loop_integration.rs` | `75` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/event_loop_integration.rs` | `87` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/event_loop_integration.rs` | `95` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/event_loop_integration.rs` | `98` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/event_loop_integration.rs` | `100` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/event_loop_integration.rs` | `101` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/event_loop_integration.rs` | `103` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/event_loop_integration.rs` | `106` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/event_loop_integration.rs` | `110` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/event_loop_integration.rs` | `112` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/event_loop_integration.rs` | `114` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/event_loop_integration.rs` | `116` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/event_loop_integration.rs` | `122` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/event_loop_integration.rs` | `135` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/event_loop_integration.rs` | `146` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/event_loop_integration.rs` | `149` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/event_loop_integration.rs` | `150` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/event_loop_integration.rs` | `152` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/event_loop_integration.rs` | `155` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/event_loop_integration.rs` | `165` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/event_loop_integration.rs` | `166` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/event_loop_integration.rs` | `171` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/event_loop_integration.rs` | `178` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/event_loop_integration.rs` | `188` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/event_loop_integration.rs` | `191` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/event_loop_integration.rs` | `195` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/event_loop_integration.rs` | `206` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/event_loop_integration.rs` | `211` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/event_loop_integration.rs` | `212` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/event_loop_integration.rs` | `218` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/event_loop_integration.rs` | `229` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/event_loop_integration.rs` | `234` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/event_loop_integration.rs` | `235` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/event_loop_integration.rs` | `274` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/event_loop_integration.rs` | `277` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/event_loop_integration.rs` | `278` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/event_loop_integration.rs` | `280` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/event_loop_integration.rs` | `283` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/event_loop_integration.rs` | `287` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/event_loop_integration.rs` | `296` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/event_loop_integration.rs` | `335` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/event_loop_integration.rs` | `338` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/event_loop_integration.rs` | `339` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/event_loop_integration.rs` | `341` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/event_loop_integration.rs` | `344` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/event_loop_integration.rs` | `366` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/final_state_test.rs` | `22` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/final_state_test.rs` | `33` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/final_state_test.rs` | `38` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/final_state_test.rs` | `41` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/final_state_test.rs` | `42` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/graph_integration.rs` | `21` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/graph_integration.rs` | `24` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/graph_integration.rs` | `27` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/graph_integration.rs` | `30` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/graph_integration.rs` | `33` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/graph_integration.rs` | `36` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/graph_integration.rs` | `39` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/graph_integration.rs` | `40` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/graph_integration.rs` | `53` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/graph_integration.rs` | `59` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/graph_integration.rs` | `69` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/graph_integration.rs` | `73` | Call to .expect() in code |
| `contextra-agent` | `crates/contextra-agent/tests/graph_integration.rs` | `78` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/graph_integration.rs` | `81` | Call to .unwrap() in code |
| `contextra-agent` | `crates/contextra-agent/tests/graph_integration.rs` | `84` | Call to .unwrap() in code |

*... und 11811 weitere Befunde (siehe JSON-Export).*

---
*Automatisch generiert durch `pattern_diagnostic.py` für Agentic Engineering & Context Auditing.*