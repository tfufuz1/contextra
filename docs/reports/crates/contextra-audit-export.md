# Audit- & Messreport: `contextra-audit-export`

- **Crate-Pfad:** `crates/contextra-audit-export`
- **Gesamtstatus:** **FEHLGESCHLAGEN (FAIL)**
- **Test-Statistik:** 8 bestanden, 1 fehlgeschlagen, 0 ignoriert
- **Kompilierung (cargo check):** ✅ Erfolgreich (0.3338s)

---

## 1. Komponenten & Abhängigkeiten

- **Abhängigkeiten (4):** `contextra-types`, `serde`, `serde_json`, `thiserror`
- **Features (5):** `default`, `std`, `workspace`, `types-compat`, `test-utils`

---

## 2. Test-Ergebnisse im Detail

### Unit-Tests (`--lib`)
- **Status:** ✅ Bestanden
- **Laufzeit:** 0.3583s
- **Tests bestanden:** 4
- **Tests fehlgeschlagen:** 0
- **Tests ignoriert:** 0

### Integrationstests (`--tests`)
| Test-Target | Status | Bestanden | Fehlgeschlagen | Laufzeit (s) |
|---|---|---|---|---|
| `__all__` | ❌ FAIL | 4 | 1 | 0.3542s |

### Dokumentationstests (`--doc`)
- **Status:** ✅ Bestanden
- **Laufzeit:** 0.4333s
- **Tests bestanden:** 0

---

## 3. Performance & Benchmark-Messungen

| Benchmark-Target | Status | Ausführungszeit (s) |
|---|---|---|
| `audit_export_bench` | ⏱️ TIMEOUT | 20.0257s |

---

## 4. Aufgetretene Fehler & Compiler-Befunde

### Fehlgeschlagene Tests:
- ❌ `tests::test_h10_gdpr_art30_processing_registry_and_canary_leak_check`

### Compiler- & Laufzeitfehler:
```text
[tests] error: test failed, to rerun pass `-p contextra-audit-export --test campaign_j27_compliance_test`
```
