# CLAUDE.md — contextra-rank

> Crate-Kontext. Gilt zusätzlich zur Root-`CLAUDE.md` und zu `docs/SPEC.md` (maßgeblich bei Widerspruch).

## Rolle
Ergebnis-Fusion und Ranking: RRF (Reciprocal Rank Fusion), Top-k, Zeitgewichtung, Erklärung, Isotonic- und Platt-Kalibrierung (Wahrscheinlichkeiten sind kalibriert, README §1), Drift-Erkennung. SPEC §6.6, REM-18.

## Eckdaten (Messung 21.11.2026)
- Ebene: **E1** (SPEC §3.1). Ziel laut Spec: ~4.000 Zeilen.
- Umfang: 5.851 Zeilen `src`, 25 Testdateien, 136 `#[test]`.
- Features: `default`, `std`, `workspace`, `types-compat`, `ports-compat`, `dibud`, `exp-dibud`, `exp-resonance`, `experimental-all`, `experimental-rank`.
- Build: ok.
- Abhängigkeiten (Ist): ports, types.
- Genutzt von: `contextra-db`, `contextra-mcp`, `contextra-bench`.

## Befehle
```bash
RUSTUP_TOOLCHAIN=stable cargo test -p contextra-rank
RUSTUP_TOOLCHAIN=stable cargo test -p contextra-rank --no-default-features   # G-01
RUSTUP_TOOLCHAIN=stable cargo clippy -p contextra-rank -- -D warnings
```

## Ist-Abweichungen zur Spec
- **R7 verletzt:** `types-compat`, `ports-compat`, `workspace`; `src/compat/` (G-11).
- **Experimental-Features** (`dibud`, `exp-dibud`, `exp-resonance`, `experimental-all`, `experimental-rank`) sind Ballast. Entscheiden, ob sie bleiben.
- **G-12:** 2 `Instant`/`SystemTime`-Aufrufe außerhalb `contextra-env`.
- Calibrierung: Kalibrierungsaussagen („80 % relevant entspricht ungefähr 80 %“) brauchen Tests als Beleg. Änderungen daran nur mit Messung.
- `contract.rs` vorhanden (REM-14).

## Sitzungsnotizen
<!-- Nur anhängen. Format: ### YYYY-MM-DD · Branch · Kurzfassung · Offen: … -->
