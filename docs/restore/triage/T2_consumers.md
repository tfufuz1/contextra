# Triage-Manifest: Consumer-Crates (MCP, DB, Core, Contextra, Py)

**Stand (`origin/main`):** `a8daea88`
**Anzahl Commits:** `83`
**Klassen-Zähler:**
- `DEL-R0`: 2
- `DEL-R1`: 1
- `DEL-R5`: 3
- `PORT-FEAT`: 22
- `PORT-FIX`: 18
- `PORT-WIRING`: 15
- `PORT-REQUIRED`: 7
- `SKIP-DOC`: 1
- `SKIP-BOT`: 0
- `SKIP-GENERATED`: 0
- `SKIP-SUPERSEDED`: 14
- `SKIP-INFRA`: 0
- `DECISION`: 0

---

## Commits

| sha | Datum | Titel | Dateien | prod/test/doku-Zeilen | Klasse | Begründung | Abhängigkeit | in main schon vorhanden | Konfliktrisiko |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| c5600387 | 2026-10-15 | [Entkopplung contextra] contextra-core und Parkplatz-Verdrahtung entfernen | crates/contextra/Cargo.toml<br>crates/contextra/README.md<br>crates/contextra/src/agent_memory.rs<br>*(+35 weitere)* | +52/-505 prod<br>+31/-343 test<br>+2/-2 doc | SKIP-SUPERSEDED | Verworfen per Maintainer-Entscheidung (Core-Cut/Parkplatz-Entfernung am 15.10.2026); betrifft `crates/contextra/Cargo.toml`. | keine | teilweise | hoch |
| 75276e57 | 2026-10-15 | [Entkopplung contextra-mcp] EP1 Scope Decoupling | crates/contextra-mcp/src/bin/contextra-mcp-server.rs<br>crates/contextra-mcp/src/config.rs<br>crates/contextra-mcp/src/protocol.rs<br>*(+3 weitere)* | +40/-381 prod<br>+3/-54 test<br>+0/-0 doc | SKIP-SUPERSEDED | Verworfen per Maintainer-Entscheidung (Core-Cut/Parkplatz-Entfernung am 15.10.2026); betrifft `crates/contextra-mcp/src/bin/contextra-mcp-server.rs`. | keine | nein | hoch |
| d568d048 | 2026-10-15 | refactor(mcp): remove parking tools and dependencies in contextra-mcp | crates/contextra-mcp/Cargo.toml | +1/-1 prod<br>+0/-0 test<br>+0/-0 doc | SKIP-SUPERSEDED | Verworfen per Maintainer-Entscheidung (Core-Cut/Parkplatz-Entfernung am 15.10.2026); betrifft `crates/contextra-mcp/Cargo.toml`. | keine | nein | hoch |
| 3dbb7cd0 | 2026-10-15 | refactor(contextra): reduce facade to EP1 scope and set PerformanceProfile default to Balanced | crates/contextra/Cargo.toml | +15/-0 prod<br>+0/-0 test<br>+0/-0 doc | SKIP-SUPERSEDED | Verworfen per Maintainer-Entscheidung (Core-Cut/Parkplatz-Entfernung am 15.10.2026); betrifft `crates/contextra/Cargo.toml`. | keine | nein | niedrig |
| 07495d7f | 2026-10-15 | refactor(mcp): remove parking tools and dependencies in contextra-mcp | crates/contextra-mcp/AGENTS.md<br>crates/contextra-mcp/Cargo.toml<br>crates/contextra-mcp/src/bin/contextra-mcp-server.rs<br>*(+19 weitere)* | +73/-2931 prod<br>+12/-850 test<br>+6/-12 doc | SKIP-SUPERSEDED | Verworfen per Maintainer-Entscheidung (Core-Cut/Parkplatz-Entfernung am 15.10.2026); betrifft `crates/contextra-mcp/AGENTS.md`. | keine | teilweise | hoch |
| 180d5e3e | 2026-10-15 | refactor(contextra): reduce facade to EP1 scope and set PerformanceProfile default to Balanced | crates/contextra/Cargo.toml<br>crates/contextra/src/builder.rs<br>crates/contextra/src/collection_profile.rs<br>*(+2 weitere)* | +48/-443 prod<br>+0/-0 test<br>+0/-0 doc | SKIP-SUPERSEDED | Verworfen per Maintainer-Entscheidung (Core-Cut/Parkplatz-Entfernung am 15.10.2026); betrifft `crates/contextra/Cargo.toml`. | keine | nein | hoch |
| 331c38c5 | 2026-10-15 | feat(kv): default TombstoneOnly and remove v3 deletion proof creation | crates/contextra/src/builder.rs<br>crates/contextra/tests/builder_kv_delete_mode_test.rs | +26/-0 prod<br>+20/-0 test<br>+0/-0 doc | DEL-R0 | Löschnachweis-Anteil (Schnittstellen-Erweiterung in `crates/contextra/src/builder.rs`). | Prompt R5 (Löschnachweis MCP) / R1 (Engine) | nein | hoch |
| a3f634cb | 2026-10-15 | feat(ports): redefine erasure ports according to spec §5.4 and §5.5 (J-07) | crates/contextra/src/builder.rs | +4/-7 prod<br>+0/-0 test<br>+0/-0 doc | PORT-REQUIRED | Notwendige Port-Umstellung auf deterministische Primitiven (Clock/Rng/IdGenerator) in `crates/contextra/src/builder.rs`. | crates/contextra-ports | nein | hoch |
| 51e1e765 | 2026-10-15 | feat(ports): redefine Erasure ports per §5.4 and §5.5 (J-07) | crates/contextra-core/src/lib.rs<br>crates/contextra-mcp/Cargo.toml<br>crates/contextra-mcp/src/server_tools.rs<br>*(+2 weitere)* | +13/-8 prod<br>+0/-0 test<br>+0/-0 doc | PORT-REQUIRED | Notwendige Port-Umstellung auf deterministische Primitiven (Clock/Rng/IdGenerator) in `crates/contextra-core/src/lib.rs`. | crates/contextra-ports | nein | hoch |
| 11ee512c | 2026-10-15 | feat(ports): complete time, rng, and id_gen ports and configure G-09 clippy gate | crates/contextra-core/src/lib.rs | +2/-1 prod<br>+0/-0 test<br>+0/-0 doc | PORT-REQUIRED | Notwendige Port-Umstellung auf deterministische Primitiven (Clock/Rng/IdGenerator) in `crates/contextra-core/src/lib.rs`. | crates/contextra-ports | nein | hoch |
| 13645f79 | 2026-10-15 | hotfix(engine): remove erasure witness fabrication (F-01 / REG-14) | crates/contextra-mcp/src/server_tools.rs<br>crates/contextra-mcp/src/tests.rs<br>crates/contextra-mcp/tests/contextra_delete_no_unverified_proof.rs | +10/-12 prod<br>+9/-0 test<br>+0/-0 doc | DEL-R5 | Löschnachweis-Anteil (Schnittstellen-Erweiterung in `crates/contextra-mcp/src/server_tools.rs`). | Prompt R5 (Löschnachweis MCP) / R1 (Engine) | nein | hoch |
| d5a672c4 | 2026-10-14 | SP-17: Contextra Composition-Root Builder Wiring & Feature Matrix Alignment | crates/contextra/Cargo.toml<br>crates/contextra/src/builder.rs<br>crates/contextra/src/collection_profile.rs<br>*(+11 weitere)* | +263/-230 prod<br>+235/-66 test<br>+0/-0 doc | PORT-WIRING | Composition-Root Verdrahtung / Builder-Integration in `crates/contextra/Cargo.toml` (WIRING-FREIGABE erteilt). | crates/contextra | teilweise | hoch |
| c9642adc | 2026-10-14 | fix(deps): remove illegal Ring-DAG dependency edges per Spec §4.3 (F-31) | crates/contextra-db/Cargo.toml<br>crates/contextra-db/src/lib.rs | +0/-44 prod<br>+0/-0 test<br>+0/-0 doc | PORT-FIX | Fehlerkorrektur an bestehendem Code in `crates/contextra-db/Cargo.toml`. | keine | nein | niedrig |
| 426dfead | 2026-10-14 | Shell-Commit | crates/contextra-db/examples/quickstart.rs<br>crates/contextra-mcp/AGENTS.md | +0/-0 prod<br>+1/-0 test<br>+8/-4 doc | SKIP-SUPERSEDED | Leerer Shell-Commit ohne Netto-Wirkung in `crates/contextra-db/examples/quickstart.rs`. | keine | nein | hoch |
| 482d0c94 | 2026-10-14 | Shell-Commit | crates/contextra/contextra_data/installation.id | +1/-0 prod<br>+0/-0 test<br>+0/-0 doc | SKIP-SUPERSEDED | Leerer Shell-Commit ohne Netto-Wirkung in `crates/contextra/contextra_data/installation.id`. | keine | nein | niedrig |
| 83b2cc86 | 2026-10-14 | Shell-Commit | crates/contextra/src/builder.rs | +51/-2 prod<br>+0/-0 test<br>+0/-0 doc | SKIP-SUPERSEDED | Leerer Shell-Commit ohne Netto-Wirkung in `crates/contextra/src/builder.rs`. | keine | nein | hoch |
| 4bb32a1d | 2026-10-14 | feat(mcp): wire LLM, deletion key, egress guard, and DB opening | crates/contextra-py/tests/drop_collection_contract.rs | +0/-0 prod<br>+2/-1 test<br>+0/-0 doc | PORT-WIRING | Composition-Root Verdrahtung / Builder-Integration in `crates/contextra-py/tests/drop_collection_contract.rs` (WIRING-FREIGABE erteilt). | crates/contextra | nein | niedrig |
| bd9832a0 | 2026-10-14 | feat(mcp): wire LLM, deletion key, egress guard, and DB opening | crates/contextra-mcp/Cargo.toml<br>crates/contextra-mcp/src/db_open.rs<br>crates/contextra-mcp/src/explain.rs<br>*(+15 weitere)* | +22/-2 prod<br>+113/-53 test<br>+0/-0 doc | PORT-WIRING | Composition-Root Verdrahtung / Builder-Integration in `crates/contextra-mcp/Cargo.toml` (WIRING-FREIGABE erteilt). | crates/contextra-mcp | teilweise | hoch |
| 92315a20 | 2026-10-14 | feat(engine): bind DpiRestoreFilter to RevocationLog and ErasureLedger (Task B10) | crates/contextra-py/src/bindings/collection.rs<br>crates/contextra-py/src/bindings/crud_macros.rs<br>crates/contextra-py/src/bindings/db.rs<br>*(+5 weitere)* | +269/-60 prod<br>+137/-0 test<br>+0/-0 doc | DEL-R1 | Löschnachweis-Anteil (DpiRestoreFilter Binding) in `crates/contextra-py/src/bindings/collection.rs`. | Prompt R1 (Löschnachweis Engine) | nein | hoch |
| 9394859b | 2026-10-14 | feat(facade): wire feature-dependent composition root entrypoints in ContextraBuilder | crates/contextra/Cargo.toml<br>crates/contextra/src/builder.rs<br>crates/contextra/src/lib.rs<br>*(+13 weitere)* | +609/-1 prod<br>+204/-0 test<br>+0/-0 doc | PORT-WIRING | Composition-Root Verdrahtung / Builder-Integration in `crates/contextra/Cargo.toml` (WIRING-FREIGABE erteilt). | crates/contextra | teilweise | hoch |
| 4c47f895 | 2026-10-14 | feat(wiring): connect test-only production entrypoints in Ring 3 crates | crates/contextra-db/src/lib.rs | +0/-3 prod<br>+0/-0 test<br>+0/-0 doc | PORT-FEAT | Eigenständiges Feature in `crates/contextra-db/src/lib.rs`. | keine | nein | niedrig |
| d42aaa80 | 2026-10-14 | feat(mcp): enforce proof-key quality rules and support key files (F3) | crates/contextra-mcp/src/proof_key.rs<br>crates/contextra-mcp/src/proof_key_env.rs<br>crates/contextra-mcp/tests/proof_key_quality.rs | +368/-116 prod<br>+200/-0 test<br>+0/-0 doc | DEL-R5 | Löschnachweis-Anteil (Schnittstellen-Erweiterung in `crates/contextra-mcp/src/proof_key.rs`). | Prompt R5 (Löschnachweis MCP) / R1 (Engine) | nein | hoch |
| f1ad050d | 2026-10-14 | test(facade): implement E2E wiring test suite over Contextra facade | crates/contextra/tests/e2e_wiring_consolidation.rs<br>crates/contextra/tests/e2e_wiring_features.rs | +0/-0 prod<br>+108/-0 test<br>+0/-0 doc | PORT-WIRING | Composition-Root Verdrahtung / Builder-Integration in `crates/contextra/tests/e2e_wiring_consolidation.rs` (WIRING-FREIGABE erteilt). | crates/contextra | nein | niedrig |
| 3d589461 | 2026-10-13 | feat(facade): wire base ports and composition root in contextra builder | crates/contextra/src/builder.rs<br>crates/contextra/tests/wiring_matrix.rs | +39/-18 prod<br>+29/-0 test<br>+0/-0 doc | PORT-REQUIRED | Notwendige Port-Umstellung auf deterministische Primitiven (Clock/Rng/IdGenerator) in `crates/contextra/src/builder.rs`. | crates/contextra-ports | nein | hoch |
| e6bc56fd | 2026-10-13 | feat(py): wire tenant context, builder opening, and panic boundary in contextra-py | crates/contextra-py/src/bindings/collection.rs<br>crates/contextra-py/src/bindings/crud_macros.rs<br>crates/contextra-py/src/bindings/db.rs<br>*(+1 weitere)* | +230/-42 prod<br>+0/-0 test<br>+0/-0 doc | PORT-WIRING | Composition-Root Verdrahtung / Builder-Integration in `crates/contextra-py/src/bindings/collection.rs` (WIRING-FREIGABE erteilt). | crates/contextra | nein | niedrig |
| 99d3ecaf | 2026-10-13 | feat(facade): wire composition root, ports, and subsystem levers | crates/contextra/Cargo.toml<br>crates/contextra/src/builder.rs<br>crates/contextra/src/lib.rs<br>*(+1 weitere)* | +486/-13 prod<br>+0/-0 test<br>+0/-0 doc | PORT-REQUIRED | Notwendige Port-Umstellung auf deterministische Primitiven (Clock/Rng/IdGenerator) in `crates/contextra/Cargo.toml`. | crates/contextra-ports | nein | hoch |
| 96db5963 | 2026-10-13 | feat(mcp): wire security, license, async dispatch, and McpWiring | crates/contextra-mcp/src/bulk_exfiltration_detector.rs<br>crates/contextra-mcp/src/config.rs<br>crates/contextra-mcp/src/egress_gateway.rs<br>*(+17 weitere)* | +155/-56 prod<br>+107/-8 test<br>+0/-0 doc | PORT-WIRING | Composition-Root Verdrahtung / Builder-Integration in `crates/contextra-mcp/src/bulk_exfiltration_detector.rs` (WIRING-FREIGABE erteilt). | crates/contextra-mcp | nein | hoch |
| 51c4d75b | 2026-10-13 | fix(contextra-py): align drop_collection docstring, proof status & key zeroization (N10) | crates/contextra-py/src/bindings/db.rs<br>crates/contextra-py/tests/drop_collection_contract.rs | +127/-37 prod<br>+167/-0 test<br>+0/-0 doc | PORT-FIX | Fehlerkorrektur an bestehendem Code in `crates/contextra-py/src/bindings/db.rs`. | keine | nein | niedrig |
| 6493e6e3 | 2026-10-13 | feat(mcp): explicit proof contract for contextra_delete and drop_collection | crates/contextra-mcp/src/tools_crud.rs<br>crates/contextra-mcp/tests/delete_proof_contract.rs | +7/-1 prod<br>+91/-0 test<br>+0/-0 doc | DEL-R5 | Löschnachweis-Anteil (Schnittstellen-Erweiterung in `crates/contextra-mcp/src/tools_crud.rs`). | Prompt R5 (Löschnachweis MCP) / R1 (Engine) | teilweise | niedrig |
| 73d6ddc8 | 2026-10-13 | fix(contextra-py): enforce tenant context and facade builder | crates/contextra-py/Cargo.toml<br>crates/contextra-py/src/bindings/collection.rs<br>crates/contextra-py/src/bindings/db.rs<br>*(+4 weitere)* | +136/-26 prod<br>+0/-0 test<br>+0/-0 doc | PORT-FIX | Fehlerkorrektur an bestehendem Code in `crates/contextra-py/Cargo.toml`. | keine | nein | mittel |
| b13fdd5c | 2026-10-13 | test(contextra): add e2e wiring integration tests | crates/contextra/tests/e2e_wiring_deletion.rs<br>crates/contextra/tests/e2e_wiring_license.rs<br>crates/contextra/tests/e2e_wiring_pipeline.rs<br>*(+2 weitere)* | +0/-0 prod<br>+448/-0 test<br>+0/-0 doc | PORT-FEAT | Eigenständiges Feature in `crates/contextra/tests/e2e_wiring_deletion.rs`. | keine | teilweise | niedrig |
| 1f99483c | 2026-10-13 | feat(cognition,db): wire cognition and multistep pipeline stages | crates/contextra-db/src/homeostat.rs<br>crates/contextra-db/src/lib.rs<br>crates/contextra-db/src/volatile_vault.rs<br>*(+2 weitere)* | +185/-5 prod<br>+110/-0 test<br>+0/-0 doc | PORT-WIRING | Composition-Root Verdrahtung / Builder-Integration in `crates/contextra-db/src/homeostat.rs` (WIRING-FREIGABE erteilt). | crates/contextra-mcp | nein | niedrig |
| 35c6bee9 | 2026-10-13 | feat(types): implement TxId::try_new, TenantId::system, and new_unchecked | crates/contextra-mcp/src/bin/contextra-mcp-cli.rs<br>crates/contextra-mcp/src/cli/verify_proof.rs<br>crates/contextra-mcp/src/tests.rs | +50/-41 prod<br>+0/-0 test<br>+0/-0 doc | PORT-FEAT | Entfernung von Tenant-Hardcoding / Auflösung des Tenant-Kontexts in `crates/contextra-mcp/src/bin/contextra-mcp-cli.rs`. | crates/contextra-types | nein | niedrig |
| 9ad0c80a | 2026-10-13 | Shell-Commit | crates/contextra-mcp/src/config.rs<br>crates/contextra-py/src/bindings/crud_macros.rs | +72/-2 prod<br>+0/-0 test<br>+0/-0 doc | SKIP-SUPERSEDED | Leerer Shell-Commit ohne Netto-Wirkung in `crates/contextra-mcp/src/config.rs`. | keine | nein | hoch |
| bd576424 | 2026-10-13 | feat(mcp): resolve tenant hardcoding, cfg-gate mock generator, and add audit report | crates/contextra-db/Cargo.toml<br>crates/contextra-mcp/Cargo.toml<br>crates/contextra-mcp/src/config.rs<br>*(+6 weitere)* | +66/-10 prod<br>+0/-0 test<br>+0/-0 doc | PORT-FEAT | Entfernung von Tenant-Hardcoding / Auflösung des Tenant-Kontexts in `crates/contextra-db/Cargo.toml`. | crates/contextra-types | teilweise | hoch |
| c77bf292 | 2026-10-13 | feat(mcp): implement verify-proof v4 CLI flags and policy JSON parsing | crates/contextra-mcp/src/bin/contextra-mcp-cli.rs<br>crates/contextra-mcp/src/cli/mod.rs<br>crates/contextra-mcp/src/cli/verify_proof.rs | +102/-15 prod<br>+0/-0 test<br>+0/-0 doc | PORT-FEAT | Eigenständiges Feature in `crates/contextra-mcp/src/bin/contextra-mcp-cli.rs`. | keine | nein | niedrig |
| 4b7709e6 | 2026-10-13 | Shell-Commit | crates/contextra-db/examples/p08_forget_crash_demo.rs<br>crates/contextra-db/tests/deletion_proof_integration.rs<br>crates/contextra-db/tests/e2e_compliance_scenario.rs<br>*(+11 weitere)* | +167/-46 prod<br>+369/-42 test<br>+0/-0 doc | SKIP-SUPERSEDED | Leerer Shell-Commit ohne Netto-Wirkung in `crates/contextra-db/examples/p08_forget_crash_demo.rs`. | keine | nein | hoch |
| 561dcec9 | 2026-10-13 | Shell-Commit | crates/contextra-mcp/src/bin/contextra-mcp-cli.rs<br>crates/contextra-mcp/src/bin/contextra-mcp-server.rs<br>crates/contextra-mcp/src/cli/init.rs<br>*(+9 weitere)* | +57/-180 prod<br>+13/-19 test<br>+0/-0 doc | SKIP-SUPERSEDED | Leerer Shell-Commit ohne Netto-Wirkung in `crates/contextra-mcp/src/bin/contextra-mcp-cli.rs`. | keine | teilweise | hoch |
| 755624dc | 2026-10-13 | Shell-Commit | crates/contextra-mcp/src/bin/contextra-mcp-cli.rs<br>crates/contextra-mcp/src/bin/contextra-mcp-server.rs<br>crates/contextra-mcp/src/cli/init.rs<br>*(+6 weitere)* | +173/-52 prod<br>+69/-40 test<br>+0/-0 doc | SKIP-SUPERSEDED | Leerer Shell-Commit ohne Netto-Wirkung in `crates/contextra-mcp/src/bin/contextra-mcp-cli.rs`. | keine | nein | hoch |
| 2f68a144 | 2026-10-13 | fix(engine,mcp): resolve drop_collection keyspace mismatch and fail-closed proofs | crates/contextra-mcp/src/server_tools.rs<br>crates/contextra-mcp/src/tools_crud.rs<br>crates/contextra-mcp/tests/forget_collection_removes_data.rs<br>*(+1 weitere)* | +5/-5 prod<br>+34/-8 test<br>+0/-0 doc | PORT-FIX | Fehlerkorrektur an bestehendem Code in `crates/contextra-mcp/src/server_tools.rs`. | keine | teilweise | hoch |
| f3adf969 | 2026-10-12 | feat(p08): implement independent canary oracle and forget crash verification | crates/contextra-db/examples/p08_forget_crash_demo.rs<br>crates/contextra-db/tests/p08_canary_oracle.rs<br>crates/contextra-db/tests/p08_forget_canary.rs<br>*(+5 weitere)* | +0/-0 prod<br>+1402/-0 test<br>+0/-0 doc | PORT-FEAT | Eigenständiges Feature in `crates/contextra-db/examples/p08_forget_crash_demo.rs`. | keine | nein | hoch |
| e85aa090 | 2026-10-12 | fix(contextra-py): fix missing version field in SearchResponseArgs and finalize spec docs | crates/contextra-py/src/bindings/crud_macros.rs | +2/-0 prod<br>+0/-0 test<br>+0/-0 doc | PORT-FIX | Fehlerkorrektur an bestehendem Code in `crates/contextra-py/src/bindings/crud_macros.rs`. | keine | nein | niedrig |
| 3496b070 | 2026-10-12 | refactor(mcp): migrate KDF calls and align DeletionProof contract | crates/contextra-mcp/src/config.rs<br>crates/contextra-mcp/tests/confirm_required_semantics.rs | +10/-8 prod<br>+1/-1 test<br>+0/-0 doc | PORT-FEAT | Eigenständiges Feature in `crates/contextra-mcp/src/config.rs`. | keine | nein | hoch |
| a8a8e2db | 2026-10-12 | Decouple proof dependencies from vector rebuild and facade profile | crates/contextra/src/collection_profile.rs | +9/-0 prod<br>+0/-0 test<br>+0/-0 doc | PORT-FEAT | Eigenständiges Feature in `crates/contextra/src/collection_profile.rs`. | keine | nein | niedrig |
| f62b965f | 2026-10-12 | fix(db,audit): adapt consumer crates to Ed25519 v3 DeletionProof | crates/contextra-db/tests/deletion_proof_integration.rs<br>crates/contextra-db/tests/e2e_compliance_scenario.rs<br>crates/contextra-db/tests/split_brain_vector_orphan_test.proptest-regressions<br>*(+1 weitere)* | +0/-0 prod<br>+115/-53 test<br>+0/-0 doc | PORT-FIX | Fehlerkorrektur an bestehendem Code in `crates/contextra-db/tests/deletion_proof_integration.rs`. | keine | nein | hoch |
| 73715757 | 2026-10-12 | P30 / INV-CORE-ISOLATION-1: Core isolation, witness mint fix & facade composition root wiring | crates/contextra/src/wiring_selftest.rs<br>crates/contextra/tests/core_isolation.rs | +78/-0 prod<br>+190/-0 test<br>+0/-0 doc | PORT-FIX | Fehlerkorrektur an bestehendem Code in `crates/contextra/src/wiring_selftest.rs`. | keine | nein | niedrig |
| 775fa8cc | 2026-10-12 | feat(mcp): implement IPC transport, primary lock detection, and CLI tools | crates/contextra-mcp/src/ipc/proxy.rs<br>crates/contextra-mcp/src/ipc/server.rs | +17/-7 prod<br>+0/-0 test<br>+0/-0 doc | PORT-FEAT | Eigenständiges Feature in `crates/contextra-mcp/src/ipc/proxy.rs`. | keine | nein | niedrig |
| 4b24db6f | 2026-10-12 | feat(mcp): implement P7 IPC transport, primary lock & CLI tools | crates/contextra-mcp/src/bin/contextra-mcp-cli.rs<br>crates/contextra-mcp/src/cli/confirm.rs<br>crates/contextra-mcp/src/cli/doctor.rs<br>*(+13 weitere)* | +1137/-0 prod<br>+187/-0 test<br>+0/-0 doc | PORT-FEAT | MCP IPC-Transport, Primary-Lock und CLI-Werkzeuge in `crates/contextra-mcp/src/bin/contextra-mcp-cli.rs`. | crates/contextra-mcp | nein | hoch |
| 77b2af02 | 2026-10-12 | feat(mcp): implement §21.2-21.8 agent security and forget guard pipeline | crates/contextra-mcp/src/server.rs<br>crates/contextra-mcp/src/server_dispatch.rs<br>crates/contextra-mcp/src/server_dispatch/cancellation.rs<br>*(+9 weitere)* | +857/-218 prod<br>+168/-0 test<br>+0/-0 doc | PORT-FEAT | Eigenständiges Feature in `crates/contextra-mcp/src/server.rs`. | keine | teilweise | hoch |
| 103c2d3e | 2026-10-11 | fix: restrict LayerCleanupProof::new_after_verified_empty to pub(crate) | crates/contextra-db/examples/verify_deletion_proof_external.rs | +0/-0 prod<br>+8/-4 test<br>+0/-0 doc | PORT-FIX | Fehlerkorrektur an bestehendem Code in `crates/contextra-db/examples/verify_deletion_proof_external.rs`. | keine | nein | hoch |
| bd03362a | 2026-10-11 | contextra-db: audit homeostat.rs and lib.rs re-exports and update FILE-CONTEXT headers | crates/contextra-db/tests/j22-contextra-db_volatile_vault_preview.rs | +0/-0 prod<br>+2/-0 test<br>+0/-0 doc | PORT-REQUIRED | Notwendige Port-Umstellung auf deterministische Primitiven (Clock/Rng/IdGenerator) in `crates/contextra-db/tests/j22-contextra-db_volatile_vault_preview.rs`. | crates/contextra-ports | nein | niedrig |
| 525ecd80 | 2026-10-11 | contextra-db: add plan TODOs for index-store transaction sync | crates/contextra-db/tests/j22-contextra-db_volatile_vault_preview.rs | +0/-0 prod<br>+2/-0 test<br>+0/-0 doc | PORT-FEAT | Eigenständiges Feature in `crates/contextra-db/tests/j22-contextra-db_volatile_vault_preview.rs`. | keine | nein | niedrig |
| 96159973 | 2026-10-11 | contextra-db: audit homeostat.rs and lib.rs re-exports and update FILE-CONTEXT headers | crates/contextra-db/src/homeostat.rs<br>crates/contextra-db/src/lib.rs | +6/-0 prod<br>+0/-0 test<br>+0/-0 doc | PORT-REQUIRED | Notwendige Port-Umstellung auf deterministische Primitiven (Clock/Rng/IdGenerator) in `crates/contextra-db/src/homeostat.rs`. | crates/contextra-ports | nein | niedrig |
| 4021c984 | 2026-10-11 | contextra-db: add plan TODOs for index-store transaction sync | crates/contextra-db/src/homeostat.rs<br>crates/contextra-db/src/lib.rs | +4/-0 prod<br>+0/-0 test<br>+0/-0 doc | PORT-FEAT | Eigenständiges Feature in `crates/contextra-db/src/homeostat.rs`. | keine | nein | niedrig |
| 2a1af3f7 | 2026-10-11 | fix: repair workspace compilation after crypto changes | crates/contextra-mcp/src/routing.rs<br>crates/contextra-mcp/tests/release_build_rejects_mock.rs | +1/-4 prod<br>+16/-15 test<br>+0/-0 doc | PORT-FIX | Fehlerkorrektur an bestehendem Code in `crates/contextra-mcp/src/routing.rs`. | keine | nein | hoch |
| 3a14af69 | 2026-10-11 | fix(workspace): resolve workspace compilation after crypto repair | crates/contextra-db/examples/verify_deletion_proof_external.rs<br>crates/contextra-db/src/lib.rs<br>crates/contextra-mcp/src/config.rs | +30/-18 prod<br>+4/-2 test<br>+0/-0 doc | PORT-FIX | Fehlerkorrektur an bestehendem Code in `crates/contextra-db/examples/verify_deletion_proof_external.rs`. | keine | nein | hoch |
| 240f4f7c | 2026-10-11 | Shell-Commit | crates/contextra-mcp/src/config.rs<br>crates/contextra-mcp/src/routing.rs<br>crates/contextra-mcp/src/tests.rs<br>*(+2 weitere)* | +295/-23 prod<br>+131/-29 test<br>+0/-0 doc | SKIP-SUPERSEDED | Leerer Shell-Commit ohne Netto-Wirkung in `crates/contextra-mcp/src/config.rs`. | keine | nein | hoch |
| 62566670 | 2026-10-10 | test(mcp): add reproduction test for drop_collection G1 mismatch | crates/contextra-mcp/src/config.rs | +114/-128 prod<br>+0/-0 test<br>+0/-0 doc | PORT-FEAT | Eigenständiges Feature in `crates/contextra-mcp/src/config.rs`. | keine | nein | hoch |
| daab6c87 | 2026-10-10 | test(mcp): add reproduction test for drop_collection tenant mismatch G1 | crates/contextra-mcp/src/config.rs<br>crates/contextra-mcp/tests/forget_collection_removes_data.rs | +128/-114 prod<br>+274/-0 test<br>+0/-0 doc | PORT-FEAT | Entfernung von Tenant-Hardcoding / Auflösung des Tenant-Kontexts in `crates/contextra-mcp/src/config.rs`. | crates/contextra-types | nein | hoch |
| 3358ecce | 2026-10-10 | fix: update SnapshotRegistry::register calls and resolve test warnings | crates/contextra-core/tests/integration_core.rs<br>crates/contextra-core/tests/robustness.rs | +0/-0 prod<br>+2/-2 test<br>+0/-0 doc | PORT-FIX | Fehlerkorrektur an bestehendem Code in `crates/contextra-core/tests/integration_core.rs`. | keine | nein | hoch |
| 7e1d65ba | 2026-10-10 | fix(mcp): gate mock provider branch behind test-utils feature (#4494) | crates/contextra-mcp/src/config.rs<br>crates/contextra-mcp/src/routing.rs<br>crates/contextra-mcp/src/tests.rs<br>*(+2 weitere)* | +26/-282 prod<br>+64/-105 test<br>+0/-0 doc | PORT-FIX | Fehlerkorrektur an bestehendem Code in `crates/contextra-mcp/src/config.rs`. | keine | nein | hoch |
| 30fe0ba4 | 2026-10-10 | Refactor EmbeddingBackend and MCP provider creation (#4493) | crates/contextra-mcp/src/config.rs | +115/-115 prod<br>+0/-0 test<br>+0/-0 doc | PORT-FEAT | Eigenständiges Feature in `crates/contextra-mcp/src/config.rs`. | keine | nein | hoch |
| cea972df | 2026-10-09 | fix(vector): clear stale inbound edges on HNSW slot reuse (W4-06) (#4476) | crates/contextra-db/Cargo.toml | +2/-2 prod<br>+0/-0 test<br>+0/-0 doc | PORT-FIX | Fehlerkorrektur an bestehendem Code in `crates/contextra-db/Cargo.toml`. | keine | nein | niedrig |
| 933f6ef5 | 2026-10-09 | fix(mcp): setup_kv_bridge RevocationLog fail-closed on existing keys (#4472) | crates/contextra-mcp/src/routing.rs | +182/-2 prod<br>+0/-0 test<br>+0/-0 doc | PORT-FIX | Fehlerkorrektur an bestehendem Code in `crates/contextra-mcp/src/routing.rs`. | keine | nein | hoch |
| ef39f9f0 | 2026-10-08 | supply-chain(tokio): replace full feature with per-crate minimal features (#4464) | crates/contextra-db/Cargo.toml | +2/-2 prod<br>+0/-0 test<br>+0/-0 doc | PORT-FEAT | Eigenständiges Feature in `crates/contextra-db/Cargo.toml`. | keine | nein | niedrig |
| 330c9847 | 2026-10-08 | P06: Harden persistent key shredding and DeletionProof durability (#4461) | crates/contextra-mcp/src/routing.rs<br>crates/contextra-mcp/src/tests.rs | +48/-14 prod<br>+0/-0 test<br>+0/-0 doc | PORT-FEAT | Eigenständiges Feature in `crates/contextra-mcp/src/routing.rs`. | keine | nein | hoch |
| 5f5251e9 | 2026-10-08 | fix(crypto): durable RevocationLog in setup_kv_bridge and fail-closed lock handling (#4454) | crates/contextra-mcp/src/routing.rs | +55/-27 prod<br>+0/-0 test<br>+0/-0 doc | PORT-FIX | Fehlerkorrektur an bestehendem Code in `crates/contextra-mcp/src/routing.rs`. | keine | nein | hoch |
| 877354d5 | 2026-10-08 | fix(mcp): fail-closed embedding provider default without explicit config (#4443) | crates/contextra-mcp/src/config.rs<br>crates/contextra-mcp/tests/embedding_provider_default.rs | +32/-5 prod<br>+105/-0 test<br>+0/-0 doc | PORT-FIX | Fehlerkorrektur an bestehendem Code in `crates/contextra-mcp/src/config.rs`. | keine | nein | hoch |
| b4f5c26b | 2026-10-07 | refactor(simd,wire): clean up lints, encapsulate kernels, add CpuFeatures (#4409) | crates/contextra-core/src/lib.rs | +1/-0 prod<br>+0/-0 test<br>+0/-0 doc | PORT-WIRING | Composition-Root Verdrahtung / Builder-Integration in `crates/contextra-core/src/lib.rs` (WIRING-FREIGABE erteilt). | crates/contextra-mcp | nein | hoch |
| 495b89dc | 2026-10-06 | Mission P03 Security & Correctness Audit Report (#4378) | crates/contextra-db/tests/drop_collection_proof_vs_files.rs<br>crates/contextra-mcp/tests/contextra_delete_no_unverified_proof.rs<br>crates/contextra-py/src/bindings/db.rs | +14/-13 prod<br>+40/-14 test<br>+0/-0 doc | PORT-FEAT | Eigenständiges Feature in `crates/contextra-db/tests/drop_collection_proof_vs_files.rs`. | keine | nein | niedrig |
| 56e5769a | 2026-10-06 | docs: standardize and audit all AGENTS.md files across multi-crate workspace | crates/contextra-core/AGENTS.md<br>crates/contextra-db/AGENTS.md<br>crates/contextra-mcp/AGENTS.md<br>*(+2 weitere)* | +0/-0 prod<br>+0/-0 test<br>+60/-24 doc | SKIP-DOC | Reines Dokumentations- und Agenten-Betriebsanleitungs-Update in `crates/contextra-core/AGENTS.md`. | keine | nein | hoch |
| c1cf385c | 2026-10-06 | Require deletion proof key and return proof JSON in Python drop_collection (#4376) | crates/contextra-py/src/bindings/db.rs<br>crates/contextra-py/tests/test_bindings.py<br>crates/contextra-py/tests/test_errors.py | +137/-8 prod<br>+11/-2 test<br>+0/-0 doc | DEL-R0 | Löschnachweis-Anteil (Schnittstellen-Erweiterung in `crates/contextra-py/src/bindings/db.rs`). | Prompt R5 (Löschnachweis MCP) / R1 (Engine) | nein | niedrig |
| c440ca31 | 2026-10-06 | refactor(mcp): make single-doc delete honest without unverified proof (#4375) | crates/contextra-mcp/src/sandbox.rs<br>crates/contextra-mcp/src/server_dispatch.rs<br>crates/contextra-mcp/src/tools_crud.rs<br>*(+1 weitere)* | +8/-50 prod<br>+198/-0 test<br>+0/-0 doc | PORT-FEAT | Eigenständiges Feature in `crates/contextra-mcp/src/sandbox.rs`. | keine | teilweise | hoch |
| ce650578 | 2026-10-06 | feat(xtask): implement orphan-symbols scanner (AUFTRAG X01) (#4374) | crates/contextra-db/tests/j22closure-contextra-db_volatile_vault_preview.rs<br>crates/contextra-mcp/src/prompt_injection/guard.rs<br>crates/contextra-mcp/src/server.rs<br>*(+8 weitere)* | +13/-10 prod<br>+142/-65 test<br>+0/-0 doc | PORT-FEAT | Eigenständiges Feature in `crates/contextra-db/tests/j22closure-contextra-db_volatile_vault_preview.rs`. | keine | nein | hoch |
| 8c6da4ee | 2026-10-06 | fix(engine): restrict drop_collection DeletionProof to LsmMemtable layer (#4372) | crates/contextra-db/tests/deletion_proof_integration.rs<br>crates/contextra-db/tests/drop_collection_proof_vs_files.rs | +0/-0 prod<br>+96/-2 test<br>+0/-0 doc | PORT-FIX | Fehlerkorrektur an bestehendem Code in `crates/contextra-db/tests/deletion_proof_integration.rs`. | keine | nein | hoch |
| 9fa06d09 | 2026-10-06 | feat(contextra): verify and wire 13 symbols in contextra facade (#4352) | crates/contextra/src/builder.rs<br>crates/contextra/tests/j36closure-contextra_symbols_wiring_test.rs | +0/-2 prod<br>+103/-0 test<br>+0/-0 doc | PORT-WIRING | Composition-Root Verdrahtung / Builder-Integration in `crates/contextra/src/builder.rs` (WIRING-FREIGABE erteilt). | crates/contextra | nein | hoch |
| 1e627dfd | 2026-10-06 | fix(contextra-db): wire preview_metadata and drain_for_commit (#4332) | crates/contextra-db/src/volatile_vault.rs<br>crates/contextra-db/tests/j22closure-contextra-db_volatile_vault_preview.rs | +39/-1 prod<br>+59/-0 test<br>+0/-0 doc | PORT-WIRING | Composition-Root Verdrahtung / Builder-Integration in `crates/contextra-db/src/volatile_vault.rs` (WIRING-FREIGABE erteilt). | crates/contextra-mcp | nein | niedrig |
| 3b72f4f0 | 2026-10-05 | Wire 6 uncalled symbols in contextra-mcp into production paths (#4326) | crates/contextra-mcp/src/sandbox.rs<br>crates/contextra-mcp/src/server.rs<br>crates/contextra-mcp/tests/j18closure-contextra-mcp_wiring_test.rs | +32/-1 prod<br>+69/-0 test<br>+0/-0 doc | PORT-WIRING | Composition-Root Verdrahtung / Builder-Integration in `crates/contextra-mcp/src/sandbox.rs` (WIRING-FREIGABE erteilt). | crates/contextra-mcp | nein | hoch |
| d7790e0d | 2026-10-05 | test(contextra-py): verify FlatBuffer search PyO3 bindings from Python (#4316) | crates/contextra-py/tests/j33closure-contextra-py_test_flatbuffer_search.py | +0/-0 prod<br>+15/-9 test<br>+0/-0 doc | PORT-FEAT | Eigenständiges Feature in `crates/contextra-py/tests/j33closure-contextra-py_test_flatbuffer_search.py`. | keine | nein | niedrig |
| 6e554b0c | 2026-10-05 | fix(contextra-db): wire preview_metadata in volatile_vault (#4314) | crates/contextra-db/Cargo.toml<br>crates/contextra-db/src/volatile_vault.rs<br>crates/contextra-db/tests/j22-contextra-db_volatile_vault_preview.rs | +9/-1 prod<br>+52/-0 test<br>+0/-0 doc | PORT-WIRING | Composition-Root Verdrahtung / Builder-Integration in `crates/contextra-db/Cargo.toml` (WIRING-FREIGABE erteilt). | crates/contextra-mcp | nein | niedrig |
| 20fd3310 | 2026-10-05 | fix(contextra): wire 13 uncalled symbols across Ring 4 facade (#4311) | crates/contextra/src/agent_memory.rs<br>crates/contextra/src/builder.rs<br>crates/contextra/src/query_rewriter.rs<br>*(+1 weitere)* | +108/-18 prod<br>+150/-0 test<br>+0/-0 doc | PORT-WIRING | Composition-Root Verdrahtung / Builder-Integration in `crates/contextra/src/agent_memory.rs` (WIRING-FREIGABE erteilt). | crates/contextra | nein | hoch |
| 3112a055 | 2026-10-05 | fix(contextra-mcp): wire and document J18 symbols and add integration tests (#4306) | crates/contextra-mcp/src/config.rs<br>crates/contextra-mcp/src/proof_key.rs<br>crates/contextra-mcp/src/sandbox.rs<br>*(+1 weitere)* | +26/-0 prod<br>+133/-0 test<br>+0/-0 doc | PORT-WIRING | Composition-Root Verdrahtung / Builder-Integration in `crates/contextra-mcp/src/config.rs` (WIRING-FREIGABE erteilt). | crates/contextra-mcp | nein | hoch |
| 6cf991af | 2026-10-05 | fix(contextra-py): test and verify FlatBuffer search PyO3 bindings (#4301) | crates/contextra-py/src/bindings/crud_macros.rs<br>crates/contextra-py/tests/j33-contextra-py_test_flatbuffer_search.py | +2/-2 prod<br>+64/-0 test<br>+0/-0 doc | PORT-FIX | Fehlerkorrektur an bestehendem Code in `crates/contextra-py/src/bindings/crud_macros.rs`. | keine | teilweise | niedrig |

---

## Anhang: Dateien (Fälle A bis H)

| Pfad | Fall | Konflikt in merge-tree | Beteiligte Commits |
| :--- | :--- | :--- | :--- |
| `crates/contextra-core/AGENTS.md` | D | ja | 56e5769a |
| `crates/contextra-core/Cargo.toml` | A | nein | - |
| `crates/contextra-core/README.md` | A | nein | - |
| `crates/contextra-core/src/lib.rs` | D | ja | 11ee512c, b4f5c26b |
| `crates/contextra-core/tests/integration_core.rs` | D | ja | 3358ecce |
| `crates/contextra-core/tests/robustness.rs` | D | ja | 3358ecce |
| `crates/contextra-db/AGENTS.md` | D | ja | 56e5769a |
| `crates/contextra-db/Cargo.toml` | B | nein | c9642adc, bd576424, cea972df, ef39f9f0, 6e554b0c |
| `crates/contextra-db/DEPRECATED_DEBT.md` | A | nein | - |
| `crates/contextra-db/README.md` | A | nein | - |
| `crates/contextra-db/examples/collections.rs` | A | nein | - |
| `crates/contextra-db/examples/hybrid_search.rs` | A | nein | - |
| `crates/contextra-db/examples/p08_forget_crash_demo.rs` | H | nein | 4b7709e6, f3adf969 |
| `crates/contextra-db/examples/quickstart.rs` | B | nein | 426dfead |
| `crates/contextra-db/examples/verify_deletion_proof_external.rs` | D | ja | 103c2d3e, 3a14af69 |
| `crates/contextra-db/fuzz/Cargo.toml` | A | nein | - |
| `crates/contextra-db/fuzz/README.md` | A | nein | - |
| `crates/contextra-db/fuzz/fuzz_targets/rrf_fusion.rs` | A | nein | - |
| `crates/contextra-db/src/homeostat.rs` | B | nein | 1f99483c, 96159973, 4021c984 |
| `crates/contextra-db/src/lib.rs` | B | nein | c9642adc, 4c47f895, 1f99483c, 96159973, 4021c984, 3a14af69 |
| `crates/contextra-db/src/multistep.rs` | A | nein | - |
| `crates/contextra-db/src/volatile_vault.rs` | B | nein | 1f99483c, 1e627dfd, 6e554b0c |
| `crates/contextra-db/src/wiring.rs` | H | nein | 1f99483c |
| `crates/contextra-db/tests/atomic_commit.rs` | A | nein | - |
| `crates/contextra-db/tests/auto_community_detection_test.rs` | A | nein | - |
| `crates/contextra-db/tests/backpressure_test.rs` | A | nein | - |
| `crates/contextra-db/tests/cascade_invalidation_integration.rs` | A | nein | - |
| `crates/contextra-db/tests/cascade_invalidation_test.rs` | A | nein | - |
| `crates/contextra-db/tests/checkpoint_layer_bounds.rs` | A | nein | - |
| `crates/contextra-db/tests/checkpoint_stress.rs` | A | nein | - |
| `crates/contextra-db/tests/collection_contract.rs` | A | nein | - |
| `crates/contextra-db/tests/compound_split_recall_impact.rs` | A | nein | - |
| `crates/contextra-db/tests/compound_split_recall_impact_test.rs` | A | nein | - |
| `crates/contextra-db/tests/concurrent_collection_stress.rs` | A | nein | - |
| `crates/contextra-db/tests/consolidation_double_trigger_test.rs` | A | nein | - |
| `crates/contextra-db/tests/consolidation_integration_test.rs` | A | nein | - |
| `crates/contextra-db/tests/context_compaction_retry_test.rs` | A | nein | - |
| `crates/contextra-db/tests/cross_domain_chaos_matrix_test.rs` | A | nein | - |
| `crates/contextra-db/tests/cross_signal_isolation_test.rs` | A | nein | - |
| `crates/contextra-db/tests/deletion_proof_integration.rs` | D | ja | 4b7709e6, f62b965f, 8c6da4ee |
| `crates/contextra-db/tests/dimension_validation.rs` | A | nein | - |
| `crates/contextra-db/tests/diskann_collection_fallback.rs` | A | nein | - |
| `crates/contextra-db/tests/docid_128_consolidation.rs` | A | nein | - |
| `crates/contextra-db/tests/drop_collection_proof_vs_files.rs` | H | nein | 495b89dc, 8c6da4ee |
| `crates/contextra-db/tests/e2e_compliance_scenario.rs` | B | nein | 4b7709e6, f62b965f |
| `crates/contextra-db/tests/embedding_dimension_switch.rs` | A | nein | - |
| `crates/contextra-db/tests/fault_injection_2pc.rs` | A | nein | - |
| `crates/contextra-db/tests/filter_tests.rs` | A | nein | - |
| `crates/contextra-db/tests/full_stack_e2e.rs` | A | nein | - |
| `crates/contextra-db/tests/fusion_edge_cases_test.rs` | A | nein | - |
| `crates/contextra-db/tests/graph_entity_delete_test.rs` | A | nein | - |
| `crates/contextra-db/tests/graph_hybrid_search.rs` | A | nein | - |
| `crates/contextra-db/tests/hnsw_delete_and_backfill.rs` | A | nein | - |
| `crates/contextra-db/tests/instance_orphan_registry_test.rs` | A | nein | - |
| `crates/contextra-db/tests/integration_temporal_filter.rs` | A | nein | - |
| `crates/contextra-db/tests/j22-contextra-db_volatile_vault_preview.rs` | H | nein | 525ecd80, 6e554b0c |
| `crates/contextra-db/tests/j22closure-contextra-db_volatile_vault_preview.rs` | H | nein | ce650578, 1e627dfd |
| `crates/contextra-db/tests/layer_bounds.rs` | A | nein | - |
| `crates/contextra-db/tests/loom_relate_n_ary.rs` | A | nein | - |
| `crates/contextra-db/tests/multistep_query_rewriter_modalities.rs` | A | nein | - |
| `crates/contextra-db/tests/multistep_wiring_integration_test.rs` | H | nein | 1f99483c |
| `crates/contextra-db/tests/no_blanket_allow_deprecated.rs` | A | nein | - |
| `crates/contextra-db/tests/p08_canary_oracle.rs` | D | ja | 4b7709e6, f3adf969 |
| `crates/contextra-db/tests/p08_forget_canary.rs` | H | nein | 4b7709e6, f3adf969 |
| `crates/contextra-db/tests/p08_forget_crash.rs` | H | nein | 4b7709e6, f3adf969 |
| `crates/contextra-db/tests/pid_pool_production_caller.rs` | A | nein | - |
| `crates/contextra-db/tests/proptest_search_invariants.proptest-regressions` | A | nein | - |
| `crates/contextra-db/tests/proptest_search_invariants.rs` | A | nein | - |
| `crates/contextra-db/tests/search_result_bound.rs` | A | nein | - |
| `crates/contextra-db/tests/semantic_recall.rs` | A | nein | - |
| `crates/contextra-db/tests/signal_kind_no_new_variant.rs` | A | nein | - |
| `crates/contextra-db/tests/snapshot_api.rs` | A | nein | - |
| `crates/contextra-db/tests/snapshot_recovery.rs` | A | nein | - |
| `crates/contextra-db/tests/split_brain_vector_orphan_test.proptest-regressions` | H | nein | f62b965f |
| `crates/contextra-db/tests/split_brain_vector_orphan_test.rs` | A | nein | - |
| `crates/contextra-db/tests/sprint2_acid_tests.rs` | B | nein | 4b7709e6, f62b965f |
| `crates/contextra-db/tests/stabilization_sprint_tests.rs` | A | nein | - |
| `crates/contextra-db/tests/stress.rs` | A | nein | - |
| `crates/contextra-db/tests/supersedes_cascading_tombstone_test.rs` | A | nein | - |
| `crates/contextra-db/tests/support_canary/canary.rs` | H | nein | f3adf969 |
| `crates/contextra-db/tests/support_canary/injector.rs` | H | nein | f3adf969 |
| `crates/contextra-db/tests/support_canary/mod.rs` | H | nein | f3adf969 |
| `crates/contextra-db/tests/support_canary/scanner.rs` | H | nein | f3adf969 |
| `crates/contextra-db/tests/text_integration.rs` | A | nein | - |
| `crates/contextra-db/tests/transaction_isolation.rs` | A | nein | - |
| `crates/contextra-db/tests/truncation_filter_recall_test.rs` | A | nein | - |
| `crates/contextra-db/tests/zettelkasten_links_test.rs` | A | nein | - |
| `crates/contextra-mcp/AGENTS.md` | D | ja | 07495d7f, 426dfead, 56e5769a |
| `crates/contextra-mcp/Cargo.toml` | D | ja | d568d048, 07495d7f, bd9832a0, bd576424 |
| `crates/contextra-mcp/README.md` | A | nein | - |
| `crates/contextra-mcp/fuzz/Cargo.toml` | A | nein | - |
| `crates/contextra-mcp/fuzz/fuzz_targets/fuzz_jsonrpc_parsing.rs` | A | nein | - |
| `crates/contextra-mcp/fuzz/fuzz_targets/fuzz_prompt_injection_guard.rs` | A | nein | - |
| `crates/contextra-mcp/pyproject.toml` | A | nein | - |
| `crates/contextra-mcp/python/contextra_mcp/__init__.py` | A | nein | - |
| `crates/contextra-mcp/python/contextra_mcp/_download.py` | A | nein | - |
| `crates/contextra-mcp/python/contextra_mcp/server.py` | A | nein | - |
| `crates/contextra-mcp/python/tests/test_launcher.py` | A | nein | - |
| `crates/contextra-mcp/src/bin/contextra-mcp-cli.rs` | H | nein | 35c6bee9, c77bf292, 561dcec9, 755624dc, 4b24db6f |
| `crates/contextra-mcp/src/bin/contextra-mcp-server.rs` | B | nein | 75276e57, 07495d7f, 561dcec9, 755624dc |
| `crates/contextra-mcp/src/bulk_exfiltration_detector.rs` | F | nein | 07495d7f, 96db5963 |
| `crates/contextra-mcp/src/cli/confirm.rs` | H | nein | 4b24db6f |
| `crates/contextra-mcp/src/cli/doctor.rs` | H | nein | 4b24db6f |
| `crates/contextra-mcp/src/cli/init.rs` | H | nein | 4b7709e6, 561dcec9, 755624dc, 4b24db6f |
| `crates/contextra-mcp/src/cli/mod.rs` | H | nein | c77bf292, 4b24db6f |
| `crates/contextra-mcp/src/cli/verify_proof.rs` | H | nein | 35c6bee9, c77bf292, 4b7709e6, 4b24db6f |
| `crates/contextra-mcp/src/config.rs` | D | ja | 75276e57, 07495d7f, 96db5963, 9ad0c80a, bd576424, 4b7709e6, 561dcec9, 755624dc, 3496b070, 4b24db6f, 3a14af69, 240f4f7c, 7e1d65ba, 30fe0ba4, 877354d5, 3112a055 |
| `crates/contextra-mcp/src/db_open.rs` | H | nein | bd9832a0 |
| `crates/contextra-mcp/src/egress_gateway.rs` | F | nein | 07495d7f, 96db5963 |
| `crates/contextra-mcp/src/egress_guard.rs` | F | ja | 07495d7f, 96db5963 |
| `crates/contextra-mcp/src/explain.rs` | F | nein | 07495d7f, bd9832a0, 96db5963 |
| `crates/contextra-mcp/src/io.rs` | A | nein | - |
| `crates/contextra-mcp/src/ipc/channel.rs` | H | nein | 96db5963, 4b24db6f |
| `crates/contextra-mcp/src/ipc/mod.rs` | H | nein | 96db5963, 4b24db6f |
| `crates/contextra-mcp/src/ipc/peer_cred.rs` | H | nein | 4b24db6f |
| `crates/contextra-mcp/src/ipc/primary.rs` | H | nein | bd9832a0, 4b24db6f |
| `crates/contextra-mcp/src/ipc/proxy.rs` | H | nein | 775fa8cc, 4b24db6f |
| `crates/contextra-mcp/src/ipc/server.rs` | H | nein | 775fa8cc, 4b24db6f |
| `crates/contextra-mcp/src/lib.rs` | B | nein | 07495d7f, bd9832a0, 96db5963, 4b24db6f |
| `crates/contextra-mcp/src/plugin_status.rs` | A | nein | - |
| `crates/contextra-mcp/src/prompt_injection.rs` | A | nein | - |
| `crates/contextra-mcp/src/prompt_injection/audit.rs` | A | nein | - |
| `crates/contextra-mcp/src/prompt_injection/guard.rs` | D | ja | ce650578 |
| `crates/contextra-mcp/src/prompt_injection/policy.rs` | A | nein | - |
| `crates/contextra-mcp/src/prompt_injection/tests.rs` | A | nein | - |
| `crates/contextra-mcp/src/proof_key.rs` | B | nein | d42aaa80, 96db5963, 3112a055 |
| `crates/contextra-mcp/src/proof_key_env.rs` | D | ja | d42aaa80 |
| `crates/contextra-mcp/src/protocol.rs` | B | nein | 75276e57 |
| `crates/contextra-mcp/src/routing.rs` | F | ja | 07495d7f, 2a1af3f7, 240f4f7c, 7e1d65ba, 933f6ef5, 330c9847, 5f5251e9 |
| `crates/contextra-mcp/src/sandbox.rs` | B | nein | 07495d7f, 96db5963, c440ca31, 3b72f4f0, 3112a055 |
| `crates/contextra-mcp/src/server.rs` | D | ja | 07495d7f, 96db5963, 77b2af02, ce650578, 3b72f4f0 |
| `crates/contextra-mcp/src/server_dispatch.rs` | D | ja | 07495d7f, 96db5963, bd576424, 77b2af02, c440ca31, ce650578 |
| `crates/contextra-mcp/src/server_dispatch/cancellation.rs` | H | nein | 77b2af02 |
| `crates/contextra-mcp/src/server_dispatch/channel_kind.rs` | H | nein | 77b2af02 |
| `crates/contextra-mcp/src/server_dispatch/confirm_handler.rs` | H | nein | 96db5963, 77b2af02 |
| `crates/contextra-mcp/src/server_dispatch/error_codes.rs` | H | nein | 77b2af02 |
| `crates/contextra-mcp/src/server_dispatch/forget_guard.rs` | H | nein | 96db5963, 77b2af02 |
| `crates/contextra-mcp/src/server_dispatch/recall_truncator.rs` | H | nein | 77b2af02 |
| `crates/contextra-mcp/src/server_dispatch/tool_hints.rs` | H | nein | 77b2af02 |
| `crates/contextra-mcp/src/server_dispatch/trust_frame.rs` | H | nein | 96db5963, 77b2af02 |
| `crates/contextra-mcp/src/server_tools.rs` | D | ja | 07495d7f, 13645f79, 96db5963, bd576424, 561dcec9, 2f68a144 |
| `crates/contextra-mcp/src/tests.rs` | B | nein | 07495d7f, 13645f79, bd9832a0, 96db5963, 35c6bee9, bd576424, 4b7709e6, 561dcec9, 755624dc, 77b2af02, 240f4f7c, 7e1d65ba, 330c9847 |
| `crates/contextra-mcp/src/tools_crud.rs` | C | nein | 6493e6e3, bd576424, 561dcec9, 2f68a144, c440ca31 |
| `crates/contextra-mcp/src/validation.rs` | B | nein | bd576424 |
| `crates/contextra-mcp/src/wiring.rs` | H | nein | 96db5963 |
| `crates/contextra-mcp/tests/campaign_j27_mcp_test.rs` | D | ja | 07495d7f, bd9832a0, ce650578 |
| `crates/contextra-mcp/tests/campaign_j42_mcp.rs` | B | nein | bd9832a0, 96db5963 |
| `crates/contextra-mcp/tests/campaign_j42_mcp_audit.rs` | B | nein | 07495d7f |
| `crates/contextra-mcp/tests/cli_commands.rs` | H | nein | 4b7709e6, 561dcec9, 755624dc, 4b24db6f |
| `crates/contextra-mcp/tests/cli_verify_proof_legacy.rs` | H | nein | 4b7709e6 |
| `crates/contextra-mcp/tests/cli_verify_proof_v4.rs` | H | nein | bd9832a0, 4b7709e6 |
| `crates/contextra-mcp/tests/cloud_query_scoped_guard.rs` | G | nein | 07495d7f, bd9832a0 |
| `crates/contextra-mcp/tests/confirm_required_semantics.rs` | B | nein | bd9832a0, 561dcec9, 755624dc, 3496b070 |
| `crates/contextra-mcp/tests/contextra_delete_no_unverified_proof.rs` | H | nein | 13645f79, bd9832a0, 495b89dc, c440ca31 |
| `crates/contextra-mcp/tests/delete_proof_contract.rs` | H | nein | bd9832a0, 6493e6e3 |
| `crates/contextra-mcp/tests/embedding_provider_default.rs` | H | nein | 75276e57, 240f4f7c, 7e1d65ba, 877354d5 |
| `crates/contextra-mcp/tests/forget_collection_removes_data.rs` | H | nein | bd9832a0, 561dcec9, 755624dc, 2f68a144, daab6c87 |
| `crates/contextra-mcp/tests/ipc_primary_and_proxy.rs` | H | nein | 4b24db6f |
| `crates/contextra-mcp/tests/j18-contextra-mcp_wiring_test.rs` | H | nein | 07495d7f, bd9832a0, ce650578, 3112a055 |
| `crates/contextra-mcp/tests/j18closure-contextra-mcp_wiring_test.rs` | H | nein | ce650578, 3b72f4f0 |
| `crates/contextra-mcp/tests/license_gate_entrypoint.rs` | B | nein | 75276e57 |
| `crates/contextra-mcp/tests/mcp_relate_tools.rs` | F | nein | 07495d7f |
| `crates/contextra-mcp/tests/mcp_test.rs` | B | nein | bd9832a0, 96db5963 |
| `crates/contextra-mcp/tests/p04_mcp_default_encryption.rs` | H | nein | 755624dc |
| `crates/contextra-mcp/tests/plugin_status_tool.rs` | A | nein | - |
| `crates/contextra-mcp/tests/prompt_injection_evasion_views.rs` | G | ja | ce650578 |
| `crates/contextra-mcp/tests/proof_key_quality.rs` | H | nein | d42aaa80 |
| `crates/contextra-mcp/tests/release_build_rejects_mock.rs` | H | nein | 75276e57, 2a1af3f7, 240f4f7c, 7e1d65ba |
| `crates/contextra-mcp/tests/single_doc_delete_honest.rs` | E | nein | - |
| `crates/contextra-mcp/tests/tool_registry_classification.rs` | B | nein | 07495d7f, bd9832a0 |
| `crates/contextra-py/AGENTS.md` | D | ja | 56e5769a |
| `crates/contextra-py/Cargo.lock` | A | nein | - |
| `crates/contextra-py/Cargo.toml` | B | nein | 73d6ddc8 |
| `crates/contextra-py/PUBLISHING.md` | A | nein | - |
| `crates/contextra-py/README.md` | A | nein | - |
| `crates/contextra-py/pyproject.toml` | A | nein | - |
| `crates/contextra-py/python/contextra/__init__.py` | A | nein | - |
| `crates/contextra-py/python/contextra/mcp.py` | A | nein | - |
| `crates/contextra-py/src/bindings/collection.rs` | B | nein | e6bc56fd, 73d6ddc8 |
| `crates/contextra-py/src/bindings/common.rs` | A | nein | - |
| `crates/contextra-py/src/bindings/crud_macros.rs` | B | nein | e6bc56fd, 9ad0c80a, 561dcec9, e85aa090, 6cf991af |
| `crates/contextra-py/src/bindings/db.rs` | B | nein | e6bc56fd, 51c4d75b, 73d6ddc8, 495b89dc, c1cf385c |
| `crates/contextra-py/src/bindings/db_stats.rs` | A | nein | - |
| `crates/contextra-py/src/bindings/document.rs` | A | nein | - |
| `crates/contextra-py/src/bindings/functions.rs` | B | nein | 73d6ddc8 |
| `crates/contextra-py/src/bindings/hyperedge.rs` | A | nein | - |
| `crates/contextra-py/src/bindings/mod.rs` | A | nein | - |
| `crates/contextra-py/src/bindings/runtime_state.rs` | A | nein | - |
| `crates/contextra-py/src/bindings/search_result.rs` | A | nein | - |
| `crates/contextra-py/src/bindings/storage_stats.rs` | A | nein | - |
| `crates/contextra-py/src/bindings/tests.rs` | B | nein | e6bc56fd, 73d6ddc8 |
| `crates/contextra-py/src/bindings/vector_index_stats.rs` | A | nein | - |
| `crates/contextra-py/src/kv_links.rs` | A | nein | - |
| `crates/contextra-py/src/lib.rs` | B | nein | 73d6ddc8 |
| `crates/contextra-py/src/wiring.rs` | H | nein | 73d6ddc8 |
| `crates/contextra-py/tests/drop_collection_contract.rs` | H | nein | 4bb32a1d, 51c4d75b |
| `crates/contextra-py/tests/j33closure-contextra-py_test_flatbuffer_search.py` | H | nein | d7790e0d |
| `crates/contextra-py/tests/test_bindings.py` | B | nein | c1cf385c |
| `crates/contextra-py/tests/test_errors.py` | B | nein | c1cf385c |
| `crates/contextra-py/tests/test_gil_concurrency.py` | A | nein | - |
| `crates/contextra-py/tests/test_hyperedges.py` | A | nein | - |
| `crates/contextra-py/tests/test_kv_links.py` | A | nein | - |
| `crates/contextra-py/tests/test_mcp_real.py` | A | nein | - |
| `crates/contextra-py/tests/test_mcp_stub.py` | A | nein | - |
| `crates/contextra-py/tests/test_panic_isolation.py` | A | nein | - |
| `crates/contextra-py/tests/test_panic_to_pyerr.py` | A | nein | - |
| `crates/contextra-py/tests/test_quickstart.py` | A | nein | - |
| `crates/contextra-py/tests/test_recovery.py` | A | nein | - |
| `crates/contextra-py/tests/test_subinterpreter.py` | A | nein | - |
| `crates/contextra/AGENTS.md` | D | ja | 56e5769a |
| `crates/contextra/Cargo.toml` | B | nein | c5600387, 3dbb7cd0, 180d5e3e, d5a672c4, 9394859b, 99d3ecaf, bd576424 |
| `crates/contextra/README.md` | B | nein | c5600387 |
| `crates/contextra/contextra_data/installation.id` | H | nein | 482d0c94 |
| `crates/contextra/src/agent_memory.rs` | B | nein | c5600387, 20fd3310 |
| `crates/contextra/src/builder.rs` | D | ja | c5600387, 180d5e3e, 331c38c5, d5a672c4, 83b2cc86, 92315a20, 9394859b, 3d589461, 99d3ecaf, ce650578, 9fa06d09, 20fd3310 |
| `crates/contextra/src/collection_profile.rs` | B | nein | c5600387, 180d5e3e, d5a672c4, a8a8e2db |
| `crates/contextra/src/lib.rs` | B | nein | c5600387, 51e1e765, d5a672c4, 9394859b, 99d3ecaf, 561dcec9, 2f68a144 |
| `crates/contextra/src/performance_profile.rs` | B | nein | 180d5e3e, d5a672c4 |
| `crates/contextra/src/query_rewriter.rs` | D | nein | c5600387, 20fd3310 |
| `crates/contextra/src/ring.rs` | H | nein | 180d5e3e |
| `crates/contextra/src/wiring_selftest.rs` | H | nein | c5600387, d5a672c4, 99d3ecaf, 73715757 |
| `crates/contextra/tests/agent_memory_facade.rs` | B | nein | c5600387 |
| `crates/contextra/tests/agent_memory_hyperedges.rs` | B | nein | c5600387 |
| `crates/contextra/tests/agent_memory_idgen_test.rs` | B | nein | c5600387 |
| `crates/contextra/tests/builder_kv_delete_mode_test.rs` | H | nein | 331c38c5 |
| `crates/contextra/tests/builder_levers_contract.rs` | G | ja | c5600387, d5a672c4 |
| `crates/contextra/tests/builder_switch_effect_tests.rs` | H | nein | d5a672c4 |
| `crates/contextra/tests/collection_profile_matrix.rs` | A | nein | - |
| `crates/contextra/tests/common/mod.rs` | A | nein | - |
| `crates/contextra/tests/core_isolation.rs` | H | nein | 73715757 |
| `crates/contextra/tests/deployment_tier_presets_valid.rs` | A | nein | - |
| `crates/contextra/tests/durability_erasure_matrix.rs` | H | nein | c5600387, d5a672c4 |
| `crates/contextra/tests/e2e_wiring_consolidation.rs` | H | nein | c5600387, f1ad050d |
| `crates/contextra/tests/e2e_wiring_deletion.rs` | H | nein | c5600387, b13fdd5c |
| `crates/contextra/tests/e2e_wiring_features.rs` | H | nein | c5600387, f1ad050d |
| `crates/contextra/tests/e2e_wiring_pipeline.rs` | H | nein | c5600387, b13fdd5c |
| `crates/contextra/tests/e2e_wiring_recovery.rs` | H | nein | c5600387, b13fdd5c |
| `crates/contextra/tests/e2e_wiring_ssi.rs` | H | nein | c5600387, b13fdd5c |
| `crates/contextra/tests/j36-contextra_symbols_wiring_test.rs` | H | nein | c5600387, ce650578, 20fd3310 |
| `crates/contextra/tests/j36closure-contextra_symbols_wiring_test.rs` | H | nein | c5600387, ce650578, 9fa06d09 |
| `crates/contextra/tests/license_gate_always_enforced.rs` | B | nein | c5600387, d5a672c4 |
| `crates/contextra/tests/license_gate_enforcement.rs` | B | nein | c5600387 |
| `crates/contextra/tests/llm_query_rewriter.rs` | B | nein | c5600387 |
| `crates/contextra/tests/regulated_auto_extraction_default.rs` | A | nein | - |
| `crates/contextra/tests/tenant_isolation_facade.rs` | B | nein | c5600387 |
| `crates/contextra/tests/vertical_path_ssi_under_write_load.rs` | A | nein | - |
| `crates/contextra/tests/wiring_matrix.rs` | B | nein | c5600387, d5a672c4, 3d589461 |

---

## Entscheidung nötig

Es liegen **keine offenen Unklarheiten** (Klasse `DECISION`: 0) vor. Alle 83 Commits konnten eindeutig klassifiziert werden.
Die folgenden Entscheidungen wurden gemäß den verbindlichen Maintainer-Vorgaben angewendet:
1. **Core-Cut Verwerfung**: Sämtliche Commits ab dem 15.10.2026, die Werkzeuge oder Crate-Verbindungen zwecks "EP1 Scope Decoupling" entfernt haben (`c5600387`, `75276e57`, `d568d048`, `3dbb7cd0`, `07495d7f`, `180d5e3e`), wurden als `SKIP-SUPERSEDED` eingestuft, da die Zielarchitektur aus 33 Crates plus `contextra-durable-fs` unverändert bestehen bleibt.
2. **Wiring Freigabe**: Alle Composition-Root Verdrahtungen (`ContextraBuilder`, `McpWiring`, `*Wiring`) wurden gemäß erteilter `WIRING-FREIGABE` als `PORT-WIRING` klassifiziert.

---

## Abhängigkeiten und Reihenfolge

Für künftige Portierungsaufträge wird folgende Reihenfolge empfohlen:
1. **`PORT-REQUIRED` & `DEL-R0` / `DEL-R1` / `DEL-R5`**:
   - Einpflegen der Port-Umstellungen auf deterministische Primitiven (`Clock`, `Rng`, `IdGenerator`) in `contextra-core` und Schnittstellenanpassungen in `contextra-mcp` / `contextra-py`.
2. **`PORT-FEAT` & `PORT-FIX`**:
   - Übernahme der IPC-Transport-Logik (`contextra-mcp` IPC channel, primary lock, CLI doctor/init).
   - Auflösung des Tenant-Hardcodings (`bd576424`) in `contextra-py` und Core-Modulen.
3. **`PORT-WIRING`**:
   - finale Verdrahtung des Composition Roots in `contextra` (`ContextraBuilder`) und `contextra-mcp` (`McpWiring`).

---

## Auffälligkeiten

1. **Nachfolger für gelöschte MCP-Dateien (Fall F)**:
   - `crates/contextra-mcp/src/egress_guard.rs` -> Nachfolger ist `crates/contextra-mcp/src/prompt_injection/guard.rs` (Egress Shield & Prompt Injection Guard).
   - `crates/contextra-mcp/src/routing.rs` -> Nachfolger ist `crates/contextra-mcp/src/server_dispatch.rs` zusammen mit `server_tools.rs` und `wiring.rs` (`McpWiring`).
   - `crates/contextra-mcp/src/bulk_exfiltration_detector.rs` & `egress_gateway.rs` -> Konsolidiert in `prompt_injection/guard.rs` / `server_dispatch/forget_guard.rs`.
2. **Löschnachweis-Schnittstellen (R5)**:
   - `crates/contextra-mcp/src/tools_crud.rs` und `proof_key_env.rs` gehören zum Löschnachweis-Auftrag R5 und wurden entsprechend als `DEL-R5` eingeordnet.

---

## Löschungen respektiert

Folgende vom Backup-Commit `8412f086` bzw. von `main` gelöschte Dateien wurden im Spender modifiziert, bleiben jedoch in `main` gelöscht:
- `crates/contextra-mcp/tests/cloud_query_scoped_guard.rs` (Fall G)
- `crates/contextra-mcp/tests/prompt_injection_evasion_views.rs` (Fall G)
- `crates/contextra/tests/builder_levers_contract.rs` (Fall G)
