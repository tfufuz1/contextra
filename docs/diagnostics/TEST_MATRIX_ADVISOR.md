# Test-Matrix & Pyramiden-Audit (AGENTS.md §8)

> Generiert durch `tools/test_matrix_advisor.py`.
> Untersucht Test-Stufen: Unit -> Proptest -> Concurrency (Loom) -> Chaos (FaultVfs) -> Benchmarks.

## Crate Testpyramiden-Übersicht

| Crate | Unit-Tests | Proptests | Loom Checks | FaultVfs | Benchmarks | Empfehlungen |
|---|---|---|---|---|---|---|
| `contextra` | 51 | 0 | 0 | 0 | 0 | Property-Tests mit proptest ergänzen; Criterion-Benchmarks für Hot Paths anlegen |
| `contextra-checkpoint` | 162 | 2 | 1 | 0 | 6 | Vollständig aufgestellt |
| `contextra-crypto` | 242 | 5 | 17 | 0 | 10 | Vollständig aufgestellt |
| `contextra-db` | 318 | 4 | 7 | 0 | 1 | Vollständig aufgestellt |
| `contextra-durable-fs` | 43 | 0 | 0 | 0 | 5 | Kritischer Concurrency-Crate: Loom-Tests erforderlich |
| `contextra-engine` | 110 | 4 | 0 | 0 | 0 | Criterion-Benchmarks für Hot Paths anlegen |
| `contextra-env` | 20 | 0 | 0 | 0 | 0 | Property-Tests mit proptest ergänzen; Criterion-Benchmarks für Hot Paths anlegen |
| `contextra-erasure` | 229 | 2 | 15 | 0 | 0 | Vollständig aufgestellt |
| `contextra-graph` | 420 | 7 | 0 | 6 | 10 | Vollständig aufgestellt |
| `contextra-ids` | 17 | 0 | 0 | 0 | 0 | Property-Tests mit proptest ergänzen; Criterion-Benchmarks für Hot Paths anlegen |
| `contextra-mcp` | 111 | 0 | 0 | 0 | 0 | Property-Tests mit proptest ergänzen; Criterion-Benchmarks für Hot Paths anlegen |
| `contextra-mvcc` | 138 | 5 | 21 | 0 | 5 | Vollständig aufgestellt |
| `contextra-ports` | 40 | 1 | 0 | 0 | 0 | Criterion-Benchmarks für Hot Paths anlegen |
| `contextra-privacy` | 166 | 0 | 0 | 0 | 5 | Property-Tests mit proptest ergänzen |
| `contextra-rank` | 136 | 8 | 0 | 0 | 5 | Vollständig aufgestellt |
| `contextra-simd` | 99 | 3 | 0 | 0 | 5 | Vollständig aufgestellt |
| `contextra-store` | 636 | 19 | 29 | 7 | 31 | Vollständig aufgestellt |
| `contextra-sys` | 47 | 0 | 0 | 0 | 5 | Property-Tests mit proptest ergänzen |
| `contextra-testkit` | 89 | 0 | 0 | 26 | 5 | Property-Tests mit proptest ergänzen |
| `contextra-text` | 238 | 7 | 0 | 0 | 6 | Vollständig aufgestellt |
| `contextra-types` | 188 | 6 | 0 | 0 | 5 | Vollständig aufgestellt |
| `contextra-vault` | 16 | 0 | 0 | 0 | 0 | Property-Tests mit proptest ergänzen; Criterion-Benchmarks für Hot Paths anlegen |
| `contextra-vector` | 370 | 8 | 10 | 0 | 51 | Chaos-Tests mit FaultVfs zur Crash-Validierung anlegen |

## Handlungsempfehlungen für Coding-Agenten

### `contextra`

- [ ] Property-Tests mit proptest ergänzen
- [ ] Criterion-Benchmarks für Hot Paths anlegen
### `contextra-durable-fs`

- [ ] Kritischer Concurrency-Crate: Loom-Tests erforderlich
### `contextra-engine`

- [ ] Criterion-Benchmarks für Hot Paths anlegen
### `contextra-env`

- [ ] Property-Tests mit proptest ergänzen
- [ ] Criterion-Benchmarks für Hot Paths anlegen
### `contextra-ids`

- [ ] Property-Tests mit proptest ergänzen
- [ ] Criterion-Benchmarks für Hot Paths anlegen
### `contextra-mcp`

- [ ] Property-Tests mit proptest ergänzen
- [ ] Criterion-Benchmarks für Hot Paths anlegen
### `contextra-ports`

- [ ] Criterion-Benchmarks für Hot Paths anlegen
### `contextra-privacy`

- [ ] Property-Tests mit proptest ergänzen
### `contextra-sys`

- [ ] Property-Tests mit proptest ergänzen
### `contextra-testkit`

- [ ] Property-Tests mit proptest ergänzen
### `contextra-vault`

- [ ] Property-Tests mit proptest ergänzen
- [ ] Criterion-Benchmarks für Hot Paths anlegen
### `contextra-vector`

- [ ] Chaos-Tests mit FaultVfs zur Crash-Validierung anlegen
