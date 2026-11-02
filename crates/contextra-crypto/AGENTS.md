# AGENTS – contextra-crypto

Abgeleitet aus [`docs/SPEC.md`](../../docs/SPEC.md). Bei Widerspruch gilt die SPEC.

**Zweck:** Kryptoprimitive: AEAD, Argon2id, HKDF, HMAC, Ed25519, Tags.

**Ring und Abhängigkeiten:** Ring 1. Erlaubt: `types`, `ports`.

**Verboten:** Feature-Schalter für Verschlüsselung oder Integrität (D-05), `legacy-wal-key` und `allow_legacy_integrity_key_fallback` (D-05), Passphrasen-Ableitung über HKDF (D-06), Klartextmodus, `alg_id = 2` (reserviert, nicht implementieren, D-80), nicht zurückgesetzte Schlüssel (`ZeroizeOnDrop`)

**Entscheidungen:** D-05, D-06, D-52, D-60, D-68, D-80

**Tests:** T-KDF-1, T-KDF-2, T-KEYSLOT-1 bis T-KEYSLOT-4, T-NONCE-1, T-ENC-1 bis T-ENC-3, T-FMT-1

**Befunde zu prüfen:** F-09, F-11, F-21, F-24
