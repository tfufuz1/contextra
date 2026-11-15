# Extraktion fuer `acta-crypto`

**Aufgabe:** AEAD, HKDF, HMAC, Ed25519, Sealed<P>, Schluesseltypen, HashChain, Padding  
**Level:** L1 · **Budget:** 4200 LOC · **Spezifikation:** docs/SPEC.md Kap. 4.1, 4.2, 4.5  
**Interne Abhaengigkeiten (Soll):** acta-base  
**Umfang des Archivs:** 26 Dateien, 5690 Zeilen (inkl. Tests)  

## Herkunft und Umbauauftrag

| Altquelle (Crate) | Umbauauftrag |
|---|---|
| `crypto` | AEAD/HKDF/Argon2id/HashChain (Kap. 4.1, 4.5); auf XChaCha20-Poly1305 + Nonce intern umstellen |
| `erasure` | Umschlag/DEK-Ableitung (4.2) und Ed25519 (4.1) |

## Pruefhinweise (automatisch ermittelt)

| Befund | Treffer | Folge |
|---|---:|---|
| float | 1 | f32/f64 (Z7: Festkomma in Ranking-Pfaden) |
| serde | 10 | serde/bincode (G-NOSERDE-DISK: Plattenformate von Hand kodieren) |
| async | 8 | tokio/async (G-NOTOKIO: Kern synchron) |
| unsafe | 4 | unsafe (G-UNSAFE: nur acta-sys) |
| zeit_zufall | 13 | Zeit/Zufall direkt (G3: nur ueber Ports Clock/CryptoRng) |
| panic | 294 | unwrap/expect/panic (G-LINT) |
| std_fs | 1 | std::fs direkt (G-FS: nur acta-fs) |
| hub | 4 | Hub-Import ex types/ports (HR-2/HR-3) |
| alt_spec | 7 | Alt-Referenzen (D-/ER-/INV-Kennungen): durch SPEC-Verweise ersetzen |

**Externe Abhaengigkeiten der Altquelle ausserhalb der Whitelist (3.8):** `aes-gcm-siv`, `bincode`, `blake3`, `rand` — ersetzen oder per ADR aufnehmen.

## Dateien

| Datei (relativ zu `archive/acta-crypto/`) | LOC | Befunde |
|---|---:|---|
| `crypto/benches/crypto_benchmarks.rs` | 142 | panic=13, hub=1 |
| `crypto/fuzz/fuzz_targets/wal_hmac_chain_verify_fuzz.rs` | 159 |  |
| `crypto/src/aead.rs` | 208 | zeit_zufall=2, panic=3 |
| `crypto/src/anti_tamper.rs` | 211 | unsafe=1, alt_spec=4 |
| `crypto/src/audit_chain.rs` | 375 | serde=2 |
| `crypto/src/contract.rs` | 155 | panic=4 |
| `crypto/src/crypto.rs` | 1042 | async=4, unsafe=3, zeit_zufall=7, panic=60 |
| `crypto/src/dek.rs` | 171 | panic=3, alt_spec=2 |
| `crypto/src/error.rs` | 97 |  |
| `crypto/src/kdf.rs` | 320 | zeit_zufall=2 |
| `crypto/src/lib.rs` | 95 |  |
| `crypto/src/secret.rs` | 163 | serde=6 |
| `crypto/tests/anti_tamper_matrix.rs` | 171 | panic=6 |
| `crypto/tests/audit_chain_tests.rs` | 111 | panic=14, hub=1 |
| `crypto/tests/crypto_property_and_invariants.rs` | 103 | panic=5 |
| `crypto/tests/crypto_vectors_extended.rs` | 351 | panic=13 |
| `crypto/tests/golden_corpus.rs` | 267 | serde=2, panic=25, std_fs=1, hub=1 |
| `crypto/tests/kdf_argon2id.rs` | 272 | panic=31 |
| `crypto/tests/key_separation_and_edge_cases.rs` | 189 | panic=37 |
| `crypto/tests/namespace_isolation.rs` | 132 | panic=21 |
| `crypto/tests/nonce_reuse.rs` | 104 | panic=19 |
| `crypto/tests/nonce_stress.rs` | 99 | float=1, async=4, panic=7 |
| `crypto/tests/proptests.rs` | 154 | panic=22, hub=1 |
| `crypto/tests/rfc_vectors.rs` | 157 | panic=11 |
| `erasure/src/crypto_local.rs` | 275 | alt_spec=1 |
| `erasure/src/ed25519_proof.rs` | 167 | zeit_zufall=2 |
