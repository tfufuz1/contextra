# P12 Offene Punkte & Fremdfehler

## Fremdfehler (außerhalb BESITZ)

1. **`contextra-graph` Kompilierungsfehler:**
   - Datei: `crates/contextra-graph/src/lib.rs:37, 42`
   - Fehler: `E0428: the name contract is defined multiple times`
   - Sowie fehlende Imports / `E0433` (unresolved `contextra_types`, `contextra_ports`) in `csr/graph_index/`, `csr/graph_write/`, etc.
   - Status: Fremdfehler, gemeldet für den Integrator.

2. **`contextra-vector` Kompilierungsfehler:**
   - Datei: `crates/contextra-vector/src/hnsw/deletion.rs:310`, `types.rs:597`, etc.
   - Fehler: `E0592: duplicate definitions with name remove / rebuild_sync`, `E0599: no method named zeroize`, `E0609: no field kernel on type HnswColdCore`, `E0599: try_new_with_rng_and_kernel`.
   - Status: Fremdfehler, gemeldet für den Integrator.

3. **`contextra-crypto` Kompilierungsfehler:**
   - Datei: `crates/contextra-crypto/src/secret.rs:94, 105`
   - Fehler: unexpected `cfg` condition value `serde`.
   - Status: Fremdfehler, gemeldet für den Integrator.

## Workspace- und CI-Fremdfehler (Welle 1 / HEAD 65e1d68a5)
- **CI: xtask nicht im Workspace**: Workflows (`check-compile`, `shell-commit-audit`, `check-ring-layering`, `check-marker-drift`, `check-fbs-drift`, `check-duplicate-symbols-cross-file`) schlagen fehl, da `xtask/Cargo.toml` nicht mehr in `workspace.members` im Wurzel-`Cargo.toml` vorhanden ist (`package(s) xtask not found in workspace`). Wurzel-`Cargo.toml` und `.github/` liegen außerhalb BESITZ.
- **CI: Cargo.lock --locked Desynchronisation**: Benchmarks und CI-Jobs mit `--locked` schlagen fehl mit `the lock file Cargo.lock needs to be updated but --locked was passed`. Wurzel-`Cargo.lock` darf laut globaler Regel 2 nicht modifiziert werden.
- **CI: RustSec Security Advisories**: `cargo deny` schlägt fehl aufgrund von Sicherheitslücken in `wasmtime` (RUSTSEC-2026-0424) und `zip` (RUSTSEC-2026-0367, RUSTSEC-2026-0368, RUSTSEC-2026-0369, RUSTSEC-2026-0429), die von fremden Crates (`contextra-sandbox`, `contextra-ingest`, `contextra-infer-candle`) transitiv bezogen werden.
- **CI: Kompilierungsfehler in Fremd-Crates**:
  - `contextra-crypto`: E0609 (`store.rs`), E0308 (`segment.rs`)
  - `contextra-graph`: E0433 (`use of unresolved module contextra_ports / contextra_types`)
  - `contextra-vector`: E0592 (`remove`, `rebuild_sync`), E0599 (`zeroize`), E0609 (`kernel`)
  - `contextra-text`: Clippy-Lints (P28 `HashMap`/`HashSet`, indexing/slicing, unnecessary_cast)
- **CI: Production Unwrap Baseline Überschreitung**:
  - Baseline in `.github/unwrap_baseline.txt` steht auf 1197, der Workspace-Gesamtwert in fremden Crates stieg auf über 1300. `crates/contextra/src` enthält 0 `unwrap()` und 0 `expect()`.
