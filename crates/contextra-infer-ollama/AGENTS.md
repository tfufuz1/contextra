# AGENTS.md — `contextra-infer-ollama`

> **Ring 2** · Klasse `[R]` (Rand) · Reifegrad `siehe `capabilities.toml`` · Unsafe: nein · Rolle: HTTP-Adapter für lokale/entfernte Ollama-Instanzen (Embeddings, Generierung, Query-Rewrite).
> Gemeinsame Regeln stehen in der Root-`AGENTS.md`; diese Datei nennt nur Crate-Spezifisches. Stand 2026-11-07, statisch gelesen, **nicht kompiliert**; Zahlen sind grep-Heuristiken.

## 1. Baukasten-Profil

| Aspekt | Wert |
|---|---|
| Standalone-Build | `cargo build -p contextra-infer-ollama --no-default-features --locked` |
| Contextra-Abhängigkeiten **Ist** (Cargo) | `ports`, `types` |
| Contextra-Abhängigkeiten **Soll** (PLAN §4) | `ports`, `types` |
| Features | keine |
| Lints | `#![forbid(unsafe_code)]` |

## 2. Abhängigkeiten & Schulden

- Kanten Ist = Soll. `capabilities.toml` nennt zusätzlich `rank` (stale).
- Externe: `reqwest`, `tokio`, `futures`.

## 3. Modul-Karte (am Code verifiziert)

| Pfad | Verantwortung |
|---|---|
| `client/` (`core`, `config`, `policy`, `circuit_breaker`, `validation`, `wire`, `errors`, `trait_impls`) | HTTP-Client mit Retry, Circuit-Breaker, Validierung |
| `embedding.rs`, `query_rewriter.rs`, `context_prefixer.rs`, `importance.rs`, `model_info.rs` | Port-Adapter und Helfer |

## 4. Verträge & Invarianten

1. **Immer über `OllamaClient`** (Retry, Backoff, Fehlerabbildung); kein rohes `reqwest`.
2. **Egress:** jeder Netzaufruf ist Egress und läuft hinter der Egress-Richtlinie der Fassade (INV-EGRESS-BUDGET-1); Standard ist lokal.
3. Antworten sind untrusted: Größen-/Schemavalidierung (`validation.rs`).
4. Dimension per `known_dimension()` oder einmaliger Messung; abweichende Dimension ⇒ Fehler.
5. Keine Nutzinhalte in Logs (INV-LOG-1).

## 5. Ports

**Implementiert:** `EmbeddingProvider`, `QueryRewriter`.

## 6. Nebenläufigkeit & Performance

- Circuit-Breaker begrenzt Fehlerlawinen; Concurrency begrenzen (lokale GPU/OOM).

## 7. Anti-Patterns

- Modellverfügbarkeit nicht prüfen.
- Unbegrenzte parallele `embed_batch`-Aufrufe.

## 8. Tests & Messung

```bash
cargo test -p contextra-infer-ollama --locked
cargo clippy -p contextra-infer-ollama --all-targets -- -D warnings
```

- Backend-Tests laufen ohne Modell/Netz über Fakes; echte Modelle nur hinter Feature und in separatem CI-Job.
- Konformitätslauf gegen `ports::contract_tests` für `EmbeddingProvider`.

## 9. Offene Arbeit

| WP | Inhalt |
|---|---|
| 5.3 | `mcp → infer-ollama` nur über Feature `ollama` (bereits optional) |
