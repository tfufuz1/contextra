# Audit- & Messreport: `contextra-license`

- **Crate-Pfad:** `crates/contextra-license`
- **Gesamtstatus:** **FEHLGESCHLAGEN (FAIL)**
- **Test-Statistik:** 18 bestanden, 3 fehlgeschlagen, 0 ignoriert
- **Kompilierung (cargo check):** ✅ Erfolgreich (5.0162s)

---

## 1. Komponenten & Abhängigkeiten

- **Abhängigkeiten (8):** `thiserror`, `serde`, `bincode`, `blake3`, `ed25519-dalek`, `tracing`, `contextra-types`, `contextra-ports`
- **Features (4):** `default`, `std`, `workspace`, `ports-compat`

---

## 2. Test-Ergebnisse im Detail

### Unit-Tests (`--lib`)
- **Status:** ✅ Bestanden
- **Laufzeit:** 0.3818s
- **Tests bestanden:** 0
- **Tests fehlgeschlagen:** 0
- **Tests ignoriert:** 0

### Integrationstests (`--tests`)
| Test-Target | Status | Bestanden | Fehlgeschlagen | Laufzeit (s) |
|---|---|---|---|---|
| `__all__` | ❌ FAIL | 18 | 3 | 0.5102s |

### Dokumentationstests (`--doc`)
- **Status:** ✅ Bestanden
- **Laufzeit:** 0.462s
- **Tests bestanden:** 0

---

## 3. Performance & Benchmark-Messungen

| Benchmark-Target | Status | Ausführungszeit (s) |
|---|---|---|
| `license_benchmark` | ⏱️ TIMEOUT | 20.0252s |

---

## 4. Aufgetretene Fehler & Compiler-Befunde

### Fehlgeschlagene Tests:
- ❌ `tests::test_golden_tamper_and_truncation_rejects_without_panic`
- ❌ `tests::test_backward_read_license_golden_files`
- ❌ `tests::test_golden_license_manifest_and_payload_exact`

### Compiler- & Laufzeitfehler:
```text
[tests] error: test failed, to rerun pass `-p contextra-license --test golden_contract_test`
```
