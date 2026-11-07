# Test-Matrix & Pyramiden-Audit (AGENTS.md §8)

> Generiert durch `tools/test_matrix_advisor.py`.
> Untersucht Test-Stufen: Unit -> Proptest -> Concurrency (Loom) -> Chaos (FaultVfs) -> Benchmarks.

## Crate Testpyramiden-Übersicht

| Crate | Unit-Tests | Proptests | Loom Checks | FaultVfs | Benchmarks | Empfehlungen |
|---|---|---|---|---|---|---|
| `contextra` | 65 | 0 | 0 | 0 | 0 | Property-Tests mit proptest ergänzen; Criterion-Benchmarks für Hot Paths anlegen |
| `contextra-adapt` | 141 | 1 | 0 | 0 | 0 | Criterion-Benchmarks für Hot Paths anlegen |
| `contextra-agent` | 121 | 0 | 0 | 0 | 0 | Property-Tests mit proptest ergänzen; Criterion-Benchmarks für Hot Paths anlegen |
| `contextra-audit-export` | 32 | 0 | 0 | 0 | 0 | Property-Tests mit proptest ergänzen; Criterion-Benchmarks für Hot Paths anlegen |
| `contextra-avv-generator` | 25 | 0 | 0 | 0 | 0 | Property-Tests mit proptest ergänzen; Criterion-Benchmarks für Hot Paths anlegen |
| `contextra-checkpoint` | 143 | 1 | 1 | 3 | 6 | Vollständig aufgestellt |
| `contextra-cognition` | 110 | 0 | 0 | 0 | 0 | Property-Tests mit proptest ergänzen; Criterion-Benchmarks für Hot Paths anlegen |
| `contextra-core` | 7 | 0 | 0 | 0 | 0 | Criterion-Benchmarks für Hot Paths anlegen |
| `contextra-crypto` | 321 | 7 | 17 | 6 | 10 | Vollständig aufgestellt |
| `contextra-db` | 186 | 3 | 0 | 0 | 1 | Vollständig aufgestellt |
| `contextra-durable-fs` | 37 | 0 | 0 | 0 | 0 | Property-Tests mit proptest ergänzen; Kritischer Concurrency-Crate: Loom-Tests erforderlich; Criterion-Benchmarks für Hot Paths anlegen |
| `contextra-engine` | 260 | 3 | 7 | 0 | 0 | Criterion-Benchmarks für Hot Paths anlegen |
| `contextra-erasure` | 94 | 0 | 5 | 0 | 0 | Vollständig aufgestellt |
| `contextra-graph` | 399 | 6 | 0 | 3 | 10 | Vollständig aufgestellt |
| `contextra-infer-candle` | 119 | 2 | 0 | 0 | 10 | Vollständig aufgestellt |
| `contextra-infer-ollama` | 117 | 2 | 0 | 0 | 0 | Criterion-Benchmarks für Hot Paths anlegen |
| `contextra-infer-onnx` | 41 | 1 | 0 | 0 | 5 | Vollständig aufgestellt |
| `contextra-ingest` | 204 | 9 | 0 | 0 | 0 | Criterion-Benchmarks für Hot Paths anlegen |
| `contextra-kvcache` | 101 | 2 | 0 | 0 | 0 | Criterion-Benchmarks für Hot Paths anlegen |
| `contextra-license` | 67 | 0 | 0 | 0 | 0 | Property-Tests mit proptest ergänzen; Criterion-Benchmarks für Hot Paths anlegen |
| `contextra-mcp` | 145 | 0 | 0 | 0 | 0 | Property-Tests mit proptest ergänzen; Criterion-Benchmarks für Hot Paths anlegen |
| `contextra-mvcc` | 110 | 5 | 9 | 0 | 5 | Vollständig aufgestellt |
| `contextra-ports` | 52 | 1 | 0 | 0 | 0 | Criterion-Benchmarks für Hot Paths anlegen |
| `contextra-privacy` | 147 | 0 | 0 | 0 | 0 | Property-Tests mit proptest ergänzen; Criterion-Benchmarks für Hot Paths anlegen |
| `contextra-py` | 91 | 0 | 0 | 0 | 0 | Property-Tests mit proptest ergänzen; Criterion-Benchmarks für Hot Paths anlegen |
| `contextra-rank` | 94 | 6 | 0 | 0 | 0 | Criterion-Benchmarks für Hot Paths anlegen |
| `contextra-router` | 142 | 3 | 0 | 0 | 15 | Vollständig aufgestellt |
| `contextra-sandbox` | 74 | 0 | 0 | 0 | 0 | Property-Tests mit proptest ergänzen; Criterion-Benchmarks für Hot Paths anlegen |
| `contextra-simd` | 80 | 1 | 0 | 0 | 8 | Vollständig aufgestellt |
| `contextra-store` | 567 | 17 | 23 | 7 | 31 | Vollständig aufgestellt |
| `contextra-sys` | 31 | 0 | 0 | 0 | 0 | Property-Tests mit proptest ergänzen; Criterion-Benchmarks für Hot Paths anlegen |
| `contextra-testkit` | 62 | 0 | 0 | 22 | 0 | Property-Tests mit proptest ergänzen; Criterion-Benchmarks für Hot Paths anlegen |
| `contextra-text` | 206 | 5 | 0 | 0 | 1 | Vollständig aufgestellt |
| `contextra-types` | 186 | 6 | 0 | 0 | 0 | Criterion-Benchmarks für Hot Paths anlegen |
| `contextra-vector` | 264 | 3 | 4 | 0 | 45 | Chaos-Tests mit FaultVfs zur Crash-Validierung anlegen |
| `contextra-wire` | 108 | 0 | 0 | 0 | 0 | Property-Tests mit proptest ergänzen; Criterion-Benchmarks für Hot Paths anlegen |

## Handlungsempfehlungen für Coding-Agenten

### `contextra`

- [ ] Property-Tests mit proptest ergänzen
- [ ] Criterion-Benchmarks für Hot Paths anlegen
### `contextra-adapt`

- [ ] Criterion-Benchmarks für Hot Paths anlegen
### `contextra-agent`

- [ ] Property-Tests mit proptest ergänzen
- [ ] Criterion-Benchmarks für Hot Paths anlegen
### `contextra-audit-export`

- [ ] Property-Tests mit proptest ergänzen
- [ ] Criterion-Benchmarks für Hot Paths anlegen
### `contextra-avv-generator`

- [ ] Property-Tests mit proptest ergänzen
- [ ] Criterion-Benchmarks für Hot Paths anlegen
### `contextra-cognition`

- [ ] Property-Tests mit proptest ergänzen
- [ ] Criterion-Benchmarks für Hot Paths anlegen
### `contextra-core`

- [ ] Criterion-Benchmarks für Hot Paths anlegen
### `contextra-durable-fs`

- [ ] Property-Tests mit proptest ergänzen
- [ ] Kritischer Concurrency-Crate: Loom-Tests erforderlich
- [ ] Criterion-Benchmarks für Hot Paths anlegen
### `contextra-engine`

- [ ] Criterion-Benchmarks für Hot Paths anlegen
### `contextra-infer-ollama`

- [ ] Criterion-Benchmarks für Hot Paths anlegen
### `contextra-ingest`

- [ ] Criterion-Benchmarks für Hot Paths anlegen
### `contextra-kvcache`

- [ ] Criterion-Benchmarks für Hot Paths anlegen
### `contextra-license`

- [ ] Property-Tests mit proptest ergänzen
- [ ] Criterion-Benchmarks für Hot Paths anlegen
### `contextra-mcp`

- [ ] Property-Tests mit proptest ergänzen
- [ ] Criterion-Benchmarks für Hot Paths anlegen
### `contextra-ports`

- [ ] Criterion-Benchmarks für Hot Paths anlegen
### `contextra-privacy`

- [ ] Property-Tests mit proptest ergänzen
- [ ] Criterion-Benchmarks für Hot Paths anlegen
### `contextra-py`

- [ ] Property-Tests mit proptest ergänzen
- [ ] Criterion-Benchmarks für Hot Paths anlegen
### `contextra-rank`

- [ ] Criterion-Benchmarks für Hot Paths anlegen
### `contextra-sandbox`

- [ ] Property-Tests mit proptest ergänzen
- [ ] Criterion-Benchmarks für Hot Paths anlegen
### `contextra-sys`

- [ ] Property-Tests mit proptest ergänzen
- [ ] Criterion-Benchmarks für Hot Paths anlegen
### `contextra-testkit`

- [ ] Property-Tests mit proptest ergänzen
- [ ] Criterion-Benchmarks für Hot Paths anlegen
### `contextra-types`

- [ ] Criterion-Benchmarks für Hot Paths anlegen
### `contextra-vector`

- [ ] Chaos-Tests mit FaultVfs zur Crash-Validierung anlegen
### `contextra-wire`

- [ ] Property-Tests mit proptest ergänzen
- [ ] Criterion-Benchmarks für Hot Paths anlegen
