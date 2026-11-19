# contextra-crypto — Implementierungsplan zum Soll-Zustand

**Normative Basis:** `docs/SPEC.md` §4 und `docs/crates/contextra-crypto.md`. **Ausgangslage:** AEAD/KDF-Bausteine existieren, daneben historisches KV-/Egress-/Audit-System; vier E2-Abhängigkeiten und zahlreiche Produktions-Panikmuster verletzen die reine E1-Eigentümerschaft.

1. Reduziere die Closure auf `ids, env`; entferne Ports/Types-Compat sowie KV-Cache-, Egress- und fachliche Auditlogik aus dem Kryptokern.
2. Implementiere die Schlüsselhierarchie exklusiv hier: Argon2id (m=256 MiB, t=4, p=1), HKDF-SHA-256 mit Domänentrennung, Tenant-/Collection-/Subject-/Document-/Erasure-Schlüssel.
3. Kanonisiere Secret-Typen: kein Clone/Debug/Serialize, `ZeroizeOnDrop`; OS-RNG via `try_fill_bytes`, Fehler stoppt Verschlüsselung.
4. Implementiere AES-256-GCM-SIV mit 96-Bit Zufallsnonce, verbindlichem AAD `(tenant, collection, doc, version)`, versioniertem Record-/Wrap-Format und KeyTable/Registry.
5. Implementiere idempotentes Slot-Shredding (zufällig überschreiben, nullen, fsync, Abwesenheit prüfen), Epoch-Seal und Witness ohne Klartext in Logs; ersetze alle Panics durch `CryptoError`.

**Abnahme:** KAT-/Roundtrip-/AAD-/Nonce-RNG-Fehler-Tests, Zeroize-/No-Clone-Compile-Tests, Shred-Recovery und Format-Golden-Tests.
