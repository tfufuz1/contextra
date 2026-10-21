# RESTORE-KARTE: PARK-ONNX

- **ID**: PARK-ONNX
- **Status**: parked
- **Crates und Pfade**: `crates/contextra-infer-onnx`
- **Größe in Zeilen**: 2.826 Zeilen Code/Rust-Quellcode (6.957 Zeilen inkl. Fixtures & Docs)
- **BASE_SHA**: `0a629bfb605071274d06c1a9ff6a45054934c153`
- **Tag**: `park/PARK-ONNX/pre`
- **Wiring-Patch**: `docs/restore/PARK-ONNX/wiring.patch`

## Gelöste Aufrufstellen (Un-wired Edges)
- `crates/contextra/Cargo.toml:28`: Optionale Abhängigkeit `contextra-infer-onnx` entfernt.
- `crates/contextra/Cargo.toml:90`: Feature `onnx = ["contextra-infer-onnx/onnx", "contextra-db/onnx"]` entfernt.
- `crates/contextra/src/lib.rs:48`: `#[cfg(feature = "onnx")] pub use contextra_infer_onnx as onnx;` Re-Export entfernt.
- `crates/contextra-mcp/Cargo.toml:15`: Feature `onnx = ["contextra-infer-onnx", "contextra-infer-onnx/onnx"]` entfernt.
- `crates/contextra-mcp/Cargo.toml:31`: Optionale Abhängigkeit `contextra-infer-onnx` entfernt.
- `crates/contextra-mcp/src/config.rs:12`: Enum-Variante `EmbeddingBackend::LocalOnnx` entfernt.
- `crates/contextra-mcp/src/config.rs:75`: Provider-Parsing angepasst: Anfragen für Provider `"onnx"` / `"localonnx"` liefern typisierten Fehler `ContextraError::InvalidInput("Backend onnx ist in v1.0 geparkt, siehe docs/restore/PARK-ONNX")`.
- `crates/contextra-infer-ollama/Cargo.toml:31`: `dev-dependencies` Eintrag `contextra-infer-onnx` entfernt.
- `crates/contextra-infer-ollama/src/client/config.rs:12`: Doc-Kommentar Verweis auf `contextra_infer_onnx::MAX_EMBED_BATCH_SIZE` bereinigt.

## Entfernte Features, Tools & Konfigurationswerte
- Feature `onnx` in `crates/contextra` und `crates/contextra-mcp`.
- Config-Wert `onnx` / `LocalOnnx` in `crates/contextra-mcp` (jetzt mit explizitem Park-Fehler hinterlegt).

## Abhängigkeiten & Schnittstellen
- Ports/Traits: `contextra_ports::EmbeddingProvider`, `contextra_ports::Reranker`.
- System-Dependencies: Native ONNX Runtime C-Bibliothek (`ort`).

## Test-Manifest & Verschobene Tests
- `docs/restore/PARK-ONNX/tests.manifest`: Keine isolierten Test-Funktionen aus den Allowlist-Crates verschoben.
- Verschobene Test-Anzahl: 0 (Crate-interne Tests wurden zusammen mit `crates/contextra-infer-onnx` entfernt).

## Aktivierungsregel & Abnahme-Gate
- **Wiederherstellungsbedingung**: R-INFER-ALT (v1.8) oder für Candle-/ONNX-Reranker (v1.3).
- **Abnahme**: `rg -n 'infer.onnx|LocalOnnx' crates/contextra crates/contextra-mcp crates/contextra-infer-ollama` liefert nur Karten-/Fehlertext, keinen Abhängigkeits-Rest.

## Wiederherstellungs-Aufwandsschätzung
- **Aufwand**: M (Medium)
- **Begründung**: C-FFI / Native `ort` Bindings und Feature-Gating in Facade/MCP müssen wieder eingehängt und verifiziert werden.

## Bekannte Drift
- *(anfangs leer)*

## Wiederherstellungsschritte
1. Branch `restore/PARK-ONNX` von Ziel-Branch erstellen.
2. Code wiederherstellen: `git restore --source=0a629bfb605071274d06c1a9ff6a45054934c153 -- crates/contextra-infer-onnx`
3. Manifeste & Workspace-Dateien in Wurzel `Cargo.toml`, `capabilities.toml`, `crates/contextra/Cargo.toml`, `crates/contextra-mcp/Cargo.toml` u. a. wieder ergänzen.
4. Patch anwenden: `git apply -R --3way docs/restore/PARK-ONNX/wiring.patch`
5. Isolierter Build-Check: `cargo check -p contextra-infer-onnx`
6. Integrationstests und Gates ausführen: `cargo test -p contextra-infer-onnx`
7. Wiederherstellung dokumentieren und Tag `restore/PARK-ONNX/done` setzen.
