# Audit- & Messreport: `contextra-checkpoint`

- **Crate-Pfad:** `crates/contextra-checkpoint`
- **Gesamtstatus:** **BESTANDEN (PASS)**
- **Test-Statistik:** 200 bestanden, 0 fehlgeschlagen, 1 ignoriert
- **Kompilierung (cargo check):** ✅ Erfolgreich (0.3435s)

---

## 1. Komponenten & Abhängigkeiten

- **Abhängigkeiten (12):** `contextra-ids`, `contextra-env`, `contextra-types`, `contextra-ports`, `bytes`, `tokio`, `tracing`, `serde`, `serde_json`, `parking_lot`, `blake3`, `thiserror`
- **Features (5):** `default`, `workspace`, `types-compat`, `ports-compat`, `docid-128`

---

## 2. Test-Ergebnisse im Detail

### Unit-Tests (`--lib`)
- **Status:** ✅ Bestanden
- **Laufzeit:** 0.5942s
- **Tests bestanden:** 59
- **Tests fehlgeschlagen:** 0
- **Tests ignoriert:** 0

### Integrationstests (`--tests`)
| Test-Target | Status | Bestanden | Fehlgeschlagen | Laufzeit (s) |
|---|---|---|---|---|
| `__all__` | ✅ OK | 141 | 0 | 0.9973s |

### Dokumentationstests (`--doc`)
- **Status:** ✅ Bestanden
- **Laufzeit:** 0.547s
- **Tests bestanden:** 0

---

## 3. Performance & Benchmark-Messungen

### Gemessene Benchmark-Targets:

| Benchmark-Target | Status | Ausführungszeit (s) | Testpfade |
|---|---|---:|---:|
| `checkpoint_bench` | ✅ OK | **0.83s** | 9 Pfade |

---|---|---|
| `checkpoint_bench` | ⏱️ TIMEOUT | 20.0284s |

---

## 4. Aufgetretene Fehler & Compiler-Befunde

_Keine Compiler- oder Systemfehler registriert._
