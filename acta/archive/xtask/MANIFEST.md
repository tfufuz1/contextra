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
