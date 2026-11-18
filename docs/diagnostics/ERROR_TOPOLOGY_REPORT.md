# Error Topology & Data Leak Audit (INV-LOG-1 / HR-4)

> Generiert durch `tools/error_topology_mapper.py`.
> Untersucht typisierte Fehlerstrukturen und verhindert Nutzdaten-/Geheimnis-Leaks in Fehlermeldungen.

## Crate Übersicht

| Crate | Definierte Fehler-Typen | Potenzielle Nutzdaten-Leaks |
|---|---|---|
| `contextra` | 3 | 0 |
| `contextra-checkpoint` | 1 | 0 |
| `contextra-crypto` | 1 | 0 |
| `contextra-db` | 0 | 0 |
| `contextra-durable-fs` | 1 | 0 |
| `contextra-engine` | 0 | 0 |
| `contextra-env` | 0 | 0 |
| `contextra-erasure` | 16 | 0 |
| `contextra-graph` | 5 | 0 |
| `contextra-ids` | 1 | 0 |
| `contextra-mcp` | 2 | 0 |
| `contextra-mvcc` | 1 | 0 |
| `contextra-ports` | 6 | 0 |
| `contextra-privacy` | 3 | **2** |
| `contextra-rank` | 2 | 0 |
| `contextra-simd` | 1 | 0 |
| `contextra-store` | 3 | 0 |
| `contextra-sys` | 1 | 0 |
| `contextra-testkit` | 0 | 0 |
| `contextra-text` | 2 | 0 |
| `contextra-types` | 2 | 0 |
| `contextra-vault` | 1 | 0 |
| `contextra-vector` | 3 | 0 |

## Verdächtige Fehlermeldungs-Formate (Möglicher Leak von {key}, {secret}...)

### `contextra-privacy`

- `/home/freddy/Projekte/contextra/crates/contextra-privacy/src/error.rs:51` [enthält `plaintext`]: `Surrogate token collision detected for token '{surrogate}': existing plaintext '{existing_plaintext}' != new plaintext '{new_plaintext}'`
- `/home/freddy/Projekte/contextra/crates/contextra-privacy/src/egress_vault.rs:279` [enthält `plaintext`]: `Surrogate token collision detected for token '{surrogate}': existing plaintext '{existing_plaintext}' != new plaintext '{new_plaintext}'`
