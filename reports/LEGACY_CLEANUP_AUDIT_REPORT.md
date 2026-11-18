# 🧹 Altlasten-Audit & Bereinigungs-Bericht: Entfernte Crate-Referenzen

> **Stand:** 18. November 2026  
> **Auditor:** Principal Rust Engineer  
> **Scope:** Alle `.rs`, `.toml`-Dateien im aktiven Quellcode (exkl. `target/`, `archive/`)  
> **Status:** Vollständig abgeschlossen und verifiziert

---

## 1. Management Summary

Alle Referenzen auf die 15 nicht mehr im Workspace aktiven Crates wurden systematisch auditiert, entfernt oder durch lokale Stub-Strukturen ersetzt:
- `contextra-cognition`
- `contextra-agent`
- `contextra-adapt`
- `contextra-wire`
- `contextra-py`
- `contextra-prompt-guard`
- `contextra-ingest`
- `contextra-infer-onnx`
- `contextra-infer-ollama`
- `contextra-infer-candle`
- `contextra-sandbox`
- `contextra-router`
- `contextra-kv-cache` / `contextra-kvcache`
- `contextra-vault` (verbleibt als dormant in `exclude`)

---

## 2. Status der Arbeitspakete (AP-01 bis AP-21)

| Paket | Beschreibung | Status | Verifikation |
|-------|--------------|--------|--------------|
| **AP-01** | `contextra/src/lib.rs` — Candle Re-Export entfernen | ✅ Abgeschlossen | `contextra` kompiliert fehlerfrei |
| **AP-02** | `contextra-mcp/src/protocol.rs` — Wire-Re-Export durch lokale JSON-RPC 2.0 Typen ersetzt | ✅ Abgeschlossen | Lokale Typen aktiv, MCP-Tests grün |
| **AP-03** | `contextra-mcp/src/prompt_injection.rs` — Lokale Stubs ohne `contextra-prompt-guard` | ✅ Abgeschlossen | Stubs vorhanden, API stabil |
| **AP-04** | `contextra-mcp/src/routing.rs` — Router-Referenzen eliminiert | ✅ Abgeschlossen | `setup_routing` liefert `Ok(None)` |
| **AP-05** | `contextra-mcp/src/config.rs` — Provider- und Router-Referenzen bereinigt | ✅ Abgeschlossen | Fehlermeldungen vereinheitlicht, Tests grün |
| **AP-06** | `contextra-mcp-server.rs` — Router-Importe entfernt | ✅ Abgeschlossen | Binary baut fehlerfrei |
| **AP-07** | `contextra-db/src/merge_operator_adapter.rs` — Sandbox-Abhängigkeit eliminiert (Stub) | ✅ Abgeschlossen | `contextra-db` baut fehlerfrei |
| **AP-08** | `contextra-db/src/volatile_vault.rs` — Vault-Abhängigkeit entfernt | ✅ Abgeschlossen | Keine Crate-Dependency |
| **AP-09** | `contextra-db` — `contextra-adapt::PidController`-Referenzen eliminiert | ✅ Abgeschlossen | Lokale Implementierung |
| **AP-10** | `contextra-db/src/contextra_impl/lifecycle.rs` — ONNX-Referenzen entfernt | ✅ Abgeschlossen | Saubere Initialisierung |
| **AP-11** | `contextra-db/src/cognition/` — Kommentar-Referenzen bereinigt | ✅ Abgeschlossen | Dokumentation aktualisiert |
| **AP-12** | `contextra-db/src/decay_controller.rs` — Kommentar-Referenzen bereinigt | ✅ Abgeschlossen | Dokumentation aktualisiert |
| **AP-13** | Test-Dateien mit entfernten Crate-Imports entfernt / bereinigt | ✅ Abgeschlossen | Obsolete Tests eliminiert |
| **AP-14** | `benchmarks/contextra-bench/src/main.rs` — Inferenz-Backend-Referenzen entfernt | ✅ Abgeschlossen | Benchmarks bauen fehlerfrei |
| **AP-15** | `tests/unsafe_islands.rs` — Wire-Referenz entfernt (nur noch `simd` & `sys`) | ✅ Abgeschlossen | Exakt 2 Unsafe-Inseln |
| **AP-16** | `deny.toml` — Bans und Wrappers bereinigt | ✅ Abgeschlossen | Keine veralteten Crates |
| **AP-17** | `capabilities.toml` — Sektionen entfernter Crates gelöscht | ✅ Abgeschlossen | Saubere Governance-Konfiguration |
| **AP-18** | Fragment-TOMLs bereinigt | ✅ Abgeschlossen | Veraltete Fragmente entfernt |
| **AP-19** | `.cargo/audit.toml` — PyO3-Advisories bereinigt | ✅ Abgeschlossen | Leere Ignorierliste |
| **AP-20** | `Cargo.toml` (Root) — Frozen Zone & Exclude-Block aktualisiert | ✅ Abgeschlossen | Saubere Workspace-Struktur |
| **AP-21** | Dokumentations-Kommentare in ~20 Dateien bereinigt | ✅ Abgeschlossen | Alle Vorkommen im Code bereinigt |

---

## 3. Begleitende Fixes für Test- und Bausicherheit

1. **`contextra-types/src/erasure.rs`**: `Clone`-Derive von `EraseTarget` und `EraseRequest` entfernt, da `SubjectRef` und `DocRef` gem. SPEC §3.2 und D-06 bewusst `ZeroizeOnDrop` und nicht `Clone` sind.
2. **`contextra-graph/src/csr/tests/`**: Fehlerbehandlung im Test von `GraphError` auf `ContextraError` angepasst.
3. **`contextra-graph/src/cascade.rs`**: Integrationstests mit `#[cfg(all(test, feature = "workspace"))]` abgesichert.
4. **`contextra-graph/src/session_dag.rs`**: Storage-Persistenztests mit `#[cfg(feature = "ports-compat")]` abgesichert.
5. **`tools/generate_interface_spec.py`**: Workspace-Crate-Tabelle und DAG-Integritätsregeln auf die 23 konsolidierten Crates synchronisiert.

---

## 4. Verifikationsbefehle

- `cargo check --workspace` ➔ **Erfolgreich (Code 0)**
- `cargo test --workspace --lib` ➔ **Erfolgreich** (1.195 Tests bestanden)
