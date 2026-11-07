# AGENTS.md — `contextra-infer-candle`

> **Ring 2** · Klasse `[R]` (Rand) · Reifegrad `siehe `capabilities.toml`` · Unsafe: nein · Rolle: Candle/GGUF-Embedding- und Inferenzadapter (lokal, optional CUDA).
> Gemeinsame Regeln stehen in der Root-`AGENTS.md`; diese Datei nennt nur Crate-Spezifisches. Stand 2026-11-07, statisch gelesen, **nicht kompiliert**; Zahlen sind grep-Heuristiken.

## 1. Baukasten-Profil

| Aspekt | Wert |
|---|---|
| Standalone-Build | `cargo build -p contextra-infer-candle --no-default-features --locked` |
| Contextra-Abhängigkeiten **Ist** (Cargo) | `crypto`, `ports`, `rank`, `types` |
| Contextra-Abhängigkeiten **Soll** (PLAN §4) | `ports`, `types`, `rank` |
| Features | `candle`, `cuda`, `kv-bridge`, `kv-stage-b`, `test-utils` |
| Lints | `#![forbid(unsafe_code)]` |

## 2. Abhängigkeiten & Schulden

- **Überzählige Kante `crypto`** (Soll entfällt).
- `contextra` (Fassade) hängt hart an diesem Crate (Ist); Soll: nur optional (Feature `candle`, SPEC §4.5).
- ~11 direkte `fs`-Treffer (Modell-Laden): über `Vfs` oder klar als Rand-I/O kennzeichnen.

## 3. Modul-Karte (am Code verifiziert)

| Pfad | Verantwortung |
|---|---|
| `embedding.rs`, `embedding_provider.rs` | Embedder |
| `inference/`, `model/`, `gguf_loader.rs`, `model_registry.rs` | Inferenz, Modellladen |
| `kv_bridge.rs`, `kv_state.rs`, `attention_exporter.rs`, `gasp.rs` | Brücke zum KV-Cache, Aufmerksamkeitsexport |

## 4. Verträge & Invarianten

1. Siehe `infer-onnx` (Feature-Gate, Blockierung, Modell-Hash, Nichtdeterminismus als WAL-Wert).
2. Keine Schlüssel-/Löschlogik hier; KV-Verschlüsselung nur über Ports.
3. `test-utils` nie im Release-/Default-Set (INV-EMBED-1).

## 5. Ports

**Implementiert:** `EmbeddingProvider`, KV-Brückenports.

## 6. Nebenläufigkeit & Performance

- CPU-lastig: Compute-Pool, nie Executor-Thread.
- Prefix-Wiederverwendung (`prefix_reuse`) nur innerhalb eines Tenants.

## 7. Anti-Patterns

- Standard-Build der Fassade erzwingt Candle.
- Mock-Embedder als Produktionsstandard.

## 8. Tests & Messung

```bash
cargo test -p contextra-infer-candle --locked
cargo clippy -p contextra-infer-candle --all-targets -- -D warnings
```

- Backend-Tests laufen ohne Modell/Netz über Fakes; echte Modelle nur hinter Feature und in separatem CI-Job.
- Konformitätslauf gegen `ports::contract_tests` für `EmbeddingProvider`.

## 9. Offene Arbeit

| WP | Inhalt |
|---|---|
| 5.3 | `crypto`-Kante und harte Fassadenkante lösen |
