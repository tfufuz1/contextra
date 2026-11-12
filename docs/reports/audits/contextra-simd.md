# Audit- & Messreport: `contextra-simd`

- **Crate-Pfad:** `crates/contextra-simd`
- **Gesamtstatus:** **BESTANDEN (PASS)**
- **Test-Statistik:** 111 bestanden, 0 fehlgeschlagen, 0 ignoriert
- **Kompilierung (cargo check):** ✅ Erfolgreich (0.5616s)

---

## 1. Komponenten & Abhängigkeiten

- **Abhängigkeiten (5):** `thiserror`, `bytemuck`, `tracing`, `serde`, `contextra-types`
- **Features (6):** `default`, `std`, `serde`, `workspace`, `types-compat`, `simd-avx512`

---

## 2. Test-Ergebnisse im Detail

### Unit-Tests (`--lib`)
- **Status:** ✅ Bestanden
- **Laufzeit:** 10.2351s
- **Tests bestanden:** 13
- **Tests fehlgeschlagen:** 0
- **Tests ignoriert:** 0

### Integrationstests (`--tests`)
| Test-Target | Status | Bestanden | Fehlgeschlagen | Laufzeit (s) |
|---|---|---|---|---|
| `__all__` | ✅ OK | 96 | 0 | 5.6726s |

### Dokumentationstests (`--doc`)
- **Status:** ✅ Bestanden
- **Laufzeit:** 0.6759s
- **Tests bestanden:** 2

---

## 3. Performance & Benchmark-Messungen

### Gemessene Benchmark-Targets:

| Benchmark-Target | Status | Ausführungszeit (s) | Testpfade |
|---|---|---:|---:|
| `simd_bench` | ✅ OK | **1.11s** | 56 Pfade |

---|---|---|
| `simd_bench` | ⏱️ TIMEOUT | 20.0191s |

---

## 4. Aufgetretene Fehler & Compiler-Befunde

_Keine Compiler- oder Systemfehler registriert._
