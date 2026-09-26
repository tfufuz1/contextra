<!-- Dieses Dokument wird aus crates/contextra-audit-export/src/bsi_mapping.rs::bsi_mapping_table() generiert bzw. muss synchron dazu gehalten werden — bei Abweichung gilt der Code als normativ, analog Leitprinzip §0 -->

# BSI Grundschutz / TR-02102 Kryptographische Zuordnung

> **Hinweis:** Dieses Dokument / dieser Export dient der technischen Referenzzuordnung nach BSI TR-02102 und stellt KEINE Zertifizierungs- oder Konformitätsbehauptung dar. Die BSI-Empfehlungen sind regelmäßig gegen die neuesten Veröffentlichungen des BSI zu prüfen.

| Krypto-Primitive | Code-Fundstelle | BSI-Referenz | Anmerkung / Kontext |
| --- | --- | --- | --- |
| Ed25519 | `crates/contextra-crypto/src/deletion_proof.rs` | BSI TR-02102-1 | Digitale Signatur für Löschnachweise (DeletionProof v3, §14.1). BSI TR-02102-1 §1.5/§3.2 bewertet Ed25519 (Curve25519) als empfohlenes Asymmetrisches Signaturverfahren mit 128-Bit Sicherheit. Stand: März 2024, zu prüfen gegen aktuelle BSI-Veröffentlichung (keine Konformitätsbehauptung). |
| HMAC-SHA256 | `crates/contextra-crypto/src/wal_crypto.rs` | BSI TR-02102-1 | WAL-Integritätskette und Tamper-Detection (§6.5). BSI TR-02102-1 §1.6 empfiehlt HMAC in Kombination mit SHA-256 (Mindestschlüssellänge 128 Bit, Contextra nutzt 256 Bit IntegrityKey) für Keyed Hash Message Authentication. Stand: März 2024, zu prüfen gegen aktuelle BSI-Veröffentlichung (keine Konformitätsbehauptung). |
| AES-256-GCM-SIV | `crates/contextra-crypto/src/crypto.rs` | BSI TR-02102-1 | Symmetrische Verschlüsselung (Envelope Encryption, KeyManager, §10). BSI TR-02102-1 §1.3 empfiehlt AES mit ≥128-Bit Schlüssellänge; AES-256-GCM-SIV stellt missbrauchsresistente authentifizierte Authenticated Encryption (AEAD) nach RFC 8452 bereit. Stand: März 2024, zu prüfen gegen aktuelle BSI-Veröffentlichung (keine Konformitätsbehauptung). |
| Argon2id | `crates/contextra-crypto/src/kdf.rs` | BSI TR-02102-1 / BSI TR-02102-4 | Passwortbasierte Schlüsselableitung (Tenant-DEK / Passphrase-DEK, KdfHeader, §10). Argon2id erfüllt OWASP/BSI-Anforderungen für speicherharte Schlüsselableitungsfunktionen (KDF) mit m_cost >= 19456 KiB, t_cost >= 2, p_cost >= 1. Stand: März 2024, zu prüfen gegen aktuelle BSI-Veröffentlichung (keine Konformitätsbehauptung). |
