# Audit- & Messreport: `contextra-infer-onnx`

- **Crate-Pfad:** `crates/contextra-infer-onnx`
- **Gesamtstatus:** **BESTANDEN (PASS)**
- **Test-Statistik:** 6 bestanden, 0 fehlgeschlagen, 0 ignoriert
- **Kompilierung (cargo check):** ✅ Erfolgreich (0.5157s)

---

## 1. Komponenten & Abhängigkeiten

- **Abhängigkeiten (13):** `contextra-ports`, `contextra-types`, `thiserror`, `parking_lot`, `blake3`, `tracing`, `serde`, `serde_json`, `sha2`, `reqwest`, `ort`, `tokenizers`, `ndarray`
- **Features (9):** `default`, `std`, `serde`, `tracing`, `download`, `reranking`, `onnx`, `test-utils`, `workspace`

---

## 2. Test-Ergebnisse im Detail

### Unit-Tests (`--lib`)
- **Status:** ✅ Bestanden
- **Laufzeit:** 0.3726s
- **Tests bestanden:** 2
- **Tests fehlgeschlagen:** 0
- **Tests ignoriert:** 0

### Integrationstests (`--tests`)
| Test-Target | Status | Bestanden | Fehlgeschlagen | Laufzeit (s) |
|---|---|---|---|---|
| `__all__` | ✅ OK | 4 | 0 | 0.3685s |

### Dokumentationstests (`--doc`)
- **Status:** ✅ Bestanden
- **Laufzeit:** 0.4307s
- **Tests bestanden:** 0

---

## 3. Performance & Benchmark-Messungen

| Benchmark-Target | Status | Ausführungszeit (s) |
|---|---|---|
| `infer_bench` | ❌ FAIL | 0.3349s |

---

## 4. Aufgetretene Fehler & Compiler-Befunde

_Keine Compiler- oder Systemfehler registriert._
