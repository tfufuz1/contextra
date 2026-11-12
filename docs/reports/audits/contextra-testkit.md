# Audit- & Messreport: `contextra-testkit`

- **Crate-Pfad:** `crates/contextra-testkit`
- **Gesamtstatus:** **BESTANDEN (PASS)**
- **Test-Statistik:** 104 bestanden, 0 fehlgeschlagen, 2 ignoriert
- **Kompilierung (cargo check):** ✅ Erfolgreich (3.2711s)

---

## 1. Komponenten & Abhängigkeiten

- **Abhängigkeiten (5):** `contextra-ports`, `contextra-types`, `bytes`, `parking_lot`, `tracing`
- **Features (3):** `default`, `testkit`, `erasure-conformance`

---

## 2. Test-Ergebnisse im Detail

### Unit-Tests (`--lib`)
- **Status:** ✅ Bestanden
- **Laufzeit:** 7.9677s
- **Tests bestanden:** 16
- **Tests fehlgeschlagen:** 0
- **Tests ignoriert:** 0

### Integrationstests (`--tests`)
| Test-Target | Status | Bestanden | Fehlgeschlagen | Laufzeit (s) |
|---|---|---|---|---|
| `__all__` | ✅ OK | 88 | 0 | 7.2049s |

### Dokumentationstests (`--doc`)
- **Status:** ✅ Bestanden
- **Laufzeit:** 0.4646s
- **Tests bestanden:** 0

---

## 3. Performance & Benchmark-Messungen

### Gemessene Benchmark-Targets:

| Benchmark-Target | Status | Ausführungszeit (s) | Testpfade |
|---|---|---:|---:|
| `testkit_bench` | ✅ OK | **54.13s** | 2 Pfade |

---|---|---|
| `testkit_bench` | ⏱️ TIMEOUT | 20.0285s |

---

## 4. Aufgetretene Fehler & Compiler-Befunde

_Keine Compiler- oder Systemfehler registriert._
