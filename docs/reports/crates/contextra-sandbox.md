# Audit- & Messreport: `contextra-sandbox`

- **Crate-Pfad:** `crates/contextra-sandbox`
- **Gesamtstatus:** **BESTANDEN (PASS)**
- **Test-Statistik:** 22 bestanden, 0 fehlgeschlagen, 0 ignoriert
- **Kompilierung (cargo check):** ✅ Erfolgreich (0.5059s)

---

## 1. Komponenten & Abhängigkeiten

- **Abhängigkeiten (6):** `wasmtime`, `zeroize`, `anyhow`, `thiserror`, `tokio`, `tracing`
- **Features (2):** `default`, `wasm`

---

## 2. Test-Ergebnisse im Detail

### Unit-Tests (`--lib`)
- **Status:** ✅ Bestanden
- **Laufzeit:** 6.5967s
- **Tests bestanden:** 11
- **Tests fehlgeschlagen:** 0
- **Tests ignoriert:** 0

### Integrationstests (`--tests`)
| Test-Target | Status | Bestanden | Fehlgeschlagen | Laufzeit (s) |
|---|---|---|---|---|
| `__all__` | ✅ OK | 11 | 0 | 0.3457s |

### Dokumentationstests (`--doc`)
- **Status:** ✅ Bestanden
- **Laufzeit:** 0.7572s
- **Tests bestanden:** 0

---

## 3. Performance & Benchmark-Messungen

### Gemessene Benchmark-Targets:

| Benchmark-Target | Status | Ausführungszeit (s) | Testpfade |
|---|---|---:|---:|
| `sandbox_bench` | ✅ OK | **0.82s** | 3 Pfade |

---|---|---|
| `sandbox_bench` | ⏱️ TIMEOUT | 20.0175s |

---

## 4. Aufgetretene Fehler & Compiler-Befunde

_Keine Compiler- oder Systemfehler registriert._
