# Copilot-Anweisungen für Contextra

<!-- doc-stamp: 2026-10-30 | Normativ: docs/SPEC.md Fassung 4.0 -->

Contextra ist eine eingebettete Rust-Bibliothek für hybride Suche (HNSW und BM25) mit Mandantentrennung und Verschlüsselung im Ruhezustand. Der Scope ist auf Kern v1 reduziert.

**Build-Status:** Nicht grün. Siehe `docs/PLAN.md` Phase R0. Vorschläge, die einen bestehenden Verweis auf eine entfernte Crate fortführen, sind falsch.

## Regeln

- `unsafe` nur in `contextra-sys` und `contextra-simd`, jeweils mit `// SAFETY:`. Alle anderen Crates: `#![forbid(unsafe_code)]`.
- Keine `unwrap`, `expect`, `panic!`, `todo!`, `unimplemented!`, `unreachable!` oder ungeprüftes Indexieren in Produktionscode.
- Zeit, Zufall und IDs nur über `Clock`, `Rng`, `IdGen` aus `contextra-ports`.
- I/O nur über `Vfs`. `fsync`-Fehler mit `?` propagieren.
- Mandant immer aus `RequestContext`, nie als Literal oder Default.
- Verschlüsselung ist nicht optional. Kein `cfg(feature)`-Schalter für Sicherheitscode.
- Keine Doppeldefinitionen öffentlicher Typen. Kanonische Definition vor dem Hinzufügen suchen.

## Ringe

| Ring | Crates |
|---|---|
| 0 | `contextra-types`, `contextra-ports`, `contextra-sys`, `contextra-simd`, `contextra-mvcc`, `contextra-crypto`, `contextra-vector`, `contextra-text`, `contextra-rank` |
| 1 | `contextra-store` |
| 2 | `contextra-engine` |
| 3 | `contextra-db` |
| 4 | `contextra` |

Abhängigkeiten zeigen nur auf gleichen oder niedrigeren Ring.

## Nicht vorschlagen

Diese Crates sind entfernt und dürfen nicht neu angelegt oder referenziert werden: `contextra-graph`, `contextra-cognition`, `contextra-adapt`, `contextra-kvcache`, `contextra-router`, `contextra-agent`, `contextra-infer-*`, `contextra-mcp`, `contextra-py`, `contextra-privacy`, `contextra-ingest`, `contextra-sandbox`, `contextra-wire`, `contextra-checkpoint`, `contextra-license`, `contextra-audit-export`, `contextra-avv-generator`, `contextra-core`.
