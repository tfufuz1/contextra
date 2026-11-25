# contextra-crypto

`contextra-crypto` stellt die kryptographischen Grundlagen at rest und zur Datenintegrität bereit (Ring 0).

## Zweck

Verwaltet AEAD-Verschlüsselung (AES-256-GCM-SIV), HKDF-Schlüsselableitung, HMAC-Integritätsketten für WAL-Einträge sowie DSGVO-Art.-17-Löschnachweise (`DeletionProof`).

## Ring-Zugehörigkeit & Status

- **Ring:** Ring 0 (Kryptographischer Kern)
- **Status:** 🟢 Fertig
- **Sicherheits-Invariante:** Safe API (In-Memory Zeroization, Key Hierarchy)

## Öffentliche API-Übersicht

- **Key Management:** `KeyManager`, `CryptoKey`
- **Deletion Proofs:** `DeletionProof`, `DeletionLayer`, `DeletionScope`, `LayerCleanupProof`
- **Egress Vault & Filter:** `EgressVault`, `EgressClassifier`, `EgressClassification`, `CompiledPattern`

## Architektur & Verweise

Details zum Sicherheitsmodell finden sich in [`ARCHITECTURE.md`](../../ARCHITECTURE.md) (folgt in Kürze) sowie `README.md` §10.
