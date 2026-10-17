# Triage-Manifest T4: Restliche Crates

**Stand origin/main:** `a8daea88a5f80ec6995d9e1d5da160516f93d4e2`
**Anzahl Commits im Scope:** `119`

### Zähler je Klasse

| Klasse | Anzahl | Beschreibung |
| :--- | :--- | :--- |
| `DEL-R5` | 9 | Löschnachweis-Anteil im Testkit / Oracle (abgedeckt durch R5/R10) |
| `PORT-FEAT` | 25 | Eigenständige funktionale Erweiterung |
| `PORT-FIX` | 38 | Fehlerkorrektur im Crate-Scope |
| `PORT-REQUIRED` | 2 | Erforderlich für Unterbau-Kompilierung (Clock/Rng/IdGen/Erasure Ports) |
| `PORT-WIRING` | 13 | Composition Root Verdrahtung / Integration |
| `SKIP-BOT` | 5 | Automatisierter Bot-/Clean-Up-Commit |
| `SKIP-DOC` | 8 | Reine Dokumentationsänderung |
| `SKIP-GENERATED` | 15 | Generierter Code aus FlatBuffers-Schema (`flatc`) |
| `SKIP-SUPERSEDED` | 4 | Überholt durch spätere Iteration im Spenderbereich |

## Commits

| sha | Datum | Titel | Dateien | prod/test/doku-Zeilen | Klasse | Begründung | Abhängigkeit | in main schon vorhanden | Konfliktrisiko |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| 00ccf8a4 | 2026-10-15 | [Entkopplung contextra-simd] Decouple contextra-simd from contextra-core | crates/contextra-simd/Cargo.toml, crates/contextra-simd/src/dispatch.rs (+2) | +8/-8 \| +0/-0 \| +0/-0 | `PORT-FIX` | Entkoppelt contextra-simd von contextra-core. | contextra-simd | nein | hoch |
| d5e55fe0 | 2026-10-15 | feat(testkit): erasure_harness module for deletion guarantees | crates/contextra-testkit/src/erasure_harness/reference_store.rs | +67/-99 \| +0/-0 \| +0/-0 | `DEL-R5` | Löschnachweis-Anteil für Testkit/Oracle (abgedeckt durch Auftrag R5/R10). | crates/contextra-testkit | nein | niedrig |
| 885145d8 | 2026-10-15 | feat(testkit): implement shared erasure_harness module (REG-15, §11.2) | crates/contextra-testkit/src/dst/rng.rs, crates/contextra-testkit/src/manual_clock.rs | +20/-31 \| +0/-0 \| +0/-0 | `SKIP-SUPERSEDED` | Ersetzt durch d5e55fe0 (erasure_harness Modul). | d5e55fe0 | nein | hoch |
| a3f634cb | 2026-10-15 | feat(ports): redefine erasure ports according to spec §5.4 and §5.5 (J-07) | crates/contextra-adapt/src/bandit.rs, crates/contextra-cognition/src/context_compaction/compactor.rs (+1) | +12/-8 \| +0/-0 \| +0/-0 | `PORT-REQUIRED` | Definiert Erasure Ports gemäß Spec §5.4 und §5.5 (J-07) in contextra-ports. | contextra-ports | nein | niedrig |
| 9b33317a | 2026-10-15 | testkit: Gemeinsames Test-Harness fuer Loeschgarantien | crates/contextra-testkit/Cargo.toml, crates/contextra-testkit/src/dst/rng.rs (+8) | +636/-23 \| +109/-0 \| +0/-0 | `DEL-R5` | Löschnachweis-Anteil für Testkit/Oracle (abgedeckt durch Auftrag R5/R10). | crates/contextra-testkit | nein | hoch |
| 51e1e765 | 2026-10-15 | feat(ports): redefine Erasure ports per §5.4 and §5.5 (J-07) | crates/contextra-adapt/src/bandit.rs, crates/contextra-cognition/src/context_compaction/compactor.rs (+2) | +17/-12 \| +0/-0 \| +0/-0 | `SKIP-SUPERSEDED` | Vorgänger-Iteration von a3f634cb/d5e55fe0 (Erasure-Ports). | a3f634cb | nein | hoch |
| 11ee512c | 2026-10-15 | feat(ports): complete time, rng, and id_gen ports and configure G-09 clippy gate | crates/contextra-testkit/src/dst/rng.rs, crates/contextra-testkit/src/lib.rs (+2) | +85/-8 \| +0/-0 \| +0/-0 | `PORT-REQUIRED` | Stellt contextra-ports und testkit auf Clock, Rng, IdGenerator um; erforderlich für Unterbau-Kompilierung. | contextra-ports | nein | hoch |
| bbb605cd | 2026-10-14 | feat(testkit): update test infrastructure for deletion testing (SP-08) | crates/contextra-wire/src/contextra_generated.rs | +51/-51 \| +0/-0 \| +0/-0 | `SKIP-GENERATED` | Generierter Code aus FlatBuffers-Schema (`flatc`), wird neu generiert. | - | nein | niedrig |
| 5776b652 | 2026-10-14 | feat(testkit): update test infrastructure for deletion testing (SP-08) | crates/contextra-testkit/src/dst/p07_runner.rs, crates/contextra-testkit/src/dst/p07_sut.rs (+2) | +35/-18 \| +0/-0 \| +0/-0 | `SKIP-SUPERSEDED` | Teil einer mehrfachen Iterationsreihe, durch nachfolgenden Commit bbb605cd/d5e55fe0 ersetzt. | bbb605cd | nein | niedrig |
| a2bd6341 | 2026-10-14 | feat(testkit): update test infrastructure for deletion testing (SP-08) | crates/contextra-testkit/Cargo.toml, crates/contextra-testkit/src/dst/mod.rs (+3) | +63/-1 \| +0/-0 \| +0/-0 | `SKIP-SUPERSEDED` | Teil einer mehrfachen Iterationsreihe, durch nachfolgenden Commit bbb605cd/d5e55fe0 ersetzt. | bbb605cd | nein | niedrig |
| 426dfead | 2026-10-14 | Shell-Commit | crates/contextra-adapt/AGENTS.md, crates/contextra-cognition/AGENTS.md (+1) | +0/-0 \| +0/-0 \| +25/-13 | `SKIP-BOT` | Automatisierter Bot-/Clean-Up-Commit ohne Relevanz für Endprodukt. | - | nein | hoch |
| 83b2cc86 | 2026-10-14 | Shell-Commit | crates/contextra-wire/src/contextra_generated.rs | +51/-51 \| +0/-0 \| +0/-0 | `SKIP-GENERATED` | Generierter Code aus FlatBuffers-Schema (`flatc`), wird neu generiert. | - | nein | niedrig |
| 1a59bf54 | 2026-10-14 | docs(testkit): update audit verification note in INVARIANT_MATRIX.md | crates/contextra-testkit/INVARIANT_MATRIX.md | +0/-0 \| +0/-0 \| +1/-0 | `SKIP-DOC` | Reine Dokumentationsänderung ohne Quelltextauswirkung. | - | nein | niedrig |
| a778c647 | 2026-10-14 | feat(testkit): harden deletion oracle raw_sweep and add per-layer canaries | crates/contextra-testkit/src/deletion_oracle/marker.rs, crates/contextra-testkit/src/deletion_oracle/mod.rs (+2) | +203/-20 \| +0/-0 \| +0/-0 | `DEL-R5` | Löschnachweis-Anteil für Testkit/Oracle (abgedeckt durch Auftrag R5/R10). | crates/contextra-testkit | nein | niedrig |
| 4c47f895 | 2026-10-14 | feat(wiring): connect test-only production entrypoints in Ring 3 crates | crates/contextra-agent/src/wiring.rs, crates/contextra-agent/tests/wiring_test.rs (+3) | +48/-22 \| +27/-0 \| +0/-0 | `PORT-WIRING` | Verbindet Test-Only Entrypoints in Ring 3 Crates. | contextra-testkit | nein | hoch |
| cc6027e4 | 2026-10-14 | feat(audit-export): add audit scope limits, dual timestamps, and updated norm references | crates/contextra-audit-export/src/bsi_mapping.rs, crates/contextra-audit-export/src/markdown_template.rs (+1) | +154/-8 \| +217/-0 \| +0/-0 | `PORT-FEAT` | Audit-Export: Scope Limits, Dual Timestamps und Norm-Referenzen. | contextra-audit-export | nein | hoch |
| 51f55cb6 | 2026-10-14 | feat(sys,wire,simd): replace mock wiring with functional dispatch and fix fbs drift | crates/contextra-simd/src/kernels/scalar.rs, crates/contextra-simd/src/lib.rs (+3) | +1378/-1517 \| +0/-0 \| +0/-0 | `PORT-FEAT` | Funktionaler Dispatch für sys/wire/simd und Fix von fbs-Drift. | contextra-sys | nein | hoch |
| a5803c00 | 2026-10-13 | fix(testkit): wire ManualClock monotonicity, FaultVfs faults & WAL crash harness | crates/contextra-testkit/Cargo.toml, crates/contextra-testkit/INVARIANT_MATRIX.md (+10) | +290/-21 \| +110/-0 \| +49/-0 | `DEL-R5` | Löschnachweis-Anteil für Testkit/Oracle (abgedeckt durch Auftrag R5/R10). | crates/contextra-testkit | nein | hoch |
| 3dc67417 | 2026-10-13 | fix(governance): align capabilities manifest, rules, and task cards | crates/contextra-audit-export/AGENTS.md, crates/contextra-avv-generator/AGENTS.md | +0/-0 \| +0/-0 \| +4/-2 | `SKIP-DOC` | Reine Dokumentationsänderung ohne Quelltextauswirkung. | - | nein | hoch |
| 02cd3fbe | 2026-10-13 | feat(compliance): add legal disclaimer and entrypoint wiring | crates/contextra-audit-export/src/bsi_mapping.rs, crates/contextra-audit-export/src/lib.rs (+10) | +196/-2 \| +256/-0 \| +0/-0 | `PORT-FEAT` | AVV-Generator: Rechtlicher Disclaimer und Entrypoint Wiring. | contextra-avv-generator | nein | hoch |
| 42cf7835 | 2026-10-13 | feat(rank,infer-onnx): wire RerankWiring and RerankStage | crates/contextra-infer-onnx/src/lib.rs, crates/contextra-infer-onnx/src/wiring.rs (+1) | +99/-0 \| +74/-0 \| +0/-0 | `PORT-WIRING` | Verdrahtung RerankWiring in infer-onnx. | contextra-infer-onnx | nein | niedrig |
| 0079a31c | 2026-10-13 | fix(contextra-agent): status encapsulation, state matrix & AgentWiring | crates/contextra-agent/src/context.rs, crates/contextra-agent/src/engine.rs (+10) | +180/-8 \| +111/-6 \| +0/-0 | `PORT-WIRING` | AgentWiring und State-Matrix encapsulation. | contextra-agent | nein | mittel |
| 1f99483c | 2026-10-13 | feat(cognition,db): wire cognition and multistep pipeline stages | crates/contextra-cognition/src/lib.rs, crates/contextra-cognition/src/memory_consolidation.rs (+4) | +184/-6 \| +64/-0 \| +0/-0 | `PORT-WIRING` | Verdrahtung Cognition Pipeline Stages. | contextra-cognition | nein | mittel |
| 4c1b7315 | 2026-10-13 | feat(adapt): wire AdaptWiring and refactor time calls to Clock port | crates/contextra-adapt/src/homeostat.rs, crates/contextra-adapt/src/lib.rs (+6) | +199/-11 \| +166/-0 \| +0/-0 | `PORT-WIRING` | Implementiert AdaptWiring und refactored Zeitaufrufe auf Clock-Port. | contextra-adapt | nein | mittel |
| 3313c87e | 2026-10-13 | feat(license): implement key rotation keyring and wiring entry point | crates/contextra-license/Cargo.toml, crates/contextra-license/src/lib.rs (+7) | +317/-8 \| +183/-0 \| +0/-0 | `PORT-FEAT` | Implementiert License Key-Rotation und Keyring. | contextra-license | nein | mittel |
| 35c6bee9 | 2026-10-13 | feat(types): implement TxId::try_new, TenantId::system, and new_unchecked | crates/contextra-testkit/tests/p07_dst_runner_test.rs, crates/contextra-testkit/tests/p07_meta_tests.rs (+2) | +1483/-1219 \| +30/-22 \| +0/-0 | `PORT-FEAT` | Funktionale Erweiterung in contextra-testkit. | contextra-testkit | nein | hoch |
| 12df2e39 | 2026-10-13 | feat(sandbox): implement Ed25519ModuleVerifier and SandboxWiring | crates/contextra-sandbox/Cargo.toml, crates/contextra-sandbox/src/admission.rs (+6) | +230/-9 \| +389/-0 \| +0/-0 | `PORT-FEAT` | Implementiert Ed25519ModuleVerifier und SandboxWiring. | contextra-sandbox | nein | mittel |
| 0b7ec9d2 | 2026-10-13 | feat(router): implement RouterWiring composition root and DriftStatusProvider | crates/contextra-router/src/dispatch.rs, crates/contextra-router/src/lib.rs (+3) | +144/-3 \| +248/-0 \| +0/-0 | `PORT-WIRING` | Implementiert RouterWiring Composition Root und DriftStatusProvider. | contextra-router | nein | niedrig |
| 6affc64b | 2026-10-13 | fix(sys,wire,simd): wire Ring 0 unsafe islands, regenerate flatbuffers and harden ACL | crates/contextra-simd/src/lib.rs, crates/contextra-simd/src/wiring.rs (+3) | +103/-51 \| +0/-0 \| +0/-0 | `PORT-FIX` | Fehlerkorrektur in contextra-simd. | contextra-simd | nein | hoch |
| e93d61be | 2026-10-13 | feat(sandbox): add module cache, worker pool, and CRDT gate | crates/contextra-sandbox/src/executor.rs, crates/contextra-sandbox/src/merge.rs (+3) | +457/-60 \| +416/-0 \| +0/-0 | `PORT-FEAT` | Implementiert Sandbox Modulcache, Worker-Pool und CRDT-Gate. | contextra-sandbox | nein | niedrig |
| d1d4a52e | 2026-10-13 | feat(privacy): wire PrivacyWiring and harden egress fail-closed tests | crates/contextra-privacy/AGENTS.md, crates/contextra-privacy/src/avv_generator.rs (+4) | +161/-1 \| +148/-0 \| +2/-2 | `PORT-WIRING` | Implementiert PrivacyWiring Composition Root. | contextra-privacy | nein | hoch |
| 59420f7d | 2026-10-13 | Shell-Commit | crates/contextra-wire/src/contextra_generated.rs | +1218/-1526 \| +0/-0 \| +0/-0 | `SKIP-GENERATED` | Generierter Code aus FlatBuffers-Schema (`flatc`), wird neu generiert. | - | nein | niedrig |
| 4b7709e6 | 2026-10-13 | Shell-Commit | crates/contextra-testkit/src/deletion_oracle/rawsweep.rs, crates/contextra-testkit/src/dst/engine.rs (+3) | +1535/-1229 \| +5/-12 \| +0/-0 | `SKIP-BOT` | Automatisierter Bot-/Clean-Up-Commit ohne Relevanz für Endprodukt. | - | nein | niedrig |
| 561dcec9 | 2026-10-13 | Shell-Commit | crates/contextra-testkit/src/dst/engine.rs, crates/contextra-testkit/src/dst/mod.rs (+5) | +14/-128 \| +19/-24 \| +0/-0 | `SKIP-BOT` | Automatisierter Bot-/Clean-Up-Commit ohne Relevanz für Endprodukt. | - | nein | mittel |
| 1ed3a86d | 2026-10-13 | feat(store): Cache-Scan und SecureBuf (Spezifikation C4.7) | crates/contextra-wire/build.rs | +1/-1 \| +0/-0 \| +0/-0 | `PORT-FEAT` | Funktionale Erweiterung in contextra-wire. | contextra-wire | nein | hoch |
| efeacc60 | 2026-10-12 | feat(contextra-testkit): implement DST framework, SimFs simulator and oracle | crates/contextra-testkit/src/dst/engine.rs, crates/contextra-testkit/src/dst/mod.rs (+16) | +1411/-16 \| +842/-17 \| +0/-0 | `DEL-R5` | Löschnachweis-Anteil für Testkit/Oracle (abgedeckt durch Auftrag R5/R10). | crates/contextra-testkit | nein | hoch |
| fd31a72e | 2026-10-12 | feat(testkit): implement Canary-Oracle raw sweep and marker set | crates/contextra-testkit/src/deletion_oracle/marker.rs, crates/contextra-testkit/src/deletion_oracle/mod.rs (+12) | +400/-5 \| +235/-8 \| +0/-0 | `DEL-R5` | Löschnachweis-Anteil für Testkit/Oracle (abgedeckt durch Auftrag R5/R10). | crates/contextra-testkit | nein | hoch |
| f62b965f | 2026-10-12 | fix(db,audit): adapt consumer crates to Ed25519 v3 DeletionProof | crates/contextra-audit-export/Cargo.toml, crates/contextra-audit-export/src/lib.rs | +25/-0 \| +0/-0 \| +0/-0 | `DEL-R5` | Anpassung der Audit/DB Consumern an Ed25519 v3 DeletionProof. | crates/contextra-audit-export | nein | hoch |
| 648538d0 | 2026-10-12 | Implement seed-reproducible DST framework in contextra-testkit | crates/contextra-testkit/src/dst/engine.rs, crates/contextra-testkit/src/dst/invariants.rs (+8) | +825/-0 \| +123/-0 \| +0/-0 | `PORT-FEAT` | Implementiert seed-reproduzierbares DST-Framework und SimFs im Testkit. | contextra-testkit | nein | mittel |
| e6fe4332 | 2026-10-12 | test(testkit): add independent Canary Oracle for raw byte deletion verification | crates/contextra-testkit/tests/canary_verification_test.rs, crates/contextra-testkit/tests/support/canary.rs | +0/-0 \| +307/-0 \| +0/-0 | `DEL-R5` | Löschnachweis-Anteil für Testkit/Oracle (abgedeckt durch Auftrag R5/R10). | crates/contextra-testkit | nein | niedrig |
| 2b3e7c8f | 2026-10-12 | feat(contextra-wire): add CTRA identifier, version field, and build.rs stale-code checks | crates/contextra-wire/build.rs, crates/contextra-wire/src/contextra_generated.rs (+2) | +131/-221 \| +6/-0 \| +0/-0 | `PORT-FEAT` | Wire-Protokoll-Kennung CTRA, Versionsfeld und build.rs Stale-Code-Checks. | contextra-wire | nein | hoch |
| 2d5c5b0d | 2026-10-11 | fix(doctrine): resolve panic, safety comment, and lint scoping violations in sys, wire, and text crates | crates/contextra-wire/src/lib.rs | +13/-10 \| +0/-0 \| +0/-0 | `PORT-FIX` | Fehlerkorrektur in contextra-wire. | contextra-wire | nein | niedrig |
| 8cedf9fb | 2026-10-11 | Shell-Commit | crates/contextra-wire/src/contextra_generated.rs | +69/-68 \| +0/-0 \| +0/-0 | `SKIP-GENERATED` | Generierter Code aus FlatBuffers-Schema (`flatc`), wird neu generiert. | - | nein | niedrig |
| 536e891e | 2026-10-11 | contextra-infer-candle: audit Candle ML inference path and review passes | crates/contextra-infer-candle/src/embedding.rs, crates/contextra-infer-candle/src/lib.rs (+1) | +9/-10 \| +2/-10 \| +0/-0 | `PORT-FEAT` | Funktionale Erweiterung in contextra-infer-candle. | contextra-infer-candle | nein | niedrig |
| 525ecd80 | 2026-10-11 | contextra-db: add plan TODOs for index-store transaction sync | crates/contextra-wire/src/contextra_generated.rs | +1492/-1189 \| +0/-0 \| +0/-0 | `SKIP-GENERATED` | Generierter Code aus FlatBuffers-Schema (`flatc`), wird neu generiert. | - | nein | niedrig |
| af63a482 | 2026-10-11 | contextra-rank: sanitize DriftDetector threshold inputs and fix CI gates | crates/contextra-wire/src/contextra_generated.rs | +67/-67 \| +0/-0 \| +0/-0 | `SKIP-GENERATED` | Generierter Code aus FlatBuffers-Schema (`flatc`), wird neu generiert. | - | nein | niedrig |
| bd0a2525 | 2026-10-11 | contextra-rank: harden DriftDetector state and non-finite handling | crates/contextra-wire/src/contextra_generated.rs | +67/-67 \| +0/-0 \| +0/-0 | `SKIP-GENERATED` | Generierter Code aus FlatBuffers-Schema (`flatc`), wird neu generiert. | - | nein | niedrig |
| 7a2a4734 | 2026-10-11 | contextra-infer-candle: add plan markers for Candle ML inference path | crates/contextra-infer-candle/src/embedding.rs, crates/contextra-infer-candle/src/gasp.rs (+2) | +7/-0 \| +0/-0 \| +0/-0 | `PORT-FEAT` | Funktionale Erweiterung in contextra-infer-candle. | contextra-infer-candle | nein | niedrig |
| 3e8d78e0 | 2026-10-11 | contextra-cognition: plan memory consolidation pass separation | crates/contextra-cognition/src/lib.rs, crates/contextra-cognition/src/memory_consolidation.rs (+1) | +4/-0 \| +0/-0 \| +0/-0 | `PORT-FEAT` | Funktionale Erweiterung in contextra-cognition. | contextra-cognition | nein | niedrig |
| 0eafbb28 | 2026-10-11 | contextra-infer-candle: add KV-cache bridge architecture plan TODOs | crates/contextra-infer-candle/src/lib.rs | +6/-0 \| +0/-0 \| +0/-0 | `PORT-FEAT` | Funktionale Erweiterung in contextra-infer-candle. | contextra-infer-candle | nein | niedrig |
| a2b063dc | 2026-10-11 | Shell-Commit | crates/contextra-wire/src/contextra_generated.rs | +1189/-1492 \| +0/-0 \| +0/-0 | `SKIP-GENERATED` | Generierter Code aus FlatBuffers-Schema (`flatc`), wird neu generiert. | - | nein | niedrig |
| 2a1af3f7 | 2026-10-11 | fix: repair workspace compilation after crypto changes | crates/contextra-infer-candle/src/embedding.rs, crates/contextra-infer-candle/tests/embedding_semantic_sanity.rs (+3) | +18/-4 \| +3/-1 \| +0/-0 | `PORT-FIX` | Fehlerkorrektur in contextra-infer-candle. | contextra-infer-candle | nein | hoch |
| 240f4f7c | 2026-10-11 | Shell-Commit | crates/contextra-infer-candle/benches/kv_bridge_bench.rs, crates/contextra-infer-candle/src/embedding.rs (+11) | +1758/-1221 \| +175/-45 \| +0/-0 | `SKIP-BOT` | Automatisierter Bot-/Clean-Up-Commit ohne Relevanz für Endprodukt. | - | nein | mittel |
| b270bc2e | 2026-10-10 | fix(mvcc): replace deprecated register with acquire and clean lints | crates/contextra-wire/src/contextra_generated.rs | +67/-68 \| +0/-0 \| +0/-0 | `SKIP-GENERATED` | Generierter Code aus FlatBuffers-Schema (`flatc`), wird neu generiert. | - | nein | niedrig |
| d6550563 | 2026-10-10 | Shell-Commit | crates/contextra-wire/src/contextra_generated.rs | +68/-67 \| +0/-0 \| +0/-0 | `SKIP-GENERATED` | Generierter Code aus FlatBuffers-Schema (`flatc`), wird neu generiert. | - | nein | niedrig |
| 7e1d65ba | 2026-10-10 | fix(mcp): gate mock provider branch behind test-utils feature (#4494) | crates/contextra-infer-candle/benches/kv_bridge_bench.rs, crates/contextra-infer-candle/src/embedding.rs (+10) | +1220/-1756 \| +45/-175 \| +0/-0 | `PORT-FIX` | Fehlerkorrektur in contextra-infer-candle. | contextra-infer-candle | nein | mittel |
| cea972df | 2026-10-09 | fix(vector): clear stale inbound edges on HNSW slot reuse (W4-06) (#4476) | crates/contextra-agent/Cargo.toml, crates/contextra-cognition/Cargo.toml (+4) | +25/-70 \| +0/-0 \| +0/-0 | `PORT-FIX` | Fehlerkorrektur in contextra-agent. | contextra-agent | nein | mittel |
| ef39f9f0 | 2026-10-08 | supply-chain(tokio): replace full feature with per-crate minimal features (#4464) | crates/contextra-agent/Cargo.toml, crates/contextra-cognition/Cargo.toml (+3) | +9/-9 \| +0/-0 \| +0/-0 | `PORT-FEAT` | Funktionale Erweiterung in contextra-agent. | contextra-agent | nein | niedrig |
| ae1fa1b5 | 2026-10-08 | fix(wire): generate SAFETY comments for unsafe blocks in FlatBuffers generator (#4463) | crates/contextra-wire/src/contextra_generated.rs | +61/-16 \| +0/-0 \| +0/-0 | `SKIP-GENERATED` | Generierter Code aus FlatBuffers-Schema (`flatc`), wird neu generiert. | - | nein | niedrig |
| 330c9847 | 2026-10-08 | P06: Harden persistent key shredding and DeletionProof durability (#4461) | crates/contextra-infer-candle/benches/kv_bridge_bench.rs, crates/contextra-infer-candle/src/inference/tests.rs (+5) | +13/-13 \| +6/-6 \| +0/-0 | `PORT-FEAT` | Funktionale Erweiterung in contextra-infer-candle. | contextra-infer-candle | nein | mittel |
| 2fee6170 | 2026-10-08 | test(candle): add embedding semantic sanity cosine test (#4459) | crates/contextra-infer-candle/tests/embedding_semantic_sanity.rs | +0/-0 \| +59/-0 \| +0/-0 | `PORT-FEAT` | Funktionale Erweiterung in contextra-infer-candle. | contextra-infer-candle | nein | niedrig |
| 5f5251e9 | 2026-10-08 | fix(crypto): durable RevocationLog in setup_kv_bridge and fail-closed lock handling (#4454) | crates/contextra-wire/src/contextra_generated.rs | +1492/-1188 \| +0/-0 \| +0/-0 | `SKIP-GENERATED` | Generierter Code aus FlatBuffers-Schema (`flatc`), wird neu generiert. | - | nein | niedrig |
| 16e64ea9 | 2026-10-08 | fix(candle): load config.json and fail when safetensors missing (#4446) | crates/contextra-infer-candle/src/embedding.rs, crates/contextra-infer-candle/tests/embedding_config_loading.rs (+1) | +251/-19 \| +110/-39 \| +0/-0 | `PORT-FIX` | Fehlerkorrektur in contextra-infer-candle. | contextra-infer-candle | nein | niedrig |
| eaa0df92 | 2026-10-08 | test(simd): add direct kernel tests and baseline safety verification (#4436) | crates/contextra-simd/src/dispatch.rs, crates/contextra-simd/src/kernels/avx2.rs (+4) | +766/-1 \| +78/-0 \| +0/-0 | `PORT-FEAT` | Funktionale Erweiterung in contextra-simd. | contextra-simd | nein | hoch |
| 67c7ca58 | 2026-10-07 | feat(store): integrate mandatory SnapshotFloor in CompactionEngine and flush (#4418) | crates/contextra-agent/src/engine.rs, crates/contextra-wire/src/contextra_generated.rs | +1187/-1466 \| +0/-0 \| +0/-0 | `PORT-FEAT` | Funktionale Erweiterung in contextra-agent. | contextra-agent | nein | niedrig |
| 4fb1e45b | 2026-10-07 | fix(vector,crypto,wire): propagate dropped IO errors and refactor quantizer training to try_train (#4413) | crates/contextra-infer-onnx/tests/j29closure-contextra-infer-onnx_removal_verification_test.rs, crates/contextra-wire/build.rs (+1) | +1469/-1188 \| +1/-3 \| +0/-0 | `PORT-FIX` | Fehlerkorrektur in contextra-infer-onnx. | contextra-infer-onnx | nein | hoch |
| 19d12ae4 | 2026-10-07 | Shell-Commit | crates/contextra-wire/src/contextra_generated.rs | +3/-1 \| +0/-0 \| +0/-0 | `SKIP-GENERATED` | Generierter Code aus FlatBuffers-Schema (`flatc`), wird neu generiert. | - | nein | niedrig |
| 4ce7aad7 | 2026-10-07 | docs(simd): add safety documentation and verify miri coverage (#4407) (#4411) | crates/contextra-simd/src/kernels/avx512.rs | +18/-0 \| +0/-0 \| +0/-0 | `SKIP-DOC` | Reine Dokumentationsänderung ohne Quelltextauswirkung. | - | nein | hoch |
| 5f47bc4a | 2026-10-07 | refactor(simd,wire): encapsulate kernels and require CpuFeatures token (#4401) | crates/contextra-simd/benches/simd_bench.rs, crates/contextra-simd/src/dispatch.rs (+8) | +259/-102 \| +50/-36 \| +0/-0 | `PORT-WIRING` | Composition Root Verdrahtung / Integration in contextra-simd. | contextra-simd | nein | hoch |
| 9b883d83 | 2026-10-07 | Harden WASM execution with AdmittedModule and ModuleVerifier (#4400) | crates/contextra-sandbox/Cargo.toml, crates/contextra-sandbox/src/admission.rs (+10) | +174/-90 \| +218/-30 \| +0/-0 | `PORT-FEAT` | Funktionale Erweiterung in contextra-sandbox. | contextra-sandbox | nein | mittel |
| a9271b68 | 2026-10-06 | Enforce WASM Module Provenance Policy Prior to Compilation (P05/F-02 HIGH) (#4386) | crates/contextra-sandbox/src/capabilities.rs, crates/contextra-sandbox/src/executor.rs (+2) | +184/-26 \| +105/-0 \| +0/-0 | `PORT-FEAT` | Funktionale Erweiterung in contextra-sandbox. | contextra-sandbox | nein | niedrig |
| f9e51798 | 2026-10-06 | docs(simd): add structured safety comments in kernels/avx512.rs (#4385) | crates/contextra-simd/src/kernels/avx512.rs | +186/-8 \| +0/-0 \| +0/-0 | `SKIP-DOC` | Reine Dokumentationsänderung ohne Quelltextauswirkung. | - | nein | hoch |
| 93c6363f | 2026-10-06 | docs(simd): add structured SAFETY comments to avx2, neon and dispatch (#4384) | crates/contextra-simd/src/dispatch.rs, crates/contextra-simd/src/kernels/avx2.rs (+1) | +269/-19 \| +0/-0 \| +0/-0 | `SKIP-DOC` | Reine Dokumentationsänderung ohne Quelltextauswirkung. | - | nein | hoch |
| 52e6ea7e | 2026-10-06 | Shell-Commit | crates/contextra-sandbox/src/capabilities.rs, crates/contextra-sandbox/src/executor.rs (+4) | +40/-0 \| +0/-0 \| +0/-0 | `SKIP-BOT` | Automatisierter Bot-/Clean-Up-Commit ohne Relevanz für Endprodukt. | - | nein | hoch |
| 7e5ef3e6 | 2026-10-06 | docs: add LSM-Tree correctness review report (Mission P02) (#4380) | crates/contextra-adapt/tests/j11-contextra-adapt_rie_greedy.rs, crates/contextra-infer-onnx/tests/j29closure-contextra-infer-onnx_removal_verification_test.rs (+1) | +1186/-1465 \| +8/-3 \| +0/-0 | `SKIP-DOC` | Reine Dokumentationsänderung ohne Quelltextauswirkung. | - | nein | niedrig |
| 495b89dc | 2026-10-06 | Mission P03 Security & Correctness Audit Report (#4378) | crates/contextra-sandbox/src/wasi.rs, crates/contextra-wire/src/contextra_generated.rs | +1481/-1200 \| +0/-0 \| +0/-0 | `PORT-FEAT` | Funktionale Erweiterung in contextra-sandbox. | contextra-sandbox | nein | niedrig |
| 56e5769a | 2026-10-06 | docs: standardize and audit all AGENTS.md files across multi-crate workspace | crates/contextra-adapt/AGENTS.md, crates/contextra-agent/AGENTS.md (+15) | +29/-0 \| +6/-8 \| +215/-84 | `SKIP-DOC` | Reine Dokumentationsänderung ohne Quelltextauswirkung. | - | nein | hoch |
| 2eaab7b4 | 2026-10-06 | fix: resolve compile errors and clippy cast warnings (#4377) | crates/contextra-sandbox/src/executor.rs, crates/contextra-sandbox/src/wasi.rs (+2) | +1214/-1514 \| +8/-6 \| +0/-0 | `PORT-FIX` | Fehlerkorrektur in contextra-sandbox. | contextra-sandbox | nein | niedrig |
| ce650578 | 2026-10-06 | feat(xtask): implement orphan-symbols scanner (AUFTRAG X01) (#4374) | crates/contextra-adapt/src/flow_thompson.rs, crates/contextra-adapt/tests/pid_candidate_pool.rs (+41) | +40/-27 \| +354/-194 \| +0/-0 | `DEL-R5` | Löschnachweis-Anteil für Testkit/Oracle (abgedeckt durch Auftrag R5/R10). | crates/contextra-testkit | nein | hoch |
| 8e9e1a41 | 2026-10-06 | docs(infer-onnx): update module docs and AGENTS.md for Candle separation (#4364) | crates/contextra-infer-onnx/AGENTS.md, crates/contextra-infer-onnx/src/lib.rs | +2/-1 \| +0/-0 \| +2/-2 | `SKIP-DOC` | Reine Dokumentationsänderung ohne Quelltextauswirkung. | - | nein | hoch |
| 6af1d79a | 2026-10-06 | fix(contextra-wire): replace flatbuffers get unwrap with unwrap_or_default in generator (#4363) | crates/contextra-wire/build.rs, crates/contextra-wire/src/contextra_generated.rs | +79/-20 \| +0/-0 \| +0/-0 | `PORT-FIX` | Fehlerkorrektur in contextra-wire. | contextra-wire | nein | hoch |
| f396085b | 2026-10-06 | fix(testkit): J37-CLOSURE wire contextra-testkit symbols and add integration tests (#4349) | crates/contextra-testkit/src/fault_vfs.rs, crates/contextra-testkit/src/manual_clock.rs (+2) | +14/-11 \| +132/-0 \| +0/-0 | `PORT-FIX` | Fehlerkorrektur in contextra-testkit. | contextra-testkit | nein | hoch |
| ed84f743 | 2026-10-06 | fix(infer-onnx): audit and remove create_candle_embedder symbol (J29) (#4347) | crates/contextra-infer-onnx/src/lib.rs, crates/contextra-infer-onnx/tests/j29-contextra-infer-onnx_candle_embedder_test.rs (+1) | +0/-11 \| +20/-31 \| +0/-0 | `PORT-FIX` | Fehlerkorrektur in contextra-infer-onnx. | contextra-infer-onnx | nein | niedrig |
| 0ed9343b | 2026-10-06 | fix(contextra-wire): wire SearchResponse FlatBuffers helper functions (#4293) (#4346) | crates/contextra-wire/src/contextra_generated.rs, crates/contextra-wire/tests/j27closure-contextra-wire_search_response.rs | +27/-0 \| +0/-0 \| +0/-0 | `PORT-FIX` | Fehlerkorrektur in contextra-wire. | contextra-wire | nein | niedrig |
| 4f850b19 | 2026-10-06 | test(infer-ollama): add J28 closure verification test for 5 symbols (#4344) | crates/contextra-infer-ollama/tests/j28closure-contextra-infer-ollama_symbols_test.rs | +0/-0 \| +105/-0 \| +0/-0 | `PORT-FEAT` | Funktionale Erweiterung in contextra-infer-ollama. | contextra-infer-ollama | nein | niedrig |
| 1fe11901 | 2026-10-06 | feat(router): J10-CLOSURE wire router lifecycle and outcome symbols (#4338) | crates/contextra-router/src/router/lifecycle.rs, crates/contextra-router/src/router/outcomes.rs (+1) | +41/-20 \| +135/-0 \| +0/-0 | `PORT-WIRING` | Composition Root Verdrahtung / Integration in contextra-router. | contextra-router | nein | niedrig |
| cc023a25 | 2026-10-06 | fix(privacy): wire open symbols and complete J19 audit closure for contextra-privacy (#4335) | crates/contextra-privacy/src/egress_gateway.rs, crates/contextra-privacy/src/egress_guard.rs (+4) | +147/-39 \| +170/-54 \| +0/-0 | `PORT-FIX` | Fehlerkorrektur in contextra-privacy. | contextra-privacy | nein | hoch |
| 4efb4ced | 2026-10-06 | feat(agent): wire ClmScratchpad builder methods and add closure tests (#4330) | crates/contextra-agent/src/clm_scratchpad.rs, crates/contextra-agent/tests/j01closure-contextra-agent_wiring_test.rs | +21/-0 \| +106/-0 \| +0/-0 | `PORT-WIRING` | Composition Root Verdrahtung / Integration in contextra-agent. | contextra-agent | nein | niedrig |
| 14255c87 | 2026-10-06 | fix(infer-candle): wire J05 symbols into production call sites (#4329) | crates/contextra-infer-candle/src/embedding.rs, crates/contextra-infer-candle/src/gasp.rs (+4) | +30/-9 \| +169/-0 \| +0/-0 | `PORT-FIX` | Fehlerkorrektur in contextra-infer-candle. | contextra-infer-candle | nein | hoch |
| e214ae8d | 2026-10-06 | feat(cognition): wire contextra-cognition symbols and session guard (#4328) | crates/contextra-cognition/src/consolidation_executor.rs, crates/contextra-cognition/src/context.rs (+2) | +99/-7 \| +153/-0 \| +0/-0 | `PORT-WIRING` | Composition Root Verdrahtung / Integration in contextra-cognition. | contextra-cognition | nein | niedrig |
| c704224b | 2026-10-05 | Verify and test 5 license gate closure symbols (#4323) | crates/contextra-license/src/signed_gate.rs, crates/contextra-license/tests/j26closure-contextra-license_closure_tests.rs | +0/-8 \| +50/-0 \| +0/-0 | `PORT-FEAT` | Funktionale Erweiterung in contextra-license. | contextra-license | nein | niedrig |
| f3a13b62 | 2026-10-05 | wire default_technical_measures and render_avv_markdown in contextra-avv-generator (#4322) | crates/contextra-avv-generator/src/lib.rs, crates/contextra-avv-generator/tests/j17closure-contextra-avv-generator_symbols_wiring_test.rs | +13/-2 \| +70/-0 \| +0/-0 | `PORT-WIRING` | Composition Root Verdrahtung / Integration in contextra-avv-generator. | contextra-avv-generator | nein | niedrig |
| 91c344ec | 2026-10-05 | fix(sandbox): verify and finalize J20 symbol closure for contextra-sandbox (#4321) | crates/contextra-sandbox/src/approval.rs, crates/contextra-sandbox/src/capabilities.rs (+1) | +0/-2 \| +125/-0 \| +0/-0 | `PORT-FIX` | Fehlerkorrektur in contextra-sandbox. | contextra-sandbox | nein | niedrig |
| e7f83d13 | 2026-10-05 | fix(audit-export): wire audit export symbols and add integration tests (#4319) | crates/contextra-audit-export/src/lib.rs, crates/contextra-audit-export/tests/j16closure-contextra-audit-export_test.rs | +0/-1 \| +96/-0 \| +0/-0 | `PORT-FIX` | Fehlerkorrektur in contextra-audit-export. | contextra-audit-export | nein | hoch |
| 046a4a46 | 2026-10-05 | feat(router): wire 7 router symbols and route_and_dispatch_to_slm (#4317) | crates/contextra-router/src/arm_registry.rs, crates/contextra-router/src/dispatch.rs (+3) | +71/-8 \| +196/-0 \| +0/-0 | `PORT-WIRING` | Composition Root Verdrahtung / Integration in contextra-router. | contextra-router | nein | hoch |
| d8d2f628 | 2026-10-05 | fix(contextra-testkit): wire and document testkit symbols (#4313) | crates/contextra-testkit/src/fault_vfs.rs, crates/contextra-testkit/src/manual_clock.rs (+4) | +6/-11 \| +142/-0 \| +0/-0 | `PORT-FIX` | Fehlerkorrektur in contextra-testkit. | contextra-testkit | nein | hoch |
| adf205ab | 2026-10-05 | bench(store): establish durability parity for competitive KV benchmarks (#4310) | crates/contextra-wire/src/contextra_generated.rs | +1446/-1216 \| +0/-0 \| +0/-0 | `SKIP-GENERATED` | Generierter Code aus FlatBuffers-Schema (`flatc`), wird neu generiert. | - | nein | niedrig |
| 3693216b | 2026-10-05 | fix(contextra-infer-onnx): wire create_candle_embedder (#4307) | crates/contextra-infer-onnx/src/lib.rs, crates/contextra-infer-onnx/tests/j29-contextra-infer-onnx_candle_embedder_test.rs | +1/-0 \| +31/-0 \| +0/-0 | `PORT-FIX` | Fehlerkorrektur in contextra-infer-onnx. | contextra-infer-onnx | nein | niedrig |
| f85db019 | 2026-10-05 | fix(contextra-privacy): wire uncalled symbols in egress gateway and annotate facade methods (#4300) | crates/contextra-privacy/src/egress_gateway.rs, crates/contextra-privacy/src/egress_guard.rs (+4) | +52/-17 \| +170/-2 \| +0/-0 | `PORT-FIX` | Fehlerkorrektur in contextra-privacy. | contextra-privacy | nein | hoch |
| c3fac55d | 2026-10-05 | fix(contextra-infer-ollama): wire uncalled symbols and add integration tests (#4297) | crates/contextra-infer-ollama/src/client/core.rs, crates/contextra-infer-ollama/src/importance.rs (+1) | +12/-14 \| +168/-0 \| +0/-0 | `PORT-FIX` | Fehlerkorrektur in contextra-infer-ollama. | contextra-infer-ollama | nein | niedrig |
| f42bdc7e | 2026-10-05 | fix(contextra-license): wire target symbols to production paths (#4294) | crates/contextra-license/src/signed_gate.rs, crates/contextra-license/tests/j26-contextra-license_wiring_tests.rs | +55/-0 \| +53/-0 \| +0/-0 | `PORT-FIX` | Fehlerkorrektur in contextra-license. | contextra-license | nein | niedrig |
| 85ec85f3 | 2026-10-05 | fix(contextra-sandbox): wire requires_approval and pure_merge_operator (#4292) | crates/contextra-sandbox/src/approval.rs, crates/contextra-sandbox/src/capabilities.rs (+2) | +33/-13 \| +169/-0 \| +0/-0 | `PORT-FIX` | Fehlerkorrektur in contextra-sandbox. | contextra-sandbox | nein | niedrig |
| 0960e83b | 2026-10-05 | fix(contextra-wire): wire SearchResponse FlatBuffers helper functions (#4293) | crates/contextra-wire/src/contextra_generated.rs, crates/contextra-wire/tests/j27-contextra-wire_search_response.rs | +27/-0 \| +113/-0 \| +0/-0 | `PORT-FIX` | Fehlerkorrektur in contextra-wire. | contextra-wire | nein | niedrig |
| 8ac79c7e | 2026-10-05 | fix(contextra-audit-export): wire exporter symbols to AuditRegisterExporter (#4290) | crates/contextra-audit-export/src/lib.rs, crates/contextra-audit-export/tests/j16-contextra-audit-export_wiring_test.rs | +50/-0 \| +51/-0 \| +0/-0 | `PORT-FIX` | Fehlerkorrektur in contextra-audit-export. | contextra-audit-export | nein | hoch |
| 03077fd4 | 2026-10-05 | fix(contextra-avv-generator): wire default_technical_measures and render_avv_markdown into AvvContext (#4289) | crates/contextra-avv-generator/src/lib.rs, crates/contextra-avv-generator/tests/j17-contextra-avv-generator_wiring_test.rs | +92/-0 \| +60/-0 \| +0/-0 | `PORT-FIX` | Fehlerkorrektur in contextra-avv-generator. | contextra-avv-generator | nein | niedrig |
| 4446e367 | 2026-10-05 | fix(contextra-agent): wire uncalled symbols in audit, dlq, engine, event_source and graph (#4287) | crates/contextra-agent/src/audit.rs, crates/contextra-agent/src/dlq.rs (+4) | +50/-16 \| +152/-0 \| +0/-0 | `PORT-FIX` | Fehlerkorrektur in contextra-agent. | contextra-agent | nein | mittel |
| 20ee467b | 2026-10-05 | fix(contextra-router): document facade-wiring TODOs for 7 router symbols (#4286) | crates/contextra-router/src/arm_registry.rs, crates/contextra-router/src/dispatch.rs (+2) | +7/-0 \| +94/-0 \| +0/-0 | `PORT-FIX` | Fehlerkorrektur in contextra-router. | contextra-router | nein | hoch |
| 366e2cde | 2026-10-05 | fix(contextra-infer-candle): wire uncalled public symbols into production paths (#4285) | crates/contextra-infer-candle/src/embedding.rs, crates/contextra-infer-candle/src/inference/core.rs (+2) | +68/-10 \| +144/-0 \| +0/-0 | `PORT-FIX` | Fehlerkorrektur in contextra-infer-candle. | contextra-infer-candle | nein | hoch |
| d9e9c94f | 2026-10-05 | fix(contextra-cognition): wire start_worker and annotate builder symbols (#4283) | crates/contextra-cognition/src/consolidation_executor.rs, crates/contextra-cognition/src/context.rs (+2) | +10/-3 \| +224/-0 \| +0/-0 | `PORT-FIX` | Fehlerkorrektur in contextra-cognition. | contextra-cognition | nein | niedrig |
| 759c5206 | 2026-10-05 | fix(contextra-adapt): wire uncalled adaptation symbols (#4282) | crates/contextra-adapt/src/bandit.rs, crates/contextra-adapt/src/drift.rs (+9) | +118/-0 \| +180/-0 \| +0/-0 | `PORT-FIX` | Fehlerkorrektur in contextra-adapt. | contextra-adapt | nein | mittel |
| 9e6a600b | 2026-10-05 | fix(contextra-router): wire lifecycle and outcomes router symbols (#4281) | crates/contextra-router/src/router/lifecycle.rs, crates/contextra-router/src/router/outcomes.rs (+1) | +7/-0 \| +273/-0 \| +0/-0 | `PORT-FIX` | Fehlerkorrektur in contextra-router. | contextra-router | nein | niedrig |
| 5d9412fd | 2026-10-05 | fix(contextra-agent): wire scratchpad and context methods (#4279) | crates/contextra-agent/src/context.rs, crates/contextra-agent/src/engine.rs (+1) | +10/-13 \| +106/-0 \| +0/-0 | `PORT-FIX` | Fehlerkorrektur in contextra-agent. | contextra-agent | nein | niedrig |
| b6b153b2 | 2026-10-05 | fix(contextra-adapt): fix clippy warnings and ensure clean build (#4276) | crates/contextra-adapt/src/flow_thompson.rs, crates/contextra-adapt/src/lyapunov.rs (+2) | +22/-2 \| +0/-0 \| +0/-0 | `PORT-FIX` | Fehlerkorrektur in contextra-adapt. | contextra-adapt | nein | niedrig |
| 9f32df43 | 2026-10-05 | fix(testkit): fix clippy truncation warning in manual_clock (#4275) | crates/contextra-testkit/src/manual_clock.rs | +2/-1 \| +0/-0 \| +0/-0 | `PORT-FIX` | Fehlerkorrektur in contextra-testkit. | contextra-testkit | nein | hoch |
| 0ced841b | 2026-10-05 | fix(wire): resolve clippy cast_sign_loss warning in ipc_tests (#4272) | crates/contextra-wire/tests/ipc_tests.rs | +0/-0 \| +5/-3 \| +0/-0 | `PORT-FIX` | Fehlerkorrektur in contextra-wire. | contextra-wire | nein | niedrig |
| cad222dd | 2026-10-05 | feat(privacy): make CloudResponseRehydrator fail-closed without request scope (#4265) | crates/contextra-privacy/src/egress_gateway.rs, crates/contextra-privacy/tests/rehydrator_scope.rs | +15/-5 \| +58/-0 \| +0/-0 | `PORT-FEAT` | Funktionale Erweiterung in contextra-privacy. | contextra-privacy | nein | hoch |
| 83aa706a | 2026-10-05 | fix(privacy): wire pattern classifier helpers in egress_vault (#4264) | crates/contextra-privacy/src/egress_vault.rs | +9/-5 \| +0/-0 \| +0/-0 | `PORT-FIX` | Fehlerkorrektur in contextra-privacy. | contextra-privacy | nein | hoch |
| e31ed5c4 | 2026-10-05 | Shell-Commit | crates/contextra-wire/src/contextra_generated.rs | +1189/-1446 \| +0/-0 \| +0/-0 | `SKIP-GENERATED` | Generierter Code aus FlatBuffers-Schema (`flatc`), wird neu generiert. | - | nein | niedrig |
| 2b03e237 | 2026-10-05 | fix(privacy): resolve SURROGATE_PREFIX and eval_text compilation errors in contextra-privacy (#4262) | crates/contextra-privacy/src/egress_gateway.rs, crates/contextra-privacy/src/egress_vault.rs | +61/-25 \| +0/-0 \| +0/-0 | `PORT-FIX` | Fehlerkorrektur in contextra-privacy. | contextra-privacy | nein | hoch |

## Entscheidung nötig

Es wurden keine Commits der Klasse `DECISION` identifiziert. Alle Commits konnten anhand des Regelwerks, der Vorrangregeln und der Maintainer-Freigaben eindeutig klassifiziert werden.

## Abhängigkeiten und Reihenfolge

1. **Ring 0 / Unterbau-Kompilierung (`PORT-REQUIRED`):**
   - `11ee512c` (`contextra-ports`, `testkit` Umstellung auf Clock/Rng/IdGenerator) und `a3f634cb` (Erasure Ports Definition in `contextra-ports`) müssen zwingend vor allen weiteren Portierungen integriert werden, da der wiederhergestellte Unterbau diese Schnittstellen voraussetzt.
2. **System & Wire Island dispatch (`PORT-FEAT`):**
   - `51f55cb6` (`sys`, `wire`, `simd` functional dispatch) und `2b3e7c8f` (`wire` CTRA Header & Checks) sichern das Wire-Format und den SIMD-Dispatch ab.
3. **Composition Roots / Wiring (`PORT-WIRING`):**
   - Gemäß Maintainer-Freigabe (WIRING-FREIGABE = JA) werden die Composition Root Verdrahtungen in Reihenfolge der Layer-Topologie portiert: `4c1b7315` (`adapt`), `0b7ec9d2` (`router`), `d1d4a52e` (`privacy`), `42cf7835` (`infer-onnx`), `1f99483c` (`cognition`), `0079a31c` (`agent`).
4. **Modul- und Eigenständige Features (`PORT-FEAT`):**
   - `3313c87e` (`license` Keyring), `e93d61be` / `12df2e39` (`sandbox` Modulcache, Worker-Pool, CRDT, Ed25519Verifier), `cc6027e4` (`audit-export`), `02cd3fbe` (`avv-generator`), `648538d0` / `efeacc60` (`testkit` DST-Framework & SimFs).
5. **Löschnachweis-Anteile (`DEL-R5`):**
   - Testkit/Oracle Löschgarantien-Commits werden im Rahmen von Auftrag R5 verarbeitet.

## Auffälligkeiten

- **FlatBuffers Generat-Handling in `contextra-wire`:**
  - 15 Commits betreffen ausschließlich von `flatc` erzeugten Code (`contextra_generated.rs`, `fbs/`). Diese wurden strikt als `SKIP-GENERATED` eingestuft, da generierte Artefakte gemäß Maintainer-Vorgabe neu gebaut und nicht portiert werden.
- **Duplikats- und Iterationsreihen im Testkit (SP-08, J-07):**
  - Die Commits `5776b652`, `a2bd6341`, `51e1e765`, `60fe48a9`, `7d6f51f4` und `885145d8` bildeten temporäre Arbeitsstände für SP-08/J-07, die in späteren Commits (`bbb605cd`, `a3f634cb`, `11ee512c`, `d5e55fe0`) vollständig aufgesaugt wurden. Sie sind als `SKIP-SUPERSEDED` markiert.
- **Umbenannte Testdateien in `contextra-wire` / `contextra-infer-onnx`:**
  - In `0ed9343b` / `ed84f743` wurden Testdateien zur Symbol-Entfernung bzw. Closure-Verifikation umbenannt. Die finale Version im Spender gilt als Netto-Referenz.

## Löschungen respektiert

Gemäß der Maintainer-Entscheidung (Backup `8412f086` hat Dateien absichtlich gelöscht) wurden folgende Dateien, die in `main` gelöscht sind, nicht wiederhergestellt, obwohl der Spender sie noch modifiziert hat (Fall G):

| Pfad | Fall | Grund |
| :--- | :--- | :--- |
| `crates/contextra-adapt/tests/pid_candidate_pool.rs` | G | In main bewusst gelöscht, Löschung bleibt respektiert. |
| `crates/contextra-audit-export/tests/art30_register_complete.rs` | G | In main bewusst gelöscht, Löschung bleibt respektiert. |
| `crates/contextra-infer-candle/tests/attention_producer_flow.rs` | G | In main bewusst gelöscht, Löschung bleibt respektiert. |
| `crates/contextra-privacy/tests/egress_vault_additional_dlp.rs` | G | In main bewusst gelöscht, Löschung bleibt respektiert. |
| `crates/contextra-router/tests/fc_ts_routing_determinism.rs` | G | In main bewusst gelöscht, Löschung bleibt respektiert. |
| `crates/contextra-router/tests/feature_cleanup_validation.rs` | G | In main bewusst gelöscht, Löschung bleibt respektiert. |

## Anhang: Dateien (Dateifälle A bis H)

| Pfad | Fall | Konflikt in merge-tree | beteiligte Commits |
| :--- | :--- | :--- | :--- |
| crates/contextra-adapt/AGENTS.md | D | ja | 426dfead, 56e5769a |
| crates/contextra-adapt/src/bandit.rs | D | nein | 759c5206 |
| crates/contextra-adapt/src/drift.rs | B | nein | 759c5206 |
| crates/contextra-adapt/src/flow_thompson.rs | B | nein | ce650578, 759c5206, b6b153b2 |
| crates/contextra-adapt/src/homeostat.rs | D | nein | 4c1b7315 |
| crates/contextra-adapt/src/lib.rs | B | nein | 4c1b7315 |
| crates/contextra-adapt/src/lyapunov.rs | B | nein | b6b153b2 |
| crates/contextra-adapt/src/off_policy.rs | B | nein | 4c1b7315 |
| crates/contextra-adapt/src/offpolicy.rs | B | nein | b6b153b2 |
| crates/contextra-adapt/src/pid.rs | B | nein | 4c1b7315, b6b153b2 |
| crates/contextra-adapt/src/pid_latency_controller.rs | B | nein | 4c1b7315, 759c5206 |
| crates/contextra-adapt/src/rie_greedy.rs | B | nein | 759c5206 |
| crates/contextra-adapt/src/wiring.rs | H | nein | 4c1b7315 |
| crates/contextra-adapt/tests/j11-contextra-adapt_bandit_tenant.rs | H | nein | 759c5206 |
| crates/contextra-adapt/tests/j11-contextra-adapt_drift_bridge.rs | H | nein | 759c5206 |
| crates/contextra-adapt/tests/j11-contextra-adapt_drift_ensemble.rs | H | nein | 759c5206 |
| crates/contextra-adapt/tests/j11-contextra-adapt_flow_thompson.rs | H | nein | 759c5206 |
| crates/contextra-adapt/tests/j11-contextra-adapt_pid_latency.rs | H | nein | 759c5206 |
| crates/contextra-adapt/tests/j11-contextra-adapt_rie_greedy.rs | H | nein | 7e5ef3e6, 759c5206 |
| crates/contextra-adapt/tests/pid_candidate_pool.rs | G | ja | ce650578 |
| crates/contextra-adapt/tests/wiring_and_clock_test.rs | H | nein | 4c1b7315 |
| crates/contextra-adapt/tests/wiring_test.rs | H | nein | 4c1b7315 |
| crates/contextra-agent/AGENTS.md | D | ja | 56e5769a |
| crates/contextra-agent/Cargo.toml | A | nein | cea972df, ef39f9f0 |
| crates/contextra-agent/src/audit.rs | B | nein | 4446e367 |
| crates/contextra-agent/src/clm_scratchpad.rs | B | nein | 4efb4ced |
| crates/contextra-agent/src/context.rs | B | nein | 0079a31c, 5d9412fd |
| crates/contextra-agent/src/dlq.rs | B | nein | 4446e367 |
| crates/contextra-agent/src/engine.rs | B | nein | 0079a31c, 67c7ca58, 4446e367, 5d9412fd |
| crates/contextra-agent/src/engine/condition.rs | B | nein | 0079a31c |
| crates/contextra-agent/src/engine/tests.rs | B | nein | 0079a31c |
| crates/contextra-agent/src/event_source.rs | B | nein | 4446e367 |
| crates/contextra-agent/src/graph.rs | B | nein | 0079a31c, 4446e367 |
| crates/contextra-agent/src/lib.rs | B | nein | 0079a31c |
| crates/contextra-agent/src/wiring.rs | H | nein | 4c47f895, 0079a31c |
| crates/contextra-agent/tests/agent_recovery.rs | B | nein | 0079a31c |
| crates/contextra-agent/tests/final_state_test.rs | B | nein | 0079a31c |
| crates/contextra-agent/tests/graph_integration.rs | B | nein | 0079a31c |
| crates/contextra-agent/tests/j01-contextra-agent-scratchpad-kontext_test.rs | H | nein | ce650578, 5d9412fd |
| crates/contextra-agent/tests/j01closure-contextra-agent_wiring_test.rs | H | nein | 4efb4ced |
| crates/contextra-agent/tests/j02-contextra-agent-agent-audit-dlq-engine-e_wiring_test.rs | H | nein | ce650578, 4446e367 |
| crates/contextra-agent/tests/persistence_test.rs | B | nein | 0079a31c |
| crates/contextra-agent/tests/wiring_test.rs | H | nein | 4c47f895, 0079a31c |
| crates/contextra-audit-export/AGENTS.md | D | ja | 3dc67417, 56e5769a |
| crates/contextra-audit-export/Cargo.toml | B | nein | f62b965f |
| crates/contextra-audit-export/src/bsi_mapping.rs | B | nein | cc6027e4, 02cd3fbe |
| crates/contextra-audit-export/src/lib.rs | D | ja | 02cd3fbe, f62b965f, e7f83d13, 8ac79c7e |
| crates/contextra-audit-export/src/markdown_template.rs | D | ja | cc6027e4, 02cd3fbe, ce650578 |
| crates/contextra-audit-export/src/wiring.rs | H | nein | 02cd3fbe |
| crates/contextra-audit-export/tests/art30_register_complete.rs | G | ja | ce650578 |
| crates/contextra-audit-export/tests/campaign_j27_compliance_test.rs | C | nein | ce650578 |
| crates/contextra-audit-export/tests/disclaimer_and_wiring_red_test.rs | H | nein | 02cd3fbe |
| crates/contextra-audit-export/tests/export_claims_scope.rs | H | nein | cc6027e4 |
| crates/contextra-audit-export/tests/j16-contextra-audit-export_wiring_test.rs | H | nein | 8ac79c7e |
| crates/contextra-audit-export/tests/j16closure-contextra-audit-export_test.rs | H | nein | e7f83d13 |
| crates/contextra-audit-export/tests/tenant_isolation_and_wiring_e2e_test.rs | H | nein | 02cd3fbe |
| crates/contextra-avv-generator/AGENTS.md | D | ja | 3dc67417, 56e5769a |
| crates/contextra-avv-generator/Cargo.toml | B | nein | 02cd3fbe |
| crates/contextra-avv-generator/src/lib.rs | B | nein | 02cd3fbe, f3a13b62, 03077fd4 |
| crates/contextra-avv-generator/src/template.rs | B | nein | 02cd3fbe |
| crates/contextra-avv-generator/src/wiring.rs | H | nein | 02cd3fbe |
| crates/contextra-avv-generator/tests/avv_wiring_e2e_test.rs | H | nein | 02cd3fbe |
| crates/contextra-avv-generator/tests/campaign_j27_avv_test.rs | C | nein | ce650578 |
| crates/contextra-avv-generator/tests/disclaimer_and_wiring_red_test.rs | H | nein | 02cd3fbe |
| crates/contextra-avv-generator/tests/j17-contextra-avv-generator_wiring_test.rs | H | nein | ce650578, 03077fd4 |
| crates/contextra-avv-generator/tests/j17closure-contextra-avv-generator_symbols_wiring_test.rs | H | nein | ce650578, f3a13b62 |
| crates/contextra-cognition/AGENTS.md | D | ja | 426dfead, 56e5769a |
| crates/contextra-cognition/Cargo.toml | B | nein | 4c47f895, cea972df, ef39f9f0 |
| crates/contextra-cognition/src/consolidation_executor.rs | B | nein | e214ae8d, d9e9c94f |
| crates/contextra-cognition/src/context.rs | B | nein | e214ae8d, d9e9c94f |
| crates/contextra-cognition/src/context_compaction/compactor.rs | A | nein | - |
| crates/contextra-cognition/src/context_compaction/tests.rs | B | nein | 4c47f895 |
| crates/contextra-cognition/src/lib.rs | B | nein | 1f99483c, 3e8d78e0 |
| crates/contextra-cognition/src/maintenance_scheduler.rs | B | nein | ce650578, e214ae8d, d9e9c94f |
| crates/contextra-cognition/src/memory_consolidation.rs | B | nein | 1f99483c, 3e8d78e0 |
| crates/contextra-cognition/src/synthesis_phase.rs | B | nein | 1f99483c, 3e8d78e0 |
| crates/contextra-cognition/src/wiring.rs | H | nein | 1f99483c |
| crates/contextra-cognition/tests/cognition_wiring_integration_test.rs | H | nein | 1f99483c |
| crates/contextra-cognition/tests/j13-contextra-cognition_wiring_test.rs | H | nein | ce650578, d9e9c94f |
| crates/contextra-cognition/tests/j13closure-contextra-cognition_wiring_test.rs | H | nein | ce650578, e214ae8d |
| crates/contextra-cognition/tests/leanrag_aggregation.rs | B | nein | 1f99483c |
| crates/contextra-infer-candle/AGENTS.md | D | ja | 56e5769a |
| crates/contextra-infer-candle/benches/kv_bridge_bench.rs | B | nein | 240f4f7c, 7e1d65ba, 330c9847 |
| crates/contextra-infer-candle/src/embedding.rs | B | nein | 536e891e, 7a2a4734, 2a1af3f7, 240f4f7c, 7e1d65ba, 16e64ea9, 14255c87, 366e2cde |
| crates/contextra-infer-candle/src/gasp.rs | B | nein | 7a2a4734, 14255c87 |
| crates/contextra-infer-candle/src/gguf_loader.rs | B | nein | 7a2a4734 |
| crates/contextra-infer-candle/src/inference.rs | B | nein | 7a2a4734 |
| crates/contextra-infer-candle/src/inference/core.rs | D | ja | 4c47f895, ce650578, 14255c87, 366e2cde |
| crates/contextra-infer-candle/src/inference/prefix_reuse.rs | B | nein | 14255c87 |
| crates/contextra-infer-candle/src/inference/tests.rs | B | nein | 240f4f7c, 7e1d65ba, 330c9847 |
| crates/contextra-infer-candle/src/kv_bridge.rs | D | nein | 240f4f7c, 7e1d65ba, 330c9847 |
| crates/contextra-infer-candle/src/kv_state.rs | B | nein | 14255c87, 366e2cde |
| crates/contextra-infer-candle/src/lib.rs | B | nein | 536e891e, 0eafbb28 |
| crates/contextra-infer-candle/src/model_registry.rs | B | nein | 240f4f7c |
| crates/contextra-infer-candle/tests/attention_producer_flow.rs | G | ja | ce650578 |
| crates/contextra-infer-candle/tests/embedding_config_loading.rs | H | nein | 536e891e, 240f4f7c, 7e1d65ba, 16e64ea9 |
| crates/contextra-infer-candle/tests/embedding_normalization_proptest.rs | B | nein | 240f4f7c, 7e1d65ba, 16e64ea9 |
| crates/contextra-infer-candle/tests/embedding_semantic_sanity.rs | H | nein | 2a1af3f7, 240f4f7c, 7e1d65ba, 2fee6170 |
| crates/contextra-infer-candle/tests/j05-contextra-infer-candle_wiring_test.rs | H | nein | ce650578, 366e2cde |
| crates/contextra-infer-candle/tests/j05closure-contextra-infer-candle_closure_test.rs | H | nein | ce650578, 14255c87 |
| crates/contextra-infer-candle/tests/kv_bridge_and_stress_test.rs | B | nein | 240f4f7c, 7e1d65ba, 330c9847 |
| crates/contextra-infer-candle/tests/kv_bridge_roundtrip.rs | B | nein | 240f4f7c, 7e1d65ba, 330c9847 |
| crates/contextra-infer-candle/tests/kv_bridge_tenant_key_separation.rs | B | nein | 240f4f7c, 7e1d65ba, 330c9847 |
| crates/contextra-infer-candle/tests/kv_bridge_version_mismatch.rs | B | nein | 240f4f7c, 7e1d65ba, 330c9847 |
| crates/contextra-infer-ollama/AGENTS.md | D | ja | 56e5769a |
| crates/contextra-infer-ollama/src/client/core.rs | D | nein | c3fac55d |
| crates/contextra-infer-ollama/src/importance.rs | B | nein | c3fac55d |
| crates/contextra-infer-ollama/tests/j28-contextra-infer-ollama_wire_tests.rs | H | nein | ce650578, c3fac55d |
| crates/contextra-infer-ollama/tests/j28closure-contextra-infer-ollama_symbols_test.rs | H | nein | ce650578, 4f850b19 |
| crates/contextra-infer-onnx/AGENTS.md | D | ja | 56e5769a, 8e9e1a41 |
| crates/contextra-infer-onnx/Cargo.toml | A | nein | cea972df, ef39f9f0 |
| crates/contextra-infer-onnx/src/lib.rs | B | nein | 42cf7835, 8e9e1a41, ed84f743, 3693216b |
| crates/contextra-infer-onnx/src/wiring.rs | H | nein | 42cf7835 |
| crates/contextra-infer-onnx/tests/j29-contextra-infer-onnx_candle_embedder_test.rs | H (gelöscht/umbenannt) | nein | ed84f743, 3693216b |
| crates/contextra-infer-onnx/tests/j29closure-contextra-infer-onnx_removal_verification_test.rs | H | nein | 4fb1e45b, 7e5ef3e6, ed84f743 |
| crates/contextra-infer-onnx/tests/reranker_wiring_test.rs | H | nein | 42cf7835 |
| crates/contextra-license/AGENTS.md | D | ja | 56e5769a |
| crates/contextra-license/Cargo.toml | B | nein | 3313c87e |
| crates/contextra-license/src/lib.rs | B | nein | 3313c87e |
| crates/contextra-license/src/signed_gate.rs | B | nein | 3313c87e, ce650578, c704224b, f42bdc7e |
| crates/contextra-license/src/wiring.rs | H | nein | 3313c87e |
| crates/contextra-license/tests/installation_binding.rs | B | nein | 3313c87e |
| crates/contextra-license/tests/j26-contextra-license_wiring_tests.rs | H | nein | ce650578, f42bdc7e |
| crates/contextra-license/tests/j26closure-contextra-license_closure_tests.rs | H | nein | ce650578, c704224b |
| crates/contextra-license/tests/key_rotation_tests.rs | H | nein | 3313c87e |
| crates/contextra-license/tests/legacy_payload_compat.rs | B | nein | 3313c87e |
| crates/contextra-license/tests/signed_gate.rs | B | nein | 3313c87e |
| crates/contextra-license/tests/wiring_mode_tests.rs | H | nein | 3313c87e |
| crates/contextra-privacy/AGENTS.md | D | ja | 426dfead, d1d4a52e, 56e5769a |
| crates/contextra-privacy/Cargo.toml | A | nein | cea972df, ef39f9f0 |
| crates/contextra-privacy/src/avv_generator.rs | B | nein | d1d4a52e |
| crates/contextra-privacy/src/egress_gateway.rs | D | ja | ce650578, cc023a25, f85db019, cad222dd, 2b03e237 |
| crates/contextra-privacy/src/egress_guard.rs | B | nein | d1d4a52e, cc023a25, f85db019 |
| crates/contextra-privacy/src/egress_vault.rs | D | ja | ce650578, cc023a25, f85db019, 83aa706a, 2b03e237 |
| crates/contextra-privacy/src/lib.rs | D | nein | d1d4a52e |
| crates/contextra-privacy/src/wiring.rs | H | nein | d1d4a52e |
| crates/contextra-privacy/tests/campaign_cov_privacy_egress.rs | D | nein | ce650578, f85db019 |
| crates/contextra-privacy/tests/campaign_j27_privacy_test.rs | D | ja | ce650578, cc023a25, f85db019 |
| crates/contextra-privacy/tests/egress_fail_closed_and_audit_tests.rs | H | nein | d1d4a52e |
| crates/contextra-privacy/tests/egress_vault_additional_dlp.rs | G | ja | ce650578 |
| crates/contextra-privacy/tests/j19-contextra-privacy_wiring_test.rs | H | nein | ce650578, cc023a25, f85db019 |
| crates/contextra-privacy/tests/j19closure-contextra-privacy_closure_test.rs | H | nein | ce650578, cc023a25 |
| crates/contextra-privacy/tests/rehydrator_scope.rs | H | nein | ce650578, cad222dd |
| crates/contextra-router/AGENTS.md | D | ja | 56e5769a |
| crates/contextra-router/src/arm_registry.rs | D | ja | ce650578, 046a4a46, 20ee467b |
| crates/contextra-router/src/dispatch.rs | B | nein | 0b7ec9d2, 046a4a46, 20ee467b |
| crates/contextra-router/src/lib.rs | B | nein | 0b7ec9d2, 046a4a46 |
| crates/contextra-router/src/profile.rs | B | nein | 0b7ec9d2, ce650578, 046a4a46, 20ee467b |
| crates/contextra-router/src/router/lifecycle.rs | B | nein | ce650578, 1fe11901, 9e6a600b |
| crates/contextra-router/src/router/outcomes.rs | B | nein | 1fe11901, 9e6a600b |
| crates/contextra-router/src/wiring.rs | H | nein | 0b7ec9d2 |
| crates/contextra-router/tests/fc_ts_routing_determinism.rs | G | ja | ce650578 |
| crates/contextra-router/tests/feature_cleanup_validation.rs | G | ja | ce650578 |
| crates/contextra-router/tests/j09-contextra-router-arm-registry-dispatch-p_wiring_test.rs | H | nein | 20ee467b |
| crates/contextra-router/tests/j09closure-contextra-router_wiring_test.rs | H | nein | 046a4a46 |
| crates/contextra-router/tests/j10-contextra-router-router-lifecycle-outcom_test.rs | H | nein | 9e6a600b |
| crates/contextra-router/tests/j10closure-contextra-router_lifecycle_outcomes_test.rs | H | nein | ce650578, 1fe11901 |
| crates/contextra-router/tests/router_wiring_test.rs | H | nein | 0b7ec9d2 |
| crates/contextra-sandbox/AGENTS.md | D | ja | 56e5769a |
| crates/contextra-sandbox/Cargo.toml | B | nein | 12df2e39, cea972df, ef39f9f0, 9b883d83 |
| crates/contextra-sandbox/src/admission.rs | H | nein | 12df2e39, 9b883d83 |
| crates/contextra-sandbox/src/approval.rs | B | nein | 91c344ec, 85ec85f3 |
| crates/contextra-sandbox/src/capabilities.rs | B | nein | 12df2e39, 9b883d83, a9271b68, 52e6ea7e, 91c344ec, 85ec85f3 |
| crates/contextra-sandbox/src/error.rs | B | nein | 9b883d83 |
| crates/contextra-sandbox/src/executor.rs | B | nein | e93d61be, 9b883d83, a9271b68, 52e6ea7e, 2eaab7b4 |
| crates/contextra-sandbox/src/lib.rs | B | nein | 12df2e39, 9b883d83 |
| crates/contextra-sandbox/src/merge.rs | B | nein | e93d61be, 9b883d83, a9271b68 |
| crates/contextra-sandbox/src/wasi.rs | B | nein | 12df2e39, 495b89dc, 2eaab7b4 |
| crates/contextra-sandbox/src/wiring.rs | H | nein | 12df2e39 |
| crates/contextra-sandbox/tests/admission_ed25519_test.rs | H | nein | 12df2e39 |
| crates/contextra-sandbox/tests/approval_workflow_test.rs | H | nein | 12df2e39 |
| crates/contextra-sandbox/tests/crdt_gate_tests.rs | H | nein | e93d61be |
| crates/contextra-sandbox/tests/executor_cache_and_pool_tests.rs | H | nein | e93d61be |
| crates/contextra-sandbox/tests/executor_toxic_module_tests.rs | H | nein | e93d61be |
| crates/contextra-sandbox/tests/j20-contextra-sandbox_approval_test.rs | H | nein | ce650578, 85ec85f3 |
| crates/contextra-sandbox/tests/j20-contextra-sandbox_capabilities_test.rs | H | nein | ce650578, 85ec85f3 |
| crates/contextra-sandbox/tests/j20closure-contextra-sandbox_closure_test.rs | H | nein | 91c344ec |
| crates/contextra-sandbox/tests/module_admission_tests.rs | H | nein | 9b883d83 |
| crates/contextra-sandbox/tests/p05b_wasm_module_policy_rejects_untrusted.rs | H | nein | 9b883d83, a9271b68 |
| crates/contextra-sandbox/tests/pure_merge_operator_capabilities_test.rs | B | nein | 9b883d83 |
| crates/contextra-sandbox/tests/wasm_boundary_tests.rs | B | nein | 9b883d83 |
| crates/contextra-sandbox/tests/wasm_wasi_io.rs | B | nein | 9b883d83 |
| crates/contextra-simd/AGENTS.md | D | ja | 56e5769a |
| crates/contextra-simd/Cargo.toml | B | nein | 00ccf8a4 |
| crates/contextra-simd/benches/simd_bench.rs | B | nein | 5f47bc4a |
| crates/contextra-simd/src/dispatch.rs | D | ja | 00ccf8a4, eaa0df92, 5f47bc4a, 93c6363f, 52e6ea7e |
| crates/contextra-simd/src/kernels/avx2.rs | D | ja | 2a1af3f7, eaa0df92, 5f47bc4a, 93c6363f, 52e6ea7e |
| crates/contextra-simd/src/kernels/avx512.rs | D | ja | 2a1af3f7, eaa0df92, 4ce7aad7, 5f47bc4a, f9e51798, 52e6ea7e |
| crates/contextra-simd/src/kernels/mod.rs | D | ja | eaa0df92, 5f47bc4a |
| crates/contextra-simd/src/kernels/neon.rs | D | ja | 2a1af3f7, eaa0df92, 5f47bc4a, 93c6363f, 52e6ea7e |
| crates/contextra-simd/src/kernels/scalar.rs | B | nein | 51f55cb6 |
| crates/contextra-simd/src/lib.rs | D | ja | 00ccf8a4, 51f55cb6, 6affc64b, 5f47bc4a |
| crates/contextra-simd/src/wiring.rs | H | nein | 00ccf8a4, 51f55cb6, 6affc64b |
| crates/contextra-simd/tests/simd_cross_validation.rs | B | nein | eaa0df92, 5f47bc4a |
| crates/contextra-testkit/AGENTS.md | D | ja | 56e5769a |
| crates/contextra-testkit/Cargo.toml | B | nein | 9b33317a, a2bd6341, a5803c00 |
| crates/contextra-testkit/INVARIANT_MATRIX.md | H | nein | 1a59bf54, a5803c00 |
| crates/contextra-testkit/src/deletion_oracle/marker.rs | H | nein | a778c647, fd31a72e |
| crates/contextra-testkit/src/deletion_oracle/mod.rs | H | nein | a778c647, fd31a72e |
| crates/contextra-testkit/src/deletion_oracle/rawsweep.rs | H | nein | a778c647, 4b7709e6, fd31a72e |
| crates/contextra-testkit/src/deletion_oracle/report.rs | H | nein | a778c647, fd31a72e |
| crates/contextra-testkit/src/dst/engine.rs | H | nein | 4b7709e6, 561dcec9, efeacc60, fd31a72e, 648538d0 |
| crates/contextra-testkit/src/dst/invariants.rs | H | nein | 648538d0 |
| crates/contextra-testkit/src/dst/mod.rs | H | nein | a2bd6341, a5803c00, 561dcec9, efeacc60, 648538d0 |
| crates/contextra-testkit/src/dst/op.rs | H | nein | 648538d0 |
| crates/contextra-testkit/src/dst/p07_runner.rs | H | nein | 5776b652, efeacc60 |
| crates/contextra-testkit/src/dst/p07_sut.rs | H | nein | 5776b652, a5803c00, efeacc60 |
| crates/contextra-testkit/src/dst/rng.rs | H | nein | 11ee512c, a2bd6341, 648538d0 |
| crates/contextra-testkit/src/dst/runner.rs | H | nein | 648538d0 |
| crates/contextra-testkit/src/dst/scenario.rs | H | nein | 648538d0 |
| crates/contextra-testkit/src/erasure_harness/failpoints.rs | H | nein | 9b33317a |
| crates/contextra-testkit/src/erasure_harness/mod.rs | H | nein | 9b33317a |
| crates/contextra-testkit/src/erasure_harness/raw_sweep.rs | H | nein | 9b33317a |
| crates/contextra-testkit/src/erasure_harness/reference_store.rs | H | nein | d5e55fe0, 9b33317a |
| crates/contextra-testkit/src/erasure_harness/tenant_matrix.rs | H | nein | 9b33317a |
| crates/contextra-testkit/src/fault_vfs.rs | B | nein | a5803c00, f396085b, d8d2f628 |
| crates/contextra-testkit/src/lib.rs | B | nein | 9b33317a, 11ee512c, a2bd6341, a5803c00, 561dcec9, efeacc60, fd31a72e, 648538d0 |
| crates/contextra-testkit/src/manual_clock.rs | D | ja | 51e1e765, 11ee512c, a5803c00, fd31a72e, f396085b, d8d2f628, 9f32df43 |
| crates/contextra-testkit/src/reference_model.rs | B | nein | a2bd6341, a5803c00, 561dcec9, efeacc60, f396085b, d8d2f628 |
| crates/contextra-testkit/src/seeded_rng.rs | H | nein | 11ee512c |
| crates/contextra-testkit/src/sim_fs/fs.rs | H | nein | 5776b652, a5803c00, efeacc60 |
| crates/contextra-testkit/src/sim_fs/mod.rs | H | nein | efeacc60 |
| crates/contextra-testkit/src/sim_fs/vfs_mirror.rs | H | nein | efeacc60 |
| crates/contextra-testkit/src/wiring.rs | H | nein | 5776b652, a5803c00 |
| crates/contextra-testkit/tests/canary_verification_test.rs | H | nein | 4b7709e6, 561dcec9, efeacc60, fd31a72e, e6fe4332 |
| crates/contextra-testkit/tests/deletion_oracle_selftest.rs | H | nein | 4b7709e6, fd31a72e |
| crates/contextra-testkit/tests/dst_scenarios.rs | H | nein | 561dcec9, efeacc60, 648538d0 |
| crates/contextra-testkit/tests/dst_smoke.rs | H | nein | fd31a72e, 648538d0 |
| crates/contextra-testkit/tests/erasure_harness_selftest.rs | H | nein | 9b33317a |
| crates/contextra-testkit/tests/j37-contextra-testkit_fault_vfs_test.rs | H | nein | 561dcec9, efeacc60, ce650578, d8d2f628 |
| crates/contextra-testkit/tests/j37-contextra-testkit_manual_clock_test.rs | H | nein | ce650578, d8d2f628 |
| crates/contextra-testkit/tests/j37-contextra-testkit_reference_model_test.rs | H | nein | ce650578, d8d2f628 |
| crates/contextra-testkit/tests/j37closure-contextra-testkit_closure_tests.rs | H | nein | a5803c00, fd31a72e, ce650578, f396085b |
| crates/contextra-testkit/tests/p07_dst_runner_test.rs | H | nein | 35c6bee9, efeacc60 |
| crates/contextra-testkit/tests/p07_meta_tests.rs | H | nein | 35c6bee9, efeacc60 |
| crates/contextra-testkit/tests/p07_sim_fs_test.rs | H | nein | efeacc60 |
| crates/contextra-testkit/tests/regressions/seeds.txt | H | nein | efeacc60 |
| crates/contextra-testkit/tests/store_wal_crash_harness.rs | H | nein | a5803c00 |
| crates/contextra-testkit/tests/support/canary.rs | H | nein | fd31a72e, e6fe4332 |
| crates/contextra-wire/AGENTS.md | D | ja | 56e5769a |
| crates/contextra-wire/build.rs | D | ja | 35c6bee9, 1ed3a86d, 2b3e7c8f, 4fb1e45b, 6af1d79a |
| crates/contextra-wire/src/contextra_generated.rs | B | nein | bbb605cd, 83b2cc86, 51f55cb6, 35c6bee9, 59420f7d, 4b7709e6, 2b3e7c8f, 8cedf9fb, 525ecd80, bd0a2525, a2b063dc, 240f4f7c, b270bc2e, d6550563, 7e1d65ba, cea972df, ae1fa1b5, 5f5251e9, 67c7ca58, 4fb1e45b, 19d12ae4, 7e5ef3e6, 495b89dc, 2eaab7b4, 6af1d79a, 0ed9343b, adf205ab, 0960e83b, e31ed5c4 |
| crates/contextra-wire/src/jsonrpc.rs | B | nein | 5f47bc4a |
| crates/contextra-wire/src/lib.rs | B | nein | 6affc64b, 2d5c5b0d, 5f47bc4a, 56e5769a |
| crates/contextra-wire/src/wiring.rs | H | nein | 51f55cb6, 6affc64b |
| crates/contextra-wire/tests/ipc_tests.rs | B | nein | 2b3e7c8f, 0ced841b |
| crates/contextra-wire/tests/j27-contextra-wire_search_response.rs | H (gelöscht/umbenannt) | nein | 0ed9343b, 0960e83b |
| crates/contextra-wire/tests/j27closure-contextra-wire_search_response.rs | H | nein | 2b3e7c8f, 56e5769a, 2eaab7b4, ce650578, 0ed9343b |
