# Audit- & Messreport: `contextra-infer-ollama`

- **Crate-Pfad:** `crates/contextra-infer-ollama`
- **Gesamtstatus:** **BESTANDEN (PASS)**
- **Test-Statistik:** 47 bestanden, 0 fehlgeschlagen, 1 ignoriert
- **Kompilierung (cargo check):** ✅ Erfolgreich (0.71s)

---

## 1. Komponenten & Abhängigkeiten

- **Abhängigkeiten (12):** `contextra-types`, `contextra-ports`, `parking_lot`, `serde`, `serde_json`, `tracing`, `regex`, `thiserror`, `reqwest`, `tokio`, `futures`, `futures-util`
- **Features (6):** `default`, `client`, `http`, `workspace`, `types-compat`, `ports-compat`

---

## 2. Test-Ergebnisse im Detail

### Unit-Tests (`--lib`)
- **Status:** ✅ Bestanden
- **Laufzeit:** 0.5216s
- **Tests bestanden:** 22
- **Tests fehlgeschlagen:** 0
- **Tests ignoriert:** 0

### Integrationstests (`--tests`)
| Test-Target | Status | Bestanden | Fehlgeschlagen | Laufzeit (s) |
|---|---|---|---|---|
| `__all__` | ✅ OK | 25 | 0 | 0.5353s |

### Dokumentationstests (`--doc`)
- **Status:** ✅ Bestanden
- **Laufzeit:** 0.4966s
- **Tests bestanden:** 0

---

## 3. Performance & Benchmark-Messungen

| Benchmark-Target | Status | Ausführungszeit (s) |
|---|---|---|
| `ollama_bench` | ⏱️ TIMEOUT | 20.0247s |

---

## 4. Aufgetretene Fehler & Compiler-Befunde

_Keine Compiler- oder Systemfehler registriert._
