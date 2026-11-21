# CLAUDE.md — contextra-testkit

> Crate-Kontext. Gilt zusätzlich zur Root-`CLAUDE.md` und zu `docs/SPEC.md` (maßgeblich bei Widerspruch).

## Rolle
Prüfwerkzeuge für Tests: `ManualClock`, `InMemoryStorageEngine`, `FaultVfs`, Canary-Scanner, Mocks. **Nur Dev-Abhängigkeit** laut Spec (F-27). Hier soll `tests/layering.rs` liegen (G-03, **fehlt**).

## Eckdaten (Messung 21.11.2026)
- Ebene: **E3, dev** (SPEC §3.1). Ziel laut Spec: ~7.000 Zeilen.
- Umfang: 5.446 Zeilen `src`, 3 Testdateien, 39 `#[test]`.
- Features: `default`, `testkit`, `erasure-conformance`.
- Build: ok.
- Abhängigkeiten (Ist): ports, types.
- Genutzt von: Dev-Dependency anderer Crates.

## Befehle
```bash
RUSTUP_TOOLCHAIN=stable cargo test -p contextra-testkit
RUSTUP_TOOLCHAIN=stable cargo test -p contextra-testkit --test layering   # G-03, fehlt noch
```

## Ist-Abweichungen zur Spec
- **`tests/layering.rs` fehlt** (G-03, SPEC §11.6). Die Schichtregeln (Anhang E der Spec) sollen hier als Rust-Test geprüft werden, nicht über ein Python-Skript. Das frühere `tools/gate_dependencies.py` wurde entfernt. `capabilities.toml` fehlt ebenfalls; vorher klären, ob die Datei gebraucht wird.
- **FaultVfs** liegt aktuell in `durable-fs`. REM-13 verschiebt es hierher.
- 1 `Instant`/`SystemTime` außerhalb `env` (G-12).
- Zweck des Crates ist Test-Verhalten. Produktionscode darf keine Abhängigkeit darauf haben (nur `[dev-dependencies]`).

## Sitzungsnotizen
<!-- Nur anhängen. Format: ### YYYY-MM-DD · Branch · Kurzfassung · Offen: … -->
