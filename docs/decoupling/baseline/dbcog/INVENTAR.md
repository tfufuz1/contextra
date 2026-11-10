# Baseline Inventar DC1.0

## 1. volatile_vault.rs
- LOC: 365 in src/volatile_vault.rs + vault_sys.inc (29 LOC)
- contextra_types-Bezüge: DocId, TenantId, TxId
- contextra_sys::vault: LockedRegions, Memory-Locking
- Verwender: contextra-db (prelude, advanced, lib.rs), contextra-agent (clm_scratchpad.rs, context.rs, engine/tests.rs)

## 2. Engine-gebundene Dateien in contextra-cognition
- consolidation_executor.rs (Collection, StoredDocument, AdaptiveDecayController, etc.)
- semantic_aggregation_facade.rs (Collection, consolidate_semantic_hyperedges)
- leanrag_input.rs (Collection)
- maintenance_scheduler.rs (Collection, StoredDocument, AdaptiveDecayController)
- consolidation_locks.rs (Collection)
- context_compaction/ (session.rs, compactor.rs, cleanup.rs: Collection, CommitIntent)
- context.rs (SearchResult, ProvenanceRecord)

## 3. Speicher- und Fremdbindungen in contextra-agent
- contextra_db: Contextra, ContextraConfig, Collection, DistanceMetric, VolatileContextVault, VaultConfig, VaultChunk, SignalModality
- contextra_graph: CsrGraph (audit.rs dev/test)
- contextra_checkpoint: CheckpointCoordinator, CheckpointId (engine.rs)
- contextra_router: RouterEngine, DecisionId, RoutingOutcome (context.rs, engine.rs)
