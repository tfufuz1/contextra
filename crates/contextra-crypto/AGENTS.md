# AGENTS.md — `contextra-crypto`

> **Normative Vorgabe gemäß [docs/SPEC.md §8.6](file:///home/freddy/Projekte/contextra/docs/SPEC.md) und [PLAN.md](file:///home/freddy/Projekte/contextra/PLAN.md)**

## 1. Rolle & Zweck
`contextra-crypto` kapselt sämtliche kryptografischen Algorithmen, Schlüsselableitungen, AEAD-Ver-/Entschlüsselungen, Envelope-Strukturen, blinde Tags und Signaturprüfungen in einer einzigen, strikt auditierten Typfamilie.

## 2. Ring, Abhängigkeiten & Verbote
- **Ring:** Ring 1 (Kryptografie & Nachweis-Primitive)
- **Erlaubte Abhängigkeiten:** `contextra-types`, `contextra-ports`, `aes-gcm-siv`, `argon2`, `hkdf`, `hmac`, `sha2`, `blake3`, `ed25519-dalek`, `subtle`, `zeroize`, `rand` (ausschließlich `OsRng`).
- **Verbotene Abhängigkeiten:** Keine Abhängigkeiten zu Ring 2–4.
- **Verbotene Muster:**
  - **Strikt kein `unsafe`** (`#![forbid(unsafe_code)]`).
  - Kein I/O in Produktionspfaden.
  - **Kein HKDF für Passphrasen (B-15):** Passphrasen dürfen ausschließlich über Argon2id verarbeitet werden!
  - Kein Ablegen oder Ausgeben von Schlüsselmaterial in `Debug`, `Display`, `Clone` oder `Serialize`.

## 3. Modul-Inventar (Soll-Zustand laut SPEC §8.6)

| Modul | Inhalt & Verantwortung |
|---|---|
| `kdf` | Argon2id KDF: `KdfParams { m_kib, t, p }` (Minimum: 65.536 KiB = 64 MiB, t=3, p=1). `derive_master(passphrase, salt, params) -> SecretKey32`, `KdfHeader`. |
| `keys` | `KeySet`: Deterministische HKDF-Ableitung aller Teilschlüssel (`k_enc`, `k_idx`, `k_wal`, `k_journal`, `k_vault`, `k_unit`, `k_meta`, `k_hdr`, `k_sign_seed`) aus `K_master` mit festen Kontext-Labels; `commit_key(epoch)`. Ersetzt historischen `KeyManager`. |
| `aead` | Authenticated Encryption: AES-256-GCM-SIV (`seal`, `open`). 96-Bit Nonce aus `OsRng`, strikte Replay- und Zählergrenzen ($2^{32}$). |
| `envelope` | `DocDek::derive(k_unit, salt, doc_tag)` und `Envelope::seal/open` (Layout gemäß SPEC §9.3). |
| `tag` | `blind_tag(k_idx, label, parts) -> [u8; 16]` via Keyed-BLAKE3 mit Längenpräfix je Bestandteil gegen Kollisionen. |
| `wal_crypto` | HMAC-SHA256-Ketten für WAL-Frames und kryptografische Frame-Integritätsprüfung. |
| `audit_chain` | Manipulationssichere BLAKE3-Hash-Kette für Löschjournal und Restore-Guard-Ledger. |
| `sign` | Ed25519-Signaturen (`sign`, `verify`) für den amtlichen Löschbericht `ctx-erasure-report/1`. |
| `anti_tamper` | Schutz vor unbemerkter Dateimanipulation im Speicherbereich. |

## 4. Kritische Invarianten
1. **ZeroizeOnDrop:** Sämtliche Zwischenschlüssel, Hashes und sensible Byte-Arrays verwenden `Zeroizing<T>` und werden bei Scope-Verlassen im Speicher genullt.
2. **Injektivität von `blind_tag`:** Durch explizite Längenpräfixe gilt strikt: `blind_tag(l, [a, b]) ≠ blind_tag(l, [ab])`.
3. **Nonce-Disziplin:** Wiederverwendung von Nonces unter demselben Schlüssel ist kryptografisch und strukturell ausgeschlossen.
4. **Kein Staging alter Features:** Veraltete Module (`deletion_proof`, `ed25519_proof`, `kv_cipher`, `kv_segment`, `kv_shredding`, `revocation_log`) sind restlos gelöscht; Deklarationen in `lib.rs` (Z. 47–54, 64–79) werden getilgt.

## 5. Crate-Arbeitspakete zum SOLL-Zustand (PLAN.md)

- **[ ] WP-1.2: Veraltete Krypto-Deklarationen & Features bereinigen**
  - Streichen der toten Modul-Deklarationen in `lib.rs`.
  - Entfernen der Features `kv-encryption`, `cloud-egress-guard`, `test-utils` und unnötiger Abhängigkeiten (`lru`, `regex`, `bincode`, `ahash`).
- **[ ] WP-2.5: Sicherheitsdefaults umsetzen (D-05, D-06)**
  - Argon2id Mindestgrenze ($m \ge 64\,\text{MiB}$) erzwingen.
  - Ersatz von `KeyManager` durch das deterministische `KeySet`.
- **[ ] WP-3.2: `blind_tag` und Envelope-Codec implementieren**
  - Bereitstellung von `blind_tag` für `DocTag`, `CollTag`, `TenantTag` und `TermTag`.
- **[ ] WP-5.2: Berichtssignatur & Hashkette fertigstellen (D-13)**
  - Ed25519-Signaturerzeugung und -Verifikation für Löschberichte v1.

## 6. Verifikation & Benchmarks
- **T-KDF-1:** Ableitungstests mit variierenden Argon2id-Parametern.
- **T-ENC-1 / T-ENC-2:** Known-Answer-Tests gegen offizielle RFC-Testvektoren (AES-GCM-SIV, HKDF, BLAKE3, Ed25519, Argon2id).
- **T-TAG-1:** Injektivitäts- und Kollisionstests für `blind_tag`.
- **B-CRYPTO:** Durchsatzmessungen für `seal`/`open` (1 KiB und 64 KiB) sowie `blind_tag`.
