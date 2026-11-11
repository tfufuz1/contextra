# Baseline-Bericht: `contextra-vector` (S-36)

**Stand:** 2026-11-11
**Commit:** `f92caabaa`
**Toolchain:** `rustc 1.89.0`, `cargo 1.89.0`

---

## 1. Werkzeugläufe & Metriken

- **Modularitäts-Score (`decouple_inspector.py`):** 29/100 (Stark verdrahtet)
- **LOC:** 13.607 LOC in 55 Quelltextdateien
- **Bridge Refs (`count_bridge_refs.py`):** 160 Referenzen auf `types`/`ports` im Quelltext außerhalb `compat/` (ca. 162 Fundstellen mit grep)
- **Zero-Panic (`zero_panic_analyzer.py`):** 0 echte unzulässige Panics im Kern (`#![forbid(unsafe_code)]` aktiv)
- **Determinismus (`determinism_guard.py`):** 6 Fundstellen in Nicht-Test-Code (`DiskAnnHmac`, RaBitQ, HashMaps ohne deterministischen Hasher)
- **G-L1 Layering (`gate_layering.py`):** GRÜN (Kern-LOC 17.312, direkte Abhängigkeiten leer)
- **Abhängigkeitsbudget (`abhaengigkeits_budget.py`):** GRÜN

---

## 2. Baseline-Build- und Testbefunde

- `cargo check -p contextra-vector --locked`: **GRÜN** (mit 15 Deprecation-Warnungen bezüglich `contextra_types::HnswConnectivityDegraded` und `GraphRepairFailed`)
- `cargo check -p contextra-vector --no-default-features --locked`: **ROT**
  - Befund: Hypothese B18 bestätigt. `--no-default-features` scheitert mit 99 Fehlern, da der Kern direkt `contextra_ports` und `contextra_types` importiert.
- `cargo test -p contextra-vector --lib --locked`: **GRÜN** (alle 64 Unittests bestanden; `test_hnsw_rebuild_exponential_backoff_and_alarm` einzeln und zusammen bestanden).

---

## 3. V-VETO Wachtest-Verifikation

1. `tests/partial_rebuild_veto_f02_guard.rs`:
   - `test_partial_rebuild_feature_gate_guard`: **GRÜN** (sichert ab, dass `partial-index-rebuild` nicht im Default aktiv ist).
   - `test_veto_f02_guard_partial_rebuild_tombstone_pruning`: **GRÜN** (sichert INV-VEC-ZEROIZE-1 und INV-VEC-CONNECTIVITY-1 ab).
2. `tests/partial_rebuild_recall_regression.rs`:
   - Pre-existing Bug: `rebuild_region()` ist eine synchrone Methode, wurde im Test jedoch mit `.await` aufgerufen (`E0277`). Nach minimaler Reparatur laufen beide Tests:
     - `test_partial_rebuild_recall_regression`: **GRÜN** (Recall-Delta innerhalb des Toleranzbandes).
     - `test_partial_rebuild_node_degree_no_isolation`: **GRÜN** (Knotenisolierung ausgeschlossen).
