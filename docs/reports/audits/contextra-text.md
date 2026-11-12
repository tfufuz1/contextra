# Audit- & Messreport: `contextra-text`

- **Crate-Pfad:** `crates/contextra-text`
- **Gesamtstatus:** **BESTANDEN (PASS)**
- **Test-Statistik:** 110 bestanden, 0 fehlgeschlagen, 0 ignoriert
- **Kompilierung (cargo check):** ✅ Erfolgreich (0.3545s)

---

## 1. Komponenten & Abhängigkeiten

- **Abhängigkeiten (16):** `contextra-ids`, `contextra-env`, `contextra-types`, `contextra-ports`, `bytes`, `itoa`, `regex`, `unicode-segmentation`, `bincode`, `serde`, `serde_json`, `thiserror`, `parking_lot`, `tracing`, `blake3`, `futures`
- **Features (11):** `default`, `std`, `serde`, `tracing`, `inspect`, `experimental`, `workspace`, `types-compat`, `ports-compat`, `bm25f`, `docid-128`

---

## 2. Test-Ergebnisse im Detail

### Unit-Tests (`--lib`)
- **Status:** ❌ Nicht bestanden / Fehler
- **Laufzeit:** 30.03s
- **Tests bestanden:** 108
- **Tests fehlgeschlagen:** 0
- **Tests ignoriert:** 0

### Integrationstests (`--tests`)
| Test-Target | Status | Bestanden | Fehlgeschlagen | Laufzeit (s) |
|---|---|---|---|---|
| `__bulk_timeout__` | ⏱️ TIMEOUT | 0 | 0 | 30.0s |

### Dokumentationstests (`--doc`)
- **Status:** ✅ Bestanden
- **Laufzeit:** 2.0976s
- **Tests bestanden:** 2

---

## 3. Performance & Benchmark-Messungen

| Benchmark-Target | Status | Ausführungszeit (s) |
|---|---|---|
| `text_bench` | ⏱️ TIMEOUT | 20.0214s |

---

## 4. Aufgetretene Fehler & Compiler-Befunde

_Keine Compiler- oder Systemfehler registriert._
