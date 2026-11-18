# Zero-Panic & Unsafe Indexing Audit Report

> Generiert automatisch durch `tools/zero_panic_analyzer.py`.
> Workspace Root: `/home/freddy/Projekte/contextra`
> Gesamte gefundene Produktivcode-Verletzungen: **307**

## Crate Übersicht (Produktivpfade)

| Crate | Prod unwrap | Prod expect | Prod panic | Prod todo | Prod assert | Test Findings |
|---|---|---|---|---|---|---|
| `contextra` | 0 | 0 | 0 | 0 | 0 | 185 |
| `contextra-checkpoint` | 0 | 0 | 0 | 0 | 0 | 946 |
| `contextra-crypto` | 0 | 0 | 0 | 0 | 1 | 1027 |
| `contextra-db` | 107 | 1 | 6 | 0 | 64 | 2173 |
| `contextra-durable-fs` | 0 | 0 | 0 | 0 | 0 | 231 |
| `contextra-engine` | 0 | 0 | 0 | 0 | 0 | 655 |
| `contextra-env` | 0 | 0 | 0 | 0 | 0 | 35 |
| `contextra-erasure` | 0 | 0 | 0 | 0 | 0 | 1466 |
| `contextra-graph` | 49 | 0 | 0 | 0 | 25 | 2650 |
| `contextra-ids` | 0 | 0 | 0 | 0 | 4 | 65 |
| `contextra-mcp` | 0 | 0 | 0 | 0 | 0 | 829 |
| `contextra-mvcc` | 0 | 0 | 0 | 0 | 0 | 541 |
| `contextra-ports` | 0 | 0 | 0 | 0 | 0 | 183 |
| `contextra-privacy` | 0 | 0 | 0 | 0 | 0 | 590 |
| `contextra-rank` | 0 | 0 | 0 | 0 | 0 | 509 |
| `contextra-simd` | 0 | 0 | 0 | 0 | 0 | 458 |
| `contextra-store` | 31 | 4 | 3 | 0 | 12 | 5683 |
| `contextra-sys` | 0 | 0 | 0 | 0 | 0 | 153 |
| `contextra-testkit` | 0 | 0 | 0 | 0 | 0 | 390 |
| `contextra-text` | 0 | 0 | 0 | 0 | 0 | 833 |
| `contextra-types` | 0 | 0 | 0 | 0 | 0 | 1013 |
| `contextra-vault` | 0 | 0 | 0 | 0 | 0 | 51 |
| `contextra-vector` | 0 | 0 | 0 | 0 | 0 | 2334 |
| `contextra-bench` | 0 | 0 | 0 | 0 | 0 | 231 |

## Detaillierte Produktiv-Fundstellen

### `contextra-crypto`

- `/home/freddy/Projekte/contextra/crates/contextra-crypto/fuzz/fuzz_targets/wal_hmac_chain_verify_fuzz.rs:144` [assert]: `assert!(`
### `contextra-db`

- `/home/freddy/Projekte/contextra/crates/contextra-db/fuzz/fuzz_targets/rrf_fusion.rs:78` [assert]: `assert!(`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/extraction/tests.rs:58` [unwrap]: `.unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/extraction/tests.rs:60` [assert]: `assert_eq!(triples.len(), 1);`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/extraction/tests.rs:61` [assert]: `assert_eq!(`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/extraction/tests.rs:71` [assert]: `assert_eq!(generator.call_count.load(Ordering::SeqCst), 1);`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/extraction/tests.rs:86` [assert]: `assert!(res.is_err());`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/extraction/tests.rs:89` [assert]: `assert_eq!(limit, 2);`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/extraction/tests.rs:90` [assert]: `assert!(context.contains("exceeds max_llm_calls_per_cycle limit"));`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/extraction/tests.rs:92` [panic]: `err => panic!("Expected LimitExceeded error, got {err:?}"),`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/extraction/tests.rs:94` [assert]: `assert_eq!(generator.call_count.load(Ordering::SeqCst), 0);`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/extraction/tests.rs:105` [assert]: `assert!(res.is_err());`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/extraction/tests.rs:108` [assert]: `assert!(msg.contains("Failed to parse LLM entity extraction response as JSON"));`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/extraction/tests.rs:110` [panic]: `err => panic!("Expected ParseError, got {err:?}"),`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/transaction/tests.rs:20` [unwrap]: `let dir = tempdir().unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/transaction/tests.rs:25` [unwrap]: `let storage = Arc::new(LsmStorage::new(lsm_config).await.unwrap());`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/transaction/tests.rs:31` [unwrap]: `.unwrap(),`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/transaction/tests.rs:52` [unwrap]: `let tx_id = col.allocate_tx().unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/transaction/tests.rs:62` [assert]: `assert!(commit_res.is_ok());`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/transaction/tests.rs:70` [unwrap]: `let tx_id = col.allocate_tx().unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/transaction/tests.rs:80` [assert]: `assert!(rollback_res.is_ok());`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/transaction/tests.rs:86` [unwrap]: `let tx_id = col.allocate_tx().unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/transaction/tests.rs:96` [assert]: `assert!(results.is_ok());`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/transaction/tests.rs:97` [unwrap]: `assert!(results.unwrap().is_empty());`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/transaction/tests.rs:97` [assert]: `assert!(results.unwrap().is_empty());`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/query_builder/tests.rs:110` [unwrap]: `let dir = TempDir::new().unwrap(); // unwrap`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/query_builder/tests.rs:115` [unwrap]: `let storage = Arc::new(LsmStorage::new(lsm_config).await.unwrap()); // unwrap`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/query_builder/tests.rs:121` [unwrap]: `let index = Arc::new(HnswIndex::try_new(hnsw_config).unwrap()); // unwrap`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/query_builder/tests.rs:141` [unwrap]: `.unwrap(); // unwrap`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/query_builder/tests.rs:144` [unwrap]: `.unwrap(); // unwrap`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/query_builder/tests.rs:147` [unwrap]: `let legacy = col.search(&[1.0, 0.0, 0.0, 0.0], 2).await.unwrap(); // unwrap`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/query_builder/tests.rs:154` [unwrap]: `.unwrap(); // unwrap`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/query_builder/tests.rs:156` [assert]: `assert_eq!(builder_res.len(), legacy.len());`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/query_builder/tests.rs:157` [assert]: `assert_eq!(builder_res[0].id, legacy[0].id);`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/query_builder/tests.rs:158` [assert]: `assert_eq!(builder_res[1].id, legacy[1].id);`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/query_builder/tests.rs:166` [unwrap]: `.unwrap(); // unwrap`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/query_builder/tests.rs:169` [unwrap]: `.unwrap(); // unwrap`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/query_builder/tests.rs:180` [unwrap]: `.unwrap(); // unwrap`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/query_builder/tests.rs:189` [unwrap]: `.unwrap(); // unwrap`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/query_builder/tests.rs:191` [assert]: `assert_eq!(builder_res.len(), 1);`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/query_builder/tests.rs:192` [assert]: `assert_eq!(builder_res.len(), legacy.len());`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/query_builder/tests.rs:193` [assert]: `assert_eq!(builder_res[0].id, legacy[0].id);`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/query_builder/tests.rs:194` [assert]: `assert_eq!(builder_res[0].id, "doc-1");`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/query_builder/tests.rs:206` [unwrap]: `.unwrap(); // unwrap`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/query_builder/tests.rs:213` [unwrap]: `.unwrap(); // unwrap`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/query_builder/tests.rs:219` [unwrap]: `.unwrap(); // unwrap`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/query_builder/tests.rs:228` [unwrap]: `.unwrap(); // unwrap`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/query_builder/tests.rs:230` [assert]: `assert_eq!(builder_res.len(), legacy.len());`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/query_builder/tests.rs:232` [assert]: `assert_eq!(builder_res[0].id, legacy[0].id);`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/query_builder/tests.rs:245` [unwrap]: `.unwrap(); // unwrap`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/query_builder/tests.rs:247` [unwrap]: `let weights = SignalWeights::new(0.6, 0.4, 0.0).unwrap(); // unwrap`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/query_builder/tests.rs:257` [unwrap]: `.unwrap(); // unwrap`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/query_builder/tests.rs:259` [assert]: `assert!(!builder_res.is_empty());`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/query_builder/tests.rs:260` [assert]: `assert_eq!(builder_res[0].id, "doc-1");`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/query_builder/tests.rs:268` [unwrap]: `.unwrap(); // unwrap`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/query_builder/tests.rs:274` [unwrap]: `.unwrap(); // unwrap`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/query_builder/tests.rs:277` [unwrap]: `let legacy = col.hybrid_search_with_query(&hybrid_query).await.unwrap(); // unwrap`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/query_builder/tests.rs:284` [unwrap]: `.unwrap(); // unwrap`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/query_builder/tests.rs:286` [assert]: `assert_eq!(builder_res.len(), legacy.len());`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/query_builder/tests.rs:287` [assert]: `assert_eq!(builder_res[0].id, legacy[0].id);`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/query_builder/tests.rs:304` [unwrap]: `.unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/query_builder/tests.rs:316` [unwrap]: `.unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/query_builder/tests.rs:318` [assert]: `assert_eq!(res.len(), 10, "Final results truncated to k=10");`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/query_builder/tests.rs:320` [unwrap]: `assert!(res[0].metadata.as_ref().unwrap().get("ce_score").is_some());`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/query_builder/tests.rs:320` [assert]: `assert!(res[0].metadata.as_ref().unwrap().get("ce_score").is_some());`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/query_builder/tests.rs:337` [unwrap]: `.unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/query_builder/tests.rs:351` [unwrap]: `.unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/query_builder/tests.rs:353` [assert]: `assert_eq!(res_default.len(), 100);`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/query_builder/tests.rs:365` [unwrap]: `.unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/query_builder/tests.rs:368` [assert]: `assert_eq!(`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/query_builder/tests.rs:384` [unwrap]: `.unwrap(); // unwrap`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/query_builder/tests.rs:391` [unwrap]: `.unwrap(); // unwrap`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/query_builder/tests.rs:398` [unwrap]: `.unwrap(); // unwrap`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/query_builder/tests.rs:410` [unwrap]: `.unwrap(); // unwrap`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/query_builder/tests.rs:412` [assert]: `assert_eq!(res.len(), 2, "Results must be truncated to k=2");`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/query_builder/tests.rs:415` [expect]: `let meta = item.metadata.as_ref().expect("metadata must exist"); // expect`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/query_builder/tests.rs:416` [assert]: `assert!(`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/query_builder/tests.rs:421` [assert]: `assert!(`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/query_builder/tests.rs:428` [assert]: `assert!(`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/query_builder/tests.rs:444` [unwrap]: `.unwrap(); // unwrap`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/query_builder/tests.rs:451` [unwrap]: `.unwrap(); // unwrap`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/query_builder/tests.rs:464` [assert]: `assert!(`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/query_builder/tests.rs:468` [unwrap]: `let results = res.unwrap(); // unwrap`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/query_builder/tests.rs:469` [assert]: `assert_eq!(results.len(), 2);`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/query_builder/tests.rs:472` [assert]: `assert!(`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/query_builder/tests.rs:485` [unwrap]: `.unwrap(); // unwrap`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/query_builder/tests.rs:488` [unwrap]: `.unwrap(); // unwrap`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/query_builder/tests.rs:498` [unwrap]: `.unwrap(); // unwrap`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/query_builder/tests.rs:500` [assert]: `assert_eq!(res.len(), 2);`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/query_builder/tests.rs:503` [assert]: `assert!(`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:15` [unwrap]: `let dir = tempfile::tempdir().unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:24` [unwrap]: `.unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:28` [unwrap]: `.unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:34` [unwrap]: `.unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:39` [assert]: `assert!(res_default.is_err());`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:44` [unwrap]: `let dir = tempfile::tempdir().unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:53` [unwrap]: `.unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:54` [unwrap]: `let col = db.collection("test_scan_prefix_equal").await.unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:59` [unwrap]: `.unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:62` [unwrap]: `let results = col.scan_prefix("", Some(10)).await.unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:63` [assert]: `assert_eq!(results.len(), 10);`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:65` [unwrap]: `let results_larger = col.scan_prefix("", Some(20)).await.unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:66` [assert]: `assert_eq!(results_larger.len(), 10);`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:71` [unwrap]: `let dir = tempfile::tempdir().unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:80` [unwrap]: `.unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:81` [unwrap]: `let col = db.collection("test_scan_prefix_exceed").await.unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:86` [unwrap]: `.unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:90` [assert]: `assert!(matches!(`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:96` [assert]: `assert!(res_exact.is_ok());`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:97` [unwrap]: `assert_eq!(res_exact.unwrap().len(), 15);`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:97` [assert]: `assert_eq!(res_exact.unwrap().len(), 15);`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:102` [unwrap]: `let dir = tempfile::tempdir().unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:111` [unwrap]: `.unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:112` [unwrap]: `let col = db.collection("test_scan_default").await.unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:117` [unwrap]: `.unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:123` [unwrap]: `.unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:124` [assert]: `assert_eq!(results.len(), 50);`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:125` [assert]: `assert!(results.len() <= MAX_SCAN_RESULTS);`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:130` [unwrap]: `let dir = tempfile::tempdir().unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:139` [unwrap]: `.unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:140` [unwrap]: `let col = db.collection("test_scan_equal").await.unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:145` [unwrap]: `.unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:151` [unwrap]: `.unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:152` [assert]: `assert_eq!(results.len(), 10);`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:157` [unwrap]: `.unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:158` [assert]: `assert_eq!(results_larger.len(), 10);`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:163` [unwrap]: `let dir = tempfile::tempdir().unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:172` [unwrap]: `.unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:173` [unwrap]: `let col = db.collection("test_scan_exceed").await.unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:178` [unwrap]: `.unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:182` [assert]: `assert!(matches!(`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:188` [assert]: `assert!(res_exact.is_ok());`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:189` [unwrap]: `assert_eq!(res_exact.unwrap().len(), 15);`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:189` [assert]: `assert_eq!(res_exact.unwrap().len(), 15);`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:194` [unwrap]: `let dir = tempfile::tempdir().unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:203` [unwrap]: `.unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:204` [unwrap]: `let collection = Arc::new(db.collection("test_race").await.unwrap());`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:219` [unwrap]: `match res.unwrap() {`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:222` [panic]: `Err(other) => panic!("Unexpected error: {:?}", other),`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:226` [assert]: `assert_eq!(ok_count, 1, "Exactly one task must succeed");`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:227` [assert]: `assert_eq!(`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:235` [unwrap]: `let dir = tempfile::tempdir().unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:244` [unwrap]: `.unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:246` [unwrap]: `let col_a = Arc::new(db.collection("col_a").await.unwrap());`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:247` [unwrap]: `let col_b = Arc::new(db.collection("col_b").await.unwrap());`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:265` [assert]: `assert!(`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:269` [unwrap]: `let put_res = res.unwrap().unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:270` [assert]: `assert!(`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:281` [unwrap]: `let dir = tempfile::tempdir().unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:290` [unwrap]: `.unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:291` [unwrap]: `let collection = Arc::new(db.collection("test_scan_cap").await.unwrap());`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:298` [unwrap]: `.unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:303` [panic]: `Err(e) => panic!("scan_prefix failed: {e:?}"),`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:305` [assert]: `assert_eq!(`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:314` [unwrap]: `let dir = tempfile::tempdir().unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:323` [unwrap]: `.unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:324` [unwrap]: `let col = db.collection("dim_test").await.unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:331` [assert]: `assert!(res.is_err());`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:334` [panic]: `Err(other) => panic!("Expected InvalidInput error, got: {:?}", other),`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:335` [panic]: `Ok(_) => panic!("Expected insert to fail due to dimension mismatch"),`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:338` [assert]: `assert!(`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:347` [unwrap]: `let dir = tempfile::tempdir().unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:356` [unwrap]: `.unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:357` [unwrap]: `let col = Arc::new(db.collection("concurrency_test").await.unwrap());`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:369` [assert]: `assert!(`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:373` [unwrap]: `assert!(res.unwrap().unwrap().is_ok());`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:373` [assert]: `assert!(res.unwrap().unwrap().is_ok());`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:380` [unwrap]: `let dir = tempfile::tempdir().unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:389` [unwrap]: `.unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:390` [unwrap]: `let col = Arc::new(db.collection("delete_lock_test").await.unwrap());`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:395` [unwrap]: `.unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:398` [unwrap]: `col.delete("doc_del_1").await.unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:402` [assert]: `assert!(`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:411` [unwrap]: `let dir = tempfile::tempdir().unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:420` [unwrap]: `.unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:421` [unwrap]: `let col = Arc::new(db.collection("update_lock_test").await.unwrap());`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:426` [unwrap]: `.unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:431` [unwrap]: `.unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/collection/crud/tests.rs:435` [assert]: `assert!(`
### `contextra-graph`

- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/storage.rs:96` [unwrap]: `assert_eq!(storage.get(b"nonexistent").await.unwrap(), None);`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/storage.rs:96` [assert]: `assert_eq!(storage.get(b"nonexistent").await.unwrap(), None);`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/storage.rs:97` [unwrap]: `storage.put(tx, b"key1", b"val1").await.unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/storage.rs:98` [unwrap]: `assert_eq!(storage.get(b"key1").await.unwrap(), Some(Bytes::from_static(b"val1")));`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/storage.rs:98` [assert]: `assert_eq!(storage.get(b"key1").await.unwrap(), Some(Bytes::from_static(b"val1")));`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/storage.rs:99` [unwrap]: `storage.put(tx, b"key1", b"val1_updated").await.unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/storage.rs:100` [unwrap]: `assert_eq!(storage.get(b"key1").await.unwrap(), Some(Bytes::from_static(b"val1_updated")));`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/storage.rs:100` [assert]: `assert_eq!(storage.get(b"key1").await.unwrap(), Some(Bytes::from_static(b"val1_updated")));`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/storage.rs:101` [unwrap]: `storage.delete(tx, b"key1").await.unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/storage.rs:102` [unwrap]: `assert_eq!(storage.get(b"key1").await.unwrap(), None);`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/storage.rs:102` [assert]: `assert_eq!(storage.get(b"key1").await.unwrap(), None);`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/storage.rs:103` [unwrap]: `storage.delete(tx, b"key1").await.unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/storage.rs:106` [unwrap]: `storage.put(tx2, b"pref:c", b"3").await.unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/storage.rs:107` [unwrap]: `storage.put(tx2, b"pref:a", b"1").await.unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/storage.rs:108` [unwrap]: `storage.put(tx2, b"pref:b", b"2").await.unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/storage.rs:109` [unwrap]: `storage.put(tx2, b"other:x", b"9").await.unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/storage.rs:111` [unwrap]: `let scanned = storage.scan_prefix(b"pref:").await.unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/storage.rs:112` [assert]: `assert_eq!(scanned.len(), 3);`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/storage.rs:113` [assert]: `assert_eq!(scanned[0], (b"pref:a".to_vec(), b"1".to_vec()));`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/storage.rs:114` [assert]: `assert_eq!(scanned[1], (b"pref:b".to_vec(), b"2".to_vec()));`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/storage.rs:115` [assert]: `assert_eq!(scanned[2], (b"pref:c".to_vec(), b"3".to_vec()));`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/storage.rs:117` [unwrap]: `let all = storage.scan_prefix(b"").await.unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/storage.rs:118` [assert]: `assert_eq!(all.len(), 4);`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/storage.rs:119` [assert]: `assert_eq!(all[0].0, b"other:x");`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/storage.rs:122` [unwrap]: `storage.put(tx2, &bin_key, b"binary").await.unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/storage.rs:123` [unwrap]: `assert_eq!(storage.get(&bin_key).await.unwrap(), Some(Bytes::from_static(b"binary")));`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/storage.rs:123` [assert]: `assert_eq!(storage.get(&bin_key).await.unwrap(), Some(Bytes::from_static(b"binary")));`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/storage.rs:125` [unwrap]: `storage.commit(tx2).await.unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/storage.rs:126` [unwrap]: `storage.flush().await.unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/storage.rs:127` [unwrap]: `assert!(storage.last_tx_id().await.unwrap().inner() >= 2);`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/storage.rs:127` [assert]: `assert!(storage.last_tx_id().await.unwrap().inner() >= 2);`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/fuzz/fuzz_targets/fuzz_graph_persist.rs:190` [assert]: `assert!(`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/fuzz/fuzz_targets/fuzz_graph_persist.rs:201` [assert]: `assert!(`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/community/tests.rs:28` [unwrap]: `.unwrap(); // unwrap allowed`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/community/tests.rs:35` [unwrap]: `.unwrap(); // unwrap allowed`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/community/tests.rs:39` [unwrap]: `.unwrap(); // unwrap allowed`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/community/tests.rs:43` [unwrap]: `.unwrap(); // unwrap allowed`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/community/tests.rs:47` [unwrap]: `.unwrap(); // unwrap allowed`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/community/tests.rs:48` [unwrap]: `graph.commit(tx).await.unwrap(); // unwrap allowed`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/community/tests.rs:56` [unwrap]: `let run1 = detect_communities(&graph, &config).await.unwrap(); // unwrap allowed`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/community/tests.rs:57` [unwrap]: `let run2 = detect_communities(&graph, &config).await.unwrap(); // unwrap allowed`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/community/tests.rs:59` [assert]: `assert_eq!(`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/community/tests.rs:79` [unwrap]: `.unwrap(); // unwrap allowed`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/community/tests.rs:84` [unwrap]: `.unwrap(); // unwrap allowed`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/community/tests.rs:88` [unwrap]: `.unwrap(); // unwrap allowed`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/community/tests.rs:92` [unwrap]: `.unwrap(); // unwrap allowed`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/community/tests.rs:102` [unwrap]: `.unwrap(); // unwrap allowed`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/community/tests.rs:110` [unwrap]: `.unwrap(); // unwrap allowed`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/community/tests.rs:117` [unwrap]: `.unwrap(); // unwrap allowed`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/community/tests.rs:124` [unwrap]: `.unwrap(); // unwrap allowed`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/community/tests.rs:126` [unwrap]: `graph.commit(tx).await.unwrap(); // unwrap allowed`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/community/tests.rs:129` [unwrap]: `let assignments = detect_communities(&graph, &config).await.unwrap(); // unwrap allowed`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/community/tests.rs:138` [assert]: `assert_eq!(map[&2], c1_community);`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/community/tests.rs:139` [assert]: `assert_eq!(map[&3], c1_community);`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/community/tests.rs:143` [assert]: `assert_eq!(map[&101], c2_community);`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/community/tests.rs:144` [assert]: `assert_eq!(map[&102], c2_community);`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/community/tests.rs:147` [assert]: `assert_ne!(`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/community/tests.rs:174` [unwrap]: `.unwrap(); // unwrap allowed`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/community/tests.rs:179` [unwrap]: `.unwrap(); // unwrap allowed`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/community/tests.rs:183` [unwrap]: `.unwrap(); // unwrap allowed`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/community/tests.rs:187` [unwrap]: `.unwrap(); // unwrap allowed`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/community/tests.rs:197` [unwrap]: `.unwrap(); // unwrap allowed`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/community/tests.rs:202` [unwrap]: `.unwrap(); // unwrap allowed`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/community/tests.rs:206` [unwrap]: `.unwrap(); // unwrap allowed`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/community/tests.rs:210` [unwrap]: `.unwrap(); // unwrap allowed`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/community/tests.rs:219` [unwrap]: `.unwrap(); // unwrap allowed`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/community/tests.rs:221` [unwrap]: `graph.commit(tx).await.unwrap(); // unwrap allowed`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/community/tests.rs:224` [unwrap]: `let assignments = detect_communities(&graph, &config).await.unwrap(); // unwrap allowed`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/community/tests.rs:233` [assert]: `assert_eq!(`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/community/tests.rs:237` [assert]: `assert_eq!(`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/community/tests.rs:244` [assert]: `assert_eq!(`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/community/tests.rs:248` [assert]: `assert_eq!(`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/community/tests.rs:254` [assert]: `assert_ne!(`
- `/home/freddy/Projekte/contextra/crates/contextra-graph/src/community/tests.rs:271` [unwrap]: `self.0.lock().unwrap().push(visitor.0); // unwrap allowed`
### `contextra-ids`

- `/home/freddy/Projekte/contextra/crates/contextra-ids/src/lib.rs:20` [assert]: `assert!(size_of::<DocId>() == 8);`
- `/home/freddy/Projekte/contextra/crates/contextra-ids/src/lib.rs:21` [assert]: `assert!(align_of::<DocId>() == 8);`
- `/home/freddy/Projekte/contextra/crates/contextra-ids/src/lib.rs:26` [assert]: `assert!(size_of::<DocId>() == 16);`
- `/home/freddy/Projekte/contextra/crates/contextra-ids/src/lib.rs:27` [assert]: `assert!(align_of::<DocId>() == 16);`
### `contextra-store`

- `/home/freddy/Projekte/contextra/crates/contextra-store/fuzz/fuzz_targets/fuzz_compaction_interleave.rs:102` [assert]: `assert_eq!(`
- `/home/freddy/Projekte/contextra/crates/contextra-store/fuzz/fuzz_targets/fuzz_compaction_interleave.rs:111` [panic]: `panic!(`
- `/home/freddy/Projekte/contextra/crates/contextra-store/fuzz/fuzz_targets/fuzz_compaction_interleave.rs:117` [panic]: `panic!("Read error during oracle validation: {:?}", e);`
- `/home/freddy/Projekte/contextra/crates/contextra-store/fuzz/fuzz_targets/fuzz_memtable_concurrent.rs:106` [panic]: `panic!("Deadlock or execution timeout detected in fuzz_memtable_concurrent");`
- `/home/freddy/Projekte/contextra/crates/contextra-store/fuzz/fuzz_targets/wal_mutation_chaos.rs:288` [assert]: `assert!(`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/compaction/tests.rs:22` [unwrap]: `let mut builder = SstableBuilder::create(&path).await.unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/compaction/tests.rs:24` [unwrap]: `builder.add(k, v, seq, 0).await.unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/compaction/tests.rs:26` [unwrap]: `builder.finish().await.unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/compaction/tests.rs:30` [unwrap]: `.unwrap(),`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/memtable/tombstone.rs:10` [assert]: `const _: () = assert!(TOMBSTONE_BIT == 1u64 << 63);`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/compat/scanner/tests.rs:24` [expect]: `let dir = tempdir().expect("tempdir creation");`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/compat/scanner/tests.rs:29` [expect]: `let storage = Arc::new(LsmStorage::open(config).await.expect("LsmStorage open"));`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/compat/scanner/tests.rs:37` [unwrap]: `let tenant_id = TenantId::try_new(1).unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/compat/scanner/tests.rs:50` [unwrap]: `storage.put(tx, &key1, b"val1").await.unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/compat/scanner/tests.rs:51` [unwrap]: `storage.put(tx, &key2, b"val2").await.unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/compat/scanner/tests.rs:52` [unwrap]: `storage.commit(tx).await.unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/compat/scanner/tests.rs:55` [unwrap]: `storage.flush().await.unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/compat/scanner/tests.rs:62` [unwrap]: `.unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/compat/scanner/tests.rs:63` [unwrap]: `storage.commit(tx_del).await.unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/compat/scanner/tests.rs:75` [unwrap]: `.unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/compat/scanner/tests.rs:79` [unwrap]: `.unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/compat/scanner/tests.rs:81` [assert]: `assert!(`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/compat/scanner/tests.rs:85` [assert]: `assert!(`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/compat/scanner/tests.rs:95` [unwrap]: `let tenant_id = TenantId::try_new(1).unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/compat/scanner/tests.rs:100` [unwrap]: `storage.put(tx, &key, b"value_to_purge").await.unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/compat/scanner/tests.rs:101` [unwrap]: `storage.commit(tx).await.unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/compat/scanner/tests.rs:103` [unwrap]: `storage.flush().await.unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/compat/scanner/tests.rs:109` [unwrap]: `.unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/compat/scanner/tests.rs:110` [unwrap]: `storage.commit(tx_del).await.unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/compat/scanner/tests.rs:118` [expect]: `.expect("purge should succeed");`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/compat/scanner/tests.rs:125` [expect]: `.expect("scan should succeed after purge");`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/compat/scanner/tests.rs:127` [assert]: `assert_eq!(`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/compat/scanner/tests.rs:133` [assert]: `assert!(`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/compat/scanner/tests.rs:145` [unwrap]: `let tenant_id = TenantId::try_new(2).unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/compat/scanner/tests.rs:158` [unwrap]: `.unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/compat/scanner/tests.rs:159` [unwrap]: `builder.add(&key, b"sabotage_value", 1, 1).await.unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/compat/scanner/tests.rs:160` [unwrap]: `builder.finish().await.unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/compat/scanner/tests.rs:170` [unwrap]: `.unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/compat/scanner/tests.rs:172` [assert]: `assert!(`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/compat/scanner/tests.rs:181` [unwrap]: `let tenant_id = TenantId::try_new(3).unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/compat/scanner/tests.rs:191` [assert]: `assert!(`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/compat/scanner/tests.rs:196` [assert]: `assert!(`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/compat/scanner/tests.rs:207` [unwrap]: `let tenant_id = TenantId::try_new(4).unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/compat/scanner/tests.rs:214` [unwrap]: `storage.put(tx1, &key_wal, b"wal_data").await.unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/compat/scanner/tests.rs:215` [unwrap]: `storage.commit(tx1).await.unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/compat/scanner/tests.rs:225` [unwrap]: `.unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/compat/scanner/tests.rs:227` [assert]: `assert!(`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/compat/scanner/tests.rs:234` [unwrap]: `purgers[0].purge(&ctx, &target).unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/compat/scanner/tests.rs:238` [unwrap]: `.unwrap();`
- `/home/freddy/Projekte/contextra/crates/contextra-store/src/compat/scanner/tests.rs:240` [assert]: `assert_eq!(`
