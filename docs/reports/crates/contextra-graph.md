# Audit- & Messreport: `contextra-graph`

- **Crate-Pfad:** `crates/contextra-graph`
- **Gesamtstatus:** **FEHLGESCHLAGEN (FAIL)**
- **Test-Statistik:** 0 bestanden, 0 fehlgeschlagen, 0 ignoriert
- **Kompilierung (cargo check):** ❌ Fehlgeschlagen (3.3751s)

---

## 1. Komponenten & Abhängigkeiten

- **Abhängigkeiten (15):** `contextra-ids`, `contextra-env`, `contextra-types`, `contextra-ports`, `bytes`, `parking_lot`, `tracing`, `ahash`, `serde`, `serde_json`, `bincode`, `blake3`, `arc-swap`, `scc`, `thiserror`
- **Features (16):** `default`, `workspace`, `session-dag`, `entity-extraction`, `hyperedge-suggest`, `apprh-diffusion`, `k-path-diffusion`, `graph-connectivity-health`, `edge-reinforcement-learning`, `docid-128`, `test-support`, `types-compat`, `ports-compat`, `experimental-graph`, `physio-percolation`, `ppr-forward-push`

---

## 2. Test-Ergebnisse im Detail

### Unit-Tests (`--lib`)
- **Status:** ❌ Nicht bestanden / Fehler
- **Laufzeit:** 0s
- **Tests bestanden:** 0
- **Tests fehlgeschlagen:** 0
- **Tests ignoriert:** 0

### Integrationstests (`--tests`)
_Keine separaten Integrationstest-Targets deklariert._

### Dokumentationstests (`--doc`)
- **Status:** ❌ Fehler / Keine
- **Laufzeit:** 0s
- **Tests bestanden:** 0

---

## 3. Performance & Benchmark-Messungen

_Keine expliziten Criterion-Benchmarks in diesem Crate vorhanden._

---

## 4. Aufgetretene Fehler & Compiler-Befunde

### Compiler- & Laufzeitfehler:
```text
error[E0428]: the name `types` is defined multiple times
error: could not compile `contextra-graph` (lib) due to 1 previous error; 4 warnings emitted
```
