# Audit- & Messreport: `contextra-mcp`

- **Crate-Pfad:** `crates/contextra-mcp`
- **Gesamtstatus:** **FEHLGESCHLAGEN (FAIL)**
- **Test-Statistik:** 0 bestanden, 0 fehlgeschlagen, 0 ignoriert
- **Kompilierung (cargo check):** ❌ Fehlgeschlagen (7.1144s)

---

## 1. Komponenten & Abhängigkeiten

- **Abhängigkeiten (25):** `contextra`, `contextra-types`, `contextra-ports`, `contextra-wire`, `contextra-rank`, `contextra-crypto`, `contextra-privacy`, `contextra-prompt-guard`, `contextra-router`, `contextra-infer-candle`, `contextra-infer-ollama`, `contextra-infer-onnx`, `zeroize`, `rand`, `parking_lot`, `scc`, `tokio`, `serde`, `serde_json`, `thiserror`, `tracing`, `tracing-subscriber`, `unicode-normalization`, `tokio-util`, `futures-util`
- **Features (5):** `default`, `candle`, `ollama`, `onnx`, `test-utils`

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
71 | use crate::error::{StoreError, StoreResult as Result};
warning: unused import: `crate::error::StoreResult as Result`
13 | use crate::error::StoreResult as Result;
warning: unused import: `crate::error::StoreResult as Result`
14 | use crate::error::StoreResult as Result;
error: could not compile `contextra-graph` (lib) due to 1 previous error; 4 warnings emitted
```
