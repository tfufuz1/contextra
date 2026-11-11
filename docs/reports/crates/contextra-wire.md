# Audit- & Messreport: `contextra-wire`

- **Crate-Pfad:** `crates/contextra-wire`
- **Gesamtstatus:** **BESTANDEN (PASS)**
- **Test-Statistik:** 137 bestanden, 0 fehlgeschlagen, 0 ignoriert
- **Kompilierung (cargo check):** ✅ Erfolgreich (0.7107s)

---

## 1. Komponenten & Abhängigkeiten

- **Abhängigkeiten (6):** `flatbuffers`, `bytes`, `serde`, `serde_json`, `tracing`, `thiserror`
- **Features (5):** `default`, `std`, `serde`, `tracing`, `inspect`

---

## 2. Test-Ergebnisse im Detail

### Unit-Tests (`--lib`)
- **Status:** ✅ Bestanden
- **Laufzeit:** 1.8573s
- **Tests bestanden:** 26
- **Tests fehlgeschlagen:** 0
- **Tests ignoriert:** 0

### Integrationstests (`--tests`)
| Test-Target | Status | Bestanden | Fehlgeschlagen | Laufzeit (s) |
|---|---|---|---|---|
| `__all__` | ✅ OK | 111 | 0 | 3.5835s |

### Dokumentationstests (`--doc`)
- **Status:** ✅ Bestanden
- **Laufzeit:** 0.4567s
- **Tests bestanden:** 0

---

## 3. Performance & Benchmark-Messungen

### Gemessene Benchmark-Targets:

| Benchmark-Target | Status | Ausführungszeit (s) | Testpfade |
|---|---|---:|---:|
| `wire_bench` | ✅ OK | **59.75s** | 6 Pfade |

---|---|---|
| `wire_bench` | ⏱️ TIMEOUT | 20.0297s |

---

## 4. Aufgetretene Fehler & Compiler-Befunde

_Keine Compiler- oder Systemfehler registriert._
