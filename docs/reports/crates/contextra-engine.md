# Audit- & Messreport: `contextra-engine`

- **Crate-Pfad:** `crates/contextra-engine`
- **Gesamtstatus:** **FEHLGESCHLAGEN (FAIL)**
- **Test-Statistik:** 0 bestanden, 0 fehlgeschlagen, 0 ignoriert
- **Kompilierung (cargo check):** ❌ Fehlgeschlagen (6.7239s)

---

## 1. Komponenten & Abhängigkeiten

- **Abhängigkeiten (20):** `contextra-types`, `contextra-ports`, `contextra-mvcc`, `bytes`, `contextra-sandbox`, `contextra-infer-onnx`, `contextra-adapt`, `contextra-store`, `contextra-vector`, `contextra-text`, `contextra-checkpoint`, `contextra-rank`, `contextra-graph`, `ahash`, `serde_json`, `serde`, `tokio`, `tracing`, `parking_lot`, `tokio-util`
- **Features (12):** `default`, `onnx`, `reranking`, `sandbox`, `experimental-diskann`, `background-maintenance`, `graph-connectivity-health`, `edge-reinforcement-learning`, `adaptive-candidate-pool-sizing`, `exp-extraction`, `entity-extraction`, `auto-extraction-opt-out`

---

## 2. Test-Ergebnisse im Detail

### Unit-Tests (`--lib`)
- **Status:** ❌ Nicht bestanden / Fehler
- **Laufzeit:** 4.0208s
- **Tests bestanden:** 0
- **Tests fehlgeschlagen:** 0
- **Tests ignoriert:** 0

### Integrationstests (`--tests`)
| Test-Target | Status | Bestanden | Fehlgeschlagen | Laufzeit (s) |
|---|---|---|---|---|
| `__all__` | ❌ FAIL | 0 | 0 | 4.8382s |

### Dokumentationstests (`--doc`)
- **Status:** ❌ Fehler / Keine
- **Laufzeit:** 3.5429s
- **Tests bestanden:** 0

---

## 3. Performance & Benchmark-Messungen

_Keine expliziten Criterion-Benchmarks in diesem Crate vorhanden._

---

## 4. Aufgetretene Fehler & Compiler-Befunde

### Compiler- & Laufzeitfehler:
```text
error[E0428]: the name `types` is defined multiple times
71 | use crate::error::{StoreError, StoreResult as Result};
warning: unused import: `crate::error::StoreResult as Result`
13 | use crate::error::StoreResult as Result;
warning: unused import: `crate::error::StoreResult as Result`
14 | use crate::error::StoreResult as Result;
error: could not compile `contextra-graph` (lib) due to 1 previous error; 4 warnings emitted
error[E0428]: the name `types` is defined multiple times
error[E0433]: failed to resolve: use of undeclared type `VectorError`
7  + use crate::error::VectorError;
error[E0433]: failed to resolve: use of undeclared type `VectorError`
7  + use crate::error::VectorError;
71 | use crate::error::{StoreError, StoreResult as Result};
warning: unused import: `crate::error::StoreResult as Result`
13 | use crate::error::StoreResult as Result;
warning: unused import: `crate::error::StoreResult as Result`
14 | use crate::error::StoreResult as Result;
error[E0308]: mismatched types
error[E0308]: mismatched types
error[E0599]: no method named `search` found for struct `Arc<GenericHnswIndex>` in the current scope
error[E0599]: no method named `search` found for struct `Arc<GenericHnswIndex>` in the current scope
error[E0599]: no method named `search` found for struct `Arc<GenericHnswIndex>` in the current scope
error[E0631]: type mismatch in function arguments
error[E0631]: type mismatch in function arguments
error[E0631]: type mismatch in function arguments
error[E0631]: type mismatch in function arguments
error[E0631]: type mismatch in function arguments
error[E0631]: type mismatch in function arguments
error[E0631]: type mismatch in function arguments
error[E0631]: type mismatch in function arguments
```
