# Triage-Manifest T5: Infrastruktur, Tooling, Doku und CI

**Stand (`origin/main`):** `a26b9f37`
**Anzahl Commits:** `339`
**Klassen-Zähler:**
- `DECISION`: 9
- `PORT-FEAT`: 48
- `PORT-FIX`: 16
- `PORT-WIRING`: 10
- `SKIP-BOT`: 17
- `SKIP-DOC`: 237
- `SKIP-SUPERSEDED`: 2

---

## Commits

| sha | Datum | Titel | Dateien | prod/test/doku-Zeilen | Klasse | Begründung | Abhängigkeit | in main schon vorhanden | Konfliktrisiko |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| 59dacde0 | 2026-10-15 | docs(parking): add INVENTORY.md protocol (W4-01) | `docs/parking/INVENTORY.md` | +0/-0 prod, +0/-0 test, +231/-0 doku | SKIP-DOC | Dokumentation / Inventar zu Parkplatz-Crates. | keine | nein | niedrig |
| 6153a394 | 2026-10-15 | [Entkopplung contextra-bench] Decouple benchmarks/contextra-bench | `benchmarks/contextra-bench/AGENTS.md`<br>`benchmarks/contextra-bench/Cargo.toml`<br>`benchmarks/contextra-bench/README.md`<br>*(+8 weitere)* | +0/-0 prod, +56/-522 test, +0/-0 doku | SKIP-SUPERSEDED | Parkplatz-Entkopplung oder Revert/Entfernung; im Spender-Endstand verworfen oder superseded. | keine | teilweise | hoch |
| e40902ac | 2026-10-15 | Shell-Commit | `docs/parking/ENTFERNUNG.md`<br>`docs/parking/INVENTORY.md` | +0/-0 prod, +0/-0 test, +74/-93 doku | SKIP-DOC | Dokumentations- oder Sonstige Anpassung (`docs/parking/ENTFERNUNG.md`). | keine | nein | niedrig |
| 1a028782 | 2026-10-15 | [SPEC-v4.1] Entfernen statt Einfrieren | `docs/spec/GESAMTSPEZIFIKATION-v4.md` | +0/-0 prod, +0/-0 test, +1843/-0 doku | DECISION | Änderung an der Spezifikation (`docs/spec/**`); Entscheidung des Maintainers zur Übernahme erforderlich. | keine | nein | niedrig |
| 2c9e7d3d | 2026-10-15 | docs: create parking crates inventory (W1-J01) | `docs/parking/INVENTORY.md` | +0/-0 prod, +0/-0 test, +93/-0 doku | SKIP-DOC | Dokumentation / Inventar zu Parkplatz-Crates. | keine | nein | niedrig |
| 6e1c6d1e | 2026-10-15 | PARKING-ADR FOKUSSIERUNG und große UMSTRUKTurIERUNG | `docs/decisions/ADR-PARK-01.md` | +0/-0 prod, +0/-0 test, +32/-0 doku | SKIP-SUPERSEDED | Parkplatz-Entkopplung oder Revert/Entfernung; im Spender-Endstand verworfen oder superseded. | keine | nein | niedrig |
| 331c38c5 | 2026-10-15 | feat(kv): default TombstoneOnly and remove v3 deletion proof creation | `docs/spec/CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md`<br>`docs/verification/INV_MATRIX.md` | +0/-0 prod, +0/-0 test, +4/-4 doku | DECISION | Änderung an der Spezifikation (`docs/spec/**`); Entscheidung des Maintainers zur Übernahme erforderlich. | keine | teilweise | hoch |
| 51e1e765 | 2026-10-15 | feat(ports): redefine Erasure ports per §5.4 and §5.5 (J-07) | `.jules/tasks/J-07.toml` | +0/-0 prod, +0/-0 test, +28/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/J-07.toml`). | keine | nein | niedrig |
| c859626d | 2026-10-15 | feat(xtask): align capabilities.toml and implement G-04 & G-21 gates | `docs/SOURCE_OF_TRUTH.md`<br>`xtask/src/check_exception_ledger.rs`<br>`xtask/src/check_ring_capabilities_consistency.rs`<br>*(+4 weitere)* | +366/-1 prod, +0/-0 test, +4/-36 doku | PORT-WIRING | Infrastruktur-Verdrahtung oder Refactoring (`docs/SOURCE_OF_TRUTH.md`). | xtask | teilweise | hoch |
| d929ce62 | 2026-10-15 | feat(xtask): implement generate-core-mapping command | `docs/refactor/core-mapping.json`<br>`docs/refactor/core-mapping.md`<br>`xtask/src/harness/generate_core_mapping.rs` | +6/-6 prod, +0/-0 test, +16/-16 doku | PORT-FEAT | Neue Infrastruktur-/Xtask-Funktion oder Quality-Gate (`docs/refactor/core-mapping.json`). | xtask | nein | niedrig |
| 090339f8 | 2026-10-15 | feat(xtask): implement generate-core-mapping command for core removal | `docs/refactor/core-mapping.json`<br>`docs/refactor/core-mapping.md`<br>`xtask/src/harness/generate_core_mapping.rs` | +6/-6 prod, +0/-0 test, +16/-16 doku | PORT-FEAT | Neue Infrastruktur-/Xtask-Funktion oder Quality-Gate (`docs/refactor/core-mapping.json`). | xtask | nein | niedrig |
| 11ee512c | 2026-10-15 | feat(ports): complete time, rng, and id_gen ports and configure G-09 clippy gate | `docs/gaps/pub_fn_ohne_aufrufer.csv`<br>`docs/gaps/undokumentiert_streng.csv`<br>`xtask/build.rs`<br>*(+3 weitere)* | +172/-0 prod, +0/-0 test, +45/-12 doku | PORT-FEAT | Neue Infrastruktur-/Xtask-Funktion oder Quality-Gate (`docs/gaps/pub_fn_ohne_aufrufer.csv`). | xtask | teilweise | hoch |
| b1998340 | 2026-10-15 | feat(compliance): implement gate G-19 check-claims and remediate claims | `docs/decisions/claims/claims-forbidden.toml`<br>`docs/deletion/LEGAL_MATRIX_ERASURE.md`<br>`xtask/src/check_claims.rs`<br>*(+3 weitere)* | +296/-0 prod, +0/-0 test, +57/-3 doku | PORT-FEAT | Neue Infrastruktur-/Xtask-Funktion oder Quality-Gate (`docs/decisions/claims/claims-forbidden.toml`). | xtask | teilweise | hoch |
| 49036c0b | 2026-10-15 | feat(xtask): add generate-core-mapping command for core removal | `docs/refactor/core-mapping.json`<br>`docs/refactor/core-mapping.md` | +0/-0 prod, +0/-0 test, +1648/-145 doku | PORT-FEAT | Neue Infrastruktur-/Xtask-Funktion oder Quality-Gate (`docs/refactor/core-mapping.json`). | xtask | nein | niedrig |
| bf089c57 | 2026-10-15 | feat(xtask): implement generate-core-mapping command for contextra-core removal preparation | `docs/refactor/core-mapping.json`<br>`docs/refactor/core-mapping.md`<br>`xtask/registry.toml`<br>*(+2 weitere)* | +414/-0 prod, +63/-0 test, +530/-0 doku | PORT-FEAT | Neue Infrastruktur-/Xtask-Funktion oder Quality-Gate (`docs/refactor/core-mapping.json`). | xtask | teilweise | hoch |
| b5796772 | 2026-10-15 | feat(xtask): maintain gate registry and implement missing gates G-02, G-11, G-12 | `xtask/registry.toml`<br>`xtask/src/harness/check_no_downcast.rs`<br>`xtask/src/harness/check_port_features.rs`<br>*(+1 weitere)* | +297/-0 prod, +0/-0 test, +0/-0 doku | PORT-FEAT | Neue Infrastruktur-/Xtask-Funktion oder Quality-Gate (`xtask/registry.toml`). | xtask | teilweise | hoch |
| fceeae4b | 2026-10-14 | docs: konsolidierung planungsdokumente & adr-register ADR-E01..E37 | `docs/14-10-26/specs/EP1_CONTEXTRA_MEMORY_SPEC.md`<br>`docs/14-10-26/specs/EP2_CONTEXTRA_PROOF_SPEC.md`<br>`docs/14-10-26/specs/EP6_CONTEXTRA_GUARD_SANDBOX_SPEC.md`<br>*(+84 weitere)* | +0/-0 prod, +13/-0 test, +684/-7842 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | mittel |
| 25fe0dfb | 2026-10-14 | docs: update governance files AGENTS.md, VETOES.md and ADR-T drafts | `docs/decisions/ADR-T01-P29-kein-globaler-veraenderlicher-zustand.md`<br>`docs/decisions/ADR-T02-INV-KMS-1-kms-provider-verdrahtung.md`<br>`docs/decisions/ADR-T03-INV-DELETION-WIRE-1-loeschpfad-verdrahtung.md`<br>*(+4 weitere)* | +0/-0 prod, +0/-0 test, +92/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | teilweise | mittel |
| bbb605cd | 2026-10-14 | feat(testkit): update test infrastructure for deletion testing (SP-08) | `docs/generated/capability-markers.md` | +0/-0 prod, +0/-0 test, +6/-5 doku | PORT-FEAT | Funktionale Erweiterung im Pfadebereich (`docs/generated/capability-markers.md`). | keine | teilweise | niedrig |
| b8ac557d | 2026-10-14 | docs: baseline measurements and report in docs/baseline/2026-10.md | `docs/baseline/2026-10.md` | +0/-0 prod, +0/-0 test, +150/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| 5d73ffd1 | 2026-10-14 | docs: align rules/ with capabilities.toml for unsafe islands | `rules/loom-miri.md`<br>`rules/simd_safety.md`<br>`rules/tag_taxonomy.md` | +0/-0 prod, +0/-0 test, +4/-4 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | teilweise | niedrig |
| f00bb2af | 2026-10-14 | Shell-Commit | `docs/{ => 14-10-26/specs}/IMPLEMENTIERUNGSSPEZIFIKATION_ENDPRODUKTE_DISJUNKTIV.md`<br>`docs/{ => 14-10-26/specs}/SPEZIFIKATION_EP1_CONTEXTRA_MEMORY.md`<br>`docs/{ => 14-10-26/specs}/SPEZIFIKATION_EP2_CONTEXTRA_PROOF.md`<br>*(+23 weitere)* | +0/-0 prod, +0/-0 test, +5873/-0 doku | SKIP-DOC | Dokumentations- oder Sonstige Anpassung (`docs/{ => 14-10-26/specs}/IMPLEMENTIERUNGSSPEZIFIKATION_ENDPRODUKTE_DISJUNKTIV.md`). | keine | nein | mittel |
| 426dfead | 2026-10-14 | Shell-Commit | `docs/14-10-26/SPEZIFIKATION_EP1_CONTEXTRA_MEMORY.md`<br>`docs/14-10-26/SPEZIFIKATION_EP2_CONTEXTRA_PROOF.md`<br>`docs/14-10-26/SPEZIFIKATION_EP3_CONTEXTRA_RETRIEVAL_PLUS.md`<br>*(+23 weitere)* | +0/-0 prod, +0/-0 test, +9960/-875 doku | SKIP-DOC | Dokumentations- oder Sonstige Anpassung (`docs/14-10-26/SPEZIFIKATION_EP1_CONTEXTRA_MEMORY.md`). | keine | nein | mittel |
| ce52bda8 | 2026-10-14 | Shell-Commit | `docs/14-10-26/ARCHITEKTUR_MASTERPLAN_v2.md`<br>`docs/14-10-26/DISJUNKTIVE_MIKRO_IMPLEMENTIERUNGSSPEZIFIKATION_v1.md`<br>`docs/14-10-26/IMPLEMENTIERUNGSSPEZIFIKATION_ENDPRODUKTE_DISJUNKTIV.md`<br>*(+8 weitere)* | +0/-0 prod, +0/-0 test, +4434/-0 doku | SKIP-DOC | Dokumentations- oder Sonstige Anpassung (`docs/14-10-26/ARCHITEKTUR_MASTERPLAN_v2.md`). | keine | nein | mittel |
| b5037b46 | 2026-10-14 | Shell-Commit | `docs/{ => 14-10-26}/ADVANCED_ALGORITHMS_SPEC_v1.md`<br>`docs/{ => 14-10-26}/ARCHITECTURE_PLAN_v1.md`<br>`docs/{ => 14-10-26}/ARCHITEKTUR_MASTERPLAN_v2.md`<br>*(+15 weitere)* | +0/-1950 prod, +0/-0 test, +0/-2391 doku | SKIP-DOC | Dokumentations- oder Sonstige Anpassung (`docs/{ => 14-10-26}/ADVANCED_ALGORITHMS_SPEC_v1.md`). | keine | nein | mittel |
| 0c315cbf | 2026-10-14 | Shell-Commit | `docs/ARCHITECTURE_PLAN_v1.md`<br>`docs/ARCHITEKTUR_MASTERPLAN_v2.md`<br>`docs/analysis/AUDIT_GESAMTSYNTHESE_2026-10-14.md`<br>*(+1 weitere)* | +0/-0 prod, +0/-0 test, +3052/-498 doku | DECISION | Änderung an der Spezifikation (`docs/spec/**`); Entscheidung des Maintainers zur Übernahme erforderlich. | keine | nein | niedrig |
| 899d09ac | 2026-10-14 | docs: add micro-fine-grained algorithmic specification for SOTA 2026 features | `docs/spec/SOTA_ALGORITHMIC_FEINSPEZIFIKATION_2026.md` | +0/-0 prod, +0/-0 test, +676/-0 doku | DECISION | Änderung an der Spezifikation (`docs/spec/**`); Entscheidung des Maintainers zur Übernahme erforderlich. | keine | nein | niedrig |
| 230372f0 | 2026-10-14 | Shell-Commit | `docs/ADVANCED_ALGORITHMS_SPEC_v1.md`<br>`docs/ARCHITECTURE_PLAN_v1.md`<br>`rules/tag_taxonomy.md`<br>*(+3 weitere)* | +144/-40 prod, +0/-0 test, +2017/-0 doku | PORT-FEAT | Neue Infrastruktur-/Xtask-Funktion oder Quality-Gate (`docs/ADVANCED_ALGORITHMS_SPEC_v1.md`). | xtask | teilweise | hoch |
| 4a88c425 | 2026-10-14 | Shell-Commit | `.jules/JULES_CONTEXT.md`<br>`.jules/QUICK_REFERENCE.md`<br>`.jules/SESSION_BOOTSTRAP.md`<br>*(+8 weitere)* | +0/-0 prod, +0/-0 test, +2309/-853 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/JULES_CONTEXT.md`). | keine | teilweise | hoch |
| f375ad8a | 2026-10-14 | Shell-Commit | `docs/ARCHITECTURE_PLAN_v1.md`<br>`docs/ARCHITEKTUR_OPTIMIERUNG_v1.md` | +0/-0 prod, +0/-0 test, +1744/-0 doku | SKIP-DOC | Dokumentations- oder Sonstige Anpassung (`docs/ARCHITECTURE_PLAN_v1.md`). | keine | nein | niedrig |
| 561c46fb | 2026-10-14 | Shell-Commit | `"docs/14-10-26/Contextra \342\200\223 Gesamtanalyse Architektur und L\303\266schnachweis (HEAD 16764583).md"`<br>`docs/14-10-26/Contextra_Algorithmen_Spezifikation.md`<br>`docs/14-10-26/Contextra_Endprodukt_Plan_v7.md`<br>*(+7 weitere)* | +1950/-0 prod, +0/-0 test, +4341/-0 doku | SKIP-DOC | Dokumentations- oder Sonstige Anpassung (`"docs/14-10-26/Contextra \342\200\223 Gesamtanalyse Architektur und L\303\266schnachweis (HEAD 16764583).md"`). | keine | nein | mittel |
| 83b2cc86 | 2026-10-14 | Shell-Commit | `benchmarks/contextra-bench/src/main.rs` | +0/-0 prod, +6/-0 test, +0/-0 doku | PORT-FEAT | Benchmark-Testsuite oder Performance-Parität (`benchmarks/contextra-bench/src/main.rs`). | benchmarks | teilweise | hoch |
| f7be670e | 2026-10-14 | docs: add Google-Jules autonomous agent & VM operational specification | `docs/JULES_AUTONOMOUS_AGENT_SPECIFICATION.md`<br>`docs/SOURCE_OF_TRUTH.md`<br>`docs/spec/22-jules-autonomous-agent-specification.md`<br>*(+1 weitere)* | +0/-0 prod, +0/-0 test, +682/-1 doku | DECISION | Änderung an der Spezifikation (`docs/spec/**`); Entscheidung des Maintainers zur Übernahme erforderlich. | keine | teilweise | hoch |
| 92315a20 | 2026-10-14 | feat(engine): bind DpiRestoreFilter to RevocationLog and ErasureLedger (Task B10) | `.jules/tasks/T-2026-LA12.toml`<br>`.jules/tasks/T-2026-LB10.toml`<br>`docs/analysis/AUDIT_governance_harness_DEEP_2026-10-13.md`<br>*(+2 weitere)* | +0/-0 prod, +6/-0 test, +102/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-LA12.toml`). | keine | teilweise | niedrig |
| 9230ec3f | 2026-10-14 | feat(store): wire CostBasedAdaptivePlanner, ErasureLedger, and SeqBitTombstone | `docs/analysis/AUDIT_contextra-store_DEEP_2026-10-14.md` | +0/-0 prod, +0/-0 test, +98/-0 doku | PORT-FEAT | Funktionale Erweiterung im Pfadebereich (`docs/analysis/AUDIT_contextra-store_DEEP_2026-10-14.md`). | keine | nein | niedrig |
| 8d738993 | 2026-10-14 | refactor(mvcc): consolidate SnapshotFloor trait and eliminate u64::MAX snapshot usage | `docs/analysis/AUDIT_contextra-mvcc_DEEP_2026-10-14.md` | +0/-0 prod, +0/-0 test, +84/-0 doku | PORT-WIRING | Refactoring oder Infrastruktur-Anpassung (`docs/analysis/AUDIT_contextra-mvcc_DEEP_2026-10-14.md`). | keine | nein | niedrig |
| aa2632bd | 2026-10-14 | docs(crypto): perform deep audit of contextra-crypto module | `docs/analysis/AUDIT_contextra-crypto_DEEP_2026-10-13.md` | +0/-0 prod, +0/-0 test, +16/-61 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| 15b361b6 | 2026-10-14 | docs: add deep audit report for unsafe islands | `docs/analysis/AUDIT_unsafe-islands_DEEP_2026-10-14.md` | +0/-0 prod, +0/-0 test, +183/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| f3731763 | 2026-10-14 | fix(store): handle DisposePolicy::Disabled and complete WAL audit report | `docs/analysis/AUDIT_wal_durable_fs_DEEP_2026-10-13.md` | +0/-0 prod, +0/-0 test, +2/-2 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| 8fb718bb | 2026-10-14 | docs: Spiegelung der Löschnachweis-Spezifikation und Verifier-Spec (B21) | `.jules/tasks/T-2026-LB21.toml`<br>`docs/spec/CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md`<br>`docs/verification/VERIFIER_SPEC.md` | +0/-0 prod, +0/-0 test, +129/-18 doku | DECISION | Änderung an der Spezifikation (`docs/spec/**`); Entscheidung des Maintainers zur Übernahme erforderlich. | keine | teilweise | hoch |
| a9cf262b | 2026-10-14 | docs(vector): add comprehensive deep audit report for contextra-vector | `docs/analysis/AUDIT_contextra-vector_DEEP_2026-10-14.md` | +0/-0 prod, +0/-0 test, +134/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| f1007346 | 2026-10-14 | test(crypto): Ed25519 strict edge-case test vectors and cross-impl checks (B19) | `.jules/tasks/T-2026-LB19.toml` | +0/-0 prod, +0/-0 test, +44/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-LB19.toml`). | keine | nein | niedrig |
| a11f9e57 | 2026-10-14 | feat(crypto): canon, claim_texts & exclusions for deletion proof (B22) | `.jules/tasks/T-2026-LB22.toml` | +0/-0 prod, +0/-0 test, +43/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-LB22.toml`). | keine | nein | niedrig |
| c67c0c8b | 2026-10-14 | docs(deletion): Add legal erasure matrix and operations baseline (B20) | `.jules/tasks/T-2026-LB20.toml`<br>`docs/deletion/LEGAL_MATRIX_ERASURE.md`<br>`docs/deletion/OPERATIONS_BASELINE_ERASURE.md` | +0/-0 prod, +0/-0 test, +116/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | teilweise | niedrig |
| 744ffa44 | 2026-10-14 | feat(crypto): RevocationLog generation tracking and rollback detection | `.jules/tasks/T-2026-LB14.toml` | +0/-0 prod, +0/-0 test, +37/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-LB14.toml`). | keine | nein | niedrig |
| 687bfce8 | 2026-10-14 | feat(engine): Stage 2 Purge-Driver real hashes and tenant verification (B09) | `.jules/tasks/T-2026-LA12.toml`<br>`.jules/tasks/T-2026-LB09.toml`<br>`docs/analysis/AUDIT_governance_harness_DEEP_2026-10-13.md`<br>*(+2 weitere)* | +0/-0 prod, +6/-0 test, +95/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-LA12.toml`). | keine | teilweise | niedrig |
| a778c647 | 2026-10-14 | feat(testkit): harden deletion oracle raw_sweep and add per-layer canaries | `.jules/tasks/T-2026-LB15.toml` | +0/-0 prod, +0/-0 test, +30/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-LB15.toml`). | keine | nein | niedrig |
| 47f608aa | 2026-10-14 | feat(sys,engine): implement runtime hardening and observed env status (B16) | `.jules/tasks/T-2026-LB16.toml` | +0/-0 prod, +0/-0 test, +38/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-LB16.toml`). | keine | nein | niedrig |
| 4b8c4545 | 2026-10-14 | feat(crypto): enforce fail-closed default policy required layers and witness validation (B02/F14) | `.jules/tasks/T-2026-LB02.toml` | +0/-0 prod, +0/-0 test, +44/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-LB02.toml`). | keine | nein | niedrig |
| f6f340fa | 2026-10-14 | feat(crypto): v4 deletion proof TLV scope and verify_strict (B03) | `.jules/tasks/T-2026-LB03.toml` | +0/-0 prod, +0/-0 test, +17/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-LB03.toml`). | keine | nein | niedrig |
| d42aaa80 | 2026-10-14 | feat(mcp): enforce proof-key quality rules and support key files (F3) | `.jules/tasks/T-2026-LB12.toml` | +0/-0 prod, +0/-0 test, +41/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-LB12.toml`). | keine | nein | niedrig |
| e54ddb3a | 2026-10-14 | feat(crypto): LayerWitness forgery prevention and verification (F7) | `.jules/tasks/T-2026-LB01.toml` | +0/-0 prod, +0/-0 test, +17/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-LB01.toml`). | keine | nein | niedrig |
| cf09b17a | 2026-10-14 | feat(checkpoint): Kopien als Geltungsbereich: CopyRegistry, Hardlinks, Observer (B11) | `.jules/tasks/T-2026-LB11.toml` | +0/-0 prod, +0/-0 test, +51/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-LB11.toml`). | keine | nein | niedrig |
| 82d7a80e | 2026-10-14 | feat(crypto): KV-Shredding key copies zeroizing and two-phase revocation (B13) | `.jules/tasks/T-2026-LB13.toml` | +0/-0 prod, +0/-0 test, +17/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-LB13.toml`). | keine | nein | niedrig |
| 2a37aa40 | 2026-10-14 | [B08] Implement real scans for relations and text deletion witnesses | `.jules/tasks/T-2026-LB08.toml` | +0/-0 prod, +0/-0 test, +36/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-LB08.toml`). | keine | nein | niedrig |
| a6b23465 | 2026-10-14 | feat(crypto): CSPRNG key source, rotation, UntrustedKeyList (B04) | `.jules/tasks/T-2026-LB04.toml` | +0/-0 prod, +0/-0 test, +37/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-LB04.toml`). | keine | nein | niedrig |
| 7ccbfc06 | 2026-10-14 | feat(crypto): RFC 3161 timestamp anchor and RFC 9162 transparency log (B06) | `.jules/tasks/T-2026-LB06.toml` | +0/-0 prod, +0/-0 test, +46/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-LB06.toml`). | keine | nein | niedrig |
| 55cd8b70 | 2026-10-14 | feat(crypto): verify_strict and rollback anchor check in audit chain | `.jules/tasks/T-2026-LB05.toml` | +0/-0 prod, +0/-0 test, +26/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| cc6027e4 | 2026-10-14 | feat(audit-export): add audit scope limits, dual timestamps, and updated norm references | `.jules/tasks/T-2026-LB07.toml` | +0/-0 prod, +0/-0 test, +44/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| 871efc73 | 2026-10-14 | Fix benchmark gate CLI flags, confidence intervals, and open-loop latency tracking | `benchmarks/contextra-bench/src/compare.rs`<br>`benchmarks/contextra-bench/src/locomo.rs`<br>`benchmarks/contextra-bench/src/long_mem_eval.rs`<br>*(+3 weitere)* | +0/-0 prod, +131/-13 test, +18/-3 doku | PORT-FIX | Fehlerkorrektur im Infra-/Xtask-Bereich (`benchmarks/contextra-bench/src/compare.rs`). | xtask | teilweise | mittel |
| e9220243 | 2026-10-14 | docs(governance): status-blocks in audit reports and wiring policy | `docs/analysis/AUDIT_governance_harness_DEEP_2026-10-13.md`<br>`docs/analysis/AUDIT_testkit-fuzz-loom-governance_DEEP_2026-10-13.md`<br>`docs/decisions/README.md` | +0/-0 prod, +6/-0 test, +16/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | teilweise | niedrig |
| c19a81f2 | 2026-10-13 | feat(vector): implement compiler-resistant HNSW zeroization | `.jules/tasks/T-2026-LA12.toml` | +0/-0 prod, +0/-0 test, +42/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-LA12.toml`). | keine | nein | niedrig |
| 38e4277f | 2026-10-13 | feat(engine): separate proof signing keys from public db_instance_id (F1) | `.jules/tasks/T-2026-LA01.toml` | +0/-0 prod, +0/-0 test, +40/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-LA01.toml`). | keine | nein | niedrig |
| ba4d0a72 | 2026-10-13 | feat(store): LSM-Zeuge aus realem Manifest-Zustand (A05) | `.jules/tasks/T-2026-LA05.toml` | +0/-0 prod, +0/-0 test, +17/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-LA05.toml`). | keine | nein | niedrig |
| 4e4d6e91 | 2026-10-13 | feat(sys): mlock-Verhalten und Vault-Schutzstatus ehrlich ausweisen (N11) | `.jules/tasks/T-2026-LA17.toml` | +0/-0 prod, +0/-0 test, +47/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-LA17.toml`). | keine | nein | niedrig |
| 51c4d75b | 2026-10-13 | fix(contextra-py): align drop_collection docstring, proof status & key zeroization (N10) | `.jules/tasks/T-2026-LA16.toml` | +0/-0 prod, +0/-0 test, +44/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-LA16.toml`). | keine | nein | niedrig |
| ad061315 | 2026-10-13 | feat(engine): evaluate DocEraseOptions and enforce ErasureLedger step in erase_document | `.jules/tasks/T-2026-LA03.toml` | +0/-0 prod, +0/-0 test, +26/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-LA03.toml`). | keine | nein | niedrig |
| a4517868 | 2026-10-13 | feat(engine): drop_collection purge_keys wiring and physical layer claims | `.jules/tasks/T-2026-LA02.toml` | +0/-0 prod, +0/-0 test, +17/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-LA02.toml`). | keine | nein | niedrig |
| 3b79bb15 | 2026-10-13 | feat(crypto): v3 deletion proof domain separation and legacy tag docs (F7) | `.jules/tasks/T-2026-LA06.toml` | +0/-0 prod, +0/-0 test, +17/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-LA06.toml`). | keine | nein | niedrig |
| 6493e6e3 | 2026-10-13 | feat(mcp): explicit proof contract for contextra_delete and drop_collection | `.jules/tasks/T-2026-LA04.toml` | +0/-0 prod, +0/-0 test, +17/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-LA04.toml`). | keine | nein | niedrig |
| 2abe7f78 | 2026-10-13 | fix(store): propagate errors in PurgeOrchestrator | `.jules/tasks/T-2026-LA09.toml` | +0/-0 prod, +0/-0 test, +36/-0 doku | PORT-FIX | Fehlerkorrektur im Infra-/Xtask-Bereich (`.jules/tasks/T-2026-LA09.toml`). | xtask | nein | niedrig |
| 3d4edf43 | 2026-10-13 | feat(crypto): enforce fail-closed KMS revocation (A13/N6) | `.jules/tasks/T-2026-LA13.toml` | +0/-0 prod, +0/-0 test, +17/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-LA13.toml`). | keine | nein | niedrig |
| 57960442 | 2026-10-13 | feat(store): enforce DisposePolicy and record residual risk on scrub refusal | `.jules/tasks/T-2026-LA10.toml` | +0/-0 prod, +0/-0 test, +46/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-LA10.toml`). | keine | nein | niedrig |
| ca7f7ad6 | 2026-10-13 | feat(store): implement compaction-based key physical purge in purge_keys | `.jules/tasks/T-2026-LA08.toml` | +0/-0 prod, +0/-0 test, +51/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-LA08.toml`). | keine | nein | niedrig |
| 9d7a3686 | 2026-10-13 | feat(checkpoint): Checkpoint-Restore Anti-Resurrection filter (N9) | `.jules/tasks/T-2026-LA15.toml` | +0/-0 prod, +0/-0 test, +17/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-LA15.toml`). | keine | nein | niedrig |
| 07f5bce0 | 2026-10-13 | fix(store): WAL-Scan haerten: Lesefehler, Klassifikation, Stichtag | `.jules/tasks/T-2026-LA14.toml` | +0/-0 prod, +0/-0 test, +17/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-LA14.toml`). | keine | nein | niedrig |
| b049a8df | 2026-10-13 | feat(durable-fs): complete scrub refusal matrix and readback zero-verification | `.jules/tasks/T-2026-LA18.toml` | +0/-0 prod, +0/-0 test, +17/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-LA18.toml`). | keine | nein | niedrig |
| b1b217b2 | 2026-10-13 | crypto: enforce domain separation tag for deletion proof v3 | `.jules/tasks/T-2026-LA06.toml` | +0/-0 prod, +0/-0 test, +17/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-LA06.toml`). | keine | nein | niedrig |
| 5df265a9 | 2026-10-13 | feat(store): fail-closed disposal and DisposePolicy | `.jules/tasks/T-2026-LA10.toml` | +0/-0 prod, +0/-0 test, +44/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-LA10.toml`). | keine | nein | niedrig |
| 3dc67417 | 2026-10-13 | fix(governance): align capabilities manifest, rules, and task cards | `.jules/tasks/W-ARCH-01.toml`<br>`.jules/tasks/W-COMPLIANCE-01.toml`<br>`.jules/tasks/W-CR-01.toml`<br>*(+10 weitere)* | +0/-0 prod, +0/-0 test, +225/-3 doku | PORT-FEAT | Jules Harness Steuerung und Task Cards (`.jules/tasks/W-ARCH-01.toml`). | .jules | teilweise | mittel |
| 42cf7835 | 2026-10-13 | feat(rank,infer-onnx): wire RerankWiring and RerankStage | `.jules/tasks/W2-03.toml` | +0/-0 prod, +0/-0 test, +22/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/W2-03.toml`). | keine | nein | niedrig |
| e79f5da3 | 2026-10-13 | feat(store,durable-fs): resolve WAL truncation TODO, fix PendingDeleteJournal error handling and P28 RNG | `.jules/tasks/T-2026-9250.toml` | +0/-0 prod, +0/-0 test, +38/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-9250.toml`). | keine | nein | niedrig |
| 60c4aaae | 2026-10-13 | docs(analysis): add deep audit report for unsafe islands | `docs/analysis/AUDIT_unsafe-islands_DEEP_2026-10-13.md` | +0/-0 prod, +0/-0 test, +201/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| 3f6e2227 | 2026-10-13 | docs: Add deep audit report for kvcache, checkpoint, and infer crates | `docs/analysis/AUDIT_contextra-deep_DEEP_2026-10-13.md` | +0/-0 prod, +0/-0 test, +291/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| 3db7585d | 2026-10-13 | docs(audit): deep audit analysis for privacy, license, and compliance crates | `docs/analysis/AUDIT_privacy-license-compliance_DEEP_2026-10-13.md` | +0/-0 prod, +0/-0 test, +269/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| 68b76940 | 2026-10-13 | docs(audit): add deep benchmark audit report 2026-10-13 | `docs/analysis/AUDIT_benchmarks_DEEP_2026-10-13.md` | +0/-0 prod, +284/-0 test, +0/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| 40d88863 | 2026-10-13 | docs: Add deep-dive audit report for governance and harness | `docs/analysis/AUDIT_governance_harness_DEEP_2026-10-13.md` | +0/-0 prod, +0/-0 test, +250/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| 7b8f0617 | 2026-10-13 | docs(vector): add HNSW vector index deep audit report | `docs/analysis/AUDIT_contextra-vector_DEEP_2026-10-13.md` | +0/-0 prod, +0/-0 test, +246/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| 7fa590b6 | 2026-10-13 | docs: add deep audit report for contextra-sandbox and WASM merge operators | `docs/analysis/AUDIT_contextra-sandbox_DEEP_2026-10-13.md` | +0/-0 prod, +0/-0 test, +251/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| d7c8fb64 | 2026-10-13 | docs(audit): add deep audit report for contextra-mvcc and commit path | `docs/analysis/AUDIT_contextra-mvcc_DEEP_2026-10-13.md` | +0/-0 prod, +0/-0 test, +228/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| df3fd25f | 2026-10-13 | docs(audit): add deep audit report for WAL and durable-fs | `docs/analysis/AUDIT_wal_durable_fs_DEEP_2026-10-13.md` | +0/-0 prod, +0/-0 test, +226/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| 6892c08f | 2026-10-13 | docs(analysis): add deep audit report for text, graph, and rank crates | `docs/analysis/AUDIT_text-graph-rank_DEEP_2026-10-13.md` | +0/-0 prod, +0/-0 test, +480/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| bd576424 | 2026-10-13 | feat(mcp): resolve tenant hardcoding, cfg-gate mock generator, and add audit report | `docs/analysis/MCP_PY_FACADE_AUDIT_REPORT.md` | +0/-0 prod, +0/-0 test, +91/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| c041918c | 2026-10-13 | docs(analysis): add deep audit report for Ring 3 crates | `docs/analysis/AUDIT_collection-cognition-router-agent-db_DEEP_2026-10-13.md` | +0/-0 prod, +0/-0 test, +343/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| 5594419f | 2026-10-13 | docs(audit): add deep audit report for contextra-crypto | `docs/analysis/AUDIT_contextra-crypto_DEEP_2026-10-13.md` | +0/-0 prod, +0/-0 test, +368/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| c0b2ffe2 | 2026-10-13 | docs(analysis): add deep audit report for contextra-mcp, contextra-py and contextra facade | `docs/analysis/AUDIT_mcp_py_facade_DEEP_2026-10-13.md` | +0/-0 prod, +0/-0 test, +235/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| c4588561 | 2026-10-13 | docs: add deep audit report for testkit, fuzz, loom and governance | `docs/analysis/AUDIT_testkit-fuzz-loom-governance_DEEP_2026-10-13.md` | +0/-0 prod, +203/-0 test, +0/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| 24cf99be | 2026-10-13 | docs(store): deep audit analysis for contextra-store LSM and compaction | `docs/analysis/AUDIT_contextra-store_DEEP_2026-10-13.md` | +0/-0 prod, +0/-0 test, +246/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| 4e29fd9a | 2026-10-13 | docs(audit): add composition root and domain types audit report | `docs/analysis/AUDIT_composition-root_DEEP_2026-10-13.md` | +0/-0 prod, +0/-0 test, +208/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| 14530cb3 | 2026-10-13 | feat(xtask): implement detect-silent-reverts, find-unwired-gaps, and generate-interface-spec | `xtask/src/harness/detect_silent_reverts.rs`<br>`xtask/src/harness/find_unwired_gaps.rs`<br>`xtask/src/harness/generate_interface_spec.rs` | +1680/-0 prod, +0/-0 test, +0/-0 doku | PORT-FEAT | Neue Infrastruktur-/Xtask-Funktion oder Quality-Gate (`xtask/src/harness/detect_silent_reverts.rs`). | xtask | nein | niedrig |
| 4b7709e6 | 2026-10-13 | Shell-Commit | `xtask/src/check_mutation_score_gate.rs`<br>`xtask/tests/verify_sabotage_test.rs` | +3/-2 prod, +9/-2 test, +0/-0 doku | PORT-FEAT | Neue Infrastruktur-/Xtask-Funktion oder Quality-Gate (`xtask/src/check_mutation_score_gate.rs`). | xtask | teilweise | niedrig |
| 2ae9a452 | 2026-10-13 | feat(engine): complete document erasure with footprint, ledger, and witnesses | `.jules/tasks/T-2026-9230.toml` | +0/-0 prod, +0/-0 test, +1/-1 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-9230.toml`). | keine | nein | niedrig |
| 28df2ae4 | 2026-10-13 | feat(engine): implement complete document erasure with footprint, fence, and witnesses | `.jules/tasks/T-2026-9230.toml` | +0/-0 prod, +0/-0 test, +22/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-9230.toml`). | keine | nein | niedrig |
| 8797638d | 2026-10-13 | feat(engine): implement proof runtime and purge driver | `.jules/tasks/T-2026-9231.toml` | +0/-0 prod, +0/-0 test, +22/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-9231.toml`). | keine | nein | niedrig |
| ee5a102e | 2026-10-13 | fix(engine): normalize tx staging order and reset RAM state on drop | `.jules/tasks/T-2026-9232.toml` | +0/-0 prod, +0/-0 test, +22/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-9232.toml`). | keine | nein | niedrig |
| 561dcec9 | 2026-10-13 | Shell-Commit | `.jules/tasks/T-2026-9213.toml`<br>`.jules/tasks/T-2026-9220.toml`<br>`docs/spec/CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md`<br>*(+2 weitere)* | +0/-0 prod, +0/-0 test, +1119/-1064 doku | DECISION | Änderung an der Spezifikation (`docs/spec/**`); Entscheidung des Maintainers zur Übernahme erforderlich. | keine | teilweise | hoch |
| e6b8a2c8 | 2026-10-13 | feat(checkpoint): Kopienregister, Cloner-Hygiene und Restore-Filter | `.jules/tasks/T-2026-9223.toml` | +0/-0 prod, +0/-0 test, +21/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-9223.toml`). | keine | nein | niedrig |
| c3db6908 | 2026-10-13 | feat(crypto): implement ProofIssuer v4 single entry point | `.jules/tasks/T-2026-9213.toml`<br>`.jules/tasks/T-2026-9224.toml`<br>`docs/deletion/vectors/v4_proof_sample.json` | +0/-0 prod, +0/-0 test, +105/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-9213.toml`). | keine | teilweise | niedrig |
| d26ac482 | 2026-10-13 | feat(engine): implement ForgetOptions public types and environment attestation TLV | `.jules/tasks/T-2026-9213.toml`<br>`.jules/tasks/T-2026-9226.toml` | +0/-0 prod, +0/-0 test, +58/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-9213.toml`). | keine | nein | niedrig |
| e69edff4 | 2026-10-13 | feat(vector): implement HNSW residue witness issuance and deletion hardening | `.jules/tasks/T-2026-9222.toml` | +0/-0 prod, +0/-0 test, +22/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-9222.toml`). | keine | nein | niedrig |
| edeb252d | 2026-10-13 | Shell-Commit | `.jules/tasks/T-2026-9209.toml` | +0/-0 prod, +0/-0 test, +31/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-9209.toml`). | keine | nein | niedrig |
| df71c383 | 2026-10-13 | feat(store): integrate ErasureLedger into WAL replay and recovery | `.jules/tasks/T-2026-9204.toml` | +0/-0 prod, +0/-0 test, +26/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-9204.toml`). | keine | nein | niedrig |
| 2f68a144 | 2026-10-13 | fix(engine,mcp): resolve drop_collection keyspace mismatch and fail-closed proofs | `.jules/tasks/T-2026-9212.toml` | +0/-0 prod, +0/-0 test, +32/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-9212.toml`). | keine | nein | niedrig |
| 989aaf12 | 2026-10-12 | feat(store): Purge-Orchestrator crash-consistent state machine (C5.1/C5.6) | `.jules/tasks/T-2026-9206.toml` | +0/-0 prod, +0/-0 test, +29/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-9206.toml`). | keine | nein | niedrig |
| ce621571 | 2026-10-12 | feat(store): implement crash-safe Purge-Orchestrator state machine (C5.1/C5.6) | `.jules/tasks/T-2026-9206.toml` | +0/-0 prod, +0/-0 test, +29/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-9206.toml`). | keine | nein | niedrig |
| ffd3d0e6 | 2026-10-12 | feat(store): implement Purge-Compaction (Spec C5.4) | `.jules/tasks/T-2026-9205.toml` | +0/-0 prod, +0/-0 test, +20/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-9205.toml`). | keine | nein | niedrig |
| e8a40e6a | 2026-10-12 | fix(verification): repair mutation config and invariant test matrix | `docs/mutation_score_history.jsonl`<br>`docs/verification/INV_MATRIX.md`<br>`docs/verification/MUTATION_BASELINE.md` | +0/-0 prod, +0/-0 test, +9/-4 doku | PORT-FIX | Korrektur im Infrastruktur-Pfadebereich (`docs/mutation_score_history.jsonl`). | keine | teilweise | niedrig |
| b713f897 | 2026-10-12 | feat(vector,text): implement residue scanners and deletion repair | `.jules/tasks/T-2026-9208.toml` | +0/-0 prod, +0/-0 test, +34/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-9208.toml`). | keine | nein | niedrig |
| 9bbb9ebc | 2026-10-12 | feat(engine): implement deletion footprint, registry and reverse-sweep (INV-D1) | `.jules/tasks/T-2026-9211.toml`<br>`docs/deletion/KEYFAMILIES.toml` | +0/-0 prod, +0/-0 test, +45/-14 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-9211.toml`). | keine | teilweise | niedrig |
| 95241b81 | 2026-10-12 | feat(store): implement prefix-scoped write fences and guarded document locks | `.jules/tasks/T-2026-9207.toml` | +0/-0 prod, +0/-0 test, +30/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-9207.toml`). | keine | nein | niedrig |
| 5b213a3b | 2026-10-12 | feat(crypto): harden v4 deletion proof policy, store, signer and builder | `.jules/tasks/T-2026-9203.toml`<br>`docs/deletion/POLICY_RULES.md` | +0/-0 prod, +0/-0 test, +50/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-9203.toml`). | keine | teilweise | niedrig |
| f3adf969 | 2026-10-12 | feat(p08): implement independent canary oracle and forget crash verification | `docs/deletion/CANARY_ORACLE.md` | +0/-0 prod, +0/-0 test, +117/-0 doku | PORT-FEAT | Funktionale Erweiterung im Pfadebereich (`docs/deletion/CANARY_ORACLE.md`). | keine | ja | niedrig |
| 5b1b9279 | 2026-10-12 | feat(xtask): add single-command verify gate and evidence generator | `.github/workflows/verify-kernel.yml`<br>`.jules/setup/required-tools.toml`<br>`docs/gates/verify.md`<br>*(+20 weitere)* | +1001/-53 prod, +82/-1 test, +55/-2 doku | PORT-FEAT | Neue Infrastruktur-/Xtask-Funktion oder Quality-Gate (`.github/workflows/verify-kernel.yml`). | xtask | teilweise | hoch |
| 43c1d52a | 2026-10-12 | feat(sys): implement environment observation (C8) in contextra-sys | `.jules/tasks/T-2026-9210.toml` | +0/-0 prod, +0/-0 test, +21/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-9210.toml`). | keine | nein | niedrig |
| 0dcbd4bf | 2026-10-12 | docs(governance): define core scope and robustness contract | `docs/scope/CORE_SCOPE.md`<br>`docs/scope/PLATFORM_SCOPE.md`<br>`docs/scope/RING_DEBT.md`<br>*(+1 weitere)* | +0/-0 prod, +0/-0 test, +276/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| 1f66e19b | 2026-10-12 | feat(verifier): implement standalone python deletion proof v4 verifier | `.github/workflows/verifier-python.yml`<br>`docs/verification/VERIFIER_SPEC.md`<br>`docs/verification/verify_export.py` | +43/-0 prod, +0/-0 test, +171/-236 doku | PORT-FEAT | Erweiterung oder neue CI-Pipeline (`.github/workflows/verifier-python.yml`). | .github | teilweise | niedrig |
| 6e22e505 | 2026-10-12 | prompter: sync workspace crates, modernization & fuzz blocks | `.jules/contextra-Prompter.html` | +0/-0 prod, +0/-0 test, +133/-70 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/contextra-Prompter.html`). | keine | teilweise | niedrig |
| fd31a72e | 2026-10-12 | feat(testkit): implement Canary-Oracle raw sweep and marker set | `.jules/tasks/T-2026-9201.toml` | +0/-0 prod, +0/-0 test, +27/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-9201.toml`). | keine | nein | niedrig |
| de96ee97 | 2026-10-12 | feat(crypto): implement WitClaimText, WitLayerEvidence, and WitIssueArgs v2 (W5-02) | `.jules/tasks/T-2026-9202.toml` | +0/-0 prod, +0/-0 test, +29/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-9202.toml`). | keine | nein | niedrig |
| c61343f8 | 2026-10-12 | feat(crypto): add DeletionLayer and ExcludedScope extensions for v4 | `.jules/tasks/T-2026-9200.toml` | +0/-0 prod, +0/-0 test, +25/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-9200.toml`). | keine | nein | niedrig |
| 47d5b308 | 2026-10-12 | refactor(xtask): exclude helper modules, ratchet lints, and add help subcommand | `xtask/Cargo.toml`<br>`xtask/build.rs`<br>`xtask/src/lib.rs`<br>*(+2 weitere)* | +127/-70 prod, +98/-18 test, +0/-0 doku | PORT-WIRING | Infrastruktur-Verdrahtung oder Refactoring (`xtask/Cargo.toml`). | xtask | teilweise | hoch |
| 27830321 | 2026-10-13 | Shell-Commit | `.jules/setup/environment_script.sh`<br>`.jules/setup/install-bench-deps.sh`<br>`.jules/setup/install-fuzz-toolchain.sh`<br>*(+1 weitere)* | +0/-0 prod, +0/-0 test, +217/-206 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/setup/environment_script.sh`). | keine | teilweise | niedrig |
| 13234803 | 2026-10-12 | feat(xtask): add context-card harness and safe refresh-context script | `.jules/setup/refresh-context.sh`<br>`xtask/src/context_pack.rs`<br>`xtask/src/harness/context_card.rs` | +1234/-43 prod, +0/-0 test, +42/-37 doku | PORT-FEAT | Neue Infrastruktur-/Xtask-Funktion oder Quality-Gate (`.jules/setup/refresh-context.sh`). | xtask | teilweise | niedrig |
| 6da9fecd | 2026-10-12 | docs(crypto): KDF migration audit - identify remaining callers of deprecated try_new | `docs/audits/AUDIT_kdf-migration-caller-inventory_2026-10-12.md` | +0/-0 prod, +0/-0 test, +88/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| 96d35bc1 | 2026-10-12 | ci(gates): wire unreferenced commands, add fuzz/bench checks and new just recipes | `.github/workflows/merge-gate.yml`<br>`.github/workflows/rust-ci.yml`<br>`justfile` | +152/-4 prod, +0/-0 test, +0/-0 doku | PORT-FEAT | Neue Infrastruktur-/Xtask-Funktion oder Quality-Gate (`.github/workflows/merge-gate.yml`). | xtask | teilweise | hoch |
| 2dcd8eb4 | 2026-10-12 | fix(prompter): sync prompter-tiers and gen-prompter-data for 35 packages | `.jules/PROMPTER_RESTRUCTURING_PLAN.md`<br>`.jules/prompter-data.json`<br>`.jules/prompter-tiers.toml`<br>*(+1 weitere)* | +197/-19 prod, +0/-0 test, +3725/-1494 doku | PORT-FIX | Fehlerkorrektur im Infra-/Xtask-Bereich (`.jules/PROMPTER_RESTRUCTURING_PLAN.md`). | xtask | teilweise | niedrig |
| 4073a18a | 2026-10-12 | docs(jules): harmonize agent docs and session protocols | `.jules/JULES_CONTEXT.md`<br>`.jules/PARALLEL_SESSIONS.md`<br>`.jules/PREAMBLE.md`<br>*(+1 weitere)* | +0/-0 prod, +0/-0 test, +61/-44 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | teilweise | hoch |
| baa0b1d2 | 2026-10-12 | docs(governance): add rules for fuzzing, benchmarking, mutants, and loom/miri | `rules/benchmarking.md`<br>`rules/fuzzing.md`<br>`rules/loom-miri.md`<br>*(+1 weitere)* | +0/-0 prod, +45/-0 test, +165/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| ab8ccaa1 | 2026-10-12 | harness: harden jules facade and risk gate execution | `.jules/harness-phases.toml`<br>`xtask/src/harness/jules.rs` | +506/-33 prod, +0/-0 test, +17/-0 doku | PORT-FEAT | Neue Infrastruktur-/Xtask-Funktion oder Quality-Gate (`.jules/harness-phases.toml`). | xtask | teilweise | hoch |
| b74a95c8 | 2026-10-12 | ci(setup): standardize Jules VM setup and toolchain pre-warming | `.jules/setup/environment_script.sh`<br>`.jules/setup/install-bench-deps.sh`<br>`.jules/setup/install-fuzz-toolchain.sh`<br>*(+1 weitere)* | +0/-0 prod, +0/-0 test, +332/-38 doku | PORT-FEAT | Erweiterung oder neue CI-Pipeline (`.jules/setup/environment_script.sh`). | .github | teilweise | niedrig |
| 396b795f | 2026-10-12 | Passe xtask-Registry an und erweitere registry-check | `xtask/registry.toml`<br>`xtask/src/harness/registry_check.rs` | +811/-289 prod, +0/-0 test, +0/-0 doku | PORT-FEAT | Neue Infrastruktur-/Xtask-Funktion oder Quality-Gate (`xtask/registry.toml`). | xtask | teilweise | hoch |
| 97e72b69 | 2026-10-12 | docs(jules): align .jules workflow docs and anti-patterns | `.jules/ANTI_PATTERNS.md`<br>`.jules/COMMON_LLM_ERRORS.md`<br>`.jules/HARNESS_QUICKSTART.md`<br>*(+4 weitere)* | +0/-0 prod, +0/-0 test, +777/-45 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | teilweise | hoch |
| 448f6f16 | 2026-10-12 | test(crypto): increase deletion proof mutation score to 98.6% | `docs/audits/MUTANTS_crypto_2026-10-12.md` | +0/-0 prod, +0/-0 test, +93/-0 doku | SKIP-DOC | Dokumentations- oder Sonstige Anpassung (`docs/audits/MUTANTS_crypto_2026-10-12.md`). | keine | nein | niedrig |
| 0efdc52e | 2026-10-12 | fix(store): enforce WAL downgrade protection & document manifest rank | `docs/audits/MANIFEST_RANK_2026-10-12.md` | +0/-0 prod, +0/-0 test, +82/-0 doku | PORT-FIX | Korrektur im Infrastruktur-Pfadebereich (`docs/audits/MANIFEST_RANK_2026-10-12.md`). | keine | nein | niedrig |
| 75c687b6 | 2026-10-12 | docs(spec): create SOURCE_OF_TRUTH_DIFF and fix orphaned path references | `docs/spec/CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md`<br>`docs/spec/CONTEXTRA_SPEZIFIKATION_v15.md`<br>`docs/spec/SOURCE_OF_TRUTH_DIFF.md` | +0/-0 prod, +0/-0 test, +806/-732 doku | DECISION | Änderung an der Spezifikation (`docs/spec/**`); Entscheidung des Maintainers zur Übernahme erforderlich. | keine | teilweise | hoch |
| 449784f4 | 2026-10-12 | docs(spec): establish source of truth diff and fix orphaned spec path references | `docs/spec/CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md`<br>`docs/spec/CONTEXTRA_SPEZIFIKATION_v15.md`<br>`docs/spec/SOURCE_OF_TRUTH_DIFF.md` | +0/-0 prod, +0/-0 test, +1747/-1646 doku | DECISION | Änderung an der Spezifikation (`docs/spec/**`); Entscheidung des Maintainers zur Übernahme erforderlich. | keine | teilweise | hoch |
| 7e014c0a | 2026-10-12 | ci: align benchmark paths, add tree-borrows miri run, and add check-loom-mocks gate | `.github/workflows/rust-ci.yml`<br>`justfile`<br>`xtask/src/check_loom_mocks.rs`<br>*(+1 weitere)* | +111/-8 prod, +0/-0 test, +0/-0 doku | PORT-WIRING | Infrastruktur-Verdrahtung oder Refactoring (`.github/workflows/rust-ci.yml`). | xtask | teilweise | hoch |
| bed02adf | 2026-10-12 | fix(mutants): justify skip test args, canonicalize G2 path and log score history | `docs/mutation_score_history.jsonl` | +0/-0 prod, +0/-0 test, +1/-0 doku | PORT-FIX | Korrektur im Infrastruktur-Pfadebereich (`docs/mutation_score_history.jsonl`). | keine | teilweise | niedrig |
| 7bf918ed | 2026-10-12 | docs(governance): review and confirm VETO-F02 and VETO-OP03 in ADR-VETO-REVIEW-F02-OP03-2026-10 | `docs/decisions/ADR-VETO-REVIEW-F02-OP03-2026-10.md` | +0/-0 prod, +0/-0 test, +43/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| 9908339b | 2026-10-12 | docs: add ADR-SCOPE-FREEZE-2026-10 for crate classification | `docs/decisions/ADR-SCOPE-FREEZE-2026-10.md`<br>`docs/decisions/README.md` | +0/-0 prod, +0/-0 test, +106/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | teilweise | niedrig |
| 88cfe2f0 | 2026-10-12 | docs(adr): add ADR-SCOPE-FREEZE-2026-10 for crate classification | `docs/decisions/ADR-SCOPE-FREEZE-2026-10.md`<br>`docs/decisions/README.md` | +0/-0 prod, +0/-0 test, +94/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | teilweise | niedrig |
| 932616ff | 2026-10-12 | docs: create verification matrix INV_MATRIX.md for specification invariants | `docs/verification/INV_MATRIX.md` | +0/-0 prod, +0/-0 test, +128/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| 6db2ca3b | 2026-10-12 | docs(audit): Mutation-Testing-Kampagne fuer Crypto und Kernalgorithmen | `docs/audits/AUDIT_mutation-testing_2026-10-12.md`<br>`docs/mutation_score_history.jsonl` | +0/-0 prod, +105/-0 test, +4/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | teilweise | niedrig |
| 73715757 | 2026-10-12 | P30 / INV-CORE-ISOLATION-1: Core isolation, witness mint fix & facade composition root wiring | `.jules/tasks/T-2026-0309.toml` | +0/-0 prod, +0/-0 test, +24/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-0309.toml`). | keine | nein | niedrig |
| 1c9bcd83 | 2026-10-12 | docs: add ADR-RING-crypto-durable-fs for durable-fs dependencies | `docs/decisions/ADR-RING-crypto-durable-fs.md` | +0/-0 prod, +0/-0 test, +40/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| 6ed22ce4 | 2026-10-12 | docs(audit): Systematische Fuzz-Kampagne Audit-Bericht (fuzz-campaign) | `docs/audits/AUDIT_fuzz-campaign_2026-10-12.md` | +0/-0 prod, +0/-0 test, +74/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| 6ee2a5bd | 2026-10-12 | docs(audit): Tiefenaudit SIMD-Kernel in contextra-simd | `docs/audits/AUDIT_simd-kernels_DEEP_2026-10-12.md` | +0/-0 prod, +0/-0 test, +134/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| 815f5ee9 | 2026-10-12 | docs(audit): Save release gate audit report v0.1.0 2026-10-12 | `docs/audits/AUDIT_release-gate_v0.1.0_2026-10-12.md` | +0/-0 prod, +0/-0 test, +55/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| 1afe4ca4 | 2026-10-12 | docs(audit): algorithm audit report for contextra-mvcc | `docs/audits/AUDIT_contextra-mvcc_2026-10-12.md` | +0/-0 prod, +0/-0 test, +98/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| 62d67a11 | 2026-10-12 | docs(audit): Algorithmitischer Tiefenaudit contextra-graph (CSR, PPR, Leiden, Bi-Temporal) | `docs/audits/AUDIT_csr-ppr_DEEP_2026-10-12.md` | +0/-0 prod, +0/-0 test, +126/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| 9e8ce5fa | 2026-10-12 | audit(mvcc): Algorithm and property test audit report for contextra-mvcc | `docs/audits/AUDIT_contextra-mvcc_2026-10-12.md` | +0/-0 prod, +0/-0 test, +107/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| d5cef864 | 2026-10-12 | docs(audit): Deep audit of BM25 search engine in contextra-text | `docs/audits/AUDIT_bm25_DEEP_2026-10-12.md` | +0/-0 prod, +0/-0 test, +233/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| a72ce152 | 2026-10-12 | docs(audit): Kryptografischer Tiefenaudit Crypto-Kernel contextra-crypto | `docs/audits/AUDIT_crypto-kernel_DEEP_2026-10-12.md` | +0/-0 prod, +0/-0 test, +143/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| 6cc33543 | 2026-10-12 | docs(facade): API-Konsistenz- und Dokumentationsaudit crates/contextra | `docs/audits/AUDIT_contextra-facade_2026-10-12.md` | +0/-0 prod, +0/-0 test, +70/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| 54a733b3 | 2026-10-12 | docs(audit): Algorithmitischer Tiefenaudit des HNSW-Graphalgorithmus | `docs/audits/AUDIT_hnsw_DEEP_2026-10-12.md` | +0/-0 prod, +0/-0 test, +165/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| 998a7642 | 2026-10-12 | docs(audit): Ring-DAG integrity audit report | `docs/audits/AUDIT_ring-dag-integrity_2026-10-12.md` | +0/-0 prod, +0/-0 test, +110/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| d987b099 | 2026-10-12 | docs(audit): Workspace Zero-Panic & Unsafe Audit Report | `docs/audits/AUDIT_zero-panic-unsafe_WORKSPACE_2026-10-12.md` | +0/-0 prod, +0/-0 test, +925/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| 6f13b854 | 2026-10-12 | docs(audit): Deep audit report for compaction engine in contextra-store | `docs/audits/AUDIT_compaction_DEEP_2026-10-12.md` | +0/-0 prod, +0/-0 test, +223/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| a832a6e7 | 2026-10-12 | docs(audit): SOLL-vs-IST gap analysis report (2026-10-12) | `docs/audits/AUDIT_soll-ist-gap_2026-10-12.md` | +0/-0 prod, +0/-0 test, +125/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| 09b58833 | 2026-10-12 | docs(audit): Algorithmitischer Tiefenaudit contextra-kvcache | `docs/audits/AUDIT_kvcache_DEEP_2026-10-12.md` | +0/-0 prod, +0/-0 test, +95/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| 320ad0ce | 2026-10-12 | docs(audit): Numerischer Stabilitäts-Tiefenaudit Adaptive Controller | `docs/audits/AUDIT_adaptive-controller_DEEP_2026-10-12.md` | +0/-0 prod, +0/-0 test, +204/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| 3ada91bc | 2026-10-12 | docs(audit): audit report for contextra-mcp | `docs/audits/AUDIT_contextra-mcp_2026-10-12.md` | +0/-0 prod, +0/-0 test, +140/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| 98d66e3d | 2026-10-12 | docs(audit): FFI-Boundary Audit-Bericht fuer contextra-py | `docs/audits/AUDIT_contextra-py_2026-10-12.md` | +0/-0 prod, +0/-0 test, +87/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| 19df22ec | 2026-10-12 | docs(audit): Algorithmitischer Tiefenaudit LSM-Tree-Kern (lsm-core-deep-audit) | `docs/audits/AUDIT_lsm-core_DEEP_2026-10-12.md` | +0/-0 prod, +0/-0 test, +121/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| c0fcc5b0 | 2026-10-12 | docs(audit): Korrektheit- und Performance-Audit SSTable Subsystem | `docs/audits/AUDIT_sstable_DEEP_2026-10-12.md` | +0/-0 prod, +0/-0 test, +145/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| 00f4cf45 | 2026-10-12 | docs(audit): deep audit report for WAL subsystem (W1-W7) | `docs/audits/AUDIT_wal-subsystem_DEEP_2026-10-12.md` | +0/-0 prod, +0/-0 test, +152/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| c59626bd | 2026-10-12 | docs(audit): add contextra-store P1-P10 audit report | `docs/audits/AUDIT_contextra-store_2026-10-12.md` | +0/-0 prod, +0/-0 test, +102/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| fc840977 | 2026-10-12 | docs(audits): complete security audit for contextra-sys (Ring 0) | `docs/audits/AUDIT_contextra-core_2026-10-12.md`<br>`docs/audits/AUDIT_contextra-types_2026-10-12.md` | +0/-0 prod, +0/-0 test, +183/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| 4747cc23 | 2026-10-12 | docs(onnx): add contextra-infer-onnx audit report | `docs/audits/AUDIT_contextra-infer-onnx_2026-10-12.md` | +0/-0 prod, +0/-0 test, +90/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| 6fa38752 | 2026-10-12 | docs(audit): quality audit report for contextra-testkit | `docs/audits/AUDIT_contextra-testkit_2026-10-12.md` | +0/-0 prod, +181/-0 test, +0/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| 17f192a2 | 2026-10-12 | docs(audit): compliance audit report for contextra-audit-export | `docs/audits/AUDIT_contextra-audit-export_2026-10-12.md` | +0/-0 prod, +0/-0 test, +106/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| a935954e | 2026-10-12 | docs(audit): Compliance-Audit für contextra-avv-generator (Art. 28 DSGVO) | `docs/audits/AUDIT_contextra-avv-generator_2026-10-12.md` | +0/-0 prod, +0/-0 test, +157/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| 4b2821b1 | 2026-10-12 | docs(audit): Sicherheitsaudit-Bericht fuer contextra-license | `docs/audits/AUDIT_contextra-license_2026-10-12.md` | +0/-0 prod, +0/-0 test, +106/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| 6d882691 | 2026-10-12 | docs(kvcache): add contextra-kvcache security and correctness audit report | `docs/audits/AUDIT_contextra-kvcache_2026-10-12.md` | +0/-0 prod, +0/-0 test, +114/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| 29a93dd2 | 2026-10-12 | docs(audit): full audit for crates/contextra-db | `docs/audits/AUDIT_contextra-db_2026-10-12.md` | +0/-0 prod, +0/-0 test, +159/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| 24628bd8 | 2026-10-12 | docs(audit): add zero-panic and numerical stability audit for contextra-adapt | `docs/audits/AUDIT_contextra-adapt_2026-10-12.md` | +0/-0 prod, +0/-0 test, +149/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| 7f0ebe29 | 2026-10-12 | docs(audit): add system audit report for contextra-engine | `docs/audits/AUDIT_contextra-engine_2026-10-12.md` | +0/-0 prod, +0/-0 test, +80/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| 4b7ddfe5 | 2026-10-12 | docs(agent): audit report for contextra-agent (Ring 3) | `docs/audits/AUDIT_contextra-agent_2026-10-12.md` | +0/-0 prod, +0/-0 test, +121/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| 7d905f2a | 2026-10-12 | docs(audit): add structure and invariant audit for contextra-vector | `docs/audits/AUDIT_contextra-vector_2026-10-12.md` | +0/-0 prod, +0/-0 test, +90/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| d0907191 | 2026-10-12 | audit(sandbox): add security audit report for contextra-sandbox | `docs/audits/AUDIT_contextra-sandbox_2026-10-12.md` | +0/-0 prod, +0/-0 test, +110/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| d18d119c | 2026-10-12 | docs(crypto): complete security audit for contextra-crypto crate | `docs/audits/AUDIT_contextra-crypto_2026-10-12.md` | +0/-0 prod, +0/-0 test, +135/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| 9cf8ce1d | 2026-10-12 | docs(audits): add contextra-graph structure and invariant audit report | `docs/audits/AUDIT_contextra-graph_2026-10-12.md` | +0/-0 prod, +0/-0 test, +233/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| 7e92a581 | 2026-10-12 | docs(cognition): add audit report for contextra-cognition | `docs/audits/AUDIT_contextra-cognition_2026-03-06.md` | +0/-0 prod, +0/-0 test, +90/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| a3218ad1 | 2026-10-12 | docs(audit): add contextra-text determinism audit and regression tests | `docs/audits/AUDIT_contextra-text_2026-10-12.md` | +0/-0 prod, +0/-0 test, +154/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| 5273990d | 2026-10-12 | docs(rank): add contextra-rank algorithm and correctness audit | `docs/audits/AUDIT_contextra-rank_2026-10-12.md` | +0/-0 prod, +0/-0 test, +140/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| fa31e8ad | 2026-10-12 | docs(router): add contextra-router algorithm and security audit report | `docs/audits/AUDIT_contextra-router_2026-10-12.md` | +0/-0 prod, +0/-0 test, +129/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| 0fc8b0ec | 2026-10-12 | audit(ollama): complete architecture and code quality audit | `docs/audits/AUDIT_contextra-infer-ollama_2026-10-12.md` | +0/-0 prod, +0/-0 test, +80/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| ceaeb33a | 2026-10-12 | docs(audit): add contextra-infer-candle architectural audit report | `docs/audits/AUDIT_contextra-infer-candle_2026-10-12.md` | +0/-0 prod, +0/-0 test, +110/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| 2d6161c5 | 2026-10-12 | docs(simd): safety and correctness audit report for contextra-simd | `docs/audits/AUDIT_contextra-simd_2026-10-12.md` | +0/-0 prod, +0/-0 test, +96/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| 11b63d0c | 2026-10-12 | docs(audit): safety and correctness audit for contextra-simd | `docs/audits/AUDIT_contextra-simd_2026-10-12.md` | +0/-0 prod, +0/-0 test, +87/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| 24790b59 | 2026-10-12 | docs(audits): full audit cycle for contextra-ports (Ring 0) | `docs/audits/AUDIT_contextra-ports_2026-10-12.md` | +0/-0 prod, +0/-0 test, +163/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| d5c45033 | 2026-10-12 | docs(audits): complete security audit for contextra-sys (Ring 0) | `docs/audits/AUDIT_contextra-sys_2026-10-12.md` | +0/-0 prod, +0/-0 test, +84/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| 2ab2e3b7 | 2026-10-12 | docs(wire): Audit Report contextra-wire 2026-10-12 | `docs/audits/AUDIT_contextra-wire_2026-10-12.md` | +0/-0 prod, +0/-0 test, +105/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| 00fbec75 | 2026-10-12 | docs(audit): Audit-Bericht für Strangler-Fassade contextra-core | `docs/audits/AUDIT_contextra-core_2026-10-12.md` | +0/-0 prod, +0/-0 test, +110/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| d23272d6 | 2026-10-12 | docs(audit): full audit report for contextra-types Ring 0 crate | `docs/audits/AUDIT_contextra-types_2026-10-12.md` | +0/-0 prod, +0/-0 test, +73/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| d8784968 | 2026-10-12 | feat(crypto): implement KmsProvider, MFKS Keystore, DeletionProof v4, and strict KDF policy | `.jules/tasks/T-2026-0306.toml` | +0/-0 prod, +0/-0 test, +17/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-0306.toml`). | keine | nein | niedrig |
| 225e8b69 | 2026-10-12 | feat(store): Format-Verifikation, MFSX/MFBL/MFMN Golden Corpus und Migration | `.jules/tasks/T-2026-0303.toml` | +0/-0 prod, +0/-0 test, +42/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-0303.toml`). | keine | nein | niedrig |
| 4b24db6f | 2026-10-12 | feat(mcp): implement P7 IPC transport, primary lock & CLI tools | `.jules/tasks/T-2026-0307.toml` | +0/-0 prod, +0/-0 test, +49/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-0307.toml`). | keine | nein | niedrig |
| d3821471 | 2026-10-12 | feat(wal): implement WAL V4 binary format and crash consistency mechanics | `.jules/tasks/T-2026-0302.toml` | +0/-0 prod, +0/-0 test, +36/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-0302.toml`). | keine | nein | niedrig |
| a95c43ed | 2026-10-12 | contextra-store: audit lib.rs API encapsulation and add file-context header | `.jules/JULES_CONTEXT.md`<br>`docs/audits/AUDIT_contextra-store_2026-10-11.md` | +0/-0 prod, +0/-0 test, +57/-1 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | teilweise | hoch |
| 72bf7ae3 | 2026-10-11 | feat(sys): implement platform durability primitives and hardening | `.jules/tasks/T-2026-0301.toml` | +0/-0 prod, +0/-0 test, +2/-2 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-0301.toml`). | keine | nein | niedrig |
| fe05038a | 2026-10-11 | feat(sys): durability primitives, fsync poisoning, fs_class, and dirlock hardening | `.jules/tasks/T-2026-0301.toml` | +0/-0 prod, +0/-0 test, +35/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-0301.toml`). | keine | nein | niedrig |
| cd3edcce | 2026-10-11 | feat(deletion): add KeyFamilyRegistry foundation and KEYFAMILIES.toml | `docs/deletion/KEYFAMILIES.toml` | +0/-0 prod, +0/-0 test, +86/-0 doku | PORT-FEAT | Funktionale Erweiterung im Pfadebereich (`docs/deletion/KEYFAMILIES.toml`). | keine | ja | niedrig |
| dec470f9 | 2026-10-11 | feat(store): WAL witness scanner, segment retirement and replay filter | `.jules/tasks/T-2026-9105.toml` | +0/-0 prod, +0/-0 test, +32/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-9105.toml`). | keine | nein | niedrig |
| 536e891e | 2026-10-11 | contextra-infer-candle: audit Candle ML inference path and review passes | `docs/audits/AUDIT_contextra-infer-candle_2026-10-11.md` | +0/-0 prod, +0/-0 test, +52/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| 996251b5 | 2026-10-11 | feat(crypto): implement DeletionProof v4 format and independent verifier | `.jules/tasks/T-2026-9102.toml` | +0/-0 prod, +0/-0 test, +20/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-9102.toml`). | keine | nein | niedrig |
| 525ecd80 | 2026-10-11 | contextra-db: add plan TODOs for index-store transaction sync | `.jules/contextra-Prompter.html`<br>`docs/audits/deletion-proof-ist-analyse-restplan-2026-10.md`<br>`governance/gates.toml`<br>*(+5 weitere)* | +3/-295 prod, +0/-0 test, +638/-2096 doku | PORT-FEAT | Neue Infrastruktur-/Xtask-Funktion oder Quality-Gate (`.jules/contextra-Prompter.html`). | xtask | teilweise | hoch |
| 2a24cf21 | 2026-10-11 | feat(store): SSTable WitnessReader, Scanner & Orphan GC (T-2026-9104) | `.jules/tasks/T-2026-9104a.toml`<br>`.jules/tasks/T-2026-9104b.toml` | +0/-0 prod, +0/-0 test, +55/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-9104a.toml`). | keine | nein | niedrig |
| 7a2a4734 | 2026-10-11 | contextra-infer-candle: add plan markers for Candle ML inference path | `docs/audits/AUDIT_contextra-infer-candle_2026-10-11.md` | +0/-0 prod, +0/-0 test, +59/-0 doku | SKIP-DOC | Dokumentations- oder Sonstige Anpassung (`docs/audits/AUDIT_contextra-infer-candle_2026-10-11.md`). | keine | nein | niedrig |
| 3e8d78e0 | 2026-10-11 | contextra-cognition: plan memory consolidation pass separation | `.jules/JULES_CONTEXT.md`<br>`docs/audits/AUDIT_contextra-cognition_2026-10-11.md` | +0/-0 prod, +0/-0 test, +55/-1 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/JULES_CONTEXT.md`). | keine | teilweise | hoch |
| 5a8c576a | 2026-10-11 | audit(contextra-infer-candle): audit KV-Cache-Bridge and fail-open invariants | `docs/audits/AUDIT_contextra-infer-candle_2026-10-11.md` | +0/-0 prod, +0/-0 test, +62/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| 96159973 | 2026-10-11 | contextra-db: audit homeostat.rs and lib.rs re-exports and update FILE-CONTEXT headers | `docs/audits/AUDIT_contextra-db_2026-10-11.md` | +0/-0 prod, +0/-0 test, +60/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| 839fefd3 | 2026-10-11 | feat(store): implement Erasure-Ledger and Copy-Registry (T-2026-9103) | `.jules/tasks/T-2026-9103.toml` | +0/-0 prod, +0/-0 test, +28/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-9103.toml`). | keine | nein | niedrig |
| 298de109 | 2026-10-11 | contextra-rank: sanitize DriftDetector threshold inputs and add tests | `docs/audits/AUDIT_contextra-rank_2026-10-11.md` | +0/-0 prod, +0/-0 test, +38/-0 doku | SKIP-DOC | Dokumentations- oder Sonstige Anpassung (`docs/audits/AUDIT_contextra-rank_2026-10-11.md`). | keine | nein | niedrig |
| b8660984 | 2026-10-11 | Shell-Commit | `.jules/contextra-Prompter.html`<br>`governance/gates.toml`<br>`xtask/src/harness/jules.rs`<br>*(+4 weitere)* | +295/-3 prod, +0/-0 test, +1779/-638 doku | PORT-FEAT | Neue Infrastruktur-/Xtask-Funktion oder Quality-Gate (`.jules/contextra-Prompter.html`). | xtask | teilweise | hoch |
| a2b063dc | 2026-10-11 | Shell-Commit | `docs/audits/deletion-proof-ist-analyse-restplan-2026-10.md` | +0/-0 prod, +0/-0 test, +317/-0 doku | SKIP-DOC | Dokumentations- oder Sonstige Anpassung (`docs/audits/deletion-proof-ist-analyse-restplan-2026-10.md`). | keine | nein | niedrig |
| 01b381f6 | 2026-10-11 | fix(crypto): restore inline deletion_proof tests to Ed25519 v3 state | `.jules/tasks/T-2026-0268.toml` | +0/-0 prod, +0/-0 test, +20/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-0268.toml`). | keine | nein | niedrig |
| ab817f7a | 2026-10-11 | fix(crypto): restore DurabilityProof and persist-before-mutate append | `.jules/tasks/T-2026-0250.toml` | +0/-0 prod, +0/-0 test, +17/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-0250.toml`). | keine | nein | niedrig |
| 2a1af3f7 | 2026-10-11 | fix: repair workspace compilation after crypto changes | `xtask/build.rs`<br>`xtask/src/check_workflow_commands.rs`<br>`xtask/src/wiring_check/mod.rs` | +71/-40 prod, +0/-0 test, +0/-0 doku | PORT-FIX | Fehlerkorrektur im Infra-/Xtask-Bereich (`xtask/build.rs`). | xtask | teilweise | niedrig |
| 3a14af69 | 2026-10-11 | fix(workspace): resolve workspace compilation after crypto repair | `.jules/tasks/T-2026-0251.toml` | +0/-0 prod, +0/-0 test, +17/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-0251.toml`). | keine | nein | niedrig |
| beb62d78 | 2026-10-11 | feat(store): implement memtable deletion witness scanner (W5-04) | `.jules/tasks/T-2026-0242.toml` | +0/-0 prod, +0/-0 test, +17/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-0242.toml`). | keine | nein | niedrig |
| 240f4f7c | 2026-10-11 | Shell-Commit | `.github/workflows/wiring-check.yml`<br>`.jules/tasks/P06.toml`<br>`.jules/tasks/T-2026-0201.toml`<br>*(+50 weitere)* | +1677/-65 prod, +6/-2 test, +2539/-0 doku | PORT-FEAT | Neue Infrastruktur-/Xtask-Funktion oder Quality-Gate (`.github/workflows/wiring-check.yml`). | xtask | teilweise | hoch |
| 8d30006c | 2026-10-10 | chore(mutation): update mutation score history [skip ci] | `docs/mutation_score_history.jsonl` | +0/-0 prod, +0/-0 test, +1/-0 doku | PORT-WIRING | Refactoring oder Infrastruktur-Anpassung (`docs/mutation_score_history.jsonl`). | keine | teilweise | niedrig |
| dd514831 | 2026-10-10 | docs: add post-mortem audit for CI gate gap in PR #4494 (#4499) | `docs/audits/gate-gap-4494-2026-10.md` | +0/-0 prod, +0/-0 test, +98/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| 56540da9 | 2026-10-10 | docs: record ADR-115 and sync ARCHITECTURE.md inventory (#4497) | `docs/decisions/ADR-115-refaktorisierungsplan-contextra-meta-und-entwicklungssystem.md` | +0/-0 prod, +0/-0 test, +48/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| 7e1d65ba | 2026-10-10 | fix(mcp): gate mock provider branch behind test-utils feature (#4494) | `.github/workflows/wiring-check.yml`<br>`.jules/tasks/P06.toml`<br>`.jules/tasks/T-2026-0201.toml`<br>*(+50 weitere)* | +65/-1677 prod, +2/-6 test, +0/-2539 doku | PORT-FIX | Fehlerkorrektur im Infra-/Xtask-Bereich (`.github/workflows/wiring-check.yml`). | xtask | teilweise | hoch |
| 84b83171 | 2026-10-10 | feat(vector): implement MmapZeroOverlay for HNSW mmap zeroing (#4492) | `.github/workflows/mutation-testing.yml`<br>`.github/workflows/shell-commit-gate.yml` | +1/-1 prod, +2/-0 test, +0/-0 doku | PORT-FEAT | Erweiterung oder neue CI-Pipeline (`.github/workflows/mutation-testing.yml`). | .github | teilweise | niedrig |
| 2983c670 | 2026-10-10 | docs(mcp): document semantics of all 15 MCP tools (#4488) | `docs/mcp/TOOL_SEMANTICS.md` | +0/-0 prod, +0/-0 test, +315/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| 6541a9ce | 2026-10-10 | chore: remove Kate swap file and ignore kate-swp/backup patterns (#4487) | `.jules/setup/.environment_script.sh.kate-swp` | +0/-0 prod, +0/-0 test, +0/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/setup/.environment_script.sh.kate-swp`). | keine | nein | niedrig |
| ace2a42a | 2026-10-09 | feat(crypto): implement LayerWitness and WitnessEvidence (W5-02) (#4485) | `.jules/tasks/T-2026-0240.toml` | +0/-0 prod, +0/-0 test, +17/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-0240.toml`). | keine | nein | niedrig |
| ddd3abd1 | 2026-10-09 | fix(store): Wal::open fail-closed before legacy rewrite (#4484) | `.jules/tasks/T-2026-0249.toml` | +0/-0 prod, +0/-0 test, +17/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-0249.toml`). | keine | nein | niedrig |
| c5cc9718 | 2026-10-09 | test(store): implement WAL tail-tolerance and resurrection test suite (A-02) (#4482) | `.jules/tasks/T-2026-0230.toml` | +0/-0 prod, +0/-0 test, +17/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-0230.toml`). | keine | nein | niedrig |
| 5d310867 | 2026-10-09 | docs(task): task card T-2026-0243 and report stopping condition (#4481) | `.jules/tasks/T-2026-0243.toml` | +0/-0 prod, +0/-0 test, +17/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| 83032b5f | 2026-10-09 | KeyRegistry: RevocationLog Mandatory - Task Card & Workspace Callers Inventory (W4-02) (#4479) | `.jules/tasks/T-2026-0237.toml` | +0/-0 prod, +0/-0 test, +17/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-0237.toml`). | keine | nein | niedrig |
| 2648c5d7 | 2026-10-09 | feat(durable-fs): implement scrub_and_remove and ScrubReceipt (#4478) | `.jules/tasks/T-2026-0239.toml` | +0/-0 prod, +0/-0 test, +17/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-0239.toml`). | keine | nein | niedrig |
| cea972df | 2026-10-09 | fix(vector): clear stale inbound edges on HNSW slot reuse (W4-06) (#4476) | `.jules/tasks/T-2026-0223.toml`<br>`.jules/tasks/T-2026-0226.toml`<br>`.jules/tasks/T-2026-0227.toml`<br>*(+4 weitere)* | +0/-0 prod, +0/-0 test, +10/-237 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-0223.toml`). | keine | nein | mittel |
| 050c6234 | 2026-10-09 | test(hnsw): verify slot reuse ghost edges and fix arena backlink cleanup (#4475) | `.jules/tasks/T-2026-0234.toml` | +0/-0 prod, +0/-0 test, +17/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-0234.toml`). | keine | nein | niedrig |
| 323f8afc | 2026-10-09 | refactor(store): replace tokio::fs::remove_file with durable_remove in compaction (#4473) | `.jules/tasks/T-2026-0232.toml` | +0/-0 prod, +0/-0 test, +17/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-0232.toml`). | keine | nein | niedrig |
| 933f6ef5 | 2026-10-09 | fix(mcp): setup_kv_bridge RevocationLog fail-closed on existing keys (#4472) | `.jules/tasks/T-2026-0231.toml` | +0/-0 prod, +0/-0 test, +17/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-0231.toml`). | keine | nein | niedrig |
| 7526b5b2 | 2026-10-09 | docs: Audit physischer Emptiness-Scanner in contextra-store (T-2026-0236) (#4471) | `.jules/tasks/T-2026-0236.toml`<br>`docs/audits/physical-scan-feasibility-2026-10.md` | +0/-0 prod, +0/-0 test, +202/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| 1b00f712 | 2026-10-09 | refactor(engine): derive CollectionId fail-closed before drop_collection deletions (#4470) | `.jules/tasks/T-2026-0233.toml` | +0/-0 prod, +0/-0 test, +17/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-0233.toml`). | keine | nein | niedrig |
| ae1a0a86 | 2026-10-09 | docs(audit): Analyse DeletionProof-Semantik und Nicht-Abdeckung (T-2026-0235) (#4469) | `.jules/tasks/T-2026-0235.toml`<br>`docs/audits/deletion-proof-coverage-semantics-2026-10.md` | +0/-0 prod, +0/-0 test, +298/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| 9a84d0ba | 2026-10-09 | test(engine): implement drop_collection tests T1-T3 from Audit A-03 (#4468) | `.jules/tasks/T-2026-0227.toml` | +0/-0 prod, +0/-0 test, +17/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| 182db152 | 2026-10-09 | test(vector): fix Tokio runtime builder enable_all panic in loom_quantizer_race_test (#4467) | `.jules/tasks/T-2026-0229.toml` | +0/-0 prod, +0/-0 test, +17/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-0229.toml`). | keine | nein | niedrig |
| b3d10828 | 2026-10-09 | fix(loom): task card for W3-02 and Out-of-scope findings for contextra-crypto (#4466) | `.jules/tasks/T-2026-0228.toml` | +0/-0 prod, +0/-0 test, +17/-0 doku | PORT-FEAT | Jules Harness Steuerung und Task Cards (`.jules/tasks/T-2026-0228.toml`). | .jules | nein | niedrig |
| 8b315f18 | 2026-10-09 | docs(crypto): Analyse-Bericht fuer Krypto-Hotpaths (A-06) (#4465) | `.jules/tasks/T-2026-0226.toml`<br>`docs/audits/crypto-hotpath-2026-10.md` | +0/-0 prod, +0/-0 test, +159/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| ef39f9f0 | 2026-10-08 | supply-chain(tokio): replace full feature with per-crate minimal features (#4464) | `.jules/tasks/T-2026-0223.toml` | +0/-0 prod, +0/-0 test, +17/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-0223.toml`). | keine | nein | niedrig |
| 8d3f8555 | 2026-10-08 | refactor(crypto): decompose deletion_proof.rs into submodules (#4462) | `.jules/tasks/T-2026-0225.toml` | +0/-0 prod, +0/-0 test, +17/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-0225.toml`). | keine | nein | niedrig |
| 330c9847 | 2026-10-08 | P06: Harden persistent key shredding and DeletionProof durability (#4461) | `.jules/tasks/P06.toml`<br>`benchmarks/contextra-bench/benches/kv_cache_effect.rs` | +0/-0 prod, +1/-1 test, +27/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/P06.toml`). | keine | nein | niedrig |
| 2fee6170 | 2026-10-08 | test(candle): add embedding semantic sanity cosine test (#4459) | `.jules/tasks/T-2026-0224.toml` | +0/-0 prod, +0/-0 test, +17/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-0224.toml`). | keine | nein | niedrig |
| 72f14e96 | 2026-10-08 | HNSW: In-memory backlink overlay for mmap-resident node repairs (#4458) | `.jules/tasks/W1-04.toml` | +0/-0 prod, +0/-0 test, +22/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/W1-04.toml`). | keine | nein | niedrig |
| 5a500e3e | 2026-10-08 | fix(wal): reject V1/V2 WAL replay in standard config without HMAC verification (#4457) | `.jules/tasks/W1-06.toml` | +0/-0 prod, +0/-0 test, +24/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/W1-06.toml`). | keine | nein | niedrig |
| 7124c9c6 | 2026-10-08 | fix(xtask): repair workflow commands gate and build collision checks (#4456) | `xtask/Cargo.toml`<br>`xtask/build.rs`<br>`xtask/src/check_workflow_commands.rs`<br>*(+2 weitere)* | +327/-56 prod, +0/-0 test, +0/-0 doku | PORT-FIX | Fehlerkorrektur im Infra-/Xtask-Bereich (`xtask/Cargo.toml`). | xtask | teilweise | hoch |
| 47ccc089 | 2026-10-08 | docs(audit): add deletion claims inventory report A-05 (#4453) | `.jules/tasks/T-2026-0215.toml`<br>`docs/audits/deletion-claims-inventory-2026-10.md` | +0/-0 prod, +0/-0 test, +1071/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| 52c8dc70 | 2026-10-08 | doc(audit): reachability analysis for ignored RUSTSEC advisories (#4451) | `.jules/tasks/T-2026-0211.toml`<br>`docs/audits/advisory-reachability-2026-10.md` | +0/-0 prod, +0/-0 test, +88/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| a1d87833 | 2026-10-08 | docs: add audit report for HNSW slots, mmap-overlay and entry-point (#4450) | `.jules/tasks/T-2026-0214.toml`<br>`docs/audits/hnsw-slots-entrypoint-2026-10.md` | +0/-0 prod, +0/-0 test, +113/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| 79ce70a3 | 2026-10-08 | docs(audit): analysis report for drop_collection race & collection_id collision (#4449) | `.jules/tasks/T-2026-0213.toml`<br>`docs/audits/drop-collection-race-2026-10.md` | +0/-0 prod, +0/-0 test, +150/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| 24791176 | 2026-10-08 | docs(store): audit report on WAL-V1 path & tail-tolerance (#4452) | `.jules/tasks/T-2026-0212.toml`<br>`docs/audits/wal-v1-tail-2026-10.md` | +0/-0 prod, +0/-0 test, +102/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| 5f5251e9 | 2026-10-08 | fix(crypto): durable RevocationLog in setup_kv_bridge and fail-closed lock handling (#4454) | `.jules/tasks/W1-03.toml`<br>`xtask/src/check_result_dropped_on_io.rs`<br>`xtask/src/harness/gc_floor_single_source.rs`<br>*(+2 weitere)* | +14/-9 prod, +5/-1 test, +23/-0 doku | PORT-FIX | Fehlerkorrektur im Infra-/Xtask-Bereich (`.jules/tasks/W1-03.toml`). | xtask | teilweise | niedrig |
| 7b197676 | 2026-10-08 | feat(xtask): add wiring-check meta-gate and workflow (#4448) | `.github/workflows/wiring-check.yml`<br>`xtask/registry.toml`<br>`xtask/src/cli/mod.rs`<br>*(+1 weitere)* | +1336/-0 prod, +0/-0 test, +0/-0 doku | PORT-WIRING | Infrastruktur-Verdrahtung oder Refactoring (`.github/workflows/wiring-check.yml`). | xtask | teilweise | hoch |
| eccf3af7 | 2026-10-08 | docs(loom): verify unwired models and create coverage inventory (#4447) | `.jules/tasks/T-2026-0205.toml`<br>`docs/audits/loom-coverage-2026-10.md` | +0/-0 prod, +0/-0 test, +84/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| 16e64ea9 | 2026-10-08 | fix(candle): load config.json and fail when safetensors missing (#4446) | `.jules/tasks/T-2026-0202.toml` | +0/-0 prod, +0/-0 test, +17/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-0202.toml`). | keine | nein | niedrig |
| 4c1afff3 | 2026-10-08 | Fix KvKeyLocks type documentation drift and lock hierarchy (#4445) | `.jules/tasks/T-2026-0204.toml` | +0/-0 prod, +0/-0 test, +17/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-0204.toml`). | keine | nein | niedrig |
| f797faf7 | 2026-10-08 | Engine: EmbeddingBackend::Candle und Stub-Beweis Härtung (#4444) | `.jules/tasks/T-2026-0203.toml` | +0/-0 prod, +0/-0 test, +17/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-0203.toml`). | keine | nein | niedrig |
| 877354d5 | 2026-10-08 | fix(mcp): fail-closed embedding provider default without explicit config (#4443) | `.jules/tasks/T-2026-0201.toml` | +0/-0 prod, +0/-0 test, +17/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/T-2026-0201.toml`). | keine | nein | niedrig |
| 808072c9 | 2026-10-08 | docs(readme): update panic inventory numbers to gate measurement (#4442) | `.jules/tasks/T-2026-0206.toml` | +0/-0 prod, +0/-0 test, +17/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| 9bf226a0 | 2026-10-08 | docs: Introduce CHANGELOG.md adhering to Keep a Changelog (#4441) | `.jules/tasks/T-2026-0207.toml` | +0/-0 prod, +0/-0 test, +17/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| a8e512d6 | 2026-10-08 | Harden RevocationLog with Persist-before-Mutate, Signed Markers, and Incremental Verification (#4440) | `.jules/tasks/W1-02.toml` | +0/-0 prod, +0/-0 test, +29/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/W1-02.toml`). | keine | nein | niedrig |
| 1b6db180 | 2026-10-08 | fix(store): consume all same-key versions in LSM N-way merge (#4439) | `.jules/tasks/W1-01.toml` | +0/-0 prod, +0/-0 test, +23/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/W1-01.toml`). | keine | nein | niedrig |
| 3a4c5192 | 2026-10-08 | HNSW commit(): Op-Normalisierung, Fehleratomaritaet, RAM-Vektor-Loeschung beim Freigeben (#4438) | `.jules/tasks/W1-05.toml` | +0/-0 prod, +0/-0 test, +22/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/W1-05.toml`). | keine | nein | niedrig |
| 79519282 | 2026-10-08 | sec(ci): prevent script injection and tighten permissions (#4437) | `.github/workflows/context-gates.yml`<br>`.github/workflows/prune-branches.yml` | +54/-10 prod, +0/-0 test, +0/-0 doku | PORT-FIX | Korrektur oder Härtung von CI-Workflows (`.github/workflows/context-gates.yml`). | .github | teilweise | niedrig |
| eaa0df92 | 2026-10-08 | test(simd): add direct kernel tests and baseline safety verification (#4436) | `governance/unsafe-safety-baseline.toml` | +2/-1 prod, +0/-0 test, +0/-0 doku | PORT-FEAT | Governance-Regeln, Rulesets oder Justfile-Kommandos (`governance/unsafe-safety-baseline.toml`). | rules | teilweise | niedrig |
| 5ca39fea | 2026-10-08 | ci: harden non-core github workflow files (#4435) | `.github/workflows/audit-integrity.yml`<br>`.github/workflows/bench-regression.yml`<br>`.github/workflows/bench.yml`<br>*(+13 weitere)* | +309/-118 prod, +0/-0 test, +0/-0 doku | PORT-FEAT | Erweiterung oder neue CI-Pipeline (`.github/workflows/audit-integrity.yml`). | .github | teilweise | mittel |
| e38fe78b | 2026-10-08 | ci: add ruleset drift verification workflow (#4434) | `.github/workflows/ruleset-drift.yml` | +185/-0 prod, +0/-0 test, +0/-0 doku | PORT-FEAT | Erweiterung oder neue CI-Pipeline (`.github/workflows/ruleset-drift.yml`). | .github | nein | niedrig |
| a0eca3f9 | 2026-10-08 | ci(unwrap): unify panic-inventory counter and eliminate direct push to main (#4433) | `.github/unwrap_baseline.txt`<br>`.github/workflows/unwrap-baseline-check.yml`<br>`.github/workflows/unwrap-gate.yml` | +79/-23 prod, +0/-0 test, +0/-0 doku | PORT-WIRING | Anpassung und Restrukturierung von CI-Workflows (`.github/unwrap_baseline.txt`). | .github | teilweise | hoch |
| 6248dc74 | 2026-10-08 | ci: update runner labels, action pins, and py lockfile check (#4432) | `.github/workflows/build-wheels.yml`<br>`.github/workflows/contextra-py-ci.yml`<br>`.github/workflows/publish-pypi.yml`<br>*(+1 weitere)* | +68/-55 prod, +0/-0 test, +0/-0 doku | PORT-WIRING | Anpassung und Restrukturierung von CI-Workflows (`.github/workflows/build-wheels.yml`). | .github | teilweise | niedrig |
| 69b395a0 | 2026-10-08 | fix(ci): repair scheduled-audit workflow YAML heredoc syntax (#4431) | `.github/scripts/render_audit_task.sh`<br>`.github/workflows/scheduled-audit.yml` | +80/-63 prod, +0/-0 test, +0/-0 doku | PORT-FIX | Korrektur oder Härtung von CI-Workflows (`.github/scripts/render_audit_task.sh`). | .github | teilweise | niedrig |
| 8f47213e | 2026-10-08 | ci: refactor rust-ci workflow with matrix, pinned toolchains and cross-platform crates (#4430) | `.github/workflows/rust-ci.yml` | +179/-103 prod, +0/-0 test, +0/-0 doku | PORT-WIRING | Anpassung und Restrukturierung von CI-Workflows (`.github/workflows/rust-ci.yml`). | .github | teilweise | niedrig |
| 757cb4a2 | 2026-10-08 | ci: instrument TSan with build-std and add native ARM SIMD cross-validation (#4429) | `.github/workflows/simd-cross-validation.yml`<br>`.github/workflows/tsan.yml` | +55/-22 prod, +0/-0 test, +0/-0 doku | PORT-FEAT | Erweiterung oder neue CI-Pipeline (`.github/workflows/simd-cross-validation.yml`). | .github | teilweise | niedrig |
| 8fa65d73 | 2026-10-08 | ci: fix and harden mutation testing pipeline (#4428) | `.github/workflows/mutation-testing.yml` | +0/-0 prod, +116/-40 test, +0/-0 doku | PORT-FEAT | Erweiterung oder neue CI-Pipeline (`.github/workflows/mutation-testing.yml`). | .github | teilweise | niedrig |
| f32623dc | 2026-10-08 | Fix and expand extended fuzzing suite across security-critical crates (#4427) | `.github/workflows/fuzz-extended.yml`<br>`justfile` | +55/-14 prod, +0/-0 test, +0/-0 doku | PORT-FIX | Korrektur oder Härtung von CI-Workflows (`.github/workflows/fuzz-extended.yml`). | .github | teilweise | hoch |
| 40543e50 | 2026-10-08 | fix(ci): normalize gate names in verdict engine and align deep-gates (#4426) | `.github/workflows/deep-gates.yml`<br>`governance/verdict-required.toml`<br>`xtask/src/harness/verdict.rs`<br>*(+1 weitere)* | +90/-31 prod, +56/-1 test, +0/-0 doku | PORT-FIX | Fehlerkorrektur im Infra-/Xtask-Bereich (`.github/workflows/deep-gates.yml`). | xtask | teilweise | niedrig |
| a27247fe | 2026-10-08 | fix: update verdict workflow schema, concurrency, triggers, and producer jobs (#4425) | `.github/workflows/verdict.yml` | +101/-27 prod, +0/-0 test, +0/-0 doku | PORT-FIX | Korrektur oder Härtung von CI-Workflows (`.github/workflows/verdict.yml`). | .github | teilweise | niedrig |
| 1a8f9c16 | 2026-10-07 | chore(unwrap-baseline): update unwrap baseline history and ratchet [skip ci] | `docs/unwrap_baseline_history.jsonl` | +0/-0 prod, +0/-0 test, +1/-0 doku | SKIP-BOT | Automatisierter Bot-Commit zur Aktualisierung des Unwrap-Baseline-Zählers. | keine | teilweise | hoch |
| eaaa88d0 | 2026-10-07 | chore(unwrap-baseline): update unwrap baseline history and ratchet [skip ci] | `docs/unwrap_baseline_history.jsonl` | +0/-0 prod, +0/-0 test, +1/-0 doku | SKIP-BOT | Automatisierter Bot-Commit zur Aktualisierung des Unwrap-Baseline-Zählers. | keine | teilweise | hoch |
| f9686e4c | 2026-10-07 | chore(unwrap-baseline): update unwrap baseline history and ratchet [skip ci] | `docs/unwrap_baseline_history.jsonl` | +0/-0 prod, +0/-0 test, +1/-0 doku | SKIP-BOT | Automatisierter Bot-Commit zur Aktualisierung des Unwrap-Baseline-Zählers. | keine | teilweise | hoch |
| 2f49cbb4 | 2026-10-07 | chore(unwrap-baseline): update unwrap baseline history and ratchet [skip ci] | `docs/unwrap_baseline_history.jsonl` | +0/-0 prod, +0/-0 test, +1/-0 doku | SKIP-BOT | Automatisierter Bot-Commit zur Aktualisierung des Unwrap-Baseline-Zählers. | keine | teilweise | hoch |
| eeae0609 | 2026-10-07 | chore(unwrap-baseline): update unwrap baseline history and ratchet [skip ci] | `docs/unwrap_baseline_history.jsonl` | +0/-0 prod, +0/-0 test, +1/-0 doku | SKIP-BOT | Automatisierter Bot-Commit zur Aktualisierung des Unwrap-Baseline-Zählers. | keine | teilweise | hoch |
| 8636c956 | 2026-10-07 | docs(audit): add dynamic verification audit report for contextra-crypto (#4412) | `docs/audits/AUDIT_deletion_proof_verification_2026-10-07.md` | +0/-0 prod, +0/-0 test, +48/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| 19d12ae4 | 2026-10-07 | Shell-Commit | `benchmarks/results/results.json`<br>`benchmarks/results/summary.md` | +0/-0 prod, +23/-29 test, +0/-0 doku | PORT-FEAT | Benchmark-Testsuite oder Performance-Parität (`benchmarks/results/results.json`). | benchmarks | teilweise | niedrig |
| 285d167e | 2026-10-07 | chore(unwrap-baseline): update unwrap baseline history and ratchet [skip ci] | `docs/unwrap_baseline_history.jsonl` | +0/-0 prod, +0/-0 test, +1/-0 doku | SKIP-BOT | Automatisierter Bot-Commit zur Aktualisierung des Unwrap-Baseline-Zählers. | keine | teilweise | hoch |
| 35c5e0c5 | 2026-10-07 | chore(unwrap-baseline): update unwrap baseline history and ratchet [skip ci] | `docs/unwrap_baseline_history.jsonl` | +0/-0 prod, +0/-0 test, +1/-0 doku | SKIP-BOT | Automatisierter Bot-Commit zur Aktualisierung des Unwrap-Baseline-Zählers. | keine | teilweise | hoch |
| b4f5c26b | 2026-10-07 | refactor(simd,wire): clean up lints, encapsulate kernels, add CpuFeatures (#4409) | `.jules/tasks/P07.toml` | +0/-0 prod, +0/-0 test, +29/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/P07.toml`). | keine | nein | niedrig |
| 1ff44716 | 2026-10-07 | fix(vector): secure mmap against length mismatch and file locks (#4408) | `.jules/tasks/P10.toml` | +0/-0 prod, +0/-0 test, +36/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/P10.toml`). | keine | nein | niedrig |
| 68a4fc06 | 2026-10-07 | chore(unwrap-baseline): update unwrap baseline history and ratchet [skip ci] | `docs/unwrap_baseline_history.jsonl` | +0/-0 prod, +0/-0 test, +1/-0 doku | SKIP-BOT | Automatisierter Bot-Commit zur Aktualisierung des Unwrap-Baseline-Zählers. | keine | teilweise | hoch |
| 28bca5a3 | 2026-10-07 | feat(xtask): clippy unsafe audit, extended IO check & gc-floor gate (#4406) | `.github/workflows/context-gates.yml`<br>`governance/gates.toml`<br>`governance/gc-floor-single-source-baseline.toml`<br>*(+9 weitere)* | +609/-108 prod, +121/-6 test, +0/-0 doku | PORT-FEAT | Neue Infrastruktur-/Xtask-Funktion oder Quality-Gate (`.github/workflows/context-gates.yml`). | xtask | teilweise | hoch |
| 584a1fa2 | 2026-10-07 | chore(unwrap-baseline): update unwrap baseline history and ratchet [skip ci] | `docs/unwrap_baseline_history.jsonl` | +0/-0 prod, +0/-0 test, +1/-0 doku | SKIP-BOT | Automatisierter Bot-Commit zur Aktualisierung des Unwrap-Baseline-Zählers. | keine | teilweise | hoch |
| 07417308 | 2026-10-07 | chore(unwrap-baseline): update unwrap baseline history and ratchet [skip ci] | `.github/unwrap_baseline.txt`<br>`docs/unwrap_baseline_history.jsonl` | +1/-1 prod, +0/-0 test, +1/-0 doku | SKIP-BOT | Automatisierter Bot-Commit zur Aktualisierung des Unwrap-Baseline-Zählers. | keine | teilweise | hoch |
| 4b246d3f | 2026-10-07 | feat(durable-fs): add contextra-durable-fs crate and mvcc floor/lease contracts (#4402) | `.jules/tasks/P00.toml`<br>`docs/generated/capability-markers.md` | +0/-0 prod, +0/-0 test, +35/-0 doku | SKIP-DOC | Jules Prompt-/Sitzungsdokumentation (`.jules/tasks/P00.toml`). | keine | teilweise | niedrig |
| 9b883d83 | 2026-10-07 | Harden WASM execution with AdmittedModule and ModuleVerifier (#4400) | `.github/unwrap_baseline.txt` | +1/-1 prod, +0/-0 test, +0/-0 doku | PORT-FEAT | Erweiterung oder neue CI-Pipeline (`.github/unwrap_baseline.txt`). | .github | teilweise | niedrig |
| 094e8d4b | 2026-10-07 | chore(unwrap-baseline): update unwrap baseline history and ratchet [skip ci] | `docs/unwrap_baseline_history.jsonl` | +0/-0 prod, +0/-0 test, +1/-0 doku | SKIP-BOT | Automatisierter Bot-Commit zur Aktualisierung des Unwrap-Baseline-Zählers. | keine | teilweise | hoch |
| 8eee10c9 | 2026-10-06 | Remove obsolete documentation files and campaign logs (#4383) | `.jules/JULES_CONTEXT.md`<br>`docs/analysis/PRODUKT-1-LSM-STORAGE-ANALYSE.md`<br>`docs/analysis/PRODUKT-2-MVCC-SSI-ANALYSE.md`<br>*(+36 weitere)* | +0/-0 prod, +0/-0 test, +1/-6417 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | teilweise | hoch |
| 21891b7f | 2026-10-06 | chore(unwrap-baseline): update unwrap baseline history and ratchet [skip ci] | `docs/unwrap_baseline_history.jsonl` | +0/-0 prod, +0/-0 test, +1/-0 doku | SKIP-BOT | Automatisierter Bot-Commit zur Aktualisierung des Unwrap-Baseline-Zählers. | keine | teilweise | hoch |
| aa50a53b | 2026-10-06 | Add Mission P05 security and correctness review report (#4382) | `docs/audits/P05_unsafe_mmap_wasm_isolation_review.md` | +0/-0 prod, +0/-0 test, +212/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| 7e5ef3e6 | 2026-10-06 | docs: add LSM-Tree correctness review report (Mission P02) (#4380) | `docs/audits/P02_LSM_Tree_Korrektheitspruefung.md` | +0/-0 prod, +0/-0 test, +204/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| 053dbcdc | 2026-10-06 | Mission P04 — DeletionProof & Crypto-Shredding Korrektheitsprüfung Audit Report (#4379) | `docs/reports/review_p04_deletion_proof_crypto_shredding.md` | +0/-0 prod, +0/-0 test, +316/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| 495b89dc | 2026-10-06 | Mission P03 Security & Correctness Audit Report (#4378) | `docs/audits/AUDIT_contextra-vector_P03_2026-10-06.md`<br>`xtask/build.rs`<br>`xtask/src/harness/unwired_ports.rs`<br>*(+2 weitere)* | +21/-12 prod, +2/-1 test, +240/-0 doku | PORT-FEAT | Neue Infrastruktur-/Xtask-Funktion oder Quality-Gate (`docs/audits/AUDIT_contextra-vector_P03_2026-10-06.md`). | xtask | teilweise | niedrig |
| 977ecfe4 | 2026-10-06 | Save Mission P01 WAL Correctness Review Report (#4381) | `docs/analysis/kampagne/P01.md` | +0/-0 prod, +0/-0 test, +322/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| 3b5e7ea5 | 2026-10-06 | chore(unwrap-baseline): update unwrap baseline history and ratchet [skip ci] | `docs/unwrap_baseline_history.jsonl` | +0/-0 prod, +0/-0 test, +1/-0 doku | SKIP-BOT | Automatisierter Bot-Commit zur Aktualisierung des Unwrap-Baseline-Zählers. | keine | teilweise | hoch |
| 56e5769a | 2026-10-06 | docs: standardize and audit all AGENTS.md files across multi-crate workspace | `benchmarks/contextra-bench/AGENTS.md` | +0/-0 prod, +6/-2 test, +0/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | teilweise | hoch |
| 31ab268f | 2026-10-06 | chore(unwrap-baseline): update unwrap baseline history and ratchet [skip ci] | `docs/unwrap_baseline_history.jsonl` | +0/-0 prod, +0/-0 test, +1/-0 doku | SKIP-BOT | Automatisierter Bot-Commit zur Aktualisierung des Unwrap-Baseline-Zählers. | keine | teilweise | hoch |
| 6df6083c | 2026-10-06 | chore(unwrap-baseline): update unwrap baseline history and ratchet [skip ci] | `docs/unwrap_baseline_history.jsonl` | +0/-0 prod, +0/-0 test, +1/-0 doku | SKIP-BOT | Automatisierter Bot-Commit zur Aktualisierung des Unwrap-Baseline-Zählers. | keine | teilweise | hoch |
| 6ec40dd4 | 2026-10-06 | chore(unwrap-baseline): update unwrap baseline history and ratchet [skip ci] | `docs/unwrap_baseline_history.jsonl` | +0/-0 prod, +0/-0 test, +1/-0 doku | SKIP-BOT | Automatisierter Bot-Commit zur Aktualisierung des Unwrap-Baseline-Zählers. | keine | teilweise | hoch |
| c1cf385c | 2026-10-06 | Require deletion proof key and return proof JSON in Python drop_collection (#4376) | `xtask/build.rs`<br>`xtask/src/harness/unwired_ports.rs`<br>`xtask/src/workspace_index.rs`<br>*(+1 weitere)* | +12/-21 prod, +1/-2 test, +0/-0 doku | PORT-FEAT | Neue Infrastruktur-/Xtask-Funktion oder Quality-Gate (`xtask/build.rs`). | xtask | teilweise | niedrig |
| ce650578 | 2026-10-06 | feat(xtask): implement orphan-symbols scanner (AUFTRAG X01) (#4374) | `benchmarks/contextra-bench/benches/competitive_kv.rs`<br>`xtask/build.rs`<br>`xtask/registry.toml`<br>*(+25 weitere)* | +790/-328 prod, +434/-142 test, +0/-0 doku | PORT-FEAT | Neue Infrastruktur-/Xtask-Funktion oder Quality-Gate (`benchmarks/contextra-bench/benches/competitive_kv.rs`). | xtask | teilweise | hoch |
| 1b8df0a1 | 2026-10-06 | chore(unwrap-baseline): update unwrap baseline history and ratchet [skip ci] | `.github/unwrap_baseline.txt`<br>`docs/unwrap_baseline_history.jsonl` | +1/-1 prod, +0/-0 test, +1/-0 doku | SKIP-BOT | Automatisierter Bot-Commit zur Aktualisierung des Unwrap-Baseline-Zählers. | keine | teilweise | hoch |
| f142c6d8 | 2026-10-06 | impl(xtask): add shared scan_common and workspace_index infrastructure (#4369) | `xtask/Cargo.toml`<br>`xtask/build.rs`<br>`xtask/registry.toml`<br>*(+4 weitere)* | +820/-2 prod, +106/-0 test, +0/-0 doku | PORT-FEAT | Neue Infrastruktur-/Xtask-Funktion oder Quality-Gate (`xtask/Cargo.toml`). | xtask | teilweise | hoch |
| f5cfd215 | 2026-10-06 | feat(xtask): implement unwired-ports harness command (#4367) | `xtask/src/harness/unwired_ports.rs`<br>`xtask/tests/harness_unwired_ports_test.rs` | +555/-0 prod, +157/-0 test, +0/-0 doku | PORT-FEAT | Neue Infrastruktur-/Xtask-Funktion oder Quality-Gate (`xtask/src/harness/unwired_ports.rs`). | xtask | nein | niedrig |
| 5bbe1e71 | 2026-10-06 | feat(xtask): implement cargo xtask repo-doctor harness command (#4361) | `xtask/src/harness/repo_doctor.rs`<br>`xtask/tests/harness_repo_doctor_test.rs` | +337/-0 prod, +66/-0 test, +0/-0 doku | PORT-FEAT | Neue Infrastruktur-/Xtask-Funktion oder Quality-Gate (`xtask/src/harness/repo_doctor.rs`). | xtask | nein | niedrig |
| dfcce0de | 2026-10-06 | feat(xtask): implement fast-diff-lint harness command (#4357) | `xtask/src/harness/fast_diff_lint.rs`<br>`xtask/tests/harness_fast_diff_lint_test.rs` | +365/-0 prod, +85/-0 test, +0/-0 doku | PORT-FEAT | Neue Infrastruktur-/Xtask-Funktion oder Quality-Gate (`xtask/src/harness/fast_diff_lint.rs`). | xtask | nein | niedrig |
| f07adb3e | 2026-10-06 | feat(xtask): implement stub-impls harness command (#4356) | `xtask/src/harness/stub_impls.rs`<br>`xtask/tests/harness_stub_impls_test.rs` | +470/-0 prod, +106/-0 test, +0/-0 doku | PORT-FEAT | Neue Infrastruktur-/Xtask-Funktion oder Quality-Gate (`xtask/src/harness/stub_impls.rs`). | xtask | nein | niedrig |
| 06910aa7 | 2026-10-06 | feat(xtask): implement doctrine-scan harness command (#4355) | `xtask/src/harness/doctrine_scan.rs`<br>`xtask/tests/harness_doctrine_scan_test.rs` | +380/-0 prod, +86/-0 test, +0/-0 doku | PORT-FEAT | Neue Infrastruktur-/Xtask-Funktion oder Quality-Gate (`xtask/src/harness/doctrine_scan.rs`). | xtask | nein | niedrig |
| 4e3392b8 | 2026-10-06 | feat(xtask): implement orphan-symbols harness command (#4353) | `xtask/src/harness/orphan_symbols.rs`<br>`xtask/tests/harness_orphan_symbols_test.rs` | +437/-0 prod, +68/-0 test, +0/-0 doku | PORT-FEAT | Neue Infrastruktur-/Xtask-Funktion oder Quality-Gate (`xtask/src/harness/orphan_symbols.rs`). | xtask | nein | niedrig |
| ace01f7c | 2026-10-06 | feat(xtask): add dependency-graph-audit harness command (#4351) | `xtask/src/harness/dependency_graph_audit.rs`<br>`xtask/tests/harness_dependency_graph_audit_test.rs` | +376/-0 prod, +80/-0 test, +0/-0 doku | PORT-FEAT | Neue Infrastruktur-/Xtask-Funktion oder Quality-Gate (`xtask/src/harness/dependency_graph_audit.rs`). | xtask | nein | niedrig |
| cd740f72 | 2026-10-06 | refactor: decouple xtask from heavy domain dependencies (#4350) | `xtask/Cargo.toml`<br>`xtask/src/bench_gate.rs`<br>`xtask/src/check_bandit_latency_budget.rs`<br>*(+3 weitere)* | +21/-256 prod, +0/-0 test, +0/-0 doku | PORT-WIRING | Infrastruktur-Verdrahtung oder Refactoring (`xtask/Cargo.toml`). | xtask | teilweise | hoch |
| adf205ab | 2026-10-05 | bench(store): establish durability parity for competitive KV benchmarks (#4310) | `benchmarks/contextra-bench/benches/competitive_kv.rs`<br>`docs/reports/competitive_kv_parity.md` | +0/-0 prod, +206/-32 test, +94/-0 doku | PORT-FEAT | Benchmark-Testsuite oder Performance-Parität (`benchmarks/contextra-bench/benches/competitive_kv.rs`). | benchmarks | teilweise | niedrig |
| c9d6d3df | 2026-10-05 | docs: add ADR-saos-query-fields evidence record (#4269) | `docs/decisions/ADR-saos-query-fields.md` | +0/-0 prod, +0/-0 test, +45/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |
| c26a4539 | 2026-10-05 | Fix compile errors in xtask harness loop guard wiring test (#4268) | `xtask/tests/harness_loop_guard_wiring_test.rs` | +0/-0 prod, +12/-0 test, +0/-0 doku | PORT-FIX | Fehlerkorrektur im Infra-/Xtask-Bereich (`xtask/tests/harness_loop_guard_wiring_test.rs`). | xtask | teilweise | niedrig |
| 3cb593f2 | 2026-10-05 | docs(audit): add RA-1 re-audit report for KV-cache-core wiring (#4267) | `docs/audits/RA1-kv-cache-kern.md` | +0/-0 prod, +0/-0 test, +88/-0 doku | SKIP-DOC | Reine Dokumentation, Audit-Bericht, ADR oder Session-Log. | keine | nein | niedrig |

---

## Anhang: Dateien

| Pfad | Fall | Konflikt in merge-tree | beteiligte Commits |
| :--- | :--- | :--- | :--- |
| `.github/scripts/render_audit_task.sh` | Fall H | nein | 69b395a0 |
| `.github/unwrap_baseline.txt` | Fall B | nein | a0eca3f9, 07417308, 9b883d83, 1b8df0a1 |
| `.github/workflows/audit-integrity.yml` | Fall B | nein | 5ca39fea |
| `.github/workflows/bench-regression.yml` | Fall B | nein | 5ca39fea |
| `.github/workflows/bench.yml` | Fall B | nein | 5ca39fea |
| `.github/workflows/build-wheels.yml` | Fall B | nein | 6248dc74 |
| `.github/workflows/chaos.yml` | Fall B | nein | 5ca39fea |
| `.github/workflows/commit-integrity-guard.yml` | Fall B | nein | 5ca39fea |
| `.github/workflows/context-gates.yml` | Fall B | nein | 79519282, 28bca5a3 |
| `.github/workflows/contextra-py-ci.yml` | Fall B | nein | 6248dc74 |
| `.github/workflows/coverage.yml` | Fall B | nein | 5ca39fea |
| `.github/workflows/deep-gates.yml` | Fall B | nein | 40543e50 |
| `.github/workflows/deny.yml` | Fall B | nein | 5ca39fea |
| `.github/workflows/flake-report.yml` | Fall B | nein | 5ca39fea |
| `.github/workflows/fuzz-extended.yml` | Fall B | nein | f32623dc |
| `.github/workflows/loom-gate.yml` | Fall B | nein | 5ca39fea |
| `.github/workflows/merge-gate.yml` | Fall B | nein | 96d35bc1, 5ca39fea |
| `.github/workflows/mutation-testing.yml` | Fall B | nein | 84b83171, 8fa65d73 |
| `.github/workflows/nucleation-recall-history.yml` | Fall B | nein | 5ca39fea |
| `.github/workflows/post-merge-verification.yml` | Fall B | nein | 5ca39fea |
| `.github/workflows/proof-correctness.yml` | Fall B | nein | 5ca39fea |
| `.github/workflows/prune-branches.yml` | Fall B | nein | 79519282 |
| `.github/workflows/publish-pypi.yml` | Fall B | nein | 6248dc74 |
| `.github/workflows/release-mcp-binary.yml` | Fall B | nein | 6248dc74 |
| `.github/workflows/ruleset-drift.yml` | Fall H | nein | e38fe78b |
| `.github/workflows/rust-ci.yml` | Fall B | nein | 96d35bc1, 7e014c0a, 8f47213e |
| `.github/workflows/scheduled-audit.yml` | Fall B | nein | 69b395a0 |
| `.github/workflows/shell-commit-gate.yml` | Fall B | nein | 84b83171, 5ca39fea |
| `.github/workflows/simd-cross-validation.yml` | Fall B | nein | 757cb4a2 |
| `.github/workflows/tsan.yml` | Fall B | nein | 757cb4a2 |
| `.github/workflows/unwrap-baseline-check.yml` | Fall B | nein | a0eca3f9 |
| `.github/workflows/unwrap-gate.yml` | Fall D | ja | a0eca3f9 |
| `.github/workflows/update-prompter-data.yml` | Fall B | nein | 5ca39fea |
| `.github/workflows/verdict.yml` | Fall B | nein | a27247fe |
| `.github/workflows/verifier-python.yml` | Fall H | nein | 1f66e19b |
| `.github/workflows/verify-kernel.yml` | Fall H | nein | 5b1b9279 |
| `.github/workflows/veto-deadline-gate.yml` | Fall B | nein | 5ca39fea |
| `.github/workflows/wiring-check.yml` | Fall H | nein | 240f4f7c, 7e1d65ba, 7b197676 |
| `.jules/ANTI_PATTERNS.md` | Fall H | nein | 97e72b69 |
| `.jules/COMMON_LLM_ERRORS.md` | Fall B | nein | 97e72b69 |
| `.jules/HARNESS_QUICKSTART.md` | Fall D | ja | 97e72b69 |
| `.jules/JULES_CONTEXT.md` | Fall D | ja | 4a88c425, 4073a18a, 97e72b69, 3e8d78e0, 8eee10c9 |
| `.jules/PARALLEL_SESSIONS.md` | Fall H | nein | 4073a18a, 97e72b69 |
| `.jules/PREAMBLE.md` | Fall D | ja | 4073a18a, 97e72b69 |
| `.jules/PROMPTER_RESTRUCTURING_PLAN.md` | Fall B | nein | 2dcd8eb4 |
| `.jules/QUICK_REFERENCE.md` | Fall H | nein | 4a88c425 |
| `.jules/SESSION_BOOTSTRAP.md` | Fall D | ja | 4a88c425, 4073a18a, 97e72b69 |
| `.jules/TASK_TEMPLATE.md` | Fall H | nein | 4a88c425 |
| `.jules/contextra-Prompter.html` | Fall B | nein | 6e22e505, 525ecd80, b8660984 |
| `.jules/harness-phases.toml` | Fall D | ja | 4a88c425, ab8ccaa1 |
| `.jules/prompter-data.json` | Fall B | nein | 2dcd8eb4 |
| `.jules/prompter-tiers.toml` | Fall B | nein | 2dcd8eb4 |
| `.jules/setup/.environment_script.sh.kate-swp` | Fall C | nein | 6541a9ce |
| `.jules/setup/environment_script.sh` | Fall B | nein | 4a88c425, 27830321, b74a95c8 |
| `.jules/setup/install-bench-deps.sh` | Fall H | nein | 27830321, b74a95c8 |
| `.jules/setup/install-fuzz-toolchain.sh` | Fall B | nein | 27830321, b74a95c8 |
| `.jules/setup/refresh-context.sh` | Fall B | nein | 13234803 |
| `.jules/setup/required-tools.toml` | Fall B | nein | 4a88c425, 5b1b9279, 27830321, b74a95c8 |
| `.jules/tasks/J-07.toml` | Fall H | nein | 51e1e765 |
| `.jules/tasks/P00.toml` | Fall H | nein | 4b246d3f |
| `.jules/tasks/P06.toml` | Fall H | nein | 240f4f7c, 7e1d65ba, 330c9847 |
| `.jules/tasks/P07.toml` | Fall H | nein | b4f5c26b |
| `.jules/tasks/P10.toml` | Fall H | nein | 1ff44716 |
| `.jules/tasks/T-2026-0201.toml` | Fall H | nein | 240f4f7c, 7e1d65ba, 877354d5 |
| `.jules/tasks/T-2026-0202.toml` | Fall H | nein | 240f4f7c, 7e1d65ba, 16e64ea9 |
| `.jules/tasks/T-2026-0203.toml` | Fall H | nein | 240f4f7c, 7e1d65ba, f797faf7 |
| `.jules/tasks/T-2026-0204.toml` | Fall H | nein | 240f4f7c, 7e1d65ba, 4c1afff3 |
| `.jules/tasks/T-2026-0205.toml` | Fall H | nein | 240f4f7c, 7e1d65ba, eccf3af7 |
| `.jules/tasks/T-2026-0206.toml` | Fall H | nein | 240f4f7c, 7e1d65ba, 808072c9 |
| `.jules/tasks/T-2026-0207.toml` | Fall H | nein | 240f4f7c, 7e1d65ba, 9bf226a0 |
| `.jules/tasks/T-2026-0211.toml` | Fall H | nein | 240f4f7c, 7e1d65ba, 52c8dc70 |
| `.jules/tasks/T-2026-0212.toml` | Fall H | nein | 240f4f7c, 7e1d65ba, 24791176 |
| `.jules/tasks/T-2026-0213.toml` | Fall H | nein | 240f4f7c, 7e1d65ba, 79ce70a3 |
| `.jules/tasks/T-2026-0214.toml` | Fall H | nein | 240f4f7c, 7e1d65ba, a1d87833 |
| `.jules/tasks/T-2026-0215.toml` | Fall H | nein | 240f4f7c, 7e1d65ba, 47ccc089 |
| `.jules/tasks/T-2026-0224.toml` | Fall H | nein | 240f4f7c, 7e1d65ba, 2fee6170 |
| `.jules/tasks/T-2026-0225.toml` | Fall H | nein | 240f4f7c, 7e1d65ba, 8d3f8555 |
| `.jules/tasks/T-2026-0230.toml` | Fall H | nein | 240f4f7c, 7e1d65ba, c5cc9718 |
| `.jules/tasks/T-2026-0231.toml` | Fall H | nein | 240f4f7c, 7e1d65ba, 933f6ef5 |
| `.jules/tasks/T-2026-0232.toml` | Fall H | nein | 240f4f7c, 7e1d65ba, 323f8afc |
| `.jules/tasks/T-2026-0233.toml` | Fall H | nein | 240f4f7c, 7e1d65ba, 1b00f712 |
| `.jules/tasks/T-2026-0234.toml` | Fall H | nein | 240f4f7c, 7e1d65ba, cea972df, 050c6234 |
| `.jules/tasks/T-2026-0235.toml` | Fall H | nein | 240f4f7c, 7e1d65ba, ae1a0a86 |
| `.jules/tasks/T-2026-0236.toml` | Fall H | nein | 240f4f7c, 7e1d65ba, 7526b5b2 |
| `.jules/tasks/T-2026-0237.toml` | Fall H | nein | 240f4f7c, 7e1d65ba, 83032b5f |
| `.jules/tasks/T-2026-0239.toml` | Fall H | nein | 240f4f7c, 7e1d65ba, 2648c5d7 |
| `.jules/tasks/T-2026-0240.toml` | Fall H | nein | 240f4f7c, 7e1d65ba, ace2a42a |
| `.jules/tasks/T-2026-0242.toml` | Fall H | nein | beb62d78 |
| `.jules/tasks/T-2026-0243.toml` | Fall H | nein | 240f4f7c, 7e1d65ba, 5d310867 |
| `.jules/tasks/T-2026-0249.toml` | Fall H | nein | 240f4f7c, 7e1d65ba, ddd3abd1 |
| `.jules/tasks/T-2026-0250.toml` | Fall H | nein | ab817f7a |
| `.jules/tasks/T-2026-0251.toml` | Fall H | nein | 3a14af69 |
| `.jules/tasks/T-2026-0268.toml` | Fall H | nein | 01b381f6 |
| `.jules/tasks/T-2026-0301.toml` | Fall H | nein | 72bf7ae3, fe05038a |
| `.jules/tasks/T-2026-0302.toml` | Fall H | nein | d3821471 |
| `.jules/tasks/T-2026-0303.toml` | Fall H | nein | 225e8b69 |
| `.jules/tasks/T-2026-0306.toml` | Fall H | nein | d8784968 |
| `.jules/tasks/T-2026-0307.toml` | Fall H | nein | 4b24db6f |
| `.jules/tasks/T-2026-0309.toml` | Fall H | nein | 73715757 |
| `.jules/tasks/T-2026-9102.toml` | Fall H | nein | 996251b5 |
| `.jules/tasks/T-2026-9103.toml` | Fall H | nein | 839fefd3 |
| `.jules/tasks/T-2026-9104a.toml` | Fall H | nein | 2a24cf21 |
| `.jules/tasks/T-2026-9104b.toml` | Fall H | nein | 2a24cf21 |
| `.jules/tasks/T-2026-9105.toml` | Fall H | nein | dec470f9 |
| `.jules/tasks/T-2026-9200.toml` | Fall H | nein | c61343f8 |
| `.jules/tasks/T-2026-9201.toml` | Fall H | nein | fd31a72e |
| `.jules/tasks/T-2026-9202.toml` | Fall H | nein | de96ee97 |
| `.jules/tasks/T-2026-9203.toml` | Fall H | nein | 5b213a3b |
| `.jules/tasks/T-2026-9204.toml` | Fall H | nein | df71c383 |
| `.jules/tasks/T-2026-9205.toml` | Fall H | nein | ffd3d0e6 |
| `.jules/tasks/T-2026-9206.toml` | Fall H | nein | ce621571 |
| `.jules/tasks/T-2026-9207.toml` | Fall H | nein | 95241b81 |
| `.jules/tasks/T-2026-9208.toml` | Fall H | nein | b713f897 |
| `.jules/tasks/T-2026-9209.toml` | Fall H | nein | edeb252d |
| `.jules/tasks/T-2026-9210.toml` | Fall H | nein | 43c1d52a |
| `.jules/tasks/T-2026-9211.toml` | Fall H | nein | 9bbb9ebc |
| `.jules/tasks/T-2026-9212.toml` | Fall H | nein | 2f68a144 |
| `.jules/tasks/T-2026-9213.toml` | Fall H | nein | 561dcec9, c3db6908 |
| `.jules/tasks/T-2026-9220.toml` | Fall H | nein | 561dcec9 |
| `.jules/tasks/T-2026-9222.toml` | Fall H | nein | e69edff4 |
| `.jules/tasks/T-2026-9223.toml` | Fall H | nein | e6b8a2c8 |
| `.jules/tasks/T-2026-9224.toml` | Fall H | nein | c3db6908 |
| `.jules/tasks/T-2026-9226.toml` | Fall H | nein | d26ac482 |
| `.jules/tasks/T-2026-9230.toml` | Fall H | nein | 2ae9a452, 28df2ae4 |
| `.jules/tasks/T-2026-9231.toml` | Fall H | nein | 8797638d |
| `.jules/tasks/T-2026-9232.toml` | Fall H | nein | ee5a102e |
| `.jules/tasks/T-2026-9250.toml` | Fall H | nein | e79f5da3 |
| `.jules/tasks/T-2026-LA01.toml` | Fall H | nein | 38e4277f |
| `.jules/tasks/T-2026-LA02.toml` | Fall H | nein | a4517868 |
| `.jules/tasks/T-2026-LA03.toml` | Fall H | nein | ad061315 |
| `.jules/tasks/T-2026-LA04.toml` | Fall H | nein | 6493e6e3 |
| `.jules/tasks/T-2026-LA05.toml` | Fall H | nein | ba4d0a72 |
| `.jules/tasks/T-2026-LA06.toml` | Fall H | nein | 3b79bb15 |
| `.jules/tasks/T-2026-LA08.toml` | Fall H | nein | ca7f7ad6 |
| `.jules/tasks/T-2026-LA09.toml` | Fall H | nein | 2abe7f78 |
| `.jules/tasks/T-2026-LA10.toml` | Fall H | nein | 57960442 |
| `.jules/tasks/T-2026-LA12.toml` | Fall H | nein | c19a81f2 |
| `.jules/tasks/T-2026-LA13.toml` | Fall H | nein | 3d4edf43 |
| `.jules/tasks/T-2026-LA14.toml` | Fall H | nein | 07f5bce0 |
| `.jules/tasks/T-2026-LA15.toml` | Fall H | nein | 9d7a3686 |
| `.jules/tasks/T-2026-LA16.toml` | Fall H | nein | 51c4d75b |
| `.jules/tasks/T-2026-LA17.toml` | Fall H | nein | 4e4d6e91 |
| `.jules/tasks/T-2026-LA18.toml` | Fall H | nein | b049a8df |
| `.jules/tasks/T-2026-LB01.toml` | Fall H | nein | e54ddb3a |
| `.jules/tasks/T-2026-LB02.toml` | Fall H | nein | 4b8c4545 |
| `.jules/tasks/T-2026-LB03.toml` | Fall H | nein | f6f340fa |
| `.jules/tasks/T-2026-LB04.toml` | Fall H | nein | a6b23465 |
| `.jules/tasks/T-2026-LB05.toml` | Fall H | nein | 55cd8b70 |
| `.jules/tasks/T-2026-LB06.toml` | Fall H | nein | 7ccbfc06 |
| `.jules/tasks/T-2026-LB07.toml` | Fall H | nein | cc6027e4 |
| `.jules/tasks/T-2026-LB08.toml` | Fall H | nein | 2a37aa40 |
| `.jules/tasks/T-2026-LB09.toml` | Fall H | nein | 687bfce8 |
| `.jules/tasks/T-2026-LB10.toml` | Fall H | nein | 92315a20 |
| `.jules/tasks/T-2026-LB11.toml` | Fall H | nein | cf09b17a |
| `.jules/tasks/T-2026-LB12.toml` | Fall H | nein | d42aaa80 |
| `.jules/tasks/T-2026-LB13.toml` | Fall H | nein | 82d7a80e |
| `.jules/tasks/T-2026-LB14.toml` | Fall H | nein | 744ffa44 |
| `.jules/tasks/T-2026-LB15.toml` | Fall H | nein | a778c647 |
| `.jules/tasks/T-2026-LB16.toml` | Fall H | nein | 47f608aa |
| `.jules/tasks/T-2026-LB19.toml` | Fall H | nein | f1007346 |
| `.jules/tasks/T-2026-LB20.toml` | Fall H | nein | c67c0c8b |
| `.jules/tasks/T-2026-LB21.toml` | Fall H | nein | 8fb718bb |
| `.jules/tasks/T-2026-LB22.toml` | Fall H | nein | a11f9e57 |
| `.jules/tasks/W-ARCH-01.toml` | Fall H | nein | 3dc67417 |
| `.jules/tasks/W-COMPLIANCE-01.toml` | Fall H | nein | 3dc67417 |
| `.jules/tasks/W-CR-01.toml` | Fall H | nein | 3dc67417 |
| `.jules/tasks/W-CR-02.toml` | Fall H | nein | 3dc67417 |
| `.jules/tasks/W-CR-04.toml` | Fall H | nein | 3dc67417 |
| `.jules/tasks/W-CR-05.toml` | Fall H | nein | 3dc67417 |
| `.jules/tasks/W-CRYPTO-01.toml` | Fall H | nein | 3dc67417 |
| `.jules/tasks/W-CRYPTO-02.toml` | Fall H | nein | 3dc67417 |
| `.jules/tasks/W-CRYPTO-04.toml` | Fall H | nein | 3dc67417 |
| `.jules/tasks/W-SYS-001.toml` | Fall H | nein | 3dc67417 |
| `.jules/tasks/W-SYS-002.toml` | Fall H | nein | 3dc67417 |
| `.jules/tasks/W1-01.toml` | Fall H | nein | 1b6db180 |
| `.jules/tasks/W1-02.toml` | Fall H | nein | 240f4f7c, 7e1d65ba, a8e512d6 |
| `.jules/tasks/W1-03.toml` | Fall H | nein | 240f4f7c, 7e1d65ba, 5f5251e9 |
| `.jules/tasks/W1-04.toml` | Fall H | nein | 240f4f7c, 7e1d65ba, 72f14e96 |
| `.jules/tasks/W1-05.toml` | Fall H | nein | 3a4c5192 |
| `.jules/tasks/W1-06.toml` | Fall H | nein | 240f4f7c, 7e1d65ba, 5a500e3e |
| `.jules/tasks/W2-03.toml` | Fall H | nein | 42cf7835 |
| `benchmarks/contextra-bench/AGENTS.md` | Fall D | ja | 6153a394, 56e5769a |
| `benchmarks/contextra-bench/Cargo.toml` | Fall D | ja | 6153a394 |
| `benchmarks/contextra-bench/README.md` | Fall B | nein | 6153a394 |
| `benchmarks/contextra-bench/benches/competitive_kv.rs` | Fall B | nein | 6153a394, ce650578, adf205ab |
| `benchmarks/contextra-bench/benches/kv_cache_effect.rs` | Fall C | nein | 6153a394, 240f4f7c, 7e1d65ba, 330c9847 |
| `benchmarks/contextra-bench/src/ann_benchmarks.rs` | Fall B | nein | 6153a394 |
| `benchmarks/contextra-bench/src/compare.rs` | Fall B | nein | 871efc73 |
| `benchmarks/contextra-bench/src/locomo.rs` | Fall B | nein | 6153a394, 871efc73 |
| `benchmarks/contextra-bench/src/long_mem_eval.rs` | Fall B | nein | 6153a394, 871efc73 |
| `benchmarks/contextra-bench/src/main.rs` | Fall D | ja | 6153a394, 83b2cc86 |
| `benchmarks/contextra-bench/src/path_rag_sweep.rs` | Fall B | nein | 6153a394 |
| `benchmarks/contextra-bench/src/regression_gate.rs` | Fall B | nein | 871efc73 |
| `benchmarks/contextra-bench/tests/external_benchmarks_test.rs` | Fall B | nein | 6153a394 |
| `benchmarks/results/results.json` | Fall B | nein | 19d12ae4 |
| `benchmarks/results/summary.md` | Fall B | nein | 19d12ae4 |
| `docs/BENCH-LOG.md` | Fall B | nein | 871efc73 |
| `docs/BENCHMARKS.md` | Fall B | nein | 871efc73 |
| `docs/EP1_CONTEXTRA_MEMORY_SPEC.md` | Fall H | nein | fceeae4b, 426dfead |
| `docs/EP2_CONTEXTRA_PROOF_SPEC.md` | Fall H | nein | fceeae4b, 426dfead |
| `docs/EP3_CONTEXTRA_RETRIEVAL_PLUS_SPEC.md` | Fall H | nein | fceeae4b, 426dfead |
| `docs/EP4_CONTEXTRA_INFERENCE_SPEC.md` | Fall H | nein | fceeae4b, 426dfead |
| `docs/EP5_CONTEXTRA_COGNITION_ROUTER_SPEC.md` | Fall H | nein | fceeae4b, 426dfead |
| `docs/EP6_CONTEXTRA_GUARD_SANDBOX_SPEC.md` | Fall H | nein | fceeae4b, 426dfead |
| `docs/SOLO_DEVELOPER_GUIDE.md` | Fall H | nein | 426dfead |
| `docs/SOURCE_OF_TRUTH.md` | Fall D | ja | c859626d, f7be670e |
| `docs/analysis/AUDIT_GESAMTSYNTHESE_2026-10-14.md` | Fall H | nein | 0c315cbf |
| `docs/analysis/AUDIT_benchmarks_DEEP_2026-10-13.md` | Fall H | nein | 68b76940 |
| `docs/analysis/AUDIT_collection-cognition-router-agent-db_DEEP_2026-10-13.md` | Fall H | nein | c041918c |
| `docs/analysis/AUDIT_composition-root_DEEP_2026-10-13.md` | Fall H | nein | 4e29fd9a |
| `docs/analysis/AUDIT_contextra-crypto_DEEP_2026-10-13.md` | Fall H | nein | aa2632bd, 5594419f |
| `docs/analysis/AUDIT_contextra-deep_DEEP_2026-10-13.md` | Fall H | nein | 3f6e2227 |
| `docs/analysis/AUDIT_contextra-mvcc_DEEP_2026-10-13.md` | Fall H | nein | d7c8fb64 |
| `docs/analysis/AUDIT_contextra-mvcc_DEEP_2026-10-14.md` | Fall H | nein | 8d738993 |
| `docs/analysis/AUDIT_contextra-sandbox_DEEP_2026-10-13.md` | Fall H | nein | 7fa590b6 |
| `docs/analysis/AUDIT_contextra-store_DEEP_2026-10-13.md` | Fall H | nein | 24cf99be |
| `docs/analysis/AUDIT_contextra-store_DEEP_2026-10-14.md` | Fall H | nein | 9230ec3f |
| `docs/analysis/AUDIT_contextra-vector_DEEP_2026-10-13.md` | Fall H | nein | 7b8f0617 |
| `docs/analysis/AUDIT_contextra-vector_DEEP_2026-10-14.md` | Fall H | nein | a9cf262b |
| `docs/analysis/AUDIT_governance_harness_DEEP_2026-10-13.md` | Fall H | nein | e9220243, 40d88863 |
| `docs/analysis/AUDIT_mcp_py_facade_DEEP_2026-10-13.md` | Fall H | nein | c0b2ffe2 |
| `docs/analysis/AUDIT_privacy-license-compliance_DEEP_2026-10-13.md` | Fall H | nein | 3db7585d |
| `docs/analysis/AUDIT_testkit-fuzz-loom-governance_DEEP_2026-10-13.md` | Fall H | nein | e9220243, c4588561 |
| `docs/analysis/AUDIT_text-graph-rank_DEEP_2026-10-13.md` | Fall H | nein | 6892c08f |
| `docs/analysis/AUDIT_unsafe-islands_DEEP_2026-10-13.md` | Fall H | nein | 60c4aaae |
| `docs/analysis/AUDIT_unsafe-islands_DEEP_2026-10-14.md` | Fall H | nein | 15b361b6 |
| `docs/analysis/AUDIT_wal_durable_fs_DEEP_2026-10-13.md` | Fall H | nein | f3731763, df3fd25f |
| `docs/analysis/MCP_PY_FACADE_AUDIT_REPORT.md` | Fall H | nein | bd576424 |
| `docs/analysis/PRODUKT-1-LSM-STORAGE-ANALYSE.md` | Fall B | nein | 8eee10c9 |
| `docs/analysis/PRODUKT-2-MVCC-SSI-ANALYSE.md` | Fall B | nein | 8eee10c9 |
| `docs/analysis/PRODUKT-3-VEKTOR-INDEX-ANALYSE.md` | Fall B | nein | 8eee10c9 |
| `docs/analysis/PRODUKT-4-KRYPTO-COMPLIANCE-ANALYSE.md` | Fall B | nein | 8eee10c9 |
| `docs/analysis/kampagne/J-00.md` | Fall B | nein | 8eee10c9 |
| `docs/analysis/kampagne/J-01.md` | Fall B | nein | 8eee10c9 |
| `docs/analysis/kampagne/J-10.md` | Fall B | nein | 8eee10c9 |
| `docs/analysis/kampagne/J-11.md` | Fall B | nein | 8eee10c9 |
| `docs/analysis/kampagne/J-12.md` | Fall B | nein | 8eee10c9 |
| `docs/analysis/kampagne/J-14.md` | Fall B | nein | 8eee10c9 |
| `docs/analysis/kampagne/J-15.md` | Fall B | nein | 8eee10c9 |
| `docs/analysis/kampagne/J-16.md` | Fall B | nein | 8eee10c9 |
| `docs/analysis/kampagne/J-17.md` | Fall B | nein | 8eee10c9 |
| `docs/analysis/kampagne/J-19.md` | Fall B | nein | 8eee10c9 |
| `docs/analysis/kampagne/J-20.md` | Fall B | nein | 8eee10c9 |
| `docs/analysis/kampagne/J-21.md` | Fall B | nein | 8eee10c9 |
| `docs/analysis/kampagne/J-23.md` | Fall B | nein | 8eee10c9 |
| `docs/analysis/kampagne/J-24.md` | Fall B | nein | 8eee10c9 |
| `docs/analysis/kampagne/J-25.md` | Fall B | nein | 8eee10c9 |
| `docs/analysis/kampagne/J-26.md` | Fall B | nein | 8eee10c9 |
| `docs/analysis/kampagne/J-27.md` | Fall B | nein | 8eee10c9 |
| `docs/analysis/kampagne/J-41.md` | Fall B | nein | 8eee10c9 |
| `docs/analysis/kampagne/J-42.md` | Fall B | nein | 8eee10c9 |
| `docs/analysis/kampagne/M00.md` | Fall C | nein | 8eee10c9 |
| `docs/analysis/kampagne/M00_reaudit.sh` | Fall C | nein | 8eee10c9 |
| `docs/analysis/kampagne/M04.md` | Fall C | nein | 8eee10c9 |
| `docs/analysis/kampagne/M07.md` | Fall C | nein | 8eee10c9 |
| `docs/analysis/kampagne/M08.md` | Fall C | nein | 8eee10c9 |
| `docs/analysis/kampagne/N00.md` | Fall C | nein | 8eee10c9 |
| `docs/archive/14-10-26/ADVANCED_ALGORITHMS_SPEC_v1.md` | Fall H | nein | fceeae4b |
| `docs/archive/14-10-26/ARCHITECTURE_PLAN_v1.md` | Fall H | nein | fceeae4b |
| `docs/archive/14-10-26/ARCHITEKTUR_MASTERPLAN_v2.md` | Fall H | nein | fceeae4b |
| `docs/archive/14-10-26/ARCHITEKTUR_OPTIMIERUNG_v1.md` | Fall H | nein | fceeae4b |
| `docs/archive/14-10-26/CONTEXTRA_FEINSPEZIFIKATION_CODE_BLUEPRINTS_v1.md` | Fall H | nein | fceeae4b |
| `docs/archive/14-10-26/CONTEXTRA_SYSTEM_DESIGN_SPECIFICATION_v1.md` | Fall H | nein | fceeae4b |
| `docs/archive/14-10-26/DISJUNKTIVE_MIKRO_IMPLEMENTIERUNGSSPEZIFIKATION_v1.md` | Fall H | nein | fceeae4b |
| `docs/archive/14-10-26/IMPLEMENTIERUNGSSPEZIFIKATION.md` | Fall H | nein | fceeae4b |
| `docs/archive/14-10-26/IMPLEMENTIERUNGSSPEZIFIKATION_ENDPRODUKTE_DISJUNKTIV.md` | Fall H | nein | fceeae4b |
| `docs/archive/14-10-26/JULES_AUTONOMOUS_AGENT_SPECIFICATION.md` | Fall H | nein | fceeae4b |
| `docs/archive/14-10-26/LOESCHNACHWEIS_SYSTEMSPEZIFIKATION_v2.md` | Fall H | nein | fceeae4b |
| `docs/archive/14-10-26/SOTA_ALGORITHMIC_FEINSPEZIFIKATION_2026.md` | Fall H | nein | fceeae4b |
| `docs/archive/14-10-26/specs/EP1_CONTEXTRA_MEMORY_SPEC.md` | Fall H | nein | fceeae4b |
| `docs/archive/14-10-26/specs/EP2_CONTEXTRA_PROOF_SPEC.md` | Fall H | nein | fceeae4b |
| `docs/archive/14-10-26/specs/EP3_CONTEXTRA_RETRIEVAL_PLUS_SPEC.md` | Fall H | nein | fceeae4b |
| `docs/archive/14-10-26/specs/EP4_CONTEXTRA_INFERENCE_SPEC.md` | Fall H | nein | fceeae4b |
| `docs/archive/14-10-26/specs/EP5_CONTEXTRA_COGNITION_ROUTER_SPEC.md` | Fall H | nein | fceeae4b |
| `docs/archive/14-10-26/specs/EP6_CONTEXTRA_GUARD_SANDBOX_SPEC.md` | Fall H | nein | fceeae4b |
| `docs/archive/14-10-26/specs/SPEZIFIKATION_EP3_CONTEXTRA_RETRIEVAL_PLUS.md` | Fall H | nein | fceeae4b |
| `docs/archive/14-10-26/specs/SPEZIFIKATION_EP4_CONTEXTRA_INFERENCE.md` | Fall H | nein | fceeae4b |
| `docs/archive/14-10-26/specs/SPEZIFIKATION_EP5_CONTEXTRA_COGNITION.md` | Fall H | nein | fceeae4b |
| `docs/archive/14-10-26/specs/specs-3/SPEZIFIKATION_EP1_CONTEXTRA_MEMORY.md` | Fall H | nein | fceeae4b |
| `docs/archive/14-10-26/specs/specs-3/SPEZIFIKATION_EP2_CONTEXTRA_PROOF.md` | Fall H | nein | fceeae4b |
| `docs/archive/14-10-26/specs/specs-3/SPEZIFIKATION_EP3_CONTEXTRA_RETRIEVAL_PLUS.md` | Fall H | nein | fceeae4b |
| `docs/archive/14-10-26/specs/specs-3/SPEZIFIKATION_EP4_CONTEXTRA_INFERENCE.md` | Fall H | nein | fceeae4b |
| `docs/archive/14-10-26/specs/specs-3/SPEZIFIKATION_EP5_CONTEXTRA_COGNITION.md` | Fall H | nein | fceeae4b |
| `docs/archive/14-10-26/specs/specs-3/SPEZIFIKATION_EP6_CONTEXTRA_GUARD.md` | Fall H | nein | fceeae4b |
| `docs/audits/AUDIT_adaptive-controller_DEEP_2026-10-12.md` | Fall H | nein | 320ad0ce |
| `docs/audits/AUDIT_bm25_DEEP_2026-10-12.md` | Fall H | nein | d5cef864 |
| `docs/audits/AUDIT_compaction_DEEP_2026-10-12.md` | Fall H | nein | 6f13b854 |
| `docs/audits/AUDIT_contextra-adapt_2026-10-12.md` | Fall H | nein | 24628bd8 |
| `docs/audits/AUDIT_contextra-agent_2026-10-12.md` | Fall H | nein | 4b7ddfe5 |
| `docs/audits/AUDIT_contextra-audit-export_2026-10-12.md` | Fall H | nein | 17f192a2 |
| `docs/audits/AUDIT_contextra-avv-generator_2026-10-12.md` | Fall H | nein | a935954e |
| `docs/audits/AUDIT_contextra-cognition_2026-03-06.md` | Fall H | nein | 7e92a581 |
| `docs/audits/AUDIT_contextra-cognition_2026-10-11.md` | Fall H | nein | 3e8d78e0 |
| `docs/audits/AUDIT_contextra-core_2026-10-12.md` | Fall H | nein | 00fbec75 |
| `docs/audits/AUDIT_contextra-crypto_2026-10-12.md` | Fall H | nein | d18d119c |
| `docs/audits/AUDIT_contextra-db_2026-10-11.md` | Fall H | nein | 96159973 |
| `docs/audits/AUDIT_contextra-db_2026-10-12.md` | Fall H | nein | 29a93dd2 |
| `docs/audits/AUDIT_contextra-engine_2026-10-12.md` | Fall H | nein | 7f0ebe29 |
| `docs/audits/AUDIT_contextra-facade_2026-10-12.md` | Fall H | nein | 6cc33543 |
| `docs/audits/AUDIT_contextra-graph_2026-10-12.md` | Fall H | nein | 9cf8ce1d |
| `docs/audits/AUDIT_contextra-infer-candle_2026-10-11.md` | Fall H | nein | 536e891e |
| `docs/audits/AUDIT_contextra-infer-candle_2026-10-12.md` | Fall H | nein | ceaeb33a |
| `docs/audits/AUDIT_contextra-infer-ollama_2026-10-12.md` | Fall H | nein | 0fc8b0ec |
| `docs/audits/AUDIT_contextra-infer-onnx_2026-10-12.md` | Fall H | nein | 4747cc23 |
| `docs/audits/AUDIT_contextra-kvcache_2026-10-12.md` | Fall H | nein | 6d882691 |
| `docs/audits/AUDIT_contextra-license_2026-10-12.md` | Fall H | nein | 4b2821b1 |
| `docs/audits/AUDIT_contextra-mcp_2026-10-12.md` | Fall H | nein | 3ada91bc |
| `docs/audits/AUDIT_contextra-mvcc_2026-10-12.md` | Fall H | nein | 1afe4ca4, 9e8ce5fa |
| `docs/audits/AUDIT_contextra-ports_2026-10-12.md` | Fall H | nein | 24790b59 |
| `docs/audits/AUDIT_contextra-py_2026-10-12.md` | Fall H | nein | 98d66e3d |
| `docs/audits/AUDIT_contextra-rank_2026-10-11.md` | Fall H | nein | 298de109 |
| `docs/audits/AUDIT_contextra-rank_2026-10-12.md` | Fall H | nein | 5273990d |
| `docs/audits/AUDIT_contextra-router_2026-10-12.md` | Fall H | nein | fa31e8ad |
| `docs/audits/AUDIT_contextra-sandbox_2026-10-12.md` | Fall H | nein | d0907191 |
| `docs/audits/AUDIT_contextra-simd_2026-10-12.md` | Fall H | nein | 2d6161c5 |
| `docs/audits/AUDIT_contextra-store_2026-10-11.md` | Fall H | nein | a95c43ed |
| `docs/audits/AUDIT_contextra-store_2026-10-12.md` | Fall H | nein | c59626bd |
| `docs/audits/AUDIT_contextra-sys_2026-10-12.md` | Fall H | nein | d5c45033 |
| `docs/audits/AUDIT_contextra-testkit_2026-10-12.md` | Fall H | nein | 6fa38752 |
| `docs/audits/AUDIT_contextra-text_2026-10-12.md` | Fall H | nein | a3218ad1 |
| `docs/audits/AUDIT_contextra-types_2026-10-12.md` | Fall H | nein | d23272d6 |
| `docs/audits/AUDIT_contextra-vector_2026-10-12.md` | Fall H | nein | 7d905f2a |
| `docs/audits/AUDIT_contextra-vector_P03_2026-10-06.md` | Fall H | nein | 495b89dc |
| `docs/audits/AUDIT_contextra-wire_2026-10-12.md` | Fall H | nein | 2ab2e3b7 |
| `docs/audits/AUDIT_crypto-kernel_DEEP_2026-10-12.md` | Fall H | nein | a72ce152 |
| `docs/audits/AUDIT_csr-ppr_DEEP_2026-10-12.md` | Fall H | nein | 62d67a11 |
| `docs/audits/AUDIT_deletion_proof_verification_2026-10-07.md` | Fall H | nein | 8636c956 |
| `docs/audits/AUDIT_fuzz-campaign_2026-10-12.md` | Fall H | nein | 6ed22ce4 |
| `docs/audits/AUDIT_hnsw_DEEP_2026-10-12.md` | Fall H | nein | 54a733b3 |
| `docs/audits/AUDIT_kdf-migration-caller-inventory_2026-10-12.md` | Fall H | nein | 6da9fecd |
| `docs/audits/AUDIT_kvcache_DEEP_2026-10-12.md` | Fall H | nein | 09b58833 |
| `docs/audits/AUDIT_lsm-core_DEEP_2026-10-12.md` | Fall H | nein | 19df22ec |
| `docs/audits/AUDIT_mutation-testing_2026-10-12.md` | Fall H | nein | 6db2ca3b |
| `docs/audits/AUDIT_release-gate_v0.1.0_2026-10-12.md` | Fall H | nein | 815f5ee9 |
| `docs/audits/AUDIT_ring-dag-integrity_2026-10-12.md` | Fall H | nein | 998a7642 |
| `docs/audits/AUDIT_simd-kernels_DEEP_2026-10-12.md` | Fall H | nein | 6ee2a5bd |
| `docs/audits/AUDIT_soll-ist-gap_2026-10-12.md` | Fall H | nein | a832a6e7 |
| `docs/audits/AUDIT_sstable_DEEP_2026-10-12.md` | Fall H | nein | c0fcc5b0 |
| `docs/audits/AUDIT_wal-subsystem_DEEP_2026-10-12.md` | Fall H | nein | 00f4cf45 |
| `docs/audits/AUDIT_zero-panic-unsafe_WORKSPACE_2026-10-12.md` | Fall H | nein | d987b099 |
| `docs/audits/MANIFEST_RANK_2026-10-12.md` | Fall H | nein | 0efdc52e |
| `docs/audits/MUTANTS_crypto_2026-10-12.md` | Fall H | nein | 448f6f16 |
| `docs/audits/P02_LSM_Tree_Korrektheitspruefung.md` | Fall H | nein | 7e5ef3e6 |
| `docs/audits/P05_unsafe_mmap_wasm_isolation_review.md` | Fall H | nein | aa50a53b |
| `docs/audits/RA1-kv-cache-kern.md` | Fall H | nein | 3cb593f2 |
| `docs/audits/advisory-reachability-2026-10.md` | Fall H | nein | 240f4f7c, 7e1d65ba, 52c8dc70 |
| `docs/audits/deletion-claims-inventory-2026-10.md` | Fall H | nein | 240f4f7c, 7e1d65ba, 47ccc089 |
| `docs/audits/deletion-proof-coverage-semantics-2026-10.md` | Fall H | nein | 240f4f7c, 7e1d65ba, ae1a0a86 |
| `docs/audits/drop-collection-race-2026-10.md` | Fall H | nein | 240f4f7c, 7e1d65ba, 79ce70a3 |
| `docs/audits/gate-gap-4494-2026-10.md` | Fall H | nein | d13013ca |
| `docs/audits/hnsw-slots-entrypoint-2026-10.md` | Fall H | nein | 240f4f7c, 7e1d65ba, a1d87833 |
| `docs/audits/loom-coverage-2026-10.md` | Fall H | nein | 240f4f7c, 7e1d65ba, eccf3af7 |
| `docs/audits/physical-scan-feasibility-2026-10.md` | Fall H | nein | 240f4f7c, 7e1d65ba, 7526b5b2 |
| `docs/audits/wal-v1-tail-2026-10.md` | Fall H | nein | 240f4f7c, 7e1d65ba, 24791176 |
| `docs/baseline/2026-10.md` | Fall H | nein | b8ac557d |
| `docs/decisions/ADR-115-refaktorisierungsplan-contextra-meta-und-entwicklungssystem.md` | Fall H | nein | 56540da9 |
| `docs/decisions/ADR-E01-ep1-off-assured-attested-ep2.md` | Fall H | nein | fceeae4b |
| `docs/decisions/ADR-E02-einheitliches-ringmodell.md` | Fall H | nein | fceeae4b |
| `docs/decisions/ADR-E03-drei-unsafe-inseln.md` | Fall H | nein | fceeae4b |
| `docs/decisions/ADR-E04-kryptosuite-nach-sec6-1.md` | Fall H | nein | fceeae4b |
| `docs/decisions/ADR-E05-beweisformat-v4-tlv.md` | Fall H | nein | fceeae4b |
| `docs/decisions/ADR-E06-v3-beweiserzeugung-entfernen.md` | Fall H | nein | fceeae4b |
| `docs/decisions/ADR-E07-14-deletion-layer-varianten.md` | Fall H | nein | fceeae4b |
| `docs/decisions/ADR-E08-zweistufiges-crypto-erase.md` | Fall H | nein | fceeae4b |
| `docs/decisions/ADR-E09-commit-punkt-durables-ledger-append.md` | Fall H | nein | fceeae4b |
| `docs/decisions/ADR-E10-purgestate-repr-u8-enum.md` | Fall H | nein | fceeae4b |
| `docs/decisions/ADR-E11-kvdeletemode-default-tombstoneonly.md` | Fall H | nein | fceeae4b |
| `docs/decisions/ADR-E12-fence-protokoll-b-drain-fence.md` | Fall H | nein | fceeae4b |
| `docs/decisions/ADR-E13-kein-crate-contextra-erasure-in-ep1.md` | Fall H | nein | fceeae4b |
| `docs/decisions/ADR-E14-facade-default-fast-engine-default-empty.md` | Fall H | nein | fceeae4b |
| `docs/decisions/ADR-E15-contextra-core-ersatzlos-entfernen.md` | Fall H | nein | fceeae4b |
| `docs/decisions/ADR-E16-contextra-crypto-ohne-shim-splitten.md` | Fall H | nein | fceeae4b |
| `docs/decisions/ADR-E17-formatversionierung-upgrade-on-open.md` | Fall H | nein | fceeae4b |
| `docs/decisions/ADR-E18-mcp-mandant-prozessstart-gebunden.md` | Fall H | nein | fceeae4b |
| `docs/decisions/ADR-E19-mcp-dispatcher-spawn-blocking.md` | Fall H | nein | fceeae4b |
| `docs/decisions/ADR-E20-durabilitymode-erasuremode-matrix.md` | Fall H | nein | fceeae4b |
| `docs/decisions/ADR-E21-ledger-speichert-nach-purged-nur-digests.md` | Fall H | nein | fceeae4b |
| `docs/decisions/ADR-E22-fliesskomma-politik-isa-reproduzierbarkeit.md` | Fall H | nein | fceeae4b |
| `docs/decisions/ADR-E23-dibud-nur-auf-materialisierten-listen.md` | Fall H | nein | fceeae4b |
| `docs/decisions/ADR-E24-bandit-sparse-jl-reset-re-sketch.md` | Fall H | nein | fceeae4b |
| `docs/decisions/ADR-E25-openie-ergebnisse-als-ereignis-cachen.md` | Fall H | nein | fceeae4b |
| `docs/decisions/ADR-E26-kv-cache-prefix-trie-je-mandant.md` | Fall H | nein | fceeae4b |
| `docs/decisions/ADR-E27-find-k-paths-pub-crate.md` | Fall H | nein | fceeae4b |
| `docs/decisions/ADR-E28-lock-reihenfolge-global.md` | Fall H | nein | fceeae4b |
| `docs/decisions/ADR-E29-lizenz-steuert-nur-beweisausstellung.md` | Fall H | nein | fceeae4b |
| `docs/decisions/ADR-E30-snapshot-pin-policy-max-age.md` | Fall H | nein | fceeae4b |
| `docs/decisions/ADR-E31-checkpoints-scrub-restore-barriere.md` | Fall H | nein | fceeae4b |
| `docs/decisions/ADR-E32-licensegate-port-contextra-license-ausserhalb-ep1.md` | Fall H | nein | fceeae4b |
| `docs/decisions/ADR-E33-benchmark-gate-instruktionszaehler.md` | Fall H | nein | fceeae4b |
| `docs/decisions/ADR-E34-unwrap-ratchet-governance-ratchet-toml.md` | Fall H | nein | fceeae4b |
| `docs/decisions/ADR-E35-physicalscope-konstruktor-port-feature.md` | Fall H | nein | fceeae4b |
| `docs/decisions/ADR-E36-rangordnung-der-quellen.md` | Fall H | nein | fceeae4b |
| `docs/decisions/ADR-E37-loeschumfang-alle-abgeleiteten-speicher.md` | Fall H | nein | fceeae4b |
| `docs/decisions/ADR-PARK-01.md` | Fall H | nein | 6e1c6d1e |
| `docs/decisions/ADR-RING-crypto-durable-fs.md` | Fall H | nein | 1c9bcd83 |
| `docs/decisions/ADR-SCOPE-FREEZE-2026-10.md` | Fall H | nein | 9908339b |
| `docs/decisions/ADR-T01-P29-kein-globaler-veraenderlicher-zustand.md` | Fall H | nein | 25fe0dfb |
| `docs/decisions/ADR-T02-INV-KMS-1-kms-provider-verdrahtung.md` | Fall H | nein | 25fe0dfb |
| `docs/decisions/ADR-T03-INV-DELETION-WIRE-1-loeschpfad-verdrahtung.md` | Fall H | nein | 25fe0dfb |
| `docs/decisions/ADR-T04-INV-SSI-ACTIVE-1-ssi-durchsetzung.md` | Fall H | nein | 25fe0dfb |
| `docs/decisions/ADR-T05-INV-DOC-SINGLE-SOURCE-1-doc-single-source.md` | Fall H | nein | 25fe0dfb |
| `docs/decisions/ADR-T06-INV-RING0-DETERMINISM-1-ring0-determinismus.md` | Fall H | nein | 25fe0dfb |
| `docs/decisions/ADR-VETO-REVIEW-F02-OP03-2026-10.md` | Fall H | nein | 7bf918ed |
| `docs/decisions/ADR-saos-query-fields.md` | Fall H | nein | c9d6d3df |
| `docs/decisions/README.md` | Fall B | nein | 25fe0dfb, e9220243, 3dc67417, 9908339b, 88cfe2f0 |
| `docs/decisions/claims/claims-forbidden.toml` | Fall H | nein | b1998340 |
| `docs/deletion/CANARY_ORACLE.md` | Fall C | nein | f3adf969 |
| `docs/deletion/KEYFAMILIES.toml` | Fall C | nein | 9bbb9ebc, cd3edcce |
| `docs/deletion/LEGAL_MATRIX_ERASURE.md` | Fall C | nein | b1998340, c67c0c8b |
| `docs/deletion/OPERATIONS_BASELINE_ERASURE.md` | Fall C | nein | c67c0c8b |
| `docs/deletion/POLICY_RULES.md` | Fall C | nein | 5b213a3b |
| `docs/deletion/vectors/v4_proof_sample.json` | Fall C | nein | c3db6908 |
| `docs/gaps/pub_fn_ohne_aufrufer.csv` | Fall H | nein | 11ee512c, ce52bda8 |
| `docs/gaps/undokumentiert_streng.csv` | Fall H | nein | 11ee512c, ce52bda8 |
| `docs/gates/verify.md` | Fall H | nein | 5b1b9279 |
| `docs/generated/capability-markers.md` | Fall B | nein | bbb605cd, 4b246d3f |
| `docs/mcp/TOOL_SEMANTICS.md` | Fall H | nein | 2983c670 |
| `docs/mutation_score_history.jsonl` | Fall B | nein | e8a40e6a, bed02adf, 8d30006c |
| `docs/parking/ENTFERNUNG.md` | Fall H | nein | e40902ac |
| `docs/parking/INVENTORY.md` | Fall H | nein | 59dacde0, e40902ac, 2c9e7d3d |
| `docs/refactor/core-mapping.json` | Fall H | nein | d929ce62, 090339f8, 49036c0b, bf089c57 |
| `docs/refactor/core-mapping.md` | Fall H | nein | d929ce62, 090339f8, 49036c0b, bf089c57 |
| `docs/reports/compilability_report_2026-09-25.md` | Fall B | nein | 8eee10c9 |
| `docs/reports/deletion_proof_verifiability_report.md` | Fall B | nein | 8eee10c9 |
| `docs/reports/kv_cache_effect.md` | Fall C | nein | 8eee10c9 |
| `docs/reports/performance_baseline_2026-09-25.md` | Fall B | nein | 8eee10c9 |
| `docs/reports/scale_100k_measurement_2026-09-30.md` | Fall B | nein | 8eee10c9 |
| `docs/reports/std_fs_audit_store.md` | Fall B | nein | 8eee10c9 |
| `docs/scope/CORE_SCOPE.md` | Fall H | nein | 0dcbd4bf |
| `docs/scope/PLATFORM_SCOPE.md` | Fall H | nein | 0dcbd4bf |
| `docs/scope/RING_DEBT.md` | Fall H | nein | 0dcbd4bf |
| `docs/scope/ROBUSTHEITSVERTRAG.md` | Fall H | nein | 0dcbd4bf |
| `docs/spec/22-jules-autonomous-agent-specification.md` | Fall H | nein | f7be670e |
| `docs/spec/CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md` | Fall D | ja | 331c38c5, 8fb718bb, 561dcec9, 75c687b6, 449784f4 |
| `docs/spec/CONTEXTRA_SPEZIFIKATION_v15.md` | Fall D | ja | 561dcec9, 75c687b6, 449784f4 |
| `docs/spec/GESAMTSPEZIFIKATION-v4.md` | Fall H | nein | 1a028782 |
| `docs/spec/INDEX.md` | Fall B | nein | f7be670e |
| `docs/spec/SOURCE_OF_TRUTH_DIFF.md` | Fall H | nein | 561dcec9, 75c687b6 |
| `docs/unwrap_baseline_history.jsonl` | Fall D | ja | 1a8f9c16, eaaa88d0, f9686e4c, 2f49cbb4, eeae0609, 285d167e, 35c5e0c5, 68a4fc06, 584a1fa2, 07417308, 094e8d4b, 21891b7f, 3b5e7ea5, 31ab268f, 6df6083c, 6ec40dd4, 1b8df0a1 |
| `docs/verification/INV_MATRIX.md` | Fall H | nein | 331c38c5, e8a40e6a, 932616ff |
| `docs/verification/MUTATION_BASELINE.md` | Fall B | nein | e8a40e6a |
| `docs/verification/VERIFIER_SPEC.md` | Fall H | nein | 8fb718bb, 1f66e19b |
| `docs/verification/verify_export.py` | Fall B | nein | 1f66e19b |
| `governance/gates.toml` | Fall D | ja | 525ecd80, b8660984, 28bca5a3 |
| `governance/gc-floor-single-source-baseline.toml` | Fall H | nein | 28bca5a3 |
| `governance/io-result-baseline.toml` | Fall H | nein | 28bca5a3 |
| `governance/unsafe-safety-baseline.toml` | Fall D | nein | eaa0df92 |
| `governance/verdict-required.toml` | Fall D | nein | 40543e50 |
| `justfile` | Fall D | ja | 5b1b9279, 96d35bc1, 7e014c0a, f32623dc |
| `rules/benchmarking.md` | Fall H | nein | baa0b1d2 |
| `rules/fuzzing.md` | Fall H | nein | baa0b1d2 |
| `rules/loom-miri.md` | Fall H | nein | 5d73ffd1, baa0b1d2 |
| `rules/mutation-testing.md` | Fall H | nein | baa0b1d2 |
| `rules/simd_safety.md` | Fall B | nein | 5d73ffd1, 3dc67417 |
| `rules/tag_taxonomy.md` | Fall H | nein | 5d73ffd1, 230372f0 |
| `xtask/Cargo.toml` | Fall B | nein | 47d5b308, 240f4f7c, 7e1d65ba, 7124c9c6, f142c6d8, cd740f72 |
| `xtask/build.rs` | Fall B | nein | 11ee512c, 5b1b9279, 47d5b308, 2a1af3f7, 240f4f7c, 7e1d65ba, 7124c9c6, 495b89dc, c1cf385c, ce650578, f142c6d8 |
| `xtask/registry.toml` | Fall D | ja | bf089c57, b5796772, 230372f0, 5b1b9279, 396b795f, 240f4f7c, 7e1d65ba, 7b197676, 28bca5a3, ce650578, f142c6d8 |
| `xtask/src/bench_gate.rs` | Fall B | nein | cd740f72 |
| `xtask/src/check_bandit_latency_budget.rs` | Fall B | nein | cd740f72 |
| `xtask/src/check_claims.rs` | Fall H | nein | b1998340 |
| `xtask/src/check_exception_ledger.rs` | Fall H | nein | c859626d |
| `xtask/src/check_loom_mocks.rs` | Fall H | nein | 5b1b9279, 7e014c0a |
| `xtask/src/check_mutation_score_gate.rs` | Fall B | nein | 4b7709e6, 5b1b9279 |
| `xtask/src/check_result_dropped_on_io.rs` | Fall B | nein | 240f4f7c, 7e1d65ba, 5f5251e9, 28bca5a3 |
| `xtask/src/check_ring_capabilities_consistency.rs` | Fall B | nein | c859626d |
| `xtask/src/check_workflow_commands.rs` | Fall B | nein | 2a1af3f7, 240f4f7c, 7e1d65ba, 7124c9c6, ce650578 |
| `xtask/src/cli/bench.rs` | Fall D | ja | cd740f72 |
| `xtask/src/cli/checks.rs` | Fall D | ja | c859626d, b1998340, 28bca5a3, ce650578, cd740f72 |
| `xtask/src/cli/mod.rs` | Fall D | ja | c859626d, 11ee512c, b1998340, 7e014c0a, 240f4f7c, 7e1d65ba, 7b197676, 28bca5a3 |
| `xtask/src/context_pack.rs` | Fall B | nein | 13234803 |
| `xtask/src/gen_prompter_data.rs` | Fall B | nein | 5b1b9279, 2dcd8eb4 |
| `xtask/src/harness/blast_radius.rs` | Fall D | ja | ce650578 |
| `xtask/src/harness/check_loom_mocks.rs` | Fall H | nein | 5b1b9279 |
| `xtask/src/harness/check_no_downcast.rs` | Fall H | nein | b5796772 |
| `xtask/src/harness/check_no_stubs.rs` | Fall H | nein | 11ee512c |
| `xtask/src/harness/check_port_features.rs` | Fall H | nein | b5796772 |
| `xtask/src/harness/check_port_names.rs` | Fall H | nein | 11ee512c |
| `xtask/src/harness/context_card.rs` | Fall H | nein | 13234803 |
| `xtask/src/harness/deletion_guarantee_gate.rs` | Fall H | nein | b5796772 |
| `xtask/src/harness/dependency_graph_audit.rs` | Fall H | nein | ce650578, ace01f7c |
| `xtask/src/harness/detect_silent_reverts.rs` | Fall H | nein | 14530cb3 |
| `xtask/src/harness/doctrine_scan.rs` | Fall H | nein | ce650578, 06910aa7 |
| `xtask/src/harness/fast_diff_lint.rs` | Fall H | nein | ce650578, dfcce0de |
| `xtask/src/harness/find_unwired_gaps.rs` | Fall H | nein | 14530cb3 |
| `xtask/src/harness/fuzz_smoke.rs` | Fall B | nein | 5b1b9279 |
| `xtask/src/harness/gate_arch.rs` | Fall D | ja | ce650578 |
| `xtask/src/harness/gate_guard.rs` | Fall D | ja | ce650578 |
| `xtask/src/harness/gate_integrity.rs` | Fall D | ja | ce650578 |
| `xtask/src/harness/gc_floor_single_source.rs` | Fall H | nein | 240f4f7c, 7e1d65ba, 5f5251e9, 28bca5a3 |
| `xtask/src/harness/generate_core_mapping.rs` | Fall H | nein | d929ce62, 090339f8, bf089c57 |
| `xtask/src/harness/generate_interface_spec.rs` | Fall H | nein | 14530cb3 |
| `xtask/src/harness/jules.rs` | Fall D | ja | 5b1b9279, ab8ccaa1, 525ecd80, b8660984, ce650578 |
| `xtask/src/harness/orphan_symbols.rs` | Fall H | nein | ce650578, 4e3392b8 |
| `xtask/src/harness/registry_check.rs` | Fall D | ja | 5b1b9279, 396b795f, ce650578 |
| `xtask/src/harness/repo_doctor.rs` | Fall H | nein | ce650578, 5bbe1e71 |
| `xtask/src/harness/scan_common.rs` | Fall H | nein | 525ecd80, b8660984, f142c6d8 |
| `xtask/src/harness/stub_impls.rs` | Fall H | nein | ce650578, f07adb3e |
| `xtask/src/harness/unsafe_audit.rs` | Fall B | nein | 240f4f7c, 7e1d65ba, 5f5251e9, 28bca5a3 |
| `xtask/src/harness/unwired_ports.rs` | Fall H | nein | 495b89dc, c1cf385c, ce650578, f5cfd215 |
| `xtask/src/harness/verdict.rs` | Fall B | nein | 40543e50 |
| `xtask/src/harness/verify.rs` | Fall H | nein | 5b1b9279 |
| `xtask/src/lib.rs` | Fall D | ja | c859626d, b1998340, 230372f0, 5b1b9279, 47d5b308, 240f4f7c, 7e1d65ba, 7124c9c6, f142c6d8, cd740f72 |
| `xtask/src/main.rs` | Fall D | ja | 5b1b9279, 47d5b308, 525ecd80, b8660984, 240f4f7c, 7e1d65ba, 7124c9c6 |
| `xtask/src/record_mutation_score.rs` | Fall B | nein | 5b1b9279 |
| `xtask/src/sync_docs.rs` | Fall H | nein | c859626d |
| `xtask/src/tag_health.rs` | Fall B | nein | 230372f0 |
| `xtask/src/verify/evidence.rs` | Fall H | nein | 5b1b9279 |
| `xtask/src/verify/mod.rs` | Fall H | nein | 5b1b9279 |
| `xtask/src/verify/report.rs` | Fall H | nein | 5b1b9279 |
| `xtask/src/verify/steps.rs` | Fall H | nein | 5b1b9279 |
| `xtask/src/wiring_check/mod.rs` | Fall H | nein | 2a1af3f7, 240f4f7c, 7e1d65ba, 7b197676 |
| `xtask/src/workspace_index.rs` | Fall H | nein | 495b89dc, c1cf385c, ce650578, f142c6d8 |
| `xtask/tests/cli_dispatch_test.rs` | Fall D | ja | 5b1b9279, 47d5b308 |
| `xtask/tests/generate_core_mapping_test.rs` | Fall H | nein | bf089c57 |
| `xtask/tests/harness_dependency_graph_audit_test.rs` | Fall H | nein | ce650578, ace01f7c |
| `xtask/tests/harness_doctrine_scan_test.rs` | Fall H | nein | 06910aa7 |
| `xtask/tests/harness_fast_diff_lint_test.rs` | Fall H | nein | ce650578, dfcce0de |
| `xtask/tests/harness_gate_guard_test.rs` | Fall D | ja | ce650578 |
| `xtask/tests/harness_gate_integrity_test.rs` | Fall D | ja | ce650578 |
| `xtask/tests/harness_gc_floor_single_source_test.rs` | Fall H | nein | 240f4f7c, 7e1d65ba, 5f5251e9, 28bca5a3 |
| `xtask/tests/harness_jules_parallel_test.rs` | Fall D | ja | ce650578 |
| `xtask/tests/harness_loop_guard_wiring_test.rs` | Fall B | nein | c26a4539 |
| `xtask/tests/harness_orphan_symbols_test.rs` | Fall H | nein | ce650578, 4e3392b8 |
| `xtask/tests/harness_repo_doctor_test.rs` | Fall H | nein | 5bbe1e71 |
| `xtask/tests/harness_stub_impls_test.rs` | Fall H | nein | ce650578, f07adb3e |
| `xtask/tests/harness_unsafe_audit_test.rs` | Fall B | nein | 28bca5a3 |
| `xtask/tests/harness_unwired_ports_test.rs` | Fall H | nein | f5cfd215 |
| `xtask/tests/harness_verdict_test.rs` | Fall B | nein | 40543e50 |
| `xtask/tests/orphan_symbols.rs` | Fall H | nein | ce650578 |
| `xtask/tests/scan_common.rs` | Fall H | nein | 495b89dc, c1cf385c, ce650578, f142c6d8 |
| `xtask/tests/verify_sabotage_test.rs` | Fall H | nein | 4b7709e6, 5b1b9279 |

---

## Entscheidung nötig

Die folgenden Commits beinhalten Änderungen an den Spezifikationsdokumenten unter `docs/spec/**`. Gemäß den Vorgaben müssen Spezifikationsänderungen gesondert unter 'Entscheidung nötig' gelistet werden und dürfen nicht direkt als Übernahmekandidat eingestuft werden:

- **Commit `1a028782`** (2026-10-15): `[SPEC-v4.1] Entfernen statt Einfrieren`
  - *Dateien:* `docs/spec/GESAMTSPEZIFIKATION-v4.md`
  - *Frage an Maintainer:* Sollen die Anpassungen an der Spezifikation in `main` nachgezogen werden oder bleibt die bestehende Spezifikation in `main` unverändert?

- **Commit `331c38c5`** (2026-10-15): `feat(kv): default TombstoneOnly and remove v3 deletion proof creation`
  - *Dateien:* `docs/spec/CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md`, `docs/verification/INV_MATRIX.md`
  - *Frage an Maintainer:* Sollen die Anpassungen an der Spezifikation in `main` nachgezogen werden oder bleibt die bestehende Spezifikation in `main` unverändert?

- **Commit `0c315cbf`** (2026-10-14): `Shell-Commit`
  - *Dateien:* `docs/ARCHITECTURE_PLAN_v1.md`, `docs/ARCHITEKTUR_MASTERPLAN_v2.md`, `docs/analysis/AUDIT_GESAMTSYNTHESE_2026-10-14.md`, `docs/spec/LOESCHNACHWEIS_SYSTEMSPEZIFIKATION_v2.md`
  - *Frage an Maintainer:* Sollen die Anpassungen an der Spezifikation in `main` nachgezogen werden oder bleibt die bestehende Spezifikation in `main` unverändert?

- **Commit `899d09ac`** (2026-10-14): `docs: add micro-fine-grained algorithmic specification for SOTA 2026 features`
  - *Dateien:* `docs/spec/SOTA_ALGORITHMIC_FEINSPEZIFIKATION_2026.md`
  - *Frage an Maintainer:* Sollen die Anpassungen an der Spezifikation in `main` nachgezogen werden oder bleibt die bestehende Spezifikation in `main` unverändert?

- **Commit `f7be670e`** (2026-10-14): `docs: add Google-Jules autonomous agent & VM operational specification`
  - *Dateien:* `docs/JULES_AUTONOMOUS_AGENT_SPECIFICATION.md`, `docs/SOURCE_OF_TRUTH.md`, `docs/spec/22-jules-autonomous-agent-specification.md`, `docs/spec/INDEX.md`
  - *Frage an Maintainer:* Sollen die Anpassungen an der Spezifikation in `main` nachgezogen werden oder bleibt die bestehende Spezifikation in `main` unverändert?

- **Commit `8fb718bb`** (2026-10-14): `docs: Spiegelung der Löschnachweis-Spezifikation und Verifier-Spec (B21)`
  - *Dateien:* `.jules/tasks/T-2026-LB21.toml`, `docs/spec/CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md`, `docs/verification/VERIFIER_SPEC.md`
  - *Frage an Maintainer:* Sollen die Anpassungen an der Spezifikation in `main` nachgezogen werden oder bleibt die bestehende Spezifikation in `main` unverändert?

- **Commit `561dcec9`** (2026-10-13): `Shell-Commit`
  - *Dateien:* `.jules/tasks/T-2026-9213.toml`, `.jules/tasks/T-2026-9220.toml`, `docs/spec/CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md`, `docs/spec/CONTEXTRA_SPEZIFIKATION_v15.md`, `docs/spec/SOURCE_OF_TRUTH_DIFF.md`
  - *Frage an Maintainer:* Sollen die Anpassungen an der Spezifikation in `main` nachgezogen werden oder bleibt die bestehende Spezifikation in `main` unverändert?

- **Commit `75c687b6`** (2026-10-12): `docs(spec): create SOURCE_OF_TRUTH_DIFF and fix orphaned path references`
  - *Dateien:* `docs/spec/CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md`, `docs/spec/CONTEXTRA_SPEZIFIKATION_v15.md`, `docs/spec/SOURCE_OF_TRUTH_DIFF.md`
  - *Frage an Maintainer:* Sollen die Anpassungen an der Spezifikation in `main` nachgezogen werden oder bleibt die bestehende Spezifikation in `main` unverändert?

- **Commit `449784f4`** (2026-10-12): `docs(spec): establish source of truth diff and fix orphaned spec path references`
  - *Dateien:* `docs/spec/CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md`, `docs/spec/CONTEXTRA_SPEZIFIKATION_v15.md`, `docs/spec/SOURCE_OF_TRUTH_DIFF.md`
  - *Frage an Maintainer:* Sollen die Anpassungen an der Spezifikation in `main` nachgezogen werden oder bleibt die bestehende Spezifikation in `main` unverändert?

---

## Abhängigkeiten und Reihenfolge

1. **Xtask Core Infrastructure (`xtask/src/shared/`, `scan_common`)**: Bildet das Fundament für alle Subkommandos (`unwired-ports`, `repo-doctor`, `orphan-symbols`).
2. **CI-Workflows (`.github/workflows/`)**: Benötigen die entsprechenden `xtask`-Subkommandos und Gate-Definitionen vor der Aktivierung.
3. **Jules Harness & Task Cards (`.jules/`)**: Können unabhängig verarbeitet werden.

---

## Auffälligkeiten

- **Bot-Commits (`chore(unwrap-baseline)`)**: 17 Commits aktualisieren ausschließlich den Unwrap-Ratchet-Zähler. Diese sind als `SKIP-BOT` klassifiziert.
- **Spezifikationsanpassungen (`docs/spec/`)**: 9 Commits betreffen Spezifikationen und wurden als `DECISION` markiert.

---

## Löschungen respektiert

- Dateien, die in `main` durch Backup/Reset-Entscheidung (`8412f086`) gelöscht wurden, bleiben gelöscht.
