# Audit- & Messreport: `contextra-avv-generator`

- **Crate-Pfad:** `crates/contextra-avv-generator`
- **Gesamtstatus:** **FEHLGESCHLAGEN (FAIL)**
- **Test-Statistik:** 27 bestanden, 1 fehlgeschlagen, 0 ignoriert
- **Kompilierung (cargo check):** ✅ Erfolgreich (0.3012s)

---

## 1. Komponenten & Abhängigkeiten

- **Abhängigkeiten (2):** `thiserror`, `serde`
- **Features (3):** `default`, `std`, `serde`

---

## 2. Test-Ergebnisse im Detail

### Unit-Tests (`--lib`)
- **Status:** ✅ Bestanden
- **Laufzeit:** 0.3335s
- **Tests bestanden:** 0
- **Tests fehlgeschlagen:** 0
- **Tests ignoriert:** 0

### Integrationstests (`--tests`)
| Test-Target | Status | Bestanden | Fehlgeschlagen | Laufzeit (s) |
|---|---|---|---|---|
| `__all__` | ❌ FAIL | 24 | 1 | 0.3327s |

### Dokumentationstests (`--doc`)
- **Status:** ✅ Bestanden
- **Laufzeit:** 0.6729s
- **Tests bestanden:** 3

---

## 3. Performance & Benchmark-Messungen

| Benchmark-Target | Status | Ausführungszeit (s) |
|---|---|---|
| `avv_bench` | ⏱️ TIMEOUT | 20.0283s |

---

## 4. Aufgetretene Fehler & Compiler-Befunde

### Fehlgeschlagene Tests:
- ❌ `tests::test_h11_gdpr_art28_avv_generator_technical_completeness`

### Compiler- & Laufzeitfehler:
```text
[tests] error: test failed, to rerun pass `-p contextra-avv-generator --test campaign_j27_avv_test`
```
