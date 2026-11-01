# Governance Rule: Dependency Hygiene & Ring Layering (`rules/dependencies.md`)

> **Normative Vorgabe nach docs/SPEC.md §5.2, §5.3 (Matrix) und T-ARCH-3.**

## Gilt für
- `Cargo.toml` (Workspace Root)
- Alle `Cargo.toml` in `crates/*/`
- `tests/arch`

## Pflichtregeln
1. **Striktes Ringmodell (Ring 4 → 0, AR-1)**:
   - Jede Abhängigkeit darf ausschließlich auf denselben oder einen niedrigeren Ring zeigen.
   - Innerhalb von Ring 2 gilt strikt die Abhängigkeitsmatrix aus SPEC §5.3:
     - `store` kennt weder `text`, `vector` noch `erasure`.
     - `erasure` kennt **keine** Ring-2-Crates (Kopplung ausschließlich über `contextra-ports`).
     - `store` darf `Vfs` nur als Trait aus `ports` nutzen, nie direkt `contextra-durable-fs`.
2. **Keine Netzwerk-Crates (AR-7, Single-Node-Doktrin)**:
   - Keine HTTP-, TCP-, Socket-, P2P- oder RPC-Crates im Abhängigkeitsbaum der 14 Produktionscrates (`cargo tree`).
3. **Ring 0 Asynchronitäts-Reinheit (AR-5, T-ARCH-1)**:
   - `contextra-types` enthält weder `async`, I/O noch Abhängigkeiten zu `tokio`.
   - `contextra-ports` nutzt `BoxFuture` für Trait-Signaturen, enthält aber keine Runtime-Abhängigkeit.
4. **Keine verbotenen historischen Crates**:
   - Die gestrichenen Crates (`graph`, `checkpoint`, `ingest`, `privacy`, `mcp`, `infer-*`, `candle`, `wire`, `license`, `db`) dürfen nirgends im Workspace als Abhängigkeit auftauchen.

## Verweise
- `docs/SPEC.md` §5.2, §5.3
- `AGENTS.md` Regel 1
- `tests/arch/AGENTS.md`
