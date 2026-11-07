# AGENTS.md — `contextra-crypto`

> **Ring 0** · Klasse `[K]` · Reifegrad `beta` · Unsafe: nein · Rolle: Reiner Kryptografiekern: AEAD, KDF, WAL-HMAC, Anti-Tamper. **Soll:** ohne Lösch-/Nachweislogik.
> Gemeinsame Regeln stehen in der Root-`AGENTS.md`; diese Datei nennt nur Crate-Spezifisches. Stand 2026-11-07, statisch gelesen, **nicht kompiliert**; Zahlen sind grep-Heuristiken.

## 1. Baukasten-Profil

| Aspekt | Wert |
|---|---|
| Standalone-Build | `cargo build -p contextra-crypto --no-default-features --locked` |
| Contextra-Abhängigkeiten **Ist** (Cargo) | `ports`, `types` |
| Contextra-Abhängigkeiten **Soll** (PLAN §4) | `ports`, `types` |
| Features | `kv-encryption`, `cloud-egress-guard`, `test-utils` (dev-Selbstreferenz mit `test-utils`) |
| Lints | `#![forbid(unsafe_code)]` |

## 2. Abhängigkeiten & Schulden

- Kanten Ist = Soll, **inhaltlich aber nicht entkoppelt**: das Crate enthält noch die Löschdomäne (88 `DeletionProof`-, 31 Shredding-/Revocation-Treffer).
- `KmsProvider` liegt noch in `wal_crypto.rs:32` (Soll: `ports`).
- Konsumenten: `store`, `vector` (nur `wal_crypto`), `engine`, `erasure`, `kvcache`, `infer-candle`, `mcp`.

## 3. Modul-Karte (am Code verifiziert)

| Pfad | Verantwortung |
|---|---|
| `aead.rs`, `kdf.rs`, `secret.rs`, `dek.rs`, `crypto.rs`, `error.rs` | **Verbleibt:** AEAD, Argon2id/HKDF, `SecretKey` (zeroize), `KeyManager`, `derive_dek` |
| `wal_crypto.rs`, `wal_completeness.rs`, `anti_tamper.rs`, `audit_chain.rs` | **Verbleibt:** WAL-HMAC-Kette, Vollständigkeit, Manipulationserkennung, HMAC-Audit-Kette |
| `deletion_proof/` (6 Dateien), `ed25519_proof.rs`, `kv_shredding.rs`, `revocation_log.rs` | **Wandert nach `contextra-erasure` (PLAN 3.1)** |
| `kv_cipher.rs`, `kv_segment/` | KV-Cache-Verschlüsselung; PLAN nennt sie nicht, Konsument `kvcache`: Zuordnung per ADR |

## 4. Verträge & Invarianten

1. **Nie eigene Primitive bauen (EP12).** Nur geprüfte Bibliotheken (`aes-gcm-siv`, `argon2`, `hkdf`, `hmac`, `ed25519-dalek`, `blake3`); Konstantzeit-Vergleich über `subtle`.
2. **KDF (INV-KDF-1):** Argon2id mit zufälligem, persistiertem Salt (`KdfHeader`), m ≥ 64 MiB, t ≥ 3. Kein HKDF direkt auf Passphrase, kein fester Salt.
3. **Schlüssel:** jeder Schlüssel hat genau einen Zweck und ein eigenes HKDF-Label (INV-KEYPURPOSE-1, `governance/key_purposes.toml`). Schlüsselmaterial implementiert `Zeroize`/`ZeroizeOnDrop`; keine API gibt es im Klartext zurück (INV-SCAN-1).
4. **I/O-frei.** Persistenz nur über Ports; keine `std::fs`.
5. **Zufall:** `rand::thread_rng` nur für Schlüssel/Salt/Nonce (Ausnahme ADR-098, `determinism-allow.toml`).
6. **Verbot für Neucode:** keine neuen `DeletionProof`-, Ed25519-Bericht-, Shredding- oder Revocation-Typen hier (Root-§5).

## 5. Ports

**Implementiert:** `KmsProvider` (lokal). **Konsumiert:** `ports` (u. a. `Clock`, `Rng`).

## 6. Nebenläufigkeit & Performance

- Zustandslos bis auf `KeyManager`; Locks (`parking_lot`) kurz, nie über Fremdaufrufe.
- Hot Path: AEAD pro Block. Benchmarks: `benches/crypto_benchmarks`; `deletion_proof_latency_bench` ist Erasure-Last und wandert mit.

## 7. Anti-Patterns

- Konstanten-Salt oder Passphrase→Schlüssel ohne Argon2id.
- Nonce-Wiederverwendung (Nonce-Quelle nur aus CSPRNG/Zähler mit Test).
- `==` auf MACs/Tags.
- Neue Berichts-/Beweistypen in `crypto` ablegen.

## 8. Tests & Messung

```bash
cargo test -p contextra-crypto --locked
cargo clippy -p contextra-crypto --all-targets -- -D warnings
cargo test -p contextra-crypto --locked --features test-utils
RUSTFLAGS="--cfg loom" cargo test -p contextra-crypto --locked
cargo mutants --config mutants-guarantees.toml -p contextra-crypto
```

- Testvektoren (RFC/KAT), Property-Tests (Roundtrip, Manipulation ⇒ Fehler), Fuzzing (WAL-Replay/Proof-Deserialisierung, Ziel `fuzz/`), `loom` für Zeroize-/Zugriffspfade.
- `mutants-guarantees.toml` verweist noch auf `kv_shredding.rs`/`deletion_proof.rs` (G2): nach Verlagerung auf `contextra-erasure` umstellen.

## 9. Offene Arbeit

| WP | Inhalt |
|---|---|
| 3.1 | `deletion_proof/`, `ed25519_proof.rs`, `kv_shredding.rs`, `revocation_log.rs` nach `contextra-erasure`; danach Crate-README, Budget 3.200 Zeilen |
| 2.x | `KmsProvider` nach `ports` |
| 5.3 | `vector → crypto`-Kante (WAL-HMAC) lösen |
