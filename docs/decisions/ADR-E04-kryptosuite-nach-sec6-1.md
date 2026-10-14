# ADR-E04: kryptosuite-nach-sec6-1

* **Status:** Akzeptiert (aus Spezifikation §13 E04)
* **Datum:** 2026-10-14

## Entscheidung
Kryptosuite nach §6.1 (AES-256-GCM-SIV, Argon2id, HKDF-SHA-256, HMAC-SHA-256, BLAKE3, Ed25519 `verify_strict`). Kein Claim „BSI-konform".

## Begründung
Entspricht dem Code; TR-02102 ist keine Löschnorm.

## Verworfen
SHA-256 als Beweisdigest, ChaCha20, plain GCM
