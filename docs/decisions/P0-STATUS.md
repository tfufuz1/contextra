# Store-Refactoring: Status (Stand 2026-11-18)

Toolchain: rustc/cargo 1.89.0 (conda-forge), Features `workspace,test-hooks,fault-injection,legacy-wal-key`.

## Baseline (unveraenderter main-Code, gemessen)
| Befund | Ergebnis |
|---|---|
| Test-Targets, die nicht kompilieren | 28 von 150 (Typfehler E0308, Trait-Drift `merge`, fehlendes `crate::support`, `src/wal/io.rs` ist Verzeichnis) |
| Haengender Test | `compaction::tests::advanced::test_compaction_stress_and_gc` (>5 min) |
| Rote Targets (stabil, auf main reproduziert) | crash_recovery 1, flush_durability 1, flush_recovery_lifecycle 2, format_version_compat 1, golden_corpus 2 |
| Lib-Test (scanner) | `test_wal_purge_with_retained_records_preserves_hmac_chain_and_replays_cleanly` rot, auch mit Original-Purge |
| Clippy `-D warnings` | rot: 65 Fehler (`indexing_slicing`), ~370 Warnungen, vorbestehend |
| `--features docid-128` | kompiliert nicht (`tenant_codec.rs:109`, `v2.rs:140`), vorbestehend |
| Tests veraendern Repo | `tests/fixtures/current_v1.manifest` wird beim Testlauf ueberschrieben, `*.proptest-regressions` entsteht |

## Befund-Korrekturen
- B2: Key-Codec v2 existiert (`tenant_codec/v2.rs`, `KeyCodecV2`), ist aber nirgends verdrahtet (nur Re-Export).
- B3/P3: `ManifestEntry::WalFloor`, `Manifest::extract_wal_floor` und `manifest::Version` existieren; `Version` wird im `lsm/`-Code nicht benutzt.
- B1: `contextra-durable-fs` ist Abhaengigkeit; `contextra-testkit` ist keine Dev-Dependency dieses Crates.
- B8: `shred_file` existiert nirgends; `scrub_file` nur in Docs (Code: `scrub_if_safe`).

## Geliefert
- P0b Purge-Hotfix (`purge.rs`, `lsm/engine.rs`): keine verschluckten Fehler, Key-Manager statt `None`, Nullen mit fsync + Dir-Fsync. Scanner-Tests: 8 gruen, 1 vorbestehend rot, keine Regression.
- P2 Teil: `KeyTag` Eq/Ord/Hash nach kodiertem Byte (vorher `Custom(1) != Chunk`, Ord inkonsistent zur Byte-Ordnung); `tests/key_codec_v2_properties.rs` (7 Property-Tests: Roundtrip, Ordnung, Praefix, Cluster-Kontiguitaet, v1/v2-Disjunktheit, Robustheit) gruen.
- Docs: `scrub_if_safe`, D-76-Entwurf.

## Offen
P1 (E2), P3-P7, lazy v1->v2-Migration, Verdrahtung v2, Reparatur der 28 Test-Targets, vorbestehende roten Tests.
