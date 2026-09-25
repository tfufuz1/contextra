# BSI TR-02102 Kryptografie-Mapping & Audit-Matrix

- **Datum:** 18. September 2026
- **Geprüfter Commit-Hash:** `7e6ee5396a84102fc5357c8489b558bd8d2a9d17`
- **Geltungsbereich:** `crates/contextra-crypto/Cargo.toml` & `crates/contextra-crypto/src/*.rs`
- **Lebenszyklus-Hinweis:** Dieses Dokument stellt eine Momentaufnahme des genannten Commits dar. Bei jeder Änderung der kryptografischen Dependencies oder Algorithmen in `crates/contextra-crypto/Cargo.toml` muss dieses Mapping erneut geprüft und aktualisiert werden.

---

## 1. Mapping der kryptografischen Primitiven

| Primitive (`Crate::Funktion`) | Verwendungszweck im Code (Datei:Zeile-Bereich) | Relevanter BSI-TR-02102-Teil/Abschnitt & Grundschutz | Konformitätsstatus |
| :--- | :--- | :--- | :--- |
| `aes-gcm-siv` (`aes_gcm_siv::Aes256GcmSiv`) | Symmetrische Verschlüsselung von WAL-Chunks und KV-Cache-Tensoren (`crates/contextra-crypto/src/crypto.rs:293-370`, `crates/contextra-crypto/src/kv_cipher.rs:131-160`) | BSI TR-02102-1 §2.3 & §2.4 (Authentisierte Verschlüsselung AEAD, AES-256) | `konform` (Verwendet AES-256-GCM-SIV nach RFC 8452 mit 128-Bit Auth-Tag und Nonce-Misuse-Resistance) |
| `argon2` (`argon2::Argon2`) | Passwortbasierte Schlüsselableitung aus Benutzereingaben (`crates/contextra-crypto/src/kdf.rs:254-280`) | BSI TR-02102-1 §3.2 (Passwortbasierte Schlüsselableitung) & BSI IT-Grundschutz CON.1.A8 | `konform` (Verwendet Argon2id v0x13 mit OWASP-Standardparametern `m_cost` >= 19456 KiB, `t_cost` >= 2, `p_cost` >= 1) |
| `sha2` (`sha2::Sha256`) | Kryptografische Hashes für HKDF, HMAC und Hash-Ketten (`crates/contextra-crypto/src/crypto.rs:88`, `crates/contextra-crypto/src/deletion_proof.rs:393`, `crates/contextra-crypto/src/wal_crypto.rs:124`) | BSI TR-02102-1 §4.1 (Hashfunktionen) | `konform` (SHA-256 mit 256 Bit Ausgabelänge) |
| `hmac` (`hmac::Hmac<Sha256>`) | Nachrichtenauthentifizierung für WAL-Integritätsketten & DeletionProofs (`crates/contextra-crypto/src/deletion_proof.rs:391-400`, `crates/contextra-crypto/src/wal_crypto.rs:138-150`) | BSI TR-02102-1 §5.1 (Message Authentication Codes) | `nicht bewertbar, externes Review ausstehend` (Integritätsschutz via HMAC-SHA256 vorhanden; asymmetrischer Nachweis per Ed25519 ausstehend) |
| `hkdf` (`hkdf::Hkdf<Sha256>`) | Hierarchische Schlüsselableitung und Domain-Separation (`crates/contextra-crypto/src/crypto.rs:88-95`, `157-175`, `226-255`) | BSI TR-02102-1 §3.1 (Schlüsselableitungsfunktionen) | `konform` (HKDF-Extract/Expand nach RFC 5869 mit SHA-256) |
| `blake3` (`blake3::Hasher`) | Deterministische Akkumulation gelöschter Dokumentschlüssel im DeletionProof (`crates/contextra-crypto/src/deletion_proof.rs:242-250`) | BSI TR-02102-1 §4.1 | `abweichend, Grund: BLAKE3 ist nicht explizit in BSI TR-02102-1 gelistet, wird ausschliesslich als interner Hash-Akkumulator genutzt` |
| `subtle` (`subtle::ConstantTimeEq`) | Timing-Resistenter Vergleich von HMACs und Prüfsummen (`crates/contextra-crypto/src/anti_tamper.rs:59-65`, `crates/contextra-crypto/src/deletion_proof.rs:335`, `crates/contextra-crypto/src/wal_crypto.rs:251`) | BSI TR-02102-1 §1.2 & BSI IT-Grundschutz CON.1 (Seitenkanal-Schutz) | `konform` (Verhindert Timing-Attaken durch byteweisen Konstantzeit-Vergleich) |
| `zeroize` (`zeroize::Zeroize`, `zeroize::ZeroizeOnDrop`) | Sicheres Löschen flüchtiger Schlüssel im RAM (`crates/contextra-crypto/src/anti_tamper.rs:11-35`, `crates/contextra-crypto/src/kv_cipher.rs:41-55`, `crates/contextra-crypto/src/wal_crypto.rs:162-192`) | BSI IT-Grundschutz CON.1.A9 & BSI TR-02102-1 §1.2 | `konform` (Garantiert Überschreiben des Speicherbereichs bei Objektdrop und Notfall-Wipe) |
| `ed25519` (`ed25519-dalek`) | Digitale Signaturen für unverfälschbare DeletionProofs (`signature_version: 3`) | BSI TR-02102-1 §3.4 (Asymmetrische digitale Signaturen) | `in Migration, siehe Contextra_Verifizierungsplan.md §2.1` (Ist-Zustand im Code: HMAC-SHA256 mit `signature_version: 2`) |

---

## 2. Bekannte Lücken

Die folgenden Punkte sind als bestätigte echte Lücken im Sicherheitsfundament identifiziert und dokumentiert:

1. **HMAC statt Ed25519 für DeletionProofs:** DeletionProofs nutzen aktuell `HMAC-SHA256` (`signature_version: 2`) zur Bestätigung von Löschvorgängen. Da HMAC symmetrisch ist, besitzt der Inhaber des WAL-Schlüssels die technische Möglichkeit, rückwirkend Löschnachweise auszustellen. Die Umstellung auf asymmetrische `Ed25519`-Signaturen (`signature_version: 3`) ist in Vorbereitung (siehe `Contextra_Verifizierungsplan.md` §2.1).
2. **`deleted_keys_hash`-Kollisionsrisiko:** Der Hash der gelöschten Dokumentschlüssel wird über eine ungeordnete BLAKE3-Akkumulation erzeugt. Ohne strikt geordnete Kanonisierung besteht bei extrem großen Schlüsselmengen ein theoretisches Risiko von Reihenfolge-Abweichungen.

---

## 3. Nicht im Scope dieses Mappings

Folgende Komponenten des Workspaces enthalten Low-Level-Code oder Unsafe-Blöcke (gemäß `AGENTS.md` §6), werden jedoch **nicht** als kryptografische Primitiven eingestuft und sind daher **nicht** Gegenstand der BSI TR-02102 Bewertung:

- `contextra-simd`: Enthält hardwarebeschleunigte Vektorableitungen (AVX2/NEON) für Distanzberechnungen (Cosine, L2, Dot Product).
- `contextra-sys`: Stellt FFI-Bindungen und Systemschnittstellen für MMAP und Speicher-Locking (`mlock`) bereit.
- `contextra-wire`: Definiert IPC-Serialisierungsstrukturen (FlatBuffers) ohne eigene Kryptografie.

---

## 4. Methodischer Hinweis & Audit-Status

Dieses Dokument dient als strukturiertes Audit-Artefakt für Rechtsanwaltskanzleien, Notariate und medizinische Einrichtungen. Eine Einstufung als `konform` beschreibt die Übereinstimmung der Algorithmusauswahl mit den Empfehlungen der BSI-Richtlinie TR-02102 (Teile 1–4). Sie ersetzt **nicht** das in `Contextra_Verifizierungsplan.md` §2.3 geforderte externe Kryptografie- und Penetrations-Review durch eine unabhängige Prüfstelle.
