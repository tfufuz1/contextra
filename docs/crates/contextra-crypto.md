# contextra-crypto — Crate-Spezifikation

**Ebene:** E1a (Kern-Closure: `ids`, `env`)  
**Bindung:** `docs/SPEC.md` §4, §5, §7

---

## 1 Zweck & Kryptographische Primitive

`contextra-crypto` implementiert die kryptographischen Kernalgorithmen für Verschlüsselung, Hashing, Key-Derivation und Schlüsselzerstörung.

## 2 Kryptographischer Standard

| Funktion | Algorithmus | Spezifikation |
|---|---|---|
| Authenticated Encryption | AES-256-GCM-SIV | RFC 8452 (Misuse-resistant, schützt vor Nonce-Wiederverwendung) |
| Key Derivation | HKDF-SHA-256 | RFC 5869 |
| Master-Key-Derivation | Argon2id | m=256MB, t=4, p=1 |
| Nonces | 96 Bit | Zufall via `rand::rngs::OsRng::try_fill_bytes` (fehlbar) |
| Schlüsselzerstörung | Memory Wiping | `zeroize::ZeroizeOnDrop` für alle internen Puffer |

## 3 Rolle in der Löschkette & PII-Kapselung

- **Kryptographisches Shredding:** `contextra-crypto` liefert die Primitive zum Überschreiben und Nullen von Schlüsselslots im persistenten Tresor (`SagaVault`).
- **Keine stillen Fallbacks:** Schlägt HKDF, AEAD oder RNG fehl, wird strikt mit `Err(CryptoError)` abgebrochen. Kein Fallback auf Klartext oder unsichere BLAKE3-Masken.
- **Bindung an Kontext (AAD):** Verschlüsselungen fordern verpflichtend Additional Authenticated Data (AAD): `tenant_id || collection_id || doc_id || version`. Ein Chiffretext kann kryptographisch nicht in eine andere Collection oder Version verschoben werden.
- **ErasureLayer-Rolle:** Implementiert `ErasureLayer` für verbleibende Slot-Bereinigungen (Phase 4g).

## 4 Invarianten

- Schlüsselmaterial besitzt weder `Clone`, `Debug` noch unsichere Kopierroutinen.
- `#![forbid(unsafe_code)]`.
- Constant-time Vergleiche für MACs und Tags (`subtle::ConstantTimeEq`).
