# Audit- & Messreport: `contextra-adapt`

- **Crate-Pfad:** `crates/contextra-adapt`
- **Gesamtstatus:** **BESTANDEN (PASS)**
- **Test-Statistik:** 120 bestanden, 0 fehlgeschlagen, 0 ignoriert
- **Kompilierung (cargo check):** ✅ Erfolgreich (0.3297s)

---

## 1. Komponenten & Abhängigkeiten

- **Abhängigkeiten (8):** `ahash`, `blake3`, `contextra-ports`, `contextra-types`, `serde`, `serde_json`, `thiserror`, `tracing`
- **Features (11):** `default`, `std`, `workspace`, `types-compat`, `ports-compat`, `bandit-routing`, `exp-bandit-routing`, `flow-corrected-thompson`, `exp-flow-thompson`, `rie-greedy-personalization`, `exp-rie-greedy`

---

## 2. Test-Ergebnisse im Detail

### Unit-Tests (`--lib`)
- **Status:** ✅ Bestanden
- **Laufzeit:** 0.4123s
- **Tests bestanden:** 52
- **Tests fehlgeschlagen:** 0
- **Tests ignoriert:** 0

### Integrationstests (`--tests`)
| Test-Target | Status | Bestanden | Fehlgeschlagen | Laufzeit (s) |
|---|---|---|---|---|
| `__all__` | ✅ OK | 68 | 0 | 0.4663s |

### Dokumentationstests (`--doc`)
- **Status:** ✅ Bestanden
- **Laufzeit:** 0.5238s
- **Tests bestanden:** 0

---

## 3. Performance & Benchmark-Messungen

### Gemessene Benchmark-Targets:

| Benchmark-Target | Status | Ausführungszeit (s) | Testpfade |
|---|---|---:|---:|
| `adapt_bench` | ✅ OK | **0.83s** | 1 Pfade |

---|---|---|
| `adapt_bench` | ⏱️ TIMEOUT | 20.0262s |

---

## 4. Aufgetretene Fehler & Compiler-Befunde

_Keine Compiler- oder Systemfehler registriert._
