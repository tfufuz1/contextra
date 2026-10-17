# Triage Manifest: `crates/contextra-engine`

- **Stand origin/main**: `a8daea88`
- **Anzahl Commits**: 75
- **Klassen-Übersicht**: DEL-R0: 1, DEL-R4: 28, PORT-FEAT: 7, PORT-FIX: 7, PORT-REQUIRED: 1, PORT-WIRING: 11, SKIP-DOC: 3, SKIP-INFRA: 1, SKIP-SUPERSEDED: 16

## Commits

| sha | Datum | Titel | Dateien | prod/test/doku-Zeilen | Klasse | Begründung | Abhängigkeit | in main schon vorhanden | Konfliktrivialität/Risiko |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| 0003eece | 2026-10-15 | [Entkopplung contextra-engine] Unbenutzte Abhaengigkeiten und Features entfernen | 18 | +264/-679 prod, +0/-0 test, +0/-0 doku | SKIP-SUPERSEDED | Entkopplungs-Cleanup in Spender (Entfernung von `parking_lot`/Unused Dependencies); in `main` ungestört. | Keine | nein | hoch |
| 830c0399 | 2026-10-15 | [Entkopplung contextra-engine] Remove parking tests W1-15 | 35 | +0/-0 prod, +62/-2965 test, +0/-0 doku | SKIP-SUPERSEDED | Entfernung von `parking`-Tests im Spender-Core-Cut; in `main` verbleiben die Basetests. | Keine | nein | hoch |
| 62bf203a | 2026-10-15 | [Entkopplung engine] Decouple collection from parking logic | 11 | +6/-996 prod, +0/-0 test, +0/-0 doku | SKIP-SUPERSEDED | Entkopplung von `parking`-Logik aus `collection`; im Core-Cut verworfen. | Keine | nein | hoch |
| 1d743e19 | 2026-10-15 | refactor(engine): decouple parking modules and cleanup background workers (W1-14) | 14 | +2/-1946 prod, +0/-0 test, +0/-0 doku | SKIP-SUPERSEDED | Entkopplung von `parking`-Modulen und Löschung von `src/decay_controller.rs` u.a.; nach maintainer-entscheid verworfen, in `main` bleibt Re-export. | Keine | nein | hoch |
| 51e1e765 | 2026-10-15 | feat(ports): redefine Erasure ports per §5.4 and §5.5 (J-07) | 2 | +2/-2 prod, +0/-0 test, +0/-0 doku | DEL-R4 | Redefinition der Erasure ports per §5.4 und §5.5 für Löschnachweis. | R4 | nein | niedrig |
| e932829e | 2026-10-15 | feat(store,engine): complete ErasureBackend implementation & fail-closed tenant resolution | 2 | +24/-22 prod, +0/-0 test, +0/-0 doku | DEL-R4 | ErasureBackend-Implementierung und Fail-closed Tenant-Resolution. | R4 | nein | niedrig |
| 13645f79 | 2026-10-15 | hotfix(engine): remove erasure witness fabrication (F-01 / REG-14) | 2 | +30/-100 prod, +86/-0 test, +0/-0 doku | DEL-R4 | Entfernung von Erasure-Witness-Fabrication (Hotfix F-01 / REG-14). | R4 | nein | hoch |
| d5a672c4 | 2026-10-14 | SP-17: Contextra Composition-Root Builder Wiring & Feature Matrix Alignment | 3 | +12/-12 prod, +0/-0 test, +0/-0 doku | PORT-WIRING | Composition-Root Builder Wiring & Feature Matrix Alignment in `src/contextra_impl/wiring.rs` und `Cargo.toml`. | contextra-core | nein | hoch |
| b295df6e | 2026-10-14 | feat(engine): search pipeline ReadSnapshot consistency & reranker integration (SP-22) | 7 | +26/-12 prod, +184/-7 test, +0/-0 doku | PORT-FEAT | Search-Pipeline ReadSnapshot-Konsistenz & Reranker-Integration. | contextra-ports | nein | hoch |
| acc363b1 | 2026-10-14 | feat(engine): add EngineLifecycle, ErasureSupervisor and bitemporal filter | 2 | +286/-0 prod, +0/-0 test, +0/-0 doku | PORT-FEAT | EngineLifecycle, ErasureSupervisor und bitemporaler Filter in `src/lifecycle.rs`. | contextra-ports | nein | hoch |
| 426dfead | 2026-10-14 | Shell-Commit | 1 | +0/-0 prod, +0/-0 test, +13/-8 doku | SKIP-DOC | Shell-Commit mit Doku-Aktualisierungen. | Keine | nein | niedrig |
| 482d0c94 | 2026-10-14 | Shell-Commit | 1 | +3/-16 prod, +0/-0 test, +0/-0 doku | SKIP-SUPERSEDED | Shell-Commit mit temporären Anpassungen an background_workers; durch neuere Commits abgedeckt. | Keine | nein | hoch |
| 83b2cc86 | 2026-10-14 | Shell-Commit | 3 | +28/-10 prod, +0/-0 test, +0/-0 doku | DEL-R4 | Shell-Commit für Purge-Driver / DpiRestoreFilter Verdrahtung. | R4 | nein | hoch |
| 92315a20 | 2026-10-14 | feat(engine): bind DpiRestoreFilter to RevocationLog and ErasureLedger (Task B10) | 2 | +86/-6 prod, +153/-0 test, +0/-0 doku | DEL-R4 | Bindung von DpiRestoreFilter an RevocationLog und ErasureLedger. | R4 | nein | niedrig |
| e001456c | 2026-10-14 | feat(engine): wire data and search paths in contextra-engine | 5 | +24/-2 prod, +103/-0 test, +0/-0 doku | PORT-WIRING | Verdrahtung der Data- und Search-Pfade in `contextra-engine`. | PORT-FEAT | nein | hoch |
| 8d738993 | 2026-10-14 | refactor(mvcc): consolidate SnapshotFloor trait and eliminate u64::MAX snapshot usage | 2 | +5/-5 prod, +0/-0 test, +0/-0 doku | PORT-REQUIRED | SnapshotFloor Trait-Konsolidierung und Elimination von u64::MAX Snapshot Usage. | contextra-mvcc | nein | niedrig |
| 687bfce8 | 2026-10-14 | feat(engine): Stage 2 Purge-Driver real hashes and tenant verification (B09) | 2 | +130/-48 prod, +356/-0 test, +0/-0 doku | DEL-R4 | Purge-Driver Stage 2 Reale Hashes und Tenant-Verifikation (Task B09). | R4 | nein | niedrig |
| a778c647 | 2026-10-14 | feat(testkit): harden deletion oracle raw_sweep and add per-layer canaries | 1 | +0/-0 prod, +159/-0 test, +0/-0 doku | DEL-R4 | Deletion Oracle raw_sweep Härtung und per-layer Canaries. | R4 | nein | niedrig |
| 47f608aa | 2026-10-14 | feat(sys,engine): implement runtime hardening and observed env status (B16) | 1 | +6/-6 prod, +0/-0 test, +0/-0 doku | PORT-FEAT | Runtime Hardening und Abfrage des beobachteten Environment Status (B16). | contextra-sys | nein | niedrig |
| cf09b17a | 2026-10-14 | feat(checkpoint): Kopien als Geltungsbereich: CopyRegistry, Hardlinks, Observer (B11) | 1 | +25/-2 prod, +0/-0 test, +0/-0 doku | PORT-FEAT | CopyRegistry Integration in `src/deletion/copy_observer.rs` (B11). | contextra-checkpoint | nein | niedrig |
| 2a37aa40 | 2026-10-14 | [B08] Implement real scans for relations and text deletion witnesses | 3 | +92/-3 prod, +177/-0 test, +0/-0 doku | DEL-R4 | Reale Scans für Relations- und Text-Deletion-Witnesses (Task B08). | R4 | nein | niedrig |
| a51c4b97 | 2026-10-14 | feat(crypto,engine,kvcache): wire deletion, key shredding, and proof verification | 3 | +53/-8 prod, +67/-0 test, +0/-0 doku | DEL-R4 | Verdrahtung von Deletion, Key-Shredding und Proof-Verifikation. | R4 | nein | hoch |
| 7b48449d | 2026-10-14 | feat(engine): wire lifecycle open path, controllers and operators | 9 | +116/-8 prod, +225/-8 test, +0/-0 doku | PORT-WIRING | Verdrahtung des Lifecycle Open-Pfads, Controller und Operatoren in `src/lifecycle.rs`. | PORT-FEAT | nein | hoch |
| 38e4277f | 2026-10-13 | feat(engine): separate proof signing keys from public db_instance_id (F1) | 2 | +90/-19 prod, +176/-0 test, +0/-0 doku | DEL-R4 | Trennung von Proof Signing Keys und öffentlicher db_instance_id. | R4 | nein | niedrig |
| ad061315 | 2026-10-13 | feat(engine): evaluate DocEraseOptions and enforce ErasureLedger step in erase_document | 2 | +64/-5 prod, +91/-0 test, +0/-0 doku | DEL-R4 | Auswertung von DocEraseOptions und Erzielung des ErasureLedger-Schritts in erase_document. | R4 | nein | hoch |
| a4517868 | 2026-10-13 | feat(engine): drop_collection purge_keys wiring and physical layer claims | 2 | +147/-68 prod, +122/-0 test, +0/-0 doku | DEL-R4 | drop_collection purge_keys Verdrahtung und physikalische Layer Claims. | R4 | nein | hoch |
| ce5dc41c | 2026-10-13 | feat(engine): wire Wasm merge operator pooling and KV cache crypto-shredding | 3 | +366/-96 prod, +0/-0 test, +0/-0 doku | PORT-FEAT | Wasm Merge Operator Pooling und KV-Cache Crypto-Shredding in `src/lib.rs`. | contextra-ports | nein | niedrig |
| 390166dc | 2026-10-13 | Fix bitemporal filtering and wire GDPR deletion paths in contextra-engine | 16 | +737/-187 prod, +150/-0 test, +0/-0 doku | DEL-R4 | Bitemporale Filterung und GDPR-Löschpfade in `contextra-engine`. | R4 | nein | hoch |
| 619c370c | 2026-10-13 | feat(engine): wire SSI gate, tenant context, and idempotent compensation | 7 | +227/-6 prod, +0/-0 test, +0/-0 doku | PORT-FEAT | SSI Gate, Tenant Context und idempotente Kompensation in Transaction Engine. | contextra-mvcc | nein | hoch |
| 9408b9c7 | 2026-10-13 | feat(engine): wire engine lifecycle, license gates, and background workers | 8 | +137/-25 prod, +0/-0 test, +0/-0 doku | PORT-WIRING | Engine Lifecycle, License Gates und Background Workers Verdrahtung. | contextra-ports | nein | hoch |
| b1bf1061 | 2026-10-13 | fix(engine): enforce snapshot isolation in collection write paths | 6 | +13/-13 prod, +0/-0 test, +0/-0 doku | PORT-FIX | Erzwingung von Snapshot Isolation in Collection Write Paths. | PORT-FEAT | nein | hoch |
| 35c6bee9 | 2026-10-13 | feat(types): implement TxId::try_new, TenantId::system, and new_unchecked | 24 | +81/-54 prod, +186/-51 test, +0/-0 doku | DEL-R0 | Verwendung von TxId::try_new, TenantId::system und new_unchecked (Typen in R0 wiederhergestellt). | R0 | ja | hoch |
| 9ad0c80a | 2026-10-13 | Shell-Commit | 1 | +0/-0 prod, +1/-0 test, +0/-0 doku | SKIP-SUPERSEDED | Shell-Commit mit minimaler Test-Zeile; von neueren Commits überschrieben. | Keine | nein | niedrig |
| bd576424 | 2026-10-13 | feat(mcp): resolve tenant hardcoding, cfg-gate mock generator, and add audit report | 1 | +15/-130 prod, +0/-0 test, +0/-0 doku | PORT-WIRING | MCP Tenant Hardcoding Behebung & Audit Report Anpassung. | contextra-mcp | nein | hoch |
| 3cbd7ec4 | 2026-10-13 | feat(contextra-engine): implement document erasure with footprint, ledger append, and layer witnesses (INV-D1/D2/D3) | 18 | +851/-133 prod, +579/-0 test, +0/-0 doku | DEL-R4 | Dokumentenlösung mit Footprint, Ledger Append und Layer Witnesses (INV-D1/D2/D3). | R4 | nein | hoch |
| 4b7709e6 | 2026-10-13 | Shell-Commit | 2 | +68/-69 prod, +0/-0 test, +0/-0 doku | SKIP-SUPERSEDED | Shell-Commit für Refactoring von read.rs; in späteren Commits angepasst. | Keine | nein | hoch |
| 2ae9a452 | 2026-10-13 | feat(engine): complete document erasure with footprint, ledger, and witnesses | 1 | +0/-1 prod, +0/-0 test, +0/-0 doku | SKIP-SUPERSEDED | Shell-Commit mit 1-Zeilen Entfernung; in 3cbd7ec4 integriert. | Keine | nein | hoch |
| 28df2ae4 | 2026-10-13 | feat(engine): implement complete document erasure with footprint, fence, and witnesses | 10 | +398/-29 prod, +267/-0 test, +0/-0 doku | DEL-R4 | Dokumenten-Erasure mit Footprint, Fence und Witnesses. | R4 | nein | hoch |
| 8797638d | 2026-10-13 | feat(engine): implement proof runtime and purge driver | 10 | +640/-14 prod, +548/-0 test, +0/-0 doku | DEL-R4 | Proof Runtime und Purge Driver Implementierung. | R4 | nein | mittel |
| ee5a102e | 2026-10-13 | fix(engine): normalize tx staging order and reset RAM state on drop | 9 | +340/-105 prod, +312/-0 test, +0/-0 doku | PORT-FIX | Normalisierung der Tx Staging Order und Reset des RAM-States bei Drop. | PORT-FEAT | nein | hoch |
| 561dcec9 | 2026-10-13 | Shell-Commit | 4 | +106/-184 prod, +0/-0 test, +0/-0 doku | SKIP-SUPERSEDED | Shell-Commit für Redundanzbereinigung in read.rs / collections.rs; durch spätere Commits ersetzt. | Keine | nein | hoch |
| d26ac482 | 2026-10-13 | feat(engine): implement ForgetOptions public types and environment attestation TLV | 4 | +319/-6 prod, +252/-0 test, +0/-0 doku | DEL-R4 | ForgetOptions öffentliche Typen und Environment Attestation TLV. | R4 | nein | niedrig |
| 755624dc | 2026-10-13 | Shell-Commit | 2 | +23/-0 prod, +147/-0 test, +0/-0 doku | SKIP-SUPERSEDED | Shell-Commit für Internal Key Scans; in 2a37aa40 integriert. | Keine | nein | niedrig |
| 2f68a144 | 2026-10-13 | fix(engine,mcp): resolve drop_collection keyspace mismatch and fail-closed proofs | 4 | +161/-106 prod, +49/-0 test, +0/-0 doku | DEL-R4 | Korrektur von drop_collection Keyspace Mismatch und Fail-closed Proofs. | R4 | nein | hoch |
| 9bbb9ebc | 2026-10-12 | feat(engine): implement deletion footprint, registry and reverse-sweep (INV-D1) | 14 | +482/-16 prod, +230/-34 test, +0/-0 doku | DEL-R4 | Deletion Footprint, Registry und Reverse-Sweep Basis (INV-D1). | R4 | nein | mittel |
| 54e2b018 | 2026-10-12 | refactor(contextra-engine): remove deletion proof stub | 6 | +16/-349 prod, +94/-164 test, +0/-0 doku | DEL-R4 | Entfernung alter Deletion Proof Stubs. | R4 | nein | hoch |
| 73715757 | 2026-10-12 | P30 / INV-CORE-ISOLATION-1: Core isolation, witness mint fix & facade composition root wiring | 5 | +428/-34 prod, +0/-0 test, +0/-0 doku | PORT-WIRING | Core Isolation, Witness Mint Fix & Facade Composition Root Wiring. | contextra-core | nein | hoch |
| 103c2d3e | 2026-10-11 | fix: restrict LayerCleanupProof::new_after_verified_empty to pub(crate) | 1 | +30/-0 prod, +0/-0 test, +0/-0 doku | DEL-R4 | Einschränkung von LayerCleanupProof::new_after_verified_empty auf pub(crate). | R4 | nein | hoch |
| cd3edcce | 2026-10-11 | feat(deletion): add KeyFamilyRegistry foundation and KEYFAMILIES.toml | 4 | +342/-43 prod, +161/-0 test, +0/-0 doku | DEL-R4 | KeyFamilyRegistry Basis und KEYFAMILIES.toml Integration. | R4 | nein | niedrig |
| 6ba2898e | 2026-10-11 | Shell-Commit | 5 | +246/-7 prod, +3/-3 test, +0/-0 doku | SKIP-SUPERSEDED | Shell-Commit für Deletion Registry; durch cd3edcce / 9bbb9ebc abgedeckt. | Keine | nein | hoch |
| 8cedf9fb | 2026-10-11 | Shell-Commit | 2 | +0/-0 prod, +4/-4 test, +0/-0 doku | SKIP-SUPERSEDED | Shell-Commit für Test-Fixes; durch neuere Commits ersetzt. | Keine | nein | niedrig |
| 525ecd80 | 2026-10-11 | contextra-db: add plan TODOs for index-store transaction sync | 2 | +0/-0 prod, +4/-4 test, +0/-0 doku | SKIP-DOC | contextra-db TODO Plan-Kommentare in Tests. | Keine | nein | niedrig |
| a2b063dc | 2026-10-11 | Shell-Commit | 2 | +0/-0 prod, +4/-4 test, +0/-0 doku | SKIP-SUPERSEDED | Shell-Commit mit identischer Zeilenänderung zu 8cedf9fb. | Keine | nein | niedrig |
| 2a1af3f7 | 2026-10-11 | fix: repair workspace compilation after crypto changes | 4 | +9/-10 prod, +13/-8 test, +0/-0 doku | PORT-FIX | Reparatur der Workspace-Kompilierung nach Crypto-Änderungen. | contextra-crypto | nein | hoch |
| 240f4f7c | 2026-10-11 | Shell-Commit | 6 | +90/-17 prod, +128/-0 test, +0/-0 doku | SKIP-SUPERSEDED | Shell-Commit für Embedding Backend / drop_collection; durch neuere Commits abgedeckt. | Keine | nein | hoch |
| 7e1d65ba | 2026-10-10 | fix(mcp): gate mock provider branch behind test-utils feature (#4494) | 6 | +11/-75 prod, +0/-127 test, +0/-0 doku | PORT-FIX | Gating des Mock Providers hinter test-utils Feature. | contextra-mcp | nein | hoch |
| 30fe0ba4 | 2026-10-10 | Refactor EmbeddingBackend and MCP provider creation (#4493) | 1 | +71/-6 prod, +0/-0 test, +0/-0 doku | PORT-WIRING | Refactoring von EmbeddingBackend und MCP Provider Creation. | contextra-mcp | nein | hoch |
| cea972df | 2026-10-09 | fix(vector): clear stale inbound edges on HNSW slot reuse (W4-06) (#4476) | 4 | +2/-2 prod, +0/-285 test, +0/-0 doku | PORT-FIX | Bereinigung veralteter Kanten bei HNSW Slot Reuse in tests. | contextra-vector | nein | hoch |
| 1b00f712 | 2026-10-09 | refactor(engine): derive CollectionId fail-closed before drop_collection deletions (#4470) | 2 | +40/-7 prod, +89/-0 test, +0/-0 doku | DEL-R4 | Derivation von CollectionId fail-closed vor drop_collection Deletions. | R4 | nein | hoch |
| 9a84d0ba | 2026-10-09 | test(engine): implement drop_collection tests T1-T3 from Audit A-03 (#4468) | 3 | +0/-0 prod, +285/-0 test, +0/-0 doku | DEL-R4 | Implementierung von drop_collection Tests T1-T3. | R4 | nein | niedrig |
| ef39f9f0 | 2026-10-08 | supply-chain(tokio): replace full feature with per-crate minimal features (#4464) | 1 | +2/-2 prod, +0/-0 test, +0/-0 doku | PORT-FIX | Supply-Chain Optimierung von tokio Features in Cargo.toml. | Keine | ja | hoch |
| 4c1afff3 | 2026-10-08 | Fix KvKeyLocks type documentation drift and lock hierarchy (#4445) | 1 | +11/-0 prod, +0/-0 test, +0/-0 doku | PORT-FIX | Dokumentations- und Hierarchie-Korrektur für KvKeyLocks. | Keine | ja | niedrig |
| f797faf7 | 2026-10-08 | Engine: EmbeddingBackend::Candle und Stub-Beweis Härtung (#4444) | 3 | +19/-4 prod, +38/-0 test, +0/-0 doku | PORT-FEAT | EmbeddingBackend::Candle und Stub-Beweis Härtung in engine. | contextra-infer-candle | nein | hoch |
| 8fb5fef6 | 2026-10-06 | fix(engine): document drop_collection WAL durability and add test (#4393) | 2 | +5/-7 prod, +95/-0 test, +0/-0 doku | DEL-R4 | drop_collection WAL Durability Dokumentation und Test. | R4 | nein | hoch |
| 52e6ea7e | 2026-10-06 | Shell-Commit | 1 | +8/-0 prod, +0/-0 test, +0/-0 doku | SKIP-SUPERSEDED | Shell-Commit für collections.rs; durch spätere Commits ersetzt. | Keine | nein | hoch |
| 56e5769a | 2026-10-06 | docs: standardize and audit all AGENTS.md files across multi-crate workspace | 1 | +0/-0 prod, +0/-0 test, +25/-11 doku | SKIP-DOC | Standardisierung und Audit aller AGENTS.md Dateien. | Keine | ja | hoch |
| ce650578 | 2026-10-06 | feat(xtask): implement orphan-symbols scanner (AUFTRAG X01) (#4374) | 5 | +15/-7 prod, +4/-7 test, +0/-0 doku | SKIP-INFRA | Orphan-Symbols Scanner Anpassung (X01). | xtask | nein | hoch |
| 8f6938fc | 2026-10-06 | Make no_crypto_stubs DeletionProof implementation in contextra-engine fail-closed (#4373) | 2 | +106/-17 prod, +159/-0 test, +0/-0 doku | DEL-R4 | no_crypto_stubs DeletionProof Implementierung fail-closed. | R4 | nein | hoch |
| 8c6da4ee | 2026-10-06 | fix(engine): restrict drop_collection DeletionProof to LsmMemtable layer (#4372) | 1 | +18/-12 prod, +0/-0 test, +0/-0 doku | DEL-R4 | Einschränkung von drop_collection DeletionProof auf LsmMemtable Layer. | R4 | nein | hoch |
| 9032658f | 2026-10-06 | feat(engine): add tracing error logging to no-crypto stubs and update docs (#4365) | 1 | +39/-4 prod, +0/-0 test, +0/-0 doku | DEL-R4 | Tracing Error Logging in no-crypto stubs. | R4 | nein | hoch |
| 28ba05f6 | 2026-10-06 | fix(engine): J04-CLOSURE wire engine symbols and cleanup dead stubs (#4343) | 1 | +0/-0 prod, +4/-4 test, +0/-0 doku | SKIP-SUPERSEDED | J04-CLOSURE Symbol-Verdrahtung in Tests; durch neuere Commits überschrieben. | Keine | nein | niedrig |
| f2cb5aa8 | 2026-10-05 | wire remaining 16 contextra-store public symbols (#4327) | 2 | +73/-1 prod, +0/-0 test, +0/-0 doku | PORT-WIRING | Verdrahtung von 16 contextra-store öffentlichen Symbolen in engine. | contextra-store | ja | hoch |
| 04804666 | 2026-10-05 | fix(engine): wire open symbols and add closure integration tests (#4289) (#4324) | 5 | +33/-6 prod, +146/-0 test, +0/-0 doku | PORT-WIRING | Verdrahtung offener Symbole und Closure Integration Tests. | PORT-WIRING | ja | hoch |
| d57e1661 | 2026-10-05 | fix(contextra-engine): wire and clean up collection/maintenance/kv symbols (#4288) | 6 | +14/-80 prod, +144/-0 test, +0/-0 doku | PORT-WIRING | Verdrahtung und Bereinigung von Collection/Maintenance/KV Symbolen. | PORT-WIRING | ja | mittel |
| 13e4349e | 2026-10-05 | refactor(engine): change Collection::kv_store field type to KvLifecycleHooks port trait object (#4277) | 1 | +9/-7 prod, +0/-0 test, +0/-0 doku | PORT-WIRING | Umstellung von Collection::kv_store Feldtyp auf KvLifecycleHooks Trait Object. | contextra-ports | ja | hoch |

## Anhang: Datei-Status (Dateifälle A bis H)

| Pfad | Fall A bis H | Konflikt in merge-tree | beteiligte Commits |
| :--- | :--- | :--- | :--- |
| `crates/contextra-engine/AGENTS.md` | Fall D | ja | 426dfead, 56e5769a |
| `crates/contextra-engine/Cargo.toml` | Fall D | ja | 0003eece, d5a672c4, 9408b9c7, 54e2b018, 73715757, 6ba2898e, cea972df, ef39f9f0 |
| `crates/contextra-engine/src/background_workers/expiry_workers.rs` | Fall B | nein | 1d743e19 |
| `crates/contextra-engine/src/background_workers/hyperedge_worker.rs` | Fall B | nein | 1d743e19 |
| `crates/contextra-engine/src/background_workers/mod.rs` | Fall B | nein | 1d743e19, 9408b9c7 |
| `crates/contextra-engine/src/background_workers/orphan_workers.rs` | Fall B | nein | 9408b9c7 |
| `crates/contextra-engine/src/background_workers/tests.rs` | Fall B | nein | 1d743e19, 9408b9c7 |
| `crates/contextra-engine/src/chunker.rs` | Fall B | nein | 0003eece |
| `crates/contextra-engine/src/collection/crud/auto_extraction.rs` | Fall B | nein | 62bf203a, 04804666, d57e1661 |
| `crates/contextra-engine/src/collection/crud/delete.rs` | Fall D | ja | 13645f79, 83b2cc86, ad061315, b1bf1061, 35c6bee9, bd576424, 3cbd7ec4 |
| `crates/contextra-engine/src/collection/crud/insert.rs` | Fall B | nein | b1bf1061 |
| `crates/contextra-engine/src/collection/crud/links.rs` | Fall B | nein | 62bf203a, b295df6e, e001456c, b1bf1061 |
| `crates/contextra-engine/src/collection/crud/mod.rs` | Fall B | nein | 62bf203a, 28df2ae4, d57e1661 |
| `crates/contextra-engine/src/collection/crud/read.rs` | Fall B | nein | e001456c, 561dcec9, 755624dc |
| `crates/contextra-engine/src/collection/crud/update.rs` | Fall B | nein | b1bf1061 |
| `crates/contextra-engine/src/collection/kv_lock.rs` | Fall B | nein | 240f4f7c, 7e1d65ba, 4c1afff3 |
| `crates/contextra-engine/src/collection/maintenance.rs` | Fall B | nein | 62bf203a, 35c6bee9, ee5a102e, d57e1661 |
| `crates/contextra-engine/src/collection/mod.rs` | Fall D | ja | 62bf203a, e932829e, ce650578, f2cb5aa8, 13e4349e |
| `crates/contextra-engine/src/collection/query_builder/builder.rs` | Fall B | nein | 62bf203a |
| `crates/contextra-engine/src/collection/query_builder/builder_exec.rs` | Fall B | nein | 62bf203a, 51e1e765, b295df6e, b1bf1061 |
| `crates/contextra-engine/src/collection/query_builder/scope.rs` | Fall B | nein | 04804666 |
| `crates/contextra-engine/src/collection/query_builder/tests.rs` | Fall B | nein | 62bf203a |
| `crates/contextra-engine/src/collection/relate.rs` | Fall B | nein | 62bf203a, b1bf1061 |
| `crates/contextra-engine/src/collection/search/hybrid/mod.rs` | Fall B | nein | 62bf203a, 390166dc |
| `crates/contextra-engine/src/collection/search/hybrid/query.rs` | Fall D | ja | 62bf203a, 51e1e765, b295df6e, e001456c, 390166dc |
| `crates/contextra-engine/src/collection/search/mod.rs` | Fall B | nein | 390166dc |
| `crates/contextra-engine/src/collection/tests/concurrency_tests.rs` | Fall B | nein | 830c0399 |
| `crates/contextra-engine/src/collection/tests/hybrid_search_tests.rs` | Fall B | nein | 830c0399 |
| `crates/contextra-engine/src/collection/tests/insert_tests.rs` | Fall B | nein | 830c0399 |
| `crates/contextra-engine/src/collection/tests/link_tests.rs` | Fall B | nein | 830c0399 |
| `crates/contextra-engine/src/collection/tests/misc_tests.rs` | Fall B | nein | 830c0399 |
| `crates/contextra-engine/src/collection/tests/mod.rs` | Fall B | nein | 830c0399 |
| `crates/contextra-engine/src/collection/tests/read_tests.rs` | Fall B | nein | 830c0399 |
| `crates/contextra-engine/src/collection/tests/search_tests.rs` | Fall B | nein | 830c0399, b295df6e |
| `crates/contextra-engine/src/collection/tests/update_delete_tests.rs` | Fall B | nein | 830c0399 |
| `crates/contextra-engine/src/contextra_impl/collections.rs` | Fall D | ja | 0003eece, 482d0c94, 83b2cc86, a51c4b97, a4517868, 390166dc, 4b7709e6, 561dcec9, 2f68a144, 54e2b018, 73715757, 2a1af3f7, 240f4f7c, 7e1d65ba, 1b00f712, 8fb5fef6, 52e6ea7e, ce650578, 8c6da4ee, 04804666 |
| `crates/contextra-engine/src/contextra_impl/diagnostics.rs` | Fall B | nein | 0003eece |
| `crates/contextra-engine/src/contextra_impl/lifecycle.rs` | Fall D | ja | 0003eece, d5a672c4, 7b48449d, 9408b9c7, 2a1af3f7, 240f4f7c, 7e1d65ba, f797faf7, f2cb5aa8 |
| `crates/contextra-engine/src/contextra_impl/mod.rs` | Fall B | nein | 9408b9c7, 561dcec9, 2f68a144 |
| `crates/contextra-engine/src/contextra_impl/wiring.rs` | Fall B | nein | 0003eece, 9408b9c7 |
| `crates/contextra-engine/src/decay_controller.rs` | Fall D | ja | 1d743e19 |
| `crates/contextra-engine/src/deletion/copy_observer.rs` | Fall C | nein | cf09b17a, 8797638d, 9bbb9ebc |
| `crates/contextra-engine/src/deletion/environment.rs` | Fall C | nein | 47f608aa, d26ac482, 9bbb9ebc |
| `crates/contextra-engine/src/deletion/footprint.rs` | Fall C | nein | 9bbb9ebc, cd3edcce |
| `crates/contextra-engine/src/deletion/footprint_handle.rs` | Fall C | nein | 0003eece, 8d738993, 390166dc, 35c6bee9, 28df2ae4, 9bbb9ebc |
| `crates/contextra-engine/src/deletion/forget_options.rs` | Fall C | nein | 8797638d, d26ac482, 9bbb9ebc |
| `crates/contextra-engine/src/deletion/mod.rs` | Fall C | nein | 390166dc, 35c6bee9, 8797638d, 9bbb9ebc, cd3edcce, 6ba2898e |
| `crates/contextra-engine/src/deletion/proof_issue.rs` | Fall C | nein | 0003eece, 38e4277f, 35c6bee9, 8797638d, 9bbb9ebc |
| `crates/contextra-engine/src/deletion/purge_driver.rs` | Fall C | nein | 0003eece, 83b2cc86, 687bfce8, 390166dc, 35c6bee9, 8797638d |
| `crates/contextra-engine/src/deletion/registry.rs` | Fall C | nein | 0003eece, 9bbb9ebc, cd3edcce, 6ba2898e |
| `crates/contextra-engine/src/deletion/relations_witness.rs` | Fall C | nein | 2a37aa40, 390166dc, 35c6bee9, 28df2ae4, 9bbb9ebc |
| `crates/contextra-engine/src/deletion/restore_filter.rs` | Fall C | nein | 92315a20, 8797638d, 9bbb9ebc |
| `crates/contextra-engine/src/deletion/reverse_sweep.rs` | Fall C | nein | 390166dc, 4b7709e6, 9bbb9ebc |
| `crates/contextra-engine/src/deletion/text_witness.rs` | Fall C | nein | 2a37aa40, 390166dc, 35c6bee9, 28df2ae4, 9bbb9ebc |
| `crates/contextra-engine/src/deletion/wiring.rs` | Fall C | nein | 0003eece, a51c4b97, 390166dc |
| `crates/contextra-engine/src/extraction/mod.rs` | Fall B | nein | 1d743e19 |
| `crates/contextra-engine/src/extraction/open_ie.rs` | Fall B | nein | 1d743e19 |
| `crates/contextra-engine/src/extraction/rule_based.rs` | Fall B | nein | 1d743e19 |
| `crates/contextra-engine/src/extraction/tests.rs` | Fall B | nein | 1d743e19 |
| `crates/contextra-engine/src/extraction/types.rs` | Fall B | nein | 1d743e19 |
| `crates/contextra-engine/src/kv_cache_integration/mod.rs` | Fall B | nein | 1d743e19, ce5dc41c, 04804666, d57e1661 |
| `crates/contextra-engine/src/lib.rs` | Fall D | ja | 0003eece, d5a672c4, b295df6e, acc363b1, 7b48449d, 390166dc, 9408b9c7, 561dcec9, 2f68a144, 54e2b018, 73715757, 103c2d3e, 6ba2898e, 240f4f7c, 7e1d65ba, 30fe0ba4, f797faf7, 8f6938fc, 9032658f |
| `crates/contextra-engine/src/lifecycle.rs` | Fall H | nein | 0003eece, acc363b1 |
| `crates/contextra-engine/src/limits.rs` | Fall H | nein | 73715757 |
| `crates/contextra-engine/src/merge_operator_adapter.rs` | Fall B | nein | 1d743e19, ce5dc41c |
| `crates/contextra-engine/src/temporal_filter.rs` | Fall B | nein | 0003eece, 390166dc |
| `crates/contextra-engine/src/transaction/cleanup.rs` | Fall B | nein | d57e1661 |
| `crates/contextra-engine/src/transaction/compensating_actions.rs` | Fall D | nein | 619c370c, ee5a102e |
| `crates/contextra-engine/src/transaction/db_transaction.rs` | Fall B | nein | 0003eece, 619c370c, ee5a102e |
| `crates/contextra-engine/src/transaction/db_transaction_commit.rs` | Fall B | nein | 0003eece, e932829e, 619c370c, 35c6bee9, ee5a102e |
| `crates/contextra-engine/src/transaction/db_transaction_rollback.rs` | Fall D | ja | 619c370c, ee5a102e |
| `crates/contextra-engine/src/transaction/mod.rs` | Fall B | nein | 0003eece, 619c370c |
| `crates/contextra-engine/src/transaction/tests.rs` | Fall B | nein | 0003eece, 619c370c |
| `crates/contextra-engine/src/transaction/wiring.rs` | Fall H | nein | 619c370c |
| `crates/contextra-engine/tests/auto_extraction_default_enabled.rs` | Fall B | nein | 830c0399 |
| `crates/contextra-engine/tests/auto_extraction_runtime_mode.rs` | Fall B | nein | 830c0399 |
| `crates/contextra-engine/tests/auto_extraction_wiring.rs` | Fall B | nein | 830c0399 |
| `crates/contextra-engine/tests/auto_triple_extraction.rs` | Fall B | nein | 830c0399 |
| `crates/contextra-engine/tests/collection_drop_name_reuse.rs` | Fall H | nein | 35c6bee9, ee5a102e |
| `crates/contextra-engine/tests/collection_drop_ram_reset.rs` | Fall H | nein | 35c6bee9, ee5a102e |
| `crates/contextra-engine/tests/commit_crash_consistency.rs` | Fall B | nein | 830c0399 |
| `crates/contextra-engine/tests/deletion_e2e_canary_per_layer.rs` | Fall H | nein | a778c647 |
| `crates/contextra-engine/tests/deletion_e2e_public_path.rs` | Fall H | nein | a51c4b97 |
| `crates/contextra-engine/tests/drop_collection_concurrent_writer.rs` | Fall H | nein | 390166dc |
| `crates/contextra-engine/tests/drop_collection_id_derivation.rs` | Fall H | nein | 2a1af3f7, 240f4f7c, 7e1d65ba, 1b00f712 |
| `crates/contextra-engine/tests/drop_collection_layer_claims.rs` | Fall H | nein | a4517868 |
| `crates/contextra-engine/tests/drop_collection_scope.rs` | Fall H | nein | 35c6bee9, 2f68a144 |
| `crates/contextra-engine/tests/engine_commit_crash_points.rs` | Fall B | nein | 830c0399 |
| `crates/contextra-engine/tests/engine_recovery_crash_windows.rs` | Fall B | nein | 830c0399 |
| `crates/contextra-engine/tests/engine_restore_filter_revocation.rs` | Fall H | nein | 92315a20 |
| `crates/contextra-engine/tests/environment_attest_fixtures.rs` | Fall H | nein | 35c6bee9, d26ac482 |
| `crates/contextra-engine/tests/erase_document_ledger_and_witnesses.rs` | Fall H | nein | 830c0399, ad061315 |
| `crates/contextra-engine/tests/erasure_hotfix_f01.rs` | Fall H | nein | 13645f79 |
| `crates/contextra-engine/tests/erasure_purge_sweep_proof_test.rs` | Fall H | nein | 390166dc |
| `crates/contextra-engine/tests/forget_document_footprint.rs` | Fall H | nein | 35c6bee9, 28df2ae4 |
| `crates/contextra-engine/tests/forget_document_relations.rs` | Fall H | nein | 35c6bee9, 28df2ae4 |
| `crates/contextra-engine/tests/forget_document_resurrection.rs` | Fall H | nein | 35c6bee9, 28df2ae4 |
| `crates/contextra-engine/tests/forget_document_witnesses.rs` | Fall H | nein | 35c6bee9, 28df2ae4 |
| `crates/contextra-engine/tests/forget_options_defaults.rs` | Fall H | nein | 35c6bee9, 9ad0c80a, d26ac482 |
| `crates/contextra-engine/tests/hybrid_adaptive_overfetch.rs` | Fall B | nein | 830c0399 |
| `crates/contextra-engine/tests/hybrid_pathrag_snapshot.rs` | Fall B | nein | 830c0399 |
| `crates/contextra-engine/tests/hybrid_signal_failure_semantics.rs` | Fall B | nein | 830c0399 |
| `crates/contextra-engine/tests/invariant_tenant_isolation.rs` | Fall B | nein | 54e2b018 |
| `crates/contextra-engine/tests/j03_contextra_engine_collection_crud_mainten_wiring_tests.rs` | Fall H | nein | 830c0399, ce650578, d57e1661 |
| `crates/contextra-engine/tests/j03closure-contextra-engine_wiring_test.rs` | Fall H | nein | 830c0399, ce650578, 04804666 |
| `crates/contextra-engine/tests/key_family_registry.rs` | Fall H | nein | 9bbb9ebc, cd3edcce |
| `crates/contextra-engine/tests/kv_cache_integration_test.rs` | Fall B | nein | 830c0399 |
| `crates/contextra-engine/tests/kv_delete_cascade_e2e.rs` | Fall C | nein | 830c0399 |
| `crates/contextra-engine/tests/kv_hooks_production_wiring.rs` | Fall C | nein | 830c0399, 7b48449d, 8cedf9fb, 525ecd80, a2b063dc, ce650578, 28ba05f6 |
| `crates/contextra-engine/tests/kv_lifecycle_hooks_rollback.rs` | Fall C | nein | 830c0399 |
| `crates/contextra-engine/tests/lifecycle_wiring.rs` | Fall H | nein | 830c0399, 7b48449d |
| `crates/contextra-engine/tests/lifecycle_worker_shutdown.rs` | Fall H | nein | 7b48449d |
| `crates/contextra-engine/tests/metrics_instrumentation.rs` | Fall B | nein | 7b48449d |
| `crates/contextra-engine/tests/no_crypto_proof_stub.rs` | Fall H | nein | 54e2b018, 8f6938fc |
| `crates/contextra-engine/tests/openie_determinism_and_provenance.rs` | Fall B | nein | 830c0399 |
| `crates/contextra-engine/tests/orphan_worker_started_on_open.rs` | Fall D | ja | 7b48449d |
| `crates/contextra-engine/tests/p02c_drop_collection_proof_durability.rs` | Fall H | nein | 54e2b018, 8fb5fef6 |
| `crates/contextra-engine/tests/p04_scan_internal_keys.rs` | Fall H | nein | 755624dc |
| `crates/contextra-engine/tests/proof_issue_two_stage.rs` | Fall H | nein | 35c6bee9, 8797638d |
| `crates/contextra-engine/tests/proof_signer_key_independence.rs` | Fall H | nein | 38e4277f |
| `crates/contextra-engine/tests/purge_driver_blocked.rs` | Fall H | nein | 35c6bee9, 8797638d |
| `crates/contextra-engine/tests/purge_driver_copies.rs` | Fall H | nein | 35c6bee9, 8797638d |
| `crates/contextra-engine/tests/purge_driver_crash.rs` | Fall H | nein | 35c6bee9, 8797638d |
| `crates/contextra-engine/tests/purge_driver_real_hashes.rs` | Fall H | nein | b295df6e, 687bfce8 |
| `crates/contextra-engine/tests/query_builder_scope_acorn.rs` | Fall B | nein | 830c0399 |
| `crates/contextra-engine/tests/relations_text_witness_real_scan.rs` | Fall H | nein | b295df6e, 2a37aa40 |
| `crates/contextra-engine/tests/reverse_sweep.rs` | Fall H | nein | 9bbb9ebc |
| `crates/contextra-engine/tests/rule_based_extraction_determinism.rs` | Fall B | nein | 830c0399 |
| `crates/contextra-engine/tests/search_reranker_test.rs` | Fall H | nein | e001456c |
| `crates/contextra-engine/tests/ssi_read_path_coverage.rs` | Fall B | nein | 830c0399 |
| `crates/contextra-engine/tests/tenant_index_persistence_reload.rs` | Fall B | nein | 6ba2898e, 8cedf9fb, 525ecd80, a2b063dc |
| `crates/contextra-engine/tests/text_domain_vocabulary_test.rs` | Fall H | nein | e001456c |
| `crates/contextra-engine/tests/tx_erase_compensation.rs` | Fall H | nein | 35c6bee9, ee5a102e |
| `crates/contextra-engine/tests/tx_erase_insert_delete_same_tx.rs` | Fall H | nein | 35c6bee9, ee5a102e |
| `crates/contextra-engine/tests/wasm_merge_operator.rs` | Fall B | nein | 830c0399 |

## Entscheidung nötig

Keine offenen `DECISION`-Klassifizierungen verblieben. Alle Commits und Dateien konnten eindeutig kategorisiert werden.

- **`src/decay_controller.rs`**: Im Spender eb10652f wurde `src/decay_controller.rs` im Zuge des Aufräumens der Parking-Logik gelöscht (`1d743e19`). In `main` ist die Datei als 1-zeiliger Re-export (`pub use contextra_adapt::decay_controller::*;`) vorhanden. Gemäß Maintainer-Entscheid ("Von main gelöschte Dateien bleiben gelöscht... Kopien/Re-exports in main behalten") verbleibt `src/decay_controller.rs` in `main` als Re-export von Ring 0 `contextra-adapt` erhalten.

## Abhängigkeiten und Reihenfolge

1. **Unterbau (R0)**: `contextra-types`, `contextra-ports`, `contextra-mvcc`, `contextra-store`, `contextra-crypto` sind bereits wiederhergestellt.
2. **PORT-REQUIRED & PORT-FIX**: `8d738993` (SnapshotFloor Trait), `b1bf1061` (Snapshot Isolation in Collection Writes) und `ee5a102e` (Tx Staging Order) bilden das Fundament.
3. **PORT-FEAT**: `619c370c` (SSI Gate), `acc363b1` (EngineLifecycle), `b295df6e` (Search Pipeline ReadSnapshot), `47f608aa` (Runtime Hardening), `cf09b17a` (CopyRegistry), `ce5dc41c` (Wasm Merge Pooling & KV Cache Crypto-Shredding) und `f797faf7` (EmbeddingBackend::Candle).
4. **PORT-WIRING**: `d56a72c4`, `e001456c`, `7b48449d`, `9408b9c7`, `73715757`, `30fe0ba4` (Composition Root & Facade Wiring).
5. **DEL-R4**: Löschnachweis-Spur in Engine (wird getrennt von R4 verarbeitet).

## Auffälligkeiten

- **Exzellente Testabdeckung**: Neue Features wie `SSI Gate`, `CopyRegistry`, `EngineLifecycle` und `ReadSnapshot` bringen umfangreiche Test-Suites mit.
- **Entkopplungs-Commits im Spender-Core-Cut**: Die Commits `0003eece`, `830c0399`, `62bf203a` und `1d743e19` versuchten `parking_lot` / `parking`-Module im Zuge des ungenehmigten 18-Crate "Core-Cut" zu entfernen. Da der Maintainer den Stand eb10652f (33-Crate + durable-fs = 34 Crates Architecture) als Referenz gewählt hat und spätere Core-Cuts verwirft, wurden diese Commits als `SKIP-SUPERSEDED` klassifiziert.

## Löschungen respektiert

Die folgenden Dateien wurden im Spender gelöscht und verbleiben gelöscht, da sie sich auf entfallene oder veraltete Hilfsklassen/Subsysteme bezogen bzw. durch den Re-export oder neuere Strukturen abgedeckt sind:
- `crates/contextra-engine/src/background_workers/hyperedge_worker.rs`
- `crates/contextra-engine/src/collection/crud/auto_extraction.rs`
- `crates/contextra-engine/src/collection/relate.rs`
- `crates/contextra-engine/src/collection/tests/link_tests.rs`
- `crates/contextra-engine/src/extraction/**` (`mod.rs`, `open_ie.rs`, `rule_based.rs`, `tests.rs`, `types.rs`)
- `crates/contextra-engine/src/kv_cache_integration/mod.rs`
- `crates/contextra-engine/src/merge_operator_adapter.rs`
- `crates/contextra-engine/tests/auto_extraction_*.rs`
- `crates/contextra-engine/tests/auto_triple_extraction.rs`
- `crates/contextra-engine/tests/hybrid_pathrag_snapshot.rs`
- `crates/contextra-engine/tests/kv_cache_integration_test.rs`
- `crates/contextra-engine/tests/openie_determinism_and_provenance.rs`
- `crates/contextra-engine/tests/rule_based_extraction_determinism.rs`
- `crates/contextra-engine/tests/wasm_merge_operator.rs`
