# Audit- & Messreport: `contextra-rank`

- **Crate-Pfad:** `crates/contextra-rank`
- **Gesamtstatus:** **BESTANDEN (PASS)**
- **Test-Statistik:** 144 bestanden, 0 fehlgeschlagen, 1 ignoriert
- **Kompilierung (cargo check):** ✅ Erfolgreich (2.9848s)

---

## 1. Komponenten & Abhängigkeiten

- **Abhängigkeiten (7):** `contextra-types`, `contextra-ports`, `serde`, `serde_json`, `ahash`, `tracing`, `thiserror`
- **Features (10):** `default`, `std`, `workspace`, `types-compat`, `ports-compat`, `dibud`, `exp-dibud`, `exp-resonance`, `experimental-all`, `experimental-rank`

---

## 2. Test-Ergebnisse im Detail

### Unit-Tests (`--lib`)
- **Status:** ✅ Bestanden
- **Laufzeit:** 9.076s
- **Tests bestanden:** 21
- **Tests fehlgeschlagen:** 0
- **Tests ignoriert:** 0

### Integrationstests (`--tests`)
| Test-Target | Status | Bestanden | Fehlgeschlagen | Laufzeit (s) |
|---|---|---|---|---|
| `__all__` | ✅ OK | 122 | 0 | 12.215s |

### Dokumentationstests (`--doc`)
- **Status:** ✅ Bestanden
- **Laufzeit:** 1.2767s
- **Tests bestanden:** 1

---

## 3. Performance & Benchmark-Messungen

### Gemessene Benchmark-Targets:

| Benchmark-Target | Status | Ausführungszeit (s) | Testpfade |
|---|---|---:|---:|
| `rank_bench` | ✅ OK | **1.48s** | 15 Pfade |

---|---|---|
| `rank_bench` | ⏱️ TIMEOUT | 20.0239s |

---

## 4. Aufgetretene Fehler & Compiler-Befunde

_Keine Compiler- oder Systemfehler registriert._
