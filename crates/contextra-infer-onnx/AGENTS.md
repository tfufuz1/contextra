# AGENTS.md — `contextra-infer-onnx`

> **Ring 2** · Klasse `[R]` (Rand) · Reifegrad `siehe `capabilities.toml`` · Unsafe: nein · Rolle: ONNX-Embedding- und Cross-Encoder-Reranking-Adapter (`ort`).
> Gemeinsame Regeln stehen in der Root-`AGENTS.md`; diese Datei nennt nur Crate-Spezifisches. Stand 2026-11-07, statisch gelesen, **nicht kompiliert**; Zahlen sind grep-Heuristiken.

## 1. Baukasten-Profil

| Aspekt | Wert |
|---|---|
| Standalone-Build | `cargo build -p contextra-infer-onnx --no-default-features --locked` |
| Contextra-Abhängigkeiten **Ist** (Cargo) | `ports`, `rank`, `types` |
| Contextra-Abhängigkeiten **Soll** (PLAN §4) | `ports`, `types`, `rank` |
| Features | `onnx`, `reranking` (`default` enthält sie nicht zwingend: prüfen) |
| Lints | `#![forbid(unsafe_code)]` |

## 2. Abhängigkeiten & Schulden

- Kanten Ist = Soll. Externe Schwergewichte: `ort`, `reqwest`, `tokio`.
- `engine` hängt hart an diesem Crate (Ist): Soll nur über Port/Feature (WP 5.3).

## 3. Modul-Karte (am Code verifiziert)

| Pfad | Verantwortung |
|---|---|
| `lib.rs` | Embedder, Konfiguration |
| `reranker/` (`onnx`, `cross_encoder`, `config`) | Cross-Encoder-Reranking |

## 4. Verträge & Invarianten

1. **Feature-Gate:** `onnx` ist nie im Minimalbau (M5); aufrufender Code liegt hinter `cfg(feature = "onnx")`.
2. **Inferenz blockiert:** `session.run()` nie direkt im Executor, sondern `spawn_blocking`/Compute-Pool.
3. **Modell-Identität (INV-EMBED-2):** Modell-Hash und Dimension werden gemeldet; abweichender Hash wird von der Collection abgewiesen.
4. **Nichtdeterministische Ausgabe** geht als Wert ins WAL (INV-NONDET-1); Index-Rebuild ruft das Modell nie erneut.
5. Session-Pool einmal laden und per `Arc` teilen.

## 5. Ports

**Implementiert:** `EmbeddingProvider`, `Reranker`.

## 6. Nebenläufigkeit & Performance

- Sessions poolen; Thread-Anzahl konfigurierbar; kein Laden pro Request.

## 7. Anti-Patterns

- Modell pro Anfrage laden.
- Embedder ohne Dimensionsprüfung.

## 8. Tests & Messung

```bash
cargo test -p contextra-infer-onnx --locked
cargo clippy -p contextra-infer-onnx --all-targets -- -D warnings
```

- Backend-Tests laufen ohne Modell/Netz über Fakes; echte Modelle nur hinter Feature und in separatem CI-Job.
- Konformitätslauf gegen `ports::contract_tests` für `EmbeddingProvider`.

## 9. Offene Arbeit

| WP | Inhalt |
|---|---|
| 5.3 | Kante `engine → infer-onnx` per Feature/Port lösen |
