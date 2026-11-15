# Extraktion fuer `acta-rag`

**Aufgabe:** Abrufpipeline: Hybridsuche, RRF, Reranking, Kontextaufbau, Belegpruefung, Eval  
**Level:** L4 · **Budget:** 7000 LOC · **Spezifikation:** docs/SPEC.md Kap. 9.4-9.7  
**Interne Abhaengigkeiten (Soll):** acta-base, acta-crypto, acta-ledger, acta-text, acta-vec, acta-history, acta-pstore, acta-gate, acta-access, acta-infer  
**Umfang des Archivs:** 35 Dateien, 5837 Zeilen (inkl. Tests)  

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
| `prompt-guard/fuzz/fuzz_targets/fuzz_prompt_injection_guard.rs` | 67 |  |
| `prompt-guard/src/audit.rs` | 83 | serde=1, std_fs=3 |
| `prompt-guard/src/clock.rs` | 32 | zeit_zufall=1 |
| `prompt-guard/src/contract.rs` | 14 |  |
| `prompt-guard/src/error.rs` | 83 |  |
| `prompt-guard/src/guard.rs` | 662 | std_fs=2 |
| `prompt-guard/src/lib.rs` | 26 |  |
| `prompt-guard/src/policy.rs` | 72 | serde=6 |
| `prompt-guard/src/tests.rs` | 403 | zeit_zufall=1, panic=10, std_fs=1 |
| `prompt-guard/tests/golden_characterization.rs` | 79 | serde=1, panic=3 |
| `prompt-guard/tests/golden_mutation_tests.rs` | 61 | panic=4 |
| `prompt-guard/tests/kats_unicode_evasion.rs` | 136 | zeit_zufall=2 |
| `prompt-guard/tests/property_determinism.rs` | 71 | panic=3 |
| `rank/src/contract.rs` | 16 |  |
| `rank/src/drift.rs` | 138 | float=11, serde=2 |
| `rank/src/error.rs` | 24 |  |
| `rank/src/fusion/mod.rs` | 45 |  |
| `rank/src/fusion/rrf.rs` | 754 | float=35 |
| `rank/src/fusion/signal.rs` | 201 | float=4, serde=2 |
| `rank/src/fusion/topk.rs` | 100 | float=3 |
| `rank/src/fusion/types.rs` | 217 | float=14, serde=13 |
| `rank/src/lib.rs` | 104 |  |
| `rank/src/metrics.rs` | 322 | float=21 |
| `rank/tests/fusion_deep_property_tests.rs` | 274 | float=1 |
| `rank/tests/fusion_tie_break_proof.rs` | 145 | float=1, panic=2 |
| `rank/tests/golden/drift_status.json` | 1 |  |
| `rank/tests/golden/id_serde.json` | 52 |  |
| `rank/tests/golden/isotonic_calibration.json` | 30 |  |
| `rank/tests/golden/rrf_fusion.json` | 110 |  |
| `rank/tests/golden_tests.rs` | 192 | float=3, serde=1, panic=15, std_fs=1 |
| `rank/tests/nan_inf_safety_tests.rs` | 165 | float=30 |
| `rank/tests/rank_invariants.rs` | 371 | float=24, panic=18 |
| `rank/tests/rrf_golden_tests.rs` | 312 | float=1 |
| `rank/tests/rrf_property_tests.rs` | 259 | float=9 |
| `rank/tests/rrf_reference.rs` | 216 | float=8, panic=1 |
