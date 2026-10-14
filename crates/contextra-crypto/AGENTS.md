# AGENTS.md — contextra-crypto
> Ring 0 · stable · Quelle: capabilities.toml · Spec: K.8, III.4, IX.4, L.1

## 1. Zweck
Encryption-at-Rest (AES-256-GCM-SIV), HKDF-Schlüsselableitung, WAL-HMAC-Integritätsketten, Zeroize-Speicherhygiene, Anti-Tamper Schutz und DSGVO-konforme kryptographische Löschbeweise (`DeletionProof`). Der Crate erzwingt `#![forbid(unsafe_code)]` im Produktionscode.

## 2. Modul-Karte
| Datei/Verzeichnis | Verantwortung |
|---|---|
| `src/lib.rs` | Modul-Deklarationen und Crate-Einstiegspunkt (`#![forbid(unsafe_code)]`) |
| `src/crypto.rs` | `KeyManager` — HKDF Subkey Derivation, AES-256-GCM-SIV Ver-/Entschlüsselung |
| `src/deletion_proof.rs` | `DeletionProof` — Kryptographische Löschnachweise und `hash_deleted_keys_length_prefixed` |
| `src/deletion_proof_typestate.rs` | Typestate-Pattern zur compile-zeitlichen Abdeckungsprüfung aller 7 Deletion-Layer |
| `src/ed25519_proof.rs` | Ed25519-Signierung und Verifikation von Version-3 Löschbeweisen |
| `src/anti_tamper.rs` | `VolatileEncryptionKey` — Speichersicherer Schlüsselschutz mit Zeroize-on-Drop |
| `src/audit_chain.rs` | Audit-Chain Verifikation und fälschungssichere Log-Verkettung |
| `src/error.rs` | `CryptoError` — Crate-spezifische Fehler-Enum |
| `src/kdf.rs` | HKDF-SHA256 Schlüsselableitungs-Mechanismen und Salt-Verwaltung |
| `src/kv_cipher.rs` | KV-Segment Verschlüsselungs-Cipher und Modell-Fingerprinting |
| `src/kv_segment/` | Mandanten-isolierte KV-Cache-Sicherheit (`store.rs`, `segment.rs`, `eviction_worker.rs`) |
| `src/kv_shredding.rs` | Crypto-Shredding per-Group Subkey Derivation und O(1) Key-Revokation (Spec L.1) |
| `src/revocation_log.rs` | Revokations-Log für entzogene Schlüssel und Zertifikate |
| `src/wal_completeness.rs` | I/O-freie, konstanter-Zeit Verifikation der WAL-Kettenvollständigkeit |
| `src/wal_crypto.rs` | `WalHmac`, `IntegrityVerifier`, `EncryptedWal` — HMAC-Chaining Protokoll |

## 3. Invarianten
- `INV-DELETION-1`: `DeletionProof` darf erst erzeugt werden, wenn alle deklarierten Layer physisch bereinigt wurden.
- `INV-CRYPTO-DUPLICATE-1`: Genau EINE Definition von `hash_deleted_keys_length_prefixed` in `deletion_proof.rs`.
- `INV-WAL-TRUNCATION-1`: HMAC-Chaining verhindert WAL-Truncation und Replay-Attacken.
- `INV-P28-CSPRNG`: Ausdrückliche P28-Ausnahme für `rand::thread_rng()` / `OsRng` bei der Schlüssel- und Salt-Generierung.

## 4. Verboten / Anti-Patterns
- Plaintext-Vektoren für Schlüssel verwenden (Stets `zeroize::Zeroizing` oder `VolatileEncryptionKey` nutzen).
- Hartcodierte kryptographische Schlüssel im Code ablegen.
- HMAC-Fehler ignorieren statt als `ContextraError::WalCorruption` / `CryptoError` abzufangen.

## 5. Nebenläufigkeit, Async- und Lock-Regeln
- Kryptographische Operationen sind rein synchron und CPU-bound.
- Thread-safe `KeyManager` und `KeyRegistry` nutzen interne Leser/Schreiber-Sperren ohne Deadlock-Gefahren.

## 6. Verifikation

```bash
cargo xtask crate-context contextra-crypto
cargo xtask doctrine-scan --crate contextra-crypto
cargo xtask determinism-check
cargo xtask check-unsafe-islands
cargo xtask check-ring0-async-purity
cargo test -p contextra-crypto --locked
cargo test -p contextra-crypto --test deletion_proof_v4_tamper_matrix
```

## 7. Bekannte Lücken / SOLL (EP2 & Phase 5 Roadmap)

- **FINDING-CRYPTO-01 (KmsProvider-Verdrahtung)**: Aktuell existiert ein Mock-Fallback; vollständige Anbindung an `contextra-ports::KmsProvider` in Phase 1a.
- **FEHLER-A1 (Lock-Free Revocation)**: Migration von `RwLock<BitVec>` auf `AtomicU64[16]` für wait-free O(1) Key-Revokation im Hotpath.
- **FEHLER-D1 / D2 (DeletionProof v4 Konsolidierung)**: Entfernung von `mandatory_ids`, strikte Durchsetzung von `REQUIRED_LAYER_BITS = 0x7F` (7-Layer Pflicht).
- **Crate-Split (Phase 5)**: Modularisierung in `contextra-crypto-core`, `contextra-erasure-evidence`, `contextra-proof-verify` und `contextra-avv-generator`.
- **Historische Deklaration**: Code erzwingt strikt `#![forbid(unsafe_code)]` und `capabilities.toml` führt `unsafe_island = false`.
