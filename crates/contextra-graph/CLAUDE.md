# CLAUDE.md — contextra-graph

> Crate-Kontext. Gilt zusätzlich zur Root-`CLAUDE.md` und zu `docs/SPEC.md` (maßgeblich bei Widerspruch).

## Rolle
Optionaler CSR-Graph für Entitäten-Beziehungen, Traversierung, PathRAG, Seal-Provenienz. **Optional** in der Zielarchitektur (ADR-21 bis ADR-27, SPEC §6.10). Die Reduktion (§12.4) passiert nur, wenn Gate G-GRAPH grün ist.

## Eckdaten (Messung 21.11.2026)
- Ebene: **E1, optional** (SPEC §3.1). Ziel laut Spec: ~6.000 Zeilen.
- Umfang: **23.607 Zeilen `src`** (Anhang B: 34.611 inkl. Tests), 63 Testdateien, 224 `#[test]`.
- Features (Default: `session-dag`, `entity-extraction`, `hyperedge-suggest`, `experimental-graph`): `default`, `workspace`, `session-dag`, `entity-extraction`, `hyperedge-suggest`, `apprh-diffusion`, `k-path-diffusion`, `graph-connectivity-health`, `edge-reinforcement-learning`, `docid-128`, `test-support`, `types-compat`, `ports-compat`, `experimental-graph`, `physio-percolation`, `ppr-forward-push`.
- Build: ok.
- Abhängigkeiten (Ist): env, ids, ports, types.
- Genutzt von: `contextra-db`.

## Befehle
```bash
RUSTUP_TOOLCHAIN=stable cargo test -p contextra-graph
RUSTUP_TOOLCHAIN=stable cargo test -p contextra-graph --no-default-features   # G-01, G-06
RUSTUP_TOOLCHAIN=stable cargo clippy -p contextra-graph -- -D warnings
```

## Ist-Abweichungen zur Spec
- **Feature-Ballast:** 16 Features, Ziel ist deutlich weniger. `experimental-graph`, `physio-percolation`, `ppr-forward-push` usw. prüfen.
- **R3/G-06 (`default = []`):** Das Default-Set ist nicht leer. Begründung nötig oder Entfernung.
- **R6 stark verletzt (grob):** 741 `unwrap`/`expect`/`panic`-Treffer inkl. Tests. Produktcode getrennt zählen.
- **G-12:** 6 `Instant`/`SystemTime`-Aufrufe außerhalb `contextra-env`.
- **`compat/`** vorhanden (G-11). `types-compat`, `ports-compat` (R7).
- **Selbstdeklarationen:** 124 Dateien mit `ANCHOR`/`FILE-CONTEXT` (REM-14).
- **`unsafe`:** nur in einer Testdatei (`ppr_alloc_test.rs`). Entfällt laut Spec mit §12.4.
- **Entfernung nicht ohne Gate-Grün.** Vorher G-GRAPH-Bedingungen in SPEC §12.4 prüfen.

## Sitzungsnotizen
<!-- Nur anhängen. Format: ### YYYY-MM-DD · Branch · Kurzfassung · Offen: … -->
