# Audit- & Messreport: `contextra-durable-fs`

- **Crate-Pfad:** `crates/contextra-durable-fs`
- **Gesamtstatus:** **BESTANDEN (PASS)**
- **Test-Statistik:** 43 bestanden, 0 fehlgeschlagen, 0 ignoriert
- **Kompilierung (cargo check):** ✅ Erfolgreich (0.6237s)

---

## 1. Komponenten & Abhängigkeiten

- **Abhängigkeiten (4):** `thiserror`, `serde`, `tracing`, `crc32fast`
- **Features (6):** `default`, `std`, `crc`, `serde`, `tracing`, `inspect`

---

## 2. Test-Ergebnisse im Detail

### Unit-Tests (`--lib`)
- **Status:** ✅ Bestanden
- **Laufzeit:** 0.351s
- **Tests bestanden:** 0
- **Tests fehlgeschlagen:** 0
- **Tests ignoriert:** 0

### Integrationstests (`--tests`)
| Test-Target | Status | Bestanden | Fehlgeschlagen | Laufzeit (s) |
|---|---|---|---|---|
| `__all__` | ✅ OK | 43 | 0 | 0.5011s |

### Dokumentationstests (`--doc`)
- **Status:** ✅ Bestanden
- **Laufzeit:** 0.4156s
- **Tests bestanden:** 0

---

## 3. Performance & Benchmark-Messungen

### Gemessene Benchmark-Targets:

| Benchmark-Target | Status | Ausführungszeit (s) | Testpfade |
|---|---|---:|---:|
| `fs_bench` | ✅ OK | **0.82s** | 2 Pfade |

---|---|---|
| `fs_bench` | ⏱️ TIMEOUT | 20.0294s |

---

## 4. Aufgetretene Fehler & Compiler-Befunde

_Keine Compiler- oder Systemfehler registriert._
