# Supply-Chain Advisory Triage — Contextra Workspace

Stand: 2026-10-29
Governance-Ref: SPEC §4.9 (W8), ADR-D10, EP16

Diese Datei dokumentiert die Sicherheitsbewertung und Triage aller unmaintained und unsound Abhängigkeiten im Contextra-Workspace.

| Paket | Version | Typ / ID | Betroffene Crates | Erreichbarkeit | Alternative | Entscheidung & Ablaufdatum |
|---|---|---|---|---|---|---|
| `lru` | 0.16.4 | Unsound<br>(RUSTSEC-2026-0253) | `contextra-kvcache`, `contextra-text` | In-Memory-LRU-Cache; `pop()` Aufrufe in panic-freien Pfaden. | `lru-rs`, `moka` | Befristet ignoriert bis 2027-04-30 (ADR folgt an GV-1). |
| `scc` | 2.4.0 | Unsound<br>(RUSTSEC-2026-0205) | Transitive Abhängigkeit (`contextra-mcp`) | `Array::insert` Exception Safety betrifft Custom-Comparator Panics, in Contextra panikfrei. | Upgrade auf `scc` 3.x bei nächstem Major-Cut. | Befristet ignoriert bis 2027-04-30 (ADR folgt an GV-1). |
| `tokio` | 1.53.1 | Unsound<br>(RUSTSEC-2026-0386) | Rand-Crates (`contextra-mcp`, `contextra-infer-ollama` etc.) | `AsyncFd` wird im Workspace nicht verwendet. | Update auf neuere Tokio-Patches. | Befristet ignoriert bis 2027-04-30 (TRANSITION E29, REQ-ASYNC-1). |
| `bincode` | 1.3.3 | Unmaintained<br>(RUSTSEC-2025-0141) | Interne Serialisierung | Nur für interne binäre Nutzdatenstrukturen im Speicher. | `bincode` v2, `postcard` | Befristet ignoriert bis 2027-04-30 (ADR folgt an GV-1). |
| `fxhash` | 0.2.1 | Unmaintained<br>(RUSTSEC-2025-0057) | In-Memory Hashing | Schnelle In-Memory Non-Cryptographic Hashmap-Keys. | `rustc-hash`, `ahash` | Befristet ignoriert bis 2027-04-30 (ADR folgt an GV-1). |
| `instant` | 0.1.13 | Unmaintained<br>(RUSTSEC-2024-0384) | WASM/Std Zeit-Fallback | Platform-Fallback für `Instant::now`. | `web-time`, `std::time::Instant` | Befristet ignoriert bis 2027-04-30 (ADR folgt an GV-1). |
| `number_prefix` | 0.4.0 | Unmaintained<br>(RUSTSEC-2025-0119) | CLI / Logging | Reine Formatierung von Byte-Größen. | Inlining der Präfix-Berechnung | Befristet ignoriert bis 2027-04-30 (ADR folgt an GV-1). |
| `paste` | 1.0.15 | Unmaintained<br>(RUSTSEC-2024-0436) | Compile-time Macros | Proc-Macro zur Bezeichner-Verknüpfung. | Standard Rust 1.89 / Macro-by-Example | Befristet ignoriert bis 2027-04-30 (ADR folgt an GV-1). |

---

## Zip & Wasmtime Triage

### Zip Crate (`zip` 0.6.6 / 8.6.0 / RUSTSEC-2026-0367, -0368, -0369)
- **Erreichbarkeit:** `ZipArchive::extract()` und `ZipStreamReader::extract()` sind im gesamten Workspace (`crates/`) unbenutzt und nicht erreichbar.
- **Herkunft:** Transitiv über `candle-core` v0.11.0 (`contextra-infer-candle`) und `contextra-ingest`.
- **Entscheidung:** Befristeter Ignore bis 2027-01-31. Kein Major-Update von Candle in diesem PR.

### Wasmtime (`wasmtime` 25.0.3 / 21 RUSTSEC IDs)
- **Erreichbarkeit:** Eingebettet in `contextra-sandbox` für isolierte WASM-Gäste.
- **Entscheidung:** Befristeter Ignore bis 2027-01-31. Vollständige Behebung und Upgrade ist Aufgabe SC-2.
