# ADR-E04: Kryptosuite nach §6.1 (AES-256-GCM-SIV, Argon2id, HKDF-SHA-2

* **Status:** Akzeptiert (aus Spezifikation §13 E04)
* **Datum:** 2026-10-15

## Entscheidung
Kryptosuite nach §6.1 (AES-256-GCM-SIV, Argon2id, HKDF-SHA-256, HMAC-SHA-256, BLAKE3, Ed25519 `verify_strict`). Kein Claim „BSI-konform".

## Begründung
Entspricht dem Code; TR-02102 ist keine Löschnorm.

## Verworfen
SHA-256 als Beweisdigest, ChaCha20, plain GCM
