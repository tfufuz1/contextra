# contextra-crypto

Kryptografische Primitive, Schlüsselverwaltung, AEAD-Verschlüsselung, blinde Tags und Signaturen.

- **Ring:** Ring 1 (Kryptografie & Nachweis-Primitive)
- **Normativ:** [docs/SPEC.md §8.6](../../docs/SPEC.md)
- **Arbeitsanweisungen:** [AGENTS.md](AGENTS.md)

## Rolle & Zweck
`contextra-crypto` kapselt sämtliche kryptografischen Operationen in einer strikt auditierten Typfamilie:
- `kdf`: Argon2id KDF ($m \ge 64\,\text{MiB}, t \ge 3, p \ge 1$, `derive_master`).
- `keys`: `KeySet` (deterministische Ableitung aller Teilschlüssel via HKDF aus $K_\text{master}$).
- `aead`: AES-256-GCM-SIV (`seal`, `open`), 96-Bit Nonce aus CSPRNG (`OsRng`).
- `envelope`: Record-Envelope-Sealing (SPEC §9.3).
- `tag`: `blind_tag` via Keyed-BLAKE3 mit Längenpräfixen gegen Mehrdeutigkeiten.
- `wal_crypto`: HMAC-SHA256-Verkettung für WAL-Frames.
- `sign`: Ed25519-Signatur und Verifikation für Löschberichte `ctx-erasure-report/1`.

## Invarianten
- `#![forbid(unsafe_code)]`
- Kein Schlüsselmaterial in `Clone`, `Debug`, `Display` oder `Serialize`. Alle Puffer in `Zeroizing<T>`.
- Kein HKDF für Passphrasen (B-15) — ausschließlich Argon2id!
- Injektivitätsgarantie für `blind_tag`.
