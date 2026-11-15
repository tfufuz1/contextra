# Extraktion fuer `xtask`

Python-Werkzeuge der Altquelle als **Vorlage fuer die noch fehlenden Gates** aus Kap. 16.6.
`cargo xtask gates` (Rust, `xtask/`) setzt bereits um: G1, G-DEF, G-HUB, G-UNSAFE, G-NOTOKIO, G-LINT, G-WORD.

| Werkzeug | Vorlage fuer |
|---|---|
| `gate_layering.py`, `gate_g1_layering.py`, `gate_dependencies.py`, `layering.rs` | G1 (negative Fixtures), G2 |
| `determinism_guard.py` | G3 (keine Zeit-/Zufallsquelle ausserhalb der Ports) |
| `async_boundary_scanner.py` | G-NOTOKIO (Vertiefung) |
| `unsafe_boundary_auditor.py`, `unsafe_islands.rs` | G-UNSAFE (Vertiefung) |
| `zero_panic_analyzer.py` | G-LINT (Vertiefung) |
| `abhaengigkeits_budget.py`, `loc.py`, `crate_inventory.py`, `modularity_score.py` | G-HUB |
| `error_topology_mapper.py`, `port_contract_auditor.py` | HR-4, G2 |
| `concurrency_lock_auditor.py` | Sperrreihenfolge 3.6 |
| `fsync_durability_tracer.py` | G-FS, WP-7 |
| `erasure_leak_prober.py` | G-ERA, T-EVID |
| `doc_lint.py` | G-WORD (Vertiefung) |

Die Skripte sind bereinigt, aber nicht angepasst; Pfade und Crate-Namen vor Gebrauch auf `crates/acta-*` abgleichen.
## Dateien

| Datei (relativ zu `archive/xtask/`) | LOC | Befunde |
|---|---:|---|
| `tools/abhaengigkeits_budget.py` | 142 |  |
| `tools/async_boundary_scanner.py` | 181 |  |
| `tools/concurrency_lock_auditor.py` | 226 |  |
| `tools/crate_inventory.py` | 603 |  |
| `tools/determinism_guard.py` | 190 |  |
| `tools/doc_lint.py` | 134 |  |
| `tools/erasure_leak_prober.py` | 141 |  |
| `tools/error_topology_mapper.py` | 134 |  |
| `tools/fsync_durability_tracer.py` | 173 |  |
| `tools/gate_dependencies.py` | 167 |  |
| `tools/gate_g1_layering.py` | 14 |  |
| `tools/gate_layering.py` | 197 |  |
| `tools/layering.rs` | 60 | panic=2 |
| `tools/loc.py` | 159 |  |
| `tools/modularity_score.py` | 216 |  |
| `tools/port_contract_auditor.py` | 186 |  |
| `tools/unsafe_boundary_auditor.py` | 188 |  |
| `tools/unsafe_islands.rs` | 127 | std_fs=1, unsafe=6 |
| `tools/zero_panic_analyzer.py` | 223 |  |
