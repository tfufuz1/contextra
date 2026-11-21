# CLAUDE.md — contextra-bench

> Crate-Kontext. Gilt zusätzlich zur Root-`CLAUDE.md` und zu `docs/SPEC.md` (maßgeblich bei Widerspruch).
> Liegt unter `benchmarks/`, nicht unter `crates/`. Das Verzeichnis enthält historische Dokumente (`PLAN.md`, `SPEC.md`, `AGENTS.md`, `README.md`), die nicht normativ sind.

## Rolle
Reproduzierbares Benchmark-Harness für Retrieval-Genauigkeit und Leistung. SPEC §10.4 (Messmethodik) und Anhang D (`benchmarks/budgets.toml`).

## Eckdaten (Messung 21.11.2026)
- Ebene: **E3, bench** (SPEC §3.1). Ziel laut Spec: ~3.000 Zeilen.
- Umfang: 4.164 Zeilen `src`, 3 Testdateien, 31 `#[test]`.
- Features: `default`, `datasets`, `external-benchmarks`.
- **Build: 2 Fehler** (Bin `scale-100k`). Ursache nicht geprüft. Vermutlich durch die Fassade-Abhängigkeit (`contextra-db`-API).
- Abhängigkeiten (Ist): db, rank, store, text, types, vector.

## Befehle
```bash
RUSTUP_TOOLCHAIN=stable cargo check -p contextra-bench
RUSTUP_TOOLCHAIN=stable cargo test -p contextra-bench
```

## Daten
- `benchmarks/data/fetch_beir.sh` lädt BEIR-Daten. Downloads liegen in `benchmarks/contextra-bench/tests/data/` (gitignored).
- `benchmarks/data/de_mini/` ist ein kleiner deutscher Testdatensatz im Repo.
- Der Download braucht Netzwerk. Im Sandbox können nur crates.io und GitHub erreichbar sein. Downloads daher nur über den Nutzer oder im Scratchpad.

## Ist-Abweichungen zur Spec
- **`benchmarks/budgets.toml` fehlt** (G-10, SPEC Anhang D). Werte aus Anhang D als Startwerte übernehmen, sobald der Benchmark sie misst (INV-8).
- 5 `Instant`/`SystemTime`-Aufrufe (G-12), im Bench-Kontext vermutlich beabsichtigt. Begründung im Code ergänzen.
- Zahlen aus Benchmarks nur mit Lauf und Datum belegen. Keine geschätzten Werte.

## Sitzungsnotizen
<!-- Nur anhängen. Format: ### YYYY-MM-DD · Branch · Kurzfassung · Offen: … -->
