# contextra-mcp — Spezifikation

> **Geltung:** Crate-lokale Spezifikation. Normativ bleibt `docs/SPEC.md` (Fassung 4.1, Stand 10.11.2026). Dieses Dokument überträgt die Anforderungen für dieses Crate, ergänzt sie um verifizierte Ist-Fakten und markiert Abweichungen.
> **Faktenbasis:** statische Analyse mit `tools/` am geklonten Repo-Stand (`main`, HEAD `f0eda1cb9`). `cargo` ist in der Analyseumgebung nicht verfügbar. **Es wurde nichts gebaut, getestet oder ausgeführt.**
> **Beleg-Marken:** *[V]* am Quelltext/Manifest verifiziert, *[T]* aus einem tools/-Bericht, *[S]* aus docs/SPEC.md übernommen, *[E]* Festlegung dieser Spezifikation.

**Pfad:** `crates/contextra-mcp/` · **Umsetzungsplan:** `PLAN.md` (im selben Verzeichnis)

## 1. Zweck

MCP-Server: Tools, Egress, Anbindung an Prompt-Guard. *[S]*

## 2. Zielzustand (docs/SPEC.md §3.1) [S]

| Merkmal | Ziel |
|---|---|
| Gruppe | apps |
| Ebene (Kern-Closure) | 3 |
| Kern-Closure | contextra, types, ports, wire, rank, crypto, privacy, router, prompt-guard |
| Default-Features | — |
| Fehlertyp (R3) | `McpError` |
| Brücke (hinter Feature `workspace`, §1.1) | — |

Ebenen (SPEC §1.1): E0 = Kern-Closure leer · E1 = Kern-Closure ⊆ E0, höchstens 4 Mitglieder · E2 = Integration · E3 = Anwendungen und Testwerkzeuge. Der Kern ist der Code mit `--no-default-features` ohne Feature `workspace`.

## 3. Ist-Zustand (tools/)

| Messgröße | Wert |
|---|---|
| Lines of Code (Produktion, `loc.py`) | 4978 Zeilen in 25 Dateien |
| Version / publish | workspace-vererbt / <unset> |
| Interne Normal-Abhängigkeiten (Ist, Cargo.toml) | contextra, types, ports, wire, rank, crypto, privacy, router, infer-candle, infer-ollama, infer-onnx |
| Interne Dev-Abhängigkeiten (Ist) | keine |
| Features (Ist, Anzahl) | 5 |
| Default-Features (Ist) | keine |
| `cfg(feature = …)`-Stellen in src/ | 26 |
| `#[test]`-Vorkommen (grep, src + tests) | 67 |
| zero-panic: Produktionsbefunde, bereinigt (Tool-Gesamt; davon in Testdateien) | 0 (Tool 0; 0 in Testdateien) |
| unsafe: Vorkommen / ohne SAFETY / Insel | 0 / 0 / False |
| Determinismus: Produktionsbefunde, bereinigt (Tool-Gesamt) | 25 (Tool 26) |
| Port-M3-Leaks (port_contract_audit) | 0 |
| Error-Enums (Ist) | McpError |

**Tool-Caveat:** `zero_panic_analyzer.py` klassifiziert `#[cfg(test)]`-Module (`tests.rs`) als Produktion. Im Repo betrifft das 449 von 478 Treffern. Die Zahlen oben sind deshalb bereinigt; die Rohzahl steht in Klammern. Die Korrektur am Tool ist eigener Schritt (S-00).

**Quellen:** `tools/loc.py`, `tools/zero_panic_analyzer.py`, `tools/unsafe_boundary_auditor.py`, `tools/determinism_guard.py`, `tools/port_contract_auditor.py`, `tools/error_topology_mapper.py`, `tools/cargo_dependency_graph.py`, Cargo.toml-Auswertung.

## 4. Ist-vs-Ziel-Abgleich [V]

- **Ist-Kanten, die das Ziel vorsieht:** `contextra`, `types`, `ports`, `wire`, `rank`, `crypto`, `privacy`, `router`, `prompt-guard` (optional: `infer-candle`, `infer-ollama`, `infer-onnx`).
- **Prompt-Guard:** Erfolgreich herausgelöst als autarker E0-Sicherheitsbaustein `contextra-prompt-guard` (MCP-2).
- **Ports-Entkopplung:** `ports_local`-Importe in `routing.rs` vollständig auf kanonische `contextra_ports` Traits umgestellt (S-62).
- **Default-Features:** Ist = Ziel (leer).
- **Baustein-Entkopplung:** Klare Trennung zwischen Protokoll (stdio JSON-RPC), Sandbox-Policy, Security-Guards und Speicherschicht.

## 5. Verifizierte Befunde und Abweichungen zu docs/SPEC.md [V]

- Determinismus-Audit: 20 `env::var`-Treffer und 26 Produktionstreffer insgesamt. docs/SPEC.md adressiert das nicht; die Stellen sind unten gelistet.

## 6. Anforderungen

### 6.1 Crate-Spezifikation (docs/SPEC.md §6) [S]

`prompt_injection/` → `prompt-guard` (Re-Export). `protocol.rs` bleibt (B16). `egress_*.rs` bleiben. `sandbox.rs` bleibt (`CryptoKey` im `crypto`-Kern).

### 6.3 Regeln für jeden Schritt [S]

INV-1 Formatgleichheit · INV-2 Pfadgleichheit (Re-Export mit `#[deprecated]`) · INV-3 Ein Schritt = Commit-Paar · INV-4 `cargo check --workspace --all-targets --locked` grün · INV-5 Determinismus (Tie-Break nach ID aufsteigend) · INV-6 Durability-Pfade unberührt · INV-7 keine Compliance-Aussagen · INV-8 keine Rückwärtspfeile. Vollständig: docs/SPEC.md §1.2, Regeln R1–R9: §3.2.

### 6.5 Fehlerabbildung [S]

Eigener Fehlertyp: `McpError` (thiserror, `#[non_exhaustive]`). Jede heute genutzte `ContextraError`-Variante hat genau eine lokale Entsprechung; `Display` bleibt bitgleich; keine Sammelvariante `Other(String)` ohne Quelle. Tabelle je Baustein: `docs/decoupling/errors/<crate>.md` (aus S-xx erzeugt). Quelle: §5.3.

## 7. Schnittstellen und Persistenz

Keine Zeile aus dem Golden-Korpus (§5.1) bezieht sich auf dieses Crate.

### 7.2 Öffentliche Schnittstelle

Ist-Stand und Ziel-Schnittstelle sind in den Abschnitten 3 bis 6 festgehalten. Die exakte Liste der `pub`-Symbole erhebt S-00 über `cargo public-api` (Gate G-A1); erlaubte Differenzen sind nur Re-Export-Pfade mit `#[deprecated]`.

## 8. Fundstellen (tools/, exakt) [T]

**Determinismus (Produktion, bereinigt: 25; die ersten 15):** Tool-Gesamt 26.

- `crates/contextra-mcp/src/validation.rs:5` — env_var — `std::env::var("CONTEXTRA_MCP_ALLOW_WRITE")`
- `crates/contextra-mcp/src/server.rs:107` — env_var — `let timeout_secs = std::env::var("CONTEXTRA_MCP_IDLE_TIMEOUT_SECS")`
- `crates/contextra-mcp/src/config.rs:205` — env_var — `let prefixed = std::env::var("CONTEXTRA_EMBEDDING_PROVIDER").ok();`
- `crates/contextra-mcp/src/config.rs:206` — env_var — `let unprefixed = std::env::var("EMBEDDING_PROVIDER").ok();`
- `crates/contextra-mcp/src/config.rs:219` — env_var — `let ollama_url = std::env::var("CONTEXTRA_OLLAMA_URL")`
- `crates/contextra-mcp/src/config.rs:222` — env_var — `let embed_model = std::env::var("CONTEXTRA_EMBED_MODEL")`
- `crates/contextra-mcp/src/config.rs:225` — env_var — `let onnx_model_path = std::env::var("CONTEXTRA_ONNX_MODEL_PATH")`
- `crates/contextra-mcp/src/config.rs:229` — env_var — `let candle_model_dir = std::env::var("CONTEXTRA_CANDLE_MODEL_DIR")`
- `crates/contextra-mcp/src/config.rs:369` — env_var — `let prefixed = std::env::var("CONTEXTRA_LLM_PROVIDER").ok();`
- `crates/contextra-mcp/src/config.rs:370` — env_var — `let unprefixed = std::env::var("LLM_PROVIDER").ok();`
- `crates/contextra-mcp/src/config.rs:383` — env_var — `let ollama_url = std::env::var("CONTEXTRA_OLLAMA_URL")`
- `crates/contextra-mcp/src/config.rs:387` — env_var — `std::env::var("CONTEXTRA_LLM_MODEL").unwrap_or_else(¦_¦ "llama3.2:3b".to_string());`
- `crates/contextra-mcp/src/config.rs:389` — env_var — `let candle_model_dir = std::env::var("CONTEXTRA_CANDLE_MODEL_DIR")`
- `crates/contextra-mcp/src/config.rs:497` — env_var — `let profiles_path = std::env::var("CONTEXTRA_ROUTER_PROFILES_PATH")`
- `crates/contextra-mcp/src/config.rs:501` — env_var — `let calibration_store_path = std::env::var("CONTEXTRA_ROUTER_CALIBRATION_PATH")`

**Fehler-Enums (Ist):**

- `crates/contextra-mcp/src/protocol.rs:16` — McpError

## 9. Befunde aus docs/SPEC.md (B-Nummern) [S]

**B16 — Einzelentscheidungen, die der Entwurf offen ließ**
> `mcp/protocol.rs` vs. `wire/jsonrpc.rs` — 5 gemeinsame, 7 nur-mcp (`McpError`, Tiefen-/Batch-Validierung), 10 nur-wire [V] — **Kein Merge**; beide bleiben. Dublette nur im Typnamen, nicht in der API.

## 10. Entscheidungen und Risiken [S]

**Entscheidungen (§9.1):**

| D-07 | `vault` baut auf `sys::vault`; `mcp::protocol` und `wire::jsonrpc` bleiben getrennt; `adapt::drift`/`rank::drift` und die beiden `query_rewriter` bleiben getrennt (B16) |

Nicht-Ziele (§9.3), gelten für alle Crates: physische Regruppierung, Mikro-Crate-Zerlegung (F-1), Verschieben von `Vfs`/`io_seam`, crates.io-Veröffentlichung, Formatänderungen außer ID-Breite, Änderung der FlatBuffers-Schemas.

## 11. Abnahme [S]

Ein Crate gilt als entnehmbar (R6), wenn in dieser Reihenfolge grün: **G-L2** (Null-Prüfungen, nur bei Brücke) → **G-L1** (Kern- und Feature-Closure) → **G-L3** (`--no-default-features`, `--each-feature`) → **G-L4** (isolierter Export). Danach **G-F1/G-D1/G-E1/G-C1** für den betroffenen Schritt (§8.4).

```bash
cargo check -p contextra-mcp --no-default-features --locked
cargo hack check -p contextra-mcp --each-feature --no-dev-deps --locked
python3 tools/isolation_proof.py --crate contextra-mcp --modes no-default,default,each-feature
```

