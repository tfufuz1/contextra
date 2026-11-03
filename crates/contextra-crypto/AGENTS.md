# AGENTS – contextra-crypto

Abgeleitet aus [`docs/SPEC.md`](../../docs/SPEC.md) (8.6, 10.1 bis 10.5, Anhang C und F). Bei Widerspruch gilt die SPEC (D-98). Gemeinsame Regeln: [`AGENTS.md`](../../AGENTS.md).

**Zweck:** Kryptoprimitive: AEAD, Argon2id, HKDF, HMAC, Ed25519, Tags.

**Ring und Abhängigkeiten:** Ring 1. Erlaubt: `types`, `ports`.

## Fachprofil (Arbeitsrolle)

Angewandter Kryptograf (Schlüsselhierarchien, AEAD-Protokolldesign); implementiert nur Standardprimitive. Dieses Profil beschreibt Arbeitsweise und Fachwissen und begründet keine eigene Anforderung.

- HKDF-Schlüsselbaum mit strikter Domänentrennung (Labels, Kontextbindung); Ableitungs- gegenüber Wrapping-Modellen und ihre Wirkung auf die Vernichtbarkeit: Was ist nach der Vernichtung eines Slots nicht mehr herleitbar?
- AEAD: Nonce-Strategie, Wiederverwendungsgrenzen, AAD-Bindung an Ort und Version; Argon2id-Parameter; Konstantzeit-Vergleiche; Ed25519 mit kanonischer Nachrichtenform (strikte Verifikation ist Arbeitshinweis, nicht SPEC-Text).
- Seitenkanäle und Speicherhygiene: `Zeroize`, keine Schlüssel in Logs, Debug-Ausgaben oder Fehlertexten.

## Normative Anker

- **Algorithmensatz v1 (D-80):** AES-256-GCM-SIV (Records, Keyslots, Block-AEAD; `alg_id = 1`), HMAC-SHA256, HKDF-SHA256, BLAKE3 keyed, Argon2id, Ed25519. `alg_id = 2` (ChaCha20-Poly1305) ist reserviert und nicht implementiert.
- **Argon2id (D-06, D-68):** Passphrase nur über Argon2id. Grenzen werden **vor** Start der Ableitung geprüft: `m` ∈ [64 MiB, 4 GiB], `t` ∈ [3, 16], `p` ∈ [1, 16]; Verstoß ist `Corrupt` (Header) bzw. `Config` (Builder). Standard `m = 128 MiB, t = 3, p = 4`. HKDF nur für Subkeys. Der Header wird vor der Passphrasenprüfung gelesen (CWE-770, FM-16, T-KDF-2).
- **Keyslots (D-52):** `K_master` zufällig (OS-CSPRNG), je Slot gewickelt unter `KEK = Argon2id(passphrase, slot_salt, params)`. Die Wrap-AAD bindet `store_id`, `slot_id`, `anchor_id`, `kdf_id`, `m`, `t`, `p`, `slot_salt` (FM-17, T-KEYSLOT-4).
- **Hierarchie (10.1, D-79):** `K_enc`, `K_idx`, `K_unit`, `K_sign`, `K_wal`, `K_commit(epoch)`, `K_ledger`; `DEK = HKDF(K_unit, salt = SHA-256("ctx/dek-salt/v2" ‖ salt_T ‖ salt_C ‖ salt_S ‖ salt_D), info = "ctx/dek/v2" ‖ doc_tag)`. Jedes Label steht in Anhang C. `K_sign` wird nur abgeleitet und nie gespeichert.
- **Nonces und AAD (D-60, 10.3):** Record-Envelope und Keyslot-Wrap 96 Bit aus dem OS-CSPRNG; Block-AEAD `0x00000000 ‖ u64(Blockzähler)` je Dateischlüssel, höchstens 2³² Blöcke je Datei; AAD bindet Dateityp, `file_id`, Blockindex (bei Appendlogs den Vorgänger-MAC).
- **Dateischlüssel-Maske (D-85):** `HKDF-Expand(K_enc, "ctx/fkmask/v1" ‖ file_id, 32)`.
- Alle Schlüsseltypen `ZeroizeOnDrop`. Falsche Passphrase: `Auth` ohne Hinweis, keine Datei wird verändert (FM-10).
- Jeder Header trägt `format_version` und `alg_ids` (D-42, T-FMT-1).

## Entscheidungshilfen

- Neue Primitive oder Laufzeit-Algorithmenwahl? Nein: keine selbstgebauten Primitive, keine Algorithmen-Agilität per Schalter (D-05, P5).
- Neues Label oder Format? Erst Anhang C und F in der SPEC ändern, dann Code, dann Testvektoren.
- Die Mindestkosten für Argon2 in `kdf` gegen D-06 prüfen (F-11 nannte eine zu niedrige Untergrenze und keine Obergrenze).

## Verboten

Feature-Schalter für Verschlüsselung oder Integrität (`legacy-wal-key`, `kv-encryption`, `cloud-egress-guard`; D-05, D-84), `allow_legacy_integrity_key_fallback`, Passphrasen-Ableitung über HKDF, Klartextmodus, `alg_id = 2`, Schlüssel in `Debug`/Logs, nicht zurückgesetzte Schlüssel.

## Abnahme

`T-KDF-1`, `T-KDF-2` (Fuzz), `T-KEYSLOT-1` bis `T-KEYSLOT-4`, `T-NONCE-1`, `T-ENC-1` bis `T-ENC-3`, `T-FMT-1`; bekannte Testvektoren; Negativtests (falsche AAD, Replay, Bit-Flip). Abdeckung Zeilen ≥ 90 % (15.1). Hintergrund zur Schlüsselreichweite: `KEY_COVERAGE_ANALYSIS.md` im Crate.

## Arbeitspakete

WP-3.2 (D-06, D-52, D-60, D-80, `K_sign`), Schnittstellenteil für WP-4.1 (Tresor-Ableitungen). Statusquelle: `.agents/jules/tasks/BACKLOG.md`.

## Prüfbefehle

```bash
cargo test -p contextra-crypto --locked
cargo test -p contextra-crypto --locked --test wp32_spec_tests
cargo test -p contextra-arch-tests --locked
```
