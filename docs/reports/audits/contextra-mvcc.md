# Audit- & Messreport: `contextra-mvcc`

- **Crate-Pfad:** `crates/contextra-mvcc`
- **Gesamtstatus:** **BESTANDEN (PASS)**
- **Test-Statistik:** 180 bestanden, 0 fehlgeschlagen, 1 ignoriert
- **Kompilierung (cargo check):** ✅ Erfolgreich (0.9942s)

---

## 1. Komponenten & Abhängigkeiten

- **Abhängigkeiten (10):** `contextra-ids`, `contextra-env`, `contextra-types`, `contextra-ports`, `parking_lot`, `ahash`, `serde`, `bytes`, `thiserror`, `tracing`
- **Features (5):** `default`, `workspace`, `types-compat`, `ports-compat`, `docid-128`

---

## 2. Test-Ergebnisse im Detail

### Unit-Tests (`--lib`)
- **Status:** ✅ Bestanden
- **Laufzeit:** 9.6547s
- **Tests bestanden:** 52
- **Tests fehlgeschlagen:** 0
- **Tests ignoriert:** 0

### Integrationstests (`--tests`)
| Test-Target | Status | Bestanden | Fehlgeschlagen | Laufzeit (s) |
|---|---|---|---|---|
| `__all__` | ✅ OK | 127 | 0 | 18.8582s |

### Dokumentationstests (`--doc`)
- **Status:** ✅ Bestanden
- **Laufzeit:** 1.16s
- **Tests bestanden:** 1

---

## 3. Performance & Benchmark-Messungen

### Gemessene Benchmark-Targets:

| Benchmark-Target | Status | Ausführungszeit (s) | Testpfade |
|---|---|---:|---:|
| `mvcc_bench` | ✅ OK | **1.36s** | 11 Pfade |

---|---|---|
| `mvcc_bench` | ⏱️ TIMEOUT | 20.0237s |

---

## 4. Aufgetretene Fehler & Compiler-Befunde

_Keine Compiler- oder Systemfehler registriert._
