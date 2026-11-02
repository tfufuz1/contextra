# `contextra-crypto` – Implementierungsspezifikation

|Feld|Wert|
|---|---|
|Zweck-Satz (D-97)|Kryptoprimitive: AEAD, Argon2id, HKDF, HMAC, Ed25519, Tags.|
|Ring|1|
|Erlaubte Workspace-Abhängigkeiten|`types`, `ports` (SPEC 5.3); Verbraucher: `store`, `text`, `vector`, `erasure`, `engine`|
|Stufe / Gates|S0; G-F (Keyslots, HEADER), G-C (Pflichtverschlüsselung, Downgrade-Pfad weg)|
|Arbeitspakete|WP-3.2 (Grenzen, Slot-AAD, Nonce-Schema, Algorithmenliste, `K_sign`), WP-1.2 (Features), WP-4.1/4.3 (Verbraucher)|
|SPEC-Anker|8.6, 9.3, 9.4, 10.1–10.5, D-05, D-06, D-32, D-52, D-60, D-68, D-79, D-80, F-09, F-11, F-21, F-24, Anhang C, F.1–F.3, F.9|

## 1. Ist-Zustand (HEAD `0ede89a26`) [V]

|Punkt|Befund|
|---|---|
|Umfang|3.260 Zeilen `src`: `wal_crypto.rs` (1.163), `crypto.rs` (947), `audit_chain.rs` (414), `kdf.rs` (308), `anti_tamper.rs` (145), `error.rs` (143), `wal_completeness.rs` (75), `lib.rs` (65)|
|AEAD|nur AES-256-GCM-SIV (`aes-gcm-siv`); `KeyManager` mit `VolatileEncryptionKey`, `nonce_prefix: [u8; 4]` und `AtomicU64`-Zähler (F-24). Das ist **nicht** das Schema D-60 (Zähler je Dateischlüssel, Record-Nonce aus OS-CSPRNG).|
|KDF|`kdf.rs`: `MIN_M_COST_KIB = 19456`, `MIN_T_COST = 2`, `DEFAULT_M_COST_KIB = 65536`, `DEFAULT_T_COST = 3`, `DEFAULT_P_COST = 1`; **keine Obergrenzen** (F-11). Soll: m ∈ [64 MiB, 4 GiB], t ∈ [3, 16], p ∈ [1, 16], Standard m = 128 MiB, t = 3, p = 4 (D-06, D-68).|
|WAL-Integrität|`wal_crypto.rs`: `WalHmac`, `IntegrityVerifier` mit `verify_and_update_v3`, `verify_and_update_v2` und `skip_hmac_verify_legacy` (**Verifikation überspringen**, Downgrade-Pfad, widerspricht D-05/P3, F-21); `EncryptedWal::with_kms` und `KmsProvider`/`LocalKeyManagerKms` (KMS-Abstraktion; Anker-Austausch läuft nach D-53 über Ports, nicht hier)|
|Features|`test-utils`, `kv-encryption`, `cloud-egress-guard` (`CloudEgressGuard` in `wal_crypto.rs:143–172`; alle drei zu entfernen bzw. zu ersetzen, D-84); drei `cfg(any(test, feature = "test-utils"))`-Stellen (`anti_tamper.rs:55`, `crypto.rs:415`, `kdf.rs:93`)|
|Audit-Kette|`audit_chain.rs`: BLAKE3-Hashkette mit periodischer Ed25519-Signatur, `EncryptedCommitmentSalt`, `DataClass`, `compute_record_commitment`. Das ist ein **Parallelentwurf** zu Journal, Ledger und Commitments (F.9, D-48: SHA-256-Kette, `K_commit(epoch)`).|
|Sonstiges|`anti_tamper.rs`: `VolatileEncryptionKey` (Zeroize, maskiertes `Debug`, `emergency_wipe`); `wal_completeness.rs`: Prüfung gegen „Checkpoint-Manifest“ (Altkonzept; Soll: `WalFloor`, F.4)|
|Manifest|Abhängigkeiten ohne Verwendung im Quelltext (Treffer außerhalb Kommentaren: 0): `lru`, `regex`, `serde_json`, `ahash`, `parking_lot`, `tracing`, `contextra-ports`; `bincode` und `serde` je 1 Treffer. Selbst-Dev-Abhängigkeit `contextra-crypto = { path = ".", features = ["test-utils"] }` (verboten, AR-14/D-97). Metadatum `ring = "0"` (Soll 1).|
|Kopf|`lib.rs`: Ring 0 und Verbraucherliste mit nicht vorhandenen Crates (`contextra-db`, `-infer-candle`, `-kvcache`, `-mcp`, `-privacy`); Dateikopf-Marken mit Zeitstempel und Sitzungskennung (Q-09)|
|Tests|18 Testdateien (u. a. `rfc_vectors.rs`, `nonce_reuse.rs`, `kdf_argon2id.rs`, `wal_integrity_audit.rs`, `crypto_crossimpl_python.rs`), Fuzz-Ziel `wal_hmac_chain_verify_fuzz`, 1 Benchmark; neun Test-/Bench-Ziele kompilieren nicht [M] (3.0.0, F-02)|

## 2. Verantwortung und Abgrenzung

`crypto` liefert **zustandslose, I/O-freie Primitive und die Schlüsselhierarchie** (10.1). Es kennt weder Dateien noch Tresor, noch Löschung: Der Tresor (`FileVault`) liegt in `erasure` und nutzt die Funktionen hier (Slot-Maske, Slot-MAC, DEK-Ableitung). `store`, `text` und `vector` erhalten Dateischlüssel nur über `FileKeyring` (Port); sie rufen `crypto` für Block-AEAD und Envelopes. Der Algorithmensatz v1 (D-80) ist fest: AES-256-GCM-SIV (`alg_id = 1`), HMAC-SHA256, HKDF-SHA256, BLAKE3 (keyed), SHA-256, Argon2id, Ed25519. `alg_id = 2` (ChaCha20-Poly1305) ist reserviert und nicht implementiert.

**Pflicht, nicht optional (D-05):** Es gibt keinen Klartextmodus, keinen `Option<KeyManager>`, kein Feature und keinen Laufzeitparameter, der Verschlüsselung oder Integrität abschwächt. Einzige Ausnahme der P5-Liste für dieses Crate: Feature `testing` (D-46) ersetzt **nur** den Argon2-Schritt durch direkte Schlüsselübergabe und lebt in `engine`, nicht hier.

## 3. Zielstruktur (Module) [E]

|Modul|Inhalt|SPEC|
|---|---|---|
|`labels`|**eine** Tabelle aller Label-Konstanten (Anhang C); Test vergleicht sie mit der Tabelle in der SPEC|Anhang C|
|`secret`|`SecretKey<const N>` (`ZeroizeOnDrop`, kein `Clone`/`Debug`/`Display`, `ct_eq` über `subtle`), `Secret` (Passphrase); Basis: `anti_tamper::VolatileEncryptionKey`|8.6, 7.1|
|`kdf`|`KdfParams` mit Grenzen, `derive_kek(passphrase, salt, params)`, `hkdf_expand(ikm, salt, info)`|D-06, D-68, D-80|
|`hierarchy`|`KeyHierarchy::from_master(K_master)` → typisierte Schlüssel `EncKey`, `IdxKey`, `UnitKey`, `SignSeed`, `WalKey`, `LedgerKey`, `HdrKey`, `CommitKey(epoch)`; `derive_file_key(kind, file_id)`, `fkmask(file_id)`, `vault_mac_key()`|10.1, D-80|
|`tag`|`tag(label, &TagPreimage) -> [u8; 16]` (BLAKE3 keyed, `K_idx`)|9.3, D-21|
|`aead`|`seal_record`/`open_record` (Envelope v2, F.6), `BlockSealer` (Block-AEAD, Zähler-Nonce, Kopf-Tag), `wrap_master`/`unwrap_master` (Keyslot, F.2)|D-32, D-52, D-60|
|`dek`|`derive_dek(unit: &UnitKey, salts: &AncestorSalts, doc: DocTag) -> Dek` (D-79, Label `ctx/dek-salt/v2`)|D-79, L-3|
|`chain`|SHA-256-Kettenfunktionen für Appendlogs (`chain_next`), HMAC-Kette für den WAL (`wal_mac`), `ledger_mac`|F.4, F.9, 10.4|
|`sign`|Ed25519 aus `SignSeed`: `sign_report`, `verify_report` (Präfix `ctx/report/v1`)|D-13, F.10|
|`header`|reine Codierung von HEADER-Exemplaren (F.2) ohne I/O: `encode`, `decode_checked`, `check16`, `mac`|D-52, F.2, **L-13**|
|`error`|`CryptoError` mit festen Texten (HR-4) → `types::Error`|7.5|

Entfallen: `audit_chain.rs` (Parallelentwurf, ersetzt durch `chain`, `tag`, Commitments in `erasure`), `wal_completeness.rs` (ersetzt durch die `WalFloor`-Prüfung in `store`, F.4), `KmsProvider`/`LocalKeyManagerKms`/`EncryptedWal::with_kms`, `CloudEgressGuard`, `anti_tamper::emergency_wipe` als eigener Begriff (Zeroize genügt).

## 4. Schnittstellen [E]

```rust
// secret + hierarchy: Schlüsseltypen sind getrennt, nicht austauschbar (Verwechslung ist ein Übersetzungsfehler)
pub struct MasterKey(SecretKey<32>);
pub struct KeyHierarchy { /* abgeleitete Schlüssel, ZeroizeOnDrop */ }
impl KeyHierarchy {
    pub fn from_master(m: MasterKey) -> Self;
    pub fn enc(&self) -> &EncKey;        // K_enc  "ctx/enc/v1"
    pub fn idx(&self) -> &IdxKey;        // K_idx  "ctx/idx/v1"
    pub fn unit(&self) -> &UnitKey;      // K_unit "ctx/unit/v1"
    pub fn wal(&self) -> &WalKey;        // K_wal  "ctx/wal/v1"
    pub fn ledger(&self) -> &LedgerKey;  // K_ledger "ctx/ledger/v1"
    pub fn hdr(&self) -> &HdrKey;        // K_hdr  "ctx/hdr/v1"
    pub fn sign_seed(&self) -> SignSeed; // abgeleitet, nie gespeichert (D-80)
    pub fn commit(&self, epoch: u32) -> CommitKey;                   // "ctx/commit/v1" ‖ epoch
    pub fn file_key(&self, kind: FileKind, id: FileId) -> FileKey;   // K_file, abgeleitet (Steuerdateien)
    pub fn fkmask(&self, id: FileId) -> [u8; 32];                    // HKDF-Expand("ctx/fkmask/v1" ‖ file_id)
    pub fn vault_mac(&self) -> VaultMacKey;                          // HKDF(K_enc, "ctx/vault-mac/v1")
}

// kdf (D-06, D-68): Prüfung VOR der Ableitung
pub struct KdfParams { m_kib: u32, t: u8, p: u8 }
impl KdfParams {
    pub const DEFAULT: Self;                                         // m = 131072, t = 3, p = 4
    pub fn new(m_kib: u32, t: u8, p: u8) -> Result<Self, CryptoError>;   // m ∈ [65536, 4194304], t ∈ [3,16], p ∈ [1,16]
}
pub fn derive_kek(pass: &Secret, salt: &[u8; 32], p: &KdfParams) -> Result<SecretKey<32>, CryptoError>;

// Keyslot (F.2): AAD bindet store_id, slot_id, anchor_id, kdf_id, m, t, p, slot_salt
pub fn wrap_master(kek: &SecretKey<32>, aad: &SlotAad, nonce: [u8; 12], m: &MasterKey) -> [u8; 48];
pub fn unwrap_master(kek: &SecretKey<32>, aad: &SlotAad, nonce: [u8; 12], w: &[u8; 48]) -> Result<MasterKey, CryptoError>; // Fehler: Auth, ohne Hinweis auf den Grund

// Record-Envelope v2 (F.6, 9.4)
pub fn seal_record(dek: &Dek, doc: DocTag, key_bytes: &[u8], plain: &[u8], rng: &mut OsCsprng) -> Vec<u8>;
pub fn open_record(dek: &Dek, key_bytes: &[u8], env: &[u8]) -> Result<Zeroizing<Vec<u8>>, CryptoError>; // Auth/Corrupt

// DEK (D-79, L-3): salt = SHA-256("ctx/dek-salt/v2" ‖ salt_T ‖ salt_C ‖ salt_S ‖ salt_D); salt_S = 0³² ohne Betroffenen
pub struct AncestorSalts { pub t: Salt, pub c: Salt, pub s: Option<Salt>, pub d: Salt }
pub fn derive_dek(unit: &UnitKey, a: &AncestorSalts, doc: DocTag) -> Dek;

// Block-AEAD (D-60, F.1): Nonce 0x00000000 ‖ u64(Blockindex); Kopf-Tag mit Präfix 0xFFFFFFFF
pub struct BlockSealer { /* Dateischlüssel, kind, file_id, nächster Index */ }
impl BlockSealer {
    pub fn new(key: FileKey, kind: FileKind, file_id: FileId, prev: Option<[u8; 32]>) -> Self;
    pub fn seal(&mut self, plain: &[u8]) -> Result<Block, CryptoError>;  // Limit nach 2³² Blöcken (neue Datei)
    pub fn header_tag(&self, header48: &[u8; 48]) -> [u8; 16];
}

// Tags (9.3): Eingaben längenpräfigiert (TagPreimage aus types)
pub fn tag(k: &IdxKey, pre: &TagPreimage) -> [u8; 16];

// Ketten (F.4, F.9, 10.4)
pub fn chain_next(prev: &[u8; 32], entry: &[u8]) -> [u8; 32];                         // SHA-256(prev ‖ entry)
pub fn wal_mac(k: &WalKey, prev: &[u8; 32], len: u32, seq: u64, ct: &[u8], tag: &[u8; 16]) -> [u8; 32];
pub fn ledger_mac(k: &LedgerKey, chain: &[u8; 32]) -> [u8; 32];

// Signatur (D-13, F.10)
pub fn sign_report(seed: &SignSeed, canon: &[u8]) -> [u8; 64];
pub fn verify_report(pubkey: &[u8; 32], canon: &[u8], sig: &[u8; 64]) -> Result<(), CryptoError>;
```

`OsCsprng` ist eine dünne Hülle um den OS-Zufall (`rand::rngs::OsRng`). Sie ist die einzige Ausnahme von P7 für Schlüssel-, Salt- und Nonce-Material und wird **nicht** durch den injizierten `Rng`-Port ersetzt (PO-10); Tests ersetzen sie über eine Schnittstelle `NonceSource` in `cfg(test)`.

## 5. Implementierungsvorgaben

|ID|Vorgabe|Quelle|Test|
|---|---|---|---|
|CR-01|Kein `Option<KeyManager>`, kein Klartextpfad, kein Feature `test-utils`/`kv-encryption`/`cloud-egress-guard`/`wal-integrity`/`legacy-wal-key`, kein Laufzeitparameter für Integritäts-Downgrade. `skip_hmac_verify_legacy`, `verify_and_update_v2` und `verify_and_update` (Mehrversionsverifier) entfallen: **ein** WAL-Format (F.4), **eine** Prüfung.|D-05, D-84, P3|T-DEF-1, T-CFG-1|
|CR-02|`KdfParams::new` prüft `m ∈ [65536, 4194304]` KiB, `t ∈ [3, 16]`, `p ∈ [1, 16]` **bevor** Argon2 Speicher anlegt. Verstoß: `Corrupt` bei Header-Herkunft, `Config` bei Builder-Herkunft. Der Aufrufer unterscheidet die Herkunft über zwei Konstruktoren (`from_header`, `from_config`) [E].|D-06, FM-16|T-KDF-1, T-KDF-2 (Fuzz)|
|CR-03|Standard: m = 128 MiB, t = 3, p = 4 (`KdfParams::DEFAULT`). Die Parameter stehen im Keyslot und sind je Slot änderbar.|D-68|T-KDF-1, BM-KDF|
|CR-04|Keyslot-Wrap: AES-256-GCM-SIV, 96-Bit-Nonce aus OS-CSPRNG, AAD = `"ctx/slot/v1" ‖ store_id ‖ slot_id ‖ anchor_id ‖ kdf_id ‖ u32(m) ‖ u8(t) ‖ u8(p) ‖ slot_salt` (F.2). Kopieren in einen fremden Store oder Herabsetzen von m/t/p lässt `unwrap_master` mit `Auth` scheitern.|D-52, FM-17|T-KEYSLOT-4|
|CR-05|Nonce-Schema: Block-AEAD `0x00000000 ‖ u64(Blockindex)` je Dateischlüssel, Kopf-Tag-Präfix `0xFFFFFFFF`; Record-Envelope und Keyslot-Wrap 96 Bit OS-CSPRNG. Höchstens 2³² Blöcke je Datei, danach `Limit` (neue Datei). Das Zufallspräfix je `KeyManager` entfällt.|D-60, F-24, 10.3|T-NONCE-1, T-ENC-3|
|CR-06|AAD des Block-AEAD: `kind ‖ file_id ‖ u64(Blockindex)`, bei Appendlogs zusätzlich der Vorgänger-Hash; Umordnen und Kreuzkopieren scheitern.|F.1, 10.3|T-ENC-2|
|CR-07|DEK nach D-79 mit Salt-Label `ctx/dek-salt/v2` (Auflösung **L-3**: D-79/9.4 sind maßgeblich, 10.1 ist anzugleichen). Ohne Betroffenen `salt_S = 0³²`.|D-79, 9.4|T-VAULT-2, T-ENC-1|
|CR-08|`K_sign` wird nur abgeleitet (`HKDF(K_master, "ctx/sign/v1")`), nie gespeichert; das Verzeichnis `keys/` und ein „verschlüsselter Seed“ existieren nicht.|D-80|T-KDF-2, T-PROOF-1|
|CR-09|Alle Label-Konstanten stehen **einmal** in `labels` und stimmen mit Anhang C überein; ein Test liest die SPEC-Tabelle und vergleicht. Kein Label als Literal an anderer Stelle (Grep).|Anhang C, D-21|Test (Datei 18)|
|CR-10|Schlüsseltypen: `ZeroizeOnDrop`, kein `Clone`, kein `Debug`/`Display` mit Inhalt, kein `Serialize`; Vergleich über `subtle::ConstantTimeEq`. Passphrase `Secret` wie in 7.1. Keine Schlüssel in Fehlertexten.|8.6, HR-4, I-11|T-LOG-1, Review|
|CR-11|Tags: BLAKE3 keyed mit `K_idx`, Eingaben über `TagPreimage` längenpräfigiert; `(a‖bc)` ≠ `(ab‖c)`.|9.3, D-21|T-KEY-1|
|CR-12|Ketten: Manifest, Journal, Gate-Log, Ledger: `chain = SHA-256(chain_prev ‖ Eintrag)` (F.4, F.9); WAL: `mac_n = HMAC-SHA256(K_wal, prev_mac ‖ len ‖ seq ‖ ct ‖ aead_tag)` (F.4); Ledger zusätzlich `ledger_mac`. Die Kettenfunktionen sind rein; Torn-Write-Regel und `WalFloor` liegen in `store`.|10.4, I-7|T-WAL-1, T-WAL-2|
|CR-13|`HEADER`: Prüfsumme `SHA-256(Bytes[0..584))[0..16]` (Korruptionserkennung, keine Authentisierung) und `header_mac` nach Unwrap (F.2); Exemplarwahl A/B nach `generation`.|F.2, D-52|T-HDR-1, T-FMT-1|
|CR-14|Jeder Header trägt `format_version` und `alg_ids = [aead, kdf_or_0, mac, hash]` (v1: `[1, 0, 1, 1]`); unbekannte Werte, insbesondere `alg_id = 2`, ⇒ `Corrupt`.|F.1, D-42, D-80|T-FMT-1|
|CR-15|Manifest ohne ungenutzte Abhängigkeiten (`lru`, `regex`, `serde_json`, `ahash`, `parking_lot`, `tracing`; `bincode`/`serde` nur bei belegter Verwendung); `contextra-ports` nur, wenn ein Port-Typ in der API vorkommt; keine Selbst-Dev-Abhängigkeit.|P8, AR-14, D-97|`cargo tree`, T-ARCH-6|
|CR-16|Keine Panics, kein `unwrap`/`expect`, kein `let _ =` auf `Result`; `ring = "1"`; Kopf nach Q-01 ohne Zeitstempel.|P6, D-91|Clippy, T-ARCH-5|

## 6. Rückbau und Migration

|Position|Maßnahme|WP|Nachweis|
|---|---|---|---|
|Features `test-utils`, `kv-encryption`, `cloud-egress-guard`; `CloudEgressGuard` (`wal_crypto.rs:143–172`)|entfernen; Tests nutzen `NonceSource` (`cfg(test)`) und ein Dev-Crate statt Selbst-Dev-Abhängigkeit|WP-1.2|T-CFG-1, Grep|
|`skip_hmac_verify_legacy`, `verify_and_update_v2`, `verify_and_update`|entfernen (CR-01); `verify_and_update_v3` wird `wal::verify_frame`|WP-3.2|T-DEF-1, Grep `legacy` = 0|
|`KmsProvider`, `LocalKeyManagerKms`, `EncryptedWal::with_kms`|entfernen (Anker-Austausch über `RecordSealer`/`FileKeyring`, D-53)|WP-3.2|Grep|
|`audit_chain.rs` (414 Z.), `EncryptedCommitmentSalt`, `DataClass`, `compute_record_commitment`|entfernen; Commitments und Ketten in `chain`/`tag` und `erasure`|WP-3.2, WP-4.3|T-JRN-3|
|`wal_completeness.rs` (75 Z.)|entfernen; `WalFloor`-Gleichung in `store` (Datei 08)|WP-3.2|T-WAL-2|
|`KeyManager` (Zufallspräfix + `AtomicU64`)|ersetzen durch `BlockSealer`/`seal_record` (CR-05)|WP-3.2|T-NONCE-1|
|`kdf.rs`: `MIN_M_COST_KIB = 19456`, `MIN_T_COST = 2`, `DEFAULT_P_COST = 1`|auf D-06/D-68 anheben; Obergrenzen ergänzen|WP-3.2|T-KDF-1|
|`lib.rs`-Kopf (Ring 0, nicht existierende Verbraucher, „Löschzertifikate“)|nach Q-01 neu; Wortlaut-Lint (6.4)|WP-1.5|T-ARCH-5, T-LINT-1|
|Manifest: ungenutzte Abhängigkeiten, Selbst-Dev-Abhängigkeit, `ring = "0"`|CR-15, CR-16|WP-1.2|`cargo tree`|

## 7. Arbeitspakete

1. WP-3.2 (rot): Known-Answer-Tests aus RFC-Vektoren (`rfc_vectors.rs` bleibt), Fuzz `KdfParams::from_header`, AAD-Bindungstest (Slot in fremden Store), Nonce-Zähler-Überlauf bei 2³², Label-Tabellenvergleich.
2. Grün: `labels`, `secret`, `kdf` mit Grenzen, `hierarchy`, `aead` (Block, Record, Slot), `dek`, `chain`, `sign`.
3. WP-1.2: Features und tote Module entfernen, Manifest bereinigen, Tests auf `NonceSource` umstellen.
4. WP-4.1: `FileVault` in `erasure` konsumiert `fkmask`, `vault_mac`, `derive_dek` (Schnittstelle hier festlegen, bevor `erasure` beginnt).
5. G-F: Keyslot-Zyklus (`add`, `check`, `remove`) gegen Golden-Dateien (`tests/golden/header-v1.bin`).

## 8. Tests und Benchmarks

|ID|Prüft|
|---|---|
|T-KDF-1/2|Grenzen D-06, falsche Passphrase ⇒ `Auth`; Fuzz auf manipulierte Parameter ohne Ressourcenerschöpfung|
|T-KEYSLOT-1..4|Slot hinzufügen/entfernen, Wechsel in O(1), KDF-Wechsel, AAD-Bindung (Kopie/Herabsetzen scheitert)|
|T-HDR-1|A/B-HEADER: Korruption eines Exemplars ohne Totalverlust (Codec-Teil)|
|T-NONCE-1, T-ENC-1..3|Eindeutigkeit je Dateischlüssel, Pflichtverschlüsselung, AAD gegen Umordnen und Kreuzkopie|
|T-FMT-1|Header: Version und `alg_ids`, unbekannt ⇒ `Corrupt`; Golden-Dateien|
|T-KEY-1|Tag-Eingaben längenpräfigiert|
|T-DEF-1, T-CFG-1|kein Klartext-/Downgrade-Pfad, keine Sicherheitsfeatures|
|T-WAL-1/2 (Primitive)|`wal_mac`-Kette, Fortsetzung mit `tail_mac`|
|T-PROOF-1 (Primitive)|Ed25519-Roundtrip; Manipulation jedes Bytes wird erkannt|
|Fuzz|`wal_hmac_chain_verify_fuzz` (bleibt, an `wal::verify_frame` angepasst); neu: Header-Decoder, Envelope-Decoder, Slot-Unwrap|
|BM-KDF, BM-WRITE (Anteil)|Argon2id 0,3–1,5 s; Kosten von Record- und Block-AEAD getrennt (D-32)|
|Mutation (`cargo-mutants`)|`kdf`-Grenzen, `chain`, `dek` (T-MUT-1-Umfeld, 15.1)|

## 9. Abnahme

Definition of Done (Index Abschnitt 8) und: Grep `legacy|skip_hmac|test-utils|kv-encryption|cloud-egress|lru|regex` in `crypto` = 0; alle Test-/Bench-Ziele kompilieren (F-02); Zeilen ≥ 90 % (15.1); `cargo tree -p contextra-crypto` zeigt nur `types` (und `ports`, falls genutzt) als Workspace-Kanten.

## 10. Offene Punkte

- **L-13 (neu): Eigentümer des HEADER-Codecs.** Die SPEC beschreibt HEADER (F.2, D-52) und die Startsequenz (14.1 Nr. 3), nennt aber kein Crate für Codierung und Prüfsummen. Vorschlag **[E]**: reine Bytes-Codierung in `crypto::header` (kein I/O, nur Primitive), das Lesen/Schreiben (A/B, `fsync`, Verzeichnis-`fsync`) in `engine` über `Vfs`; `lab`/`ctx-verify` implementieren den Codec unabhängig (AR-4). Ohne Klärung vor WP-3.2 entsteht der Codec zufällig in `engine`.
- **L-14 (neu): WAL-Versionen.** `IntegrityVerifier` kennt V1/V2/V3. D-05 und P3 verbieten Legacy-Lesepfade; F.4 beschreibt genau ein Frameformat. Entscheidung: V3 entfällt als Versionsbegriff, das Format ist `format_version = 1` des WAL (F.1). Bestehende Daten im Altformat sind nicht Ziel (kein Release am Altformat).
- **L-3** (DEK-Salt-Label, siehe Index Abschnitt 7): Umsetzung gemäß CR-07; SPEC 10.1 ist anzugleichen.
- Die Konstante `max_docs_per_store = 5·10⁷` bestimmt die Tresorgröße (D-79, D-87); die Aufteilung der Slot-MAC-Schlüssel (`K_vm` aus `K_enc`) ist in der SPEC nur im Slotlayout (F.3) genannt, nicht in 10.1. Vorschlag: `vault_mac()` im `hierarchy`-Modul, Eintrag in 10.1 ergänzen (Anhang H).
