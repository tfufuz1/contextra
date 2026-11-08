# Contextra Baukasten – Baseline Statusbericht (WP-0)

> **Datum:** 2026-11-08
> **Basis-Commit:** `88fdcc7ade05a53e42944632cc3953f65094dc0c`
> **Toolchain:** Rust 1.89.0
> **Normative Referenz:** `docs/SPEC.md` v4.0 (§1 Ausgangslage, §8.1 Gates, §8.3 Kennzahlen, §12 WP-0)

## 1. Baseline Messung Workspace Check

Befehl:
```bash
cargo check --workspace --exclude contextra-bench --exclude contextra-py 2>&1 | tail
```

Messbefund (SPEC v4.0 §1 Ausgangslage):
- Status: Kompilierung im Ausgangsstand grün
- Gesamtzahl Warnungen: 68
- Davon `use of deprecated`: 48
- Primäre Ursache der Deprecated-Warnungen: Übergangsweise re-exportierte Erasure-Typen (`ErasureId`, `GateId`, `SagaRecord`, `ScanMethod`, `HnswDeletionError`) in `contextra-types` und `contextra-ports` vor ihrer vollständigen Bündelung in `contextra-erasure` / `contextra-erasure-adapters`.

## 2. Gate G1 Isolations- & Kantenstatus

Befehl:
```bash
python3 tools/gate_g1_layering.py
```

Ergebnis:
- 61 Kantenverstöße gegen den Baukasten-Zielzustand (SPEC v4.0 §5.1 / §8.1)
- Klasse A: 18 Bausteine haben noch interne Abhängigkeiten auf `contextra-types`, `contextra-ports`, `contextra-sys` oder `contextra-testkit`.
- Klasse B (Storage-Bundle): 5 Crates (`types`, `ports`, `mvcc`, `store`, `checkpoint`)
- Klasse C (Anwendungsschicht): 8 Crates (`contextra`, `db`, `engine`, `cognition`, `agent`, `mcp`, `py`, `router`)
- Klasse T: `contextra-testkit`

## 3. Funktionsinventar (E-06)

Befehl:
```bash
python3 tools/generate_function_inventory.py --head --out-csv docs/generated/funktionsinventar.csv --out-md docs/generated/funktionsinventar.md
```

Ergebnis:
- Gesamtanzahl Funktionen im Workspace: 6.623
- Öffentlich (`pub`): 3.809
- Crate-intern (`pub(crate)`): 276
- Privat: 2.538
- Async: 710 | Unsafe: 43 | Dokumentiert: 4.023
