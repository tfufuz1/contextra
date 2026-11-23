# AGENTS.md — Contextra Bench

## Purpose
`contextra-bench` is the standalone, reproducible benchmark harness for measuring retrieval accuracy (Recall@K, MRR, error rate) across Contextra features:
1. Context-Prefix Retrieval (Baseline vs. LLM/Rule-based Context-Prefix)
2. Cross-Encoder Reranking (RRF vs. Post-RRF Reranked)

## Invariants
- All benchmarks must execute on reproducible, deterministic synthetic corpora with annotated ground truth.
- Results must be output in both JSON (`results.json`) and Markdown format.
- Code in this crate must adhere to `#![forbid(unsafe_code)]` and zero unwrap policies in non-test paths where possible.

## Storage-Umbau (`contextra-storage`)

Normativ: `docs/SPEC.md` (R8). Umsetzungsplan und Verträge: `docs/IMPLEMENTATION_SPEC.md` (nicht normativ). Bis Gate G-5 ändert sich dieses Crate nicht (Spec §22); die Angaben hier sind Dokumentation, kein Auftrag zur Codeänderung.

- Rolle: Konsument (Benchmark). `contextra-store` steht unter `[dependencies]` und wird nur in `benches/competitive_kv.rs` genutzt (`LsmConfig`, `LsmStorage`; 1 Zeile, Stand 23.11.2026).
- Vertrag IF-8: Umstellung zusammen mit dem Adapter der Konsumenten (ab G-5, WP-5.5). Danach greift der Benchmark nur über die öffentliche API von `contextra-storage` (Spec §8) zu.
- Verboten: neue Nutzung von `contextra-store`.
- Arbeitspakete: WP-5.5.
