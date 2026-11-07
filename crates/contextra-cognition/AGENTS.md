# AGENTS.md — `contextra-cognition`

> **Ring 3** · Klasse Rand `[R]` (SPEC-Katalog: Archiv, Root-§10 K5) · Reifegrad `experimental` · Unsafe: nein · Rolle: Konsolidierung und Langzeitgedächtnis: Aggregation, Synthese, Kontext-Kompaktierung, Wartungsplaner.
> Gemeinsame Regeln stehen in der Root-`AGENTS.md`; diese Datei nennt nur Crate-Spezifisches. Stand 2026-11-07, statisch gelesen, **nicht kompiliert**; Zahlen sind grep-Heuristiken.

## 1. Baukasten-Profil

| Aspekt | Wert |
|---|---|
| Standalone-Build | `cargo build -p contextra-cognition --no-default-features --locked` |
| Contextra-Abhängigkeiten **Ist** (Cargo) | `engine`, `graph`, `ports`, `types` |
| Contextra-Abhängigkeiten **Soll** (PLAN §4) | `engine`, `ports`, `types` |
| Features | `edge-reinforcement-learning`, `graph-connectivity-health` |
| Lints | `#![forbid(unsafe_code)]`, `deny(dead_code)` |

## 2. Abhängigkeiten & Schulden

- **Überzählige Kante `graph`**; Dev-Kanten `infer-candle`, `store`, `vector`, `testkit` (M6).
- ~37 `async fn`, ~23 `tokio::`-Treffer.

## 3. Modul-Karte (am Code verifiziert)

| Pfad | Verantwortung |
|---|---|
| `memory_consolidation.rs`, `consolidation_executor.rs`, `consolidation_locks.rs` | Konsolidierung |
| `aggregation_phase.rs`, `synthesis_phase.rs`, `semantic_aggregation_facade.rs`, `transitivity_veto.rs` | Phasen, Transitivitäts-Veto |
| `context_compaction/`, `context.rs`, `channel.rs`, `graph_sink.rs`, `leanrag_input.rs` | Kompaktierung, Kanäle, Graph-Senke |
| `maintenance_scheduler.rs`, `maintenance_config.rs` | Planer |

## 4. Verträge & Invarianten

1. **INV-RETENTION-1:** Ranking-Abwertung löscht nie; Löschung nur durch ausdrückliche Aktion oder opt-in `RetentionPolicy` (Dry-run, Karenz, Audit).
2. **INV-DERIVED-1:** Ableitungen (Zusammenfassungen, Fakten) hängen an Lineage; bei Löschung der Quelle neu ableiten oder als `stale` markieren: Zuständigkeit liegt bei `erasure`/`engine`-Ports, nicht hier.
3. LLM-Aufrufe nie unter Locks; Konsolidierungs-Locks haben feste Ordnung (`consolidation_locks.rs`).

## 5. Ports

Konsumiert `engine`, `GraphIndex`, `QueryRewriter`.

## 6. Nebenläufigkeit & Performance

- Hintergrundarbeit über Planer mit Abbruch (`CancelToken`); kein geteilter Veränderlicher außer in `consolidation_locks`.

## 7. Anti-Patterns

- Konsolidierung löscht Quelldaten implizit.
- Direkter Zugriff auf `store`/`vector` statt über `engine`.

## 8. Tests & Messung

```bash
cargo test -p contextra-cognition --locked
cargo clippy -p contextra-cognition --all-targets -- -D warnings
```

- 13 Testdateien; Dev-Kanten durch `testkit` ersetzen.

## 9. Offene Arbeit

| WP | Inhalt |
|---|---|
| 3.6 | Löschkaskaden entfernen |
| M6 | Dev-Kanten bereinigen |
