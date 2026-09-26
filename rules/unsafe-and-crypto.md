# SIMD, Unsafe & Crypto Rules

This document consolidates safety rules for SIMD/unsafe operations and WAL/crypto invariants.

---

## SIMD & Unsafe Safety Rules

> Origin: `rules/simd_safety.md`

### SAFETY-Kommentar-Pflicht

Jeder `unsafe`-Block in `contextra-vector/src/distance.rs` braucht:

```rust
// SAFETY: `a` und `b` haben identische Länge (geprüft durch Caller `compute_distance`
//         vor dem Dispatch). Slice-Pointer sind durch Rust-Allokator 32-Byte-aligned
//         für AVX2-Zugriffe. Keine Aliasing-Verletzung (exclusive borrows).
unsafe fn dot_product_avx2(a: &[f32], b: &[f32]) -> f32 { ... }
```

### Inhalts-Unikats-Pflicht (Schutz vor Copy-Paste-SAFETY-Blocks)

SAFETY-Kommentare dürfen NICHT wortgleich aus anderen Funktionen kopiert werden.
Jeder SAFETY-Kommentar MUSS die **KONKRETE Invariante DIESER spezifischen Funktion** benennen:
- Welcher konkrete Aufrufer garantiert welche Vorbedingung?
- Warum ist genau diese Zeigerarithmetik oder dieser Mmap-Zugriff für diese spezifische Datenstruktur sicher?
- Welche konkreten Ausrichtungs- (Alignment) oder Längen-Garantien liegen vor?

Ein SAFETY-Kommentar, der wortgleich in mehr als einer Funktion vorkommt, gilt als Qualitätsverstoß und muss individualisiert werden.

**Informativer CI-Grep-Hinweis (Redundanz-Scan)**:
```bash
grep -rn "// SAFETY:" crates/ | sort | uniq -c | grep -v " 1 "
```

### Pflicht-Fallback

Für jede SIMD-Funktion existiert ein skalarer Fallback mit **identischem numerischen Ergebnis** (Epsilon ≤ 1e-4 relativ, §4 Determinismus-Gesetz).

### Runtime Feature Detection

```rust
#[cfg(target_arch = "x86_64")]
if is_x86_feature_detected!("avx512f") { ... }
else if is_x86_feature_detected!("avx2") { ... }
else { scalar_fallback(...) }
```

Kein unconditional `target_feature`-Aufruf ohne `cfg`-Gate.

### Aktueller Status

- 42 unsafe-Blöcke in `distance.rs` (AVX2 + AVX-512) — SAFETY-Kommentare sind Voraussetzung für Merge.
- `#![deny(unsafe_op_in_unsafe_fn)]` ist gesetzt — compliant.

---

## WAL & Crypto Invarianten

> Origin: `rules/wal_crypto.md`

### WAL-First Regel

Kein Speicherzustand wird modifiziert, bevor der WAL-Eintrag physisch committed + synced ist.
Reihenfolge: `WAL::append()` → `fsync()` → `MemTable::apply()`.

### HMAC-Chaining

```
Entry_N.checksum = HMAC(key, prev_hmac_N-1 || seq_N || op_type || payload)
```

`prev_hmac` der ersten Entry = `[0u8; 32]`.
Bei Replay: Chain von Entry 0 bis letzte Entry validieren. Abbruch bei erstem Mismatch → `ContextraError::WalCorruption`.

### Crypto-Isolation

- `contextra-crypto` ist die **einzige** Stelle für Krypto-Primitiven (AES-GCM-SIV).
- Jede WAL-Datei hat ein eigenes Key (HKDF-Ableitung aus UUID-Sidecar).
- UUID-Sidecar: `<wal_path>.uuid` (16 Byte, raw). Muss vor erster WAL-Nutzung existieren.

### Was niemals passieren darf

- `bincode::deserialize(...).unwrap_or_default()` auf WAL-Einträge — Korruption wird zu Datenverlust.
- Krypto-Code außerhalb von `contextra-crypto` (auch nicht „nur für diesen einen Fall").
