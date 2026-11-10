# Ausstehende API-Änderungen (P13)

Format: Symbol | betroffene Crates | gewünschte Änderung

| Symbol | betroffene Crates | gewünschte Änderung |
|---|---|---|
| `contextra_db::Contextra` | `contextra`, `contextra-agent`, `benchmarks`, `examples` | Nach WP-100 / P42: Nutzung auf `contextra::Contextra` umstellen. |
| `contextra_db::ContextraConfig` | `contextra`, `contextra-agent`, `benchmarks`, `examples` | Nach WP-100 / P42: Nutzung auf `contextra::ContextraConfig` umstellen. |
| `contextra_db::Collection` | `contextra`, `contextra-agent` | Nach WP-100 / P42: Nutzung auf `contextra::Collection` umstellen. |
| `contextra_db::volatile_vault::*` | `contextra-agent` | Nach WP-100 / P42: Verschieben nach `contextra::volatile_vault::*`. |
| `contextra_db::QueryRewriter` | `contextra::query_rewriter` | Nach WP-100 / P42: Verschieben nach `contextra::multistep::QueryRewriter`. |
| `contextra_db::fuzz::rrf_fusion` | `contextra-rank` | Verschieben des Fuzzing-Targets nach `contextra-rank/fuzz/` (reine Rank-Fusion). |
