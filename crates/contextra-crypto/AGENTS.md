 Zentrale Bausteine: aead (seal_record_with_nonce, open_record), dek (Dek), Schlüsseltypen (SecretKey, Secret, VolatileEncryptionKey, UnitKey), Integritätsketten, Egress-Verschlüsselung und HMAC.

Harte Invarianten:
- INV-CRYPTO-CONSTANT-TIME-1: Alle Vergleiche von Hashes, Schlüsseln und Tags laufen über `subtle::ConstantTimeEq`. Kein `==` auf Geheimnissen.
- INV-CRYPTO-ZEROIZE-1: Alle Schlüsseltypen implementieren ZeroizeOnDrop. Debug und Display zeigen `[REDACTED]`.
- Eigene Krypto-Konstruktionen sind verboten. Nur geprüfte Primitive aus etablierten Crates.
- Nonce-Wiederverwendung wird durch AES-GCM-SIV abgefedert, aber nicht als erlaubt behandelt. Nonce-Erzeugung und -Verwaltung sind dokumentiert und getestet.
- Keine rechtlichen Compliance-Aussagen im Code oder in der Doku (Nicht-Ziel laut README).

Deine Stärken: Schlüsselableitung mit Domänentrennung (HKDF-Labels), Argon2-Parameterwahl, Testvektoren aus RFCs, Seitenkanal-Bewusstsein.

Pflichten: Jede Änderung an Schlüsselformaten ist ein Migrationsfall. Testvektoren und Round-Trip-Tests erweitern. Fehlermeldungen dürfen keine Schlüsselinformationen leaken.

Verifikation: `cargo test -p contextra-crypto --locked`, `cargo clippy -p contextra-crypto --all-targets -- -D warnings`, Benchmarks in `benches/`, `cargo deny check`.

Optimierungsfokus: Wiederverwendung von Cipher-Instanzen, Vermeidung unnötiger Kopien geheimer Daten, LRU-Caching nur mit Zeroize der Einträge.
