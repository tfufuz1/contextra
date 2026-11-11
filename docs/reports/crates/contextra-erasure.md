# Audit- & Messreport: `contextra-erasure`

- **Crate-Pfad:** `crates/contextra-erasure`
- **Gesamtstatus:** **FEHLGESCHLAGEN (FAIL)**
- **Test-Statistik:** 40 bestanden, 0 fehlgeschlagen, 0 ignoriert
- **Kompilierung (cargo check):** ✅ Erfolgreich (0.3538s)

---

## 1. Komponenten & Abhängigkeiten

- **Abhängigkeiten (18):** `ahash`, `rand`, `subtle`, `aes-gcm-siv`, `thiserror`, `tracing`, `serde`, `serde_json`, `zeroize`, `blake3`, `sha2`, `hmac`, `ed25519-dalek`, `bytes`, `parking_lot`, `futures-util`, `tokio`, `bincode`
- **Features (4):** `default`, `signing`, `vault`, `async`

---

## 2. Test-Ergebnisse im Detail

### Unit-Tests (`--lib`)
- **Status:** ✅ Bestanden
- **Laufzeit:** 0.5064s
- **Tests bestanden:** 40
- **Tests fehlgeschlagen:** 0
- **Tests ignoriert:** 0

### Integrationstests (`--tests`)
| Test-Target | Status | Bestanden | Fehlgeschlagen | Laufzeit (s) |
|---|---|---|---|---|
| `__all__` | ❌ FAIL | 0 | 0 | 2.3883s |

### Dokumentationstests (`--doc`)
- **Status:** ✅ Bestanden
- **Laufzeit:** 1.2814s
- **Tests bestanden:** 0

---

## 3. Performance & Benchmark-Messungen

| Benchmark-Target | Status | Ausführungszeit (s) |
|---|---|---|
| `bm_erase` | ⏱️ TIMEOUT | 20.0098s |

---

## 4. Aufgetretene Fehler & Compiler-Befunde

### Compiler- & Laufzeitfehler:
```text
[tests] error[E0603]: trait `WalSink` is private
[tests] error[E0599]: no variant or associated item named `TokenMismatch` found for enum `FenceGateError` in the current scope
[tests] error[E0599]: no variant or associated item named `NotFound` found for enum `FenceGateError` in the current scope
[tests] error[E0599]: no method named `record_wal_entry` found for struct `MemoryWalSink` in the current scope
[tests] error[E0599]: no method named `record_wal_entry` found for struct `MemoryWalSink` in the current scope
[tests] error[E0599]: no method named `dispose_scope_wal` found for struct `MemoryWalSink` in the current scope
[tests] 2360 | |     ) -> Result<u64, crate::error::ErasureError>;
[tests] error[E0599]: no method named `dispose_scope_wal` found for struct `MemoryWalSink` in the current scope
[tests] 2360 | |     ) -> Result<u64, crate::error::ErasureError>;
[tests] error[E0599]: no method named `dispose_scope_wal` found for struct `MemoryWalSink` in the current scope
[tests] 2360 | |     ) -> Result<u64, crate::error::ErasureError>;
[tests] error[E0599]: no variant or associated item named `DocStore` found for enum `WitnessLayer` in the current scope
[tests] error[E0061]: this function takes 1 argument but 4 arguments were supplied
[tests] error[E0599]: no variant or associated item named `VectorPayload` found for enum `WitnessLayer` in the current scope
[tests] error[E0061]: this function takes 1 argument but 4 arguments were supplied
[tests] error: could not compile `contextra-erasure` (test "proof_fail_closed_and_consistency") due to 12 previous errors; 2 warnings emitted
```
