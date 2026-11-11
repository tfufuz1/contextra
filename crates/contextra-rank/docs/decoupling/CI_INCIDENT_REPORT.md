# CI Incident Analysis & Handoff (Production Unwrap & Foreign Crate Gate Failures)

## 1. Analyse der gemeldeten CI-Fehlschläge
In Pull Request #6026 meldeten GitHub Actions Failures in folgenden Jobs:
1. `Production Unwrap Counter Gate`:
   - Befehl: `COUNT=$(grep -r '\.unwrap()\|\.expect(' crates/ --include="*.rs" | grep -v '/tests' | grep -v '/test' | grep -v '/bench' | grep -v '#\[cfg(test)\]' | wc -l)`
   - Baseline: `.github/unwrap_baseline.txt` = `1197`.
   - Gemessener Count: `1484`.
   - Ursache: Im Workspace existieren am aktuellen Stand (`origin/main`) ungelöste Unwraps in fremden Crates (u. a. `contextra-vector`, `contextra-store`, `contextra-engine`). In `contextra-rank` selbst wurden im Quelltext 0 unwrap/expect hinzugefügt; im Gegenteil wurde ein `unwrap()` in `src/compat/types.rs` entfernt. Da `.github/unwrap_baseline.txt` außerhalb des Eigentums von `contextra-rank` liegt, darf der Zähler nicht eigenmächtig angepasst werden (DISJUNKTHEITS-REGEL).
2. `Format Check`:
   - `cargo fmt --check` schlägt in `crates/contextra-vector/` fehl. `contextra-rank` ist format-sauber.
3. `Cargo Deny Check`:
   - RustSec Advisories für `lru`, `number_prefix`, `tokio`, `zip`, `wasmtime` in fremden Crates.
4. `Gate G-L1 Layering Enforcement`:
   - `tests/test_gate_layering_negative.py` lief auf `origin/main` fehl, ist aber lokal mit Exit 1 (erwartet) grün.
5. `DocId 128-bit Validation Gate`:
   - Typfehler in `contextra-store/src/tenant_codec.rs:110` (`u64` vs `u128`).
6. `Maturin / PyPI / Cross-Platform`:
   - Kompilierfehler in `contextra-engine/src/collection/mod.rs:662` (`TextError` vs `ContextraError`).

## 2. Status contextra-rank
- Alle Gates G-B1 bis G-B14 für `contextra-rank` sind grün.
- Standalone-Export, No-Default-Build und Unit-/Integrationstests laufen fehlerfrei durch.
