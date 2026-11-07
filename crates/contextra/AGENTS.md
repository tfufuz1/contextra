# AGENTS.md — `contextra`

> **Ring 4** · Klasse `[K]` (Fassade, Composition Root) · Reifegrad `stable` · Unsafe: nein · Rolle: Öffentliche High-Level-Fassade und einzige Composition Root (`ContextraBuilder`).
> Gemeinsame Regeln stehen in der Root-`AGENTS.md`; diese Datei nennt nur Crate-Spezifisches. Stand 2026-11-07, statisch gelesen, **nicht kompiliert**; Zahlen sind grep-Heuristiken.

## 1. Baukasten-Profil

| Aspekt | Wert |
|---|---|
| Standalone-Build | `cargo build -p contextra --no-default-features --locked` |
| Contextra-Abhängigkeiten **Ist** (Cargo) | `db`, `infer-candle`, `license`, `ports`, `rank`, `types` |
| Contextra-Abhängigkeiten **Soll** (PLAN §4) | Bausteine nach Bedarf; Rand nur `optional = true` |
| Features | `default`, `embedded`, `candle` |
| Lints | `#![forbid(unsafe_code)]` |

## 2. Abhängigkeiten & Schulden

- **Hart gebundene Randkanten:** `infer-candle`, `license` (Soll: optional; `default = ["embedded"]` ohne Inferenz, SPEC §4.5).
- Re-exportiert `rank` und `candle` (M3-Prüfung).
- Löschreste: `KvDeleteMode`, 5 `forget`-Treffer; Test `deletion_chain_e2e.rs`. API `erase`/`erase_status`/`report` nur als optionale Brücke zu `erasure` (WP 3.8).
- `capabilities.toml` erlaubt noch `core`.

## 3. Modul-Karte (am Code verifiziert)

| Pfad | Verantwortung |
|---|---|
| `builder.rs`, `boot.rs` | Verdrahtung, Startsequenz (B0–B9) |
| `agent_memory.rs` | `AgentMemory`, `Memory` |
| `collection_profile.rs`, `performance_profile.rs` | Profile, Durability-Konfiguration |
| `query_rewriter.rs` | `LlmQueryRewriter` |

## 4. Verträge & Invarianten

1. **Nur hier wird instanziiert und verdrahtet** (keine globalen Singletons, EP7). Fehlt ein Sicherheits-Port, verweigert der Start mit benanntem Grund (INV-BOOT-1, REQ-PORT-2).
2. Jede Fähigkeit ist verdrahtet **und** durch Integrationstest belegt (P29).
3. Rand-Features sind im Release-Default aus (P30).
4. Mandant aus `RequestContext`, nie aus Default.

## 5. Ports

Verdrahtet alle Ports; wählt Implementierungen.

## 6. Nebenläufigkeit & Performance

- Tokio-Runtime nur hier/`mcp`; für Bibliotheksnutzer ohne Async eine synchrone Fassade (`block_on` intern, dokumentiert).

## 7. Anti-Patterns

- Fachlogik in der Fassade.
- Rand-Crate als Pflichtabhängigkeit.

## 8. Tests & Messung

```bash
cargo test -p contextra --locked
cargo clippy -p contextra --all-targets -- -D warnings
```

- 18 Testdateien; Modul-Entfernungstest: baut ohne jedes Rand-Modul.

## 9. Offene Arbeit

| WP | Inhalt |
|---|---|
| 3.8 | Erasure-API als optionale Brücke |
| 5.3 | `infer-candle`/`license` optional |
