# contextra-crypto

Kryptografische Vaults, Zeroize, Egress-Verschlüsselung, HMAC-Chaining und Löschbeweise (`DeletionProof`) für Contextra.

Normativitätsklasse: **Kern `[K]`** (Ring 0, `stable`; §6.0, `capabilities.toml`).

## Funktionen & Spezifikationsbezug
- **AEAD-Verschlüsselung (REQ-KEY-001):** AES-256-GCM-SIV für SSTable-Blöcke und KV-Segmente über den Keystore (§20.1).
- **WAL HMAC Chaining (REQ-KEY-001):** HMAC-SHA256 Chaining zur Manipulationserkennung im WAL.
- **Kryptografischer Löschbeweis (INV-CLAIMS-1, REQ-DEL-005, REQ-DEL-006):** Erzeugung und Signatur von `DeletionProof` (Formatversion 4, §20.4) über sieben Schichten.
- **Außenformulierung (INV-CLAIMS-1, §4.3):** Aussagen zur Löschung folgen zwingend der ehrlichen Außenformulierung; Werbeaussagen wie „100 % gelöscht“ oder „DSGVO-konform“ sind unzulässig.
