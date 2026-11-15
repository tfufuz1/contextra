# Extraktion fuer `acta-rag`

**Aufgabe:** Abrufpipeline: Hybridsuche, RRF, Reranking, Kontextaufbau, Belegpruefung, Eval  
**Level:** L4 · **Budget:** 7000 LOC · **Spezifikation:** docs/SPEC.md Kap. 9.4-9.7  
**Interne Abhaengigkeiten (Soll):** acta-base, acta-crypto, acta-ledger, acta-text, acta-vec, acta-history, acta-pstore, acta-gate, acta-access, acta-infer  
**Umfang des Archivs:** 54 Dateien, 9604 Zeilen (inkl. Tests)  

## Herkunft und Umbauauftrag

| Altquelle (Crate) | Umbauauftrag |
|---|---|
| `rank` | RRF (Ganzzahl, 9.4); Recall@k/MRR/nDCG fuer eval (9.7); drift -> QualityDrift |
| `prompt-guard` | Prompt-Injection-Pruefung (T7, 9.4); Beleg-Token/Markup-Entfernung |

## Pruefhinweise (automatisch ermittelt)

| Befund | Treffer | Folge |
|---|---:|---|
| float | 165 | f32/f64 (Z7: Festkomma in Ranking-Pfaden) |
| serde | 26 | serde/bincode (G-NOSERDE-DISK: Plattenformate von Hand kodieren) |
| zeit_zufall | 4 | Zeit/Zufall direkt (G3: nur ueber Ports Clock/CryptoRng) |
| panic | 56 | unwrap/expect/panic (G-LINT) |
| std_fs | 7 | std::fs direkt (G-FS: nur acta-fs) |

**Externe Abhaengigkeiten der Altquelle ausserhalb der Whitelist (3.8):** `ahash`, `tracing` — ersetzen oder per ADR aufnehmen.

## Dateien

| Datei (relativ zu `archive/acta-rag/`) | LOC | Befunde |
|---|---:|---|
| `prompt-guard/fuzz/fuzz_targets/fuzz_prompt_injection_guard.rs` | 66 |  |
| `prompt-guard/src/audit.rs` | 82 | serde=1, std_fs=1 |
| `prompt-guard/src/clock.rs` | 31 |  |
| `prompt-guard/src/contract.rs` | 13 |  |
| `prompt-guard/src/error.rs` | 82 |  |
| `prompt-guard/src/guard.rs` | 661 | std_fs=2 |
| `prompt-guard/src/lib.rs` | 25 |  |
| `prompt-guard/src/policy.rs` | 71 | serde=6 |
| `prompt-guard/src/tests.rs` | 402 | panic=10, std_fs=1 |
| `prompt-guard/tests/golden_characterization.rs` | 78 | panic=3, serde=1 |
| `prompt-guard/tests/golden_mutation_tests.rs` | 60 | panic=4 |
| `prompt-guard/tests/kats_unicode_evasion.rs` | 135 |  |
| `prompt-guard/tests/property_determinism.rs` | 70 | panic=3 |
| `rank/src/calibration/conformal.rs` | 228 | float=28, serde=2 |
| `rank/src/calibration/isotonic.rs` | 424 | float=43, serde=2 |
| `rank/src/calibration/mod.rs` | 125 | float=2, serde=4 |
| `rank/src/calibration/platt.rs` | 166 | float=20, serde=2 |
| `rank/src/contract.rs` | 15 |  |
| `rank/src/drift.rs` | 137 | float=11, serde=2 |
| `rank/src/error.rs` | 23 |  |
| `rank/src/fusion/ext.rs` | 37 | serde=2 |
| `rank/src/fusion/global.rs` | 233 | float=7, serde=1 |
| `rank/src/fusion/mod.rs` | 44 |  |
| `rank/src/fusion/mrrf.rs` | 224 | float=15 |
| `rank/src/fusion/normalized.rs` | 196 | float=13 |
| `rank/src/fusion/provenance.rs` | 306 | float=27, alt_spec=1 |
| `rank/src/fusion/resonance.rs` | 96 | float=5, serde=1 |
| `rank/src/fusion/rrf.rs` | 753 | float=35 |
| `rank/src/fusion/signal.rs` | 200 | float=4, serde=2 |
| `rank/src/fusion/topk.rs` | 99 | float=3 |
| `rank/src/fusion/types.rs` | 216 | float=14, serde=13 |
| `rank/src/lib.rs` | 103 |  |
| `rank/src/metrics.rs` | 321 | float=21 |
| `rank/src/temporal.rs` | 385 |  |
| `rank/tests/calibration_deep_tests.rs` | 289 | panic=6, float=10 |
| `rank/tests/calibration_drift_property_tests.rs` | 141 | float=4 |
| `rank/tests/conformal_tests.rs` | 219 | panic=9, float=13 |
| `rank/tests/fusion_deep_property_tests.rs` | 273 | float=1 |
| `rank/tests/fusion_tie_break_proof.rs` | 144 | panic=2, float=1 |
| `rank/tests/global_fusion_community_aware.rs` | 136 | panic=4, float=1 |
| `rank/tests/golden/drift_status.json` | 0 |  |
| `rank/tests/golden/id_serde.json` | 0 |  |
| `rank/tests/golden/isotonic_calibration.json` | 0 |  |
| `rank/tests/golden/rrf_fusion.json` | 0 |  |
| `rank/tests/golden_tests.rs` | 191 | panic=15, float=3, serde=1, std_fs=1 |
| `rank/tests/hardening_tests.rs` | 76 | panic=4, std_fs=1 |
| `rank/tests/isotonic_mutation_hardening_test.rs` | 190 | panic=9, float=1 |
| `rank/tests/nan_inf_safety_tests.rs` | 164 | float=30 |
| `rank/tests/rank_invariants.rs` | 370 | panic=18, float=24, alt_spec=2 |
| `rank/tests/rank_tests.rs` | 338 | panic=1, float=6 |
| `rank/tests/rrf_golden_tests.rs` | 311 | float=1 |
| `rank/tests/rrf_property_tests.rs` | 258 | float=9 |
| `rank/tests/rrf_reference.rs` | 215 | panic=1, float=8 |
| `rank/tests/score_normalized_fallback.rs` | 182 | float=3 |
