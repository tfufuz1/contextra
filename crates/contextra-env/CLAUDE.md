# CLAUDE.md — contextra-env

> Crate-Kontext. Gilt zusätzlich zur Root-`CLAUDE.md` und zu `docs/SPEC.md` (maßgeblich bei Widerspruch).

## Rolle
Injizierbare Umgebung: `Clock`, `SeededRng`, `MetricsSink`, ID-Erzeugung. **Einziger Ort**, an dem `Instant::now` und `SystemTime::now` aufgerufen werden dürfen (R10, G-12).

## Eckdaten (Messung 21.11.2026)
- Ebene: **E0** (SPEC §3.1, R10). Abhängigkeit: `contextra-ids`.
- Umfang: 2.437 Zeilen `src`, 3 Testdateien, 54 `#[test]`.
- Features: `default`, `std`, `test-utils`.
- Build: ok.
- Genutzt von: checkpoint, crypto, durable-fs-Nutzer, erasure, graph, mvcc, privacy, ports, store, text, vector, db.

## Befehle
```bash
RUSTUP_TOOLCHAIN=stable cargo test -p contextra-env
RUSTUP_TOOLCHAIN=stable cargo clippy -p contextra-env -- -D warnings
```

## Hinweise
- Zeitquellen in anderen Crates sind Verstöße: aktuell **77** Aufrufe außerhalb von `contextra-env` (G-12 Soll 0). Ersatz über `Clock` aus diesem Crate.
- Zufall: `OsRng` für Schlüssel, Nonces und IDs; `SeededRng` nur in Tests und Benchmarks.
- Breaking Changes an `Clock` oder `SeededRng` betreffen viele Crates. In der Notiz des Dependents eintragen.
- 11 `unwrap`/`expect`-Stellen im Bestand (grob, inkl. Tests), R6 beachten.

## Sitzungsnotizen
<!-- Nur anhängen. Format: ### YYYY-MM-DD · Branch · Kurzfassung · Offen: … -->
