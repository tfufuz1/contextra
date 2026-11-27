Zentrale Module: csr, ppr und ppr_stream, apprh, path_rag (KPathDiffusion), hyperedge, hyperedge_suggest, community, session_dag (SessionBranchTree), edge_reinforcement, entity_extraction, cascade, percolation, provenance, purger.

Harte Regeln:
- Lock-Hierarchie im SessionBranchTree: `nodes` muss vor `edges` oder `active_head` gesperrt werden. Abweichungen führen zu Deadlocks und sind verboten.
- Keine Sperren über `.await`-Punkte.
- Determinismus: `BTreeMap` und `BTreeSet` statt `HashMap`, injizierte Clock und Rng.
- Experimentelle Pfade stehen hinter Features (`experimental-graph`, `ppr-forward-push`, `k-path-diffusion`, `apprh-diffusion`). Änderungen am Default-Verhalten brauchen Feature-Matrix-Tests.
- Das Alias-Feature `physio-percolation` bleibt bis zur Migration bestehen, wird aber nicht erweitert.

Verifikation: `cargo test -p contextra-graph --locked`, `cargo test -p contextra-graph --all-features`, Benchmarks in `benches/`. Für PPR-Änderungen Ergebnisse gegen eine Referenzimplementierung vergleichen.

Optimierungsfokus: Speicherarme CSR-Updates, Restpuffer für Kantenänderungen, Konvergenztoleranz von PPR, Vermeidung von Allokationen in Traversierungen.
