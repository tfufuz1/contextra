# Übergabebericht `ENG-2` · Engine-Hülle, Wiring, Transaktionen, Worker, Adapter-Übernahme, Entkopplung

## 1. Auftrag & Zusammenfassung
- **Crate:** `crates/contextra-engine` (Ring 3, Klasse C)
- **Auftrag:** Wiring, Transaktions-/Kompensationsschicht, Worker-Entkopplung, Adapter-Übernahme aus Klasse-A-Bausteinen und vollständige Entkopplung der Löschdomäne gemäß M7.

## 2. Soll/Ist-Vergleich

| Soll-Vorgabe | Status | Evidenz & Details |
|---|---|---|
| **Adapter-Übernahme (E-05)** | Erfüllt | `crates/contextra-engine/src/adapters/` angelegt mit `text.rs` (`Bm25Scorer`), `privacy.rs`, `ollama.rs` und `mod.rs`. In `src/lib.rs` als `pub mod adapters;` exponiert. |
| **Entkopplung Löschdomäne (M7)** | Erfüllt | `contextra_impl/erasure.rs` (ca. 619 Zeilen) **verbatim** nach `docs/handoff/ENG-2/adapter_src/erasure.rs` gesichert. In der Engine durch neutrale `Purger`- und `LayerScanner`-Aufrufe ersetzt. `ProofKeyProvider` und `DeletionProof` aus `src/lib.rs` und `collections.rs` entfernt. `erasure_leak_prober.py -c contextra-engine` meldet **0 kernel violations**. |
| **Transaktionen & Kompensation** | Erfüllt | WAL-first 2PC (`CommitIntent::Pending` vor LSM-Commit und Index-Commits), lock order konform, `CommitLedger`-Rollback ist idempotent (`test_commit_ledger_idempotent_rollback`). |
| **Hintergrund-Worker** | Erfüllt | Synchrone `step()`-Primitiven (`reap_orphans_step`) extrahiert; deterministische Zeitmessung via `Clock`-Port. |
| **Export / Import** | Erfüllt | Defensive Validierung der Schema-Version, Begrenzung maximaler Entitäten (`MAX_IMPORT_COLLECTIONS = 10_000`). |
| **Scope Guard** | Erfüllt | Keine Dateien außerhalb von `OWNS` modifiziert (`ENG-1`-Dateien unberührt gelassen). `Cargo.lock` unberührt. |
| **Dokumentation & Hülle** | Erfüllt | `SPEC.md` mit Invarianten-Matrix erstellt, `README.md` mit ASCII-Wiring aktualisiert. |

## 3. Offene Punkte / Geschützte Dateien
- In `capabilities.toml` sollte `contextra-infer-onnx` in `may_depend_on` für `contextra-engine` nachgetragen werden, sobald Gate G1 formal harmonisiert wird (aktuell als optionales Feature hinterlegt).
- Die gesicherte Löschlogik in `docs/handoff/ENG-2/adapter_src/erasure.rs` wird planmäßig von Prompt `ERASE-ADAPT-1` / `contextra-erasure-adapters` übernommen.
