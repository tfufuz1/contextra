# AGENTS.md — `contextra-graph`

> **Ring 1 (`capabilities.toml`: Ring 0 ≠)** · Klasse `[K]` im SPEC-Katalog (SPEC §0.5 archiviert; Root-§10 K5) · Reifegrad `stable` · Unsafe: nein · Rolle: CSR-Graphindex für semantische Netze, Pfad-RAG, PPR, Communities. Implementiert `GraphIndex`.
> Gemeinsame Regeln stehen in der Root-`AGENTS.md`; diese Datei nennt nur Crate-Spezifisches. Stand 2026-11-07, statisch gelesen, **nicht kompiliert**; Zahlen sind grep-Heuristiken.

## 1. Baukasten-Profil

| Aspekt | Wert |
|---|---|
| Standalone-Build | `cargo build -p contextra-graph --no-default-features --locked` |
| Contextra-Abhängigkeiten **Ist** (Cargo) | `ports`, `types` |
| Contextra-Abhängigkeiten **Soll** (PLAN §4) | `ports`, `types` |
| Features | `apprh-diffusion`, `k-path-diffusion`, `physio-percolation`, `ppr-forward-push`, `edge-reinforcement-learning`, `graph-connectivity-health`, `docid-128` |
| Lints | `#![forbid(unsafe_code)]`, `deny(dead_code)`, `deny(clippy::await_holding_lock)` |

## 2. Abhängigkeiten & Schulden

- `capabilities.toml` nennt noch `adapt` (Cargo nicht mehr): bereinigen. Dev-Kante auf `store` (M6, WP 4.3).
- ~21.600 Zeilen inkl. Tests; `entity_extraction.rs` ist Extraktionslogik (Zuordnung prüfen, evtl. `engine`/`ingest`).
- `purger.rs` implementiert `Purger`.

## 3. Modul-Karte (am Code verifiziert)

| Pfad | Verantwortung |
|---|---|
| `csr/` (`graph_index`, `graph_read`, `graph_write`, `inner`, `graph_persist`, `visibility`) | CSR-Struktur, Lesen/Schreiben, Persistenz, Sichtbarkeit |
| `ppr/`, `path_rag/`, `apprh/`, `tl_hfd/`, `percolation.rs` | Diffusions- und Pfadverfahren (Feature-gesteuert) |
| `community.rs`, `hyperedge*.rs`, `cascade.rs` | Communities, Hyperkanten, Kaskade |
| `purger.rs`, `provider.rs`, `provenance.rs` | Port-Implementierung, Provider, Herkunft |

## 4. Verträge & Invarianten

1. **Lock-freier Lesepfad** Ziel: CSR-Segment unveränderlich, Schreiben in Delta-Puffer, Merge atomar (WP 4.3).
2. **Tenant-Isolation (VETO-F10):** keine Kante über Tenants.
3. **Sichtbarkeit bitemporal** (`csr/visibility.rs`); Snapshots respektieren `born_seq`/`died_seq`.
4. Persistenz robust gegen Teilschreiben (`tests/graph_persist_robustness.rs`).
5. Keine Löschlogik; Bereinigung nur via `Purger`.

## 5. Ports

**Implementiert:** `GraphIndex`, `Purger`. **Konsumiert:** `Clock`.

## 6. Nebenläufigkeit & Performance

- Kein Guard über `.await` (Lint `deny`). Komplexität je Verfahren in `benches/` (`csr_traversal_bench`) und `tests/csr_complexity_bench.rs` belegen; Hub-Knoten separat (`hub_node_benchmark.rs`).

## 7. Anti-Patterns

- Graph-Kanten über Tenant-Grenzen.
- `graph → store`-Normalkante.
- Diffusionsverfahren ohne Schattenmodus produktiv schalten.

## 8. Tests & Messung

```bash
cargo test -p contextra-graph --locked
cargo clippy -p contextra-graph --all-targets -- -D warnings
cargo bench -p contextra-graph
```

- Vorhanden: Golden-Tests, Determinismus-Permutationstests, Crash-Recovery der Kaskade, Fuzz.

## 9. Offene Arbeit

| WP | Inhalt |
|---|---|
| 4.3 | Delta-Puffer, lock-freier Lesepfad, Dev-Kante `store` entfernen |
| 1.1 | Ring auf 1; `adapt` aus `may_depend_on` |
