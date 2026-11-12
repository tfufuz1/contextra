# Audit- & Messreport: `contextra-types`

- **Crate-Pfad:** `crates/contextra-types`
- **Gesamtstatus:** **BESTANDEN (PASS)**
- **Test-Statistik:** 307 bestanden, 0 fehlgeschlagen, 1 ignoriert
- **Kompilierung (cargo check):** ✅ Erfolgreich (0.3158s)

---

## 1. Komponenten & Abhängigkeiten

- **Abhängigkeiten (9):** `contextra-ids`, `thiserror`, `serde`, `serde_json`, `bincode`, `ahash`, `blake3`, `sha2`, `bytes`
- **Features (3):** `default`, `docid-128`, `auto-extraction-opt-out`

---

## 2. Test-Ergebnisse im Detail

### Unit-Tests (`--lib`)
- **Status:** ✅ Bestanden
- **Laufzeit:** 10.5809s
- **Tests bestanden:** 120
- **Tests fehlgeschlagen:** 0
- **Tests ignoriert:** 0

### Integrationstests (`--tests`)
| Test-Target | Status | Bestanden | Fehlgeschlagen | Laufzeit (s) |
|---|---|---|---|---|
| `__all__` | ✅ OK | 187 | 0 | 5.6453s |

### Dokumentationstests (`--doc`)
- **Status:** ✅ Bestanden
- **Laufzeit:** 0.8163s
- **Tests bestanden:** 0

---

## 3. Performance & Benchmark-Messungen

### Gemessene Benchmark-Targets:

| Benchmark-Target | Status | Ausführungszeit (s) | Testpfade |
|---|---|---:|---:|
| `types_bench` | ✅ OK | **25.76s** | 0 Pfade |

---|---|---|
| `types_bench` | ⏱️ TIMEOUT | 20.0285s |

---

## 4. Aufgetretene Fehler & Compiler-Befunde

_Keine Compiler- oder Systemfehler registriert._
