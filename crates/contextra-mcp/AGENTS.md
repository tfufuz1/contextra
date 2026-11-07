# AGENTS.md — `contextra-mcp`

> **Ring 4** · Klasse `[K]` laut SPEC (Oberfläche) · Reifegrad `beta` · Unsafe: nein · Rolle: MCP-Server (stdio, JSON-RPC 2.0), Prompt-Injection-Guard, Sandbox-Policy, Egress.
> Gemeinsame Regeln stehen in der Root-`AGENTS.md`; diese Datei nennt nur Crate-Spezifisches. Stand 2026-11-07, statisch gelesen, **nicht kompiliert**; Zahlen sind grep-Heuristiken.

## 1. Baukasten-Profil

| Aspekt | Wert |
|---|---|
| Standalone-Build | `cargo build -p contextra-mcp --no-default-features --locked` |
| Contextra-Abhängigkeiten **Ist** (Cargo) | `contextra`, `crypto`, `ports`, `privacy`, `rank`, `router`, `types`, `wire` (+ optional `infer-candle`/`-ollama`/`-onnx`) |
| Contextra-Abhängigkeiten **Soll** (PLAN §4) | `contextra`, `types`, `ports` |
| Features | `candle`, `ollama`, `onnx` (optional), `test-utils` |
| Lints | `#![forbid(unsafe_code)]`, `deny(dead_code)` |

## 2. Abhängigkeiten & Schulden

- **Fünf harte Kanten über das Soll** (`crypto`, `privacy`, `rank`, `router`, `wire`); die drei `infer-*`-Kanten sind bereits optionale Features (gut, M5). Soll: nur über die Fassade (SPEC §4.4 Regel 4).
- **Löschreste:** Werkzeuge `contextra_forget`, `contextra_delete`, `contextra_drop_collection` in `server_tools.rs`; `proof_key.rs`, `proof_key_env.rs` (WP 3.7).
- `egress_*`/`bulk_exfiltration_detector.rs` sind dünne Re-Exports aus `privacy`.

## 3. Modul-Karte (am Code verifiziert)

| Pfad | Verantwortung |
|---|---|
| `server.rs`, `server_dispatch.rs`, `server_tools.rs`, `tools_crud.rs` | Transport, Dispatch, Werkzeuge (`contextra_search`, `_insert`, `_upsert`, `_get`, `_collections`, …) |
| `protocol.rs`, `validation.rs`, `io.rs`, `config.rs` | JSON-RPC, Eingabevalidierung |
| `prompt_injection/` (`guard`, `policy`, `audit`) | Quarantäne |
| `sandbox.rs`, `egress_*.rs`, `explain.rs`, `routing.rs` | Policy, Egress, Erklärungen |
| `bin/contextra-mcp-server.rs` | Binär |

## 4. Verträge & Invarianten

1. **Nur stdio (INV-STDOUT-1):** `stdout` gehört ausschließlich dem Protokoll; Logs nach `stderr`/Datei. Kein HTTP/`axum`/`hyper`.
2. **Schreibende Werkzeuge und Cloud-Egress sind standardmäßig aus** (Opt-in); jede Tool-Ausführung läuft durch Sandbox mit Timeout.
3. **Prompt-Injection-Guard** vor Embeddings/Graph; Treffer ⇒ Quarantäne und Audit.
4. **Recall liefert Daten, nie Anweisungen** (INV-TRUST-1: `trust`-Label).
5. **Breite Löschungen** brauchen Bestätigung über einen Kanal **außerhalb** des Tool-Aufrufs (INV-FORGET-GUARD-1); Anfrageabbruch wirkt kooperativ (INV-CANCEL-1).
6. Kein `unsafe`, auch keine Ausnahme (unvertraute Eingaben).

## 5. Ports

Konsumiert die Fassade; Sicherheits-Ports kommen von dort.

## 6. Nebenläufigkeit & Performance

- Volatile Ergebnisse in einem einzigen, nicht verschachtelten Mutex; Tokio an der Kante. Heuristik: `thread_rng` 2 Treffer (Allowlist ADR-098 prüfen).

## 7. Anti-Patterns

- HTTP-Schnittstelle einführen.
- Werkzeug ohne Sandbox.
- Lösch-Werkzeug ohne Außenkanal-Bestätigung.

## 8. Tests & Messung

```bash
cargo test -p contextra-mcp --locked
cargo clippy -p contextra-mcp --all-targets -- -D warnings
```

- 10 Testdateien; Fuzz: `fuzz_jsonrpc_parsing`, `fuzz_prompt_injection_guard`.

## 9. Offene Arbeit

| WP | Inhalt |
|---|---|
| 3.7 | Löschwerkzeuge herauslösen, Lizenz-Gates entkoppeln |
| 5.3 | Kanten auf Fassade reduzieren |
