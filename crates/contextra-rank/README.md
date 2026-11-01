# contextra-rank

Deterministische Reciprocal Rank Fusion (RRF).

- **Ring:** Ring 2 (Ranking & Fusion)
- **Normativ:** [docs/SPEC.md §8.13](../../docs/SPEC.md)
- **Arbeitsanweisungen:** [AGENTS.md](AGENTS.md)

## Rolle & Zweck
`contextra-rank` ist für die deterministische Fusion von Trefferlisten aus Volltext- und Vektorsuche mittels Reciprocal Rank Fusion (RRF) zuständig:
- `rrf_fuse(lists: &[Vec<Hit>], k: u32, top: usize) -> Vec<Hit>`
- Standardparameter $k = 60$.
- Deterministischer Tie-Break nach aufsteigendem `doc_tag`.

## Invarianten
- `#![forbid(unsafe_code)]`
- Permutationsinvarianz bezüglich der Reihenfolge der Eingabelisten.
- Keine Hintergrund-Threads, keine asynchronen Tasks.
- Komplexe historische Kalibratoren (`IsotonicCalibrator`, Platt Scaling) und Abhängigkeiten zu `adapt` sind vollständig entfernt.
