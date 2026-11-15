# Extraktion fuer `acta-crypto`

**Aufgabe:** AEAD, HKDF, HMAC, Ed25519, Sealed<P>, Schluesseltypen, HashChain, Padding  
**Level:** L1 · **Budget:** 4200 LOC · **Spezifikation:** docs/SPEC.md Kap. 4.1, 4.2, 4.5  
**Interne Abhaengigkeiten (Soll):** acta-base  
**Umfang des Archivs:** 28 Dateien, 5955 Zeilen (inkl. Tests)  

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
| `crypto/benches/crypto_benchmarks.rs` | 141 | panic=13 |
| `crypto/fuzz/fuzz_targets/deletion_proof_tamper.rs` | 155 | serde=3 |
| `crypto/fuzz/fuzz_targets/deletion_proof_verify_external_fuzz.rs` | 136 | serde=3 |
| `crypto/fuzz/fuzz_targets/wal_hmac_chain_verify_fuzz.rs` | 158 |  |
| `crypto/src/aead.rs` | 207 | panic=3 |
| `crypto/src/anti_tamper.rs` | 210 | unsafe=1 |
| `crypto/src/audit_chain.rs` | 374 | serde=2 |
| `crypto/src/contract.rs` | 154 | panic=4 |
| `crypto/src/crypto.rs` | 1041 | panic=60, async=4, unsafe=3 |
| `crypto/src/dek.rs` | 170 | panic=3 |
| `crypto/src/error.rs` | 96 |  |
| `crypto/src/kdf.rs` | 319 |  |
| `crypto/src/lib.rs` | 94 |  |
| `crypto/src/secret.rs` | 162 | serde=6 |
| `crypto/tests/anti_tamper_matrix.rs` | 170 | panic=6 |
| `crypto/tests/audit_chain_tests.rs` | 110 | panic=14 |
| `crypto/tests/crypto_property_and_invariants.rs` | 102 | panic=5 |
| `crypto/tests/crypto_vectors_extended.rs` | 350 | panic=13 |
| `crypto/tests/golden_corpus.rs` | 266 | panic=25, serde=2, std_fs=1 |
| `crypto/tests/kdf_argon2id.rs` | 271 | panic=31 |
| `crypto/tests/key_separation_and_edge_cases.rs` | 188 | panic=37 |
| `crypto/tests/namespace_isolation.rs` | 131 | panic=21 |
| `crypto/tests/nonce_reuse.rs` | 103 | panic=19 |
| `crypto/tests/nonce_stress.rs` | 98 | panic=7, async=4, float=1 |
| `crypto/tests/proptests.rs` | 153 | panic=22 |
| `crypto/tests/rfc_vectors.rs` | 156 | panic=11 |
| `erasure/src/crypto_local.rs` | 274 |  |
| `erasure/src/ed25519_proof.rs` | 166 |  |
