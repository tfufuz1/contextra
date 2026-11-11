# Audit- & Messreport: `contextra-crypto`

- **Crate-Pfad:** `crates/contextra-crypto`
- **Gesamtstatus:** **FEHLGESCHLAGEN (FAIL)**
- **Test-Statistik:** 193 bestanden, 2 fehlgeschlagen, 1 ignoriert
- **Kompilierung (cargo check):** ✅ Erfolgreich (2.5376s)

---

## 1. Komponenten & Abhängigkeiten

- **Abhängigkeiten (22):** `thiserror`, `sha2`, `hmac`, `aes-gcm-siv`, `argon2`, `hkdf`, `subtle`, `blake3`, `bincode`, `serde`, `serde_json`, `contextra-ids`, `contextra-env`, `contextra-types`, `contextra-ports`, `rand`, `tracing`, `zeroize`, `parking_lot`, `ahash`, `lru`, `regex`
- **Features (7):** `default`, `workspace`, `types-compat`, `ports-compat`, `test-utils`, `kv-encryption`, `cloud-egress-guard`

---

## 2. Test-Ergebnisse im Detail

### Unit-Tests (`--lib`)
- **Status:** ❌ Nicht bestanden / Fehler
- **Laufzeit:** 40.0477s
- **Tests bestanden:** 112
- **Tests fehlgeschlagen:** 0
- **Tests ignoriert:** 0

### Integrationstests (`--tests`)
| Test-Target | Status | Bestanden | Fehlgeschlagen | Laufzeit (s) |
|---|---|---|---|---|
| `anti_tamper_matrix` | ✅ OK | 3 | 0 | 0.4163s |
| `audit_chain_tests` | ✅ OK | 3 | 0 | 0.3975s |
| `campaign_j25` | ⏱️ TIMEOUT | 19 | 0 | 12.0158s |
| `campaign_mut_j17` | ✅ OK | 5 | 0 | 0.5096s |
| `crypto_property_and_invariants` | ✅ OK | 5 | 0 | 0.573s |
| `crypto_vectors_extended` | ✅ OK | 10 | 0 | 9.9029s |
| `eviction_worker_tokio_runtime_nonblocking_regression` | ✅ OK | 1 | 0 | 2.6819s |
| `golden_corpus` | ❌ FAIL | 1 | 2 | 0.4377s |
| `kdf_argon2id` | ✅ OK | 9 | 0 | 2.5208s |
| `key_separation_and_edge_cases` | ⏱️ TIMEOUT | 9 | 0 | 12.0186s |
| `kv_segment_concurrency` | ✅ OK | 8 | 0 | 0.5314s |
| `kv_segment_integration` | ✅ OK | 2 | 0 | 0.3987s |
| `kv_segment_proptests` | ✅ OK | 3 | 0 | 0.4382s |
| `loom_eviction_worker_shutdown_race` | ✅ OK | 2 | 0 | 0.4017s |
| `loom_kv_deferred_zeroize` | ✅ OK | 1 | 0 | 0.3996s |

### Dokumentationstests (`--doc`)
- **Status:** ✅ Bestanden
- **Laufzeit:** 0.5651s
- **Tests bestanden:** 0

---

## 3. Performance & Benchmark-Messungen

### Gemessene Benchmark-Targets:

| Benchmark-Target | Status | Ausführungszeit (s) | Testpfade |
|---|---|---:|---:|
| `crypto_benchmarks` | ✅ OK | **1.01s** | 17 Pfade |

---|---|---|
| `crypto_benchmarks` | ⏱️ TIMEOUT | 20.0101s |

---

## 4. Aufgetretene Fehler & Compiler-Befunde

### Fehlgeschlagene Tests:
- ❌ `golden_corpus::test_golden_corpus_files_exist_and_match_manifest`
- ❌ `golden_corpus::test_golden_corpus_backward_reading_without_panics`

_Keine Compiler- oder Systemfehler registriert._
