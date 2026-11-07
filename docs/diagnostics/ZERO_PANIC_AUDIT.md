# Zero-Panic & Unsafe Indexing Audit Report

> Generiert automatisch durch `tools/zero_panic_analyzer.py`.
> Workspace Root: `/app`
> Gesamte gefundene Produktivcode-Verletzungen: **520**

## Crate Übersicht (Produktivpfade)

| Crate | Prod unwrap | Prod expect | Prod panic | Prod todo | Prod assert | Test Findings |
|---|---|---|---|---|---|---|
| `contextra` | 0 | 0 | 0 | 0 | 0 | 276 |
| `contextra-adapt` | 0 | 0 | 0 | 0 | 0 | 430 |
| `contextra-agent` | 2 | 19 | 0 | 0 | 38 | 921 |
| `contextra-audit-export` | 0 | 0 | 0 | 0 | 0 | 86 |
| `contextra-avv-generator` | 0 | 0 | 0 | 0 | 0 | 78 |
| `contextra-checkpoint` | 18 | 0 | 0 | 0 | 21 | 780 |
| `contextra-cognition` | 0 | 0 | 0 | 0 | 0 | 458 |
| `contextra-core` | 0 | 0 | 0 | 0 | 0 | 24 |
| `contextra-crypto` | 0 | 0 | 0 | 0 | 1 | 1638 |
| `contextra-db` | 0 | 0 | 0 | 0 | 1 | 1669 |
| `contextra-durable-fs` | 0 | 0 | 0 | 0 | 0 | 117 |
| `contextra-engine` | 111 | 1 | 6 | 0 | 66 | 1455 |
| `contextra-erasure` | 0 | 0 | 0 | 0 | 0 | 569 |
| `contextra-graph` | 30 | 0 | 0 | 0 | 13 | 2472 |
| `contextra-infer-candle` | 20 | 2 | 0 | 0 | 22 | 449 |
| `contextra-infer-ollama` | 11 | 0 | 0 | 0 | 9 | 468 |
| `contextra-infer-onnx` | 5 | 0 | 1 | 0 | 8 | 163 |
| `contextra-ingest` | 0 | 0 | 0 | 0 | 0 | 899 |
| `contextra-kvcache` | 0 | 0 | 0 | 0 | 0 | 600 |
| `contextra-license` | 0 | 0 | 0 | 0 | 0 | 156 |
| `contextra-mcp` | 0 | 0 | 0 | 0 | 0 | 1016 |
| `contextra-mvcc` | 0 | 0 | 0 | 0 | 0 | 389 |
| `contextra-ports` | 0 | 0 | 0 | 0 | 0 | 162 |
| `contextra-privacy` | 0 | 0 | 0 | 0 | 0 | 530 |
| `contextra-py` | 0 | 0 | 0 | 0 | 0 | 160 |
| `contextra-rank` | 0 | 0 | 0 | 0 | 0 | 340 |
| `contextra-router` | 3 | 0 | 0 | 0 | 11 | 600 |
| `contextra-sandbox` | 0 | 0 | 0 | 0 | 10 | 126 |
| `contextra-simd` | 0 | 0 | 0 | 0 | 0 | 296 |
| `contextra-store` | 31 | 8 | 4 | 0 | 12 | 5130 |
| `contextra-sys` | 0 | 0 | 0 | 0 | 0 | 39 |
| `contextra-testkit` | 0 | 0 | 1 | 0 | 0 | 288 |
| `contextra-text` | 0 | 0 | 0 | 0 | 0 | 653 |
| `contextra-types` | 0 | 0 | 0 | 0 | 2 | 977 |
| `contextra-vector` | 1 | 9 | 0 | 0 | 6 | 1607 |
| `contextra-wire` | 16 | 0 | 0 | 0 | 1 | 221 |

## Detaillierte Produktiv-Fundstellen

### `contextra-agent`

- `/app/crates/contextra-agent/src/engine/tests.rs:11` [expect]: `let tmp = TempDir::new().expect("temp dir");`
- `/app/crates/contextra-agent/src/engine/tests.rs:21` [expect]: `.expect("open db"),`
- `/app/crates/contextra-agent/src/engine/tests.rs:23` [expect]: `let state_col = db.collection("test-state").await.expect("collection");`
- `/app/crates/contextra-agent/src/engine/tests.rs:31` [expect]: `.expect("agent context");`
- `/app/crates/contextra-agent/src/engine/tests.rs:51` [assert]: `assert!(evaluate_condition_expr("simple_str exists", &ctx));`
- `/app/crates/contextra-agent/src/engine/tests.rs:52` [assert]: `assert!(evaluate_condition_expr("nested.status exists", &ctx));`
- `/app/crates/contextra-agent/src/engine/tests.rs:53` [assert]: `assert!(evaluate_condition_expr("task_id exists", &ctx));`
- `/app/crates/contextra-agent/src/engine/tests.rs:54` [assert]: `assert!(!evaluate_condition_expr("null_val exists", &ctx));`
- `/app/crates/contextra-agent/src/engine/tests.rs:55` [assert]: `assert!(!evaluate_condition_expr("missing_key exists", &ctx));`
- `/app/crates/contextra-agent/src/engine/tests.rs:56` [assert]: `assert!(!evaluate_condition_expr(" exists", &ctx)); // missing key`
- `/app/crates/contextra-agent/src/engine/tests.rs:59` [assert]: `assert!(evaluate_condition_expr("simple_str == hello", &ctx));`
- `/app/crates/contextra-agent/src/engine/tests.rs:60` [assert]: `assert!(evaluate_condition_expr("simple_str == \"hello\"", &ctx));`
- `/app/crates/contextra-agent/src/engine/tests.rs:61` [assert]: `assert!(evaluate_condition_expr("number_val == 42", &ctx));`
- `/app/crates/contextra-agent/src/engine/tests.rs:62` [assert]: `assert!(evaluate_condition_expr("bool_val == true", &ctx));`
- `/app/crates/contextra-agent/src/engine/tests.rs:63` [assert]: `assert!(evaluate_condition_expr("nested.status == approved", &ctx));`
- `/app/crates/contextra-agent/src/engine/tests.rs:64` [assert]: `assert!(evaluate_condition_expr("nested.code == 200", &ctx));`
- `/app/crates/contextra-agent/src/engine/tests.rs:65` [assert]: `assert!(evaluate_condition_expr("task_id == test-task-1", &ctx));`
- `/app/crates/contextra-agent/src/engine/tests.rs:66` [assert]: `assert!(!evaluate_condition_expr("simple_str == world", &ctx));`
- `/app/crates/contextra-agent/src/engine/tests.rs:67` [assert]: `assert!(!evaluate_condition_expr("missing_key == foo", &ctx));`
- `/app/crates/contextra-agent/src/engine/tests.rs:70` [assert]: `assert!(evaluate_condition_expr("simple_str != world", &ctx));`
- `/app/crates/contextra-agent/src/engine/tests.rs:71` [assert]: `assert!(evaluate_condition_expr("missing_key != foo", &ctx));`
- `/app/crates/contextra-agent/src/engine/tests.rs:72` [assert]: `assert!(!evaluate_condition_expr("simple_str != hello", &ctx));`
- `/app/crates/contextra-agent/src/engine/tests.rs:75` [assert]: `assert!(!evaluate_condition_expr("invalid condition syntax", &ctx));`
- `/app/crates/contextra-agent/src/engine/tests.rs:76` [assert]: `assert!(!evaluate_condition_expr("", &ctx));`
- `/app/crates/contextra-agent/src/engine/tests.rs:77` [assert]: `assert!(!evaluate_condition_expr("   ", &ctx));`
- `/app/crates/contextra-agent/src/engine/tests.rs:78` [assert]: `assert!(!evaluate_condition_expr("== value_without_key", &ctx));`
- `/app/crates/contextra-agent/src/engine/tests.rs:79` [assert]: `assert!(!evaluate_condition_expr("!= value_without_key", &ctx));`
- `/app/crates/contextra-agent/src/engine/tests.rs:85` [expect]: `let orchestrator = OrchestratorEngine::try_from_db(&ctx.db).expect("engine try_from_db");`
- `/app/crates/contextra-agent/src/engine/tests.rs:107` [assert]: `assert!(audit_res.is_ok());`
- `/app/crates/contextra-agent/src/engine/tests.rs:111` [unwrap]: `let audit_entry = ctx.state_collection.get_kv(&audit_id).await.unwrap();`
- `/app/crates/contextra-agent/src/engine/tests.rs:112` [assert]: `assert!(audit_entry.is_some());`
- `/app/crates/contextra-agent/src/engine/tests.rs:116` [unwrap]: `let state_entry = ctx.state_collection.get_kv(&state_doc_id).await.unwrap();`
- `/app/crates/contextra-agent/src/engine/tests.rs:117` [assert]: `assert!(state_entry.is_none());`
- `/app/crates/contextra-agent/src/engine/tests.rs:120` [assert]: `assert_eq!(ctx.step_count, 0);`
- `/app/crates/contextra-agent/src/engine/tests.rs:151` [expect]: `let mut orchestrator = OrchestratorEngine::try_from_db(&ctx.db).expect("engine try_from_db");`
- `/app/crates/contextra-agent/src/engine/tests.rs:157` [assert]: `assert!(orchestrator.try_register_tool(Box::new(valid_tool)).is_ok());`
- `/app/crates/contextra-agent/src/engine/tests.rs:163` [assert]: `assert!(matches!(`
- `/app/crates/contextra-agent/src/engine/tests.rs:172` [assert]: `assert!(matches!(`
- `/app/crates/contextra-agent/src/engine/tests.rs:181` [assert]: `assert!(matches!(`
- `/app/crates/contextra-agent/src/engine/tests.rs:192` [assert]: `assert!(orchestrator.recover_orphans().await.is_ok());`
- `/app/crates/contextra-agent/src/engine/tests.rs:240` [expect]: `.expect("scratchpad edit");`
- `/app/crates/contextra-agent/src/engine/tests.rs:242` [assert]: `assert_eq!(scratchpad.current_subgoal_index(), 0);`
- `/app/crates/contextra-agent/src/engine/tests.rs:243` [assert]: `assert!(scratchpad.total_bytes() > 0);`
- `/app/crates/contextra-agent/src/engine/tests.rs:247` [expect]: `let mut orchestrator = OrchestratorEngine::try_from_db(&ctx.db).expect("engine try_from_db");`
- `/app/crates/contextra-agent/src/engine/tests.rs:250` [expect]: `.expect("register tool");`
- `/app/crates/contextra-agent/src/engine/tests.rs:255` [expect]: `.expect("add start");`
- `/app/crates/contextra-agent/src/engine/tests.rs:258` [expect]: `.expect("add task1");`
- `/app/crates/contextra-agent/src/engine/tests.rs:261` [expect]: `.expect("add end");`
- `/app/crates/contextra-agent/src/engine/tests.rs:265` [expect]: `.expect("edge start->task1");`
- `/app/crates/contextra-agent/src/engine/tests.rs:268` [expect]: `.expect("edge task1->end");`
- `/app/crates/contextra-agent/src/engine/tests.rs:271` [assert]: `assert!(res.is_ok());`
- `/app/crates/contextra-agent/src/engine/tests.rs:272` [assert]: `assert_eq!(ctx.status, crate::context::AgentStatus::Completed);`
- `/app/crates/contextra-agent/src/engine/tests.rs:274` [expect]: `let scratchpad_after = ctx.clm_scratchpad.as_ref().expect("scratchpad exists");`
- `/app/crates/contextra-agent/src/engine/tests.rs:275` [assert]: `assert_eq!(scratchpad_after.current_subgoal_index(), 1);`
- `/app/crates/contextra-agent/src/engine/tests.rs:276` [assert]: `assert_eq!(scratchpad_after.total_bytes(), 0);`
- `/app/crates/contextra-agent/src/tools/tests.rs:43` [expect]: `let tmp = TempDir::new().expect("temp dir");`
- `/app/crates/contextra-agent/src/tools/tests.rs:53` [expect]: `.expect("open db"),`
- `/app/crates/contextra-agent/src/tools/tests.rs:55` [expect]: `let state_col = db.collection("test-col").await.expect("collection");`
- `/app/crates/contextra-agent/src/tools/tests.rs:63` [expect]: `.expect("ctx");`
### `contextra-checkpoint`

- `/app/crates/contextra-checkpoint/src/store/tests.rs:140` [unwrap]: `let store = PersistentCheckpointStore::new(storage, "test").unwrap();`
- `/app/crates/contextra-checkpoint/src/store/tests.rs:144` [unwrap]: `.unwrap(); // unwrap`
- `/app/crates/contextra-checkpoint/src/store/tests.rs:147` [unwrap]: `.unwrap()`
- `/app/crates/contextra-checkpoint/src/store/tests.rs:148` [unwrap]: `.unwrap(); // unwrap`
- `/app/crates/contextra-checkpoint/src/store/tests.rs:149` [assert]: `assert_eq!(loaded, meta);`
- `/app/crates/contextra-checkpoint/src/store/tests.rs:156` [unwrap]: `let store = PersistentCheckpointStore::new(storage.clone(), "test").unwrap();`
- `/app/crates/contextra-checkpoint/src/store/tests.rs:160` [unwrap]: `.unwrap(); // unwrap`
- `/app/crates/contextra-checkpoint/src/store/tests.rs:164` [unwrap]: `.unwrap(); // unwrap`
- `/app/crates/contextra-checkpoint/src/store/tests.rs:165` [unwrap]: `let all = store.list_checkpoints().await.unwrap(); // unwrap`
- `/app/crates/contextra-checkpoint/src/store/tests.rs:166` [assert]: `assert_eq!(all.len(), 1);`
- `/app/crates/contextra-checkpoint/src/store/tests.rs:167` [assert]: `assert_eq!(all[0].seq_no, 2);`
- `/app/crates/contextra-checkpoint/src/store/tests.rs:168` [assert]: `assert!(!storage.pinned.lock().contains(&1));`
- `/app/crates/contextra-checkpoint/src/store/tests.rs:169` [assert]: `assert!(storage.pinned.lock().contains(&2));`
- `/app/crates/contextra-checkpoint/src/store/tests.rs:179` [unwrap]: `let store = PersistentCheckpointStore::new(storage.clone(), "test").unwrap();`
- `/app/crates/contextra-checkpoint/src/store/tests.rs:186` [assert]: `assert!(res.is_err());`
- `/app/crates/contextra-checkpoint/src/store/tests.rs:187` [assert]: `assert!(!storage.pinned.lock().contains(&seq_no));`
- `/app/crates/contextra-checkpoint/src/store/tests.rs:194` [unwrap]: `let store = PersistentCheckpointStore::new(storage.clone(), "test_orphan_recovery").unwrap();`
- `/app/crates/contextra-checkpoint/src/store/tests.rs:198` [unwrap]: `storage.pin_checkpoint(seq_no).await.unwrap();`
- `/app/crates/contextra-checkpoint/src/store/tests.rs:204` [assert]: `assert!(storage.pinned.lock().contains(&seq_no));`
- `/app/crates/contextra-checkpoint/src/store/tests.rs:205` [assert]: `assert!(!store.orphan_registry().get_orphan_pins().is_empty());`
- `/app/crates/contextra-checkpoint/src/store/tests.rs:208` [unwrap]: `let recovered = store.recover_orphaned_pins().await.unwrap();`
- `/app/crates/contextra-checkpoint/src/store/tests.rs:210` [assert]: `assert_eq!(recovered, vec![seq_no]);`
- `/app/crates/contextra-checkpoint/src/store/tests.rs:211` [assert]: `assert!(`
- `/app/crates/contextra-checkpoint/src/store/tests.rs:215` [assert]: `assert!(`
- `/app/crates/contextra-checkpoint/src/store/tests.rs:225` [unwrap]: `let store_a = PersistentCheckpointStore::new(storage1, "ns_inst_a").unwrap();`
- `/app/crates/contextra-checkpoint/src/store/tests.rs:228` [unwrap]: `let store_b = PersistentCheckpointStore::new(storage2, "ns_inst_b").unwrap();`
- `/app/crates/contextra-checkpoint/src/store/tests.rs:232` [unwrap]: `let _guard_a = store_a.create_guard(TxId::new(5555)).unwrap();`
- `/app/crates/contextra-checkpoint/src/store/tests.rs:238` [assert]: `assert_eq!(orphans_a.len(), 1);`
- `/app/crates/contextra-checkpoint/src/store/tests.rs:239` [assert]: `assert_eq!(orphans_a[0].tx_id, TxId::new(5555));`
- `/app/crates/contextra-checkpoint/src/store/tests.rs:243` [assert]: `assert!(`
- `/app/crates/contextra-checkpoint/src/store/tests.rs:253` [unwrap]: `let store = PersistentCheckpointStore::new(storage.clone(), "test_pinguard").unwrap();`
- `/app/crates/contextra-checkpoint/src/store/tests.rs:269` [assert]: `assert!(`
- `/app/crates/contextra-checkpoint/src/store/tests.rs:278` [assert]: `assert!(`
- `/app/crates/contextra-checkpoint/src/store/tests.rs:288` [unwrap]: `let store = PersistentCheckpointStore::new(storage.clone(), "test").unwrap();`
- `/app/crates/contextra-checkpoint/src/store/tests.rs:294` [unwrap]: `.unwrap(); // unwrap`
- `/app/crates/contextra-checkpoint/src/store/tests.rs:296` [assert]: `assert!(storage.pinned.lock().contains(&1));`
- `/app/crates/contextra-checkpoint/src/store/tests.rs:307` [assert]: `assert!(res.is_err());`
- `/app/crates/contextra-checkpoint/src/store/tests.rs:310` [assert]: `assert!(`
- `/app/crates/contextra-checkpoint/src/store/tests.rs:316` [assert]: `assert!(`
### `contextra-crypto`

- `/app/crates/contextra-crypto/fuzz/fuzz_targets/wal_hmac_chain_verify_fuzz.rs:144` [assert]: `assert!(`
### `contextra-db`

- `/app/crates/contextra-db/fuzz/fuzz_targets/rrf_fusion.rs:78` [assert]: `assert!(`
### `contextra-engine`

- `/app/crates/contextra-engine/src/transaction/tests.rs:20` [unwrap]: `let dir = tempdir().unwrap();`
- `/app/crates/contextra-engine/src/transaction/tests.rs:25` [unwrap]: `let storage = Arc::new(LsmStorage::new(lsm_config).await.unwrap());`
- `/app/crates/contextra-engine/src/transaction/tests.rs:31` [unwrap]: `.unwrap(),`
- `/app/crates/contextra-engine/src/transaction/tests.rs:52` [unwrap]: `let tx_id = col.allocate_tx().unwrap();`
- `/app/crates/contextra-engine/src/transaction/tests.rs:62` [assert]: `assert!(commit_res.is_ok());`
- `/app/crates/contextra-engine/src/transaction/tests.rs:70` [unwrap]: `let tx_id = col.allocate_tx().unwrap();`
- `/app/crates/contextra-engine/src/transaction/tests.rs:80` [assert]: `assert!(rollback_res.is_ok());`
- `/app/crates/contextra-engine/src/transaction/tests.rs:86` [unwrap]: `let tx_id = col.allocate_tx().unwrap();`
- `/app/crates/contextra-engine/src/transaction/tests.rs:96` [assert]: `assert!(results.is_ok());`
- `/app/crates/contextra-engine/src/transaction/tests.rs:97` [unwrap]: `assert!(results.unwrap().is_empty());`
- `/app/crates/contextra-engine/src/transaction/tests.rs:97` [assert]: `assert!(results.unwrap().is_empty());`
- `/app/crates/contextra-engine/src/extraction/tests.rs:58` [unwrap]: `.unwrap();`
- `/app/crates/contextra-engine/src/extraction/tests.rs:60` [assert]: `assert_eq!(triples.len(), 1);`
- `/app/crates/contextra-engine/src/extraction/tests.rs:61` [assert]: `assert_eq!(`
- `/app/crates/contextra-engine/src/extraction/tests.rs:71` [assert]: `assert_eq!(generator.call_count.load(Ordering::SeqCst), 1);`
- `/app/crates/contextra-engine/src/extraction/tests.rs:86` [assert]: `assert!(res.is_err());`
- `/app/crates/contextra-engine/src/extraction/tests.rs:89` [assert]: `assert_eq!(limit, 2);`
- `/app/crates/contextra-engine/src/extraction/tests.rs:90` [assert]: `assert!(context.contains("exceeds max_llm_calls_per_cycle limit"));`
- `/app/crates/contextra-engine/src/extraction/tests.rs:92` [panic]: `err => panic!("Expected LimitExceeded error, got {err:?}"),`
- `/app/crates/contextra-engine/src/extraction/tests.rs:94` [assert]: `assert_eq!(generator.call_count.load(Ordering::SeqCst), 0);`
- `/app/crates/contextra-engine/src/extraction/tests.rs:105` [assert]: `assert!(res.is_err());`
- `/app/crates/contextra-engine/src/extraction/tests.rs:108` [assert]: `assert!(msg.contains("Failed to parse LLM entity extraction response as JSON"));`
- `/app/crates/contextra-engine/src/extraction/tests.rs:110` [panic]: `err => panic!("Expected ParseError, got {err:?}"),`
- `/app/crates/contextra-engine/src/collection/query_builder/tests.rs:110` [unwrap]: `let dir = TempDir::new().unwrap(); // unwrap`
- `/app/crates/contextra-engine/src/collection/query_builder/tests.rs:115` [unwrap]: `let storage = Arc::new(LsmStorage::new(lsm_config).await.unwrap()); // unwrap`
- `/app/crates/contextra-engine/src/collection/query_builder/tests.rs:121` [unwrap]: `let index = Arc::new(HnswIndex::try_new(hnsw_config).unwrap()); // unwrap`
- `/app/crates/contextra-engine/src/collection/query_builder/tests.rs:141` [unwrap]: `.unwrap(); // unwrap`
- `/app/crates/contextra-engine/src/collection/query_builder/tests.rs:144` [unwrap]: `.unwrap(); // unwrap`
- `/app/crates/contextra-engine/src/collection/query_builder/tests.rs:147` [unwrap]: `let legacy = col.search(&[1.0, 0.0, 0.0, 0.0], 2).await.unwrap(); // unwrap`
- `/app/crates/contextra-engine/src/collection/query_builder/tests.rs:154` [unwrap]: `.unwrap(); // unwrap`
- `/app/crates/contextra-engine/src/collection/query_builder/tests.rs:156` [assert]: `assert_eq!(builder_res.len(), legacy.len());`
- `/app/crates/contextra-engine/src/collection/query_builder/tests.rs:157` [assert]: `assert_eq!(builder_res[0].id, legacy[0].id);`
- `/app/crates/contextra-engine/src/collection/query_builder/tests.rs:158` [assert]: `assert_eq!(builder_res[1].id, legacy[1].id);`
- `/app/crates/contextra-engine/src/collection/query_builder/tests.rs:166` [unwrap]: `.unwrap(); // unwrap`
- `/app/crates/contextra-engine/src/collection/query_builder/tests.rs:169` [unwrap]: `.unwrap(); // unwrap`
- `/app/crates/contextra-engine/src/collection/query_builder/tests.rs:180` [unwrap]: `.unwrap(); // unwrap`
- `/app/crates/contextra-engine/src/collection/query_builder/tests.rs:189` [unwrap]: `.unwrap(); // unwrap`
- `/app/crates/contextra-engine/src/collection/query_builder/tests.rs:191` [assert]: `assert_eq!(builder_res.len(), 1);`
- `/app/crates/contextra-engine/src/collection/query_builder/tests.rs:192` [assert]: `assert_eq!(builder_res.len(), legacy.len());`
- `/app/crates/contextra-engine/src/collection/query_builder/tests.rs:193` [assert]: `assert_eq!(builder_res[0].id, legacy[0].id);`
- `/app/crates/contextra-engine/src/collection/query_builder/tests.rs:194` [assert]: `assert_eq!(builder_res[0].id, "doc-1");`
- `/app/crates/contextra-engine/src/collection/query_builder/tests.rs:206` [unwrap]: `.unwrap(); // unwrap`
- `/app/crates/contextra-engine/src/collection/query_builder/tests.rs:213` [unwrap]: `.unwrap(); // unwrap`
- `/app/crates/contextra-engine/src/collection/query_builder/tests.rs:219` [unwrap]: `.unwrap(); // unwrap`
- `/app/crates/contextra-engine/src/collection/query_builder/tests.rs:228` [unwrap]: `.unwrap(); // unwrap`
- `/app/crates/contextra-engine/src/collection/query_builder/tests.rs:230` [assert]: `assert_eq!(builder_res.len(), legacy.len());`
- `/app/crates/contextra-engine/src/collection/query_builder/tests.rs:232` [assert]: `assert_eq!(builder_res[0].id, legacy[0].id);`
- `/app/crates/contextra-engine/src/collection/query_builder/tests.rs:245` [unwrap]: `.unwrap(); // unwrap`
- `/app/crates/contextra-engine/src/collection/query_builder/tests.rs:247` [unwrap]: `let weights = SignalWeights::new(0.6, 0.4, 0.0).unwrap(); // unwrap`
- `/app/crates/contextra-engine/src/collection/query_builder/tests.rs:257` [unwrap]: `.unwrap(); // unwrap`
- `/app/crates/contextra-engine/src/collection/query_builder/tests.rs:259` [assert]: `assert!(!builder_res.is_empty());`
- `/app/crates/contextra-engine/src/collection/query_builder/tests.rs:260` [assert]: `assert_eq!(builder_res[0].id, "doc-1");`
- `/app/crates/contextra-engine/src/collection/query_builder/tests.rs:268` [unwrap]: `.unwrap(); // unwrap`
- `/app/crates/contextra-engine/src/collection/query_builder/tests.rs:274` [unwrap]: `.unwrap(); // unwrap`
- `/app/crates/contextra-engine/src/collection/query_builder/tests.rs:277` [unwrap]: `let legacy = col.hybrid_search_with_query(&hybrid_query).await.unwrap(); // unwrap`
- `/app/crates/contextra-engine/src/collection/query_builder/tests.rs:284` [unwrap]: `.unwrap(); // unwrap`
- `/app/crates/contextra-engine/src/collection/query_builder/tests.rs:286` [assert]: `assert_eq!(builder_res.len(), legacy.len());`
- `/app/crates/contextra-engine/src/collection/query_builder/tests.rs:287` [assert]: `assert_eq!(builder_res[0].id, legacy[0].id);`
- `/app/crates/contextra-engine/src/collection/query_builder/tests.rs:304` [unwrap]: `.unwrap();`
- `/app/crates/contextra-engine/src/collection/query_builder/tests.rs:316` [unwrap]: `.unwrap();`
- `/app/crates/contextra-engine/src/collection/query_builder/tests.rs:318` [assert]: `assert_eq!(res.len(), 10, "Final results truncated to k=10");`
- `/app/crates/contextra-engine/src/collection/query_builder/tests.rs:320` [unwrap]: `assert!(res[0].metadata.as_ref().unwrap().get("ce_score").is_some());`
- `/app/crates/contextra-engine/src/collection/query_builder/tests.rs:320` [assert]: `assert!(res[0].metadata.as_ref().unwrap().get("ce_score").is_some());`
- `/app/crates/contextra-engine/src/collection/query_builder/tests.rs:337` [unwrap]: `.unwrap();`
- `/app/crates/contextra-engine/src/collection/query_builder/tests.rs:351` [unwrap]: `.unwrap();`
- `/app/crates/contextra-engine/src/collection/query_builder/tests.rs:353` [assert]: `assert_eq!(res_default.len(), 100);`
- `/app/crates/contextra-engine/src/collection/query_builder/tests.rs:365` [unwrap]: `.unwrap();`
- `/app/crates/contextra-engine/src/collection/query_builder/tests.rs:368` [assert]: `assert_eq!(`
- `/app/crates/contextra-engine/src/collection/query_builder/tests.rs:384` [unwrap]: `.unwrap(); // unwrap`
- `/app/crates/contextra-engine/src/collection/query_builder/tests.rs:391` [unwrap]: `.unwrap(); // unwrap`
- `/app/crates/contextra-engine/src/collection/query_builder/tests.rs:398` [unwrap]: `.unwrap(); // unwrap`
- `/app/crates/contextra-engine/src/collection/query_builder/tests.rs:410` [unwrap]: `.unwrap(); // unwrap`
- `/app/crates/contextra-engine/src/collection/query_builder/tests.rs:412` [assert]: `assert_eq!(res.len(), 2, "Results must be truncated to k=2");`
- `/app/crates/contextra-engine/src/collection/query_builder/tests.rs:415` [expect]: `let meta = item.metadata.as_ref().expect("metadata must exist"); // expect`
- `/app/crates/contextra-engine/src/collection/query_builder/tests.rs:416` [assert]: `assert!(`
- `/app/crates/contextra-engine/src/collection/query_builder/tests.rs:421` [assert]: `assert!(`
- `/app/crates/contextra-engine/src/collection/query_builder/tests.rs:428` [assert]: `assert!(`
- `/app/crates/contextra-engine/src/collection/query_builder/tests.rs:444` [unwrap]: `.unwrap();`
- `/app/crates/contextra-engine/src/collection/query_builder/tests.rs:451` [unwrap]: `.unwrap();`
- `/app/crates/contextra-engine/src/collection/query_builder/tests.rs:468` [unwrap]: `.unwrap();`
- `/app/crates/contextra-engine/src/collection/query_builder/tests.rs:470` [assert]: `assert_eq!(res.len(), 2);`
- `/app/crates/contextra-engine/src/collection/query_builder/tests.rs:472` [assert]: `assert!(updated_size.is_some());`
- `/app/crates/contextra-engine/src/collection/query_builder/tests.rs:484` [unwrap]: `.unwrap();`
- `/app/crates/contextra-engine/src/collection/query_builder/tests.rs:486` [assert]: `assert_eq!(res2.len(), 2);`
- `/app/crates/contextra-engine/src/collection/query_builder/tests.rs:498` [unwrap]: `.unwrap(); // unwrap`
- `/app/crates/contextra-engine/src/collection/query_builder/tests.rs:505` [unwrap]: `.unwrap(); // unwrap`
- `/app/crates/contextra-engine/src/collection/query_builder/tests.rs:518` [assert]: `assert!(`
- `/app/crates/contextra-engine/src/collection/query_builder/tests.rs:522` [unwrap]: `let results = res.unwrap(); // unwrap`
- `/app/crates/contextra-engine/src/collection/query_builder/tests.rs:523` [assert]: `assert_eq!(results.len(), 2);`
- `/app/crates/contextra-engine/src/collection/query_builder/tests.rs:526` [assert]: `assert!(`
- `/app/crates/contextra-engine/src/collection/query_builder/tests.rs:539` [unwrap]: `.unwrap(); // unwrap`
- `/app/crates/contextra-engine/src/collection/query_builder/tests.rs:542` [unwrap]: `.unwrap(); // unwrap`
- `/app/crates/contextra-engine/src/collection/query_builder/tests.rs:552` [unwrap]: `.unwrap(); // unwrap`
- `/app/crates/contextra-engine/src/collection/query_builder/tests.rs:554` [assert]: `assert_eq!(res.len(), 2);`
- `/app/crates/contextra-engine/src/collection/query_builder/tests.rs:557` [assert]: `assert!(`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:15` [unwrap]: `let dir = tempfile::tempdir().unwrap();`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:24` [unwrap]: `.unwrap();`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:28` [unwrap]: `.unwrap();`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:34` [unwrap]: `.unwrap();`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:39` [assert]: `assert!(res_default.is_err());`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:44` [unwrap]: `let dir = tempfile::tempdir().unwrap();`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:53` [unwrap]: `.unwrap();`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:54` [unwrap]: `let col = db.collection("test_scan_prefix_equal").await.unwrap();`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:59` [unwrap]: `.unwrap();`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:62` [unwrap]: `let results = col.scan_prefix("", Some(10)).await.unwrap();`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:63` [assert]: `assert_eq!(results.len(), 10);`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:65` [unwrap]: `let results_larger = col.scan_prefix("", Some(20)).await.unwrap();`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:66` [assert]: `assert_eq!(results_larger.len(), 10);`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:71` [unwrap]: `let dir = tempfile::tempdir().unwrap();`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:80` [unwrap]: `.unwrap();`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:81` [unwrap]: `let col = db.collection("test_scan_prefix_exceed").await.unwrap();`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:86` [unwrap]: `.unwrap();`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:90` [assert]: `assert!(matches!(`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:96` [assert]: `assert!(res_exact.is_ok());`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:97` [unwrap]: `assert_eq!(res_exact.unwrap().len(), 15);`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:97` [assert]: `assert_eq!(res_exact.unwrap().len(), 15);`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:102` [unwrap]: `let dir = tempfile::tempdir().unwrap();`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:111` [unwrap]: `.unwrap();`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:112` [unwrap]: `let col = db.collection("test_scan_default").await.unwrap();`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:117` [unwrap]: `.unwrap();`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:123` [unwrap]: `.unwrap();`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:124` [assert]: `assert_eq!(results.len(), 50);`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:125` [assert]: `assert!(results.len() <= MAX_SCAN_RESULTS);`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:130` [unwrap]: `let dir = tempfile::tempdir().unwrap();`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:139` [unwrap]: `.unwrap();`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:140` [unwrap]: `let col = db.collection("test_scan_equal").await.unwrap();`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:145` [unwrap]: `.unwrap();`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:151` [unwrap]: `.unwrap();`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:152` [assert]: `assert_eq!(results.len(), 10);`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:157` [unwrap]: `.unwrap();`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:158` [assert]: `assert_eq!(results_larger.len(), 10);`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:163` [unwrap]: `let dir = tempfile::tempdir().unwrap();`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:172` [unwrap]: `.unwrap();`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:173` [unwrap]: `let col = db.collection("test_scan_exceed").await.unwrap();`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:178` [unwrap]: `.unwrap();`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:182` [assert]: `assert!(matches!(`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:188` [assert]: `assert!(res_exact.is_ok());`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:189` [unwrap]: `assert_eq!(res_exact.unwrap().len(), 15);`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:189` [assert]: `assert_eq!(res_exact.unwrap().len(), 15);`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:194` [unwrap]: `let dir = tempfile::tempdir().unwrap();`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:203` [unwrap]: `.unwrap();`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:204` [unwrap]: `let collection = Arc::new(db.collection("test_race").await.unwrap());`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:219` [unwrap]: `match res.unwrap() {`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:222` [panic]: `Err(other) => panic!("Unexpected error: {:?}", other),`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:226` [assert]: `assert_eq!(ok_count, 1, "Exactly one task must succeed");`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:227` [assert]: `assert_eq!(`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:235` [unwrap]: `let dir = tempfile::tempdir().unwrap();`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:244` [unwrap]: `.unwrap();`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:246` [unwrap]: `let col_a = Arc::new(db.collection("col_a").await.unwrap());`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:247` [unwrap]: `let col_b = Arc::new(db.collection("col_b").await.unwrap());`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:265` [assert]: `assert!(`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:269` [unwrap]: `let put_res = res.unwrap().unwrap();`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:270` [assert]: `assert!(`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:281` [unwrap]: `let dir = tempfile::tempdir().unwrap();`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:290` [unwrap]: `.unwrap();`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:291` [unwrap]: `let collection = Arc::new(db.collection("test_scan_cap").await.unwrap());`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:298` [unwrap]: `.unwrap();`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:303` [panic]: `Err(e) => panic!("scan_prefix failed: {e:?}"),`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:305` [assert]: `assert_eq!(`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:314` [unwrap]: `let dir = tempfile::tempdir().unwrap();`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:323` [unwrap]: `.unwrap();`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:324` [unwrap]: `let col = db.collection("dim_test").await.unwrap();`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:331` [assert]: `assert!(res.is_err());`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:334` [panic]: `Err(other) => panic!("Expected InvalidInput error, got: {:?}", other),`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:335` [panic]: `Ok(_) => panic!("Expected insert to fail due to dimension mismatch"),`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:338` [assert]: `assert!(`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:347` [unwrap]: `let dir = tempfile::tempdir().unwrap();`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:356` [unwrap]: `.unwrap();`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:357` [unwrap]: `let col = Arc::new(db.collection("concurrency_test").await.unwrap());`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:369` [assert]: `assert!(`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:373` [unwrap]: `assert!(res.unwrap().unwrap().is_ok());`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:373` [assert]: `assert!(res.unwrap().unwrap().is_ok());`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:380` [unwrap]: `let dir = tempfile::tempdir().unwrap();`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:389` [unwrap]: `.unwrap();`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:390` [unwrap]: `let col = Arc::new(db.collection("delete_lock_test").await.unwrap());`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:395` [unwrap]: `.unwrap();`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:398` [unwrap]: `col.delete("doc_del_1").await.unwrap();`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:402` [assert]: `assert!(`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:411` [unwrap]: `let dir = tempfile::tempdir().unwrap();`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:420` [unwrap]: `.unwrap();`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:421` [unwrap]: `let col = Arc::new(db.collection("update_lock_test").await.unwrap());`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:426` [unwrap]: `.unwrap();`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:431` [unwrap]: `.unwrap();`
- `/app/crates/contextra-engine/src/collection/crud/tests.rs:435` [assert]: `assert!(`
### `contextra-graph`

- `/app/crates/contextra-graph/src/community/tests.rs:18` [unwrap]: `.unwrap(); // unwrap allowed`
- `/app/crates/contextra-graph/src/community/tests.rs:25` [unwrap]: `.unwrap(); // unwrap allowed`
- `/app/crates/contextra-graph/src/community/tests.rs:29` [unwrap]: `.unwrap(); // unwrap allowed`
- `/app/crates/contextra-graph/src/community/tests.rs:33` [unwrap]: `.unwrap(); // unwrap allowed`
- `/app/crates/contextra-graph/src/community/tests.rs:37` [unwrap]: `.unwrap(); // unwrap allowed`
- `/app/crates/contextra-graph/src/community/tests.rs:38` [unwrap]: `graph.commit(tx).await.unwrap(); // unwrap allowed`
- `/app/crates/contextra-graph/src/community/tests.rs:46` [unwrap]: `let run1 = detect_communities(&graph, &config).await.unwrap(); // unwrap allowed`
- `/app/crates/contextra-graph/src/community/tests.rs:47` [unwrap]: `let run2 = detect_communities(&graph, &config).await.unwrap(); // unwrap allowed`
- `/app/crates/contextra-graph/src/community/tests.rs:49` [assert]: `assert_eq!(`
- `/app/crates/contextra-graph/src/community/tests.rs:69` [unwrap]: `.unwrap(); // unwrap allowed`
- `/app/crates/contextra-graph/src/community/tests.rs:74` [unwrap]: `.unwrap(); // unwrap allowed`
- `/app/crates/contextra-graph/src/community/tests.rs:78` [unwrap]: `.unwrap(); // unwrap allowed`
- `/app/crates/contextra-graph/src/community/tests.rs:82` [unwrap]: `.unwrap(); // unwrap allowed`
- `/app/crates/contextra-graph/src/community/tests.rs:92` [unwrap]: `.unwrap(); // unwrap allowed`
- `/app/crates/contextra-graph/src/community/tests.rs:100` [unwrap]: `.unwrap(); // unwrap allowed`
- `/app/crates/contextra-graph/src/community/tests.rs:107` [unwrap]: `.unwrap(); // unwrap allowed`
- `/app/crates/contextra-graph/src/community/tests.rs:114` [unwrap]: `.unwrap(); // unwrap allowed`
- `/app/crates/contextra-graph/src/community/tests.rs:116` [unwrap]: `graph.commit(tx).await.unwrap(); // unwrap allowed`
- `/app/crates/contextra-graph/src/community/tests.rs:119` [unwrap]: `let assignments = detect_communities(&graph, &config).await.unwrap(); // unwrap allowed`
- `/app/crates/contextra-graph/src/community/tests.rs:128` [assert]: `assert_eq!(map[&2], c1_community);`
- `/app/crates/contextra-graph/src/community/tests.rs:129` [assert]: `assert_eq!(map[&3], c1_community);`
- `/app/crates/contextra-graph/src/community/tests.rs:133` [assert]: `assert_eq!(map[&101], c2_community);`
- `/app/crates/contextra-graph/src/community/tests.rs:134` [assert]: `assert_eq!(map[&102], c2_community);`
- `/app/crates/contextra-graph/src/community/tests.rs:137` [assert]: `assert_ne!(`
- `/app/crates/contextra-graph/src/community/tests.rs:164` [unwrap]: `.unwrap(); // unwrap allowed`
- `/app/crates/contextra-graph/src/community/tests.rs:169` [unwrap]: `.unwrap(); // unwrap allowed`
- `/app/crates/contextra-graph/src/community/tests.rs:173` [unwrap]: `.unwrap(); // unwrap allowed`
- `/app/crates/contextra-graph/src/community/tests.rs:177` [unwrap]: `.unwrap(); // unwrap allowed`
- `/app/crates/contextra-graph/src/community/tests.rs:187` [unwrap]: `.unwrap(); // unwrap allowed`
- `/app/crates/contextra-graph/src/community/tests.rs:192` [unwrap]: `.unwrap(); // unwrap allowed`
- `/app/crates/contextra-graph/src/community/tests.rs:196` [unwrap]: `.unwrap(); // unwrap allowed`
- `/app/crates/contextra-graph/src/community/tests.rs:200` [unwrap]: `.unwrap(); // unwrap allowed`
- `/app/crates/contextra-graph/src/community/tests.rs:209` [unwrap]: `.unwrap(); // unwrap allowed`
- `/app/crates/contextra-graph/src/community/tests.rs:211` [unwrap]: `graph.commit(tx).await.unwrap(); // unwrap allowed`
- `/app/crates/contextra-graph/src/community/tests.rs:214` [unwrap]: `let assignments = detect_communities(&graph, &config).await.unwrap(); // unwrap allowed`
- `/app/crates/contextra-graph/src/community/tests.rs:223` [assert]: `assert_eq!(`
- `/app/crates/contextra-graph/src/community/tests.rs:227` [assert]: `assert_eq!(`
- `/app/crates/contextra-graph/src/community/tests.rs:234` [assert]: `assert_eq!(`
- `/app/crates/contextra-graph/src/community/tests.rs:238` [assert]: `assert_eq!(`
- `/app/crates/contextra-graph/src/community/tests.rs:244` [assert]: `assert_ne!(`
- `/app/crates/contextra-graph/src/community/tests.rs:261` [unwrap]: `self.0.lock().unwrap().push(visitor.0); // unwrap allowed`
- `/app/crates/contextra-graph/fuzz/fuzz_targets/fuzz_graph_persist.rs:190` [assert]: `assert!(`
- `/app/crates/contextra-graph/fuzz/fuzz_targets/fuzz_graph_persist.rs:201` [assert]: `assert!(`
### `contextra-infer-candle`

- `/app/crates/contextra-infer-candle/src/inference/tests.rs:60` [unwrap]: `.unwrap();`
- `/app/crates/contextra-infer-candle/src/inference/tests.rs:64` [assert]: `assert_eq!(client.fingerprint(), &fingerprint);`
- `/app/crates/contextra-infer-candle/src/inference/tests.rs:66` [unwrap]: `let output = client.generate("Hello world").await.unwrap();`
- `/app/crates/contextra-infer-candle/src/inference/tests.rs:67` [assert]: `assert_eq!(output, "Hello world -> Generated Completion");`
- `/app/crates/contextra-infer-candle/src/inference/tests.rs:97` [unwrap]: `.unwrap();`
- `/app/crates/contextra-infer-candle/src/inference/tests.rs:102` [unwrap]: `let sync_resp = client.generate("Hello world").await.unwrap();`
- `/app/crates/contextra-infer-candle/src/inference/tests.rs:107` [unwrap]: `let chunk = chunk_res.unwrap();`
- `/app/crates/contextra-infer-candle/src/inference/tests.rs:111` [assert]: `assert_eq!(sync_resp, assembled);`
- `/app/crates/contextra-infer-candle/src/inference/tests.rs:136` [unwrap]: `let tokenizer = tokenizers::Tokenizer::from_bytes(tokenizer_bytes.as_bytes()).unwrap();`
- `/app/crates/contextra-infer-candle/src/inference/tests.rs:149` [unwrap]: `let tenant = TenantId::try_new(1).unwrap();`
- `/app/crates/contextra-infer-candle/src/inference/tests.rs:153` [unwrap]: `.unwrap();`
- `/app/crates/contextra-infer-candle/src/inference/tests.rs:157` [unwrap]: `.unwrap();`
- `/app/crates/contextra-infer-candle/src/inference/tests.rs:159` [assert]: `assert_eq!(`
- `/app/crates/contextra-infer-candle/src/inference/tests.rs:184` [unwrap]: `let tokenizer = tokenizers::Tokenizer::from_bytes(tokenizer_bytes.as_bytes()).unwrap();`
- `/app/crates/contextra-infer-candle/src/inference/tests.rs:198` [unwrap]: `let tenant = TenantId::try_new(1).unwrap();`
- `/app/crates/contextra-infer-candle/src/inference/tests.rs:202` [unwrap]: `.unwrap();`
- `/app/crates/contextra-infer-candle/src/inference/tests.rs:204` [assert]: `assert_eq!(`
- `/app/crates/contextra-infer-candle/src/inference/tests.rs:208` [assert]: `assert_eq!(`
- `/app/crates/contextra-infer-candle/src/inference/tests.rs:236` [unwrap]: `let tokenizer = tokenizers::Tokenizer::from_bytes(tokenizer_bytes.as_bytes()).unwrap();`
- `/app/crates/contextra-infer-candle/src/inference/tests.rs:250` [unwrap]: `let tenant = TenantId::try_new(42).unwrap();`
- `/app/crates/contextra-infer-candle/src/inference/tests.rs:253` [assert]: `assert_eq!(client_with_adapter.prefill_count(), 0);`
- `/app/crates/contextra-infer-candle/src/inference/tests.rs:254` [assert]: `assert_eq!(client_with_adapter.prefill_skip_count(), 0);`
- `/app/crates/contextra-infer-candle/src/inference/tests.rs:260` [unwrap]: `.unwrap();`
- `/app/crates/contextra-infer-candle/src/inference/tests.rs:262` [assert]: `assert_eq!(`
- `/app/crates/contextra-infer-candle/src/inference/tests.rs:266` [assert]: `assert_eq!(`
- `/app/crates/contextra-infer-candle/src/inference/tests.rs:271` [assert]: `assert_eq!(`
- `/app/crates/contextra-infer-candle/src/inference/tests.rs:276` [assert]: `assert_eq!(adapter.consultation_count(), 2);`
- `/app/crates/contextra-infer-candle/src/inference/tests.rs:283` [expect]: `.expect("Segment 1 must be cached");`
- `/app/crates/contextra-infer-candle/src/inference/tests.rs:286` [expect]: `.expect("Segment 2 must be cached");`
- `/app/crates/contextra-infer-candle/src/inference/tests.rs:287` [assert]: `assert_eq!(`
- `/app/crates/contextra-infer-candle/src/inference/tests.rs:291` [assert]: `assert_eq!(`
- `/app/crates/contextra-infer-candle/src/inference/tests.rs:300` [unwrap]: `.unwrap();`
- `/app/crates/contextra-infer-candle/src/inference/tests.rs:302` [assert]: `assert_eq!(second_res, first_res);`
- `/app/crates/contextra-infer-candle/src/inference/tests.rs:303` [assert]: `assert_eq!(`
- `/app/crates/contextra-infer-candle/src/inference/tests.rs:308` [assert]: `assert_eq!(`
- `/app/crates/contextra-infer-candle/src/inference/tests.rs:313` [assert]: `assert_eq!(`
- `/app/crates/contextra-infer-candle/src/inference/tests.rs:339` [unwrap]: `let tokenizer = tokenizers::Tokenizer::from_bytes(tokenizer_bytes.as_bytes()).unwrap();`
- `/app/crates/contextra-infer-candle/src/inference/tests.rs:351` [unwrap]: `let tenant = TenantId::try_new(88).unwrap();`
- `/app/crates/contextra-infer-candle/src/inference/tests.rs:357` [unwrap]: `.unwrap();`
- `/app/crates/contextra-infer-candle/src/inference/tests.rs:358` [assert]: `assert_eq!(client.prefill_count(), 1);`
- `/app/crates/contextra-infer-candle/src/inference/tests.rs:359` [assert]: `assert_eq!(client.prefill_skip_count(), 0);`
- `/app/crates/contextra-infer-candle/src/inference/tests.rs:365` [unwrap]: `.unwrap();`
- `/app/crates/contextra-infer-candle/src/inference/tests.rs:366` [assert]: `assert_eq!(client.prefill_count(), 2);`
- `/app/crates/contextra-infer-candle/src/inference/tests.rs:367` [assert]: `assert_eq!(`
### `contextra-infer-ollama`

- `/app/crates/contextra-infer-ollama/src/client/tests.rs:14` [assert]: `assert!(res.is_ok());`
- `/app/crates/contextra-infer-ollama/src/client/tests.rs:21` [unwrap]: `let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();`
- `/app/crates/contextra-infer-ollama/src/client/tests.rs:22` [unwrap]: `let addr = listener.local_addr().unwrap();`
- `/app/crates/contextra-infer-ollama/src/client/tests.rs:47` [assert]: `assert_eq!(client.model_id(), DEFAULT_EMBED_MODEL);`
- `/app/crates/contextra-infer-ollama/src/client/tests.rs:50` [unwrap]: `let res = client.synthesize_segment(&texts).await.unwrap();`
- `/app/crates/contextra-infer-ollama/src/client/tests.rs:51` [assert]: `assert_eq!(res, "Synthetisierte Zusammenfassung");`
- `/app/crates/contextra-infer-ollama/src/client/tests.rs:56` [unwrap]: `let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap(); // unwrap`
- `/app/crates/contextra-infer-ollama/src/client/tests.rs:57` [unwrap]: `let addr = listener.local_addr().unwrap(); // unwrap`
- `/app/crates/contextra-infer-ollama/src/client/tests.rs:85` [unwrap]: `.unwrap(); // unwrap`
- `/app/crates/contextra-infer-ollama/src/client/tests.rs:86` [assert]: `assert_eq!(text, "Generierter Kontext-Präfix");`
- `/app/crates/contextra-infer-ollama/src/client/tests.rs:93` [assert]: `assert!(matches!(res, Err(ContextraError::InvalidInput(_))));`
- `/app/crates/contextra-infer-ollama/src/client/tests.rs:98` [unwrap]: `let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap(); // unwrap`
- `/app/crates/contextra-infer-ollama/src/client/tests.rs:99` [unwrap]: `let addr = listener.local_addr().unwrap(); // unwrap`
- `/app/crates/contextra-infer-ollama/src/client/tests.rs:118` [assert]: `assert!(client.is_available().await);`
- `/app/crates/contextra-infer-ollama/src/client/tests.rs:121` [assert]: `assert!(!dead_client.is_available().await);`
- `/app/crates/contextra-infer-ollama/src/client/tests.rs:129` [unwrap]: `let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap(); // unwrap`
- `/app/crates/contextra-infer-ollama/src/client/tests.rs:130` [unwrap]: `let addr = listener.local_addr().unwrap(); // unwrap`
- `/app/crates/contextra-infer-ollama/src/client/tests.rs:159` [unwrap]: `let res = client.embed("nomic-embed-text", "hello").await.unwrap(); // unwrap`
- `/app/crates/contextra-infer-ollama/src/client/tests.rs:160` [assert]: `assert_eq!(res, vec![0.1, 0.2]);`
- `/app/crates/contextra-infer-ollama/src/client/tests.rs:161` [assert]: `assert_eq!(attempts.load(Ordering::SeqCst), 2);`
### `contextra-infer-onnx`

- `/app/crates/contextra-infer-onnx/src/reranker/tests.rs:37` [unwrap]: `let results = reranker.rerank("query", &candidates).await.unwrap(); // unwrap`
- `/app/crates/contextra-infer-onnx/src/reranker/tests.rs:38` [assert]: `assert_eq!(results.len(), 3);`
- `/app/crates/contextra-infer-onnx/src/reranker/tests.rs:48` [unwrap]: `let results = reranker.rerank("query", &[]).await.unwrap(); // unwrap`
- `/app/crates/contextra-infer-onnx/src/reranker/tests.rs:49` [assert]: `assert!(results.is_empty());`
- `/app/crates/contextra-infer-onnx/src/reranker/tests.rs:60` [unwrap]: `let results = reranker.rerank("query", &candidates).await.unwrap(); // unwrap`
- `/app/crates/contextra-infer-onnx/src/reranker/tests.rs:63` [assert]: `assert!(first.score >= second.score);`
- `/app/crates/contextra-infer-onnx/src/reranker/tests.rs:77` [assert]: `assert!(res.is_err());`
- `/app/crates/contextra-infer-onnx/src/reranker/tests.rs:79` [assert]: `assert!(matches!(err, ContextraError::InvalidInput(_)));`
- `/app/crates/contextra-infer-onnx/src/reranker/tests.rs:80` [assert]: `assert!(err.to_string().contains("exceeds maximum allowed limit"));`
- `/app/crates/contextra-infer-onnx/src/reranker/tests.rs:82` [panic]: `panic!("Expected InvalidInput error for oversized candidate batch");`
- `/app/crates/contextra-infer-onnx/src/reranker/tests.rs:111` [unwrap]: `let res = handle.await.unwrap(); // unwrap`
- `/app/crates/contextra-infer-onnx/src/reranker/tests.rs:112` [assert]: `assert!(res.is_ok());`
- `/app/crates/contextra-infer-onnx/src/reranker/tests.rs:113` [unwrap]: `assert_eq!(res.unwrap().len(), 10); // unwrap`
- `/app/crates/contextra-infer-onnx/src/reranker/tests.rs:113` [assert]: `assert_eq!(res.unwrap().len(), 10); // unwrap`
### `contextra-router`

- `/app/crates/contextra-router/src/router/tests.rs:27` [unwrap]: `let dir = tempfile::tempdir().unwrap();`
- `/app/crates/contextra-router/src/router/tests.rs:34` [unwrap]: `.unwrap();`
- `/app/crates/contextra-router/src/router/tests.rs:35` [unwrap]: `let collection = db.collection("default").await.unwrap();`
- `/app/crates/contextra-router/src/router/tests.rs:52` [assert]: `assert_eq!(id1_a.inner(), 0);`
- `/app/crates/contextra-router/src/router/tests.rs:53` [assert]: `assert_eq!(id2_a.inner(), 0);`
- `/app/crates/contextra-router/src/router/tests.rs:58` [assert]: `assert_eq!(id1_b.inner(), 1);`
- `/app/crates/contextra-router/src/router/tests.rs:59` [assert]: `assert_eq!(id2_b.inner(), 1);`
- `/app/crates/contextra-router/src/router/tests.rs:93` [assert]: `assert_eq!(stats.len(), 2);`
- `/app/crates/contextra-router/src/router/tests.rs:94` [assert]: `assert_eq!(stats.get("p1").map(|s| s.times_selected), Some(0));`
- `/app/crates/contextra-router/src/router/tests.rs:95` [assert]: `assert_eq!(stats.get("p1").map(|s| s.calibrated_min_score), Some(0.5));`
- `/app/crates/contextra-router/src/router/tests.rs:96` [assert]: `assert_eq!(stats.get("p1").map(|s| s.original_min_score), Some(0.5));`
- `/app/crates/contextra-router/src/router/tests.rs:97` [assert]: `assert_eq!(stats.get("p2").map(|s| s.times_selected), Some(0));`
- `/app/crates/contextra-router/src/router/tests.rs:98` [assert]: `assert_eq!(stats.get("p2").map(|s| s.calibrated_min_score), Some(0.8));`
- `/app/crates/contextra-router/src/router/tests.rs:99` [assert]: `assert_eq!(stats.get("p2").map(|s| s.original_min_score), Some(0.8));`
### `contextra-sandbox`

- `/app/crates/contextra-sandbox/src/executor/tests.rs:13` [assert]: `assert!(matches!(result, Err(SandboxError::InvalidModule(_))));`
- `/app/crates/contextra-sandbox/src/executor/tests.rs:39` [assert]: `assert!(`
- `/app/crates/contextra-sandbox/src/executor/tests.rs:56` [assert]: `assert!(`
- `/app/crates/contextra-sandbox/src/executor/tests.rs:85` [assert]: `assert!(`
- `/app/crates/contextra-sandbox/src/executor/tests.rs:113` [assert]: `assert!(`
- `/app/crates/contextra-sandbox/src/executor/tests.rs:149` [assert]: `assert_eq!(&output.stdout[..], b"hello");`
- `/app/crates/contextra-sandbox/src/executor/tests.rs:158` [assert]: `assert!(output_no_stdout.stdout.is_empty());`
- `/app/crates/contextra-sandbox/src/executor/tests.rs:187` [assert]: `assert_eq!(&output.stderr[..], b"error msg");`
- `/app/crates/contextra-sandbox/src/executor/tests.rs:213` [assert]: `assert!(`
- `/app/crates/contextra-sandbox/src/executor/tests.rs:242` [assert]: `assert!(`
### `contextra-store`

- `/app/crates/contextra-store/src/memtable.rs:48` [assert]: `const _: () = assert!(TOMBSTONE_BIT == 1u64 << 63);`
- `/app/crates/contextra-store/src/scanner/sstable.rs:35` [expect]: `.expect("Failed to create tokio runtime for scanner");`
- `/app/crates/contextra-store/src/scanner/sstable.rs:47` [expect]: `.expect("Failed to create tokio runtime for scanner");`
- `/app/crates/contextra-store/src/scanner/tests.rs:22` [expect]: `let dir = tempdir().expect("tempdir creation");`
- `/app/crates/contextra-store/src/scanner/tests.rs:27` [expect]: `let storage = Arc::new(LsmStorage::open(config).await.expect("LsmStorage open"));`
- `/app/crates/contextra-store/src/scanner/tests.rs:35` [unwrap]: `let tenant_id = TenantId::try_new(1).unwrap();`
- `/app/crates/contextra-store/src/scanner/tests.rs:48` [unwrap]: `storage.put(tx, &key1, b"val1").await.unwrap();`
- `/app/crates/contextra-store/src/scanner/tests.rs:49` [unwrap]: `storage.put(tx, &key2, b"val2").await.unwrap();`
- `/app/crates/contextra-store/src/scanner/tests.rs:50` [unwrap]: `storage.commit(tx).await.unwrap();`
- `/app/crates/contextra-store/src/scanner/tests.rs:53` [unwrap]: `storage.flush().await.unwrap();`
- `/app/crates/contextra-store/src/scanner/tests.rs:60` [unwrap]: `.unwrap();`
- `/app/crates/contextra-store/src/scanner/tests.rs:61` [unwrap]: `storage.commit(tx_del).await.unwrap();`
- `/app/crates/contextra-store/src/scanner/tests.rs:73` [unwrap]: `.unwrap();`
- `/app/crates/contextra-store/src/scanner/tests.rs:77` [unwrap]: `.unwrap();`
- `/app/crates/contextra-store/src/scanner/tests.rs:79` [assert]: `assert!(`
- `/app/crates/contextra-store/src/scanner/tests.rs:83` [assert]: `assert!(`
- `/app/crates/contextra-store/src/scanner/tests.rs:93` [unwrap]: `let tenant_id = TenantId::try_new(1).unwrap();`
- `/app/crates/contextra-store/src/scanner/tests.rs:98` [unwrap]: `storage.put(tx, &key, b"value_to_purge").await.unwrap();`
- `/app/crates/contextra-store/src/scanner/tests.rs:99` [unwrap]: `storage.commit(tx).await.unwrap();`
- `/app/crates/contextra-store/src/scanner/tests.rs:101` [unwrap]: `storage.flush().await.unwrap();`
- `/app/crates/contextra-store/src/scanner/tests.rs:107` [unwrap]: `.unwrap();`
- `/app/crates/contextra-store/src/scanner/tests.rs:108` [unwrap]: `storage.commit(tx_del).await.unwrap();`
- `/app/crates/contextra-store/src/scanner/tests.rs:116` [expect]: `.expect("purge should succeed");`
- `/app/crates/contextra-store/src/scanner/tests.rs:123` [expect]: `.expect("scan should succeed after purge");`
- `/app/crates/contextra-store/src/scanner/tests.rs:125` [assert]: `assert_eq!(`
- `/app/crates/contextra-store/src/scanner/tests.rs:131` [assert]: `assert!(`
- `/app/crates/contextra-store/src/scanner/tests.rs:143` [unwrap]: `let tenant_id = TenantId::try_new(2).unwrap();`
- `/app/crates/contextra-store/src/scanner/tests.rs:156` [unwrap]: `.unwrap();`
- `/app/crates/contextra-store/src/scanner/tests.rs:157` [unwrap]: `builder.add(&key, b"sabotage_value", 1, 1).await.unwrap();`
- `/app/crates/contextra-store/src/scanner/tests.rs:158` [unwrap]: `builder.finish().await.unwrap();`
- `/app/crates/contextra-store/src/scanner/tests.rs:168` [unwrap]: `.unwrap();`
- `/app/crates/contextra-store/src/scanner/tests.rs:170` [assert]: `assert!(`
- `/app/crates/contextra-store/src/scanner/tests.rs:179` [unwrap]: `let tenant_id = TenantId::try_new(3).unwrap();`
- `/app/crates/contextra-store/src/scanner/tests.rs:189` [assert]: `assert!(`
- `/app/crates/contextra-store/src/scanner/tests.rs:194` [assert]: `assert!(`
- `/app/crates/contextra-store/src/scanner/tests.rs:205` [unwrap]: `let tenant_id = TenantId::try_new(4).unwrap();`
- `/app/crates/contextra-store/src/scanner/tests.rs:212` [unwrap]: `storage.put(tx1, &key_wal, b"wal_data").await.unwrap();`
- `/app/crates/contextra-store/src/scanner/tests.rs:213` [unwrap]: `storage.commit(tx1).await.unwrap();`
- `/app/crates/contextra-store/src/scanner/tests.rs:223` [unwrap]: `.unwrap();`
- `/app/crates/contextra-store/src/scanner/tests.rs:225` [assert]: `assert!(`
- `/app/crates/contextra-store/src/scanner/tests.rs:232` [unwrap]: `purgers[0].purge(&ctx, &target).unwrap();`
- `/app/crates/contextra-store/src/scanner/tests.rs:236` [unwrap]: `.unwrap();`
- `/app/crates/contextra-store/src/scanner/tests.rs:238` [assert]: `assert_eq!(`
- `/app/crates/contextra-store/src/lsm/tests.rs:13` [expect]: `let tmp = TempDir::new().expect("temp dir");`
- `/app/crates/contextra-store/src/lsm/tests.rs:23` [expect]: `let storage = LsmStorage::new(config).await.expect("create storage");`
- `/app/crates/contextra-store/src/compaction/tests.rs:23` [unwrap]: `let mut builder = SstableBuilder::create(&path).await.unwrap();`
- `/app/crates/contextra-store/src/compaction/tests.rs:25` [unwrap]: `builder.add(k, v, seq, 0).await.unwrap();`
- `/app/crates/contextra-store/src/compaction/tests.rs:27` [unwrap]: `builder.finish().await.unwrap();`
- `/app/crates/contextra-store/src/compaction/tests.rs:31` [unwrap]: `.unwrap(),`
- `/app/crates/contextra-store/src/wal/hmac.rs:217` [panic]: `Err(e) => panic!("Legacy WAL key support is disabled or unavailable: {e}"),`
- `/app/crates/contextra-store/fuzz/fuzz_targets/wal_mutation_chaos.rs:290` [assert]: `assert!(`
- `/app/crates/contextra-store/fuzz/fuzz_targets/fuzz_compaction_interleave.rs:102` [assert]: `assert_eq!(`
- `/app/crates/contextra-store/fuzz/fuzz_targets/fuzz_compaction_interleave.rs:111` [panic]: `panic!(`
- `/app/crates/contextra-store/fuzz/fuzz_targets/fuzz_compaction_interleave.rs:117` [panic]: `panic!("Read error during oracle validation: {:?}", e);`
- `/app/crates/contextra-store/fuzz/fuzz_targets/fuzz_memtable_concurrent.rs:106` [panic]: `panic!("Deadlock or execution timeout detected in fuzz_memtable_concurrent");`
### `contextra-testkit`

- `/app/crates/contextra-testkit/src/conformance/macros.rs:19` [panic]: `panic!("{}", failure);`
### `contextra-types`

- `/app/crates/contextra-types/src/types/domain/ids.rs:506` [assert]: `const _: () = assert!(`
- `/app/crates/contextra-types/src/types/domain/ids.rs:510` [assert]: `const _: () = assert!(TxId::INTERNAL_BASE < u64::MAX);`
### `contextra-vector`

- `/app/crates/contextra-vector/src/diskann/build.rs:759` [assert]: `assert!(`
- `/app/crates/contextra-vector/src/hnsw/tests.rs:34` [expect]: `.expect("valid config");`
- `/app/crates/contextra-vector/src/hnsw/tests.rs:39` [expect]: `let idx1 = HnswIndex::try_new_with_rng(config.clone(), rng1).expect("idx1 creation");`
- `/app/crates/contextra-vector/src/hnsw/tests.rs:40` [expect]: `let idx2 = HnswIndex::try_new_with_rng(config, rng2).expect("idx2 creation");`
- `/app/crates/contextra-vector/src/hnsw/tests.rs:52` [expect]: `idx1.insert(tx, doc_id, v).await.expect("idx1 insert");`
- `/app/crates/contextra-vector/src/hnsw/tests.rs:53` [expect]: `idx2.insert(tx, doc_id, v).await.expect("idx2 insert");`
- `/app/crates/contextra-vector/src/hnsw/tests.rs:55` [expect]: `idx1.commit(tx).await.expect("idx1 commit");`
- `/app/crates/contextra-vector/src/hnsw/tests.rs:56` [expect]: `idx2.commit(tx).await.expect("idx2 commit");`
- `/app/crates/contextra-vector/src/hnsw/tests.rs:62` [assert]: `assert_eq!(`
- `/app/crates/contextra-vector/src/hnsw/tests.rs:67` [assert]: `assert_eq!(`
- `/app/crates/contextra-vector/src/hnsw/tests.rs:74` [expect]: `let res1 = idx1.search(&query, 10).await.expect("idx1 search");`
- `/app/crates/contextra-vector/src/hnsw/tests.rs:75` [expect]: `let res2 = idx2.search(&query, 10).await.expect("idx2 search");`
- `/app/crates/contextra-vector/src/hnsw/tests.rs:77` [assert]: `assert_eq!(res1.len(), res2.len());`
- `/app/crates/contextra-vector/src/hnsw/tests.rs:79` [assert]: `assert_eq!(r1.doc_id, r2.doc_id);`
- `/app/crates/contextra-vector/src/hnsw/tests.rs:80` [assert]: `assert_eq!(`
- `/app/crates/contextra-vector/fuzz/fuzz_targets/fuzz_hnsw_persistence.rs:43` [unwrap]: `let f = f32::from_le_bytes(chunk.try_into().unwrap());`
### `contextra-wire`

- `/app/crates/contextra-wire/build.rs:46` [assert]: `assert!(status.success(), "flatc failed to generate code");`
- `/app/crates/contextra-wire/src/contextra_generated.rs:143` [unwrap]: `unsafe { self._tab.get::<i8>(Embedding::VT_METRIC, Some(0)).unwrap() }`
- `/app/crates/contextra-wire/src/contextra_generated.rs:338` [unwrap]: `.unwrap()`
- `/app/crates/contextra-wire/src/contextra_generated.rs:572` [unwrap]: `.unwrap()`
- `/app/crates/contextra-wire/src/contextra_generated.rs:586` [unwrap]: `.unwrap()`
- `/app/crates/contextra-wire/src/contextra_generated.rs:961` [unwrap]: `unsafe { self._tab.get::<u32>(RoleId::VT_ID, Some(0)).unwrap() }`
- `/app/crates/contextra-wire/src/contextra_generated.rs:1083` [unwrap]: `unsafe { self._tab.get::<u64>(HyperEdgeId::VT_ID, Some(0)).unwrap() }`
- `/app/crates/contextra-wire/src/contextra_generated.rs:1211` [unwrap]: `unsafe { self._tab.get::<u32>(RoleBinding::VT_ROLE, Some(0)).unwrap() }`
- `/app/crates/contextra-wire/src/contextra_generated.rs:1224` [unwrap]: `.unwrap()`
- `/app/crates/contextra-wire/src/contextra_generated.rs:1399` [unwrap]: `unsafe { self._tab.get::<u64>(HyperEdge::VT_ID, Some(0)).unwrap() }`
- `/app/crates/contextra-wire/src/contextra_generated.rs:1412` [unwrap]: `.unwrap()`
- `/app/crates/contextra-wire/src/contextra_generated.rs:1443` [unwrap]: `.unwrap()`
- `/app/crates/contextra-wire/src/contextra_generated.rs:1457` [unwrap]: `.unwrap()`
- `/app/crates/contextra-wire/src/contextra_generated.rs:1471` [unwrap]: `.unwrap()`
- `/app/crates/contextra-wire/src/contextra_generated.rs:1485` [unwrap]: `.unwrap()`
- `/app/crates/contextra-wire/src/contextra_generated.rs:1499` [unwrap]: `.unwrap()`
- `/app/crates/contextra-wire/src/contextra_generated.rs:1513` [unwrap]: `.unwrap()`
