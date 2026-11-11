# Audit- & Messreport: `contextra-db`

- **Crate-Pfad:** `crates/contextra-db`
- **Gesamtstatus:** **FEHLGESCHLAGEN (FAIL)**
- **Test-Statistik:** 0 bestanden, 0 fehlgeschlagen, 0 ignoriert
- **Kompilierung (cargo check):** ❌ Fehlgeschlagen (4.3743s)

---

## 1. Komponenten & Abhängigkeiten

- **Abhängigkeiten (16):** `contextra-vault`, `contextra-cognition`, `contextra-engine`, `contextra-types`, `contextra-ports`, `bytes`, `blake3`, `ahash`, `serde_json`, `serde`, `tokio`, `tracing`, `parking_lot`, `tokio-util`, `zeroize`, `thiserror`
- **Features (13):** `cognition`, `default`, `bench`, `sandbox`, `reranking`, `onnx`, `experimental-diskann`, `background-maintenance`, `graph-connectivity-health`, `coherence-bonus-fusion`, `adaptive-candidate-pool-sizing`, `volatile-vault`, `edge-reinforcement-learning`

---

## 2. Test-Ergebnisse im Detail

### Unit-Tests (`--lib`)
- **Status:** ❌ Nicht bestanden / Fehler
- **Laufzeit:** 4.7164s
- **Tests bestanden:** 0
- **Tests fehlgeschlagen:** 0
- **Tests ignoriert:** 0

### Integrationstests (`--tests`)
| Test-Target | Status | Bestanden | Fehlgeschlagen | Laufzeit (s) |
|---|---|---|---|---|
| `__all__` | ❌ FAIL | 0 | 0 | 6.9829s |

### Dokumentationstests (`--doc`)
- **Status:** ❌ Fehler / Keine
- **Laufzeit:** 4.281s
- **Tests bestanden:** 0

---

## 3. Performance & Benchmark-Messungen

_Keine expliziten Criterion-Benchmarks in diesem Crate vorhanden._

---

## 4. Aufgetretene Fehler & Compiler-Befunde

### Compiler- & Laufzeitfehler:
```text
71 | use crate::error::{StoreError, StoreResult as Result};
warning: unused import: `crate::error::StoreResult as Result`
13 | use crate::error::StoreResult as Result;
warning: unused import: `crate::error::StoreResult as Result`
14 | use crate::error::StoreResult as Result;
error[E0428]: the name `types` is defined multiple times
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
