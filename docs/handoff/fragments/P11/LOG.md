# Protokoll (Fragment P11)

## Start-Check
- [V] Repository: HEAD 65e1d68a5aadbb6903e1b768a07e3fd7f048b6b3.
- [V] Toolchain: cargo 1.89.0 (c24e10642 2025-06-23).
- [V] `BUILD_LADDER.md` war nicht vorhanden.
- [V] Vorherige Fremdfehler festgestellt bei `cargo check`:
  - `contextra-graph`: 151 Fehler bezüglich ungelinkter Crates (`contextra_types`, `contextra_ports`).
  - `contextra-vector`: 8 Fehler (`remove`, `rebuild_sync` Duplikate; Zeroize trait missing; `kernel` field missing; constructor `try_new_with_rng_and_kernel` missing).
  - `contextra-crypto`: 2 Fehler (`unexpected cfg condition value: serde`).

## Inventar (grep -rniE "proof|witness|erasure|forget|license" crates/contextra-mcp/src crates/contextra-mcp/tests)

### Quellcode (`crates/contextra-mcp/src`)
- `src/lib.rs:16`: Doc-Tabelle `contextra_forget` (Delete document or drop collection).
- `src/lib.rs:27`: Modul-Deklaration `pub mod proof_key;`.
- `src/lib.rs:28`: Modul-Deklaration `pub mod proof_key_env;`.
- `src/lib.rs:52-53`: Re-export `pub use proof_key::proof_key_from_env;`.
- `src/proof_key.rs`: Komplettes Modul (Trait `ProofKeyProvider`, `EnvProofKeyProvider`, `proof_key_from_env`).
- `src/proof_key_env.rs`: Komplettes Modul (`resolve_proof_key`, `resolve_proof_key_from_env`).
- `src/server_dispatch.rs:149-160`: Tool-Definition `contextra_forget`.
- `src/server_dispatch.rs:241`: Tool-Beschreibung `contextra_delete` erwähnt "without measured deletion proof".
- `src/server_dispatch.rs:266`: Tool-Beschreibung `contextra_drop_collection` erwähnt "verified collection-wide cryptographic drop receipt".
- `src/server_dispatch.rs:324`: Dispatch-Match auf `"contextra_forget"`.
- `src/server_tools.rs:356-478`: Match-Arm `"contextra_forget"` ruft `ProofKeyProvider::get_proof_key`, `drop_collection` und erzeugt `proof` / `erasure_measured` JSON-Strukturen.
- `src/tools_crud.rs:123-167`: `handle_drop_collection` ruft `ProofKeyProvider::get_proof_key`, `db.drop_collection` und verarbeitet DeletionProof.
- `src/tools_crud.rs:211`: `handle_delete` ruft `ProofKeyProvider::get_proof_key()`.
- `src/tools_crud.rs:228-231`: `handle_delete` gibt `"proof": null`, `"erasure_measured": false` zurück.
- `src/sandbox.rs:45, 147`: Sandbox Tool-Tabelle und Metadaten für `contextra_delete` erwähnen "without measured proof".
- `src/sandbox.rs:173`: Sandbox Tool-Metadaten für `contextra_drop_collection` erwähnen "verified collection-wide cryptographic drop receipt".
- `src/tests.rs:281, 300, 316-369, 602, 640-642, 650, 683-699`: Tests für `proof`, `CONTEXTRA_PROOF_KEY`, `contextra_forget` mit Proof, `erasure_measured`.

### Tests (`crates/contextra-mcp/tests`)
- `tests/license_gate_entrypoint.rs`: Gesamte Testdatei für Lizenz-Gates und Feature-Rings.
- `tests/campaign_j27_mcp_test.rs:110, 132`: Verweise auf `contextra_forget`.
- `tests/campaign_j42_mcp_audit.rs:19`: Registrierung von `contextra_forget`.
- `tests/confirm_required_semantics.rs:67-111`: Semantiktests für `contextra_forget` und `CONTEXTRA_PROOF_KEY`.
- `tests/dispatch_direct_tests.rs:139`: Test-Assert für `contextra_forget`.
- `tests/mcp_test.rs:125`: Test-Assert für `contextra_forget`.
- `tests/tool_registry_classification.rs:139`: Test-Assert für `contextra_forget`.
- `tests/tools_crud_direct_tests.rs:231-254, 321, 384-423`: Proof-Key Preflight-Prüfungen in CRUD-Tests.

## Prüfergebnisse
- [V] Grep-Kriterium: `grep -rniE "proof|witness|erasure_report" crates/contextra-mcp/src` → 0 Treffer.
- [V] Archivierung:
  - `archive/contextra-mcp-proofs/proof_key.rs`
  - `archive/contextra-mcp-proofs/proof_key_env.rs`
  - `archive/tests-legacy/contextra-mcp/license_gate_entrypoint.rs`
- [V] `cargo check -p contextra-mcp`: Scheitert an den vorbestehenden Fremdfehlern in `contextra-graph`, `contextra-vector`, `contextra-crypto` (siehe Start-Check und OPEN_ISSUES.md).
- [V] `contextra_forget`: Als direkter Alias auf `handle_delete` konsolidiert (ein gemeinsamer Codepfad mit `contextra_delete`).
