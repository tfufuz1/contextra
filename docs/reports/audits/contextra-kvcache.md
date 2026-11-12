# Audit- & Messreport: `contextra-kvcache`

- **Crate-Pfad:** `crates/contextra-kvcache`
- **Gesamtstatus:** **FEHLGESCHLAGEN (FAIL)**
- **Test-Statistik:** 1 bestanden, 0 fehlgeschlagen, 0 ignoriert
- **Kompilierung (cargo check):** ✅ Erfolgreich (1.3251s)

---

## 1. Komponenten & Abhängigkeiten

- **Abhängigkeiten (15):** `contextra-env`, `contextra-types`, `contextra-ports`, `contextra-crypto`, `ahash`, `blake3`, `lru`, `parking_lot`, `thiserror`, `tracing`, `zeroize`, `serde`, `serde_json`, `bincode`, `bytes`
- **Features (12):** `default`, `std`, `workspace`, `kv-encryption`, `exp-kivi`, `exp-content-addressed`, `ports-compat`, `types-compat`, `crypto-compat`, `kivi-quantization`, `kvcache-kivi-quant`, `content-addressed-kv-cache`

---

## 2. Test-Ergebnisse im Detail

### Unit-Tests (`--lib`)
- **Status:** ❌ Nicht bestanden / Fehler
- **Laufzeit:** 1.7552s
- **Tests bestanden:** 0
- **Tests fehlgeschlagen:** 0
- **Tests ignoriert:** 0

### Integrationstests (`--tests`)
| Test-Target | Status | Bestanden | Fehlgeschlagen | Laufzeit (s) |
|---|---|---|---|---|
| `__all__` | ❌ FAIL | 0 | 0 | 4.3568s |

### Dokumentationstests (`--doc`)
- **Status:** ✅ Bestanden
- **Laufzeit:** 2.0974s
- **Tests bestanden:** 1

---

## 3. Performance & Benchmark-Messungen

| Benchmark-Target | Status | Ausführungszeit (s) |
|---|---|---|
| `kvcache_bench` | ⏱️ TIMEOUT | 20.0166s |

---

## 4. Aufgetretene Fehler & Compiler-Befunde

### Compiler- & Laufzeitfehler:
```text
error[E0433]: failed to resolve: use of undeclared type `KvSegmentCipher`
error[E0433]: failed to resolve: use of undeclared type `ModelFingerprint`
error[E0433]: failed to resolve: use of undeclared type `KvSegmentCipher`
error[E0433]: failed to resolve: use of undeclared type `ModelFingerprint`
error[E0433]: failed to resolve: use of undeclared type `KvSegmentCipher`
error[E0433]: failed to resolve: use of undeclared type `ModelFingerprint`
error: could not compile `contextra-kvcache` (lib test) due to 6 previous errors
[tests] error[E0433]: failed to resolve: use of undeclared type `KvSegmentCipher`
[tests] error[E0433]: failed to resolve: use of undeclared type `ModelFingerprint`
[tests] error[E0433]: failed to resolve: use of undeclared type `KvSegmentCipher`
[tests] error[E0433]: failed to resolve: use of undeclared type `ModelFingerprint`
[tests] error[E0433]: failed to resolve: use of undeclared type `KvSegmentCipher`
[tests] error[E0433]: failed to resolve: use of undeclared type `ModelFingerprint`
[tests] error: could not compile `contextra-kvcache` (lib test) due to 6 previous errors
```
