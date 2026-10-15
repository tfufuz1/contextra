# Auditergebnis: Hülle und Gate-Verhalten (2026-10-15)

- **Datum:** 2026-10-15
- **Commit-Hash:** `67c8cd3eec4e373d14286c6d49337258b0aa1d3c`
- **Umgebung:** `cargo metadata --locked` verified, Rust toolchain 1.89.0-nightly

---

## Summary Table / Messgrößen Übersicht

| Messgröße | Wert | Exit-Code | Befehl | Abweichung / Anmerkung |
| --- | --- | --- | --- | --- |
| Kompilierprüfung contextra-mcp | `Exit 101` | `101` | `cargo check --locked -p contextra-mcp` | Exit 101 erwartet wegen Graph-Komponenten Entkopplung |
| Kompilierprüfung Tests Workspace | `Exit 101` | `101` | `cargo test --workspace --locked --no-run` | Exit 101 erwartet wegen Kompilierfehlern in Workspace |
| Hülle contextra-mcp (Default-Features) | `29` | `0` | `cargo tree -p contextra-mcp -e normal --prefix none | grep ^contextra | sort -u | wc -l` | Abweichung von Spezifikation-Richtwert (Spezi: <= 25/18, Ist: 29) |
| Hülle contextra-mcp (no-default-features) | `29` | `0` | `cargo tree -p contextra-mcp -e normal --no-default-features --prefix none | grep ^contextra | sort -u | wc -l` | Abweichung (Ist: 29) |
| Hülle contextra-mcp (all-features) | `29` | `0` | `cargo tree -p contextra-mcp -e normal --all-features --prefix none | grep ^contextra | sort -u | wc -l` | Abweichung (Spezi-Schätzung: 28, Ist: 29) |
| Hülle contextra-engine (no-default-features) | `19` | `0` | `cargo tree -p contextra-engine -e normal --no-default-features --prefix none | grep ^contextra | sort -u | wc -l` | Abweichung (Spezi-Schätzung: 15, Ist: 19) |
| Hülle contextra (Default-Features) | `27` | `0` | `cargo tree -p contextra -e normal --prefix none | grep ^contextra | sort -u | wc -l` | Abweichung (Spezi-Schätzung: 22, Ist: 27) |
| Ring-Layering Check (--strict) | `Exit 1` | `1` | `cargo xtask check-ring-layering --strict` | Exit 1 wegen 2 neuer Ring-Verstöße & 3 veralteter Allowlist-Einträge |
| DAG Check | `Exit 0` | `0` | `cargo xtask check-dag` | Erwartung 0 erfüllt |
| Anzahl Dateien mit contextra_core Import | `2` | `0` | `grep -rl "contextra_core" crates --include=*.rs | wc -l` | Ist: 2 Dateien |
| Treffer downcast_ref (ohne Tests) | `5` | `0` | `grep -rn "downcast_ref" crates --include=*.rs | grep -v test` | Ist: 5 Treffer |
| Treffer u64::MAX (ohne Tests) | `107` | `0` | `grep -rn "u64::MAX" crates --include=*.rs | grep -v test` | Ist: 107 Treffer (Lesezeitpunkte / Konstanten / Boundary-Checks) |
| Dateiliste mit `cfg(feature)` unter `crates/` | `130 Dateien in 14 Crates` | `0` | `grep -rn "cfg(feature" crates/ --include=*.rs` | Siehe Zusatzabschnitt |

---

## Detailliertes Messprotokoll (Befehle 1 bis 12)

### Befehl 1: Kompilierprüfung contextra-mcp

- **Befehl:** `cargo check --locked -p contextra-mcp`
- **Commit-Hash:** `67c8cd3eec4e373d14286c6d49337258b0aa1d3c`
- **Exit-Code:** `101`
- **Datum:** 2026-10-15
- **Ausgabe:**
```
warning: associated function `new` is never used
  --> crates/contextra-crypto/src/deletion_witness.rs:69:19
   |
66 | impl WitnessIssuerToken {
   | ----------------------- associated function in this implementation
...
69 |     pub(crate) fn new() -> Self {
   |                   ^^^
   |
   = note: `#[warn(dead_code)]` on by default

warning: field `lifecycle_state` is never read
   --> crates/contextra-crypto/src/kv_shredding.rs:149:5
    |
144 | struct GroupEntry {
    |        ---------- field in this struct
...
149 |     lifecycle_state: KeyLifecycleState,
    |     ^^^^^^^^^^^^^^^
    |
    = note: `GroupEntry` has a derived impl for the trait `Debug`, but this is intentionally ignored during dead code analysis

warning: `contextra-crypto` (lib) generated 2 warnings
warning: use of deprecated associated function `contextra_types::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
   --> crates/contextra-store/src/lsm/recovery.rs:807:39
    |
807 |                 .rollback_to_tx(TxId::new(target_tx))
    |                                       ^^^
    |
    = note: `#[warn(deprecated)]` on by default

warning: use of deprecated associated function `contextra_types::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
   --> crates/contextra-store/src/lsm/ops/compaction.rs:160:59
    |
160 |         storage.advance_visibility(contextra_types::TxId::new(sst_max_tx));
    |                                                           ^^^

warning: use of deprecated associated function `contextra_types::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
  --> crates/contextra-store/src/lsm/ops/maintenance.rs:52:14
   |
52 |     Ok(TxId::new(storage.last_committed_tx.load(Ordering::SeqCst)))
   |              ^^^

warning: use of deprecated associated function `contextra_types::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
   --> crates/contextra-store/src/wal/encode.rs:348:35
    |
348 |                 let tx_id = TxId::new(u64::from_le_bytes(
    |                                   ^^^

warning: use of deprecated associated function `contextra_types::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
   --> crates/contextra-store/src/wal/encode.rs:408:35
    |
408 |                 let tx_id = TxId::new(u64::from_le_bytes(
    |                                   ^^^

warning: use of deprecated associated function `contextra_types::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
   --> crates/contextra-store/src/wal/encode.rs:437:35
    |
437 |                 let tx_id = TxId::new(u64::from_le_bytes(
    |                                   ^^^

warning: use of deprecated associated function `contextra_types::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
   --> crates/contextra-vector/src/hnsw/vector_index_impl.rs:644:18
    |
644 |         Ok(TxId::new(self.inner.hot.last_tx_id.load(Ordering::SeqCst)))
    |                  ^^^
    |
    = note: `#[warn(deprecated)]` on by default

warning: use of deprecated associated function `contextra_types::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
   --> crates/contextra-vector/src/hnsw/witness.rs:117:52
    |
117 |         let scanned_at_tx = contextra_types::TxId::new(
    |                                                    ^^^

warning: `contextra-store` (lib) generated 6 warnings
warning: `contextra-vector` (lib) generated 2 warnings
    Checking contextra-engine v0.1.0 (/app/crates/contextra-engine)
warning: contextra-wire@0.1.0: flatc not found, using existing generated code.
error[E0432]: unresolved import `super::auto_extraction`
 --> crates/contextra-engine/src/collection/crud/insert.rs:1:12
  |
1 | use super::auto_extraction::auto_extract_and_relate;
  |            ^^^^^^^^^^^^^^^ could not find `auto_extraction` in `super`

error[E0432]: unresolved imports `crate::collection::crud::AutoExtractionConfig`, `crate::collection::crud::AutoExtractionMode`
 --> crates/contextra-engine/src/contextra_impl/collections.rs:1:31
  |
1 | use crate::collection::crud::{AutoExtractionConfig, AutoExtractionMode};
  |                               ^^^^^^^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^^^^ no `AutoExtractionMode` in `collection::crud`
  |                               |
  |                               no `AutoExtractionConfig` in `collection::crud`
  |
  = help: consider importing one of these enums instead:
          contextra_mvcc::AutoExtractionMode
          contextra_ports::AutoExtractionMode
          contextra_types::AutoExtractionMode

error[E0432]: unresolved import `crate::decay_controller`
 --> crates/contextra-engine/src/collection/maintenance.rs:8:12
  |
8 | use crate::decay_controller::{AdaptiveDecayController, DecaySignalInputs};
  |            ^^^^^^^^^^^^^^^^ could not find `decay_controller` in the crate root

warning: unused import: `internal::validate_doc_id`
  --> crates/contextra-engine/src/collection/crud/mod.rs:19:16
   |
19 | pub(super) use internal::validate_doc_id;
   |                ^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(unused_imports)]` on by default

warning: use of deprecated function `temporal_filter::apply_temporal_validity_filter_at`: Use apply_temporal_validity_filter_at_bitemporal with explicit TxId and business time instead
  --> crates/contextra-engine/src/collection/search/mod.rs:25:37
   |
25 |     apply_temporal_validity_filter, apply_temporal_validity_filter_at, FusionResult,
   |                                     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(deprecated)]` on by default

error[E0599]: no method named `check_and_trigger_community_detection` found for reference `&collection::Collection<S, V>` in the current scope
   --> crates/contextra-engine/src/collection/crud/delete.rs:141:14
    |
141 |         self.check_and_trigger_community_detection(1);
    |              ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ method not found in `&collection::Collection<S, V>`

error[E0599]: no method named `auto_extraction_config` found for reference `&collection::Collection<S, V>` in the current scope
  --> crates/contextra-engine/src/collection/crud/insert.rs:45:50
   |
45 |             if let Some((generator, cfg)) = self.auto_extraction_config() {
   |                                                  ^^^^^^^^^^^^^^^^^^^^^^ method not found in `&collection::Collection<S, V>`

error[E0599]: no method named `auto_extraction_config` found for reference `&collection::Collection<S, V>` in the current scope
  --> crates/contextra-engine/src/collection/crud/insert.rs:93:50
   |
93 |             if let Some((generator, cfg)) = self.auto_extraction_config() {
   |                                                  ^^^^^^^^^^^^^^^^^^^^^^ method not found in `&collection::Collection<S, V>`

error[E0599]: no method named `check_and_trigger_community_detection` found for reference `&collection::Collection<S, V>` in the current scope
   --> crates/contextra-engine/src/collection/crud/insert.rs:225:22
    |
225 |                 self.check_and_trigger_community_detection(1);
    |                      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ method not found in `&collection::Collection<S, V>`

error[E0599]: no method named `check_and_trigger_community_detection` found for reference `&collection::Collection<S, V>` in the current scope
   --> crates/contextra-engine/src/collection/crud/insert.rs:425:14
    |
425 |         self.check_and_trigger_community_detection(docs.len());
    |              ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ method not found in `&collection::Collection<S, V>`

error[E0599]: no method named `check_and_trigger_community_detection` found for reference `&collection::Collection<S, V>` in the current scope
   --> crates/contextra-engine/src/collection/crud/insert.rs:463:22
    |
463 |                 self.check_and_trigger_community_detection(1);
    |                      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ method not found in `&collection::Collection<S, V>`

error[E0599]: no method named `check_and_trigger_community_detection` found for reference `&collection::Collection<S, V>` in the current scope
   --> crates/contextra-engine/src/collection/crud/insert.rs:545:14
    |
545 |         self.check_and_trigger_community_detection(docs.len());
    |              ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ method not found in `&collection::Collection<S, V>`

warning: use of deprecated associated function `contextra_types::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
  --> crates/contextra-engine/src/collection/crud/read.rs:61:50
   |
61 | ...                   return Ok(Some(TxId::new(u)));
   |                                            ^^^

error[E0599]: no method named `check_and_trigger_community_detection` found for reference `&collection::Collection<S, V>` in the current scope
  --> crates/contextra-engine/src/collection/crud/update.rs:32:22
   |
32 |                 self.check_and_trigger_community_detection(1);
   |                      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ method not found in `&collection::Collection<S, V>`

error[E0609]: no field `graph_index` on type `&collection::Collection<S, V>`
   --> crates/contextra-engine/src/collection/maintenance.rs:295:50
    |
295 | ...                   self.graph_index.add_entity(recovery_tx, entity).await
    |                            ^^^^^^^^^^^ unknown field
    |
    = note: available fields are: `name`, `prefix`, `index`, `text_index`, `storage` ... and 12 others

error[E0609]: no field `graph_index` on type `&collection::Collection<S, V>`
   --> crates/contextra-engine/src/collection/maintenance.rs:331:50
    |
331 | ...                   if let Err(e) = self.graph_index.remove_entity(recovery_tx, eid).await {
    |                                            ^^^^^^^^^^^ unknown field
    |
    = note: available fields are: `name`, `prefix`, `index`, `text_index`, `storage` ... and 12 others

error[E0609]: no field `graph_index` on type `&collection::Collection<S, V>`
   --> crates/contextra-engine/src/collection/maintenance.rs:353:18
    |
353 |             self.graph_index.commit(recovery_tx).await?;
    |                  ^^^^^^^^^^^ unknown field
    |
    = note: available fields are: `name`, `prefix`, `index`, `text_index`, `storage` ... and 12 others

warning: use of deprecated associated function `contextra_types::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
   --> crates/contextra-engine/src/collection/maintenance.rs:603:79
    |
603 | ...                   let effective = imp.effective_score(TxId::new(now_tx));
    |                                                                 ^^^

warning: use of deprecated associated function `contextra_types::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
   --> crates/contextra-engine/src/collection/query_builder/builder_exec.rs:147:77
    |
147 |             let as_of_tx = self.current_tx.unwrap_or(contextra_types::TxId::new(seq_no));
    |                                                                             ^^^

warning: use of deprecated associated function `contextra_types::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
   --> crates/contextra-engine/src/collection/query_builder/builder_exec.rs:156:51
    |
156 |                 .unwrap_or(contextra_types::TxId::new(seq_no));
    |                                                   ^^^

warning: use of deprecated associated function `contextra_types::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
   --> crates/contextra-engine/src/collection/query_builder/builder_exec.rs:393:77
    |
393 |             let as_of_tx = self.current_tx.unwrap_or(contextra_types::TxId::new(seq_no));
    |                                                                             ^^^

warning: use of deprecated associated function `contextra_types::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
   --> crates/contextra-engine/src/collection/query_builder/builder_exec.rs:402:51
    |
402 |                 .unwrap_or(contextra_types::TxId::new(seq_no));
    |                                                   ^^^

error[E0599]: no method named `get_community` found for reference `&collection::Collection<S, V>` in the current scope
   --> crates/contextra-engine/src/collection/search/hybrid/query.rs:424:26
    |
424 |                     self.get_community(same_comm_entity).await?
    |                          ^^^^^^^^^^^^^ method not found in `&collection::Collection<S, V>`
    |
    = help: items from traits can only be used if the trait is implemented and in scope
    = note: the following trait defines an item `get_community`, perhaps you need to implement it:
            candidate #1: `CommunityResolver`

error[E0599]: no method named `get_community` found for reference `&collection::Collection<S, V>` in the current scope
   --> crates/contextra-engine/src/collection/search/hybrid/mod.rs:187:22
    |
187 |                 self.get_community(same_comm_entity).await?
    |                      ^^^^^^^^^^^^^ method not found in `&collection::Collection<S, V>`
    |
    = help: items from traits can only be used if the trait is implemented and in scope
    = note: the following trait defines an item `get_community`, perhaps you need to implement it:
            candidate #1: `CommunityResolver`

error[E0599]: no method named `get_communities_batch` found for reference `&collection::Collection<S, V>` in the current scope
   --> crates/contextra-engine/src/collection/search/hybrid/mod.rs:264:34
    |
264 |         let community_map = self.get_communities_batch(&candidate_eids).await?;
    |                                  ^^^^^^^^^^^^^^^^^^^^^ method not found in `&collection::Collection<S, V>`

warning: use of deprecated associated function `contextra_types::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
   --> crates/contextra-engine/src/export.rs:144:44
    |
144 |                     contextra_types::TxId::new(u64::MAX),
    |                                            ^^^

error[E0599]: no method named `relate` found for reference `&collection::Collection<S, V>` in the current scope
   --> crates/contextra-engine/src/import.rs:90:24
    |
90  |             match self.relate(&rel.from, &rel.to, &rel.label).await {
    |                        ^^^^^^
    |
help: there is a method `delete` with a similar name, but with different arguments
   --> crates/contextra-engine/src/collection/crud/delete.rs:182:5
    |
182 |     pub async fn delete(&self, id: &str) -> Result<()> {
    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated associated function `contextra_types::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
  --> crates/contextra-engine/src/transaction/cleanup.rs:24:32
   |
24 |                 let tx = TxId::new(next_tx.fetch_add(1, std::sync::atomic::Ordering::SeqCst));
   |                                ^^^

error[E0609]: no field `graph_index` on type `collection::Collection<S, V>`
   --> crates/contextra-engine/src/transaction/db_transaction_commit.rs:151:34
    |
151 | ...                   .graph_index
    |                        ^^^^^^^^^^^ unknown field
    |
    = note: available fields are: `name`, `prefix`, `index`, `text_index`, `storage` ... and 12 others

error[E0609]: no field `graph_index` on type `collection::Collection<S, V>`
   --> crates/contextra-engine/src/transaction/db_transaction_commit.rs:157:34
    |
157 | ...                   .graph_index
    |                        ^^^^^^^^^^^ unknown field
    |
    = note: available fields are: `name`, `prefix`, `index`, `text_index`, `storage` ... and 12 others

error[E0609]: no field `graph_index` on type `collection::Collection<S, V>`
   --> crates/contextra-engine/src/transaction/db_transaction_commit.rs:175:22
    |
175 |                     .graph_index
    |                      ^^^^^^^^^^^ unknown field
    |
    = note: available fields are: `name`, `prefix`, `index`, `text_index`, `storage` ... and 12 others

error[E0609]: no field `graph_index` on type `collection::Collection<S, V>`
   --> crates/contextra-engine/src/transaction/db_transaction_commit.rs:190:22
    |
190 |                     .graph_index
    |                      ^^^^^^^^^^^ unknown field
    |
    = note: available fields are: `name`, `prefix`, `index`, `text_index`, `storage` ... and 12 others

error[E0609]: no field `graph_index` on type `collection::Collection<S, V>`
   --> crates/contextra-engine/src/transaction/db_transaction_commit.rs:224:34
    |
224 | ...                   .graph_index
    |                        ^^^^^^^^^^^ unknown field
    |
    = note: available fields are: `name`, `prefix`, `index`, `text_index`, `storage` ... and 12 others

error[E0609]: no field `graph_index` on type `collection::Collection<S, V>`
   --> crates/contextra-engine/src/transaction/db_transaction_commit.rs:230:51
    |
230 | ...                   &*self.collection.graph_index,
    |                                         ^^^^^^^^^^^ unknown field
    |
    = note: available fields are: `name`, `prefix`, `index`, `text_index`, `storage` ... and 12 others

error[E0609]: no field `graph_index` on type `collection::Collection<S, V>`
   --> crates/contextra-engine/src/transaction/db_transaction_commit.rs:249:56
    |
249 |                 GraphIndex::add_edge(&*self.collection.graph_index, self.tx_id, edge).await?;
    |                                                        ^^^^^^^^^^^ unknown field
    |
    = note: available fields are: `name`, `prefix`, `index`, `text_index`, `storage` ... and 12 others

error[E0609]: no field `graph_index` on type `collection::Collection<S, V>`
   --> crates/contextra-engine/src/transaction/db_transaction_commit.rs:262:22
    |
262 |                     .graph_index
    |                      ^^^^^^^^^^^ unknown field
    |
    = note: available fields are: `name`, `prefix`, `index`, `text_index`, `storage` ... and 12 others

warning: use of deprecated associated function `contextra_types::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
   --> crates/contextra-engine/src/transaction/db_transaction_commit.rs:375:31
    |
375 |         let intent_tx = TxId::new(
    |                               ^^^

warning: use of deprecated associated function `contextra_types::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
   --> crates/contextra-engine/src/transaction/db_transaction_commit.rs:436:35
    |
436 |             let stage1_tx = TxId::new(
    |                                   ^^^

error[E0609]: no field `graph_index` on type `collection::Collection<S, V>`
   --> crates/contextra-engine/src/transaction/db_transaction_commit.rs:487:49
    |
487 |         if let Err(graph_err) = self.collection.graph_index.commit(self.tx_id).await {
    |                                                 ^^^^^^^^^^^ unknown field
    |
    = note: available fields are: `name`, `prefix`, `index`, `text_index`, `storage` ... and 12 others

warning: use of deprecated associated function `contextra_types::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
   --> crates/contextra-engine/src/transaction/db_transaction_commit.rs:500:32
    |
500 |         let cleanup_tx = TxId::new(
    |                                ^^^

warning: use of deprecated associated function `contextra_types::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
  --> crates/contextra-engine/src/transaction/db_transaction_rollback.rs:25:29
   |
25 |         let comp_tx = TxId::new(
   |                             ^^^

warning: use of deprecated associated function `contextra_types::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
  --> crates/contextra-engine/src/transaction/db_transaction_rollback.rs:45:29
   |
45 |         let comp_tx = TxId::new(
   |                             ^^^

warning: use of deprecated associated function `contextra_types::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
  --> crates/contextra-engine/src/transaction/db_transaction_rollback.rs:91:37
   |
91 |             let rollback_tx = TxId::new(
   |                                     ^^^

error[E0609]: no field `graph_index` on type `collection::Collection<S, V>`
   --> crates/contextra-engine/src/transaction/db_transaction_rollback.rs:195:41
    |
195 |         if let Err(e) = self.collection.graph_index.rollback(self.tx_id).await {
    |                                         ^^^^^^^^^^^ unknown field
    |
    = note: available fields are: `name`, `prefix`, `index`, `text_index`, `storage` ... and 12 others

error[E0609]: no field `graph_index` on type `collection::Collection<S, V>`
   --> crates/contextra-engine/src/transaction/db_transaction_rollback.rs:224:41
    |
224 |         let graph_res = self.collection.graph_index.rollback(self.tx_id).await;
    |                                         ^^^^^^^^^^^ unknown field
    |
    = note: available fields are: `name`, `prefix`, `index`, `text_index`, `storage` ... and 12 others

error[E0599]: no method named `set_community_detection_trigger_threshold` found for struct `collection::Collection` in the current scope
   --> crates/contextra-engine/src/contextra_impl/collections.rs:139:13
    |
139 |         col.set_community_detection_trigger_threshold(self.community_detection_threshold);
    |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ method not found in `collection::Collection<TenantScopedStorage<std::sync::Arc<LsmStorage>>>`
    |
   ::: crates/contextra-engine/src/collection/mod.rs:284:1
    |
284 | pub struct Collection<S: StorageEngine = LsmStorage, V: VectorIndex = HnswIndex> {
    | -------------------------------------------------------------------------------- method `set_community_detection_trigger_threshold` not found for this struct

error[E0599]: no method named `set_auto_extraction_config` found for struct `collection::Collection` in the current scope
   --> crates/contextra-engine/src/contextra_impl/collections.rs:163:13
    |
163 |         col.set_auto_extraction_config(auto_cfg);
    |             ^^^^^^^^^^^^^^^^^^^^^^^^^^ method not found in `collection::Collection<TenantScopedStorage<std::sync::Arc<LsmStorage>>>`
    |
   ::: crates/contextra-engine/src/collection/mod.rs:284:1
    |
284 | pub struct Collection<S: StorageEngine = LsmStorage, V: VectorIndex = HnswIndex> {
    | -------------------------------------------------------------------------------- method `set_auto_extraction_config` not found for this struct

error[E0599]: no method named `set_community_detection_trigger_threshold` found for struct `collection::Collection` in the current scope
   --> crates/contextra-engine/src/contextra_impl/collections.rs:252:13
    |
252 |         col.set_community_detection_trigger_threshold(self.community_detection_threshold);
    |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ method not found in `collection::Collection`
    |
   ::: crates/contextra-engine/src/collection/mod.rs:284:1
    |
284 | pub struct Collection<S: StorageEngine = LsmStorage, V: VectorIndex = HnswIndex> {
    | -------------------------------------------------------------------------------- method `set_community_detection_trigger_threshold` not found for this struct

error[E0599]: no method named `set_auto_extraction_config` found for struct `collection::Collection` in the current scope
   --> crates/contextra-engine/src/contextra_impl/collections.rs:276:13
    |
276 |         col.set_auto_extraction_config(auto_cfg);
    |             ^^^^^^^^^^^^^^^^^^^^^^^^^^ method not found in `collection::Collection`
    |
   ::: crates/contextra-engine/src/collection/mod.rs:284:1
    |
284 | pub struct Collection<S: StorageEngine = LsmStorage, V: VectorIndex = HnswIndex> {
    | -------------------------------------------------------------------------------- method `set_auto_extraction_config` not found for this struct

warning: use of deprecated associated function `contextra_crypto::deletion_proof::proof_v3::<impl contextra_crypto::DeletionProof>::create`: Creation of v3 deletion proofs is forbidden per §20 and ADR-13. Use ProofBuilder for v4 deletion proofs.
   --> crates/contextra-engine/src/contextra_impl/collections.rs:491:36
    |
491 |         let proof = DeletionProof::create(
    |                                    ^^^^^^

error[E0599]: no method named `relate` found for struct `std::sync::Arc<collection::Collection>` in the current scope
   --> crates/contextra-engine/src/contextra_impl/crud.rs:297:13
    |
297 |         col.relate(from, to, label).await
    |             ^^^^^^
    |
help: there is a method `delete` with a similar name, but with different arguments
   --> crates/contextra-engine/src/collection/crud/delete.rs:182:5
    |
182 |     pub async fn delete(&self, id: &str) -> Result<()> {
    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

error[E0599]: no method named `relate_bidirectional` found for struct `std::sync::Arc<collection::Collection>` in the current scope
   --> crates/contextra-engine/src/contextra_impl/crud.rs:303:13
    |
303 |         col.relate_bidirectional(from, to, label).await
    |             ^^^^^^^^^^^^^^^^^^^^ method not found in `std::sync::Arc<collection::Collection>`

warning: use of deprecated associated function `contextra_types::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
  --> crates/contextra-engine/src/contextra_impl/lifecycle.rs:85:28
   |
85 |             let tx = TxId::new(0);
   |                            ^^^

warning: use of deprecated associated function `contextra_types::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
  --> crates/contextra-engine/src/collection/tx.rs:26:18
   |
26 |         Ok(TxId::new(id))
   |                  ^^^

error[E0599]: no variant or associated item named `TenantIsolationViolation` found for enum `contextra_types::ContextraError` in the current scope
   --> crates/contextra-engine/src/collection/mod.rs:513:46
    |
513 |         Err(contextra_types::ContextraError::TenantIsolationViolation(
    |                                              ^^^^^^^^^^^^^^^^^^^^^^^^ variant or associated item not found in `contextra_types::ContextraError`
    |
note: if you're trying to build a new `contextra_types::ContextraError` consider using one of the following associated functions:
      contextra_types::ContextraError::limit_exceeded
      contextra_types::ContextraError::model_load
      contextra_types::ContextraError::snapshot_unsupported_for_signal
      contextra_types::ContextraError::orphaned_vector_reference
      and 11 others
   --> /app/crates/contextra-types/src/error.rs:372:5
    |
372 |       pub fn limit_exceeded(limit: usize, context: impl Into<String>) -> Self {
    |       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
...
380 |       pub fn model_load(path: impl Into<String>, reason: impl Into<String>) -> Self {
    |       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
...
388 |       pub fn snapshot_unsupported_for_signal(strategy: impl Into<String>) -> Self {
    |       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
...
393 | /     pub fn orphaned_vector_reference(
394 | |         doc_id: impl Into<String>,
395 | |         index_id: impl Into<String>,
396 | |     ) -> Self {
    | |_____________^

warning: use of deprecated associated function `contextra_types::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
   --> crates/contextra-engine/src/temporal_filter.rs:125:20
    |
125 |         .map(TxId::new);
    |                    ^^^

warning: use of deprecated associated function `contextra_types::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
   --> crates/contextra-engine/src/temporal_filter.rs:131:20
    |
131 |         .map(TxId::new);
    |                    ^^^

warning: use of deprecated associated function `contextra_types::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
   --> crates/contextra-engine/src/temporal_filter.rs:212:46
    |
212 |             window.is_valid_bitemporal(TxId::new(0), Some(as_of_timestamp))
    |                                              ^^^

warning: use of deprecated associated function `contextra_types::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
  --> crates/contextra-engine/src/transaction/compensating_actions.rs:93:41
   |
93 |                 let rollback_tx = TxId::new(
   |                                         ^^^

warning: use of deprecated associated function `contextra_types::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
   --> crates/contextra-engine/src/transaction/compensating_actions.rs:230:33
    |
230 |             let comp_tx = TxId::new(
    |                                 ^^^

error[E0609]: no field `graph_index` on type `collection::Collection<S, V>`
   --> crates/contextra-engine/src/transaction/compensating_actions.rs:239:22
    |
239 |                     .graph_index
    |                      ^^^^^^^^^^^ unknown field
    |
    = note: available fields are: `name`, `prefix`, `index`, `text_index`, `storage` ... and 12 others

error[E0609]: no field `graph_index` on type `collection::Collection<S, V>`
   --> crates/contextra-engine/src/transaction/compensating_actions.rs:250:45
    |
250 |             if let Err(e) = self.collection.graph_index.commit(comp_tx).await {
    |                                             ^^^^^^^^^^^ unknown field
    |
    = note: available fields are: `name`, `prefix`, `index`, `text_index`, `storage` ... and 12 others

warning: use of deprecated associated function `contextra_types::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
   --> crates/contextra-engine/src/transaction/compensating_actions.rs:290:33
    |
290 |             let comp_tx = TxId::new(
    |                                 ^^^

warning: use of deprecated associated function `contextra_types::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
   --> crates/contextra-engine/src/transaction/compensating_actions.rs:330:33
    |
330 |             let comp_tx = TxId::new(
    |                                 ^^^

error[E0609]: no field `graph_index` on type `collection::Collection<S, V>`
   --> crates/contextra-engine/src/transaction/compensating_actions.rs:372:45
    |
372 |             if let Err(e) = self.collection.graph_index.rollback(self.tx_id).await {
    |                                             ^^^^^^^^^^^ unknown field
    |
    = note: available fields are: `name`, `prefix`, `index`, `text_index`, `storage` ... and 12 others

error[E0609]: no field `graph_index` on type `collection::Collection<S, V>`
   --> crates/contextra-engine/src/transaction/db_transaction.rs:235:48
    |
235 |                     if let Err(e) = collection.graph_index.rollback(tx_id).await {
    |                                                ^^^^^^^^^^^ unknown field
    |
    = note: available fields are: `name`, `prefix`, `index`, `text_index`, `storage` ... and 12 others

warning: use of deprecated associated function `contextra_types::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
   --> crates/contextra-engine/src/contextra_impl/collections.rs:302:18
    |
302 |         Ok(TxId::new(id))
    |                  ^^^

warning: unused import: `contextra_mvcc::SsiValidator`
 --> crates/contextra-engine/src/transaction/db_transaction_commit.rs:8:5
  |
8 | use contextra_mvcc::SsiValidator;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: irrefutable `if let` pattern
   --> crates/contextra-engine/src/collection/mod.rs:271:12
    |
271 |         if let Ok(id) = u64::try_from(doc.inner()) {
    |            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = note: this pattern will always match, so the `if let` is useless
    = help: consider replacing the `if let` with a `let`
    = note: `#[warn(irrefutable_let_patterns)]` on by default

Some errors have detailed explanations: E0432, E0599, E0609.
For more information about an error, try `rustc --explain E0432`.
warning: `contextra-engine` (lib) generated 29 warnings
error: could not compile `contextra-engine` (lib) due to 40 previous errors; 29 warnings emitted
```

### Befehl 2: Kompilierprüfung Tests Workspace

- **Befehl:** `cargo test --workspace --locked --no-run`
- **Commit-Hash:** `67c8cd3eec4e373d14286c6d49337258b0aa1d3c`
- **Exit-Code:** `101`
- **Datum:** 2026-10-15
- **Ausgabe:**
```
warning: associated function `new` is never used
  --> crates/contextra-crypto/src/deletion_witness.rs:69:19
   |
66 | impl WitnessIssuerToken {
   | ----------------------- associated function in this implementation
...
69 |     pub(crate) fn new() -> Self {
   |                   ^^^
   |
   = note: `#[warn(dead_code)]` on by default

warning: field `lifecycle_state` is never read
   --> crates/contextra-crypto/src/kv_shredding.rs:149:5
    |
144 | struct GroupEntry {
    |        ---------- field in this struct
...
149 |     lifecycle_state: KeyLifecycleState,
    |     ^^^^^^^^^^^^^^^
    |
    = note: `GroupEntry` has a derived impl for the trait `Debug`, but this is intentionally ignored during dead code analysis

warning: `contextra-crypto` (lib) generated 2 warnings
warning: use of deprecated associated function `contextra_types::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
   --> crates/contextra-vector/src/hnsw/vector_index_impl.rs:644:18
    |
644 |         Ok(TxId::new(self.inner.hot.last_tx_id.load(Ordering::SeqCst)))
    |                  ^^^
    |
    = note: `#[warn(deprecated)]` on by default

warning: use of deprecated associated function `contextra_types::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
   --> crates/contextra-vector/src/hnsw/witness.rs:117:52
    |
117 |         let scanned_at_tx = contextra_types::TxId::new(
    |                                                    ^^^

warning: `contextra-vector` (lib) generated 2 warnings
warning: use of deprecated associated function `contextra_types::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
   --> crates/contextra-store/src/lsm/recovery.rs:807:39
    |
807 |                 .rollback_to_tx(TxId::new(target_tx))
    |                                       ^^^
    |
    = note: `#[warn(deprecated)]` on by default

warning: use of deprecated associated function `contextra_types::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
   --> crates/contextra-store/src/lsm/ops/compaction.rs:160:59
    |
160 |         storage.advance_visibility(contextra_types::TxId::new(sst_max_tx));
    |                                                           ^^^

warning: use of deprecated associated function `contextra_types::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
  --> crates/contextra-store/src/lsm/ops/maintenance.rs:52:14
   |
52 |     Ok(TxId::new(storage.last_committed_tx.load(Ordering::SeqCst)))
   |              ^^^

warning: use of deprecated associated function `contextra_types::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
   --> crates/contextra-store/src/wal/encode.rs:348:35
    |
348 |                 let tx_id = TxId::new(u64::from_le_bytes(
    |                                   ^^^

warning: use of deprecated associated function `contextra_types::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
   --> crates/contextra-store/src/wal/encode.rs:408:35
    |
408 |                 let tx_id = TxId::new(u64::from_le_bytes(
    |                                   ^^^

warning: use of deprecated associated function `contextra_types::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
   --> crates/contextra-store/src/wal/encode.rs:437:35
    |
437 |                 let tx_id = TxId::new(u64::from_le_bytes(
    |                                   ^^^

warning: `contextra-store` (lib) generated 6 warnings
warning: unused import: `contextra_types::DeterministicTimestamp`
  --> crates/contextra-testkit/src/manual_clock.rs:10:5
   |
10 | use contextra_types::DeterministicTimestamp;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(unused_imports)]` on by default

warning: use of deprecated associated function `contextra_ports::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
  --> crates/contextra-testkit/src/in_memory_store.rs:39:49
   |
39 |             last_tx: Arc::new(RwLock::new(TxId::new(0))),
   |                                                 ^^^
   |
   = note: `#[warn(deprecated)]` on by default

warning: unused import: `SeededRng`
 --> crates/contextra-testkit/src/dst/rng.rs:6:53
  |
6 | use contextra_ports::{EntropyRng as EntropyRngPort, SeededRng, SystemRng};
  |                                                     ^^^^^^^^^

warning: `contextra-testkit` (lib) generated 3 warnings (run `cargo fix --lib -p contextra-testkit` to apply 1 suggestion)
   Compiling contextra-engine v0.1.0 (/app/crates/contextra-engine)
   Compiling wat v1.259.0
warning: contextra-wire@0.1.0: flatc not found, using existing generated code.
   Compiling hdrhistogram v7.6.0
   Compiling contextra-wire v0.1.0 (/app/crates/contextra-wire)
error[E0432]: unresolved import `super::auto_extraction`
 --> crates/contextra-engine/src/collection/crud/insert.rs:1:12
  |
1 | use super::auto_extraction::auto_extract_and_relate;
  |            ^^^^^^^^^^^^^^^ could not find `auto_extraction` in `super`

error[E0432]: unresolved imports `crate::collection::crud::AutoExtractionConfig`, `crate::collection::crud::AutoExtractionMode`
 --> crates/contextra-engine/src/contextra_impl/collections.rs:1:31
  |
1 | use crate::collection::crud::{AutoExtractionConfig, AutoExtractionMode};
  |                               ^^^^^^^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^^^^ no `AutoExtractionMode` in `collection::crud`
  |                               |
  |                               no `AutoExtractionConfig` in `collection::crud`
  |
  = help: consider importing one of these enums instead:
          contextra_mvcc::AutoExtractionMode
          contextra_ports::AutoExtractionMode
          contextra_types::AutoExtractionMode

error[E0432]: unresolved import `crate::decay_controller`
 --> crates/contextra-engine/src/collection/maintenance.rs:8:12
  |
8 | use crate::decay_controller::{AdaptiveDecayController, DecaySignalInputs};
  |            ^^^^^^^^^^^^^^^^ could not find `decay_controller` in the crate root

warning: unused import: `internal::validate_doc_id`
  --> crates/contextra-engine/src/collection/crud/mod.rs:19:16
   |
19 | pub(super) use internal::validate_doc_id;
   |                ^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(unused_imports)]` on by default

warning: use of deprecated function `temporal_filter::apply_temporal_validity_filter_at`: Use apply_temporal_validity_filter_at_bitemporal with explicit TxId and business time instead
  --> crates/contextra-engine/src/collection/search/mod.rs:25:37
   |
25 |     apply_temporal_validity_filter, apply_temporal_validity_filter_at, FusionResult,
   |                                     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(deprecated)]` on by default

error[E0433]: failed to resolve: could not find `decay_controller` in the crate root
   --> crates/contextra-engine/src/collection/tests/read_tests.rs:240:35
    |
240 |     let decay_controller = crate::decay_controller::AdaptiveDecayController::with_defaults();
    |                                   ^^^^^^^^^^^^^^^^ could not find `decay_controller` in the crate root

error[E0599]: no method named `check_and_trigger_community_detection` found for reference `&collection::Collection<S, V>` in the current scope
   --> crates/contextra-engine/src/collection/crud/delete.rs:141:14
    |
141 |         self.check_and_trigger_community_detection(1);
    |              ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ method not found in `&collection::Collection<S, V>`

error[E0599]: no method named `auto_extraction_config` found for reference `&collection::Collection<S, V>` in the current scope
  --> crates/contextra-engine/src/collection/crud/insert.rs:45:50
   |
45 |             if let Some((generator, cfg)) = self.auto_extraction_config() {
   |                                                  ^^^^^^^^^^^^^^^^^^^^^^ method not found in `&collection::Collection<S, V>`

error[E0599]: no method named `auto_extraction_config` found for reference `&collection::Collection<S, V>` in the current scope
  --> crates/contextra-engine/src/collection/crud/insert.rs:93:50
   |
93 |             if let Some((generator, cfg)) = self.auto_extraction_config() {
   |                                                  ^^^^^^^^^^^^^^^^^^^^^^ method not found in `&collection::Collection<S, V>`

   Compiling contextra-ports v0.1.0 (/app/crates/contextra-ports)
error[E0599]: no method named `check_and_trigger_community_detection` found for reference `&collection::Collection<S, V>` in the current scope
   --> crates/contextra-engine/src/collection/crud/insert.rs:225:22
    |
225 |                 self.check_and_trigger_community_detection(1);
    |                      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ method not found in `&collection::Collection<S, V>`

error[E0599]: no method named `check_and_trigger_community_detection` found for reference `&collection::Collection<S, V>` in the current scope
   --> crates/contextra-engine/src/collection/crud/insert.rs:425:14
    |
425 |         self.check_and_trigger_community_detection(docs.len());
    |              ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ method not found in `&collection::Collection<S, V>`

error[E0599]: no method named `check_and_trigger_community_detection` found for reference `&collection::Collection<S, V>` in the current scope
   --> crates/contextra-engine/src/collection/crud/insert.rs:463:22
    |
463 |                 self.check_and_trigger_community_detection(1);
    |                      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ method not found in `&collection::Collection<S, V>`

error[E0599]: no method named `check_and_trigger_community_detection` found for reference `&collection::Collection<S, V>` in the current scope
   --> crates/contextra-engine/src/collection/crud/insert.rs:545:14
    |
545 |         self.check_and_trigger_community_detection(docs.len());
    |              ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ method not found in `&collection::Collection<S, V>`

warning: use of deprecated associated function `contextra_types::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
  --> crates/contextra-engine/src/collection/crud/read.rs:61:50
   |
61 | ...                   return Ok(Some(TxId::new(u)));
   |                                            ^^^

error[E0599]: no method named `check_and_trigger_community_detection` found for reference `&collection::Collection<S, V>` in the current scope
  --> crates/contextra-engine/src/collection/crud/update.rs:32:22
   |
32 |                 self.check_and_trigger_community_detection(1);
   |                      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ method not found in `&collection::Collection<S, V>`

error[E0609]: no field `graph_index` on type `&collection::Collection<S, V>`
   --> crates/contextra-engine/src/collection/maintenance.rs:295:50
    |
295 | ...                   self.graph_index.add_entity(recovery_tx, entity).await
    |                            ^^^^^^^^^^^ unknown field
    |
    = note: available fields are: `name`, `prefix`, `index`, `text_index`, `storage` ... and 12 others

error[E0609]: no field `graph_index` on type `&collection::Collection<S, V>`
   --> crates/contextra-engine/src/collection/maintenance.rs:331:50
    |
331 | ...                   if let Err(e) = self.graph_index.remove_entity(recovery_tx, eid).await {
    |                                            ^^^^^^^^^^^ unknown field
    |
    = note: available fields are: `name`, `prefix`, `index`, `text_index`, `storage` ... and 12 others

error[E0609]: no field `graph_index` on type `&collection::Collection<S, V>`
   --> crates/contextra-engine/src/collection/maintenance.rs:353:18
    |
353 |             self.graph_index.commit(recovery_tx).await?;
    |                  ^^^^^^^^^^^ unknown field
    |
    = note: available fields are: `name`, `prefix`, `index`, `text_index`, `storage` ... and 12 others

   Compiling contextra-crypto v0.1.0 (/app/crates/contextra-crypto)
warning: use of deprecated associated function `contextra_types::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
   --> crates/contextra-engine/src/collection/maintenance.rs:603:79
    |
603 | ...                   let effective = imp.effective_score(TxId::new(now_tx));
    |                                                                 ^^^

warning: use of deprecated associated function `contextra_types::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
   --> crates/contextra-engine/src/collection/query_builder/builder_exec.rs:147:77
    |
147 |             let as_of_tx = self.current_tx.unwrap_or(contextra_types::TxId::new(seq_no));
    |                                                                             ^^^

warning: use of deprecated associated function `contextra_types::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
   --> crates/contextra-engine/src/collection/query_builder/builder_exec.rs:156:51
    |
156 |                 .unwrap_or(contextra_types::TxId::new(seq_no));
    |                                                   ^^^

warning: use of deprecated associated function `contextra_types::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
   --> crates/contextra-engine/src/collection/query_builder/builder_exec.rs:393:77
    |
393 |             let as_of_tx = self.current_tx.unwrap_or(contextra_types::TxId::new(seq_no));
    |                                                                             ^^^

warning: use of deprecated associated function `contextra_types::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
   --> crates/contextra-engine/src/collection/query_builder/builder_exec.rs:402:51
    |
402 |                 .unwrap_or(contextra_types::TxId::new(seq_no));
    |                                                   ^^^

error[E0599]: no method named `get_community` found for reference `&collection::Collection<S, V>` in the current scope
   --> crates/contextra-engine/src/collection/search/hybrid/query.rs:424:26
    |
424 |                     self.get_community(same_comm_entity).await?
    |                          ^^^^^^^^^^^^^ method not found in `&collection::Collection<S, V>`
    |
    = help: items from traits can only be used if the trait is implemented and in scope
    = note: the following trait defines an item `get_community`, perhaps you need to implement it:
            candidate #1: `CommunityResolver`

error[E0599]: no method named `get_community` found for reference `&collection::Collection<S, V>` in the current scope
   --> crates/contextra-engine/src/collection/search/hybrid/mod.rs:187:22
    |
187 |                 self.get_community(same_comm_entity).await?
    |                      ^^^^^^^^^^^^^ method not found in `&collection::Collection<S, V>`
    |
    = help: items from traits can only be used if the trait is implemented and in scope
    = note: the following trait defines an item `get_community`, perhaps you need to implement it:
            candidate #1: `CommunityResolver`

error[E0599]: no method named `get_communities_batch` found for reference `&collection::Collection<S, V>` in the current scope
   --> crates/contextra-engine/src/collection/search/hybrid/mod.rs:264:34
    |
264 |         let community_map = self.get_communities_batch(&candidate_eids).await?;
    |                                  ^^^^^^^^^^^^^^^^^^^^^ method not found in `&collection::Collection<S, V>`

warning: use of deprecated associated function `contextra_types::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
   --> crates/contextra-engine/src/export.rs:144:44
    |
144 |                     contextra_types::TxId::new(u64::MAX),
    |                                            ^^^

error[E0599]: no method named `relate` found for reference `&collection::Collection<S, V>` in the current scope
   --> crates/contextra-engine/src/import.rs:90:24
    |
90  |             match self.relate(&rel.from, &rel.to, &rel.label).await {
    |                        ^^^^^^
    |
help: there is a method `delete` with a similar name, but with different arguments
   --> crates/contextra-engine/src/collection/crud/delete.rs:182:5
    |
182 |     pub async fn delete(&self, id: &str) -> Result<()> {
    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: use of deprecated associated function `contextra_types::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
  --> crates/contextra-engine/src/transaction/cleanup.rs:24:32
   |
24 |                 let tx = TxId::new(next_tx.fetch_add(1, std::sync::atomic::Ordering::SeqCst));
   |                                ^^^

error[E0609]: no field `graph_index` on type `collection::Collection<S, V>`
   --> crates/contextra-engine/src/transaction/db_transaction_commit.rs:151:34
    |
151 | ...                   .graph_index
    |                        ^^^^^^^^^^^ unknown field
    |
    = note: available fields are: `name`, `prefix`, `index`, `text_index`, `storage` ... and 12 others

error[E0609]: no field `graph_index` on type `collection::Collection<S, V>`
   --> crates/contextra-engine/src/transaction/db_transaction_commit.rs:157:34
    |
157 | ...                   .graph_index
    |                        ^^^^^^^^^^^ unknown field
    |
    = note: available fields are: `name`, `prefix`, `index`, `text_index`, `storage` ... and 12 others

error[E0609]: no field `graph_index` on type `collection::Collection<S, V>`
   --> crates/contextra-engine/src/transaction/db_transaction_commit.rs:175:22
    |
175 |                     .graph_index
    |                      ^^^^^^^^^^^ unknown field
    |
    = note: available fields are: `name`, `prefix`, `index`, `text_index`, `storage` ... and 12 others

error[E0609]: no field `graph_index` on type `collection::Collection<S, V>`
   --> crates/contextra-engine/src/transaction/db_transaction_commit.rs:190:22
    |
190 |                     .graph_index
    |                      ^^^^^^^^^^^ unknown field
    |
    = note: available fields are: `name`, `prefix`, `index`, `text_index`, `storage` ... and 12 others

error[E0609]: no field `graph_index` on type `collection::Collection<S, V>`
   --> crates/contextra-engine/src/transaction/db_transaction_commit.rs:224:34
    |
224 | ...                   .graph_index
    |                        ^^^^^^^^^^^ unknown field
    |
    = note: available fields are: `name`, `prefix`, `index`, `text_index`, `storage` ... and 12 others

error[E0609]: no field `graph_index` on type `collection::Collection<S, V>`
   --> crates/contextra-engine/src/transaction/db_transaction_commit.rs:230:51
    |
230 | ...                   &*self.collection.graph_index,
    |                                         ^^^^^^^^^^^ unknown field
    |
    = note: available fields are: `name`, `prefix`, `index`, `text_index`, `storage` ... and 12 others

error[E0609]: no field `graph_index` on type `collection::Collection<S, V>`
   --> crates/contextra-engine/src/transaction/db_transaction_commit.rs:249:56
    |
249 |                 GraphIndex::add_edge(&*self.collection.graph_index, self.tx_id, edge).await?;
    |                                                        ^^^^^^^^^^^ unknown field
    |
    = note: available fields are: `name`, `prefix`, `index`, `text_index`, `storage` ... and 12 others

error[E0609]: no field `graph_index` on type `collection::Collection<S, V>`
   --> crates/contextra-engine/src/transaction/db_transaction_commit.rs:262:22
    |
262 |                     .graph_index
    |                      ^^^^^^^^^^^ unknown field
    |
    = note: available fields are: `name`, `prefix`, `index`, `text_index`, `storage` ... and 12 others

warning: use of deprecated associated function `contextra_types::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
   --> crates/contextra-engine/src/transaction/db_transaction_commit.rs:375:31
    |
375 |         let intent_tx = TxId::new(
    |                               ^^^

warning: use of deprecated associated function `contextra_types::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
   --> crates/contextra-engine/src/transaction/db_transaction_commit.rs:436:35
    |
436 |             let stage1_tx = TxId::new(
    |                                   ^^^

error[E0609]: no field `graph_index` on type `collection::Collection<S, V>`
   --> crates/contextra-engine/src/transaction/db_transaction_commit.rs:487:49
    |
487 |         if let Err(graph_err) = self.collection.graph_index.commit(self.tx_id).await {
    |                                                 ^^^^^^^^^^^ unknown field
    |
    = note: available fields are: `name`, `prefix`, `index`, `text_index`, `storage` ... and 12 others

warning: use of deprecated associated function `contextra_types::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
   --> crates/contextra-engine/src/transaction/db_transaction_commit.rs:500:32
    |
500 |         let cleanup_tx = TxId::new(
    |                                ^^^

warning: use of deprecated associated function `contextra_types::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
  --> crates/contextra-engine/src/transaction/db_transaction_rollback.rs:25:29
   |
25 |         let comp_tx = TxId::new(
   |                             ^^^

warning: use of deprecated associated function `contextra_types::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
  --> crates/contextra-engine/src/transaction/db_transaction_rollback.rs:45:29
   |
45 |         let comp_tx = TxId::new(
   |                             ^^^

warning: use of deprecated associated function `contextra_types::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
  --> crates/contextra-engine/src/transaction/db_transaction_rollback.rs:91:37
   |
91 |             let rollback_tx = TxId::new(
   |                                     ^^^

error[E0609]: no field `graph_index` on type `collection::Collection<S, V>`
   --> crates/contextra-engine/src/transaction/db_transaction_rollback.rs:195:41
    |
195 |         if let Err(e) = self.collection.graph_index.rollback(self.tx_id).await {
    |                                         ^^^^^^^^^^^ unknown field
    |
    = note: available fields are: `name`, `prefix`, `index`, `text_index`, `storage` ... and 12 others

error[E0609]: no field `graph_index` on type `collection::Collection<S, V>`
   --> crates/contextra-engine/src/transaction/db_transaction_rollback.rs:224:41
    |
224 |         let graph_res = self.collection.graph_index.rollback(self.tx_id).await;
    |                                         ^^^^^^^^^^^ unknown field
    |
    = note: available fields are: `name`, `prefix`, `index`, `text_index`, `storage` ... and 12 others

error[E0599]: no method named `set_community_detection_trigger_threshold` found for struct `collection::Collection` in the current scope
   --> crates/contextra-engine/src/contextra_impl/collections.rs:139:13
    |
139 |         col.set_community_detection_trigger_threshold(self.community_detection_threshold);
    |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ method not found in `collection::Collection<TenantScopedStorage<std::sync::Arc<LsmStorage>>>`
    |
   ::: crates/contextra-engine/src/collection/mod.rs:284:1
    |
284 | pub struct Collection<S: StorageEngine = LsmStorage, V: VectorIndex = HnswIndex> {
    | -------------------------------------------------------------------------------- method `set_community_detection_trigger_threshold` not found for this struct

error[E0599]: no method named `set_auto_extraction_config` found for struct `collection::Collection` in the current scope
   --> crates/contextra-engine/src/contextra_impl/collections.rs:163:13
    |
163 |         col.set_auto_extraction_config(auto_cfg);
    |             ^^^^^^^^^^^^^^^^^^^^^^^^^^ method not found in `collection::Collection<TenantScopedStorage<std::sync::Arc<LsmStorage>>>`
    |
   ::: crates/contextra-engine/src/collection/mod.rs:284:1
    |
284 | pub struct Collection<S: StorageEngine = LsmStorage, V: VectorIndex = HnswIndex> {
    | -------------------------------------------------------------------------------- method `set_auto_extraction_config` not found for this struct

error[E0599]: no method named `set_community_detection_trigger_threshold` found for struct `collection::Collection` in the current scope
   --> crates/contextra-engine/src/contextra_impl/collections.rs:252:13
    |
252 |         col.set_community_detection_trigger_threshold(self.community_detection_threshold);
    |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ method not found in `collection::Collection`
    |
   ::: crates/contextra-engine/src/collection/mod.rs:284:1
    |
284 | pub struct Collection<S: StorageEngine = LsmStorage, V: VectorIndex = HnswIndex> {
    | -------------------------------------------------------------------------------- method `set_community_detection_trigger_threshold` not found for this struct

error[E0599]: no method named `set_auto_extraction_config` found for struct `collection::Collection` in the current scope
   --> crates/contextra-engine/src/contextra_impl/collections.rs:276:13
    |
276 |         col.set_auto_extraction_config(auto_cfg);
    |             ^^^^^^^^^^^^^^^^^^^^^^^^^^ method not found in `collection::Collection`
    |
   ::: crates/contextra-engine/src/collection/mod.rs:284:1
    |
284 | pub struct Collection<S: StorageEngine = LsmStorage, V: VectorIndex = HnswIndex> {
    | -------------------------------------------------------------------------------- method `set_auto_extraction_config` not found for this struct

warning: use of deprecated associated function `contextra_crypto::deletion_proof::proof_v3::<impl contextra_crypto::DeletionProof>::create`: Creation of v3 deletion proofs is forbidden per §20 and ADR-13. Use ProofBuilder for v4 deletion proofs.
   --> crates/contextra-engine/src/contextra_impl/collections.rs:491:36
    |
491 |         let proof = DeletionProof::create(
    |                                    ^^^^^^

warning: unused `Result` that must be used
   --> crates/contextra-crypto/tests/proof_v4_policy_rules.rs:323:5
    |
323 |     non_critical_field.encode(&mut canon);
    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = note: this `Result` may be an `Err` variant, which should be handled
    = note: `#[warn(unused_must_use)]` on by default
help: use `let _ = ...` to ignore the resulting value
    |
323 |     let _ = non_critical_field.encode(&mut canon);
    |     +++++++

error[E0599]: no method named `relate` found for struct `std::sync::Arc<collection::Collection>` in the current scope
   --> crates/contextra-engine/src/contextra_impl/crud.rs:297:13
    |
297 |         col.relate(from, to, label).await
    |             ^^^^^^
    |
help: there is a method `delete` with a similar name, but with different arguments
   --> crates/contextra-engine/src/collection/crud/delete.rs:182:5
    |
182 |     pub async fn delete(&self, id: &str) -> Result<()> {
    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

error[E0599]: no method named `relate_bidirectional` found for struct `std::sync::Arc<collection::Collection>` in the current scope
   --> crates/contextra-engine/src/contextra_impl/crud.rs:303:13
    |
303 |         col.relate_bidirectional(from, to, label).await
    |             ^^^^^^^^^^^^^^^^^^^^ method not found in `std::sync::Arc<collection::Collection>`

warning: use of deprecated associated function `contextra_types::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
  --> crates/contextra-engine/src/contextra_impl/lifecycle.rs:85:28
   |
85 |             let tx = TxId::new(0);
   |                            ^^^

warning: use of deprecated associated function `contextra_types::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
  --> crates/contextra-engine/src/collection/tx.rs:26:18
   |
26 |         Ok(TxId::new(id))
   |                  ^^^

warning: `contextra-crypto` (test "proof_v4_policy_rules") generated 1 warning
error[E0599]: no variant or associated item named `TenantIsolationViolation` found for enum `contextra_types::ContextraError` in the current scope
   --> crates/contextra-engine/src/collection/mod.rs:513:46
    |
513 |         Err(contextra_types::ContextraError::TenantIsolationViolation(
    |                                              ^^^^^^^^^^^^^^^^^^^^^^^^ variant or associated item not found in `contextra_types::ContextraError`
    |
note: if you're trying to build a new `contextra_types::ContextraError` consider using one of the following associated functions:
      contextra_types::ContextraError::limit_exceeded
      contextra_types::ContextraError::model_load
      contextra_types::ContextraError::snapshot_unsupported_for_signal
      contextra_types::ContextraError::orphaned_vector_reference
      and 11 others
   --> /app/crates/contextra-types/src/error.rs:372:5
    |
372 |       pub fn limit_exceeded(limit: usize, context: impl Into<String>) -> Self {
    |       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
...
380 |       pub fn model_load(path: impl Into<String>, reason: impl Into<String>) -> Self {
    |       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
...
388 |       pub fn snapshot_unsupported_for_signal(strategy: impl Into<String>) -> Self {
    |       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
...
393 | /     pub fn orphaned_vector_reference(
394 | |         doc_id: impl Into<String>,
395 | |         index_id: impl Into<String>,
396 | |     ) -> Self {
    | |_____________^

warning: use of deprecated associated function `contextra_types::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
   --> crates/contextra-engine/src/background_workers/tests.rs:232:21
    |
232 |     let tx1 = TxId::new(1);
    |                     ^^^

warning: use of deprecated associated function `contextra_types::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
   --> crates/contextra-engine/src/temporal_filter.rs:125:20
    |
125 |         .map(TxId::new);
    |                    ^^^

warning: use of deprecated associated function `contextra_types::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
   --> crates/contextra-engine/src/temporal_filter.rs:131:20
    |
131 |         .map(TxId::new);
    |                    ^^^

warning: use of deprecated associated function `contextra_types::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
   --> crates/contextra-engine/src/temporal_filter.rs:212:46
    |
212 |             window.is_valid_bitemporal(TxId::new(0), Some(as_of_timestamp))
    |                                              ^^^

warning: use of deprecated associated function `contextra_types::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
  --> crates/contextra-engine/src/transaction/compensating_actions.rs:93:41
   |
93 |                 let rollback_tx = TxId::new(
   |                                         ^^^

warning: use of deprecated associated function `contextra_types::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
   --> crates/contextra-engine/src/transaction/compensating_actions.rs:230:33
    |
230 |             let comp_tx = TxId::new(
    |                                 ^^^

error[E0609]: no field `graph_index` on type `collection::Collection<S, V>`
   --> crates/contextra-engine/src/transaction/compensating_actions.rs:239:22
    |
239 |                     .graph_index
    |                      ^^^^^^^^^^^ unknown field
    |
    = note: available fields are: `name`, `prefix`, `index`, `text_index`, `storage` ... and 12 others

error[E0609]: no field `graph_index` on type `collection::Collection<S, V>`
   --> crates/contextra-engine/src/transaction/compensating_actions.rs:250:45
    |
250 |             if let Err(e) = self.collection.graph_index.commit(comp_tx).await {
    |                                             ^^^^^^^^^^^ unknown field
    |
    = note: available fields are: `name`, `prefix`, `index`, `text_index`, `storage` ... and 12 others

warning: use of deprecated associated function `contextra_types::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
   --> crates/contextra-engine/src/transaction/compensating_actions.rs:290:33
    |
290 |             let comp_tx = TxId::new(
    |                                 ^^^

warning: use of deprecated associated function `contextra_types::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
   --> crates/contextra-engine/src/transaction/compensating_actions.rs:330:33
    |
330 |             let comp_tx = TxId::new(
    |                                 ^^^

error[E0609]: no field `graph_index` on type `collection::Collection<S, V>`
   --> crates/contextra-engine/src/transaction/compensating_actions.rs:372:45
    |
372 |             if let Err(e) = self.collection.graph_index.rollback(self.tx_id).await {
    |                                             ^^^^^^^^^^^ unknown field
    |
    = note: available fields are: `name`, `prefix`, `index`, `text_index`, `storage` ... and 12 others

error[E0609]: no field `graph_index` on type `collection::Collection<S, V>`
   --> crates/contextra-engine/src/transaction/db_transaction.rs:235:48
    |
235 |                     if let Err(e) = collection.graph_index.rollback(tx_id).await {
    |                                                ^^^^^^^^^^^ unknown field
    |
    = note: available fields are: `name`, `prefix`, `index`, `text_index`, `storage` ... and 12 others

warning: use of deprecated associated function `contextra_types::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
   --> crates/contextra-engine/src/contextra_impl/collections.rs:302:18
    |
302 |         Ok(TxId::new(id))
    |                  ^^^

warning: unused import: `contextra_mvcc::SsiValidator`
 --> crates/contextra-engine/src/transaction/db_transaction_commit.rs:8:5
  |
8 | use contextra_mvcc::SsiValidator;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

error[E0061]: this function takes 2 arguments but 1 argument was supplied
   --> crates/contextra-engine/src/collection/tests/search_tests.rs:334:17
    |
334 |     let snap1 = ReadSnapshot::new(42);
    |                 ^^^^^^^^^^^^^^^^^----
    |                                  ||
    |                                  |expected `SnapshotId`, found integer
    |                                  argument #2 of type `contextra_types::TxId` is missing
    |
note: associated function defined here
   --> /app/crates/contextra-ports/src/snapshot.rs:39:12
    |
39  |     pub fn new(id: SnapshotId, max_seq: TxId) -> Self {
    |            ^^^
help: try wrapping the expression in `contextra_ports::SnapshotId`
    |
334 |     let snap1 = ReadSnapshot::new(contextra_ports::SnapshotId(42));
    |                                   ++++++++++++++++++++++++++++  +
help: provide the argument
    |
334 -     let snap1 = ReadSnapshot::new(42);
334 +     let snap1 = ReadSnapshot::new(/* SnapshotId */, /* contextra_types::TxId */);
    |

error[E0061]: this function takes 2 arguments but 1 argument was supplied
   --> crates/contextra-engine/src/collection/tests/search_tests.rs:335:17
    |
335 |     let snap2 = ReadSnapshot::new(42);
    |                 ^^^^^^^^^^^^^^^^^----
    |                                  ||
    |                                  |expected `SnapshotId`, found integer
    |                                  argument #2 of type `contextra_types::TxId` is missing
    |
note: associated function defined here
   --> /app/crates/contextra-ports/src/snapshot.rs:39:12
    |
39  |     pub fn new(id: SnapshotId, max_seq: TxId) -> Self {
    |            ^^^
help: try wrapping the expression in `contextra_ports::SnapshotId`
    |
335 |     let snap2 = ReadSnapshot::new(contextra_ports::SnapshotId(42));
    |                                   ++++++++++++++++++++++++++++  +
help: provide the argument
    |
335 -     let snap2 = ReadSnapshot::new(42);
335 +     let snap2 = ReadSnapshot::new(/* SnapshotId */, /* contextra_types::TxId */);
    |

error[E0061]: this function takes 2 arguments but 1 argument was supplied
   --> crates/contextra-engine/src/collection/tests/search_tests.rs:369:20
    |
369 |     let snapshot = ReadSnapshot::new(snap_at_doc1);
    |                    ^^^^^^^^^^^^^^^^^--------------
    |                                     ||
    |                                     |expected `SnapshotId`, found `u64`
    |                                     argument #2 of type `contextra_types::TxId` is missing
    |
note: associated function defined here
   --> /app/crates/contextra-ports/src/snapshot.rs:39:12
    |
39  |     pub fn new(id: SnapshotId, max_seq: TxId) -> Self {
    |            ^^^
help: try wrapping the expression in `contextra_ports::SnapshotId`
    |
369 |     let snapshot = ReadSnapshot::new(contextra_ports::SnapshotId(snap_at_doc1));
    |                                      ++++++++++++++++++++++++++++            +
help: provide the argument
    |
369 -     let snapshot = ReadSnapshot::new(snap_at_doc1);
369 +     let snapshot = ReadSnapshot::new(/* SnapshotId */, /* contextra_types::TxId */);
    |

warning: irrefutable `if let` pattern
   --> crates/contextra-engine/src/collection/mod.rs:271:12
    |
271 |         if let Ok(id) = u64::try_from(doc.inner()) {
    |            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = note: this pattern will always match, so the `if let` is useless
    = help: consider replacing the `if let` with a `let`
    = note: `#[warn(irrefutable_let_patterns)]` on by default

warning: use of deprecated associated function `contextra_types::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
   --> crates/contextra-ports/src/graph_index.rs:403:58
    |
403 |             .traverse_at_time(EntityId::new(1), 2, TxId::new(1))
    |                                                          ^^^
    |
    = note: `#[warn(deprecated)]` on by default

warning: use of deprecated associated function `contextra_types::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
   --> crates/contextra-ports/src/graph_index.rs:414:64
    |
414 |             .traverse_at_bitemporal(EntityId::new(1), 2, TxId::new(1), Some(1000))
    |                                                                ^^^

warning: use of deprecated associated function `contextra_types::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
   --> crates/contextra-ports/src/graph_index.rs:479:58
    |
479 |             .traverse_at_time(EntityId::new(1), 2, TxId::new(10))
    |                                                          ^^^

warning: use of deprecated associated function `contextra_types::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
   --> crates/contextra-engine/src/temporal_filter.rs:248:32
    |
248 |         let current_tx = TxId::new(1);
    |                                ^^^

warning: use of deprecated associated function `contextra_types::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
   --> crates/contextra-engine/src/temporal_filter.rs:265:32
    |
265 |         let current_tx = TxId::new(10);
    |                                ^^^

warning: use of deprecated associated function `contextra_types::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
   --> crates/contextra-engine/src/temporal_filter.rs:304:73
    |
304 |             apply_temporal_validity_filter_at_bitemporal(results, TxId::new(30), Some(30));
    |                                                                         ^^^

warning: use of deprecated associated function `contextra_types::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
   --> crates/contextra-engine/src/temporal_filter.rs:336:73
    |
336 |             apply_temporal_validity_filter_at_bitemporal(results, TxId::new(30), Some(30));
    |                                                                         ^^^

warning: use of deprecated associated function `contextra_types::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
   --> crates/contextra-engine/src/temporal_filter.rs:357:19
    |
357 |             TxId::new(1),
    |                   ^^^

warning: use of deprecated associated function `contextra_types::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
   --> crates/contextra-engine/src/temporal_filter.rs:365:19
    |
365 |             TxId::new(1),
    |                   ^^^

warning: use of deprecated associated function `contextra_types::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
   --> crates/contextra-engine/src/temporal_filter.rs:373:19
    |
373 |             TxId::new(1),
    |                   ^^^

warning: use of deprecated associated function `contextra_types::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
   --> crates/contextra-engine/src/temporal_filter.rs:381:19
    |
381 |             TxId::new(1),
    |                   ^^^

warning: use of deprecated associated function `contextra_types::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
   --> crates/contextra-engine/src/temporal_filter.rs:389:19
    |
389 |             TxId::new(1),
    |                   ^^^

warning: use of deprecated associated function `contextra_types::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
   --> crates/contextra-engine/src/temporal_filter.rs:397:19
    |
397 |             TxId::new(1),
    |                   ^^^

warning: use of deprecated associated function `contextra_types::TxId::new`: Nutze TxId::try_new() oder TxId::internal() — new() umgeht AGT-GRAPH-001 und wird in einer künftigen Version entfernt.
   --> crates/contextra-engine/src/temporal_filter.rs:420:19
    |
420 |             TxId::new(5),
    |                   ^^^

Some errors have detailed explanations: E0432, E0599, E0609.
For more information about an error, try `rustc --explain E0432`.
warning: `contextra-engine` (lib) generated 29 warnings
error: could not compile `contextra-engine` (lib) due to 40 previous errors; 29 warnings emitted
warning: build failed, waiting for other jobs to finish...
warning: `contextra-ports` (lib test) generated 3 warnings
Some errors have detailed explanations: E0061, E0432, E0433, E0599, E0609.
For more information about an error, try `rustc --explain E0061`.
warning: `contextra-engine` (lib test) generated 41 warnings (29 duplicates)
error: could not compile `contextra-engine` (lib test) due to 44 previous errors; 41 warnings emitted
```

### Befehl 3: Hülle contextra-mcp (Default-Features)

- **Befehl:** `cargo tree -p contextra-mcp -e normal --prefix none | grep ^contextra | sort -u | wc -l`
- **Commit-Hash:** `67c8cd3eec4e373d14286c6d49337258b0aa1d3c`
- **Exit-Code:** `0`
- **Datum:** 2026-10-15
- **Ausgabe:**
```
29
```

### Befehl 4: Hülle contextra-mcp (no-default-features)

- **Befehl:** `cargo tree -p contextra-mcp -e normal --no-default-features --prefix none | grep ^contextra | sort -u | wc -l`
- **Commit-Hash:** `67c8cd3eec4e373d14286c6d49337258b0aa1d3c`
- **Exit-Code:** `0`
- **Datum:** 2026-10-15
- **Ausgabe:**
```
29
```

### Befehl 5: Hülle contextra-mcp (all-features)

- **Befehl:** `cargo tree -p contextra-mcp -e normal --all-features --prefix none | grep ^contextra | sort -u | wc -l`
- **Commit-Hash:** `67c8cd3eec4e373d14286c6d49337258b0aa1d3c`
- **Exit-Code:** `0`
- **Datum:** 2026-10-15
- **Ausgabe:**
```
29
```

### Befehl 6: Hülle contextra-engine (no-default-features)

- **Befehl:** `cargo tree -p contextra-engine -e normal --no-default-features --prefix none | grep ^contextra | sort -u | wc -l`
- **Commit-Hash:** `67c8cd3eec4e373d14286c6d49337258b0aa1d3c`
- **Exit-Code:** `0`
- **Datum:** 2026-10-15
- **Ausgabe:**
```
19
```

### Befehl 7: Hülle contextra (Default-Features)

- **Befehl:** `cargo tree -p contextra -e normal --prefix none | grep ^contextra | sort -u | wc -l`
- **Commit-Hash:** `67c8cd3eec4e373d14286c6d49337258b0aa1d3c`
- **Exit-Code:** `0`
- **Datum:** 2026-10-15
- **Ausgabe:**
```
27
```

### Befehl 8: Ring-Layering Check (--strict)

- **Befehl:** `cargo xtask check-ring-layering --strict`
- **Commit-Hash:** `67c8cd3eec4e373d14286c6d49337258b0aa1d3c`
- **Exit-Code:** `1`
- **Datum:** 2026-10-15
- **Ausgabe:**
```
=== Running xtask check-ring-layering (strict=true, all_features=false) ===
❌ check-ring-layering: 5 UNDOKUMENTIERTE Ring-Verstöße/Stale Entries gefunden:
  RING-VIOLATION: contextra-vector (Ring 0) -> contextra-durable-fs (Ring 0) [kind=normal]: Ring 0 backward dependency edge: contextra-vector (order 4) -> contextra-durable-fs (order 99)
  RING-VIOLATION: contextra-crypto (Ring 0) -> contextra-durable-fs (Ring 0) [kind=normal]: Ring 0 backward dependency edge: contextra-crypto (order 1) -> contextra-durable-fs (order 99)
  RING-VIOLATION: contextra-graph (Ring 0) -> contextra-store (Ring 1) [kind=stale]: STALE ALLOWLIST ENTRY: Exception contextra-graph -> contextra-store is no longer active in codebase!
  RING-VIOLATION: contextra-infer-onnx (Ring 0) -> contextra-infer-candle (Ring 0) [kind=stale]: STALE ALLOWLIST ENTRY: Exception contextra-infer-onnx -> contextra-infer-candle is no longer active in codebase!
  RING-VIOLATION: contextra-infer-ollama (Ring 0) -> contextra-infer-onnx (Ring 0) [kind=stale]: STALE ALLOWLIST ENTRY: Exception contextra-infer-ollama -> contextra-infer-onnx is no longer active in codebase!
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.14s
     Running `target/debug/xtask check-ring-layering --strict`
❌ check-ring-layering (--strict active): Fail due to 5 unallowlisted Ring violation(s) / stale entries.
```

### Befehl 9: DAG Check

- **Befehl:** `cargo xtask check-dag`
- **Commit-Hash:** `67c8cd3eec4e373d14286c6d49337258b0aa1d3c`
- **Exit-Code:** `0`
- **Datum:** 2026-10-15
- **Ausgabe:**
```
=== xtask check-dag ===
=== xtask check-dag PASSED ===
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.14s
     Running `target/debug/xtask check-dag`
```

### Befehl 10: Anzahl Dateien mit contextra_core Import

- **Befehl:** `grep -rl "contextra_core" crates --include=*.rs | wc -l`
- **Commit-Hash:** `67c8cd3eec4e373d14286c6d49337258b0aa1d3c`
- **Exit-Code:** `0`
- **Datum:** 2026-10-15
- **Ausgabe:**
```
2
```

### Befehl 11: Treffer downcast_ref (ohne Tests)

- **Befehl:** `grep -rn "downcast_ref" crates --include=*.rs | grep -v test`
- **Commit-Hash:** `67c8cd3eec4e373d14286c6d49337258b0aa1d3c`
- **Exit-Code:** `0`
- **Datum:** 2026-10-15
- **Ausgabe:**
```
crates/contextra-store/src/lsm/observer.rs:166:                            let msg = if let Some(s) = payload.downcast_ref::<&str>() {
crates/contextra-store/src/lsm/observer.rs:168:                            } else if let Some(s) = payload.downcast_ref::<String>() {
crates/contextra-vector/src/compute_pool.rs:75:                                let panic_msg = if let Some(s) = err.downcast_ref::<&str>() {
crates/contextra-vector/src/compute_pool.rs:77:                                } else if let Some(s) = err.downcast_ref::<String>() {
crates/contextra-engine/src/collection/crud/delete.rs:91:        let maybe_lsm = (self.storage.as_ref() as &dyn std::any::Any).downcast_ref::<LsmStorage>();
```

### Befehl 12: Treffer u64::MAX (ohne Tests)

- **Befehl:** `grep -rn "u64::MAX" crates --include=*.rs | grep -v test`
- **Commit-Hash:** `67c8cd3eec4e373d14286c6d49337258b0aa1d3c`
- **Exit-Code:** `0`
- **Datum:** 2026-10-15
- **Ausgabe:**
```
crates/contextra-ports/src/clock.rs:40:            .map(|d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX))
crates/contextra-ports/src/clock.rs:47:        .unwrap_or(u64::MAX);
crates/contextra-ports/src/clock.rs:59:        .unwrap_or(u64::MAX)
crates/contextra-ports/src/graph_index.rs:30:/// 2. **Interner Systembereich**: `TxId::INTERNAL_BASE` (`u64::MAX - 1_000_000`)
crates/contextra-ports/src/embedding_sanity.rs:268:                    let val = ((state as f32) / (u64::MAX as f32)) - 0.5;
crates/contextra-simd/src/kernels/neon.rs:181:            let val = (self.next_u64() as f64) / (u64::MAX as f64);
crates/contextra-simd/src/kernels/avx2.rs:589:            let val = (self.next_u64() as f64) / (u64::MAX as f64);
crates/contextra-simd/src/kernels/avx512.rs:589:            let val = (self.next_u64() as f64) / (u64::MAX as f64);
crates/contextra-store/src/lsm/ops/read.rs:28:/// rather than `u64::MAX`. Therefore, the registered sequence number is clamped to
crates/contextra-store/src/lsm/engine.rs:165:        crate::wal::FAIL_APPEND_FOR_TX.store(u64::MAX, std::sync::atomic::Ordering::SeqCst);
crates/contextra-store/src/lsm/scan.rs:179:                        u64::MAX,
crates/contextra-store/src/lsm/scan.rs:189:                        u64::MAX,
crates/contextra-store/src/lsm/gc_floor.rs:36:        let tx_min = self.tx_buffer.min_read_snapshot().unwrap_or(u64::MAX);
crates/contextra-store/src/compaction/wiring.rs:61:        assert_eq!(wiring.engine.min_snapshot_seq_bound(), u64::MAX);
crates/contextra-store/src/compaction/retention.rs:22:        if floor_seq == u64::MAX || raw_seq > floor_seq_masked {
crates/contextra-store/src/compaction/retention.rs:54:        // Versions: 30, 20, 10, 5 for floor = u64::MAX
crates/contextra-store/src/compaction/retention.rs:62:        let retained = retain_key_versions(versions, |(_, seq)| *seq, u64::MAX);
crates/contextra-store/src/compaction/engine.rs:678:                        .unwrap_or(u64::MAX);
crates/contextra-store/src/compaction/engine.rs:779:                                    .unwrap_or(u64::MAX);
crates/contextra-store/src/compaction/adaptive.rs:285:    let mut best_tier_size = u64::MAX;
crates/contextra-store/src/compaction/adaptive.rs:333:        let mut min_fallback_size = u64::MAX;
crates/contextra-store/src/wal/flusher.rs:480:                        let written_len_u64 = u64::try_from(written_len).unwrap_or(u64::MAX);
crates/contextra-store/src/wal/encode.rs:7://! - I-4: Monotone Sequenznummern: seq_no == u64::MAX wird als Überlauf abgelehnt.
crates/contextra-store/src/wal/encode.rs:136:        if seq_no == u64::MAX {
crates/contextra-store/src/wal/io.rs:564:                    .any(|e| e.tx_id().inner() == fail_tx || fail_tx == u64::MAX)
crates/contextra-store/src/memtable.rs:137:            min_tx: AtomicU64::new(u64::MAX),
crates/contextra-store/src/memtable.rs:287:        let mut new_min_tx = u64::MAX;
crates/contextra-store/src/memtable.rs:322:            self.min_tx.store(u64::MAX, Ordering::Release);
crates/contextra-store/src/memtable.rs:704:        assert!(mt.get_at_seq(b"key1", 5, u64::MAX).is_none());
crates/contextra-store/src/memtable.rs:763:            .get_at_seq(&key, 25, u64::MAX)
crates/contextra-store/src/memtable.rs:936:        // 2. tx_range on empty memtable -> (u64::MAX, 0)
crates/contextra-store/src/memtable.rs:937:        assert_eq!(mt.tx_range(), (u64::MAX, 0));
crates/contextra-store/src/memtable.rs:942:        assert_eq!(mt.tx_range(), (u64::MAX, 0));
crates/contextra-store/src/memtable.rs:958:        assert_eq!(mt.tx_range(), (u64::MAX, 0));
crates/contextra-store/src/memtable.rs:1128:            .get_at_seq(&key, 15, u64::MAX)
crates/contextra-store/src/memtable.rs:1136:            .get_at_seq(&key, 120, u64::MAX)
crates/contextra-store/src/sstable/builder.rs:268:            min_tx_id: u64::MAX,
crates/contextra-store/src/sstable/builder.rs:270:            min_seq: u64::MAX,
crates/contextra-store/src/sstable/builder.rs:387:            .map_err(|_| ContextraError::Storage("Block size exceeds u64::MAX".into()))?;
crates/contextra-store/src/sstable/reader.rs:914:        self.get_at(key, u64::MAX, u64::MAX).await
crates/contextra-store/benches/memtable_bench.rs:34:            min_tx: AtomicU64::new(u64::MAX),
crates/contextra-crypto/src/crypto.rs:514:                if val == u64::MAX {
crates/contextra-crypto/src/crypto.rs:756:        km.nonce_counter.store(u64::MAX, Ordering::SeqCst);
crates/contextra-crypto/src/crypto.rs:760:            "Encryption MUST fail when nonce counter reaches u64::MAX"
crates/contextra-sys/src/determinism.rs:38:            .map(|d| u64::try_from(d.as_nanos()).unwrap_or(u64::MAX))
crates/contextra-sys/src/determinism.rs:51:        .unwrap_or(u64::MAX)
crates/contextra-sys/src/determinism.rs:170:            .map(|d| u64::try_from(d.as_nanos()).unwrap_or(u64::MAX))
crates/contextra-mvcc/src/tx_buffer.rs:232:            u64::try_from(config.tx_timeout.as_nanos()).unwrap_or(u64::MAX);
crates/contextra-mvcc/src/tx_buffer.rs:1248:            tx_id_raw in 0..u64::MAX,
crates/contextra-mvcc/src/snapshot.rs:77:            min_active_seqno: AtomicU64::new(u64::MAX),
crates/contextra-mvcc/src/snapshot.rs:121:        let ticks = u64::try_from(duration.as_nanos()).unwrap_or(u64::MAX);
crates/contextra-mvcc/src/snapshot.rs:142:        let ticks = u64::try_from(duration.as_nanos()).unwrap_or(u64::MAX);
crates/contextra-mvcc/src/snapshot.rs:197:    /// Returns the minimum active sequence number (`u64::MAX` if none).
crates/contextra-mvcc/src/snapshot.rs:338:        let min = active.keys().next().copied().unwrap_or(u64::MAX);
crates/contextra-mvcc/src/snapshot.rs:352:        assert_eq!(registry.min_active_seqno(), u64::MAX);
crates/contextra-mvcc/src/snapshot.rs:359:        assert_eq!(registry.min_active_seqno(), u64::MAX);
crates/contextra-mvcc/src/snapshot.rs:388:        assert_eq!(registry.min_active_seqno(), u64::MAX);
crates/contextra-mvcc/src/snapshot.rs:413:        assert_eq!(registry.min_active_seqno(), u64::MAX);
crates/contextra-mvcc/src/snapshot.rs:420:        assert_eq!(registry.min_active_seqno(), u64::MAX);
crates/contextra-mvcc/src/snapshot.rs:426:        assert_eq!(registry.min_active_seqno(), u64::MAX);
crates/contextra-mvcc/src/snapshot.rs:429:        assert_eq!(registry.min_active_seqno(), u64::MAX);
crates/contextra-mvcc/src/snapshot.rs:450:            u64::MAX,
crates/contextra-mvcc/src/snapshot.rs:451:            "min_active_seqno must return u64::MAX after all leases are dropped"
crates/contextra-mvcc/src/snapshot.rs:511:                prop_assert_eq!(registry.min_active_seqno(), u64::MAX);
crates/contextra-mvcc/src/snapshot.rs:513:                let remaining_min = leases.iter().map(|l| l.seq_no()).min().unwrap_or(u64::MAX);
crates/contextra-mvcc/src/snapshot.rs:550:                let expected_min = active_seqs.first().copied().unwrap_or(u64::MAX);
crates/contextra-mvcc/src/snapshot.rs:581:            let expected_min = ref_counts.keys().next().copied().unwrap_or(u64::MAX);
crates/contextra-mvcc/src/snapshot.rs:594:            let expected_min2 = ref_counts.keys().next().copied().unwrap_or(u64::MAX);
crates/contextra-mvcc/src/snapshot.rs:598:            prop_assert_eq!(registry.min_active_seqno(), u64::MAX);
crates/contextra-mvcc/src/seq_log.rs:127:        self.max_pin_duration_ticks = u64::try_from(duration.as_nanos()).unwrap_or(u64::MAX);
crates/contextra-mvcc/src/floor.rs:62:            .unwrap_or(u64::MAX);
crates/contextra-graph/src/community.rs:332:            let u64_id = u64::MAX - (global_idx - reverse_map.len()) as u64;
crates/contextra-graph/src/csr/graph_index.rs:143:                "AGT-GRAPH-001: Verdächtiger oder unallozierter TxId in add_entity (weder im plausiblen next_tx-Bereich noch im INTERNAL_BASE-Bereich [u64::MAX - 1_000_000]) — \
crates/contextra-graph/src/csr/graph_index.rs:176:                "AGT-GRAPH-001: Verdächtiger oder unallozierter TxId in add_edge (weder im plausiblen next_tx-Bereich noch im INTERNAL_BASE-Bereich [u64::MAX - 1_000_000]) — \
crates/contextra-graph/src/csr/graph_index.rs:660:                "AGT-GRAPH-001: Verdächtiger oder unallozierter TxId in commit (weder im plausiblen next_tx-Bereich noch im INTERNAL_BASE-Bereich [u64::MAX - 1_000_000]) — \
crates/contextra-graph/src/csr/graph_index.rs:781:                "AGT-GRAPH-001: Verdächtiger oder unallozierter TxId in remove_edge (weder im plausiblen next_tx-Bereich noch im INTERNAL_BASE-Bereich [u64::MAX - 1_000_000]) — \
crates/contextra-graph/src/cascade.rs:626:        let report = cascade_invalidate_edges_for_superseded_doc(&graph, doc_a, u64::MAX)
crates/contextra-adapt/src/decay_controller.rs:186:        let score = t.effective_score(1.0, u64::MAX / 2, &inputs);
crates/contextra-text/src/wand.rs:293:                        let raw_entries = storage.scan_prefix_at(&prefix, u64::MAX).await?;
crates/contextra-text/src/wand.rs:589:            Box::pin(async move { self.get_at_seq(key, u64::MAX).await })
crates/contextra-text/src/wand.rs:729:            Box::pin(async move { self.scan_prefix_at(prefix, u64::MAX).await })
crates/contextra-vector/src/hnsw/core_rebuild.rs:536:                    let multiplier = 1u64.checked_shl(shift).unwrap_or(u64::MAX);
crates/contextra-engine/src/collection/crud/links.rs:65:            .get_at_seq_tracked(tx, &doc_key, u64::MAX)
crates/contextra-engine/src/collection/crud/links.rs:139:                        .get_at_seq_tracked(tx, &user_key, u64::MAX)
crates/contextra-engine/src/collection/crud/links.rs:191:        if let Some(bytes) = self.storage.get_at_seq(&doc_key, u64::MAX).await? {
crates/contextra-engine/src/collection/crud/read.rs:19:    // instead of silently mapping them to u64::MAX (ID: AGT-DB-001).
crates/contextra-engine/src/transaction/compensating_actions.rs:204:        let max_u64_doc = DocId::new(u64::MAX as _);
crates/contextra-engine/src/transaction/compensating_actions.rs:205:        assert_eq!(max_u64_doc.inner(), u64::MAX);
crates/contextra-engine/src/transaction/compensating_actions.rs:207:        let overflow_raw: u128 = (u64::MAX as u128) + 1;
crates/contextra-engine/src/transaction/db_transaction_rollback.rs:280:        let overflow_raw: u128 = (u64::MAX as u128) + 100;
crates/contextra-engine/src/background_workers/config.rs:38:        let multiplier = 1u64.checked_shl(shift).unwrap_or(u64::MAX);
crates/contextra-engine/src/export.rs:144:                    contextra_types::TxId::new(u64::MAX),
crates/contextra-types/src/types/domain/ids.rs:167:    /// Maximum possible `DocId` value (`u64::MAX`).
crates/contextra-types/src/types/domain/ids.rs:168:    pub const MAX: Self = Self(u64::MAX);
crates/contextra-types/src/types/domain/ids.rs:397:    /// Exact numeric value: `u64::MAX - 1_000_000` (`18_446_744_073_708_551_615`).
crates/contextra-types/src/types/domain/ids.rs:401:    /// (`[INTERNAL_BASE, u64::MAX]`, verwaltet von `INTERNAL_BASE + atomic counter`).
crates/contextra-types/src/types/domain/ids.rs:409:    pub const INTERNAL_BASE: u64 = u64::MAX - 1_000_000;
crates/contextra-types/src/types/domain/ids.rs:480:    /// Valid system transaction range is `[INTERNAL_BASE, u64::MAX]`.
crates/contextra-types/src/types/domain/ids.rs:481:    /// Offsets must not cause integer overflow beyond `u64::MAX`, nor can wraparound occur
crates/contextra-types/src/types/domain/ids.rs:485:    /// Returns `ContextraError::Transaction` if `INTERNAL_BASE + offset` overflows `u64::MAX`.
crates/contextra-types/src/types/domain/ids.rs:492:                    "Internal TxId allocation offset {offset} overflows u64::MAX (INTERNAL_BASE={})",
crates/contextra-types/src/types/domain/ids.rs:513:const _: () = assert!(TxId::INTERNAL_BASE < u64::MAX);
crates/contextra-types/src/types/budget.rs:365:    /// atomic underflow (wrapping to `u64::MAX`) which would permanently block
crates/contextra-types/src/types/budget.rs:512:        // Try to consume more than u64::MAX (total)
crates/contextra-types/src/types/budget.rs:513:        let result = tracker.consume_memory(u64::MAX);
crates/contextra-types/src/types/budget.rs:518:        let result = tracker.consume_memory(u64::MAX - 100);
crates/contextra-cognition/src/memory_consolidation.rs:675:            representative_embedding[i] = (chunk_val as f32) / (u64::MAX as f32);
```

---

## Zusätzlich: Liste aller Dateien unter `crates/` mit `cfg(feature)`-Attributen je Crate

**Gesamtanzahl betroffener Dateien:** 130 in 14 Crates.

| Crate | Anzahl Dateien |
| --- | --- |
| `contextra-adapt` | 11 |
| `contextra-cognition` | 4 |
| `contextra-crypto` | 3 |
| `contextra-db` | 1 |
| `contextra-engine` | 8 |
| `contextra-graph` | 19 |
| `contextra-mcp` | 1 |
| `contextra-ports` | 1 |
| `contextra-router` | 21 |
| `contextra-store` | 43 |
| `contextra-testkit` | 3 |
| `contextra-text` | 5 |
| `contextra-types` | 1 |
| `contextra-vector` | 9 |

### Detaillierte Dateiliste je Crate:

#### Crate: `contextra-adapt` (Anzahl: 11)
- `crates/contextra-adapt/src/drift.rs`
- `crates/contextra-adapt/src/lib.rs`
- `crates/contextra-adapt/src/lyapunov.rs`
- `crates/contextra-adapt/src/shadow_mode.rs`
- `crates/contextra-adapt/src/wiring.rs`
- `crates/contextra-adapt/tests/bandit_sherman_morrison_long_run_stability.rs`
- `crates/contextra-adapt/tests/fcts_drift_update_ring3_only.rs`
- `crates/contextra-adapt/tests/fcts_numerics.rs`
- `crates/contextra-adapt/tests/j11-contextra-adapt_rie_greedy.rs`
- `crates/contextra-adapt/tests/rie_greedy_numerics.rs`
- `crates/contextra-adapt/tests/wiring_test.rs`

#### Crate: `contextra-cognition` (Anzahl: 4)
- `crates/contextra-cognition/src/maintenance_config.rs`
- `crates/contextra-cognition/src/maintenance_scheduler.rs`
- `crates/contextra-cognition/tests/j13-contextra-cognition_wiring_test.rs`
- `crates/contextra-cognition/tests/j13closure-contextra-cognition_wiring_test.rs`

#### Crate: `contextra-crypto` (Anzahl: 3)
- `crates/contextra-crypto/src/kv_segment/segment.rs`
- `crates/contextra-crypto/src/kv_segment/store.rs`
- `crates/contextra-crypto/tests/kv_segment_integration.rs`

#### Crate: `contextra-db` (Anzahl: 1)
- `crates/contextra-db/tests/fusion_edge_cases_test.rs`

#### Crate: `contextra-engine` (Anzahl: 8)
- `crates/contextra-engine/src/collection/mod.rs`
- `crates/contextra-engine/src/lib.rs`
- `crates/contextra-engine/src/transaction/tests.rs`
- `crates/contextra-engine/tests/drop_collection_concurrent_writer.rs`
- `crates/contextra-engine/tests/drop_collection_layer_claims.rs`
- `crates/contextra-engine/tests/drop_collection_scope.rs`
- `crates/contextra-engine/tests/invariant_tenant_isolation.rs`
- `crates/contextra-engine/tests/p02c_drop_collection_proof_durability.rs`

#### Crate: `contextra-graph` (Anzahl: 19)
- `crates/contextra-graph/src/csr/graph_read.rs`
- `crates/contextra-graph/src/csr/graph_write.rs`
- `crates/contextra-graph/src/csr/inner.rs`
- `crates/contextra-graph/src/csr/tests/graph_index_tests.rs`
- `crates/contextra-graph/src/csr/types.rs`
- `crates/contextra-graph/src/edge_reinforcement_buffer.rs`
- `crates/contextra-graph/src/lib.rs`
- `crates/contextra-graph/src/path_rag/mod.rs`
- `crates/contextra-graph/tests/apprh_determinism_permutation.rs`
- `crates/contextra-graph/tests/apprh_gate_monitor.rs`
- `crates/contextra-graph/tests/apprh_selector_shadow_to_production.rs`
- `crates/contextra-graph/tests/apprh_tests.rs`
- `crates/contextra-graph/tests/budgeted_k_path_hub_entity.rs`
- `crates/contextra-graph/tests/diffusion_shadow_gate_thresholds.rs`
- `crates/contextra-graph/tests/docid_128_path_rag.rs`
- `crates/contextra-graph/tests/j14_contextra_graph_graph_schreibpfad_csr_ko_tests.rs`
- `crates/contextra-graph/tests/j14closure-contextra-graph_tests.rs`
- `crates/contextra-graph/tests/j15-contextra-graph-graph-nebenmodule-appr-p_apprh_tests.rs`
- `crates/contextra-graph/tests/j15closure-contextra-graph_symbol_verification.rs`

#### Crate: `contextra-mcp` (Anzahl: 1)
- `crates/contextra-mcp/tests/release_build_rejects_mock.rs`

#### Crate: `contextra-ports` (Anzahl: 1)
- `crates/contextra-ports/src/erasure.rs`

#### Crate: `contextra-router` (Anzahl: 21)
- `crates/contextra-router/benches/bandit_latency_gate.rs`
- `crates/contextra-router/src/arm_registry.rs`
- `crates/contextra-router/src/lib.rs`
- `crates/contextra-router/src/profile.rs`
- `crates/contextra-router/src/router/dispatch_core.rs`
- `crates/contextra-router/src/router/lifecycle.rs`
- `crates/contextra-router/src/router/mod.rs`
- `crates/contextra-router/src/router/outcomes.rs`
- `crates/contextra-router/src/router/tests.rs`
- `crates/contextra-router/src/routing_strategy.rs`
- `crates/contextra-router/src/tests/bandit_tests.rs`
- `crates/contextra-router/src/tests/routing_tests/engine_part2.rs`
- `crates/contextra-router/src/transport.rs`
- `crates/contextra-router/tests/bandit_latency.rs`
- `crates/contextra-router/tests/fc_ts_dispatch_test.rs`
- `crates/contextra-router/tests/fc_ts_routing_determinism.rs`
- `crates/contextra-router/tests/feature_cleanup_validation.rs`
- `crates/contextra-router/tests/j10-contextra-router-router-lifecycle-outcom_test.rs`
- `crates/contextra-router/tests/j10closure-contextra-router_lifecycle_outcomes_test.rs`
- `crates/contextra-router/tests/regret_monotonicity.rs`
- `crates/contextra-router/tests/router_bandit_dispatch.rs`

#### Crate: `contextra-store` (Anzahl: 43)
- `crates/contextra-store/benches/wal_bench.rs`
- `crates/contextra-store/src/engine/mod.rs`
- `crates/contextra-store/src/lsm/engine.rs`
- `crates/contextra-store/src/lsm/purge/failpoint.rs`
- `crates/contextra-store/src/lsm/purge/orchestrator.rs`
- `crates/contextra-store/src/lsm/tests/commit_tests.rs`
- `crates/contextra-store/src/lsm/validate.rs`
- `crates/contextra-store/src/sstable/block_cache.rs`
- `crates/contextra-store/src/wal/encode.rs`
- `crates/contextra-store/src/wal/flusher.rs`
- `crates/contextra-store/src/wal/hmac.rs`
- `crates/contextra-store/src/wal/io.rs`
- `crates/contextra-store/src/wal/mod.rs`
- `crates/contextra-store/src/wal/open_heal_tests.rs`
- `crates/contextra-store/src/wal/replay.rs`
- `crates/contextra-store/src/wal/tests.rs`
- `crates/contextra-store/src/wal/tests/encode_tests.rs`
- `crates/contextra-store/src/wal/tests/flusher_tests.rs`
- `crates/contextra-store/src/wal/tests/hmac_tests.rs`
- `crates/contextra-store/src/wal/tests/io_tests.rs`
- `crates/contextra-store/tests/commit_rollback_batch_scoped.rs`
- `crates/contextra-store/tests/crash_kill_durability.rs`
- `crates/contextra-store/tests/disk_full_during_write.rs`
- `crates/contextra-store/tests/docid_128_kv_engine.rs`
- `crates/contextra-store/tests/group_commit_flush_race.rs`
- `crates/contextra-store/tests/group_commit_overlapping_batch_rollback.rs`
- `crates/contextra-store/tests/group_commit_test.rs`
- `crates/contextra-store/tests/group_commit_timeout.rs`
- `crates/contextra-store/tests/kv_crypto_shred_instant_unrecoverable.rs`
- `crates/contextra-store/tests/metrics_sink_test.rs`
- `crates/contextra-store/tests/p01a_wal_fsync_failure_truncates_page_cache.rs`
- `crates/contextra-store/tests/wal_flusher_rollback.rs`
- `crates/contextra-store/tests/wal_hmac_chain_ordering.rs`
- `crates/contextra-store/tests/wal_hmac_rollback_race.rs`
- `crates/contextra-store/tests/wal_legacy_key.rs`
- `crates/contextra-store/tests/wal_legacy_key_migration_store.rs`
- `crates/contextra-store/tests/wal_lifecycle_crash_point_matrix.rs`
- `crates/contextra-store/tests/wal_torn_write_fault_injection.rs`
- `crates/contextra-store/tests/wal_truncate_ordering.rs`
- `crates/contextra-store/tests/wal_v4_cdc_privacy_tests.rs`
- `crates/contextra-store/tests/wal_v4_enospc_and_fsync_tests.rs`
- `crates/contextra-store/tests/wal_v4_format_tests.rs`
- `crates/contextra-store/tests/wal_v4_property_tests.rs`

#### Crate: `contextra-testkit` (Anzahl: 3)
- `crates/contextra-testkit/src/erasure_harness/failpoints.rs`
- `crates/contextra-testkit/src/erasure_harness/mod.rs`
- `crates/contextra-testkit/src/lib.rs`

#### Crate: `contextra-text` (Anzahl: 5)
- `crates/contextra-text/src/inverted/index_struct.rs`
- `crates/contextra-text/src/inverted/tests/search_tests.rs`
- `crates/contextra-text/src/posting_list.rs`
- `crates/contextra-text/src/wand.rs`
- `crates/contextra-text/tests/docid_128_tombstone_resolution.rs`

#### Crate: `contextra-types` (Anzahl: 1)
- `crates/contextra-types/src/types/domain/ids.rs`

#### Crate: `contextra-vector` (Anzahl: 9)
- `crates/contextra-vector/src/hnsw/config.rs`
- `crates/contextra-vector/src/hnsw/core_search.rs`
- `crates/contextra-vector/src/hnsw/types.rs`
- `crates/contextra-vector/src/hnsw/vector_index_impl.rs`
- `crates/contextra-vector/src/partial_rebuild.rs`
- `crates/contextra-vector/src/persistence/node.rs`
- `crates/contextra-vector/tests/campaign_j23_vector_index.rs`
- `crates/contextra-vector/tests/docid_high_bits_no_collision.rs`
- `crates/contextra-vector/tests/partial_rebuild_recall_regression.rs`

---

## Abweichungsanalyse zur Spezifikation (GESAMTSPEZIFIKATION-v4)

Gemäß Arbeitsanweisung werden alle Messwerte unkorrigiert übernommen und Abweichungen explizit vermerkt:

1. **Kompilierverhalten (`cargo check` / `cargo test`):**
   - `cargo check --locked -p contextra-mcp` schlägt mit Exit-Code 101 fehl.
   - `cargo test --workspace --locked --no-run` schlägt mit Exit-Code 101 fehl.
   - *Ursache:* Auslagerung von Graph-Komponenten aus dem Workspace führt zu 40 Kompilierfehlern in `contextra-engine` (fehlende Felder `graph_index`, fehlende Methoden `relate`/`relate_bidirectional`, etc.).

2. **Hüllengrößen (`cargo tree`):**
   - `contextra-mcp` (Default): 29 Crates (Spezifikation Schätzung: 25 Crates / Ziel v0.1.0: <= 18 Crates).
   - `contextra-mcp` (--no-default-features): 29 Crates.
   - `contextra-mcp` (--all-features): 29 Crates (Spezifikation Schätzung: 28 Crates).
   - `contextra-engine` (--no-default-features): 19 Crates (Spezifikation Schätzung: 15 Crates).
   - `contextra` (Default): 27 Crates (Spezifikation Schätzung: 22 Crates).

3. **Gates & Schichtenregeln:**
   - `cargo xtask check-ring-layering --strict`: Exit-Code 1. Es wurden 5 unallowlistete Ring-Verstöße/Stale Entries gefunden (2 Ring 0 Rückwärtsabhängigkeiten zu `contextra-durable-fs`, 3 veraltete Allowlist-Einträge).
   - `cargo xtask check-dag`: Exit-Code 0. PASSED wie erwartet.

4. **Refactoring-Fortschritt & Invarianten:**
   - `contextra_core` Importe: 2 Dateien im Repository (2 Treffer in `crates/contextra-core/tests/`).
   - `downcast_ref`: 5 Treffer im Quelltext außerhalb von Tests (Ziel: 0 für LsmStorage;
     Treffer verteilen sich auf Panic-Handling in Compute-Pool, Observer-Payloads und Downcast in delete.rs).
   - `u64::MAX`: 107 Treffer im Workspace außerhalb von Testdateien (Lesezeitpunkte, Konstanten wie `DocId::MAX`, `TxId::INTERNAL_BASE`, Boundary-Checks).
